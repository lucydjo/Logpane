//! Logpane: resident macOS overlay for instant observation capture.
//!
//! Layering:
//! - `core`: pure domain logic, no UI and no AppKit.
//! - `storage`: SQLite persistence behind a writer thread.
//! - `ui`: egui widgets and themes.
//! - `platform::macos`: all AppKit calls, and all `unsafe` except the lock in `instance`.

pub mod app;
pub mod core;
pub mod export;
pub mod fsutil;
pub mod instance;
pub mod platform;
pub mod shortcuts;
pub mod storage;
pub mod strings;
pub mod ui;

pub const APP_ID: &str = "io.github.lucydjo.logpane";
pub const APP_NAME: &str = "Logpane";
pub const MIN_WINDOW_WIDTH: f32 = 720.0;
pub const MIN_WINDOW_HEIGHT: f32 = 460.0;
pub const DEFAULT_WINDOW_WIDTH: f32 = 1050.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 680.0;
