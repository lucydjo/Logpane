//! Content-addressed image assets.
//!
//! The invariant of this module: nothing outside it ever sees a source path. Bytes go in, a
//! stable hash and a permanent relative path come out. The original file is written before any
//! note can reference it.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, SyncSender, TrySendError};
use std::thread;

use chrono::{DateTime, Utc};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const THUMB_MAX_WIDTH: u32 = 1600;
pub const THUMB_MAX_HEIGHT: u32 = 1200;
/// Hard cap before any read or decode. A screenshot is never this large.
pub const MAX_IMPORT_BYTES: u64 = 40 * 1024 * 1024;
pub const MAX_IMAGE_PIXELS: u64 = 40_000_000;
pub const MAX_IMAGE_SIDE: u32 = 12_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRecord {
    pub hash: String,
    pub rel_path: String,
    pub thumb_path: String,
    pub mime: String,
    pub width: u32,
    pub height: u32,
    pub byte_size: u64,
    pub imported_at: DateTime<Utc>,
}

impl AssetRecord {
    pub fn absolute_path(&self, root: &Path) -> PathBuf {
        root.join(&self.rel_path)
    }

    pub fn absolute_thumb(&self, root: &Path) -> PathBuf {
        root.join(&self.thumb_path)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AssetError {
    #[error("unsupported image format")]
    UnsupportedFormat,
    #[error("unreadable image: {0}")]
    Decode(String),
    #[error("could not write file: {0}")]
    Io(String),
    #[error("too many simultaneous imports")]
    QueueFull,
    #[error("image is too large")]
    TooLarge,
}

/// Filesystem layout for assets, injected so tests never touch the real data directory.
#[derive(Debug, Clone)]
pub struct AssetPaths {
    pub root: PathBuf,
}

impl AssetPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }

    pub fn thumbs_dir(&self) -> PathBuf {
        self.root.join("assets").join("thumbnails")
    }

    pub fn prepare(&self) -> Result<(), AssetError> {
        fs::create_dir_all(self.assets_dir()).map_err(|e| AssetError::Io(e.to_string()))?;
        fs::create_dir_all(self.thumbs_dir()).map_err(|e| AssetError::Io(e.to_string()))?;
        crate::fsutil::restrict_dir(&self.root);
        crate::fsutil::restrict_dir(&self.assets_dir());
        crate::fsutil::restrict_dir(&self.thumbs_dir());
        Ok(())
    }
}

/// Import bytes into permanent storage. Identical bytes reuse the existing file.
pub fn import_bytes(paths: &AssetPaths, bytes: &[u8]) -> Result<AssetRecord, AssetError> {
    if bytes.is_empty() {
        return Err(AssetError::Decode("empty buffer".into()));
    }
    if bytes.len() as u64 > MAX_IMPORT_BYTES {
        return Err(AssetError::TooLarge);
    }
    let format = image::guess_format(bytes).map_err(|_| AssetError::UnsupportedFormat)?;
    let (ext, mime) = match format {
        ImageFormat::Png => ("png", "image/png"),
        ImageFormat::Jpeg => ("jpg", "image/jpeg"),
        ImageFormat::Tiff => ("tiff", "image/tiff"),
        ImageFormat::WebP => ("webp", "image/webp"),
        _ => return Err(AssetError::UnsupportedFormat),
    };

    // Decode with hard limits before anything is written: protects against decompression bombs.
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_SIDE);
    limits.max_image_height = Some(MAX_IMAGE_SIDE);
    limits.max_alloc = Some(256 * 1024 * 1024);
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| AssetError::Decode(e.to_string()))?;
    reader.limits(limits);
    let decoded = reader
        .decode()
        .map_err(|e| AssetError::Decode(e.to_string()))?;
    if u64::from(decoded.width()) * u64::from(decoded.height()) > MAX_IMAGE_PIXELS {
        return Err(AssetError::TooLarge);
    }
    let (width, height) = (decoded.width(), decoded.height());
    if width == 0 || height == 0 {
        return Err(AssetError::Decode("zero-sized image".into()));
    }

    let hash = sha256_hex(bytes);
    paths.prepare()?;

    let original = paths.assets_dir().join(format!("{hash}.{ext}"));
    if !original.exists() {
        write_atomic(&original, bytes)?;
    }

    let thumb = paths.thumbs_dir().join(format!("{hash}.png"));
    if !thumb.exists() {
        let thumb_bytes = render_thumbnail(&decoded)?;
        write_atomic(&thumb, &thumb_bytes)?;
    }

    Ok(AssetRecord {
        rel_path: format!("assets/{hash}.{ext}"),
        thumb_path: format!("assets/thumbnails/{hash}.png"),
        hash,
        mime: mime.to_string(),
        width,
        height,
        byte_size: bytes.len() as u64,
        imported_at: Utc::now(),
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest.iter() {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn render_thumbnail(decoded: &image::DynamicImage) -> Result<Vec<u8>, AssetError> {
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    let scale = f32::min(
        1.0,
        f32::min(
            THUMB_MAX_WIDTH as f32 / width as f32,
            THUMB_MAX_HEIGHT as f32 / height as f32,
        ),
    );
    let target_w = ((width as f32 * scale).round() as u32).max(1);
    let target_h = ((height as f32 * scale).round() as u32).max(1);
    let thumb = if scale < 1.0 {
        image::imageops::resize(
            &rgba,
            target_w,
            target_h,
            image::imageops::FilterType::Triangle,
        )
    } else {
        rgba
    };

    let mut buffer = Cursor::new(Vec::new());
    PngEncoder::new(&mut buffer)
        .write_image(thumb.as_raw(), target_w, target_h, ExtendedColorType::Rgba8)
        .map_err(|e| AssetError::Decode(e.to_string()))?;
    Ok(buffer.into_inner())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AssetError> {
    let parent = path
        .parent()
        .ok_or_else(|| AssetError::Io("no parent".into()))?;
    fs::create_dir_all(parent).map_err(|e| AssetError::Io(e.to_string()))?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AssetError::Io("invalid file name".into()))?;
    let tmp = parent.join(format!(".{file_name}.{}.tmp", Uuid::new_v4()));
    fs::write(&tmp, bytes).map_err(|e| AssetError::Io(e.to_string()))?;
    if let Ok(file) = fs::File::open(&tmp) {
        let _ = file.sync_all();
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        AssetError::Io(e.to_string())
    })?;
    crate::fsutil::restrict_file(path);
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }
    Ok(())
}

/// Read a dropped or pasted file immediately, while macOS still grants access to it.
pub fn read_source(path: &Path) -> Result<Vec<u8>, AssetError> {
    let metadata = fs::metadata(path).map_err(|e| AssetError::Io(e.to_string()))?;
    if !metadata.is_file() {
        return Err(AssetError::Io("not a file".into()));
    }
    if metadata.len() > MAX_IMPORT_BYTES {
        return Err(AssetError::TooLarge);
    }
    fs::read(path).map_err(|e| AssetError::Io(e.to_string()))
}

pub fn is_supported_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "tif" | "tiff" | "webp")
    )
}

pub struct AssetJob {
    /// Correlates the completion event with the loading chip shown by the UI.
    pub token: Uuid,
    pub note_id: Uuid,
    pub after: Option<Uuid>,
    /// Caret position captured at paste time, in characters.
    pub caret: Option<usize>,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum AssetEvent {
    Imported {
        token: Uuid,
        note_id: Uuid,
        after: Option<Uuid>,
        caret: Option<usize>,
        asset: AssetRecord,
    },
    Failed {
        token: Uuid,
        note_id: Uuid,
        message: String,
    },
}

/// One dedicated worker, bounded queue. Never blocks the UI thread.
pub struct AssetWorker {
    tx: SyncSender<AssetJob>,
}

impl AssetWorker {
    pub fn spawn(paths: AssetPaths, events: Sender<AssetEvent>) -> Self {
        let (tx, rx): (SyncSender<AssetJob>, Receiver<AssetJob>) = std::sync::mpsc::sync_channel(8);
        thread::Builder::new()
            .name("logpane-assets".into())
            .spawn(move || {
                while let Ok(job) = rx.recv() {
                    let result = import_bytes(&paths, &job.bytes);
                    let event = match result {
                        Ok(asset) => AssetEvent::Imported {
                            token: job.token,
                            note_id: job.note_id,
                            after: job.after,
                            caret: job.caret,
                            asset,
                        },
                        Err(error) => AssetEvent::Failed {
                            token: job.token,
                            note_id: job.note_id,
                            message: error.to_string(),
                        },
                    };
                    if events.send(event).is_err() {
                        break;
                    }
                }
            })
            .expect("spawn asset worker");
        Self { tx }
    }

    pub fn try_import(&self, job: AssetJob) -> Result<(), AssetError> {
        match self.tx.try_send(job) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(AssetError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(AssetError::QueueFull),
        }
    }
}

/// Validate that a path-like clipboard string points at an image, without decoding the whole
/// file on the UI thread (the real decode happens on the asset worker).
pub fn path_is_valid_image(path: &Path) -> bool {
    use std::io::Read as _;

    if !path.is_file() || !is_supported_extension(path) {
        return false;
    }
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut header = [0u8; 512];
    let Ok(read) = file.read(&mut header) else {
        return false;
    };
    read > 0 && image::guess_format(&header[..read]).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(width: u32, height: u32, tint: u8) -> Vec<u8> {
        let mut image = image::RgbaImage::new(width, height);
        for pixel in image.pixels_mut() {
            *pixel = image::Rgba([tint, 40, 80, 255]);
        }
        let mut buffer = Cursor::new(Vec::new());
        PngEncoder::new(&mut buffer)
            .write_image(image.as_raw(), width, height, ExtendedColorType::Rgba8)
            .unwrap();
        buffer.into_inner()
    }

    #[test]
    fn imports_png_and_creates_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AssetPaths::new(dir.path());
        let record = import_bytes(&paths, &png_bytes(40, 20, 10)).unwrap();
        assert_eq!(record.mime, "image/png");
        assert_eq!((record.width, record.height), (40, 20));
        assert!(record.absolute_path(dir.path()).exists());
        assert!(record.absolute_thumb(dir.path()).exists());
        assert_eq!(record.hash.len(), 64);
    }

    #[test]
    fn duplicate_bytes_reuse_the_same_file() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AssetPaths::new(dir.path());
        let bytes = png_bytes(10, 10, 99);
        let a = import_bytes(&paths, &bytes).unwrap();
        let b = import_bytes(&paths, &bytes).unwrap();
        assert_eq!(a.hash, b.hash);
        let originals = fs::read_dir(paths.assets_dir())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .count();
        assert_eq!(originals, 1);
    }

    #[test]
    fn rejects_non_image_payloads() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AssetPaths::new(dir.path());
        assert!(matches!(
            import_bytes(&paths, b"not an image"),
            Err(AssetError::UnsupportedFormat)
        ));
    }

    #[test]
    fn thumbnail_is_downscaled_but_never_upscaled() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AssetPaths::new(dir.path());
        let record = import_bytes(&paths, &png_bytes(3200, 2400, 50)).unwrap();
        let thumb = image::open(record.absolute_thumb(dir.path())).unwrap();
        assert!(thumb.width() <= THUMB_MAX_WIDTH);
        assert!(thumb.height() <= THUMB_MAX_HEIGHT);

        let small = import_bytes(&paths, &png_bytes(8, 8, 51)).unwrap();
        let thumb = image::open(small.absolute_thumb(dir.path())).unwrap();
        assert_eq!((thumb.width(), thumb.height()), (8, 8));
    }

    #[test]
    fn path_validation_rejects_missing_and_non_image_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!path_is_valid_image(&dir.path().join("absent.png")));
        let text = dir.path().join("note.png");
        fs::write(&text, b"hello").unwrap();
        assert!(!path_is_valid_image(&text));
    }
}
