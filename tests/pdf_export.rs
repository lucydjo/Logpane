use std::collections::HashMap;

use image::ImageEncoder;
use logpane::core::assets::{import_bytes, AssetPaths};
use logpane::core::models::{Note, NoteBlock};
use logpane::export::{export_note, export_note_bytes};

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(width, height, image::Rgba([60, 110, 140, 255]));
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(
            image.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .expect("encode png");
    buffer.into_inner()
}

fn parse(bytes: &[u8]) -> printpdf::PdfDocument {
    let mut warnings = Vec::new();
    printpdf::PdfDocument::parse(bytes, &printpdf::PdfParseOptions::default(), &mut warnings)
        .expect("parse generated PDF")
}

#[test]
fn exports_text_and_images_with_an_extractable_text_layer() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let asset = import_bytes(&paths, &png(1200, 800)).unwrap();

    let mut assets = HashMap::new();
    assets.insert(asset.hash.clone(), asset.clone());

    let mut note = Note::new().with_title("Latency comparison");
    note.blocks = vec![
        NoteBlock::text("09:15 — Baseline observation, é à ç, on the small model."),
        NoteBlock::image(asset.hash.clone()),
        NoteBlock::text("09:23 — Second run with a longer prompt."),
    ];

    let bytes = export_note_bytes(&note, &assets, dir.path()).unwrap();
    if let Ok(path) = std::env::var("LOGPANE_PDF_TEST_OUT") {
        let _ = std::fs::write(path, &bytes);
    }
    assert!(bytes.starts_with(b"%PDF"));

    let doc = parse(&bytes);
    assert_eq!(doc.pages.len(), 1);
    let text = doc
        .extract_text()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("Latency comparison"), "text was: {text}");
    assert!(text.contains("Baseline observation"), "text was: {text}");
    assert!(text.contains("é à ç"), "accents lost: {text}");
    assert!(text.contains("longer prompt"), "text was: {text}");
    assert!(text.contains("page 1"), "footer missing: {text}");
}

#[test]
fn long_notes_break_across_pages_and_keep_reading_order() {
    let dir = tempfile::tempdir().unwrap();
    let mut note = Note::new().with_title("Long note");
    let mut blocks = Vec::new();
    for index in 0..120 {
        blocks.push(NoteBlock::text(format!(
            "line {index:03} — a monospace paragraph that should wrap and paginate cleanly."
        )));
    }
    note.blocks = blocks;

    let bytes = export_note_bytes(&note, &HashMap::new(), dir.path()).unwrap();
    let doc = parse(&bytes);
    assert!(doc.pages.len() > 1, "expected several pages");
    let text = doc
        .extract_text()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("line 000"));
    assert!(text.contains("line 119"));
}

#[test]
fn missing_images_are_reported_without_failing_the_export() {
    let dir = tempfile::tempdir().unwrap();
    let mut note = Note::new().with_title("Missing image");
    note.blocks = vec![
        NoteBlock::text("The image is gone."),
        NoteBlock::image("a".repeat(64)),
    ];

    let bytes = export_note_bytes(&note, &HashMap::new(), dir.path()).unwrap();
    let text = parse(&bytes)
        .extract_text()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("[image not found]"), "text was: {text}");
}

#[test]
fn writes_a_pdf_file_with_a_safe_default_name() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let asset = import_bytes(&paths, &png(20, 20)).unwrap();
    let mut assets = HashMap::new();
    assets.insert(asset.hash.clone(), asset.clone());

    let mut note = Note::new().with_title("Notes / 2026: test");
    note.blocks = vec![NoteBlock::image(asset.hash.clone())];

    let filename = logpane::export::default_filename(&note);
    assert_eq!(filename, "Notes - 2026- test.pdf");

    let path = dir.path().join(&filename);
    export_note(&note, &assets, dir.path(), &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    assert!(bytes.len() > 1_000);
}

#[test]
fn other_scripts_stay_in_the_pdf_and_emoji_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    let mut note = Note::new().with_title("日本語のノート");
    note.blocks = vec![NoteBlock::text(
        "cjk 日本語 end1\nhangul 한국어 end2\nemoji 😀 end3",
    )];

    let path = dir.path().join("scripts.pdf");
    let replaced = export_note(&note, &HashMap::new(), dir.path(), &path).unwrap();
    assert_eq!(replaced, 1, "only the emoji has no font");

    let bytes = std::fs::read(&path).unwrap();
    let text = parse(&bytes)
        .extract_text()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
    for expected in ["日本語のノート", "日本語", "한국어", "□", "end3"] {
        assert!(text.contains(expected), "{expected} lost: {text}");
    }
}
