use rusqlite::Connection;

pub fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS clips (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_type TEXT NOT NULL,
            content_text TEXT NOT NULL DEFAULT '',
            content_blob BLOB,
            image_path TEXT,
            pinned INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            last_used_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS snippets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
            content_text,
            content='clips',
            content_rowid='id',
            tokenize='unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
            INSERT INTO clips_fts(rowid, content_text) VALUES (new.id, new.content_text);
        END;

        CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
            INSERT INTO clips_fts(clips_fts, rowid, content_text) VALUES('delete', old.id, old.content_text);
        END;

        CREATE TRIGGER IF NOT EXISTS clips_au AFTER UPDATE ON clips BEGIN
            INSERT INTO clips_fts(clips_fts, rowid, content_text) VALUES('delete', old.id, old.content_text);
            INSERT INTO clips_fts(rowid, content_text) VALUES (new.id, new.content_text);
        END;
        ",
    )
    .map_err(|e| e.to_string())?;

    let _ = conn.execute("ALTER TABLE clips ADD COLUMN thumb_path TEXT", []);
    let _ = conn.execute("ALTER TABLE clips ADD COLUMN source_app TEXT", []);
    let _ = conn.execute("ALTER TABLE clips ADD COLUMN deleted_at TEXT", []);
    let _ = conn.execute("ALTER TABLE clips ADD COLUMN ocr_text TEXT", []);
    Ok(())
}
