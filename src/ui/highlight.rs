//! Turns the pure classification of `core::highlight` into an egui layout job.

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Color32, FontId, Stroke};

use crate::core::highlight::{spans, Kind};
use crate::ui::theme::{Palette, BORDER};

/// Tinted layout of a text block. Glyph metrics are identical for every kind, so the caret and
/// the wrapping never move when a tint appears.
pub fn job(text: &str, palette: &Palette, font: &FontId, line_height: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    for span in spans(text) {
        let mut format = TextFormat {
            font_id: font.clone(),
            color: palette.text,
            line_height: Some(line_height),
            ..Default::default()
        };
        match span.kind {
            Kind::Plain => {}
            Kind::Stamp | Kind::TodoOpen => format.color = palette.stamp,
            Kind::Tag => format.color = palette.accent,
            Kind::Link => {
                format.color = palette.accent;
                format.underline = Stroke::new(BORDER, palette.accent);
            }
            Kind::Code => {
                format.color = palette.strong;
                format.background = palette.separator;
            }
            Kind::Heading => format.color = palette.strong,
            Kind::Quote | Kind::Bullet => format.color = palette.secondary,
            Kind::TodoDone => {
                format.color = palette.secondary;
                format.strikethrough = Stroke::new(BORDER, palette.secondary);
            }
        }
        job.append(&text[span.start..span.end], 0.0, format);
    }
    job
}

/// A single-line label where the characters in `range` (character indices) are emphasised.
pub fn matched(
    text: &str,
    range: Option<(usize, usize)>,
    size: f32,
    color: Color32,
    palette: &Palette,
) -> LayoutJob {
    let font = FontId::monospace(size);
    let plain = TextFormat {
        font_id: font.clone(),
        color,
        ..Default::default()
    };
    let mut job = LayoutJob::default();
    let Some((start, end)) = range.filter(|(start, end)| start < end) else {
        job.append(text, 0.0, plain);
        return job;
    };
    let byte = |index: usize| crate::core::models::byte_index_for_char(text, index);
    let (from, to) = (byte(start), byte(end));
    job.append(&text[..from], 0.0, plain.clone());
    job.append(
        &text[from..to],
        0.0,
        TextFormat {
            font_id: font,
            color: palette.stamp,
            underline: Stroke::new(BORDER, palette.stamp),
            ..Default::default()
        },
    );
    job.append(&text[to..], 0.0, plain);
    job
}
