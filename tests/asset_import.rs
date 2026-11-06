use std::fs;

use image::ImageEncoder;

use logpane::core::assets::{
    import_bytes, is_supported_extension, path_is_valid_image, read_source, sha256_hex, AssetError,
    AssetPaths, AssetWorker, THUMB_MAX_HEIGHT, THUMB_MAX_WIDTH,
};

fn png(width: u32, height: u32, tint: u8) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(width, height, image::Rgba([tint, 90, 30, 255]));
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .expect("encode");
    buffer.into_inner()
}

#[test]
fn image_is_copied_and_hash_is_content_addressed() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let bytes = png(64, 48, 5);
    let record = import_bytes(&paths, &bytes).unwrap();

    assert_eq!(record.hash, sha256_hex(&bytes));
    assert_eq!(record.hash.len(), 64);
    assert!(record.rel_path.starts_with("assets/"));
    assert!(record.thumb_path.starts_with("assets/thumbnails/"));
    assert_eq!(fs::read(record.absolute_path(dir.path())).unwrap(), bytes);
}

#[test]
fn importing_identical_bytes_keeps_one_original() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let bytes = png(32, 32, 7);
    let first = import_bytes(&paths, &bytes).unwrap();
    let second = import_bytes(&paths, &bytes).unwrap();
    assert_eq!(first.hash, second.hash);

    let originals = fs::read_dir(paths.assets_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .count();
    assert_eq!(originals, 1);
}

#[test]
fn invalid_payload_never_creates_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let result = import_bytes(&paths, b"definitely not an image");
    assert!(matches!(result, Err(AssetError::UnsupportedFormat)));
    if let Ok(entries) = fs::read_dir(paths.assets_dir()) {
        let files = entries
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
            .count();
        assert_eq!(files, 0);
    }
}

#[test]
fn thumbnail_respects_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let record = import_bytes(&paths, &png(4000, 3000, 3)).unwrap();
    let thumb = image::open(record.absolute_thumb(dir.path())).unwrap();
    assert!(thumb.width() <= THUMB_MAX_WIDTH);
    assert!(thumb.height() <= THUMB_MAX_HEIGHT);
    assert!(thumb.width() > 0 && thumb.height() > 0);
}

#[test]
fn temporary_source_can_be_deleted_after_import() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let source = dir.path().join("capture-temporaire.png");
    fs::write(&source, png(80, 60, 12)).unwrap();
    let bytes = read_source(&source).unwrap();
    let record = import_bytes(&paths, &bytes).unwrap();
    fs::remove_file(&source).unwrap();

    assert!(record.absolute_path(dir.path()).exists());
    assert!(image::open(record.absolute_path(dir.path())).is_ok());
}

#[test]
fn text_path_is_only_valid_for_real_images() {
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("shot.PNG");
    fs::write(&good, png(8, 8, 1)).unwrap();
    let bad = dir.path().join("broken-image.png");
    fs::write(&bad, b"not an image").unwrap();
    let text = dir.path().join("notes.txt");
    fs::write(&text, b"hello").unwrap();

    assert!(is_supported_extension(&good));
    assert!(path_is_valid_image(&good));
    assert!(!path_is_valid_image(&bad));
    assert!(!path_is_valid_image(&text));
    assert!(!path_is_valid_image(&dir.path().join("absent.png")));
}

#[test]
fn worker_imports_off_thread_and_reports_back() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AssetPaths::new(dir.path());
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = AssetWorker::spawn(paths, tx);
    let note_id = uuid::Uuid::new_v4();
    worker
        .try_import(logpane::core::assets::AssetJob {
            token: uuid::Uuid::new_v4(),
            note_id,
            after: None,
            caret: None,
            bytes: png(20, 20, 9),
        })
        .unwrap();

    let event = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    match event {
        logpane::core::assets::AssetEvent::Imported {
            note_id: id, asset, ..
        } => {
            assert_eq!(id, note_id);
            assert_eq!(asset.width, 20);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}
