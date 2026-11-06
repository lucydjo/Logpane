//! AppKit window bridge: floating level, Spaces/full-screen behaviour and geometry clamping.
//!
//! All `unsafe` and all AppKit usage of the project lives in `platform/macos`.

use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSFloatingWindowLevel, NSScreen, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::NSSize;

/// Configure the window created by eframe. Called once the native window exists.
pub fn configure_windows() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let app = NSApplication::sharedApplication(mtm);
    // Accessory: resident overlay, no Dock icon, no menu bar takeover.
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let windows = app.windows();
    let mut configured = false;
    for window in windows.iter() {
        configure_window(&window);
        configured = true;
    }
    configured
}

fn configure_window(window: &NSWindow) {
    window.setLevel(NSFloatingWindowLevel);
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::IgnoresCycle
            | NSWindowCollectionBehavior::Stationary,
    );
    window.setHidesOnDeactivate(false);
    window.setCanHide(false);
    window.setMovableByWindowBackground(false);
    window.setOpaque(true);

    // A borderless window stays user-resizable as soon as NSWindowStyleMaskResizable is set.
    // Force it (winit normally adds it from `with_resizable(true)`) and set the content minimum
    // explicitly, so edge and corner resizing are handled natively by AppKit.
    let mut mask = window.styleMask();
    mask.insert(NSWindowStyleMask::Resizable);
    window.setStyleMask(mask);
    window.setContentMinSize(NSSize::new(720.0, 460.0));
    tracing::debug!(
        resizable = window.styleMask().contains(NSWindowStyleMask::Resizable),
        "window configured for native resizing"
    );

    // Safety: keeps the window alive when it is ordered out, which is exactly the
    // resident-overlay model we need.
    unsafe { window.setReleasedWhenClosed(false) };
}

/// Bring the accessory app to the front when the overlay is shown.
pub fn activate() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
}

/// Visible frames of every screen, expressed in egui's top-left coordinate space.
pub fn visible_frames() -> Vec<(f32, f32, f32, f32)> {
    let Some(mtm) = MainThreadMarker::new() else {
        return Vec::new();
    };
    let Some(main) = NSScreen::mainScreen(mtm) else {
        return Vec::new();
    };
    let main_height = main.frame().size.height as f32;

    let mut frames = Vec::new();
    for screen in NSScreen::screens(mtm).iter() {
        let rect = screen.visibleFrame();
        let x = rect.origin.x as f32;
        let y_bottom = rect.origin.y as f32;
        let w = rect.size.width as f32;
        let h = rect.size.height as f32;
        // Cocoa origin is bottom-left; egui/winit use top-left with the main screen at y = 0.
        let y_top = main_height - (y_bottom + h);
        frames.push((x, y_top, w, h));
    }
    frames
}

/// Clamp a saved window rectangle into a visible screen.
///
/// If the rectangle still intersects a screen, it is nudged inside; otherwise it is centred on
/// the main screen, which covers the "external display disconnected" scenario.
pub fn clamp_geometry(x: f32, y: f32, width: f32, height: f32) -> (f32, f32, f32, f32) {
    const MARGIN: f32 = 8.0;
    let min_width = crate::MIN_WINDOW_WIDTH;
    let min_height = crate::MIN_WINDOW_HEIGHT;

    let frames = visible_frames();
    if frames.is_empty() {
        return (x, y, width.max(min_width), height.max(min_height));
    }

    let target = frames.iter().copied().find(|(fx, fy, fw, fh)| {
        x + width > fx + 40.0 && x < fx + fw - 40.0 && y + height > fy + 40.0 && y < fy + fh - 40.0
    });

    match target {
        Some((fx, fy, fw, fh)) => {
            let w = width.max(min_width).min((fw - MARGIN * 2.0).max(320.0));
            let h = height.max(min_height).min((fh - MARGIN * 2.0).max(240.0));
            let nx = x.clamp(fx + MARGIN, (fx + fw - w - MARGIN).max(fx + MARGIN));
            let ny = y.clamp(fy + MARGIN, (fy + fh - h - MARGIN).max(fy + MARGIN));
            (nx, ny, w, h)
        }
        None => {
            let (fx, fy, fw, fh) = frames[0];
            let w = width.max(min_width).min((fw - MARGIN * 2.0).max(320.0));
            let h = height.max(min_height).min((fh - MARGIN * 2.0).max(240.0));
            (
                fx + ((fw - w) / 2.0).max(MARGIN),
                fy + ((fh - h) / 2.0).max(MARGIN),
                w,
                h,
            )
        }
    }
}
