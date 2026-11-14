//! Native save dialog for exporting a note.

use std::path::PathBuf;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSModalResponseOK, NSSavePanel};
use objc2_foundation::{NSArray, NSString};

/// Ask where to save a file. The chosen path always ends with `extension`.
pub fn choose_path(default_name: &str, extension: &str) -> Option<PathBuf> {
    let mtm = MainThreadMarker::new()?;
    let panel = NSSavePanel::new(mtm);
    panel.setNameFieldStringValue(&NSString::from_str(default_name));
    panel.setCanCreateDirectories(true);

    #[allow(deprecated)]
    {
        let types = NSArray::from_retained_slice(&[NSString::from_str(extension)]);
        panel.setAllowedFileTypes(Some(&types));
    }

    if panel.runModal() != NSModalResponseOK {
        return None;
    }
    let url = panel.URL()?;
    let path = url.path()?;
    let mut path = PathBuf::from(path.to_string());
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|found| !found.eq_ignore_ascii_case(extension))
        .unwrap_or(true)
    {
        path.set_extension(extension);
    }
    Some(path)
}
