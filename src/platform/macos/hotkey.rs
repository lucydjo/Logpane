//! Global ⌘⇧D registration.
//!
//! `global-hotkey` requires the manager and the event loop on the main thread; the manager is
//! therefore created in `main` and kept alive for the whole process. A dedicated thread waits on
//! the event channel (no polling) and wakes egui through `request_repaint`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use eframe::egui;

use crate::shortcuts::{FALLBACK, FALLBACK_CODE, TOGGLE, TOGGLE_CODE};
use global_hotkey::hotkey::{HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

pub struct HotkeyService {
    pub requested: Arc<AtomicBool>,
    pub label: String,
    pub conflict: bool,
    _manager: Option<GlobalHotKeyManager>,
    _thread: Option<JoinHandle<()>>,
}

impl HotkeyService {
    /// Register ⌘⇧D, falling back to ⌘⇧J if macOS refuses it. Another application using the same
    /// combination is not reported as a refusal.
    /// Never fails: a conflict is reported to the UI instead of preventing startup.
    pub fn start(ctx: egui::Context) -> Self {
        let requested = Arc::new(AtomicBool::new(false));
        let manager = match GlobalHotKeyManager::new() {
            Ok(manager) => manager,
            Err(error) => {
                tracing::warn!(%error, "global hotkey manager unavailable");
                return Self {
                    requested,
                    label: TOGGLE.into(),
                    conflict: true,
                    _manager: None,
                    _thread: None,
                };
            }
        };

        let primary = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), TOGGLE_CODE);
        let fallback = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), FALLBACK_CODE);

        let label = if manager.register(primary).is_ok() {
            TOGGLE
        } else if manager.register(fallback).is_ok() {
            FALLBACK
        } else {
            return Self {
                requested,
                label: TOGGLE.into(),
                conflict: true,
                _manager: Some(manager),
                _thread: None,
            };
        };

        let flag = requested.clone();
        let thread = std::thread::Builder::new()
            .name("logpane-hotkey".into())
            .spawn(move || {
                let receiver = GlobalHotKeyEvent::receiver();
                while let Ok(event) = receiver.recv() {
                    if event.state == HotKeyState::Pressed {
                        flag.store(true, Ordering::SeqCst);
                        ctx.request_repaint();
                    }
                }
            })
            .ok();

        Self {
            requested,
            label: label.to_string(),
            conflict: false,
            _manager: Some(manager),
            _thread: thread,
        }
    }
}
