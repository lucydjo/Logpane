# Logpane Constitution

## Core Principles

### I. Instant, Frictionless Capture (NON-NEGOTIABLE)

Logpane exists to record an observation at the exact moment it appears, so the
time from "I noticed something" to "it is persisted" MUST stay in the range of a
few seconds. The process MUST remain resident with the overlay pre-initialized;
showing the overlay after `⌘⇧D` targets ≤ 150 ms on the reference Mac (M2), measured
on a Release build. On show, the previously active note, its content, the caret, the
theme and the window geometry MUST be restored, and keyboard focus MUST land in the
editor without an extra click.

No workflow may require the user to create a project, name a file, choose a folder
or press a save button. Any design decision that adds a step between observation and
persistence MUST be rejected or explicitly justified against this principle.

### II. Keyboard-First, Mouse-Complete

The global shortcut `⌘⇧D` is the primary entry point and MUST toggle the overlay from
any application. Every user action MUST be reachable from the keyboard, and `Esc` MUST
hide the overlay unless an open dialog or internal control must consume it first.
Hiding MUST never discard text, and closing the window MUST NOT terminate the process.

Mouse behaviour is equally non-negotiable: click to place the caret, drag to select
across lines, copy/paste, drag-and-drop, header-only window dragging and native edge/
corner resizing. Whole-surface window dragging is forbidden because it conflicts with
text selection. All shortcuts MUST be declared in one dedicated module; a global and
an internal binding MUST NOT fire twice for the same keystroke.

### III. Images Are Content, Never Paths (NON-NEGOTIABLE)

A pasted or dropped screenshot MUST render as an image inside the document flow.
Raw references — `/tmp/...`, `file:///...`, bare filesystem paths, or literal Markdown
image syntax — MUST NEVER be shown to the user as a substitute for the image, and MUST
NEVER be stored as the durable reference.

Every image import MUST follow the same pipeline: identify the available representation
(pasteboard type, drop payload, or file URL) → read the bytes immediately while access is
still valid → decode and validate off the UI thread → compute SHA-256 → write into the
application asset store via temp-file plus atomic rename → generate a display thumbnail →
insert an image block at the drop/caret position (or the end of the active block when the
position is unreliable) → persist the asset reference with the note. Duplicate hashes MUST
reuse the existing asset instead of copying again.

Source temporary files are transient: their deletion, an application restart or a machine
reboot MUST NOT affect the note. Assets referenced by any note MUST NOT be deleted, and
removing an image block MUST NOT delete a potentially shared asset. Invalid or unsupported
formats MUST surface an understandable message and preserve existing text; they MUST NOT
crash or corrupt the editor. Losing an image is a functional blocker for the MVP, not a
cosmetic defect.

### IV. Immediate Durable Local Persistence (NON-NEGOTIABLE)

There is no save button and no deferred autosave window. Every real change — text, title,
block insertion or removal, metadata — MUST produce a new revision and be persisted through
a serialized write queue on a dedicated database thread. Per-frame writes and timer-based
5–10 second autosave intervals are forbidden. The write queue MUST be flushed before hiding
the overlay, switching notes, quitting or shutting down the process.

The save indicator MUST reflect the last persisted revision, never merely the presence of
the text in memory. On a write failure the data MUST stay in memory, an error state MUST be
shown and the write MUST be retried; silent content loss is forbidden. SQLite MUST run with
WAL, foreign keys and a sane `busy_timeout`; the `synchronous` level MUST be chosen
deliberately and documented rather than disabled for convenience. Any scenario that can lose
a persisted-looking note is a release blocker.

### V. Native Rust, No Web Runtime (NON-NEGOTIABLE)

The application MUST be built as a direct native desktop app in stable Rust using
`egui` + `eframe` (with `winit`), bridging to AppKit through `objc2` only where the UI
layer genuinely cannot provide the behaviour: floating window level, Spaces and full-screen
auxiliary behaviour, pasteboard image representations, and precise move/resize or activation
signals.

Electron, Tauri/WebView, React, an embedded JavaScript runtime and a local HTTP server MUST
NOT be introduced. All AppKit calls, and all `unsafe` except the file lock in `src/instance.rs`, MUST be isolated
under `src/platform/macos/`; models, note logic, persistence and asset handling MUST remain
UI-independent and unit-testable. Prefer configuring the `NSWindow` created by eframe over
reimplementing the window as an `NSPanel`; introduce a panel wrapper only after real testing
proves the window insufficient. Dependencies MUST use exact or caret-pinned versions — never
`*` — and `Cargo.lock` MUST be committed.

### VI. Local-Only, Privacy by Default

The MVP MUST make no network calls and MUST include no accounts, analytics or telemetry.
Notes and screenshots MUST never leave the machine. The application MUST read only the files
the user explicitly pastes or drops, and temporary paths MUST be used only during import and
never retained in note content. Logs MUST NOT contain note text or image content.

Permissions MUST be minimal: Accessibility or Screen Recording access MUST NOT be requested
without a demonstrated need for the chosen implementation. The data model MUST be designed so
cloud sync can be added later, but cloud sync MUST NOT be implemented in the MVP.

### VII. Verified on Real Hardware, Not Assumed

A macOS behaviour MUST NOT be reported as complete until it has been exercised on a real
Apple Silicon Mac (M2 as the reference target). Simulators, unit tests and reading
documentation do not satisfy this rule for shortcut, window, pasteboard and drag-and-drop
behaviour.

Performance, memory and binary-size claims MUST come from measured Release builds on the
target machine, never from estimates. When an AppKit API is uncertain, a small isolated
prototype MUST validate it before the architecture depends on it. Every phase MUST end with
a compiling version, a reproducible launch command, passing tests for the affected domain,
and an updated `TODO.md` recording what works, what remains and known issues.

## Technical Constraints

**Platform**: macOS on Apple Silicon (M1–M4), M2 as the primary target. No Windows or Linux
support in the MVP. UI language is English; strings and layout MUST be structured so another
language can be added without restructuring the UI code. The visual language MUST stay
terminal-like: monospace system font, square corners, 1 px rules, low-contrast surfaces, no
shadows, and one shared spacing scale for headers, lists, insets and dialogs.

**Approved stack**: Rust stable; `egui`/`eframe` with `winit`; `objc2`, `objc2-app-kit` and
`objc2-foundation` for the limited AppKit bridge; `global-hotkey` for `⌘⇧D`; `rusqlite` +
SQLite for storage; `serde`/`serde_json` for the versioned block schema; `image` plus ImageIO
via the native bridge where macOS formats require it; `sha2`, `uuid`, `chrono` (or `time`);
`printpdf` for PDF export; `tracing`, `thiserror`/`anyhow`. Versions MUST be verified against current crate documentation
before adoption. Substituting a different UI or persistence stack requires an explicit,
documented justification of benefit and cost.

**Storage**: `~/Library/Application Support/io.github.lucydjo.logpane/` holding
`logpane.sqlite`, `assets/<sha256>.<ext>` and `assets/thumbnails/<sha256>.png`. SQLite is the
MVP source of truth; no backup archive or persistent log is implemented yet. Markdown export and
full ZIP backup/restore are post-MVP. Tables MUST cover `notes` (id, title, timestamps, serialized blocks, revision),
`assets` (hash, internal path, MIME type, dimensions, size, import date), the note/asset
relation, and `settings` (theme, window geometry, active note, preferences).

**Document model**: a note is an ordered list of versioned blocks — text blocks and image
blocks carrying an asset reference — with a stable UUID per block, a `revision` counter and
ISO 8601 timestamps. A single serialized `String` that stores Markdown image syntax in place
of real blocks is forbidden. A text block is one plain multiline editor; all blocks share one
vertical scroll surface so the document reads as continuous.

**Window behaviour**: a borderless, non-modal floating window above normal windows, usable
across Spaces and in front of full-screen apps (evaluate `FullScreenAuxiliary`). Draggable only
by the header; natively resizable. Initial size 1050×680 pt, minimum 720×460 pt, editor column
at roughly 72–78 % of width. Geometry is stored in logical points (not Retina pixels) with a
debounced write, and restored on launch; if the recorded display is gone, the window MUST be
repositioned into the visible area of the main display.

**Threading**: the UI thread MUST NOT decode heavy images, run exports or block on disk.
Use a dedicated database worker plus a bounded image task queue with typed, ordered messages;
no thread per keystroke or per image. Workers MUST shut down gracefully after the final writes
are acknowledged.

**MVP non-goals**: cloud sync, collaboration, accounts, embedded AI, audio/video recording,
OCR, direct publishing, non-macOS builds, and rich-text formatting.

## Development Workflow & Quality Gates

Delivery proceeds in vertical slices, each producing a usable increment. Building the whole
feature set in one pass is forbidden.

1. **Phase 1 — Native skeleton**: Rust project, eframe/egui window, two-column layout,
   movable/resizable window with persisted geometry, `⌘⇧D` show/hide, resident process,
   macOS feasibility of the floating window validated. No database, no image import.
2. **Phase 2 — Notes and editing**: `Note`/`NoteBlock` models, create/select/delete,
   multiline editor, title and timestamps, save indicator.
3. **Phase 3 — Robust persistence**: SQLite schema and migrations, serialized write queue,
   per-change saves, active-note restoration, restart tests.
4. **Phase 4 — Image import**: pasteboard access, image bytes/TIFF/PNG/file-URL handling,
   drag-and-drop, internal asset copy, off-thread thumbnails, image blocks rendered in flow,
   temp-source-deletion tests.
5. **Phase 5 — Finishing**: search, dark/light themes, internal shortcuts, Spaces and
   full-screen verification, Release build, measurements, optional local `.app` packaging.

**Gates applying to every phase**:

- `cargo build` (and `cargo test`) MUST pass; the launch command MUST be documented and
  reproducible.
- Unit tests MUST cover the block/document model, asset import logic and persistence for the
  domains touched by the phase.
- No temporary path may appear in persisted content; no image may downgrade to a path.
- `TODO.md` MUST be updated with working features, remaining work and known issues.
- MVVM-style separation MUST hold: business logic and storage MUST be testable without
  launching the UI.

**MVP release gates** (all must pass on real Apple Silicon hardware): shortcut opens and
closes the overlay from Safari, Chrome, an IDE and Finder, repeatedly, including from a
full-screen app; editor receives focus on open; `Esc` hides without text loss; multiline
editing, caret placement, mouse selection, text copy/paste, undo/redo; multiple notes with
search over title and body; `⌘V` of a screenshot shows the real image; drag-and-drop of
PNG/JPEG, temp paths and file URLs imports correctly; images survive restart and source
deletion; no automatic save step is ever required and no modification is lost across note
switches or overlay hide/show; window position and size restore, with off-screen recovery;
theme restores; no network traffic; no crash on invalid image input.

## Governance

This constitution supersedes conflicting local habits, prompts and implementation shortcuts.
The specification in `project.md` defines product scope; this document defines the rules that
scope must respect. Where they conflict, the conflict MUST be resolved explicitly rather than
silently, and this constitution prevails on principles, stack, persistence and quality gates.

**Amendment procedure**: amendments are proposed as a written change to this file, must state
the affected principles and the motivation, must include the Sync Impact Report comment at the
top of the file, and require explicit maintainer approval before merge. The Sync Impact Report
is review material and MUST be removed before the amendment is committed.

**Versioning policy**: semantic versioning of governance. MAJOR for removal or redefinition of
a principle or a breaking governance change; MINOR for a new principle or materially expanded
guidance; PATCH for clarifications, wording and non-semantic refinements. Every amendment
updates `Last Amended`; the ratification date changes only on re-ratification.

**Compliance review**: every planning, task and implementation review MUST verify each artefact
against the principles above. Any deviation MUST be recorded with its
justification in the plan's Complexity Tracking section; an unjustified deviation blocks the
change. The non-negotiable principles (I, III, IV, V) admit no deviation without a formal
amendment.

**Runtime guidance**: `project.md` carries product specification and scope, `TODO.md` carries
per-phase status and known issues, and `README.md` carries build and launch instructions.
These documents are subordinate to this constitution.

**Version**: 1.0.1 | **Ratified**: 2026-11-04 | **Last Amended**: 2026-11-04
