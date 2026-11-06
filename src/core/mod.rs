#![forbid(unsafe_code)]

//! Domain layer. No egui, no AppKit, no filesystem assumptions beyond what is injected.

pub mod assets;
pub mod autosave;
pub mod models;
pub mod note_service;
pub mod search;
