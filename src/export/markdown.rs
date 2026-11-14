//! Markdown export for a note.
//!
//! Notes are already plain text, so the body is written as typed. Images become standard
//! `![…](…)` links: next to the `.md` file when exporting, or pointing at the asset store when the
//! Markdown is copied to the clipboard.

use std::collections::HashMap;
use std::path::Path;

use chrono::Local;

use super::pdf::ExportError;
use crate::core::assets::AssetRecord;
use crate::core::models::{Note, NoteBlock};

/// Where image links point.
pub enum ImageLinks<'a> {
    /// `<dir>/<short hash>.<ext>`, relative to the Markdown file.
    Folder(&'a str),
    /// Absolute paths into the asset store under this data root.
    Store(&'a Path),
}

/// File name of an exported image: short content hash plus the original extension.
fn image_name(asset: &AssetRecord) -> String {
    let extension = Path::new(&asset.rel_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("png");
    let short = asset.hash.get(..12).unwrap_or(&asset.hash);
    format!("{short}.{extension}")
}

/// Render a note as Markdown.
pub fn to_markdown(
    note: &Note,
    assets: &HashMap<String, AssetRecord>,
    links: &ImageLinks<'_>,
) -> String {
    let mut out = String::new();
    let title = note.title.trim();
    let first_line = note
        .blocks
        .iter()
        .filter_map(NoteBlock::as_text)
        .flat_map(str::lines)
        .map(str::trim)
        .find(|line| !line.is_empty());
    // An automatic title is the first line of the body: do not print it twice.
    let title_repeats_body = first_line.is_some_and(|line| line.trim_start_matches("# ") == title);
    if !title.is_empty() && !title_repeats_body {
        out.push_str(&format!("# {title}\n\n"));
    }
    out.push_str(&format!(
        "<!-- logpane · created {} · modified {} -->\n\n",
        note.created_at
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M"),
        note.updated_at
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M"),
    ));

    let mut image_number = 0;
    for block in &note.blocks {
        match block {
            NoteBlock::Text { text, .. } => {
                let text = text.trim_matches('\n').trim_end();
                if !text.is_empty() {
                    out.push_str(text);
                    out.push_str("\n\n");
                }
            }
            NoteBlock::Image {
                asset_id, caption, ..
            } => {
                image_number += 1;
                let alt = caption
                    .as_deref()
                    .map(str::trim)
                    .filter(|caption| !caption.is_empty())
                    .map(|caption| caption.replace(['[', ']'], ""))
                    .unwrap_or_else(|| format!("image {image_number}"));
                match assets.get(asset_id) {
                    Some(asset) => {
                        let target = match links {
                            ImageLinks::Folder(dir) => format!("{dir}/{}", image_name(asset)),
                            ImageLinks::Store(root) => {
                                asset.absolute_path(root).display().to_string()
                            }
                        };
                        // Angle brackets keep paths with spaces valid CommonMark.
                        out.push_str(&format!("![{alt}](<{target}>)\n\n"));
                    }
                    None => out.push_str(&format!("_[{alt}: not found]_\n\n")),
                }
            }
        }
    }
    let trimmed = out.trim_end().len();
    out.truncate(trimmed);
    out.push('\n');
    out
}

/// Write `<name>.md` and copy the note images into `<name>.assets/` next to it.
///
/// Returns how many images were copied.
pub fn export_markdown(
    note: &Note,
    assets: &HashMap<String, AssetRecord>,
    root: &Path,
    path: &Path,
) -> Result<usize, ExportError> {
    let write_error = |path: &Path, error: std::io::Error| ExportError::Write {
        path: path.display().to_string(),
        message: error.to_string(),
    };
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("logpane-note");
    let folder_name = format!("{stem}.assets");
    let folder = path.with_file_name(&folder_name);

    let mut copied = 0;
    for hash in note.asset_ids() {
        let Some(asset) = assets.get(&hash) else {
            continue;
        };
        if copied == 0 {
            std::fs::create_dir_all(&folder).map_err(|error| write_error(&folder, error))?;
        }
        let target = folder.join(image_name(asset));
        std::fs::copy(asset.absolute_path(root), &target)
            .map_err(|error| write_error(&target, error))?;
        copied += 1;
    }

    // Spaces are legal inside the angle-bracket link form used by `to_markdown`.
    let markdown = to_markdown(note, assets, &ImageLinks::Folder(&folder_name));
    std::fs::write(path, markdown).map_err(|error| write_error(path, error))?;
    Ok(copied)
}
