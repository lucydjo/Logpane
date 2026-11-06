//! Schema migrations. `PRAGMA user_version` is the only migration cursor.

use anyhow::{bail, Result};
use rusqlite::Connection;

pub const CURRENT_VERSION: i64 = 1;

pub fn apply(conn: &mut Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > CURRENT_VERSION {
        bail!(
            "database schema version {version} is newer than the supported version \
             {CURRENT_VERSION}; upgrade Logpane or restore a backup"
        );
    }
    if version < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(SCHEMA_V1)?;
        tx.pragma_update(None, "user_version", 1)?;
        tx.commit()?;
    }
    Ok(())
}

const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS notes (
    id            TEXT PRIMARY KEY NOT NULL,
    title         TEXT NOT NULL DEFAULT '',
    title_custom  INTEGER NOT NULL DEFAULT 0 CHECK (title_custom IN (0, 1)),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    blocks_json   TEXT NOT NULL,
    revision      INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0)
);

CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes (updated_at DESC);

CREATE TABLE IF NOT EXISTS assets (
    hash         TEXT PRIMARY KEY NOT NULL CHECK (length(hash) = 64),
    rel_path     TEXT NOT NULL,
    thumb_path   TEXT NOT NULL,
    mime         TEXT NOT NULL,
    width        INTEGER NOT NULL CHECK (width > 0),
    height       INTEGER NOT NULL CHECK (height > 0),
    byte_size    INTEGER NOT NULL CHECK (byte_size >= 0),
    imported_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS note_assets (
    note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    asset_hash  TEXT NOT NULL REFERENCES assets(hash) ON DELETE RESTRICT,
    PRIMARY KEY (note_id, asset_hash)
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_a_newer_schema() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        assert!(apply(&mut conn).is_err());
    }

    #[test]
    fn applies_schema_once() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply(&mut conn).unwrap();
        apply(&mut conn).unwrap();
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_VERSION);
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 4);
    }
}
