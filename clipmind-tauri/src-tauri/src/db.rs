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

        Ok(())
    }

    pub fn insert_clip(
        &self,
        content: &str,
        clip_type: ClipType,
        source: Option<&str>,
        thumbnail: Option<&str>,
    ) -> Result<Clip> {
        let now = chrono_now();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO clips (content, type, source, created_at, pinned, thumbnail) VALUES (?, ?, ?, ?, 0, ?)",
            params![content, clip_type_to_str(&clip_type), source, now, thumbnail],
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
    if content.starts_with("http://") || content.starts_with("https://") || content.starts_with("www.") {
        return ClipType::Link;
    }
    let code_signals = [
        "function ", "def ", "const ", "let ", "var ", "class ", "import ", "export ",
        "fn ", "pub fn ", "async ", "await ", "return ", "if (", "for (", "while (",
        "{ }", "{}", "=>", "===", "!==", "==", "::", "->", "#include", "#!/",
    ];
    let lower = content.to_lowercase();
    for sig in &code_signals {
        if lower.contains(&sig.to_lowercase()) {
            return ClipType::Code;
        }
    }
    if content.contains('\n') {
        let non_empty_lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        if non_empty_lines.len() > 2 {
            let indented = non_empty_lines.iter().filter(|l| l.starts_with("  ") || l.starts_with("\t")).count();
            if indented >= 1 {
                return ClipType::Code;
            }
        }
    }
    ClipType::Text
}
