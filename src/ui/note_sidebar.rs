use eframe::egui::{self, Align, Layout, RichText, Sense, UiBuilder, Vec2};
use uuid::Uuid;

use crate::app::LogpaneApp;
use crate::core::models::truncate_chars;
use crate::core::search;
use crate::shortcuts;
use crate::strings::STRINGS;
use crate::ui::theme::{
    margin_xy, CONTROL_HEIGHT, FOOTER_HEIGHT, ROW_HEIGHT, SPACE_M, SPACE_S, SPACE_XS,
};
use crate::ui::widgets;
use crate::ui::{format, highlight};

pub fn show(app: &mut LogpaneApp, ui: &mut egui::Ui) {
    let palette = app.palette;

    let total = app.notes.len();
    let ids = app.filtered_ids();
    let query = app.search.trim().to_string();
    let searching = !query.is_empty();

    egui::Panel::bottom("sidebar_footer")
        .exact_size(FOOTER_HEIGHT)
        .show_separator_line(true)
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let unit = if total == 1 {
                    STRINGS.note_count
                } else {
                    STRINGS.notes_count
                };
                let count = if searching {
                    format!("{} / {total} {unit}", ids.len())
                } else {
                    format!("{total} {unit}")
                };
                ui.label(RichText::new(count).size(10.0).color(palette.secondary));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} help", shortcuts::HELP))
                            .size(10.0)
                            .color(palette.secondary),
                    );
                });
            });
        });

    widgets::section_label(ui, &palette, STRINGS.notes);
    ui.add_space(SPACE_XS);

    let search_id = egui::Id::unique("note_search");
    let search = ui.add_sized(
        [ui.available_width(), CONTROL_HEIGHT],
        egui::TextEdit::singleline(&mut app.search)
            .id(search_id)
            .hint_text(format!(
                "{} {}",
                STRINGS.search_placeholder,
                shortcuts::SEARCH
            ))
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
    let mut pin: Option<Uuid> = None;
    let scroll_to_active = std::mem::take(&mut app.scroll_to_active);

    egui::ScrollArea::vertical()
        .id_salt("notes_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut last_group: Option<String> = None;
            for id in ids {
                let Some(note) = app.note(id) else {
                    continue;
                };
                let is_pinned = !searching && app.is_pinned(id);
                let group = if is_pinned {
                    STRINGS.pinned.to_string()
                } else {
                    app.date_label(note.updated_at)
                };
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
                let images = note
                    .blocks
                    .iter()
                    .filter(|b| b.asset_id().is_some())
                    .count();
                let room = if images > 0 { 24 } else { 30 };
                let title_match = searching
                    .then(|| search::find(&title, &query))
                    .flatten()
                    .filter(|(_, end)| *end <= 24);
                // While searching, the excerpt moves to the match so the hit is visible.
                let (excerpt, excerpt_match) = searching
                    .then(|| search::snippet(note, &query, room))
                    .flatten()
                    .map(|hit| (hit.text, Some((hit.start, hit.end))))
                    .unwrap_or_else(|| (truncate_chars(&note.excerpt(room), room), None));
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
                // Stay inside the scroll area: a row must not paint over the search field.
                let clip = inner.intersect(ui.clip_rect());
                child.set_clip_rect(clip);
                child.set_max_width(inner.width());
                child.horizontal(|ui| {
                    if is_pinned {
                        ui.spacing_mut().item_spacing.x = SPACE_XS;
                        ui.label(
                            RichText::new(STRINGS.pin_mark)
                                .size(9.0)
                                .color(palette.stamp),
                        );
                    }
                    ui.label(highlight::matched(
                        &title,
                        title_match,
                        12.5,
                        if active { palette.strong } else { palette.text },
                        &palette,
                    ));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(time).size(10.0).color(palette.secondary));
                    });
                });
                child.label(highlight::matched(
                    &excerpt,
                    excerpt_match,
                    10.5,
                    palette.secondary,
                    &palette,
                ));
                if images > 0 {
                    ui.painter().with_clip_rect(clip).text(
                        inner.right_bottom(),
                        egui::Align2::RIGHT_BOTTOM,
                        format!("{images} {}", STRINGS.images),
                        egui::FontId::monospace(9.5),
                        palette.accent,
                    );
                }
                if active && scroll_to_active {
                    response.scroll_to_me(None);
                }

                response.widget_info(|| {
                    egui::WidgetInfo::selected(egui::Role::Button, true, active, &accessible_title)
                });
                if response.clicked() {
                    selected = Some(id);
                }
                let pin_label = if app.is_pinned(id) {
                    STRINGS.unpin
                } else {
                    STRINGS.pin
                };
                response.context_menu(|ui| {
                    if widgets::text_button(ui, &palette, pin_label, palette.text).clicked() {
                        pin = Some(id);
                        ui.close();
                    }
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
    if let Some(id) = pin {
        app.toggle_pin(ui.ctx(), id);
    }
    if let Some(id) = delete {
        app.request_delete(id);
    }
}
