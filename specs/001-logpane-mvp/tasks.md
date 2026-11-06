# Tasks: Logpane — Instant Observation Capture

**Input**: Design documents from `/specs/001-logpane-mvp/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included because the project constitution makes model, import and persistence tests a
phase gate. GUI/AppKit behaviour remains a manual acceptance gate (see quickstart.md).

**Organization**: Tasks are grouped by user story so each story can be implemented, tested and
demonstrated on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)

## Path Conventions

Single project at the repository root: `src/`, `tests/`, `scripts/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and pinned dependencies.

- [x] T001 Create `Cargo.toml` with pinned versions from research.md (eframe/egui 0.37,
      global-hotkey 0.9, objc2 0.6.5, objc2-app-kit 0.3.2, rusqlite 0.41 bundled, image 0.25,
      sha2 0.11, uuid 1.29, chrono 0.4, serde, tracing, directories 6), `Cargo.lock` committed
- [x] T002 Create the module skeleton `src/{main.rs,app.rs,core,storage,ui,platform/macos}`
      plus `tests/` and `scripts/` directories per plan.md
- [x] T003 [P] Add `.gitignore` (target/, Logpane.app) and `rustfmt.toml`
- [x] T004 [P] Add `tempfile` as a dev-dependency and wire `cargo test` discovery

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Models, storage, asset pipeline, theme and platform bridge that every story needs.

**⚠️ CRITICAL**: No user story work can begin until this phase compiles and `cargo test` passes.

- [x] T005 [P] Implement `Note`, `NoteBlock`, `DocumentEnvelope`, defaults, validation and
      serialization in `src/core/models.rs`
- [x] T006 [P] Implement schema v1 and `user_version` migration in `src/storage/migrations.rs`
- [x] T007 Implement `Storage` open/pragmas/CRUD and the `SaveNote`/`DeleteNote`/`SetSetting`/
      `Flush`/`Shutdown` writer-thread protocol in `src/storage/sqlite.rs`
- [x] T008 Implement note creation, selection, deletion and title derivation in
      `src/core/note_service.rs`
- [x] T009 Implement `SaveTracker` (dirty flag, 200 ms coalescing, revision counter, persisted
      ack, error/retry state) in `src/core/autosave.rs`
- [x] T010 [P] Implement normalized search and ranking in `src/core/search.rs`
- [x] T011 Implement byte-first asset import (SHA-256, atomic write, dedup, PNG thumbnail,
      bounded worker) in `src/core/assets.rs`
- [x] T012 [P] Implement dark/light palettes and reduced-motion handling in `src/ui/theme.rs`
- [x] T013 [P] Implement the English string table in `src/strings.rs`
- [x] T014 Implement `/tmp` + `file://` + Markdown-image rejection helpers used by import and
      persistence tests in `src/core/models.rs`
- [x] T015 Implement the floating-window AppKit bridge (level, collection behaviour,
      hidesOnDeactivate, geometry clamp against `NSScreen`) in `src/platform/macos/window.rs`
- [x] T016 Implement the `⌘⇧D` manager plus listener thread in
      `src/platform/macos/hotkey.rs`
- [x] T017 Implement `NSPasteboard` extraction (PNG → TIFF → file URL → image → text path) in
      `src/platform/macos/pasteboard.rs`
- [x] T018 Implement the app state machine and bootstrap (hidden start, viewport commands,
      writer/image workers, shutdown flush) in `src/app.rs` and `src/main.rs`
- [x] T019 [P] Write persistence round-trip, revision and `note_assets` tests in
      `tests/persistence.rs`
- [x] T020 [P] Write asset dedup, atomic write, thumbnail and path-rejection tests in
      `tests/asset_import.rs`

**Checkpoint**: `cargo build` + `cargo test` green; storage and bridge compile on arm64.

---

## Phase 3: User Story 1 — Capture an observation the moment it appears (P1) 🎯 MVP

**Goal**: `⌘⇧D` shows a focused overlay, text is captured and persisted, `Esc` hides it, nothing
is lost, and the process stays resident.

**Independent Test**: Launch, switch app, `⌘⇧D`, type two paragraphs, `Esc`, `⌘⇧D` again — text
and caret are intact and no save prompt ever appeared.

- [x] T021 [P] [US1] Write document-model tests (title derivation, block split/merge, empty
      block normalization, envelope versioning) in `tests/document_model.rs`
- [x] T022 [US1] Implement the header (title bar drag zone, save badge, theme button, hide
      button) in `src/ui/header.rs`
- [x] T023 [US1] Implement the two-column shell and editor scroll surface in `src/ui/shell.rs`
- [x] T024 [US1] Implement multiline text blocks with caret, selection, wrap and focus restore
      in `src/ui/editor.rs`
- [x] T025 [US1] Wire hide/show: global hotkey toggle, `Esc`, header close, `⌘W`, close-request
      interception, `CancelClose` and hide instead of quit
- [x] T026 [US1] Wire autosave: change detection → debounce → writer → save badge; flush on
      hide, note switch and `⌘Q`/`on_exit`
- [x] T027 [US1] Restore active note, caret block and window focus on every show
- [x] T028 [US1] Implement `--smoke-test` headless startup/storage check in `src/main.rs`
- [x] T029 [US1] Add graceful error handling for hotkey registration failure (header notice)

**Checkpoint**: Text capture loop works end-to-end and survives hide/show and restart.

---

## Phase 4: User Story 2 — Attach a screenshot inside the note (P1)

**Goal**: Pasted or dropped images render inline, are copied to permanent storage, deduplicate,
and survive restart and source deletion.

**Independent Test**: Paste a screenshot, drag a PNG, restart, delete the source files — both
images still render and no path appears in the document.

- [x] T030 [US2] Implement image insertion at the caret (split text block) and at the drop
      index, with predictable fallback to the end of the active block
- [x] T031 [US2] Implement `⌘V` interception: image detected → import and remove the textual
      paste event; text → native paste untouched
- [x] T032 [US2] Implement drag-and-drop: `hovered_files` highlight, `dropped_files` bytes/path
      handling, multi-image order, non-image reporting
- [x] T033 [US2] Implement thumbnail texture cache and inline image rendering at column width
- [x] T034 [US2] Implement enlarged preview modal, context menu (Ouvrir / Copier / Supprimer)
- [x] T035 [US2] Implement loading chip, English error banner, missing-asset placeholder and
      post-import focus restoration
- [x] T036 [US2] Ensure import reference is persisted only after the asset file exists; keep
      shared assets when a block is removed
- [x] T037 [US2] Write image rejection tests (non-image drop, undecodable bytes, text path that
      no longer exists) in `tests/asset_import.rs`

**Checkpoint**: Images are content, never paths, across restart and source deletion.

---

## Phase 5: User Story 3 — Keep several experiments apart (P2)

**Goal**: Multiple notes with titles, list, search, selection and confirmed deletion.

**Independent Test**: Create three notes, restart, search a word from the second, open it, delete
the third with confirmation.

- [x] T038 [US3] Implement the sidebar list (title, excerpt, date grouping, active indicator) in
      `src/ui/note_sidebar.rs`
- [x] T039 [US3] Implement `+ Nouvelle note` / `⌘N`, note selection, and the `⌘K` search field
- [x] T040 [US3] Implement delete with confirmation modal and last-note blank fallback
- [x] T041 [US3] Implement editable title field with custom-title latch and auto-derived titles
- [x] T042 [US3] Implement filters (title + body, diacritic-insensitive) and empty-result state
- [x] T043 [US3] Persist active note id and rebuild the list after create/delete/rename
- [x] T044 [US3] Write search ranking and filter tests in `tests/search.rs`

**Checkpoint**: A multi-note library survives restart with consistent selection and search.

---

## Phase 6: User Story 4 — Timestamp observations inside a note (P2)

**Goal**: `⌘⇧;` and `+ Heure` insert `HH:mm — ` at the caret, or create a text block when the
caret is not in text.

**Independent Test**: Caret mid-paragraph → timestamp at caret; image block focused → new text
block after the image.

- [x] T045 [US4] Implement `insert_timestamp` at character caret with `HH:mm — ` formatting
- [x] T046 [US4] Implement new-block insertion when the active element is not a text block
- [x] T047 [US4] Add the `+ Heure` editor action and the `⌘⇧;` shortcut entry
- [x] T048 [US4] Keep caret after the inserted timestamp and mark the note dirty

**Checkpoint**: Chronological logs can be written without typing the time.

---

## Phase 7: User Story 5 — Comfortable themes and a stable window (P3)

**Goal**: Theme cycle, persisted geometry, off-screen recovery, minimum sizes, header-only drag.

**Independent Test**: Move/resize, switch theme, quit, relaunch — same geometry and theme.

- [x] T049 [US5] Implement the System → Dark → Light cycle button with active-state display
- [x] T050 [US5] Persist `theme_mode` and apply system preference at first launch
- [x] T051 [US5] Persist window geometry with 300 ms debounce and restore it at startup
- [x] T052 [US5] Clamp restored geometry into a visible screen frame; default centre if the
      saved display is gone
- [x] T053 [US5] Enforce minimum sizes and editor/sidebar proportions; disable content drag
- [x] T054 [US5] Honour reduced motion and increased contrast

**Checkpoint**: The overlay always comes back where and how the user left it.

---

## Phase 8: Polish & Cross-Cutting Concerns

- [x] T055 [P] Write `README.md` (build, run, data location, known limits)
- [x] T056 [P] Write `project.md` (scope summary) and `TODO.md` (status, manual gates)
- [x] T062 [US1] Implement PDF export with a text layer, embedded images and a native save panel
- [x] T057 Implement the minimal menu-bar fallback (`Show Logpane`, `Quit Logpane`) in
      `src/platform/macos/menu.rs`
- [x] T058 Add `scripts/build-app.sh` and `Info.plist` (`LSUIElement`, macOS 12 minimum)
- [x] T059 Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
      `cargo build --release`
- [ ] T060 Execute quickstart manual acceptance on the target Mac and record results in
      `TODO.md` — **deferred**: requires interactive use of the installed app (hotkey, mouse,
      clipboard, drag-and-drop); automated coverage and visual captures are recorded instead
- [x] T061 Measure release startup, idle CPU and memory; record numbers in `TODO.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies
- **Foundational (Phase 2)**: depends on Setup; blocks every story
- **US1 (Phase 3)**: depends on Foundational
- **US2 (Phase 4)**: depends on US1 (reuses editor, autosave, app state)
- **US3 (Phase 5)**: depends on US1; independent of US2
- **US4 (Phase 6)**: depends on US1; independent of US2/US3
- **US5 (Phase 7)**: depends on US1; independent of US2/US3/US4
- **Polish (Phase 8)**: depends on all stories

### Within Each Story

Models before services, services before UI wiring, UI before persistence wiring, tests alongside
the slice they cover (tests added in the same task group).

### Parallel Opportunities

- T005, T006, T010, T012, T013 are independent files.
- T019 and T020 are independent test files.
- T055 and T056 are independent documents.
- US3, US4 and US5 can be developed in parallel once US1 is complete.

---

## Implementation Strategy

### MVP First

1. Phase 1 → Phase 2 → Phase 3 (US1): text capture loop, persisted, resident.
2. Validate US1: `cargo test`, then the first six manual checks in quickstart.md.
3. Phase 4 (US2): images are the declared functional blocker — do not ship without them.
4. Phase 5 to 7: notes library, timestamps, comfort.
5. Phase 8: polish, packaging, measurements, manual acceptance record.

### Incremental Delivery

Each checkpoint leaves a compiling binary and a reproducible `cargo run`. No story may break a
previous one; the persistence tests and the image tests act as regression guards.

---

## Notes

- `[P]` marks tasks that touch different files and can be done concurrently.
- Manual macOS acceptance is mandatory before claiming a story done (constitution P7).
- Update `TODO.md` after each phase with what works, what remains, known issues.
- Never weaken `synchronous`, never store a source path, never report success before the writer
  acknowledges the revision.
