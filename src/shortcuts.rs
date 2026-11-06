//! Single source of truth for keyboard bindings.
//!
//! Global registration lives in `platform::macos::hotkey`; this module owns the labels and the
//! local detection so a binding can never be defined twice.

use eframe::egui::{self, Key};
use global_hotkey::hotkey::Code;

pub const TOGGLE: &str = "⌘⇧D";
pub const FALLBACK: &str = "⌘⇧J";
pub const TOGGLE_CODE: Code = Code::KeyD;
pub const FALLBACK_CODE: Code = Code::KeyJ;
pub const NEW_NOTE: &str = "⌘N";
pub const SEARCH: &str = "⌘K";
pub const INSERT_TIME: &str = "⌘⇧;";
pub const HIDE: &str = "⌘W";
pub const QUIT: &str = "⌘Q";

#[derive(Debug, Clone, Copy, Default)]
pub struct Pressed {
    pub escape: bool,
    pub new_note: bool,
    pub search: bool,
    pub hide: bool,
    pub quit: bool,
    pub insert_time: bool,
    pub paste: bool,
}

pub fn read(ctx: &egui::Context) -> Pressed {
    ctx.input(|input| Pressed {
        escape: input.key_pressed(Key::Escape),
        new_note: input.modifiers.command && input.key_pressed(Key::N),
        search: input.modifiers.command && input.key_pressed(Key::K),
        hide: input.modifiers.command && input.key_pressed(Key::W),
        quit: input.modifiers.command && input.key_pressed(Key::Q),
        insert_time: input.modifiers.command
            && input.modifiers.shift
            && input.key_pressed(Key::Semicolon),
        paste: input.modifiers.command && input.key_pressed(Key::V),
    })
}
