use eframe::egui::{self, Align, Layout, RichText, Sense};

use crate::app::LogpaneApp;
use crate::strings::STRINGS;
use crate::ui::theme::{SPACE_M, SPACE_S};
use crate::ui::widgets;

pub fn show(app: &mut LogpaneApp, ui: &mut egui::Ui) {
    let palette = app.palette;

    // Wide drag zone on the left; buttons on the right stay clickable because they are added
    // later in the same layer.
    let rect = ui.max_rect();
    let drag_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2((rect.max.x - 190.0).max(rect.min.x), rect.max.y),
    );
    let drag = ui.interact(drag_rect, egui::Id::unique("titlebar_drag"), Sense::drag());
    if drag.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    ui.horizontal_centered(|ui| {
        ui.label(
            RichText::new(STRINGS.app_name)
                .size(11.5)
                .color(palette.secondary),
        );

        ui.add_space(SPACE_M);

        let (label, color) = app.save_label();
        ui.label(RichText::new(format!("[{label}]")).size(10.5).color(color));

        if let Some(error) = app.format_save_error() {
            ui.add_space(SPACE_S);
            ui.label(RichText::new(error).size(10.5).color(palette.danger));
        }

        if app.hotkey_conflict {
            ui.add_space(SPACE_S);
            ui.label(
                RichText::new(format!("! {}", STRINGS.hotkey_conflict))
                    .size(10.5)
                    .color(palette.danger),
            );
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::text_button(ui, &palette, STRINGS.close_button, palette.secondary).clicked()
            {
                app.hide_window(ui.ctx());
            }

            if widgets::text_button(
                ui,
                &palette,
                format!("theme:{}", app.theme_mode.label()),
                palette.secondary,
            )
            .clicked()
            {
                app.cycle_theme(ui.ctx());
            }

            if !app.hotkey_conflict {
                ui.label(
                    RichText::new(&app.hotkey_label)
                        .size(10.5)
                        .color(palette.secondary),
                );
            }
        });
    });
}
