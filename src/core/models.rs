//! Pure domain model: notes, blocks and the versioned document envelope.
//!
//! This module must stay independent from egui and from AppKit so it can be tested headlessly.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_TITLE_CHARS: usize = 200;

/// One ordered element of a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NoteBlock {
    Text {
        id: Uuid,
        #[serde(default)]
        text: String,
    },
    Image {
        id: Uuid,
        asset_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },
}

impl NoteBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            id: Uuid::new_v4(),
            text: text.into(),
        }
    }

    pub fn image(asset_id: impl Into<String>) -> Self {
        Self::Image {
            id: Uuid::new_v4(),
            asset_id: asset_id.into(),
            caption: None,
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Self::Text { id, .. } | Self::Image { id, .. } => *id,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text, .. } => Some(text),
            Self::Image { .. } => None,
        }
    }

    pub fn as_text_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Text { text, .. } => Some(text),
            Self::Image { .. } => None,
        }
    }

    pub fn asset_id(&self) -> Option<&str> {
        match self {
            Self::Image { asset_id, .. } => Some(asset_id),
            Self::Text { .. } => None,
        }
    }

    pub fn is_empty_text(&self) -> bool {
        matches!(self, Self::Text { text, .. } if text.trim().is_empty())
    }
}

/// Versioned wrapper stored in `notes.blocks_json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentEnvelope {
    pub schema_version: u32,
    pub blocks: Vec<NoteBlock>,
}

impl DocumentEnvelope {
    pub fn new(blocks: Vec<NoteBlock>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            blocks,
        }
    }

    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(error) => {
                tracing::error!(%error, "could not serialize document");
                String::from(
                    r#"{"schema_version":1,"blocks":[{"type":"text","id":"00000000-0000-0000-0000-000000000000","text":""}]}"#,
                )
            }
        }
    }

    /// Parse a stored document. Unknown future versions are rejected, never rewritten.
    pub fn from_json(raw: &str) -> Result<Self, DocumentError> {
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| DocumentError::Malformed(e.to_string()))?;
        let version = value
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| DocumentError::Malformed("missing schema_version".into()))?;
        if version > SCHEMA_VERSION as u64 {
            return Err(DocumentError::UnsupportedVersion(version as u32));
        }
        let mut envelope: Self =
            serde_json::from_value(value).map_err(|e| DocumentError::Malformed(e.to_string()))?;
        envelope.schema_version = version as u32;
        Ok(envelope)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error("document JSON is malformed: {0}")]
    Malformed(String),
    #[error("document schema version {0} is newer than this application")]
    UnsupportedVersion(u32),
}

/// A note is one experiment or topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub title_custom: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub revision: i64,
    pub blocks: Vec<NoteBlock>,
}

impl Note {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            title_custom: false,
            created_at: now,
            updated_at: now,
            revision: 0,
            blocks: vec![NoteBlock::text("")],
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self.title_custom = !self.title.is_empty();
        self
    }

    /// Mark the note as modified. The next persisted revision is assigned by the save pipeline.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Concatenate every text block, used by search and title derivation.
    pub fn plain_text(&self) -> String {
        self.blocks
            .iter()
            .filter_map(NoteBlock::as_text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Bounded excerpt: never concatenates the whole note just to show a list row.
    pub fn excerpt(&self, max_chars: usize) -> String {
        let mut out = String::new();
        let mut remaining = max_chars;
        let mut truncated = false;
        for block in &self.blocks {
            let Some(text) = block.as_text() else {
                continue;
            };
            for ch in text.trim().chars() {
                if remaining == 0 {
                    truncated = true;
                    break;
                }
                out.push(if ch == '\n' { ' ' } else { ch });
                remaining -= 1;
            }
            if truncated {
                break;
            }
            if !out.is_empty() && remaining > 0 {
                out.push(' ');
                remaining -= 1;
            }
        }
        if truncated {
            out.push('…');
        }
        out.trim().to_string()
    }

    /// Derive the title from the first non-empty line while the title is automatic.
    pub fn refresh_auto_title(&mut self) {
        if self.title_custom {
            return;
        }
        let derived = self
            .blocks
            .iter()
            .filter_map(NoteBlock::as_text)
            .flat_map(str::lines)
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(|line| truncate_chars(line, MAX_TITLE_CHARS))
            .unwrap_or_default();
        self.title = derived;
    }

    pub fn set_custom_title(&mut self, title: impl Into<String>) {
        self.title = truncate_chars(title.into().trim(), MAX_TITLE_CHARS);
        self.title_custom = true;
        self.touch();
    }

    pub fn asset_ids(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.blocks
            .iter()
            .filter_map(NoteBlock::asset_id)
            .filter(|id| seen.insert((*id).to_string()))
            .map(str::to_string)
            .collect()
    }

    pub fn text_block(&self, id: Uuid) -> Option<&str> {
        self.blocks
            .iter()
            .find(|b| b.id() == id)
            .and_then(NoteBlock::as_text)
    }

    pub fn text_block_mut(&mut self, id: Uuid) -> Option<&mut String> {
        self.blocks
            .iter_mut()
            .find(|b| b.id() == id)
            .and_then(NoteBlock::as_text_mut)
    }

    pub fn index_of(&self, id: Uuid) -> Option<usize> {
        self.blocks.iter().position(|b| b.id() == id)
    }

    /// Insert a block after `anchor`, or at the end when the anchor is unknown.
    pub fn insert_block_after(&mut self, anchor: Option<Uuid>, block: NoteBlock) -> usize {
        let index = anchor
            .and_then(|id| self.index_of(id))
            .map(|i| i + 1)
            .unwrap_or(self.blocks.len());
        self.blocks.insert(index, block);
        self.touch();
        index
    }

    /// Split a text block at a character index and place an image between the halves.
    ///
    /// Returns the id of the text block that follows the image, so the caret can be restored.
    pub fn insert_image_at(
        &mut self,
        block_id: Uuid,
        char_index: usize,
        asset_id: impl Into<String>,
    ) -> Option<Uuid> {
        let index = self.index_of(block_id)?;
        let text = match &self.blocks[index] {
            NoteBlock::Text { text, .. } => text.clone(),
            NoteBlock::Image { .. } => return None,
        };
        let byte = byte_index_for_char(&text, char_index);
        let (before, after) = text.split_at(byte);
        let before = before.to_string();
        let after = after.to_string();

        let after_id = Uuid::new_v4();
        self.blocks[index] = NoteBlock::Text {
            id: block_id,
            text: before,
        };
        self.blocks
            .insert(index + 1, NoteBlock::image(asset_id.into()));
        if after.is_empty() {
            self.blocks.insert(
                index + 2,
                NoteBlock::Text {
                    id: after_id,
                    text: String::new(),
                },
            );
        } else {
            self.blocks.insert(
                index + 2,
                NoteBlock::Text {
                    id: after_id,
                    text: after,
                },
            );
        }
        self.touch();
        Some(after_id)
    }

    /// Insert `HH:mm — ` at the caret of a text block.
    pub fn insert_text_at(
        &mut self,
        block_id: Uuid,
        char_index: usize,
        insertion: &str,
    ) -> Option<usize> {
        let text = self.text_block_mut(block_id)?;
        let byte = byte_index_for_char(text, char_index);
        text.insert_str(byte, insertion);
        let new_index = char_index + insertion.chars().count();
        self.touch();
        Some(new_index)
    }

    /// Append a text block after `anchor` (used when no text block holds the caret).
    pub fn append_text_block(&mut self, anchor: Option<Uuid>, text: impl Into<String>) -> Uuid {
        let block = NoteBlock::text(text);
        let id = block.id();
        self.insert_block_after(anchor, block);
        id
    }

    /// Collapse consecutive empty text blocks and guarantee an editable text block after an
    /// image, so inserting at the end keeps a caret target.
    pub fn normalize(&mut self) {
        let mut index = 1;
        while index < self.blocks.len() {
            if self.blocks[index].is_empty_text() && self.blocks[index - 1].is_empty_text() {
                self.blocks.remove(index);
            } else {
                index += 1;
            }
        }
        if matches!(self.blocks.last(), Some(NoteBlock::Image { .. })) {
            self.blocks.push(NoteBlock::text(""));
        }
        if !self.blocks.iter().any(|b| b.as_text().is_some()) {
            self.blocks.push(NoteBlock::text(""));
        }
        if self.blocks.is_empty() {
            self.blocks.push(NoteBlock::text(""));
        }
    }

    /// True when every image block references a plausible content hash and no image block
    /// carries a filesystem path. User text is never inspected (it may legitimately mention
    /// a path).
    pub fn images_are_content_addressed(&self) -> bool {
        self.blocks.iter().all(|block| match block {
            NoteBlock::Image { asset_id, .. } => {
                asset_id.len() == 64
                    && asset_id.chars().all(|c| c.is_ascii_hexdigit())
                    && !asset_id.contains('/')
            }
            NoteBlock::Text { .. } => true,
        })
    }
}

impl Default for Note {
    fn default() -> Self {
        Self::new()
    }
}

pub fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut out: String = text.chars().take(max).collect();
        out.push('…');
        out
    }
}

/// Convert a character index into a byte index on a char boundary.
pub fn byte_index_for_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_note_has_one_empty_text_block() {
        let note = Note::new();
        assert_eq!(note.blocks.len(), 1);
        assert_eq!(note.revision, 0);
        assert!(note.title.is_empty());
    }

    #[test]
    fn auto_title_uses_first_non_empty_line() {
        let mut note = Note::new();
        note.blocks = vec![NoteBlock::text("\n\n  Test GPT  \nDeuxième ligne")];
        note.refresh_auto_title();
        assert_eq!(note.title, "Test GPT");
    }

    #[test]
    fn custom_title_is_never_overwritten() {
        let mut note = Note::new();
        note.set_custom_title("Mon titre");
        note.blocks = vec![NoteBlock::text("Autre chose")];
        note.refresh_auto_title();
        assert_eq!(note.title, "Mon titre");
    }

    #[test]
    fn split_text_block_at_caret_inserts_image_between_halves() {
        let mut note = Note::new();
        let block_id = note.blocks[0].id();
        note.blocks[0] = NoteBlock::Text {
            id: block_id,
            text: "abcdef".into(),
        };
        let after = note
            .insert_image_at(block_id, 3, "a".repeat(64))
            .expect("split");
        assert_eq!(note.blocks.len(), 3);
        assert_eq!(note.blocks[0].as_text(), Some("abc"));
        assert!(note.blocks[1].asset_id().is_some());
        assert_eq!(note.blocks[2].id(), after);
        assert_eq!(note.blocks[2].as_text(), Some("def"));
    }

    #[test]
    fn empty_after_half_is_kept_for_caret() {
        let mut note = Note::new();
        let block_id = note.blocks[0].id();
        note.blocks[0] = NoteBlock::Text {
            id: block_id,
            text: "abc".into(),
        };
        let after = note.insert_image_at(block_id, 3, "b".repeat(64)).unwrap();
        assert_eq!(note.blocks[2].id(), after);
        assert_eq!(note.blocks[2].as_text(), Some(""));
    }

    #[test]
    fn unicode_split_respects_char_boundaries() {
        let mut note = Note::new();
        let block_id = note.blocks[0].id();
        note.blocks[0] = NoteBlock::Text {
            id: block_id,
            text: "aé👍z".into(),
        };
        note.insert_image_at(block_id, 2, "c".repeat(64)).unwrap();
        assert_eq!(note.blocks[0].as_text(), Some("aé"));
        assert_eq!(note.blocks[2].as_text(), Some("👍z"));
    }

    #[test]
    fn envelope_round_trip() {
        let note = Note::new();
        let envelope = DocumentEnvelope::new(note.blocks.clone());
        let json = envelope.to_json();
        let parsed = DocumentEnvelope::from_json(&json).unwrap();
        assert_eq!(parsed.blocks, note.blocks);
    }

    #[test]
    fn future_schema_is_rejected() {
        let err = DocumentEnvelope::from_json(r#"{"schema_version":99,"blocks":[]}"#).unwrap_err();
        assert!(matches!(err, DocumentError::UnsupportedVersion(99)));
    }

    #[test]
    fn image_blocks_must_be_content_addressed() {
        let mut note = Note::new();
        note.blocks.push(NoteBlock::Image {
            id: Uuid::new_v4(),
            asset_id: "/tmp/shot.png".into(),
            caption: None,
        });
        assert!(!note.images_are_content_addressed());
    }

    #[test]
    fn normalize_keeps_a_text_block_after_a_trailing_image() {
        let mut note = Note::new();
        note.blocks = vec![NoteBlock::text("capture"), NoteBlock::image("a".repeat(64))];
        note.normalize();
        assert_eq!(note.blocks.len(), 3);
        assert!(note.blocks.last().unwrap().is_empty_text());
    }

    #[test]
    fn normalize_collapses_consecutive_empty_blocks() {
        let mut note = Note::new();
        note.blocks = vec![
            NoteBlock::text("text"),
            NoteBlock::text(""),
            NoteBlock::text("  "),
        ];
        note.normalize();
        assert_eq!(note.blocks.len(), 2);
        assert!(note.blocks.last().unwrap().is_empty_text());
    }

    #[test]
    fn normalize_keeps_at_least_one_text_block() {
        let mut note = Note::new();
        note.blocks = vec![];
        note.normalize();
        assert_eq!(note.blocks.len(), 1);
        assert!(note.blocks[0].as_text().is_some());
    }

    #[test]
    fn excerpt_collapses_newlines() {
        let mut note = Note::new();
        note.blocks = vec![NoteBlock::text("ligne un\nligne deux")];
        assert_eq!(note.excerpt(100), "ligne un ligne deux");
    }

    #[test]
    fn excerpt_is_bounded_by_max_chars() {
        let mut note = Note::new();
        note.blocks = vec![NoteBlock::text("a".repeat(10_000))];
        assert!(note.excerpt(40).chars().count() <= 41);
    }
}
