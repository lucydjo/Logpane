use std::collections::HashMap;

use image::ImageEncoder;
use logpane::core::assets::{import_bytes, AssetPaths};
use logpane::core::models::{Note, NoteBlock};
use logpane::export::{export_markdown, filename_for, to_markdown, ImageLinks};

fn png(seed: u8) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(8, 8, image::Rgba([seed, 110, 140, 255]));
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(image.as_raw(), 8, 8, image::ExtendedColorType::Rgba8)
        .expect("encode png");
    buffer.into_inner()
}

#[test]
fn markdown_keeps_reading_order_and_links_images() {
    let dir = tempfile::tempdir().unwrap();
    let asset = import_bytes(&AssetPaths::new(dir.path()), &png(1)).unwrap();
    let mut assets = HashMap::new();
    assets.insert(asset.hash.clone(), asset.clone());

    let mut note = Note::new().with_title("Latency comparison");
    note.blocks = vec![
        NoteBlock::text("09:15 — baseline #perf\n"),
        NoteBlock::image(asset.hash.clone()),
        NoteBlock::text("09:23 — second run"),
        NoteBlock::image("f".repeat(64)),
        NoteBlock::text(""),
    ];

    let markdown = to_markdown(&note, &assets, &ImageLinks::Folder("shots"));
    let short = &asset.hash[..12];
    assert!(markdown.starts_with("# Latency comparison\n\n<!-- logpane · created "));
    let body = markdown.split("-->\n\n").nth(1).expect("body");
    assert_eq!(
        body,
        format!(
            "09:15 — baseline #perf\n\n![image 1](<shots/{short}.png>)\n\n09:23 — second run\n\n_[image 2: not found]_\n"
        )
    );
}

#[test]
fn an_automatic_title_is_not_repeated() {
    let mut note = Note::new();
    note.blocks = vec![NoteBlock::text("First line\nsecond")];
    note.refresh_auto_title();
    let markdown = to_markdown(&note, &HashMap::new(), &ImageLinks::Folder("x"));
    assert!(!markdown.contains("# First line"));
    assert!(markdown.ends_with("First line\nsecond\n"));
}

#[test]
fn clipboard_links_point_into_the_asset_store() {
    let dir = tempfile::tempdir().unwrap();
    let asset = import_bytes(&AssetPaths::new(dir.path()), &png(2)).unwrap();
    let mut assets = HashMap::new();
    assets.insert(asset.hash.clone(), asset.clone());
    let mut note = Note::new().with_title("t");
    note.blocks = vec![NoteBlock::image(asset.hash.clone())];

    let markdown = to_markdown(&note, &assets, &ImageLinks::Store(dir.path()));
    let expected = asset.absolute_path(dir.path());
    assert!(markdown.contains(&format!("](<{}>)", expected.display())));
    assert!(expected.exists());
}

#[test]
fn export_writes_the_file_and_copies_each_image_once() {
    let store = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let asset = import_bytes(&AssetPaths::new(store.path()), &png(3)).unwrap();
    let mut assets = HashMap::new();
    assets.insert(asset.hash.clone(), asset.clone());
    let mut note = Note::new().with_title("My note");
    note.blocks = vec![
        NoteBlock::text("a"),
        NoteBlock::image(asset.hash.clone()),
        NoteBlock::image(asset.hash.clone()),
    ];

    let path = out.path().join(filename_for(&note, "md"));
    assert_eq!(path.file_name().unwrap(), "My note.md");
    let copied = export_markdown(&note, &assets, store.path(), &path).unwrap();
    assert_eq!(copied, 1);

    let written = std::fs::read_to_string(&path).unwrap();
    let image = out
        .path()
        .join("My note.assets")
        .join(format!("{}.png", &asset.hash[..12]));
    assert!(image.exists());
    assert_eq!(written.matches("](<My note.assets/").count(), 2);
}

#[test]
fn a_note_without_images_creates_no_folder() {
    let out = tempfile::tempdir().unwrap();
    let mut note = Note::new().with_title("plain");
    note.blocks = vec![NoteBlock::text("text")];
    let path = out.path().join("plain.md");
    assert_eq!(
        export_markdown(&note, &HashMap::new(), out.path(), &path).unwrap(),
        0
    );
    assert!(!out.path().join("plain.assets").exists());
}
