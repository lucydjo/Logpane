use std::time::Duration;

use image::ImageEncoder;
use logpane::core::assets::{import_bytes, AssetPaths};
use logpane::core::autosave::SaveTracker;
use logpane::core::models::{Note, NoteBlock};
use logpane::storage::sqlite::{
    load_state, open_connection, persist_note, upsert_asset, WriteEvent, WriterHandle,
};

fn png_bytes(size: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(size, size, image::Rgba([20, 120, 200, 255]));
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(img.as_raw(), size, size, image::ExtendedColorType::Rgba8)
        .expect("encode png");
    buffer.into_inner()
}

#[test]
fn writer_reports_an_unopenable_data_root() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("not-a-directory");
    std::fs::write(&file, b"x").unwrap();
    assert!(WriterHandle::spawn(file).is_err());
}

#[test]
fn flush_fails_after_the_writer_is_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let (mut writer, _events) = WriterHandle::spawn(dir.path().to_path_buf()).unwrap();
    writer.shutdown();
    assert!(writer.flush(Duration::from_millis(200)).is_err());
}

#[test]
fn note_round_trip_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let (mut writer, _events) = WriterHandle::spawn(dir.path().to_path_buf()).unwrap();

    let mut note = Note::new();
    note.title = "Test GPT".into();
    note.title_custom = true;
    note.blocks = vec![NoteBlock::text("Première observation")];
    note.revision = 1;
    writer.save_note(note.clone());
    assert!(writer.flush(Duration::from_secs(5)).is_ok());

    let state = load_state(dir.path()).unwrap();
    assert_eq!(state.notes.len(), 1);
    assert_eq!(state.notes[0].title, "Test GPT");
    assert_eq!(state.notes[0].revision, 1);
    assert_eq!(state.notes[0].plain_text(), "Première observation");
    writer.shutdown();
}

#[test]
fn revision_is_monotonic_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let mut conn = open_connection(&logpane::storage::db_path(dir.path())).unwrap();
    let mut note = Note::new();
    note.revision = 1;
    persist_note(&mut conn, &note).unwrap();
    note.revision = 2;
    note.blocks = vec![NoteBlock::text("deuxième")];
    persist_note(&mut conn, &note).unwrap();

    let state = load_state(dir.path()).unwrap();
    assert_eq!(state.notes[0].revision, 2);
}

#[test]
fn image_note_links_assets_and_rejects_unknown_asset() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let mut conn = open_connection(&logpane::storage::db_path(root)).unwrap();

    let record = import_bytes(&AssetPaths::new(root), &png_bytes(24)).unwrap();
    upsert_asset(&conn, &record).unwrap();

    let mut note = Note::new();
    note.blocks = vec![
        NoteBlock::text("capture"),
        NoteBlock::image(record.hash.clone()),
    ];
    note.revision = 1;
    persist_note(&mut conn, &note).unwrap();

    let linked: i64 = conn
        .query_row(
            "SELECT count(*) FROM note_assets WHERE note_id = ?1",
            [note.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(linked, 1);

    let mut broken = Note::new();
    broken.blocks = vec![NoteBlock::image("f".repeat(64))];
    assert!(persist_note(&mut conn, &broken).is_err());
}

#[test]
fn deleting_a_note_cascades_links_but_keeps_assets() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let mut conn = open_connection(&logpane::storage::db_path(root)).unwrap();
    let record = import_bytes(&AssetPaths::new(root), &png_bytes(12)).unwrap();
    upsert_asset(&conn, &record).unwrap();

    let mut note = Note::new();
    note.blocks = vec![NoteBlock::image(record.hash.clone())];
    note.revision = 1;
    persist_note(&mut conn, &note).unwrap();

    conn.execute("DELETE FROM notes WHERE id = ?1", [note.id.to_string()])
        .unwrap();
    let assets: i64 = conn
        .query_row("SELECT count(*) FROM assets", [], |row| row.get(0))
        .unwrap();
    let links: i64 = conn
        .query_row("SELECT count(*) FROM note_assets", [], |row| row.get(0))
        .unwrap();
    assert_eq!(assets, 1);
    assert_eq!(links, 0);
}

#[test]
fn settings_are_persisted_by_the_writer() {
    let dir = tempfile::tempdir().unwrap();
    let (mut writer, _events) = WriterHandle::spawn(dir.path().to_path_buf()).unwrap();
    writer.set_setting("theme_mode", "dark");
    assert!(writer.flush(Duration::from_secs(5)).is_ok());
    let state = load_state(dir.path()).unwrap();
    assert_eq!(
        state.settings.get("theme_mode").map(String::as_str),
        Some("dark")
    );
    writer.shutdown();
}

#[test]
fn malformed_document_is_flagged_read_only_and_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let conn = open_connection(&logpane::storage::db_path(root)).unwrap();
    let note = Note::new();
    conn.execute(
        "INSERT INTO notes (id, title, title_custom, created_at, updated_at, blocks_json, revision)
         VALUES (?1, '', 0, ?2, ?2, '{not json', 3)",
        rusqlite::params![note.id.to_string(), note.created_at.to_rfc3339()],
    )
    .unwrap();

    let state = load_state(root).unwrap();
    assert_eq!(state.notes.len(), 1);
    assert!(state.read_only.contains(&note.id));
    assert!(!state.warnings.is_empty());
    let raw: String = conn
        .query_row("SELECT blocks_json FROM notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(raw, "{not json");
}

/// Replays what the app does when a note is left while its save is still queued: the
/// acknowledgement arrives once another note is active, and must not block that note's saves.
#[test]
fn a_save_acknowledged_after_switching_notes_lets_the_next_note_save() {
    let dir = tempfile::tempdir().unwrap();
    let (writer, events) = WriterHandle::spawn(dir.path().to_path_buf()).unwrap();
    let mut tracker = SaveTracker::new().with_debounce(Duration::ZERO);

    let mut left_behind = Note::new();
    left_behind.revision = 1;
    tracker.begin_save(left_behind.id, left_behind.revision);
    assert!(writer.save_note(left_behind));
    writer.flush(Duration::from_secs(5)).unwrap();

    let now = std::time::Instant::now();
    tracker.mark_dirty(now);
    assert!(!tracker.should_save(now), "the first save is still awaited");

    match events.recv_timeout(Duration::from_secs(5)).unwrap() {
        WriteEvent::Persisted { note_id, revision } => tracker.ack(note_id, revision),
        other => panic!("unexpected event: {other:?}"),
    }
    assert!(tracker.should_save(now), "autosave stayed blocked");
}
