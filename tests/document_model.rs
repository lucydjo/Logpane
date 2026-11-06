use logpane::core::models::{
    truncate_chars, DocumentEnvelope, Note, NoteBlock, MAX_TITLE_CHARS, SCHEMA_VERSION,
};
use uuid::Uuid;

#[test]
fn envelope_is_versioned_and_round_trips() {
    let note = Note::new();
    let envelope = DocumentEnvelope::new(note.blocks.clone());
    assert_eq!(envelope.schema_version, SCHEMA_VERSION);
    let json = envelope.to_json();
    assert!(json.contains("\"schema_version\":1"));
    assert!(json.contains("\"type\":\"text\""));
    let parsed = DocumentEnvelope::from_json(&json).unwrap();
    assert_eq!(parsed.blocks, note.blocks);
}

#[test]
fn images_serialize_as_image_blocks_not_markdown() {
    let mut note = Note::new();
    note.blocks = vec![NoteBlock::image("a".repeat(64))];
    let json = DocumentEnvelope::new(note.blocks.clone()).to_json();
    assert!(json.contains("\"type\":\"image\""));
    assert!(!json.contains("!["));
    assert!(!json.contains("file://"));
}

#[test]
fn title_follows_first_line_until_customized() {
    let mut note = Note::new();
    note.blocks = vec![NoteBlock::text("\n  Comparaison Claude  \nSuite")];
    note.refresh_auto_title();
    assert_eq!(note.title, "Comparaison Claude");

    note.set_custom_title("Titre choisi");
    note.blocks = vec![NoteBlock::text("Nouvelle première ligne")];
    note.refresh_auto_title();
    assert_eq!(note.title, "Titre choisi");
}

#[test]
fn title_is_truncated_on_char_boundary() {
    let long = "é".repeat(MAX_TITLE_CHARS + 50);
    let truncated = truncate_chars(&long, MAX_TITLE_CHARS);
    assert_eq!(truncated.chars().count(), MAX_TITLE_CHARS + 1);
    assert!(truncated.ends_with('…'));
}

#[test]
fn image_split_keeps_order_and_caret_target() {
    let mut note = Note::new();
    let id = note.blocks[0].id();
    note.blocks[0] = NoteBlock::Text {
        id,
        text: "avantaprès".into(),
    };
    let after = note.insert_image_at(id, 5, "d".repeat(64)).unwrap();
    assert_eq!(note.blocks.len(), 3);
    assert_eq!(note.blocks[0].as_text(), Some("avant"));
    assert_eq!(note.blocks[1].asset_id(), Some("d".repeat(64).as_str()));
    assert_eq!(note.blocks[2].id(), after);
    assert_eq!(note.blocks[2].as_text(), Some("après"));
}

#[test]
fn timestamp_insertion_uses_char_index() {
    let mut note = Note::new();
    let id = note.blocks[0].id();
    note.blocks[0] = NoteBlock::Text {
        id,
        text: "éabc".into(),
    };
    let caret = note.insert_text_at(id, 2, "09:23 — ").unwrap();
    assert_eq!(note.blocks[0].as_text(), Some("éa09:23 — bc"));
    assert_eq!(caret, 2 + "09:23 — ".chars().count());
}

#[test]
fn appended_text_block_lands_after_anchor() {
    let mut note = Note::new();
    let anchor = note.blocks[0].id();
    let new_id = note.append_text_block(Some(anchor), "09:23 — ");
    let index = note.index_of(new_id).unwrap();
    assert_eq!(index, 1);
}

#[test]
fn normalize_removes_trailing_empty_blocks() {
    let mut note = Note::new();
    note.blocks.push(NoteBlock::text(""));
    note.blocks.push(NoteBlock::text("   "));
    note.normalize();
    assert_eq!(note.blocks.len(), 1);
}

#[test]
fn asset_ids_are_deduplicated() {
    let mut note = Note::new();
    let hash = "e".repeat(64);
    note.blocks = vec![
        NoteBlock::image(hash.clone()),
        NoteBlock::text("interlude"),
        NoteBlock::image(hash.clone()),
    ];
    assert_eq!(note.asset_ids().len(), 1);
}

#[test]
fn path_references_are_rejected_in_image_blocks() {
    let mut note = Note::new();
    note.blocks = vec![NoteBlock::Image {
        id: Uuid::new_v4(),
        asset_id: "/tmp/capture.png".into(),
        caption: None,
    }];
    assert!(!note.images_are_content_addressed());

    note.blocks = vec![NoteBlock::image("a".repeat(64))];
    assert!(note.images_are_content_addressed());
}
