//! NSPasteboard bridge.
//!
//! macOS exposes screenshots as TIFF or PNG, some apps only as a temporary file URL, and browsers
//! as either. Bitmaps cannot be trusted on their own: spreadsheets and word processors add a
//! rendering of the copied text, and Finder adds the icon of a copied file. So the order is: an
//! image file, then text, then a bitmap.

use std::path::PathBuf;

use objc2_app_kit::{
    NSPasteboard, NSPasteboardTypeFileURL, NSPasteboardTypePNG, NSPasteboardTypeString,
    NSPasteboardTypeTIFF, NSPasteboardTypeURL,
};
use objc2_foundation::{NSData, NSString, NSURL};

pub enum ClipboardImage {
    /// Raw image bytes, already copied out of the pasteboard.
    Bytes(Vec<u8>),
    /// A file that must be read immediately, while access is still granted.
    File(PathBuf),
    /// A representation existed but exceeded the import budget; surface it to the user.
    Rejected(String),
}

/// Best available image representation in the general pasteboard.
pub fn read_clipboard_image() -> Option<ClipboardImage> {
    read_image_from(&NSPasteboard::generalPasteboard())
}

/// `None` means the content should be pasted as ordinary text, if there is any.
fn read_image_from(pasteboard: &NSPasteboard) -> Option<ClipboardImage> {
    let file_url = unsafe { NSPasteboardTypeFileURL };
    let url_type = unsafe { NSPasteboardTypeURL };
    for data_type in [file_url, url_type] {
        if let Some(value) = pasteboard.stringForType(data_type) {
            if let Some(path) = file_url_to_path(&value) {
                if crate::core::assets::path_is_valid_image(&path) {
                    return Some(ClipboardImage::File(path));
                }
            }
        }
    }

    let string_type = unsafe { NSPasteboardTypeString };
    if let Some(value) = pasteboard.stringForType(string_type) {
        let text = value.to_string();
        let trimmed = text.trim();
        // A single-line text that is exactly the path of a valid image is treated as an image.
        // Anything else stays text; we never convert text that merely looks like a path.
        if !trimmed.is_empty() && !trimmed.contains('\n') {
            if let Some(path) = text_path_to_path(trimmed) {
                if crate::core::assets::path_is_valid_image(&path) {
                    return Some(ClipboardImage::File(path));
                }
            }
        }
        if text_wins_over_bitmap(trimmed) {
            return None;
        }
    }

    let png = unsafe { NSPasteboardTypePNG };
    let tiff = unsafe { NSPasteboardTypeTIFF };
    for data_type in [png, tiff] {
        if let Some(data) = pasteboard.dataForType(data_type) {
            match nsdata_to_vec(&data) {
                Ok(bytes) if !bytes.is_empty() => {
                    return Some(ClipboardImage::Bytes(bytes));
                }
                Ok(_) => {}
                Err(message) => return Some(ClipboardImage::Rejected(message)),
            }
        }
    }

    None
}

/// Text copied along with a bitmap is what the user meant, unless it is only the address of the
/// image, which is what browsers provide for "Copy Image".
fn text_wins_over_bitmap(text: &str) -> bool {
    let is_bare_url = !text.contains(char::is_whitespace)
        && ["http://", "https://", "file://", "data:"]
            .iter()
            .any(|scheme| text.starts_with(scheme));
    !text.is_empty() && !is_bare_url
}

fn nsdata_to_vec(data: &NSData) -> Result<Vec<u8>, String> {
    let length = data.length();
    if length == 0 {
        return Ok(Vec::new());
    }
    if length as u64 > crate::core::assets::MAX_IMPORT_BYTES {
        return Err("image is too large".to_string());
    }
    let mut buffer = vec![0u8; length];
    let pointer = std::ptr::NonNull::new(buffer.as_mut_ptr() as *mut std::ffi::c_void);
    if let Some(pointer) = pointer {
        // Safety: the buffer is allocated for exactly `length` bytes.
        unsafe { data.getBytes_length(pointer, length) };
    }
    Ok(buffer)
}

fn file_url_to_path(value: &NSString) -> Option<PathBuf> {
    let url = NSURL::URLWithString(value)?;
    if !url.isFileURL() {
        return None;
    }
    let path = url.path()?;
    Some(PathBuf::from(path.to_string()))
}

fn text_path_to_path(text: &str) -> Option<PathBuf> {
    if let Some(rest) = text.strip_prefix("file://") {
        let url = NSURL::URLWithString(&NSString::from_str(text))?;
        return url
            .path()
            .map(|path| PathBuf::from(path.to_string()))
            .or_else(|| {
                let decoded = rest.replace("%20", " ");
                Some(PathBuf::from(decoded))
            });
    }
    if text.starts_with('/') {
        return Some(PathBuf::from(text));
    }
    if let Some(rest) = text.strip_prefix("~/") {
        if let Some(home) = directories::BaseDirs::new() {
            return Some(home.home_dir().join(rest));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder;

    /// A private pasteboard, so that running the tests never touches the user's clipboard.
    fn pasteboard() -> objc2::rc::Retained<NSPasteboard> {
        let name = NSString::from_str(&format!("logpane.test.{}", uuid::Uuid::new_v4()));
        let pasteboard = NSPasteboard::pasteboardWithName(&name);
        pasteboard.clearContents();
        pasteboard
    }

    fn png() -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([9, 9, 9, 255]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(image.as_raw(), 2, 2, image::ExtendedColorType::Rgba8)
            .unwrap();
        bytes.into_inner()
    }

    fn set_png(pasteboard: &NSPasteboard) {
        let data = NSData::with_bytes(&png());
        assert!(pasteboard.setData_forType(Some(&data), unsafe { NSPasteboardTypePNG }));
    }

    fn set_string(pasteboard: &NSPasteboard, text: &str) {
        let text = NSString::from_str(text);
        assert!(pasteboard.setString_forType(&text, unsafe { NSPasteboardTypeString }));
    }

    #[test]
    fn a_bitmap_alone_is_an_image() {
        let pasteboard = pasteboard();
        set_png(&pasteboard);
        assert!(matches!(
            read_image_from(&pasteboard),
            Some(ClipboardImage::Bytes(_))
        ));
    }

    #[test]
    fn text_copied_with_a_rendering_stays_text() {
        let pasteboard = pasteboard();
        set_png(&pasteboard);
        set_string(&pasteboard, "Q3\t1200\nQ4\t1400");
        assert!(read_image_from(&pasteboard).is_none());
    }

    #[test]
    fn an_image_copied_from_a_browser_is_an_image() {
        let pasteboard = pasteboard();
        set_png(&pasteboard);
        set_string(&pasteboard, "https://example.com/chart.png");
        assert!(matches!(
            read_image_from(&pasteboard),
            Some(ClipboardImage::Bytes(_))
        ));
    }

    #[test]
    fn a_copied_image_file_wins_over_its_finder_icon_and_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("capture.png");
        std::fs::write(&path, png()).unwrap();
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
        let url = url.absoluteString().unwrap();

        let pasteboard = pasteboard();
        set_png(&pasteboard);
        set_string(&pasteboard, "capture.png");
        assert!(pasteboard.setString_forType(&url, unsafe { NSPasteboardTypeFileURL }));

        match read_image_from(&pasteboard) {
            Some(ClipboardImage::File(found)) => {
                assert_eq!(found.file_name(), path.file_name());
            }
            _ => panic!("the file should have been selected"),
        }
    }

    #[test]
    fn a_web_url_is_never_read_as_a_local_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("capture.png");
        std::fs::write(&path, png()).unwrap();
        let web = format!("https://example.com{}", path.display());
        assert!(file_url_to_path(&NSString::from_str(&web)).is_none());
    }

    #[test]
    fn plain_text_is_not_an_image() {
        let pasteboard = pasteboard();
        set_string(&pasteboard, "hello");
        assert!(read_image_from(&pasteboard).is_none());
    }
}
