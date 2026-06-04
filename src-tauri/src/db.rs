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
    // Single English words ("let", "new", "return", "true", "false", "class", "import")
    // are excluded to avoid false positives — real JS code typically has multiple signals
    // (semicolons, =>, braces, function, const, etc.) that will still be caught.
    let js_signals = ["function ", "const ", "var ", "async ", "await ",
                      "typeof ", "instanceof ", "this.", "null", "undefined",
                      "=>", "===", "!==", "console."];
    for sig in &js_signals {
        if lower.contains(sig) {
            return true;
        }
    }

    // Python
    let py_signals = ["def ", "class ", "import ", "from ", "elif ", "except ",
                      "lambda ", "yield ", "self.", "__init__", "__str__",
                      "if __name__", "print(", "range(", "len("];
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

    // SQL (keep only multi-word or very specific signals to avoid English-word false positives)
    let sql_signals = ["insert into ", "update set ",
                       "delete from ", "create table ", "alter table ", "drop table ",
                       "inner join ", "left join ", "right join ", "select count(",
                       "select distinct ", "select * from ", "select top "];
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
    if trimmed.contains('\n') && trimmed.lines().count() >= 3 {
        let total_lines = trimmed.lines().count();
        let kv_lines = trimmed.lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#') && t.contains(':')
                    && !t.contains("://")
                    && t.len() > t.find(':').unwrap() + 1
            })
            .count();
        // Must be >70% kv lines and at least 3 kv pairs to reduce false positives from notes
        if kv_lines as f64 / total_lines as f64 > 0.7 && kv_lines >= 3 {
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

    // Error messages and stack traces (avoid broad English words like bare "at")
    let error_signals = ["error:", "exception", "stack trace",
                         "traceback", "file \"", "panic:"];
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

    // --- Multi-line indentation heuristic REMOVED ---
    // Caused too many false positives (lists, outlines, formatted text).
    // Code is reliably detected by the 20+ other rules above.
    //

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn check_rules(content: &str) -> Vec<String> {
        let trimmed = content.trim();
        let lower = trimmed.to_lowercase();
        let mut hits = Vec::new();

        // Braces
        let ob = trimmed.matches('{').count();
        let cb = trimmed.matches('}').count();
        if ob > 0 && ob == cb && (trimmed.contains(":") || trimmed.contains(";")) {
            hits.push("braces".into());
        }

        // Semicolons
        let sc = trimmed.lines().filter(|l| { let t=l.trim(); !t.is_empty() && t.ends_with(';') }).count();
        if sc >= 2 { hits.push("semicolons".into()); }

        // Arrow
        if trimmed.contains("=>") && (trimmed.contains("=> ") || trimmed.contains("=>\n")) {
            hits.push("arrow".into());
        }

        // JS keywords
        let js = ["function ", "const ", "var ", "async ", "await ",
                  "typeof ", "instanceof ", "this.", "null", "undefined",
                  "===", "!==", "console."];
        for sig in &js { if lower.contains(sig) { hits.push(format!("js:{}", sig.trim())); } }

        // Python
        let py = ["def ", "class ", "import ", "from ", "elif ", "except ", "lambda ",
                  "yield ", "self.", "__init__", "__str__", "if __name__", "print(", "range(", "len("];
        for sig in &py {
            if lower.contains(sig) {
                let words = word_tokens(&lower);
                if words.iter().any(|w| {
                    let st = sig.trim();
                    *w == st || w.starts_with(st.trim_end_matches(|c: char| c == '(' || c == ' '))
                }) { hits.push(format!("py:{}", sig.trim().trim_end_matches('('))); }
            }
        }

        // Rust
        let rs = ["fn ", "pub ", "impl ", "struct ", "enum ", "trait ", "use ",
                  "mod ", "let mut ", "match ", "unsafe ", "where ",
                  "unwrap(", "expect(", "Some(", "None(", "Ok(", "Err(",
                  "-> ", "::", "#[", "println!"];
        for sig in &rs { if lower.contains(sig) { hits.push(format!("rust:{}", sig.trim())); } }

        // Go
        let go = ["func ", "package ", "defer ", "go ", "chan ", "select {",
                  "interface ", "map[", "nil", "error "];
        for sig in &go { if lower.contains(sig) { hits.push(format!("go:{}", sig.trim())); } }

        // SQL
        let sql = ["insert into ", "update set ", "delete from ", "create table ",
                   "alter table ", "drop table ", "inner join", "left join", "right join",
                   "select count(", "select distinct ", "select * from ", "select top "];
        let upper = trimmed.to_uppercase();
        for sig in &sql {
            let us = sig.to_uppercase().trim().to_string();
            if upper.contains(&us) { hits.push(format!("sql:{}", sig.trim())); }
        }

        // HTML
        let html = ["<div", "<span", "<p>", "<a ", "<img ", "<input", "<button",
                    "<table", "<tr>", "<td>", "<ul>", "<li>", "<html", "<body",
                    "<head", "<style", "<script", "<?xml", "<!doctype"];
        for tag in &html { if lower.contains(tag) { hits.push(format!("html:{}", tag)); } }

        // CSS
        if trimmed.contains('{') && trimmed.contains('}') && trimmed.contains(':') {
            let css = ["color:", "margin:", "padding:", "font-size:",
                       "background:", "display:", "position:", "width:", "height:"];
            for prop in &css { if lower.contains(prop) { hits.push(format!("css:{}", prop)); } }
        }

        // JSON
        let st = trimmed.trim_start();
        if (st.starts_with('{') || st.starts_with('[')) && trimmed.contains("\":\"") && trimmed.ends_with(|c: char| c == '}' || c == ']') {
            hits.push("json".into());
        }

        // YAML
        if trimmed.contains('\n') && trimmed.lines().count() >= 3 {
            let total = trimmed.lines().count();
            let kv = trimmed.lines().filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#') && t.contains(':') && !t.contains("://") && t.len() > t.find(':').unwrap() + 1
            }).count();
            if kv as f64 / total as f64 > 0.7 && kv >= 3 { hits.push("yaml".into()); }
        }

        // Git diffs
        if trimmed.lines().any(|l| l.starts_with("diff --git") || l.starts_with("--- ") || l.starts_with("+++ "))
            || (trimmed.lines().filter(|l| l.starts_with('+') || l.starts_with('-')).count() >= 3
                && !trimmed.lines().any(|l| l.starts_with("--- ") && l.contains(':'))) {
            hits.push("git".into());
        }

        // Stack traces
        let err = ["error:", "exception", "stack trace", "traceback", "file \"", "panic:"];
        for sig in &err { if lower.contains(sig) && trimmed.lines().count() >= 2 { hits.push(format!("err:{}", sig.trim())); } }

        // Shell
        let sh = ["#!/bin/", "#!/usr/bin/", "#!/opt/", "export ", "alias "];
        for sig in &sh { if lower.starts_with(sig) { hits.push(format!("shell:{}", sig.trim())); } }
        if let Some(first) = trimmed.lines().next() { if first.starts_with("#!") { hits.push("shebang".into()); } }

        // Makefile
        if trimmed.starts_with(".PHONY:") || trimmed.contains(":=") {
            if trimmed.contains('\n') && trimmed.lines().count() >= 2 { hits.push("makefile".into()); }
        }

        // Indentation (removed — too many false positives)

        // Sequential operators
        let ops = [" && ", " || ", " + ", " - ", " * ", " / "];
        let oc = ops.iter().filter(|op| trimmed.contains(*op)).count();
        if oc >= 2 { hits.push("operators".into()); }

        hits
    }

    fn make_embedding(mut vals: Vec<f32>) -> Vec<f32> {
        vals.resize(384, 0.0);
        let norm: f32 = vals.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vals.iter_mut() { *v /= norm; }
        }
        vals
    }

    #[test]
    fn test_semantic_search_ranks_by_similarity() {
        let db = Database::new(PathBuf::from(":memory:"), 500).unwrap();

        db.insert_clip("rust programming", ClipType::Text, None, None, Some(&make_embedding(vec![1.0, 0.0, 0.0]))).unwrap();
        db.insert_clip("cooking recipes", ClipType::Text, None, None, Some(&make_embedding(vec![0.0, 1.0, 0.0]))).unwrap();
        db.insert_clip("advanced rust patterns", ClipType::Code, None, None, Some(&make_embedding(vec![0.9, 0.1, 0.0]))).unwrap();
        db.insert_clip("baking bread at home", ClipType::Text, None, None, Some(&make_embedding(vec![0.0, 0.0, 1.0]))).unwrap();

        let query = make_embedding(vec![1.0, 0.0, 0.0]);
        let results = db.semantic_search(&query, 10).unwrap();

        assert_eq!(results.len(), 4, "all 4 clips returned");
        assert!(results[0].content.contains("rust"), "top result should be most relevant to query: got '{}'", results[0].content);
        assert!(results[0].score.unwrap() > 0.99, "exact match should score near 1.0");

        // The top 2 should be rust-related
        let top_contents: Vec<&str> = results.iter().take(2).map(|c| c.content.as_str()).collect();
        let joined = top_contents.join(" ");
        assert!(joined.contains("rust"), "top results should be rust-related: {:?}", top_contents);
    }

    #[test]
    fn debug_email_triggers() {
        let email = "Dear Candidate,

You are invited for an interview at Eulogik. Please report at the given address on the scheduled date and time mentioned in this invitation.

Kindly bring:

* One hard copy of your updated CV/Resume
* One recent passport-size photograph

Please ensure you arrive on time. Kindly do not expect navigation assistance over phone calls. However, if absolutely required, you may contact our base office during working hours at:
+91-755-4078091

Please note:

* This interview process is strictly in-office at Bhopal. No remote/online interviews will be conducted.
* If you are unable to attend the interview in Bhopal, you may simply ignore this message.
* If you need a different schedule, you may request it once through the reschedule request feature on Indeed.
* Candidates shortlisted after the first (technical) round will be contacted further for the HR round.
* If you have already appeared for an interview with Eulogik during the last 30 days, please ignore this message.

Regards,
Team Eulogik";
        let hits = check_rules(email);
        println!("Email triggers: {:?}", hits);
        for hit in &hits {
            println!("  => {}", hit);
        }
        assert!(hits.is_empty(), "Email should trigger NO code rules, got: {:?}", hits);
    }

    #[test]
    fn debug_list_triggers() {
        let list = "Items needed:\n  Milk\n  Bread\n  Eggs\n  Butter\n  Cheese";
        let hits = check_rules(list);
        println!("List triggers: {:?}", hits);
        for hit in &hits {
            println!("  => {}", hit);
        }
        assert!(hits.is_empty(), "List should trigger NO code rules, got: {:?}", hits);
    }

    #[test]
    fn test_insert_without_embedding_still_works() {
        let db = Database::new(PathBuf::from(":memory:"), 500).unwrap();
        let clip = db.insert_clip("just text no embedding", ClipType::Text, None, None, None).unwrap();
        assert_eq!(clip.content, "just text no embedding");
        assert_eq!(db.count().unwrap(), 1);
    }
}
