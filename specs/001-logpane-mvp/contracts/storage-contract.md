# Storage Contract — SQLite v1

Path: `~/Library/Application Support/io.github.lucydjo.logpane/logpane.sqlite`

Connection pragmas, applied on every connection before use:

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = NORMAL;
```

`synchronous = NORMAL` is deliberate: in WAL mode it keeps typing latency low while still
surviving an application crash. The last transaction may be lost only on an OS-level crash.
Every explicit user action (hide, switch note, quit) flushes and waits for an acknowledgement,
so confirmed work is never at risk.

## Schema

```sql
CREATE TABLE IF NOT EXISTS notes (
    id            TEXT PRIMARY KEY NOT NULL,
    title         TEXT NOT NULL DEFAULT '',
    title_custom  INTEGER NOT NULL DEFAULT 0 CHECK (title_custom IN (0, 1)),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    blocks_json   TEXT NOT NULL,
    revision      INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0)
);

CREATE INDEX IF NOT EXISTS idx_notes_updated_at
    ON notes (updated_at DESC);

CREATE TABLE IF NOT EXISTS assets (
    hash         TEXT PRIMARY KEY NOT NULL CHECK (length(hash) = 64),
    rel_path     TEXT NOT NULL,
    thumb_path   TEXT NOT NULL,
    mime         TEXT NOT NULL,
    width        INTEGER NOT NULL CHECK (width > 0),
    height       INTEGER NOT NULL CHECK (height > 0),
    byte_size    INTEGER NOT NULL CHECK (byte_size >= 0),
    imported_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS note_assets (
    note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    asset_hash  TEXT NOT NULL REFERENCES assets(hash) ON DELETE RESTRICT,
    PRIMARY KEY (note_id, asset_hash)
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

PRAGMA user_version = 1;
```

## Write protocol (UI → writer thread)

| Message | Payload | Acknowledgement |
|---|---|---|
| `SaveNote` | full `Note` (id, title, title_custom, timestamps, revision, blocks) | `Persisted { note_id, revision }` or `Failed { note_id, error }` |
| `DeleteNote` | `note_id` | none (the UI updates the list optimistically) |
| `SetSetting` | `key`, `value` | none (best effort, retried) |
| `Flush` | `oneshot` sender | `Flushed` |
| `Shutdown` | — | `Flushed` then thread exits |

`SaveNote` runs in one transaction:
1. `INSERT … ON CONFLICT(id) DO UPDATE` on `notes`.
2. `DELETE FROM note_assets WHERE note_id = ?`.
3. `INSERT INTO note_assets` for every distinct `asset_id` in the blocks (foreign key proves it
   exists).
4. Commit. Only then emit `Persisted`.

The writer retries a failed transaction up to 3 times with 50 ms backoff, then emits `Failed`
and keeps the note dirty in the UI. The UI never clears its in-memory copy on failure.

## Read protocol

- `load_all_notes()` returns notes ordered by `updated_at DESC`, plus a `LoadWarning` list for
  notes whose `blocks_json` could not be parsed or whose `user_version` is unknown.
- `load_settings()` returns all key/value pairs.
- Reads happen once at startup on the main thread (small dataset), plus a refresh after each
  `Persisted` ack is not needed because the UI already owns the truth.

## Block JSON

See `data-model.md` for the canonical shape. The storage contract guarantees only that:
- `blocks_json` is valid JSON with an integer `schema_version` and an array `blocks`;
- text blocks carry `type`, `id`, `text`;
- image blocks carry `type`, `id`, `asset_id`, optional `caption`;
- no field may contain a filesystem path, a `file://` URL, or Markdown image syntax.

An external integrity check (`tests/persistence.rs`) asserts these invariants and rejects any
note containing `/tmp/`, `file://` or the substring `![` in image blocks.

## Migration policy

`PRAGMA user_version` is the migration cursor. Version 1 is the initial schema. A future
migration runs inside a transaction, bumps `user_version`, and must never drop the `notes` or
`assets` tables. A database whose `user_version` is newer than the application is refused at startup with a clear error; it is never opened for writing.
