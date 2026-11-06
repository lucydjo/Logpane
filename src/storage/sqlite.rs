//! SQLite persistence: one writer thread owns the only write connection.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::core::assets::AssetRecord;
use crate::core::models::{DocumentEnvelope, Note, NoteBlock};
use crate::fsutil;

use super::migrations;

pub const DB_FILE: &str = "logpane.sqlite";

#[derive(Debug, Default)]
pub struct LoadedState {
    pub notes: Vec<Note>,
    pub assets: HashMap<String, AssetRecord>,
    pub settings: HashMap<String, String>,
    pub warnings: Vec<String>,
    /// Notes whose stored JSON could not be trusted; the UI must keep them read-only.
    pub read_only: HashSet<Uuid>,
}

pub fn db_path(root: &Path) -> PathBuf {
    root.join(DB_FILE)
}

pub fn open_connection(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create data directory")?;
    }
    let mut conn = Connection::open(path).context("open sqlite database")?;
    fsutil::restrict_file(path);
    if let Some(parent) = path.parent() {
        fsutil::restrict_dir(parent);
    }
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    migrations::apply(&mut conn)?;
    Ok(conn)
}

pub fn load_state(root: &Path) -> Result<LoadedState> {
    let conn = open_connection(&db_path(root))?;
    let mut state = LoadedState::default();

    let mut stmt = conn.prepare(
        "SELECT id, title, title_custom, created_at, updated_at, blocks_json, revision
         FROM notes ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, i64>(6)?,
        ))
    })?;

    for row in rows {
        let (id, title, title_custom, created_at, updated_at, blocks_json, revision) = row?;
        let Ok(note_id) = Uuid::parse_str(&id) else {
            state
                .warnings
                .push(format!("note skipped: invalid identifier {id}"));
            continue;
        };
        let created_at = parse_time(&created_at).unwrap_or_else(Utc::now);
        let updated_at = parse_time(&updated_at).unwrap_or(created_at);
        let blocks = match DocumentEnvelope::from_json(&blocks_json) {
            Ok(envelope) => envelope.blocks,
            Err(error) => {
                state.warnings.push(format!("note {note_id} : {error}"));
                state.read_only.insert(note_id);
                vec![NoteBlock::text(
                    "⚠ This note's content could not be read. It is kept as-is.",
                )]
            }
        };
        state.notes.push(Note {
            id: note_id,
            title,
            title_custom: title_custom != 0,
            created_at,
            updated_at,
            revision,
            blocks,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT hash, rel_path, thumb_path, mime, width, height, byte_size, imported_at FROM assets",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, String>(7)?,
        ))
    })?;
    for row in rows {
        let (hash, rel_path, thumb_path, mime, width, height, byte_size, imported_at) = row?;
        state.assets.insert(
            hash.clone(),
            AssetRecord {
                hash,
                rel_path,
                thumb_path,
                mime,
                width: width.max(1) as u32,
                height: height.max(1) as u32,
                byte_size: byte_size.max(0) as u64,
                imported_at: parse_time(&imported_at).unwrap_or_else(Utc::now),
            },
        );
    }

    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (key, value) = row?;
        state.settings.insert(key, value);
    }

    Ok(state)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

fn parse_time(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

#[derive(Debug)]
pub enum WriteCmd {
    SaveNote(Note),
    UpsertAsset(AssetRecord),
    DeleteNote(Uuid),
    SetSetting { key: String, value: String },
    Flush(Sender<std::result::Result<(), String>>),
    Shutdown(Sender<()>),
}

#[derive(Debug, Clone)]
pub enum WriteEvent {
    Persisted {
        note_id: Uuid,
        revision: i64,
    },
    Failed {
        note_id: Uuid,
        revision: i64,
        message: String,
    },
    StorageFault {
        message: String,
    },
}

pub struct WriterHandle {
    tx: Sender<WriteCmd>,
    join: Option<JoinHandle<()>>,
}

impl WriterHandle {
    pub fn spawn(root: PathBuf) -> Result<(Self, Receiver<WriteEvent>)> {
        // Open the database before spawning, so a failure is reported to the caller instead of
        // leaving a writer thread that has already exited.
        let mut conn = open_connection(&db_path(&root))?;
        let (tx, rx) = mpsc::channel::<WriteCmd>();
        let (event_tx, event_rx) = mpsc::channel::<WriteEvent>();
        let join = thread::Builder::new()
            .name("logpane-db".into())
            .spawn(move || {
                let mut last_error: Option<String> = None;
                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        WriteCmd::SaveNote(note) => {
                            let note_id = note.id;
                            let revision = note.revision;
                            match persist_with_retry(&mut conn, &note) {
                                Ok(()) => {
                                    let _ =
                                        event_tx.send(WriteEvent::Persisted { note_id, revision });
                                }
                                Err(error) => {
                                    tracing::error!(%error, %note_id, "note save failed");
                                    let message = error.to_string();
                                    last_error = Some(message.clone());
                                    let _ = event_tx.send(WriteEvent::Failed {
                                        note_id,
                                        revision,
                                        message,
                                    });
                                }
                            }
                        }
                        WriteCmd::UpsertAsset(asset) => {
                            if let Err(error) = upsert_asset(&conn, &asset) {
                                tracing::warn!(%error, "asset metadata save failed");
                                let message = error.to_string();
                                last_error = Some(message.clone());
                                let _ = event_tx.send(WriteEvent::StorageFault { message });
                            }
                        }
                        WriteCmd::DeleteNote(id) => {
                            if let Err(error) = conn
                                .execute("DELETE FROM notes WHERE id = ?1", params![id.to_string()])
                            {
                                tracing::warn!(%error, "note delete failed");
                                let message = error.to_string();
                                last_error = Some(message.clone());
                                let _ = event_tx.send(WriteEvent::StorageFault { message });
                            }
                        }
                        WriteCmd::SetSetting { key, value } => {
                            if let Err(error) = conn.execute(
                                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                                params![key, value],
                            ) {
                                tracing::warn!(%error, "setting save failed");
                                let message = error.to_string();
                                last_error = Some(message.clone());
                                let _ = event_tx.send(WriteEvent::StorageFault { message });
                            }
                        }
                        WriteCmd::Flush(ack) => {
                            let _ = ack.send(last_error.take().map_or(Ok(()), Err));
                        }
                        WriteCmd::Shutdown(ack) => {
                            let _ = ack.send(());
                            break;
                        }
                    }
                }
            })
            .context("spawn database writer")?;

        Ok((
            Self {
                tx,
                join: Some(join),
            },
            event_rx,
        ))
    }

    pub fn save_note(&self, note: Note) -> bool {
        self.tx.send(WriteCmd::SaveNote(note)).is_ok()
    }

    pub fn upsert_asset(&self, asset: AssetRecord) -> bool {
        self.tx.send(WriteCmd::UpsertAsset(asset)).is_ok()
    }

    pub fn delete_note(&self, id: Uuid) -> bool {
        self.tx.send(WriteCmd::DeleteNote(id)).is_ok()
    }

    pub fn set_setting(&self, key: impl Into<String>, value: impl Into<String>) -> bool {
        self.tx
            .send(WriteCmd::SetSetting {
                key: key.into(),
                value: value.into(),
            })
            .is_ok()
    }

    /// Wait until every previously queued command has been handled.
    ///
    /// Returns the first error encountered since the previous flush, so the UI cannot claim a
    /// note was saved when the writer rejected it.
    pub fn flush(&self, timeout: Duration) -> std::result::Result<(), String> {
        let (ack_tx, ack_rx) = mpsc::channel();
        if self.tx.send(WriteCmd::Flush(ack_tx)).is_err() {
            return Err("database writer unavailable".into());
        }
        match ack_rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => Err("database writer timed out".into()),
        }
    }

    pub fn shutdown(&mut self) {
        let (ack_tx, ack_rx) = mpsc::channel();
        if self.tx.send(WriteCmd::Shutdown(ack_tx)).is_ok() {
            let _ = ack_rx.recv_timeout(Duration::from_secs(3));
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for WriterHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn persist_with_retry(conn: &mut Connection, note: &Note) -> Result<()> {
    let mut last = None;
    for attempt in 0..3 {
        match persist_note(conn, note) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last = Some(error);
                std::thread::sleep(Duration::from_millis(50 * (attempt + 1)));
            }
        }
    }
    Err(last.unwrap_or_else(|| anyhow::anyhow!("unknown persistence error")))
}

pub fn persist_note(conn: &mut Connection, note: &Note) -> Result<()> {
    if !note.images_are_content_addressed() {
        anyhow::bail!("refusing to persist a note with a non-content-addressed image block");
    }
    let blocks_json = DocumentEnvelope::new(note.blocks.clone()).to_json();
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO notes (id, title, title_custom, created_at, updated_at, blocks_json, revision)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            title_custom = excluded.title_custom,
            updated_at = excluded.updated_at,
            blocks_json = excluded.blocks_json,
            revision = excluded.revision",
        params![
            note.id.to_string(),
            note.title,
            if note.title_custom { 1 } else { 0 },
            note.created_at.to_rfc3339(),
            note.updated_at.to_rfc3339(),
            blocks_json,
            note.revision,
        ],
    )?;

    tx.execute(
        "DELETE FROM note_assets WHERE note_id = ?1",
        params![note.id.to_string()],
    )?;
    for asset_id in note.asset_ids() {
        tx.execute(
            "INSERT INTO note_assets (note_id, asset_hash) VALUES (?1, ?2)",
            params![note.id.to_string(), asset_id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn upsert_asset(conn: &Connection, asset: &AssetRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO assets (hash, rel_path, thumb_path, mime, width, height, byte_size, imported_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(hash) DO NOTHING",
        params![
            asset.hash,
            asset.rel_path,
            asset.thumb_path,
            asset.mime,
            asset.width as i64,
            asset.height as i64,
            asset.byte_size as i64,
            asset.imported_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}
