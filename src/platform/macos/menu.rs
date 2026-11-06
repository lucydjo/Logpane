//! Menu-bar status item: the only reliable way to quit a dock-less accessory app.
//!
//! The status item lives for the whole process. Its target stores no Objective-C ivars; the
//! channels it needs are kept in a process-wide `OnceLock`, which keeps the class definition
//! trivial and avoids mixing Objective-C ownership with Rust state.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, sel, MainThreadMarker};
use objc2_app_kit::{
    NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSString};

use crate::strings::STRINGS;

struct MenuState {
    reveal: Arc<AtomicBool>,
    quit: Arc<AtomicBool>,
    ctx: eframe::egui::Context,
}

static STATE: OnceLock<MenuState> = OnceLock::new();

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "LogpaneMenuTarget"]
    #[ivars = ()]
    struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(revealLogpane:))]
        fn reveal_capture_log(&self, _sender: &AnyObject) {
            if let Some(state) = STATE.get() {
                state.reveal.store(true, Ordering::SeqCst);
                state.ctx.request_repaint();
            }
        }

        #[unsafe(method(quitLogpane:))]
        fn quit_capture_log(&self, _sender: &AnyObject) {
            if let Some(state) = STATE.get() {
                state.quit.store(true, Ordering::SeqCst);
                state.ctx.request_repaint();
            }
        }
    }
);

/// Kept alive by the application for the whole process lifetime.
pub struct MenuBar {
    _status_item: Retained<NSStatusItem>,
    _target: Retained<MenuTarget>,
}

impl MenuBar {
    pub fn install(
        ctx: eframe::egui::Context,
        reveal: Arc<AtomicBool>,
        quit: Arc<AtomicBool>,
    ) -> Option<Self> {
        let mtm = MainThreadMarker::new()?;
        let _ = STATE.set(MenuState { reveal, quit, ctx });

        let target: Retained<MenuTarget> = unsafe {
            let allocated = mtm.alloc::<MenuTarget>();
            msg_send![allocated, init]
        };
        let target_object: &AnyObject =
            unsafe { &*(Retained::as_ptr(&target) as *const AnyObject) };

        let status_bar = NSStatusBar::systemStatusBar();
        // Safety: reading an AppKit constant is safe on the main thread.
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

        if let Some(button) = status_item.button(mtm) {
            let symbol = NSString::from_str("note.text");
            let description = NSString::from_str("Logpane");
            if let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &symbol,
                Some(&description),
            ) {
                image.setTemplate(true);
                button.setImage(Some(&image));
            } else {
                button.setTitle(&NSString::from_str("J"));
            }
        }

        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);

        let show_item = NSMenuItem::new(mtm);
        show_item.setTitle(&NSString::from_str(STRINGS.menu_show));
        // Safety: the target is retained by `MenuBar` and the action is defined above.
        unsafe {
            show_item.setTarget(Some(target_object));
            show_item.setAction(Some(sel!(revealLogpane:)));
        }
        menu.addItem(&show_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let quit_item = NSMenuItem::new(mtm);
        quit_item.setTitle(&NSString::from_str(STRINGS.menu_quit));
        quit_item.setKeyEquivalent(&NSString::from_str("q"));
        // Safety: same as above.
        unsafe {
            quit_item.setTarget(Some(target_object));
            quit_item.setAction(Some(sel!(quitLogpane:)));
        }
        menu.addItem(&quit_item);

        status_item.setMenu(Some(&menu));
        tracing::debug!("menu-bar status item installed");

        Some(Self {
            _status_item: status_item,
            _target: target,
        })
    }
}
