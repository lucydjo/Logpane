# Data Model — Logpane MVP

All timestamps are stored as ISO 8601 UTC strings and displayed in the Mac's local time zone.
Identifiers are UUID v4 strings. The document body is versioned JSON, so schema evolution never
requires rewriting the note table.

## Note

| Field | Type | Rules |
|---|---|---|
| `id` | UUID string | Primary key, generated on creation, never changes |
| `title` | string | Defaults to the first non-empty line, trimmed to 200 chars; may be empty |
| `title_custom` | bool | False until the user edits the title, then true forever |
| `created_at` | ISO 8601 UTC | Set once at creation |
| `updated_at` | ISO 8601 UTC | Updated on every persisted revision, never decreases |
| `revision` | integer | Starts at 0, increments by exactly 1 per persisted save |
| `blocks` | ordered `NoteBlock[]` | At least one block; at least one text block is always present |

**Derived behaviour**: while `title_custom` is false, `title` is recomputed from the first
non-empty line of the document after each text change. Editing the title field sets
`title_custom = true` and stops derivation.

**Validation**:
- `updated_at >= created_at`.
- Block identifiers are unique within a note.
- Every text block is allowed to be empty only if it is the sole block or holds the caret;
  empty trailing blocks are removed on blur.
- Deleting a note deletes its `note_assets` rows; assets themselves are kept (shared safety).

**State transitions**:

```text
(absent) --create--> Draft(revision 0, one empty text block)
Draft --text/title/block edit--> Dirty
Dirty --persisted--> Saved(revision n+1)
Saved --edit--> Dirty
Saved|Dirty --delete (confirmed)--> Deleted
Deleted --was last note--> auto-created blank note becomes active
```

## NoteBlock

Tagged union serialized with `"type": "text" | "image"`.

```json
{ "type": "text", "id": "uuid", "text": "09:15 — Première observation" }
```

| Field | Applies to | Rules |
|---|---|---|
| `id` | both | UUID, unique in the note, stable across edits and moves |
| `text` | text | Plain text, may contain newlines; no markup |
| `asset_id` | image | SHA-256 hex of the image content; must exist in `assets` |
| `caption` | image | Optional, unused in the MVP UI but preserved |

**Ordering**: the `blocks` array order is the rendering and reading order. Inserting an image at
a caret splits the text block into `[before]`, `[image]`, `[after]`; if `before` and `after` are
both empty, they collapse to a single empty text block after the image.

## DocumentEnvelope (stored in `notes.blocks_json`)

```json
{
  "schema_version": 1,
  "blocks": [ { "type": "text", "id": "…", "text": "…" } ]
}
```

Rules:
- `schema_version` is mandatory; an unknown version is loaded read-only and reported, never
  silently rewritten.
- Missing or malformed envelope → the note is loaded with a single text block containing a
  recovery notice, and the raw JSON is left untouched in the database.

## ImageAsset

| Field | Type | Rules |
|---|---|---|
| `hash` | 64-char lowercase hex | Primary key, SHA-256 of the original bytes |
| `rel_path` | string | `assets/<hash>.<ext>`, relative to the data root |
| `thumb_path` | string | `assets/thumbnails/<hash>.png` |
| `mime` | string | `image/png`, `image/jpeg`, `image/tiff`, `image/webp` |
| `width`, `height` | u32 | Decoded pixel dimensions, > 0 |
| `byte_size` | u64 | Size of the stored original |
| `imported_at` | ISO 8601 UTC | First time this content was imported |

**Identity**: content-addressed. Importing identical bytes twice reuses the row and file.

**Lifecycle**: written to a temporary file in `assets/`, fsynced, renamed atomically, then the
row is inserted. If the row insert fails, the file is left for the orphan cleanup routine and is
never referenced. Assets are never deleted automatically in the MVP.

## PreferenceSetting

Key/value table. Defined keys for v1:

| Key | Value |
|---|---|
| `theme_mode` | `system` \| `dark` \| `light` |
| `window_geometry` | JSON `{x, y, width, height, display_name}` in logical points |
| `active_note_id` | UUID string |
| `schema_version` | `1` |

Unknown keys are preserved. Missing keys fall back to defaults: system theme, centred
1050 × 680 window, most recently updated note.

## SearchResult (transient, not persisted)

`{ note_id, title, excerpt, updated_at, score }` where `score` is derived from match position
(title match ranks above body match). Rebuilt in memory from the loaded note index.

## Relationships

```text
Note 1 ──< NoteBlock (value objects, embedded)
Note >──< ImageAsset   (many-to-many through note_assets)
PreferenceSetting      (independent key/value)
```

`note_assets` is a derived projection of the image blocks, recomputed inside the same
transaction as the note save, so a foreign-key check guarantees that a persisted image block
always points at an existing asset row.
