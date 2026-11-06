# Implementation Plan: Logpane — Instant Observation Capture

**Branch**: `001-logpane-mvp` | **Date**: 2026-11-04 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-logpane-mvp/spec.md`

## Summary

Build a resident macOS overlay that captures text observations and inline screenshots in one
gesture. The app is a native Rust desktop binary built with `eframe`/`egui 0.37`, kept alive
while hidden, toggled by a global `⌘⇧D` hotkey, and given a floating AppKit window level via a
narrow `objc2` bridge. Notes are stored in local SQLite through a serialized writer thread;
images are imported immediately into a content-addressed asset store so no temporary path can
ever leak into a note. The overlay restores its note, caret, theme and geometry on every open.

## Technical Context

**Language/Version**: Rust 1.99.1 stable, edition 2021, macOS Apple Silicon (arm64) only

**Primary Dependencies**: `eframe`/`egui` 0.37 (glow renderer), `global-hotkey` 0.9,
`objc2` 0.6.5 + `objc2-app-kit` 0.3.2 + `objc2-foundation` 0.3.2, `rusqlite` 0.41 (bundled
SQLite), `image` 0.25, `sha2` 0.11, `uuid` 1.29, `chrono` 0.4, `serde`/`serde_json`,
`directories` 6, `printpdf` 0.12 (PDF export), `tracing`, `anyhow`/`thiserror`

**Storage**: SQLite (WAL) at `~/Library/Application Support/io.github.lucydjo.logpane/logpane.sqlite`;
assets at `assets/<sha256>.<ext>`, thumbnails at `assets/thumbnails/<sha256>.png`

**Testing**: `cargo test` (model, search, asset pipeline, persistence against temp dirs);
`logpane --smoke-test` for a headless startup/storage check; manual macOS acceptance per
`quickstart.md` for shortcut, window, pasteboard and drag-and-drop behaviour

**Target Platform**: macOS 12+ on Apple Silicon (M2 reference), no Windows/Linux, no WebView

**Project Type**: Single native desktop application (binary crate + integration tests)

**Performance Goals**: overlay interactive ≤ 150 ms after shortcut; keystroke response < 50 ms
during a 20 MB image import; idle CPU < 1 % while hidden; search < 200 ms over 1 000 notes

**Constraints**: Fully offline; no accessibility or screen-recording permission; UI in English,
strings centralized for future i18n; unsafe code confined to `src/platform/macos/`; window
geometry stored in logical points; assets must survive restart and source-file deletion

**Scale/Scope**: Single local user, up to ~1 000 notes / 50 000 words, screenshots up to ~20 MB,
one window, five prioritised user stories

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|---|---|---|
| I. Instant, frictionless capture (NN) | Resident process, pre-built UI, no naming/filing step, ≤150 ms target measured | PASS — hidden viewport keeps the process and UI state alive |
| II. Keyboard-first, mouse-complete | Global shortcut single registration, `Esc` hide, header-only drag, native resize, centralized shortcuts | PASS — `src/platform/macos/hotkey.rs` + `src/shortcuts.rs`; AppKit drag disabled on content |
| III. Images are content, never paths (NN) | Byte-first import pipeline, atomic asset write, dedup, thumbnail, no raw path storage | PASS — `src/core/assets.rs` + `src/platform/macos/pasteboard.rs` |
| IV. Immediate durable local persistence (NN) | Revision per change, serialized writer, flush on hide/switch/quit, honest save indicator, no silent loss | PASS — `src/core/autosave.rs` + `src/storage/sqlite.rs` |
| V. Native Rust, no web runtime (NN) | egui/eframe only, `unsafe` isolated, UI-independent core, pinned deps | PASS — `#![forbid(unsafe_code)]` in core modules; `deny(unsafe_op_in_unsafe_fn)` in the bridge |
| VI. Local-only, privacy by default | Zero network, minimal permissions, no note content in logs | PASS — no HTTP dependency in `Cargo.toml`; logging is structural only |
| VII. Verified on real hardware | Real M2 acceptance of shortcut, window, pasteboard, drag-and-drop; measured Release numbers | PARTIAL at build time — automated tests cover logic; interactive gates are listed in `quickstart.md` and tracked in `TODO.md`, and must be signed off on the target Mac |

No violations requiring Complexity Tracking. The only deliberate deviation from the indicative
storage tree is PNG thumbnails instead of WebP, because the pinned `image` crate guarantees
WebP *decoding* but not reliable *encoding*; PNG is lossless, universally decodable, and keeps
the import pipeline dependency-free (see `research.md`, R7).

## Project Structure

### Documentation (this feature)

```text
specs/001-logpane-mvp/
├── plan.md              # This file
├── research.md          # Phase 0 decisions
├── data-model.md        # Entities, schema, invariants
├── quickstart.md        # Build, run and acceptance validation
├── contracts/
│   ├── ui-contract.md        # Shortcuts, states, English UI strings
│   └── storage-contract.md   # SQL schema + versioned block JSON
└── tasks.md             # Task breakdown
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock
README.md
TODO.md
project.md
src/
├── main.rs                  # bootstrap, hotkey manager, eframe launch, smoke test
├── app.rs                   # LogpaneApp: state machine, event routing
├── core/
│   ├── mod.rs
│   ├── models.rs            # Note, NoteBlock, DocumentEnvelope, validation
│   ├── note_service.rs      # create/select/delete, title derivation, timestamps
│   ├── autosave.rs          # SaveTracker: dirty state, revisions, debounce, indicator
│   ├── search.rs            # normalization + filtering
│   └── assets.rs            # SHA-256, atomic store, thumbnails, dedup, import worker
├── storage/
│   ├── mod.rs
│   ├── sqlite.rs            # connection setup, CRUD, writer thread protocol
│   └── migrations.rs        # schema v1
├── export/
│   └── pdf.rs               # PDF export: text layer, images, pagination
├── ui/
│   ├── mod.rs
│   ├── shell.rs             # header + two-column layout
│   ├── header.rs            # title, save state, theme toggle, close
│   ├── editor.rs            # block editor, image rendering, drop handling
│   ├── note_sidebar.rs      # search, list, new/delete
│   ├── theme.rs             # dark/light palettes
│   └── strings.rs           # English strings, i18n-ready lookup
└── platform/
    └── macos/
        ├── mod.rs
        ├── window.rs        # floating level, Spaces/fullscreen, geometry clamp
        ├── hotkey.rs        # ⌘⇧D registration + listener thread
        ├── pasteboard.rs    # NSPasteboard image/file-url/text-path extraction
        └── menu.rs           # minimal menu-bar fallback (show / quit)

tests/
├── document_model.rs
├── search.rs
├── asset_import.rs
└── persistence.rs

scripts/
└── build-app.sh            # optional unsigned .app bundle for local use
```

**Structure Decision**: Single binary crate at the repository root, split into `core` (pure,
testable), `storage` (SQLite), `ui` (egui), and `platform/macos` (all `unsafe`/AppKit). This is
the layout mandated by the constitution; `core` and `storage` never import `ui`.

## Complexity Tracking

> No constitution violations. No entries.

## Post-Design Constitution Re-Check

After producing `data-model.md` and the contracts:

- P1/P4 remain satisfied: the writer protocol exposes `Flush(ack)` and the UI blocks on it at
  hide/switch/quit; the save indicator is driven only by `Persisted { revision }` acks.
- P3 remains satisfied: the storage contract stores only `asset_id`, and the asset pipeline
  writes bytes before the reference is inserted; no contract field accepts a source path.
- P5 remains satisfied: `objc2` appears only in `platform/macos`, and `core`/`storage` compile
  without AppKit imports.
- P7 remains the only manual gate; it is explicit in `quickstart.md` §"Manual acceptance".
