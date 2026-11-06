use eframe::egui::{self, Align, Layout, RichText, Sense, UiBuilder, Vec2};
use uuid::Uuid;

use crate::app::LogpaneApp;
use crate::core::models::truncate_chars;
use crate::strings::STRINGS;
use crate::ui::format;
use crate::ui::theme::{margin_xy, CONTROL_HEIGHT, ROW_HEIGHT, SPACE_M, SPACE_S, SPACE_XS};
use crate::ui::widgets;

pub fn show(app: &mut LogpaneApp, ui: &mut egui::Ui) {
    let palette = app.palette;

    widgets::section_label(ui, &palette, STRINGS.notes);
    ui.add_space(SPACE_XS);

    let search_id = egui::Id::unique("note_search");
    let search = ui.add_sized(
        [ui.available_width(), CONTROL_HEIGHT],
        egui::TextEdit::singleline(&mut app.search)
            .id(search_id)
            .hint_text(STRINGS.search_placeholder)
            .margin(margin_xy(SPACE_S, SPACE_XS))
            .font(egui::TextStyle::Body),
    );
    if app.focus_search {
        search.request_focus();
        app.focus_search = false;
    }

    ui.add_space(SPACE_S);
    if widgets::wide_button(ui, STRINGS.new_note).clicked() {
        app.create_note(ui.ctx());
    }
    ui.add_space(SPACE_M);

    let ids = app.filtered_ids();
    if ids.is_empty() {
        ui.label(
            RichText::new(STRINGS.no_results)
                .size(11.5)
                .color(palette.secondary),
        );
        return;
    }

    let mut selected: Option<Uuid> = None;
    let mut delete: Option<Uuid> = None;

    egui::ScrollArea::vertical()
        .id_salt("notes_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut last_group: Option<String> = None;
            for id in ids {
                let Some(note) = app.note(id) else {
                    continue;
                };
                let group = app.date_label(note.updated_at);
                if last_group.as_deref() != Some(group.as_str()) {
                    ui.add_space(SPACE_M);
                    widgets::section_label(ui, &palette, &group);
                    ui.add_space(SPACE_XS);
                    last_group = Some(group);
                }

                let title = if note.title.trim().is_empty() {
                    STRINGS.empty_title.to_string()
                } else {
                    truncate_chars(&note.title, 24)
                };
                let accessible_title = title.clone();
                let excerpt = truncate_chars(&note.excerpt(30), 30);
                let time = format::time_en(note.updated_at);
                let active = id == app.active_id;

                let width = ui.available_width();
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click());

                let background = if active {
                    palette.raised
                } else if response.hovered() {
                    palette.hover
                } else {
                    egui::Color32::TRANSPARENT
                };
                if background != egui::Color32::TRANSPARENT {
                    ui.painter().rect_filled(rect, 0.0, background);
                }
                if active {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
                        0.0,
                        palette.accent,
                    );
                }
                ui.painter()
                    .hline(rect.x_range(), rect.bottom(), palette.rule());

                let inner = rect.shrink2(Vec2::new(SPACE_S + 2.0, SPACE_XS + 1.0));
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::top_down(Align::LEFT)),
                );
                child.set_clip_rect(inner);
                child.set_max_width(inner.width());
                child.horizontal(|ui| {
                    ui.label(RichText::new(title).size(12.5).color(palette.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(time).size(10.0).color(palette.secondary));
                    });
                });
                if !excerpt.is_empty() {
                    child.label(RichText::new(excerpt).size(10.5).color(palette.secondary));
                }

                response.widget_info(|| {
                    egui::WidgetInfo::selected(egui::Role::Button, true, active, &accessible_title)
                });
                if response.clicked() {
                    selected = Some(id);
                }
                response.context_menu(|ui| {
                    if widgets::text_button(ui, &palette, STRINGS.delete_note, palette.danger)
                        .clicked()
                    {
                        delete = Some(id);
                        ui.close();
                    }
                });
            }
        });

    if let Some(id) = selected {
        app.select_note(ui.ctx(), id);
    }
    if let Some(id) = delete {
        app.request_delete(id);
    }
}
