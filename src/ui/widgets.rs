//! Shared widget helpers that keep the terminal language consistent.

use eframe::egui::{self, Color32, Response, RichText, Stroke, Ui};

use crate::ui::theme::{Palette, BORDER};

/// Flat bracketed label with a one-pixel accent underline on hover.
pub fn text_button(
    ui: &mut Ui,
    palette: &Palette,
    label: impl Into<String>,
    color: Color32,
) -> Response {
    let response =
        ui.add(egui::Button::new(RichText::new(label.into()).size(12.0).color(color)).frame(false));
    if response.hovered() {
        let rect = response.rect;
        ui.painter().hline(
            rect.x_range(),
            rect.bottom() + 1.0,
            Stroke::new(BORDER, palette.accent),
        );
    }
    response
}

/// Square, thin-ruled full-width button used for primary list actions.
pub fn wide_button(ui: &mut Ui, label: impl Into<String>) -> Response {
    let width = ui.available_width();
    ui.add_sized(
        [width, 28.0],
        egui::Button::new(RichText::new(label.into()).size(12.0)),
    )
}

/// Small uppercase section label.
pub fn section_label(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(RichText::new(text).size(10.0).color(palette.secondary));
}
