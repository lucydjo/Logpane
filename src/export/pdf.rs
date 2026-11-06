//! PDF export for a note.
//!
//! The PDF keeps a real text layer (selectable and extractable, so an LLM or `pdftotext` can read
//! it) and embeds the images in reading order. Text is laid out in a single column on A4, with a
//! monospace system font and page breaks. Characters that font lacks use a fallback system font;
//! those no font can draw, such as emoji, are replaced by a visible mark and counted, never dropped
//! silently.

use std::collections::HashMap;
use std::path::Path;

use crate::core::assets::AssetRecord;
use crate::core::models::{Note, NoteBlock};
use printpdf::{
    Color, ImageCompression, ImageOptimizationOptions, Mm, Op, ParsedFont, PdfDocument,
    PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, RawImage, Rgb, TextItem, XObjectTransform,
};

const PAGE_WIDTH: f32 = 595.28; // A4, points
const PAGE_HEIGHT: f32 = 841.89;
const MARGIN: f32 = 51.0; // 18 mm
const FOOTER_BASELINE: f32 = 22.0;
const FOOTER_RESERVE: f32 = 34.0;
const CONTENT_WIDTH: f32 = PAGE_WIDTH - 2.0 * MARGIN;

const BODY_SIZE: f32 = 10.0;
const BODY_LINE: f32 = 13.6;
const BODY_CHARS: usize = 80;
const TITLE_SIZE: f32 = 17.0;
const TITLE_LINE: f32 = 21.0;
const TITLE_CHARS: usize = 47;
const META_SIZE: f32 = 8.5;
const BLOCK_GAP: f32 = 9.0;
/// Fallback glyphs are up to one em wide, which fits in two monospace columns.
const FALLBACK_COLUMNS: usize = 2;
const MISSING_GLYPH_MARK: char = '□';

/// Covers CJK, Arabic, Hebrew, Thai and Devanagari. Not emoji: macOS only has them as color bitmaps.
pub const FALLBACK_FONT_PATH: &str = "/System/Library/Fonts/Supplemental/Arial Unicode.ttf";

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("no usable system font: {0}")]
    NoFont(String),
    #[error("could not create the PDF: {0}")]
    Pdf(String),
    #[error("could not write {path}: {message}")]
    Write { path: String, message: String },
}

/// Export a note to a PDF file. The images are read from the content-addressed asset store.
///
/// Returns how many characters could not be drawn and were replaced by a visible mark.
pub fn export_note(
    note: &Note,
    assets: &HashMap<String, AssetRecord>,
    root: &Path,
    path: &Path,
) -> Result<usize, ExportError> {
    let (bytes, replaced) = build_pdf(note, assets, root)?;
    std::fs::write(path, bytes).map_err(|error| ExportError::Write {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(replaced)
}

/// Build the PDF in memory. Used by the exporter and by the tests.
pub fn export_note_bytes(
    note: &Note,
    assets: &HashMap<String, AssetRecord>,
    root: &Path,
) -> Result<Vec<u8>, ExportError> {
    build_pdf(note, assets, root).map(|(bytes, _)| bytes)
}

fn build_pdf(
    note: &Note,
    assets: &HashMap<String, AssetRecord>,
    root: &Path,
) -> Result<(Vec<u8>, usize), ExportError> {
    let font = load_system_font()?;
    // The fallback is a large file: only read it for the notes that need it.
    let fallback = needs_fallback(note, &font)
        .then(|| load_font(FALLBACK_FONT_PATH).ok().flatten())
        .flatten();
    let title = if note.title.trim().is_empty() {
        "Logpane note"
    } else {
        note.title.trim()
    };

    let mut document = PdfDocument::new(title);
    let now = printpdf::OffsetDateTime::now();
    document.metadata.info.creation_date = now;
    document.metadata.info.modification_date = now;
    document.metadata.info.metadata_date = now;
    document.metadata.info.author = "Logpane".to_string();
    document.metadata.info.creator = "Logpane".to_string();
    document.metadata.info.producer = "Logpane".to_string();
    document.metadata.info.subject = "Exported note".to_string();
    document.metadata.info.keywords = vec!["note".to_string(), "logpane".to_string()];
    let primary_handle = PdfFontHandle::External(document.add_font(&font));
    let fallback = fallback.map(|fallback| {
        let handle = PdfFontHandle::External(document.add_font(&fallback));
        (fallback, handle)
    });
    let mut builder = Builder::new(document, font, primary_handle, fallback);

    builder.paragraph(title, TITLE_SIZE, TITLE_LINE, TITLE_CHARS, black());
    builder.gap(2.0);
    builder.metadata(note);
    builder.gap(11.0);

    for block in &note.blocks {
        match block {
            NoteBlock::Text { text, .. } => {
                builder.text_block(text);
                builder.gap(BLOCK_GAP);
            }
            NoteBlock::Image { asset_id, .. } => {
                if let Some(asset) = assets.get(asset_id) {
                    builder.image(asset, root);
                } else {
                    builder.placeholder("[image not found]");
                }
                builder.gap(BLOCK_GAP);
            }
        }
    }

    Ok(builder.finish())
}

fn load_system_font() -> Result<ParsedFont, ExportError> {
    let mut last_error = String::new();
    for path in [
        "/System/Library/Fonts/SFNSMono.ttf",
        "/System/Library/Fonts/Menlo.ttc",
    ] {
        match load_font(path) {
            Ok(Some(font)) => return Ok(font),
            Ok(None) => last_error = format!("{path}: unsupported font format"),
            Err(_) => continue,
        }
    }
    Err(ExportError::NoFont(last_error))
}

fn load_font(path: &str) -> std::io::Result<Option<ParsedFont>> {
    let bytes = std::fs::read(path)?;
    let mut warnings = Vec::new();
    Ok(ParsedFont::from_bytes(&bytes, 0, &mut warnings))
}

fn has_glyph(font: &ParsedFont, ch: char) -> bool {
    font.lookup_glyph_index(ch as u32)
        .is_some_and(|glyph| glyph != 0)
}

/// Joiners and variation selectors have no appearance of their own.
fn is_invisible(ch: char) -> bool {
    matches!(ch, '\u{200B}'..='\u{200D}' | '\u{2060}' | '\u{FE00}'..='\u{FE0F}' | '\u{FEFF}')
}

fn needs_fallback(note: &Note, font: &ParsedFont) -> bool {
    let missing = |ch: char| !ch.is_control() && !is_invisible(ch) && !has_glyph(font, ch);
    note.title.chars().any(missing)
        || note
            .blocks
            .iter()
            .filter_map(NoteBlock::as_text)
            .any(|text| text.chars().any(missing))
}

fn wrap_columns(primary: &ParsedFont, ch: char) -> usize {
    if is_invisible(ch) || has_glyph(primary, ch) {
        1
    } else {
        FALLBACK_COLUMNS
    }
}

struct Builder {
    document: PdfDocument,
    primary: ParsedFont,
    font: PdfFontHandle,
    fallback: Option<(ParsedFont, PdfFontHandle)>,
    replaced: usize,
    pages: Vec<PdfPage>,
    ops: Vec<Op>,
    /// Current baseline, measured from the bottom of the page.
    y: f32,
    page_number: usize,
}

impl Builder {
    fn new(
        document: PdfDocument,
        primary: ParsedFont,
        font: PdfFontHandle,
        fallback: Option<(ParsedFont, PdfFontHandle)>,
    ) -> Self {
        let mut builder = Self {
            document,
            primary,
            font,
            fallback,
            replaced: 0,
            pages: Vec::new(),
            ops: Vec::new(),
            y: PAGE_HEIGHT - MARGIN,
            page_number: 0,
        };
        builder.start_page();
        builder
    }

    fn start_page(&mut self) {
        self.page_number += 1;
        self.y = PAGE_HEIGHT - MARGIN;
        self.ops.clear();
    }

    fn ensure_space(&mut self, height: f32) {
        if self.y - height < MARGIN + FOOTER_RESERVE {
            self.finish_page();
            self.start_page();
        }
    }

    fn finish_page(&mut self) {
        if self.ops.is_empty() {
            return;
        }
        let footer = format!("logpane · page {}", self.page_number);
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb::new(0.45, 0.45, 0.45, None)),
        });
        self.ops.push(Op::SetFont {
            font: self.font.clone(),
            size: Pt(7.5),
        });
        self.ops.push(Op::SetTextCursor {
            pos: Point::new(mm(MARGIN), mm(FOOTER_BASELINE)),
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(footer)],
        });
        self.ops.push(Op::EndTextSection);

        let ops = std::mem::take(&mut self.ops);
        self.pages.push(PdfPage::new(Mm(210.0), Mm(297.0), ops));
    }

    fn text_line(&mut self, text: &str, size: f32, line_height: f32, color: Color) {
        self.ensure_space(line_height);
        let baseline = self.y - size;
        let runs = self.font_runs(&visual_order(text));
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFillColor { col: color });
        self.ops.push(Op::SetTextCursor {
            pos: Point::new(mm(MARGIN), mm(baseline)),
        });
        for (font, run) in runs {
            self.ops.push(Op::SetFont {
                font,
                size: Pt(size),
            });
            self.ops.push(Op::ShowText {
                items: vec![TextItem::Text(run)],
            });
        }
        self.ops.push(Op::EndTextSection);
        self.y -= line_height;
    }

    /// Split a line into runs that a single font can draw.
    fn font_runs(&mut self, text: &str) -> Vec<(PdfFontHandle, String)> {
        let mark = if has_glyph(&self.primary, MISSING_GLYPH_MARK) {
            MISSING_GLYPH_MARK
        } else {
            '?'
        };
        let mut runs: Vec<(PdfFontHandle, String)> = Vec::new();
        for ch in text.chars().filter(|ch| !is_invisible(*ch)) {
            let (font, drawn) = if has_glyph(&self.primary, ch) {
                (&self.font, ch)
            } else if let Some((_, handle)) = self
                .fallback
                .as_ref()
                .filter(|(font, _)| has_glyph(font, ch))
            {
                (handle, ch)
            } else {
                self.replaced += 1;
                (&self.font, mark)
            };
            match runs.last_mut() {
                Some((current, run)) if current == font => run.push(drawn),
                _ => runs.push((font.clone(), drawn.to_string())),
            }
        }
        runs
    }

    fn paragraph(
        &mut self,
        text: &str,
        size: f32,
        line_height: f32,
        max_chars: usize,
        color: Color,
    ) {
        for line in wrap(text, max_chars, &|ch| wrap_columns(&self.primary, ch)) {
            self.text_line(&line, size, line_height, color.clone());
        }
    }

    fn metadata(&mut self, note: &Note) {
        let created = note
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M");
        let updated = note
            .updated_at
            .with_timezone(&chrono::Local)
            .format("%H:%M");
        let line = format!("created {created} · modified {updated}");
        self.text_line(
            &line,
            META_SIZE,
            META_SIZE + 4.0,
            Color::Rgb(Rgb::new(0.42, 0.42, 0.42, None)),
        );
    }

    fn text_block(&mut self, text: &str) {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let lines: Vec<&str> = normalized.split('\n').collect();
        for line in lines {
            if line.trim().is_empty() {
                self.ensure_space(BODY_LINE * 0.55);
                self.y -= BODY_LINE * 0.55;
                continue;
            }
            let wrapped_lines = wrap(line.trim_end(), BODY_CHARS, &|ch| {
                wrap_columns(&self.primary, ch)
            });
            for wrapped in wrapped_lines {
                self.text_line(&wrapped, BODY_SIZE, BODY_LINE, black());
            }
        }
        if text.is_empty() {
            self.ensure_space(BODY_LINE);
            self.y -= BODY_LINE;
        }
    }

    fn placeholder(&mut self, text: &str) {
        self.text_line(
            text,
            BODY_SIZE,
            BODY_LINE,
            Color::Rgb(Rgb::new(0.55, 0.55, 0.55, None)),
        );
    }

    fn image(&mut self, asset: &AssetRecord, root: &Path) {
        let path = asset.absolute_path(root);
        let Some(image) = load_pdf_image(&path) else {
            self.placeholder("[image could not be decoded]");
            return;
        };
        let image_id = self.document.add_image(&image);

        // Images are placed at 300 dpi by printpdf, then scaled to fit the column.
        let natural_width = image.width as f32 * 72.0 / 300.0;
        let natural_height = image.height as f32 * 72.0 / 300.0;
        if natural_width <= 0.0 || natural_height <= 0.0 {
            self.placeholder("[image has no dimensions]");
            return;
        }
        let max_height = PAGE_HEIGHT - 2.0 * MARGIN - FOOTER_RESERVE;
        let scale = (CONTENT_WIDTH / natural_width)
            .min(max_height / natural_height)
            .min(2.0);
        let width = natural_width * scale;
        let height = natural_height * scale;

        self.ensure_space(height + 6.0);
        let x = MARGIN + (CONTENT_WIDTH - width) / 2.0;
        let y = self.y - height;
        self.ops.push(Op::UseXobject {
            id: image_id,
            transform: XObjectTransform {
                translate_x: Some(Pt(x)),
                translate_y: Some(Pt(y)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(300.0),
                ..Default::default()
            },
        });
        self.y = y - 6.0;
    }

    fn gap(&mut self, height: f32) {
        self.ensure_space(height);
        self.y -= height;
    }

    fn finish(mut self) -> (Vec<u8>, usize) {
        self.finish_page();
        let pages = std::mem::take(&mut self.pages);
        self.document.with_pages(pages);
        let mut warnings = Vec::new();
        let options = PdfSaveOptions {
            // Screenshots contain text: keep full resolution and lossless compression.
            image_optimization: Some(ImageOptimizationOptions {
                quality: None,
                max_image_size: None,
                dither_greyscale: Some(false),
                convert_to_greyscale: Some(false),
                auto_optimize: Some(false),
                format: Some(ImageCompression::Flate),
            }),
            ..PdfSaveOptions::default()
        };
        let bytes = self.document.save(&options, &mut warnings);
        if !warnings.is_empty() {
            tracing::debug!(count = warnings.len(), "PDF export warnings");
        }
        (bytes, self.replaced)
    }
}

fn is_right_to_left(ch: char) -> bool {
    matches!(ch, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFC}')
}

/// PDF text is drawn left to right, so right-to-left passages are stored in the order they appear
/// on the page. Readers and text extractors then recover the logical order from the positions.
fn visual_order(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut output = String::with_capacity(line.len());
    let mut index = 0;
    while index < chars.len() {
        if !is_right_to_left(chars[index]) {
            output.push(chars[index]);
            index += 1;
            continue;
        }
        // A passage runs up to its last right-to-left character, spaces between words included.
        let mut end = index;
        let mut cursor = index;
        while cursor < chars.len() && (is_right_to_left(chars[cursor]) || chars[cursor] == ' ') {
            if is_right_to_left(chars[cursor]) {
                end = cursor;
            }
            cursor += 1;
        }
        output.extend(chars[index..=end].iter().rev());
        index = end + 1;
    }
    output
}

/// Wrap a line to a number of columns. Primary glyphs take one column, so wrapping is
/// deterministic; `columns` gives the width of each character.
fn wrap(text: &str, max_columns: usize, columns: &impl Fn(char) -> usize) -> Vec<String> {
    let expanded = text.replace('\t', "    ");
    let width = |value: &str| value.chars().map(columns).sum::<usize>();
    if width(&expanded) <= max_columns {
        return vec![expanded];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in expanded.split(' ') {
        if word.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            continue;
        }
        if !current.is_empty() && width(&current) + 1 + width(word) > max_columns {
            lines.push(std::mem::take(&mut current));
        }
        if width(word) > max_columns {
            let mut chunk = String::new();
            let mut used = 0;
            for ch in word.chars() {
                let needed = columns(ch);
                if used + needed > max_columns && !chunk.is_empty() {
                    lines.push(std::mem::take(&mut chunk));
                    used = 0;
                }
                chunk.push(ch);
                used += needed;
            }
            if !chunk.is_empty() {
                lines.push(chunk);
            }
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Decode an image for print, bounded to a sharp but memory-safe size: 2400 px on the long
/// side is enough for an A4 page at 300 dpi and prevents a 40 MP import from blowing up the
/// export.
fn load_pdf_image(path: &Path) -> Option<RawImage> {
    let reader = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?;
    let mut reader = reader;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode().ok()?;
    let (width, height) = (decoded.width(), decoded.height());
    if width == 0 || height == 0 {
        return None;
    }
    let longest = width.max(height);
    let image = if longest > 2400 {
        let scale = 2400.0 / longest as f32;
        let target_w = ((width as f32 * scale).round() as u32).max(1);
        let target_h = ((height as f32 * scale).round() as u32).max(1);
        decoded.resize(target_w, target_h, image::imageops::FilterType::Lanczos3)
    } else {
        decoded
    };
    let rgb = flatten_on_white(image.to_rgba8());
    RawImage::from_dynamic_image(image::DynamicImage::ImageRgb8(rgb)).ok()
}

fn flatten_on_white(source: image::RgbaImage) -> image::RgbImage {
    let (width, height) = source.dimensions();
    let mut output = image::RgbImage::new(width, height);
    for (destination, pixel) in output.pixels_mut().zip(source.pixels()) {
        let [r, g, b, a] = pixel.0;
        let alpha = a as u16;
        let blend = |channel: u8| ((channel as u16 * alpha + 255 * (255 - alpha)) / 255) as u8;
        *destination = image::Rgb([blend(r), blend(g), blend(b)]);
    }
    output
}

/// A safe default filename derived from the note title.
pub fn default_filename(note: &Note) -> String {
    let base = if note.title.trim().is_empty() {
        "logpane-note"
    } else {
        note.title.trim()
    };
    let sanitized: String = base
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '\n' | '\r' | '\0' => '-',
            other => other,
        })
        .take(80)
        .collect();
    let sanitized = sanitized.trim().trim_matches('.').trim();
    if sanitized.is_empty() {
        "logpane-note.pdf".to_string()
    } else if sanitized.to_ascii_lowercase().ends_with(".pdf") {
        sanitized.to_string()
    } else {
        format!("{sanitized}.pdf")
    }
}

fn mm(points: f32) -> Mm {
    Mm(points * 25.4 / 72.0)
}

fn black() -> Color {
    Color::Rgb(Rgb::new(0.10, 0.10, 0.10, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cjk_is_wide(ch: char) -> usize {
        if ch.is_ascii() {
            1
        } else {
            FALLBACK_COLUMNS
        }
    }

    #[test]
    fn right_to_left_passages_are_stored_in_visual_order() {
        assert_eq!(visual_order("plain text"), "plain text");
        assert_eq!(visual_order("a שלום b"), "a םולש b");
        assert_eq!(visual_order("a שלום עולם b"), "a םלוע םולש b");
    }

    #[test]
    fn wrap_keeps_short_lines_and_breaks_on_spaces() {
        assert_eq!(wrap("a b", 10, &cjk_is_wide), vec!["a b"]);
        assert_eq!(wrap("aaaa bbbb", 5, &cjk_is_wide), vec!["aaaa", "bbbb"]);
    }

    #[test]
    fn wrap_counts_wide_characters_and_loses_none() {
        let text = "日本語のテキスト";
        let lines = wrap(text, 6, &cjk_is_wide);
        assert_eq!(lines, vec!["日本語", "のテキ", "スト"]);
        assert_eq!(lines.concat(), text);
    }
}
