//! Minimal AppKit alert for startup failures, when stderr is invisible (Finder launch).

use objc2::MainThreadMarker;
use objc2_app_kit::NSAlert;
use objc2_foundation::NSString;

pub fn show_error(title: &str, message: &str) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str(message));
    alert.addButtonWithTitle(&NSString::from_str("OK"));
    alert.runModal();
}
