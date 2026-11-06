//! Native save dialog for exporting a note.

use std::path::PathBuf;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSModalResponseOK, NSSavePanel};
use objc2_foundation::{NSArray, NSString};

pub fn choose_pdf_path(default_name: &str) -> Option<PathBuf> {
    let mtm = MainThreadMarker::new()?;
    let panel = NSSavePanel::new(mtm);
    panel.setNameFieldStringValue(&NSString::from_str(default_name));
    panel.setCanCreateDirectories(true);

    #[allow(deprecated)]
    {
        let types = NSArray::from_retained_slice(&[NSString::from_str("pdf")]);
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
        .map(|extension| !extension.eq_ignore_ascii_case("pdf"))
        .unwrap_or(true)
    {
        path.set_extension("pdf");
    }
    Some(path)
}
