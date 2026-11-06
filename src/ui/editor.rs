use eframe::egui::{self, Align, FontId, Layout, RichText, Sense, Vec2};
use uuid::Uuid;

use crate::app::LogpaneApp;
use crate::core::models::NoteBlock;
use crate::strings::STRINGS;
use crate::ui::format;
use crate::ui::theme::{margin, margin_xy, surface, ACTION_BAR_HEIGHT, SPACE_L, SPACE_S, SPACE_XS};
use crate::ui::widgets;

enum Action {
    RemoveBlock(Uuid),
    Preview(String),
    Open(String),
}

pub fn show(app: &mut LogpaneApp, ui: &mut egui::Ui) {
    let palette = app.palette;
    let note_id = app.active_id;
    let read_only = app.read_only.contains(&note_id);
    let mut note = app.active_note().clone();
    let mut changed = false;
    let mut actions: Vec<Action> = Vec::new();
    let mut insert_time = false;
    let mut insert_image = false;
    let mut export_pdf = false;
    let mut request_delete = false;

    app.block_rects.clear();

    // Reserve the action bar before the scroll area, otherwise it is pushed out of view.
    egui::Panel::bottom("editor_actions")
        .exact_size(ACTION_BAR_HEIGHT)
        .show_separator_line(true)
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            ui.add_space(SPACE_XS);
            ui.horizontal(|ui| {
                if !read_only
                    && widgets::text_button(ui, &palette, STRINGS.insert_time, palette.secondary)
                        .clicked()
                {
                    insert_time = true;
                }
                if !read_only
                    && widgets::text_button(ui, &palette, STRINGS.insert_image, palette.secondary)
                        .clicked()
                {
                    insert_image = true;
                }
                if widgets::text_button(ui, &palette, STRINGS.export_pdf, palette.secondary)
                    .clicked()
                {
                    export_pdf = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !read_only
                        && widgets::text_button(
                            ui,
                            &palette,
                            STRINGS.delete_note,
                            palette.secondary,
                        )
                        .clicked()
                    {
                        request_delete = true;
                    }
                });
            });
        });

    if let Some(notice) = &app.notice {
        let text = notice.text.clone();
        surface(&palette, palette.raised)
            .inner_margin(margin(SPACE_S))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("! {text}"))
                            .size(11.5)
                            .color(palette.text),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if widgets::text_button(
                            ui,
                            &palette,
                            STRINGS.close_button,
                            palette.secondary,
                        )
                        .clicked()
                        {
                            app.notice = None;
                        }
                    });
                });
            });
        ui.add_space(SPACE_S);
    }

    // Title
    if read_only {
        ui.label(
            RichText::new(if note.title.is_empty() {
                STRINGS.empty_title
            } else {
                &note.title
            })
            .size(19.0)
            .color(palette.text),
        );
    } else {
        let mut title = note.title.clone();
        let response = ui.add(
            egui::TextEdit::singleline(&mut title)
                .id(egui::Id::unique("note_title"))
                .frame(egui::Frame::NONE)
                .font(FontId::proportional(19.0))
                .hint_text(STRINGS.empty_title)
                .margin(margin_xy(0.0, SPACE_XS))
                .desired_width(f32::INFINITY)
                .interactive(!read_only),
        );
        if response.changed() {
            note.set_custom_title(title);
            changed = true;
        }
    }

    ui.label(
        RichText::new(format!(
            "{} · {} {}",
            format::date_en(note.created_at.with_timezone(&chrono::Local).date_naive()),
            STRINGS.modified_at,
            format::time_en(note.updated_at)
        ))
        .size(10.5)
        .color(palette.secondary),
    );
    ui.add_space(SPACE_L);

    if read_only {
        surface(&palette, palette.raised)
            .inner_margin(margin(SPACE_S))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(STRINGS.read_only_notice)
                        .size(11.0)
                        .color(palette.danger),
                );
            });
        ui.add_space(SPACE_S);
    }

    let single_block = note.blocks.len() == 1;
    egui::ScrollArea::vertical()
        .id_salt("editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for block in note.blocks.iter_mut() {
                match block {
                    NoteBlock::Text { id, text } => {
                        let response = ui.add(
                            egui::TextEdit::multiline(text)
                                .id(egui::Id::unique(*id))
                                .frame(egui::Frame::NONE)
                                .desired_rows(1)
                                .desired_width(f32::INFINITY)
                                .margin(margin_xy(0.0, SPACE_XS))
                                .interactive(!read_only)
                                .lock_focus(true)
                                .hint_text(if single_block {
                                    STRINGS.empty_state
                                } else {
                                    ""
                                }),
                        );
                        if response.changed() && !read_only {
                            changed = true;
                        }
                        if response.has_focus() || response.clicked() {
                            app.active_block = Some(*id);
                        }
                        if app.focus_block == Some(*id) {
                            response.request_focus();
                            app.focus_block = None;
                        }
                        app.block_rects.push((*id, response.rect));
                        ui.add_space(SPACE_XS);
                    }
                    NoteBlock::Image { id, asset_id, .. } => {
                        ui.add_space(SPACE_S);
                        let asset = app
                            .assets
                            .get(asset_id)
                            .filter(|asset| !app.images.has_failed(&asset.hash))
                            .cloned();
                        match asset {
                            Some(asset) => {
                                let max_width = ui.available_width().min(920.0);
                                let width = asset.width.max(1) as f32;
                                let height = asset.height.max(1) as f32;
                                let scale = (max_width / width).min(1.0);
                                let size = Vec2::new(width * scale, height * scale);
                                let framed = surface(&palette, palette.bg)
                                    .inner_margin(margin(0.0))
                                    .show(ui, |ui| {
                                        let (rect, response) =
                                            ui.allocate_exact_size(size, Sense::click());
                                        // Images off screen keep their place without being decoded.
                                        if ui.is_rect_visible(rect) {
                                            match app.images.texture(ui.ctx(), &app.root, &asset) {
                                                Some(texture) => {
                                                    egui::Image::new(
                                                        egui::load::SizedTexture::new(
                                                            texture.id(),
                                                            size,
                                                        ),
                                                    )
                                                    .corner_radius(0.0)
                                                    .paint_at(ui, rect);
                                                }
                                                None => ui.ctx().request_repaint(),
                                            }
                                        }
                                        response
                                    });
                                let response = framed.inner;
                                app.block_rects.push((*id, response.rect));
                                if response.clicked() {
                                    actions.push(Action::Preview(asset_id.clone()));
                                }
                                response.context_menu(|ui| {
                                    if widgets::text_button(
                                        ui,
                                        &palette,
                                        format!("[ {} ]", STRINGS.preview),
                                        palette.text,
                                    )
                                    .clicked()
                                    {
                                        actions.push(Action::Preview(asset_id.clone()));
                                        ui.close();
                                    }
                                    if widgets::text_button(
                                        ui,
                                        &palette,
                                        STRINGS.open,
                                        palette.text,
                                    )
                                    .clicked()
                                    {
                                        actions.push(Action::Open(asset_id.clone()));
                                        ui.close();
                                    }
                                    if !read_only
                                        && widgets::text_button(
                                            ui,
                                            &palette,
                                            STRINGS.remove,
                                            palette.danger,
                                        )
                                        .clicked()
                                    {
                                        actions.push(Action::RemoveBlock(*id));
                                        ui.close();
                                    }
                                });
                            }
                            None => {
                                surface(&palette, palette.raised)
                                    .inner_margin(margin(SPACE_S))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(format!("! {}", STRINGS.image_missing))
                                                .size(11.0)
                                                .color(palette.secondary),
                                        );
                                    });
                            }
                        }
                        ui.add_space(SPACE_S);
                    }
                }
            }

            for pending in app.pending.iter().filter(|p| p.note_id == note_id) {
                let _ = pending;
                ui.add_space(SPACE_XS);
                ui.label(
                    RichText::new(STRINGS.image_loading)
                        .size(10.5)
                        .color(palette.secondary),
                );
                ui.add_space(SPACE_XS);
            }
        });

    for action in actions {
        match action {
            Action::RemoveBlock(id) => {
                note.blocks.retain(|block| block.id() != id);
                changed = true;
            }
            Action::Preview(hash) => app.preview = Some(hash),
            Action::Open(hash) => {
                if let Some(asset) = app.assets.get(&hash) {
                    let path = asset.absolute_path(&app.root);
                    if let Err(error) = std::process::Command::new("open").arg(path).spawn() {
                        tracing::warn!(%error, "could not open image");
                    }
                }
            }
        }
    }

    app.commit_note(ui.ctx(), note, changed);

    if insert_time {
        app.insert_timestamp(ui.ctx());
    }
    if insert_image && !app.import_clipboard_image(ui.ctx()) {
        app.set_notice(ui.ctx(), STRINGS.image_hint);
    }
    if export_pdf {
        app.export_pdf(ui.ctx());
    }
    if request_delete {
        app.request_delete(note_id);
    }
}
