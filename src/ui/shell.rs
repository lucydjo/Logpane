use eframe::egui::{self, Align, Id, Layout, RichText};

use crate::app::LogpaneApp;
use crate::strings::STRINGS;
use crate::ui::theme::{
    margin, margin_xy, surface, HEADER_HEIGHT, SIDEBAR_MAX, SIDEBAR_MIN, SPACE_L, SPACE_M, SPACE_S,
    SPACE_XL, SPACE_XS,
};
use crate::ui::widgets;
use crate::{ui::editor, ui::header, ui::note_sidebar};

pub fn show(app: &mut LogpaneApp, ui: &mut egui::Ui) {
    let palette = app.palette;

    egui::Panel::top("logpane_header")
        .exact_size(HEADER_HEIGHT)
        .show_separator_line(true)
        .frame(
            egui::Frame::new()
                .fill(palette.panel)
                .inner_margin(margin_xy(SPACE_M, SPACE_XS)),
        )
        .show(ui, |ui| header::show(app, ui));

    let sidebar_width = (ui.available_width() * 0.25).clamp(SIDEBAR_MIN, SIDEBAR_MAX);
    egui::Panel::right("logpane_notes")
        .exact_size(sidebar_width)
        .resizable(false)
        .show_separator_line(true)
        .frame(
            egui::Frame::new()
                .fill(palette.panel)
                .inner_margin(margin(SPACE_M)),
        )
        .show(ui, |ui| note_sidebar::show(app, ui));

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(palette.bg)
                .inner_margin(egui::Margin {
                    left: SPACE_XL as i8,
                    right: SPACE_XL as i8,
                    top: SPACE_L as i8,
                    bottom: SPACE_M as i8,
                }),
        )
        .show(ui, |ui| editor::show(app, ui));

    let ctx = ui.ctx().clone();
    modals(app, &ctx);
    app.images.end_frame();
}

fn modals(app: &mut LogpaneApp, ctx: &egui::Context) {
    delete_modal(app, ctx);
    preview_modal(app, ctx);
}

fn delete_modal(app: &mut LogpaneApp, ctx: &egui::Context) {
    let Some(target) = app.delete_target else {
        return;
    };
    let title = app
        .note(target)
        .map(|note| {
            if note.title.trim().is_empty() {
                STRINGS.empty_title.to_string()
            } else {
                note.title.clone()
            }
        })
        .unwrap_or_default();
    let palette = app.palette;
    let mut close = false;
    let mut confirm = false;

    let response = egui::Modal::new(Id::unique("logpane_delete_modal"))
        .frame(surface(&palette, palette.panel).inner_margin(margin(SPACE_L)))
        .backdrop_color(egui::Color32::from_black_alpha(80))
        .show(ctx, |ui| {
            ui.set_width(320.0);
            ui.label(
                RichText::new(STRINGS.confirm_delete_title)
                    .size(14.0)
                    .color(palette.text),
            );
            ui.add_space(SPACE_S);
            ui.label(
                RichText::new(format!("« {title} »"))
                    .size(11.5)
                    .color(palette.text),
            );
            ui.label(
                RichText::new(STRINGS.confirm_delete_body)
                    .size(10.5)
                    .color(palette.secondary),
            );
            ui.add_space(SPACE_L);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::text_button(ui, &palette, STRINGS.delete, palette.danger).clicked() {
                    confirm = true;
                }
                if widgets::text_button(ui, &palette, STRINGS.cancel, palette.secondary).clicked() {
                    close = true;
                }
            });
        });

    if response.backdrop_response.clicked() {
        close = true;
    }
    if close {
        app.delete_target = None;
    }
    if confirm {
        app.confirm_delete(ctx);
    }
}

fn preview_modal(app: &mut LogpaneApp, ctx: &egui::Context) {
    let Some(hash) = app.preview.clone() else {
        return;
    };
    let Some(asset) = app.assets.get(&hash).cloned() else {
        app.preview = None;
        return;
    };
    let texture = app
        .images
        .texture(ctx, &app.root, &asset)
        .map(|texture| texture.id());
    let palette = app.palette;
    let mut close = false;
    let mut open = false;

    let response = egui::Modal::new(Id::unique("logpane_preview_modal"))
        .frame(surface(&palette, palette.panel).inner_margin(margin(SPACE_L)))
        .backdrop_color(egui::Color32::from_black_alpha(80))
        .show(ctx, |ui| {
            let available = ctx.content_rect().size();
            ui.set_width((available.x * 0.8).clamp(360.0, 860.0));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(STRINGS.preview)
                        .size(12.0)
                        .color(palette.text),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::text_button(ui, &palette, STRINGS.close_button, palette.secondary)
                        .clicked()
                    {
                        close = true;
                    }
                });
            });
            ui.add_space(SPACE_S);

            if let Some(texture) = texture {
                let max_width = ui.available_width();
                let max_height = (available.y * 0.62).max(240.0);
                let width = asset.width.max(1) as f32;
                let height = asset.height.max(1) as f32;
                let scale = (max_width / width).min(max_height / height).min(1.0);
                surface(&palette, palette.bg)
                    .inner_margin(margin(0.0))
                    .show(ui, |ui| {
                        ui.add(
                            egui::Image::new(egui::load::SizedTexture::new(
                                texture,
                                egui::Vec2::new(width * scale, height * scale),
                            ))
                            .corner_radius(0.0),
                        );
                    });
            }

            ui.add_space(SPACE_M);
            ui.horizontal(|ui| {
                if widgets::text_button(ui, &palette, STRINGS.open, palette.secondary).clicked() {
                    open = true;
                }
                ui.label(
                    RichText::new(format!(
                        "{} × {} · {}",
                        asset.width, asset.height, asset.mime
                    ))
                    .size(10.0)
                    .color(palette.secondary),
                );
            });
        });

    if response.backdrop_response.clicked() {
        close = true;
    }
    if open {
        let path = asset.absolute_path(&app.root);
        if let Err(error) = std::process::Command::new("open").arg(path).spawn() {
            tracing::warn!(%error, "could not open image");
        }
    }
    if close {
        app.preview = None;
    }
}
