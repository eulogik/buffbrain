use crate::types::{Clip, ClipType};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Arc;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    max_clips: i64,
}

impl Database {
    pub fn new(db_path: PathBuf, max_clips: i64) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            max_clips,
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS clips (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                type TEXT NOT NULL DEFAULT 'text',
                source TEXT,
                created_at INTEGER NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                thumbnail TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_clips_created_at ON clips(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_clips_pinned ON clips(pinned DESC, created_at DESC);

            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;

        // Ensure columns exist (migration safety)
        let _ = conn.execute("ALTER TABLE clips ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE clips ADD COLUMN thumbnail TEXT", []);
        let _ = conn.execute("ALTER TABLE clips ADD COLUMN embedding BLOB", []);

        Ok(())
    }

    pub fn insert_clip(
        &self,
        content: &str,
        clip_type: ClipType,
        source: Option<&str>,
        thumbnail: Option<&str>,
        embedding: Option<&[f32]>,
    ) -> Result<Clip> {
        let now = chrono_now();
        let conn = self.conn.lock();
        let embedding_blob = embedding.map(embedding_to_blob);
        conn.execute(
            "INSERT INTO clips (content, type, source, created_at, pinned, thumbnail, embedding) VALUES (?, ?, ?, ?, 0, ?, ?)",
            params![content, clip_type_to_str(&clip_type), source, now, thumbnail, embedding_blob],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Clip {
            id,
            content: content.to_string(),
            clip_type,
            source: source.map(String::from),
            created_at: now,
            pinned: false,
            thumbnail: thumbnail.map(String::from),
            score: None,
        })
    }

    pub fn get_clips(&self) -> Result<Vec<Clip>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, content, type, source, created_at, pinned, thumbnail FROM clips ORDER BY pinned DESC, created_at DESC LIMIT ?",
        )?;
        let clips = stmt
            .query_map(params![self.max_clips], |row| {
                let type_str: String = row.get(2)?;
                Ok(Clip {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    clip_type: parse_clip_type(&type_str),
                    source: row.get(3)?,
                    created_at: row.get(4)?,
                    pinned: {
                        let v: i64 = row.get(5)?;
                        v != 0
                    },
                    thumbnail: row.get(6)?,
                    score: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(clips)
    }

    #[allow(dead_code)]
    pub fn find_by_content(&self, content: &str) -> Result<Option<Clip>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, content, type, source, created_at, pinned, thumbnail FROM clips WHERE content = ? LIMIT 1",
        )?;
        let mut rows = stmt.query(params![content])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                clip_type: parse_clip_type(&row.get::<_, String>(2)?),
                source: row.get(3)?,
                created_at: row.get(4)?,
                pinned: { let v: i64 = row.get(5)?; v != 0 },
                thumbnail: row.get(6)?,
                score: None,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn toggle_pin(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE clips SET pinned = CASE WHEN pinned = 0 THEN 1 ELSE 0 END WHERE id = ?",
            params![id],
        )?;
        let pinned: i64 = conn.query_row(
            "SELECT pinned FROM clips WHERE id = ?",
            params![id],
            |row| row.get(0),
        )?;
        Ok(pinned != 0)
    }

    pub fn delete_clip(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM clips WHERE id = ?", params![id])?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM clips WHERE pinned = 0", [])?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64> {
        let conn = self.conn.lock();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get(0))?;
        Ok(n)
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO config (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn semantic_search(&self, query_embedding: &[f32], max_results: usize) -> Result<Vec<Clip>> {
        let conn = self.conn.lock();
        search_semantic(&conn, query_embedding, max_results)
    }
}

pub fn search_semantic(
    conn: &Connection,
    query_embedding: &[f32],
    max_results: usize,
) -> Result<Vec<Clip>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, type, source, created_at, pinned, thumbnail, embedding FROM clips WHERE embedding IS NOT NULL ORDER BY pinned DESC, created_at DESC",
    )?;

    let mut scored: Vec<Clip> = stmt
        .query_map([], |row| {
            let embedding_blob: Vec<u8> = row.get(7)?;
            let emb = embedding_from_blob(&embedding_blob);
            let type_str: String = row.get(2)?;
            Ok((emb, Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                clip_type: parse_clip_type(&type_str),
                source: row.get(3)?,
                created_at: row.get(4)?,
                pinned: { let v: i64 = row.get(5)?; v != 0 },
                thumbnail: row.get(6)?,
                score: None,
            }))
        })?
        .filter_map(|r| r.ok())
        .map(|(emb, mut clip)| {
            let sim = cosine_similarity(query_embedding, &emb);
            clip.score = Some(sim);
            clip
        })
        .collect();

    scored.sort_by(|a, b| {
        let pinned_cmp = (b.pinned as i8).cmp(&(a.pinned as i8));
        if pinned_cmp != std::cmp::Ordering::Equal {
            return pinned_cmp;
        }
        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
    });

    scored.truncate(max_results);
    Ok(scored)
}

fn embedding_to_blob(emb: &[f32]) -> Vec<u8> {
    emb.iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect()
}

fn embedding_from_blob(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_ne_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn clip_type_to_str(t: &ClipType) -> &'static str {
    match t {
        ClipType::Text => "text",
        ClipType::Code => "code",
        ClipType::Link => "link",
        ClipType::Image => "image",
    }
}

fn parse_clip_type(s: &str) -> ClipType {
    match s {
        "code" => ClipType::Code,
        "link" => ClipType::Link,
        "image" => ClipType::Image,
        _ => ClipType::Text,
    }
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn detect_type(content: &str) -> ClipType {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return ClipType::Text;
    }

    // --- Link detection (checked first because URLs often look like code) ---
    if is_link(trimmed) {
        return ClipType::Link;
    }

    // --- Code detection ---
    if is_code(trimmed) {
        return ClipType::Code;
    }

    ClipType::Text
}

fn is_link(content: &str) -> bool {
    // Standard URLs with protocol
    if content.starts_with("http://")
        || content.starts_with("https://")
        || content.starts_with("www.")
    {
        return true;
    }

    // Email addresses
    if let Some(at_pos) = content.find('@') {
        if at_pos > 0 && at_pos < content.len() - 4 {
            let local = &content[..at_pos];
            let domain = &content[at_pos + 1..];
            if domain.contains('.') && !local.contains(' ') && !domain.contains(' ') {
                // Exclude common false positives like "x @ 2x"
                if !domain.chars().any(|c| c.is_whitespace()) && domain.len() >= 3 {
                    return true;
                }
            }
        }
    }

    // Absolute file paths (macOS/Unix)
    if content.starts_with('/') || content.starts_with('~') {
        // Must be longer than just "/" and contain path separators
        if content.len() > 2 && content.contains('/') {
            return true;
        }
    }

    // IP addresses (simple pattern: x.x.x.x optionally with :port)
    if let Some(pos) = content.find(|c: char| c == ':' || c == '/' || c == ' ' || c == '\n') {
        let first_token = &content[..pos];
        if is_ip_address(first_token) {
            return true;
        }
    } else if is_ip_address(content) && content.len() >= 7 {
        return true;
    }

    // Markdown-style links: [text](url)
    if content.starts_with('[') && content.contains("](") && content.ends_with(')') {
        return true;
    }

    // Plain domain with path (e.g., "example.com/resource")
    if !content.contains(' ') && !content.contains('\n') {
        let lower = content.to_lowercase();
        if let Some(dot_pos) = lower.rfind('.') {
            let ext = &lower[dot_pos + 1..];
            let tlds = ["com", "org", "net", "io", "app", "dev", "ai", "edu", "gov",
                        "me", "co", "uk", "de", "jp", "fr", "au", "ca", "us",
                        "xyz", "info", "io", "sh", "fm", "to", "tv", "cc", "gg"];
            if tlds.contains(&ext) {
                // Must have something before the domain
                let before_domain = &lower[..dot_pos];
                if before_domain.len() >= 3 && before_domain.contains('.') {
                    return true;
                }
            }
        }
    }

    false
}

fn is_ip_address(s: &str) -> bool {
    let s = s.trim_end_matches(|c: char| c == ':' || c.is_digit(10));
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| {
        p.parse::<u8>().is_ok() || (p.len() == 1 && p.starts_with('0'))
    })
}

fn is_code(content: &str) -> bool {
    let trimmed = content.trim();

    // --- Strong structural signals ---

    // Matching braces with content inside is a strong code signal
    let open_braces = trimmed.matches('{').count();
    let close_braces = trimmed.matches('}').count();
    if open_braces > 0 && open_braces == close_braces {
        // Check for typical code patterns inside braces
        if trimmed.contains(":") || trimmed.contains(";") {
            return true;
        }
    }

    // Semicolons at line endings (C-style languages)
    let semicolon_lines = trimmed.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && (t.ends_with(';') || t.ends_with(';'))
        })
        .count();
    if semicolon_lines >= 2 {
        return true;
    }

    // Arrow functions / lambda
    if trimmed.contains("=>") && (trimmed.contains("=> ") || trimmed.contains("=>\n")) {
        return true;
    }

    // --- Language-specific keyword signals ---
    // Use word-boundary matching to avoid false positives
    let lower = trimmed.to_lowercase();
    let words = word_tokens(&lower);

    // JavaScript/TypeScript/React
    let js_signals = ["function ", "const ", "let ", "var ", "class ", "import ", "export ",
                      "async ", "await ", "return ", "typeof ", "instanceof ", "new ",
                      "this.", "null", "undefined", "true", "false",
                      "=>", "===", "!==", "console.", "=>"];
    for sig in &js_signals {
        if lower.contains(sig) {
            return true;
        }
    }

    // Python
    let py_signals = ["def ", "class ", "import ", "from ", "elif ", "except ",
                      "lambda ", "yield ", "with ", "as ", "in ", "not ", "or ",
                      "self.", "__init__", "__str__", "if __name__", "print(",
                      "range(", "len(", "import ", "as "];
    for sig in &py_signals {
        if lower.contains(sig) && words.iter().any(|w| {
            let sig_trim = sig.trim();
                    *w == sig_trim || w.starts_with(sig_trim.trim_end_matches(|c: char| c == '(' || c == ' '))
        }) {
            return true;
        }
    }

    // Rust
    let rust_signals = ["fn ", "pub ", "impl ", "struct ", "enum ", "trait ", "use ",
                        "mod ", "let mut ", "match ", "unsafe ", "where ",
                        "unwrap(", "expect(", "Some(", "None(", "Ok(", "Err(",
                        "-> ", "::", "#[", "println!"];
    for sig in &rust_signals {
        if lower.contains(sig) {
            return true;
        }
    }

    // Go
    let go_signals = ["func ", "package ", "defer ", "go ", "chan ", "select {",
                      "interface ", "map[", "nil", "error "];
    for sig in &go_signals {
        if lower.contains(sig) {
            return true;
        }
    }

    // SQL
    let sql_signals = ["select ", "from ", "where ", "insert into ", "update set ",
                       "delete from ", "create table ", "alter table ", "drop table ",
                       "join ", "group by ", "order by ", "having ", "limit ",
                       "inner join ", "left join ", "right join "];
    let upper = trimmed.to_uppercase();
    for sig in &sql_signals {
        let upper_sig = sig.to_uppercase().trim().to_string();
        if upper.contains(&upper_sig) {
            return true;
        }
    }

    // HTML/XML
    if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() > 3 {
        return true;
    }
    let html_tags = ["<div", "<span", "<p>", "<a ", "<img ", "<input", "<button",
                     "<table", "<tr>", "<td>", "<ul>", "<li>", "<html", "<body",
                     "<head", "<style", "<script", "<?xml", "<!doctype"];
    for tag in &html_tags {
        if lower.contains(tag) {
            return true;
        }
    }

    // CSS
    if trimmed.contains('{') && trimmed.contains('}') && trimmed.contains(':') {
        let has_css_props = ["color:", "margin:", "padding:", "font-size:",
                             "background:", "display:", "position:", "width:", "height:"];
        for prop in &has_css_props {
            if lower.contains(prop) {
                return true;
            }
        }
    }

    // JSON detection (starts with { or [ and contains ":")
    let st = trimmed.trim_start();
    if (st.starts_with('{') || st.starts_with('['))
        && trimmed.contains("\":\"")
        && trimmed.ends_with(|c: char| c == '}' || c == ']')
    {
        return true;
    }

    // YAML/TOML (key: value pairs on multiple lines)
    if trimmed.contains('\n') && trimmed.lines().count() >= 2 {
        let kv_lines = trimmed.lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#') && t.contains(':')
                    && !t.contains("://")
                    && t.len() > t.find(':').unwrap() + 1
            })
            .count();
        if kv_lines as f64 / trimmed.lines().count() as f64 > 0.5 && kv_lines >= 2 {
            return true;
        }
    }

    // Git diffs
    if trimmed.lines().any(|l| l.starts_with("diff --git") || l.starts_with("--- ") || l.starts_with("+++ "))
        || (trimmed.lines().filter(|l| l.starts_with('+') || l.starts_with('-')).count() >= 3
            && !trimmed.lines().any(|l| l.starts_with("--- ") && l.contains(':')))
    {
        return true;
    }

    // Error messages and stack traces
    let error_signals = ["error:", "exception", "stack trace", "at ",
                         "traceback", "file \"", "line ", "panic:"];
    for sig in &error_signals {
        if lower.contains(sig) && trimmed.lines().count() >= 2 {
            return true;
        }
    }

    // Shell scripts / commands
    let shell_signals = ["#!/bin/", "#!/usr/bin/", "#!/opt/", "export ", "alias "];
    for sig in &shell_signals {
        if lower.starts_with(sig) {
            return true;
        }
    }
    // Shebang anywhere in first line
    if let Some(first_line) = trimmed.lines().next() {
        if first_line.starts_with("#!") {
            return true;
        }
    }

    // Makefiles and configs
    if trimmed.starts_with(".PHONY:") || trimmed.contains(":=") {
        if trimmed.contains('\n') && trimmed.lines().count() >= 2 {
            return true;
        }
    }

    // --- Multi-line indentation heuristic (improved) ---
    if trimmed.contains('\n') {
        let non_empty: Vec<&str> = trimmed.lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        if non_empty.len() > 2 {
            // Count indented lines
            let indented = non_empty.iter()
                .filter(|l| l.starts_with("  ") || l.starts_with('\t') || l.starts_with("    "))
                .count();
            // Stricter: require at least 40% of non-empty lines to be indented
            if indented as f64 / non_empty.len() as f64 >= 0.4 {
                return true;
            }
            // Also detect Python-style: consistent dedent patterns
            let has_colon_line = non_empty.iter()
                .any(|l| l.trim().ends_with(':') && !l.trim().ends_with("://"));
            if has_colon_line && indented >= 1 {
                return true;
            }
        }
    }

    // Sequential operators on a single line
    let operators = [" && ", " || ", " + ", " - ", " * ", " / "];
    let op_count = operators.iter()
        .filter(|op| trimmed.contains(*op))
        .count();
    if op_count >= 2 {
        return true;
    }

    false
}

fn word_tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .collect()
}
