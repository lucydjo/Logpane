//! Application state and event routing. UI code lives in `ui/`; nothing here draws directly.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use eframe::egui;
use uuid::Uuid;

use crate::core::assets::{AssetEvent, AssetJob, AssetRecord, AssetWorker};
use crate::core::autosave::{SaveStatus, SaveTracker};
use crate::core::models::{Note, NoteBlock};
use crate::core::note_service;
use crate::core::search;
use crate::platform::macos::{pasteboard, window};
use crate::shortcuts;
use crate::storage::{LoadedState, WriteEvent, WriterHandle};
use crate::strings::{self, STRINGS};
use crate::ui::images::ImageCache;
use crate::ui::theme::{self, Palette, ThemeMode};

pub const SETTING_THEME: &str = "theme_mode";
pub const SETTING_ACTIVE_NOTE: &str = "active_note_id";
pub const SETTING_GEOMETRY: &str = "window_geometry";

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Geometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            x: 180.0,
            y: 140.0,
            width: 1050.0,
            height: 680.0,
        }
    }
}

pub struct Startup {
    pub root: PathBuf,
    pub loaded: LoadedState,
    pub initial_geometry: Geometry,
    pub writer: WriterHandle,
    pub write_rx: Receiver<WriteEvent>,
    pub asset_worker: AssetWorker,
    pub asset_rx: Receiver<AssetEvent>,
    pub show_requested: Arc<AtomicBool>,
    pub hotkey_label: String,
    pub hotkey_conflict: bool,
    pub start_visible: bool,
}

pub struct Notice {
    pub text: String,
    pub until: Instant,
}

pub struct PendingImport {
    pub token: Uuid,
    pub note_id: Uuid,
    pub after: Option<Uuid>,
    pub caret: Option<usize>,
}

pub struct LogpaneApp {
    pub root: PathBuf,
    pub writer: WriterHandle,
    write_rx: Receiver<WriteEvent>,
    asset_worker: AssetWorker,
    asset_rx: Receiver<AssetEvent>,

    pub notes: Vec<Note>,
    pub assets: HashMap<String, AssetRecord>,
    pub active_id: Uuid,
    pub read_only: HashSet<Uuid>,

    pub search: String,
    pub focus_search: bool,
    pub theme_mode: ThemeMode,
    pub palette: Palette,
    applied_theme: Option<(ThemeMode, bool)>,

    pub save: SaveTracker,
    pub images: ImageCache,
    pub pending: Vec<PendingImport>,
    pub active_block: Option<Uuid>,
    pub focus_block: Option<Uuid>,
    pub delete_target: Option<Uuid>,
    pub preview: Option<String>,
    pub notice: Option<Notice>,

    pub visible: bool,
    start_visible: bool,
    pub hotkey_label: String,
    pub hotkey_conflict: bool,
    show_requested: Arc<AtomicBool>,
    reveal_requested: Arc<AtomicBool>,
    quit_requested: Arc<AtomicBool>,
    menu: Option<crate::platform::macos::menu::MenuBar>,
    window_configured: bool,
    quitting: bool,
    pending_hide: bool,
    pending_quit: bool,

    pub last_geometry: Geometry,
    geometry_pending: Option<(Geometry, Instant)>,
    notes_version: u64,
    search_cache: Option<(String, u64, Vec<Uuid>)>,
    pub block_rects: Vec<(Uuid, egui::Rect)>,

    // Development-only visual capture (`LOGPANE_SCREENSHOT=/path.png`).
    screenshot_path: Option<PathBuf>,
    screenshot_requested: bool,
    screenshot_done: Arc<std::sync::atomic::AtomicBool>,
}

impl LogpaneApp {
    pub fn new(cc: &eframe::CreationContext<'_>, startup: Startup) -> Self {
        let Startup {
            root,
            loaded,
            initial_geometry,
            writer,
            write_rx,
            asset_worker,
            asset_rx,
            show_requested,
            hotkey_label,
            hotkey_conflict,
            start_visible,
        } = startup;

        let warning = loaded.warnings.first().cloned();
        let mut notes = loaded.notes;
        note_service::sort_notes(&mut notes);
        if notes.is_empty() {
            note_service::create_note(&mut notes);
        }
        let active_id = loaded
            .settings
            .get(SETTING_ACTIVE_NOTE)
            .and_then(|raw| Uuid::parse_str(raw).ok())
            .filter(|id| notes.iter().any(|n| n.id == *id))
            .unwrap_or_else(|| notes[0].id);
        let theme_mode = loaded
            .settings
            .get(SETTING_THEME)
            .map(|raw| ThemeMode::parse(raw))
            .unwrap_or_default();

        let mut app = Self {
            root,
            writer,
            write_rx,
            asset_worker,
            asset_rx,
            notes,
            assets: loaded.assets,
            active_id,
            read_only: loaded.read_only,
            search: String::new(),
            focus_search: false,
            theme_mode,
            palette: Palette::for_mode(ThemeMode::Dark),
            applied_theme: None,
            save: SaveTracker::new(),
            images: ImageCache::default(),
            pending: Vec::new(),
            active_block: None,
            focus_block: None,
            delete_target: None,
            preview: None,
            notice: warning.map(|text| Notice {
                text,
                until: Instant::now() + Duration::from_secs(10),
            }),
            visible: false,
            start_visible,
            hotkey_label,
            hotkey_conflict,
            show_requested,
            reveal_requested: Arc::new(AtomicBool::new(false)),
            quit_requested: Arc::new(AtomicBool::new(false)),
            menu: None,
            window_configured: false,
            quitting: false,
            pending_hide: false,
            pending_quit: false,
            last_geometry: initial_geometry,
            geometry_pending: None,
            notes_version: 0,
            search_cache: None,
            block_rects: Vec::new(),
            screenshot_path: std::env::var("LOGPANE_SCREENSHOT").ok().map(PathBuf::from),
            screenshot_requested: false,
            screenshot_done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        app.focus_block = app.first_text_block(app.active_id);
        crate::ui::theme::install_fonts(&cc.egui_ctx);
        app.refresh_palette(&cc.egui_ctx);
        app
    }

    // ---------------------------------------------------------------- notes

    pub fn active_note(&self) -> &Note {
        self.notes
            .iter()
            .find(|n| n.id == self.active_id)
            .unwrap_or(&self.notes[0])
    }

    pub fn active_note_mut(&mut self) -> &mut Note {
        if !self.notes.iter().any(|n| n.id == self.active_id) {
            self.active_id = self.notes[0].id;
        }
        let id = self.active_id;
        self.notes
            .iter_mut()
            .find(|n| n.id == id)
            .expect("active note")
    }

    pub fn note(&self, id: Uuid) -> Option<&Note> {
        self.notes.iter().find(|n| n.id == id)
    }

    fn first_text_block(&self, note_id: Uuid) -> Option<Uuid> {
        self.note(note_id)
            .and_then(|note| note.blocks.iter().find(|b| b.as_text().is_some()))
            .map(NoteBlock::id)
    }

    pub fn filtered_ids(&mut self) -> Vec<Uuid> {
        if let Some((query, version, ids)) = &self.search_cache {
            if *query == self.search && *version == self.notes_version {
                return ids.clone();
            }
        }
        let hits = search::search(&self.notes, &self.search);
        let ids: Vec<Uuid> = hits.into_iter().map(|hit| hit.note_id).collect();
        self.search_cache = Some((self.search.clone(), self.notes_version, ids.clone()));
        ids
    }

    fn bump_notes_version(&mut self) {
        self.notes_version = self.notes_version.wrapping_add(1);
        self.search_cache = None;
    }

    /// Replace the working copy of a note after the editor mutated it.
    pub fn commit_note(&mut self, ctx: &egui::Context, mut note: Note, changed: bool) {
        if self.read_only.contains(&note.id) {
            return;
        }
        if changed {
            note.refresh_auto_title();
            note.touch();
            if let Some(existing) = self.notes.iter_mut().find(|n| n.id == note.id) {
                *existing = note;
            }
            note_service::sort_notes(&mut self.notes);
            self.bump_notes_version();
            self.save.mark_dirty(Instant::now());
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    pub fn create_note(&mut self, ctx: &egui::Context) {
        self.flush_now();
        let id = note_service::create_note(&mut self.notes);
        self.bump_notes_version();
        self.active_id = id;
        self.active_block = self.first_text_block(id);
        self.focus_block = self.active_block;
        self.search.clear();
        self.persist_setting(SETTING_ACTIVE_NOTE, id.to_string());
        let note = self.active_note().clone();
        self.persist_note_now(note);
        ctx.request_repaint();
    }

    pub fn select_note(&mut self, ctx: &egui::Context, id: Uuid) {
        if id == self.active_id {
            return;
        }
        self.flush_now();
        if self.notes.iter().any(|n| n.id == id) {
            self.active_id = id;
            self.active_block = self.first_text_block(id);
            self.focus_block = self.active_block;
            self.persist_setting(SETTING_ACTIVE_NOTE, id.to_string());
        }
        ctx.request_repaint();
    }

    pub fn request_delete(&mut self, id: Uuid) {
        self.delete_target = Some(id);
    }

    pub fn confirm_delete(&mut self, ctx: &egui::Context) {
        let Some(id) = self.delete_target.take() else {
            return;
        };
        self.flush_now();
        let was_active = id == self.active_id;
        let next = note_service::delete_note(&mut self.notes, id);
        self.bump_notes_version();
        if !self.writer.delete_note(id) {
            self.fail_save();
        }
        if self.notes.is_empty() {
            let new_id = note_service::create_note(&mut self.notes);
            self.bump_notes_version();
            self.active_id = new_id;
            let note = self.active_note().clone();
            self.persist_note_now(note);
        } else if was_active {
            if let Some(next) = next {
                self.active_id = next;
            } else if !self.notes.iter().any(|n| n.id == self.active_id) {
                self.active_id = self.notes[0].id;
            }
        }
        self.active_block = self.first_text_block(self.active_id);
        self.focus_block = self.active_block;
        self.persist_setting(SETTING_ACTIVE_NOTE, self.active_id.to_string());
        ctx.request_repaint();
    }

    pub fn export_pdf(&mut self, ctx: &egui::Context) {
        let note = self.active_note().clone();
        crate::platform::macos::window::activate();
        let default_name = crate::export::default_filename(&note);
        let Some(path) = crate::platform::macos::save_panel::choose_pdf_path(&default_name) else {
            return;
        };
        match crate::export::export_note(&note, &self.assets, &self.root, &path) {
            Ok(replaced) => {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("PDF");
                let mut text = format!("{}: {name}", STRINGS.exported);
                if replaced > 0 {
                    text.push_str(&format!(" · {replaced} {}", STRINGS.export_chars_replaced));
                }
                self.set_notice(ctx, text);
            }
            Err(error) => {
                self.set_notice(ctx, format!("{}: {error}", STRINGS.export_failed));
            }
        }
        ctx.request_repaint();
    }

    pub fn insert_timestamp(&mut self, ctx: &egui::Context) {
        if self.read_only.contains(&self.active_id) {
            return;
        }
        let stamp = format!("{} — ", Local::now().format("%H:%M"));
        let anchor = self
            .active_block
            .filter(|id| self.active_note().text_block(*id).is_some())
            .or_else(|| self.first_text_block(self.active_id));
        let caret = anchor.and_then(|id| self.caret_index(ctx, id));

        let mut next_focus = None;
        let mut next_active = None;
        let mut next_caret = None;
        {
            let note = self.active_note_mut();
            if let Some(block_id) = anchor {
                let length = note
                    .text_block(block_id)
                    .map(|t| t.chars().count())
                    .unwrap_or(0);
                let index = caret.unwrap_or(length);
                if let Some(new_index) = note.insert_text_at(block_id, index, &stamp) {
                    next_focus = Some(block_id);
                    next_caret = Some(new_index);
                }
            } else {
                let new_id = note.append_text_block(None, stamp);
                next_active = Some(new_id);
                next_focus = Some(new_id);
                next_caret = Some(0);
            }
            note.touch();
        }
        if let Some(focus) = next_focus {
            self.focus_block = Some(focus);
        }
        if let Some(active) = next_active {
            self.active_block = Some(active);
        }
        if let (Some(block_id), Some(index)) = (next_focus, next_caret) {
            Self::set_caret(ctx, block_id, index);
        }
        note_service::sort_notes(&mut self.notes);
        self.bump_notes_version();
        self.save.mark_dirty(Instant::now());
        ctx.request_repaint_after(Duration::from_millis(200));
    }

    // --------------------------------------------------------------- images

    pub fn import_clipboard_image(&mut self, ctx: &egui::Context) -> bool {
        if self.read_only.contains(&self.active_id) {
            self.set_notice(ctx, STRINGS.read_only_notice);
            return false;
        }
        let Some(source) = pasteboard::read_clipboard_image() else {
            return false;
        };
        let anchor = self
            .active_block
            .filter(|id| self.active_note().text_block(*id).is_some())
            .or_else(|| self.first_text_block(self.active_id));
        let caret = anchor.and_then(|id| self.caret_index(ctx, id));
        self.begin_import(ctx, source, anchor, caret)
    }

    pub fn begin_import(
        &mut self,
        ctx: &egui::Context,
        source: pasteboard::ClipboardImage,
        anchor: Option<Uuid>,
        caret: Option<usize>,
    ) -> bool {
        let bytes = match source {
            pasteboard::ClipboardImage::Bytes(bytes) => bytes,
            pasteboard::ClipboardImage::File(path) => {
                match crate::core::assets::read_source(&path) {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        self.set_notice(ctx, STRINGS.image_error);
                        return false;
                    }
                }
            }
            pasteboard::ClipboardImage::Rejected(message) => {
                self.set_notice(ctx, format!("{}: {message}", STRINGS.image_error));
                return false;
            }
        };
        let token = Uuid::new_v4();
        let job = AssetJob {
            token,
            note_id: self.active_id,
            after: anchor,
            caret,
            bytes,
        };
        if self.asset_worker.try_import(job).is_err() {
            self.set_notice(ctx, STRINGS.image_error);
            return false;
        }
        self.pending.push(PendingImport {
            token,
            note_id: self.active_id,
            after: anchor,
            caret,
        });
        ctx.request_repaint();
        true
    }

    pub fn handle_dropped(&mut self, ctx: &egui::Context) {
        let files = ctx.input(|i| i.raw.dropped_files.clone());
        if files.is_empty() {
            return;
        }
        let hover = ctx.input(|i| i.pointer.hover_pos());
        // The asset worker is FIFO and every import is inserted after the same anchor, so
        // enqueueing in reverse order makes the final document match the drop order.
        for file in files.into_iter().rev() {
            let path = file.path().to_path_buf();
            let hint = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            if let Ok(metadata) = std::fs::metadata(&path) {
                if metadata.len() > crate::core::assets::MAX_IMPORT_BYTES {
                    self.set_notice(ctx, format!("{}: {hint}", STRINGS.unsupported_format));
                    continue;
                }
            }
            let bytes = match file.bytes() {
                Ok(bytes) if !bytes.is_empty() => bytes,
                _ => match crate::core::assets::read_source(&path) {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        self.set_notice(ctx, format!("{}: {hint}", STRINGS.unsupported_format));
                        continue;
                    }
                },
            };
            if bytes.len() as u64 > crate::core::assets::MAX_IMPORT_BYTES {
                self.set_notice(ctx, format!("{}: {hint}", STRINGS.unsupported_format));
                continue;
            }
            if image::guess_format(&bytes).is_err() {
                self.set_notice(ctx, format!("{}: {hint}", STRINGS.unsupported_format));
                continue;
            }
            let anchor = hover
                .and_then(|pos| {
                    self.block_rects
                        .iter()
                        .find(|(_, rect)| rect.contains(pos))
                        .map(|(id, _)| *id)
                })
                .or_else(|| {
                    self.active_block
                        .filter(|id| self.active_note().text_block(*id).is_some())
                })
                .or_else(|| self.first_text_block(self.active_id));
            self.begin_import(ctx, pasteboard::ClipboardImage::Bytes(bytes), anchor, None);
        }
        ctx.request_repaint();
    }

    fn insert_imported_image(
        &mut self,
        ctx: &egui::Context,
        note_id: Uuid,
        after: Option<Uuid>,
        caret: Option<usize>,
        asset: AssetRecord,
    ) {
        self.assets.insert(asset.hash.clone(), asset.clone());
        if !self.writer.upsert_asset(asset.clone()) {
            self.fail_save();
            return;
        }

        if self.read_only.contains(&note_id) {
            return;
        }
        let hash = asset.hash.clone();
        let Some(index) = self.notes.iter().position(|n| n.id == note_id) else {
            return;
        };

        let mut inserted_at_caret = None;
        {
            let note = &mut self.notes[index];
            if let (Some(anchor), Some(caret)) = (after, caret) {
                inserted_at_caret = note.insert_image_at(anchor, caret, hash.clone());
            }
            if inserted_at_caret.is_none() {
                note.insert_block_after(after, NoteBlock::image(hash));
            }
            note.normalize();
            note.touch();
            note.revision += 1;
        }

        let trailing_text = self.notes[index]
            .blocks
            .last()
            .filter(|block| block.as_text().is_some())
            .map(NoteBlock::id);
        let snapshot = self.notes[index].clone();
        if !self.writer.save_note(snapshot) {
            self.fail_save();
        }
        note_service::sort_notes(&mut self.notes);
        self.bump_notes_version();
        if note_id == self.active_id {
            self.focus_block = inserted_at_caret.or(trailing_text);
            self.save.mark_dirty(Instant::now());
        }
        ctx.request_repaint();
    }

    // ------------------------------------------------------------- lifecycle

    pub fn show_window(&mut self, ctx: &egui::Context) {
        self.window_configured = window::configure_windows() || self.window_configured;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        window::activate();
        self.visible = true;
        if let Some(block) = self
            .active_block
            .or_else(|| self.first_text_block(self.active_id))
        {
            self.focus_block = Some(block);
            ctx.memory_mut(|memory| memory.request_focus(egui::Id::unique(block)));
        }
        ctx.request_repaint();
    }

    pub fn hide_window(&mut self, ctx: &egui::Context) {
        let _ = self.flush_now();
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.visible = false;
    }

    pub fn quit(&mut self, ctx: &egui::Context) {
        if !self.flush_now() {
            let message = "save failed; quit cancelled".to_string();
            self.set_notice(ctx, format!("{}: {message}", STRINGS.save_error));
            if !self.visible {
                self.show_window(ctx);
            }
            return;
        }
        self.quitting = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.quitting {
                return;
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_hide(ctx);
        }
    }

    pub fn request_hide(&mut self, ctx: &egui::Context) {
        self.pending_hide = true;
        ctx.request_repaint();
    }

    pub fn toggle(&mut self, ctx: &egui::Context) {
        if self.visible {
            self.request_hide(ctx);
        } else {
            self.show_window(ctx);
        }
    }

    pub fn consume_hotkey(&mut self, ctx: &egui::Context) {
        if self
            .show_requested
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            self.toggle(ctx);
        }
    }

    // -------------------------------------------------------------- autosave

    pub fn persist_note_now(&mut self, note: Note) {
        self.queue_save(note);
    }

    /// Single persistence path: normalize, assign a revision, store in memory, enqueue.
    fn queue_save(&mut self, mut note: Note) {
        if self.read_only.contains(&note.id) {
            return;
        }
        note.normalize();
        note.revision += 1;
        if let Some(existing) = self.notes.iter_mut().find(|n| n.id == note.id) {
            *existing = note.clone();
        }
        self.save.begin_save(note.id, note.revision);
        if !self.writer.save_note(note) {
            self.fail_save();
        }
    }

    fn fail_save(&mut self) {
        self.fail_save_with("database unavailable");
    }

    fn fail_save_with(&mut self, message: &str) {
        self.save.fail(message.to_string(), Instant::now());
        self.notice = Some(Notice {
            text: format!("{}: {message}", STRINGS.save_error),
            until: Instant::now() + Duration::from_secs(6),
        });
    }

    fn persist_active(&mut self, ctx: &egui::Context) {
        if self.read_only.contains(&self.active_id) {
            return;
        }
        let note = self.active_note().clone();
        self.queue_save(note);
        ctx.request_repaint_after(Duration::from_millis(120));
    }

    /// Flush the write queue and return whether the last revision is safely persisted.
    pub fn flush_now(&mut self) -> bool {
        if self.save.is_dirty() || self.save.is_inflight() {
            if self.save.is_dirty() {
                let note = self.active_note().clone();
                self.queue_save(note);
            }
            if let Err(message) = self.writer.flush(Duration::from_secs(3)) {
                self.fail_save_with(&message);
            }
        }
        !matches!(self.save.status(), SaveStatus::Error)
    }

    fn tick_autosave(&mut self, ctx: &egui::Context) {
        if self.save.should_save(Instant::now()) {
            self.persist_active(ctx);
        }
    }

    fn process_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.write_rx.try_recv() {
            match event {
                WriteEvent::Persisted { note_id, revision } => {
                    self.save.ack(note_id, revision);
                    ctx.request_repaint();
                }
                WriteEvent::Failed {
                    note_id,
                    revision,
                    message,
                } => {
                    if note_id == self.active_id {
                        self.fail_save_with(&message);
                    } else {
                        self.save.abandon(note_id, revision);
                        self.set_notice(ctx, format!("{}: {message}", STRINGS.save_error));
                    }
                }
                WriteEvent::StorageFault { message } => {
                    self.fail_save_with(&message);
                    self.set_notice(ctx, format!("{}: {message}", STRINGS.save_error));
                }
            }
        }

        while let Ok(event) = self.asset_rx.try_recv() {
            match event {
                AssetEvent::Imported {
                    token,
                    note_id,
                    after,
                    caret,
                    asset,
                } => {
                    self.pending.retain(|p| p.token != token);
                    self.insert_imported_image(ctx, note_id, after, caret, asset);
                }
                AssetEvent::Failed { token, message, .. } => {
                    self.pending.retain(|p| p.token != token);
                    self.set_notice(ctx, format!("{}: {message}", STRINGS.image_error));
                }
            }
        }
    }

    // --------------------------------------------------------------- helpers

    pub fn caret_index(&self, ctx: &egui::Context, block_id: Uuid) -> Option<usize> {
        let state = egui::text_edit::TextEditState::load(ctx, egui::Id::unique(block_id))?;
        state.cursor.char_range().map(|range| range.primary.index.0)
    }

    pub fn set_notice(&mut self, ctx: &egui::Context, text: impl Into<String>) {
        self.notice = Some(Notice {
            text: text.into(),
            until: Instant::now() + Duration::from_secs(6),
        });
        ctx.request_repaint_after(Duration::from_secs(6));
    }

    pub fn refresh_palette(&mut self, ctx: &egui::Context) {
        let system_dark = ctx
            .system_theme()
            .map(|t| t == egui::Theme::Dark)
            .unwrap_or(true);
        let resolved = self.theme_mode.resolve(system_dark);
        let key = (self.theme_mode, system_dark);
        if self.applied_theme != Some(key) {
            self.palette = Palette::for_mode(resolved);
            theme::apply(ctx, &self.palette);
            self.applied_theme = Some(key);
        }
    }

    pub fn cycle_theme(&mut self, ctx: &egui::Context) {
        self.theme_mode = self.theme_mode.next();
        self.applied_theme = None;
        self.refresh_palette(ctx);
        self.persist_setting(SETTING_THEME, self.theme_mode.as_str());
    }

    pub fn persist_setting(&mut self, key: &str, value: impl Into<String>) {
        if !self.writer.set_setting(key, value) {
            self.fail_save();
        }
    }

    pub fn save_label(&self) -> (&'static str, egui::Color32) {
        match self.save.status() {
            SaveStatus::Saved => (STRINGS.saved, self.palette.secondary),
            SaveStatus::Saving => (STRINGS.saving, self.palette.secondary),
            SaveStatus::Error => (STRINGS.save_error, self.palette.danger),
        }
    }

    pub fn format_save_error(&self) -> Option<&str> {
        self.save.error()
    }

    fn tick_geometry(&mut self, ctx: &egui::Context) {
        let rects = ctx.input(|input| {
            let viewport = input.viewport();
            let pos = viewport.outer_rect.map(|r| r.min);
            let size = viewport.inner_rect.map(|r| r.size());
            pos.zip(size)
        });
        let Some((pos, size)) = rects else {
            return;
        };
        let current = Geometry {
            x: pos.x,
            y: pos.y,
            width: size.x,
            height: size.y,
        };
        let changed = (current.x - self.last_geometry.x).abs() > 1.0
            || (current.y - self.last_geometry.y).abs() > 1.0
            || (current.width - self.last_geometry.width).abs() > 1.0
            || (current.height - self.last_geometry.height).abs() > 1.0;
        if changed {
            self.geometry_pending = Some((current, Instant::now()));
        }
        if let Some((geometry, since)) = self.geometry_pending {
            if since.elapsed() >= Duration::from_millis(300) {
                self.last_geometry = geometry;
                self.geometry_pending = None;
                if let Ok(json) = serde_json::to_string(&geometry) {
                    self.writer.set_setting(SETTING_GEOMETRY, json);
                }
            }
        }
    }

    fn tick_notice(&mut self) {
        if let Some(notice) = &self.notice {
            if Instant::now() >= notice.until {
                self.notice = None;
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let pressed = shortcuts::read(ctx);
        let modal_open = self.delete_target.is_some() || self.preview.is_some();

        if pressed.escape {
            if modal_open {
                self.delete_target = None;
                self.preview = None;
            } else {
                self.request_hide(ctx);
            }
            return;
        }
        if pressed.quit {
            self.pending_quit = true;
            return;
        }
        if pressed.hide {
            self.request_hide(ctx);
            return;
        }
        if pressed.new_note {
            self.create_note(ctx);
            return;
        }
        if pressed.search {
            self.focus_search = true;
            ctx.request_repaint();
            return;
        }
        if pressed.insert_time {
            self.insert_timestamp(ctx);
            return;
        }

        let text_focused = self.text_block_has_focus(ctx);
        let paste_event = ctx.input(|input| {
            input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Paste(_)))
        });
        if (pressed.paste || paste_event)
            && text_focused
            && !modal_open
            && pasteboard::read_clipboard_image().is_some()
        {
            ctx.input_mut(|input| {
                input
                    .events
                    .retain(|event| !matches!(event, egui::Event::Paste(_)));
            });
            self.import_clipboard_image(ctx);
        }
    }

    fn text_block_has_focus(&self, ctx: &egui::Context) -> bool {
        let Some(focused) = ctx.memory(|memory| memory.focused()) else {
            return false;
        };
        self.active_note()
            .blocks
            .iter()
            .any(|block| block.as_text().is_some() && egui::Id::unique(block.id()) == focused)
    }

    fn set_caret(ctx: &egui::Context, block_id: Uuid, index: usize) {
        let id = egui::Id::unique(block_id);
        let Some(mut state) = egui::text_edit::TextEditState::load(ctx, id) else {
            return;
        };
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(index),
            )));
        state.store(ctx, id);
    }

    pub fn date_label(&self, updated: DateTime<Utc>) -> String {
        let local = updated.with_timezone(&Local);
        let today = Local::now().date_naive();
        if local.date_naive() == today {
            STRINGS.today.to_string()
        } else if local.date_naive() == today.pred_opt().unwrap_or(today) {
            STRINGS.yesterday.to_string()
        } else {
            crate::ui::format::date_en(local.date_naive())
        }
    }

    pub fn strings(&self) -> &'static strings::Strings {
        STRINGS
    }

    fn capture_screenshot(&mut self, ctx: &egui::Context) {
        // Development-only hook: never active in a release build.
        if !cfg!(debug_assertions) {
            return;
        }
        let Some(path) = self.screenshot_path.clone() else {
            return;
        };
        if self
            .screenshot_done
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            self.quit(ctx);
            return;
        }
        if self.screenshot_requested {
            return;
        }
        self.screenshot_requested = true;
        let done = self.screenshot_done.clone();
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
            egui::ScreenshotCallback::new(move |image| {
                let [width, height] = image.size;
                let mut bytes = Vec::with_capacity(width * height * 4);
                for pixel in image.pixels.iter() {
                    let [r, g, b, a] = pixel.to_array();
                    bytes.extend_from_slice(&[r, g, b, a]);
                }
                if let Some(buffer) = image::RgbaImage::from_raw(width as u32, height as u32, bytes)
                {
                    let _ = buffer.save(&path);
                }
                done.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        ));
        ctx.request_repaint();
    }
}

impl eframe::App for LogpaneApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.menu.is_none() {
            self.menu = crate::platform::macos::menu::MenuBar::install(
                ctx.clone(),
                self.reveal_requested.clone(),
                self.quit_requested.clone(),
            );
        }
        if self
            .reveal_requested
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            if self.visible {
                window::activate();
            } else {
                self.show_window(ctx);
            }
        }
        if self
            .quit_requested
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            self.quit(ctx);
        }
        if !self.window_configured {
            self.window_configured = window::configure_windows();
        }
        if self.start_visible {
            self.start_visible = false;
            self.show_window(ctx);
        }
        self.consume_hotkey(ctx);
        self.process_events(ctx);
        self.tick_autosave(ctx);
        self.tick_notice();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.refresh_palette(&ctx);
        self.handle_close_request(&ctx);
        self.process_events(&ctx);
        self.handle_shortcuts(&ctx);
        self.handle_dropped(&ctx);

        crate::ui::shell::show(self, ui);

        self.tick_geometry(&ctx);
        self.tick_autosave(&ctx);
        self.tick_notice();
        self.capture_screenshot(&ctx);

        if self.pending_hide {
            self.pending_hide = false;
            if self.visible {
                self.hide_window(&ctx);
            }
        }
        if self.pending_quit {
            self.pending_quit = false;
            self.quit(&ctx);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.flush_now();
        self.writer.shutdown();
    }
}
