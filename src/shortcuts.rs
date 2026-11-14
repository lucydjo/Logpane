//! Single source of truth for keyboard bindings.
//!
//! Global registration lives in `platform::macos::hotkey`; this module owns the labels and the
//! local detection so a binding can never be defined twice.

use eframe::egui::{self, Key, Modifiers};
use global_hotkey::hotkey::Code;

pub const TOGGLE: &str = "⌘⇧D";
pub const FALLBACK: &str = "⌘⇧J";
pub const TOGGLE_CODE: Code = Code::KeyD;
pub const FALLBACK_CODE: Code = Code::KeyJ;
pub const NEW_NOTE: &str = "⌘N";
pub const SEARCH: &str = "⌘K";
pub const INSERT_TIME: &str = "⌘⇧;";
pub const PASTE: &str = "⌘V";
pub const COPY_MARKDOWN: &str = "⌘⇧C";
pub const PIN: &str = "⌘⇧P";
pub const NEXT_NOTE: &str = "⌥⌘↓";
pub const PREVIOUS_NOTE: &str = "⌥⌘↑";
pub const THEME: &str = "⌘⇧L";
pub const HELP: &str = "⌘/";
pub const HIDE: &str = "⌘W";
pub const HIDE_ALT: &str = "Esc";
pub const QUIT: &str = "⌘Q";

const COMMAND_SHIFT: Modifiers = Modifiers {
    alt: false,
    ctrl: false,
    shift: true,
    mac_cmd: true,
    command: true,
};
const COMMAND_ALT: Modifiers = Modifiers {
    alt: true,
    ctrl: false,
    shift: false,
    mac_cmd: true,
    command: true,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Pressed {
    pub escape: bool,
    pub new_note: bool,
    pub search: bool,
    pub hide: bool,
    pub quit: bool,
    pub insert_time: bool,
    pub paste: bool,
    pub copy_markdown: bool,
    pub pin: bool,
    pub next_note: bool,
    pub previous_note: bool,
    pub theme: bool,
    pub help: bool,
}

/// Read the bindings of this frame. Bindings that a focused text field would also interpret
/// (arrows, letters with Shift) are consumed so they are not applied twice.
pub fn read(ctx: &egui::Context) -> Pressed {
    ctx.input_mut(|input| Pressed {
        escape: input.key_pressed(Key::Escape),
        new_note: input.modifiers.command && input.key_pressed(Key::N),
        search: input.modifiers.command && input.key_pressed(Key::K),
        hide: input.modifiers.command && input.key_pressed(Key::W),
        quit: input.modifiers.command && input.key_pressed(Key::Q),
        insert_time: input.modifiers.command
            && input.modifiers.shift
            && input.key_pressed(Key::Semicolon),
        paste: input.modifiers.command && input.key_pressed(Key::V),
        copy_markdown: input.consume_key(COMMAND_SHIFT, Key::C),
        pin: input.consume_key(COMMAND_SHIFT, Key::P),
        next_note: input.consume_key(COMMAND_ALT, Key::ArrowDown),
        previous_note: input.consume_key(COMMAND_ALT, Key::ArrowUp),
        theme: input.consume_key(COMMAND_SHIFT, Key::L),
        help: input.modifiers.command
            && (input.key_pressed(Key::Slash) || input.key_pressed(Key::Questionmark)),
    })
}

/// Keys that drive the result list while the search field has focus.
#[derive(Debug, Clone, Copy, Default)]
pub struct ListKeys {
    pub down: bool,
    pub up: bool,
    pub open: bool,
}

pub fn read_list_keys(ctx: &egui::Context) -> ListKeys {
    ctx.input_mut(|input| ListKeys {
        down: input.consume_key(Modifiers::NONE, Key::ArrowDown),
        up: input.consume_key(Modifiers::NONE, Key::ArrowUp),
        open: input.key_pressed(Key::Enter),
    })
}
