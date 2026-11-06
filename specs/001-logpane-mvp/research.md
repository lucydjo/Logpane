# Research — Logpane MVP

All versions were checked against crates.io on 2026-11-04 and pinned in `Cargo.toml`; the
resolved graph is frozen in `Cargo.lock`.

## R1 — UI stack and event model

- **Decision**: `eframe`/`egui` 0.37 with the `glow` renderer, `default-features = false`.
- **Rationale**: Native Rust drawing, a mature multiline text widget with selection/IME/undo,
  texture support for inline images, and no web runtime. In 0.37 the `App` trait exposes
  `ui(&mut self, ui, frame)` and a separate `logic(&mut self, ctx, frame)` that still runs while
  the window is hidden, which is exactly what a toggleable overlay needs.
- **Alternatives**: `wgpu` renderer (heavier startup, no benefit for 2D); `iced` (no first-class
  multiline text editing with per-block caret control); Tauri/Electron (forbidden by the
  constitution).

## R2 — Resident process and hide/show

- **Decision**: Start with `ViewportBuilder::with_visible(false)`, hide with
  `ViewportCommand::Visible(false)`, show with `Visible(true)` + `Focus`. Intercept
  `close_requested`, answer `CancelClose`, then hide. Never call `Close` on the root viewport.
- **Rationale**: egui documents that a hidden viewport runs no egui pass but keeps the process
  and all widget memory alive, so reopening is a single frame and caret/undo state survives.
- **Alternatives**: destroying and recreating the window (loses state, far above 150 ms);
  `NSApp.hide` (hides every window of the app and interferes with activation).

## R3 — Global shortcut

- **Decision**: `global-hotkey` 0.9. Create `GlobalHotKeyManager` on the main thread in `main`,
  register `HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyD)`. A dedicated
  listener thread blocks on `GlobalHotKeyEvent::receiver().recv()` and, on `Pressed`, toggles an
  `AtomicBool` and calls `ctx.request_repaint()`.
- **Rationale**: matches the crate's macOS requirement (manager + event loop on the main thread),
  avoids polling entirely, and keeps the UI thread free.
- **Alternatives**: `rdev`/CGEventTap (requires Accessibility permission — forbidden by the
  spec); polling the event channel each frame (violates the idle-CPU goal).
- **Fallback**: if registration fails (shortcut owned by another app), the app stays usable via
  a minimal menu-bar item "Show Logpane" and reports the conflict in the header.

## R4 — Floating window behaviour

- **Decision**: Configure the existing `NSWindow` produced by eframe, obtained from
  `frame.window_handle()` (AppKit `ns_view` → `window`): level `NSFloatingWindowLevel` (3),
  collection behaviour `CanJoinAllSpaces | FullScreenAuxiliary | IgnoresCycle`,
  `hidesOnDeactivate = false`, `releasedWhenClosed = false`.
- **Rationale**: the constitution requires configuring eframe's window before considering an
  `NSPanel`; this is the smallest bridge that satisfies Spaces and full-screen auxiliary needs.
- **Alternatives**: `NSPanel` reimplementation (deferred until real testing proves it necessary);
  egui's `WindowLevel::AlwaysOnTop` (not enough for full-screen auxiliary).

## R5 — Clipboard images

- **Decision**: `NSPasteboard.generalPasteboard()` inspected in this order: `public.png` →
  `public.tiff` → `public.file-url` → `public.image` → `string`/`public.url`. Bytes are copied
  with `NSData::bytes`/`length` into a `Vec<u8>` synchronously (microseconds for a screenshot),
  then handed to the import worker.
- **Rationale**: covers macOS screenshots (TIFF or PNG), browser "Copy image", Finder file URLs
  and capture tools that only expose a temporary file URL. Reading bytes immediately satisfies
  the "never keep the temporary path" rule.
- **Alternatives**: `arboard` (does not expose the full set of macOS representations reliably);
  `ImageIO` conversion in the bridge (unnecessary — the `image` crate decodes TIFF/PNG/JPEG).

## R6 — Drag and drop

- **Decision**: Handle `ctx.input(|i| i.raw.dropped_files.clone())`. For each `DroppedFile`,
  prefer `bytes` when present, otherwise read `path` immediately. Hover feedback uses
  `i.raw.hovered_files` plus `i.pointer.hover_pos()` to choose the insertion index.
- **Rationale**: eframe already delivers Finder drops as paths and some apps as bytes; both are
  supported without extra platform code.
- **Alternatives**: native `NSDraggingDestination` (more code, no extra capability for files).

## R7 — Image pipeline and thumbnails

- **Decision**: Decode with `image::load_from_memory`, compute `sha2::Sha256`, store the
  original bytes under `assets/<hash>.<ext>` via temp-file + `rename`, and generate a PNG
  thumbnail (max 1 600 × 1 200, `FilterType::Triangle`) under `assets/thumbnails/<hash>.png`.
  Work runs on one dedicated image thread fed by a `sync_channel(8)`.
- **Rationale**: keeps the UI thread free, deduplicates identical screenshots, and guarantees
  durability. Thumbnails are PNG because `image` 0.25 guarantees WebP decoding but not encoding;
  PNG avoids an extra encoder dependency and remains lossless.
- **Alternatives**: WebP thumbnails (encoding reliability not guaranteed); JPEG thumbnails
  (lossy text, poor for screenshots); storing only thumbnails (loses original for copy/export).

## R8 — Persistence and write path

- **Decision**: `rusqlite` 0.41 with `bundled`; `journal_mode=WAL`, `foreign_keys=ON`,
  `busy_timeout=5000`, `synchronous=NORMAL`. One writer thread owns the `Connection`; the UI
  sends `SaveNote`, `SaveSetting`, `TouchNote`, `Flush(ack)` over an `mpsc` channel. Text changes
  are coalesced with a 200 ms debounce, then persisted with an incremented `revision`.
- **Rationale**: `NORMAL` in WAL mode survives application crashes and loses at most the last
  transaction on an OS crash — an explicit, documented trade-off between typing latency and
  durability. Every hide/switch/quit sends `Flush` and waits for the ack, so confirmed user
  actions are always durable.
- **Alternatives**: `synchronous=FULL` (fsync per keystroke batch, unnecessary with WAL);
  `OFF` (violates the constitution); one connection per thread (lock contention).

## R9 — Document model

- **Decision**: `notes.blocks_json` stores `{"schema_version":1,"blocks":[...]}` with a tagged
  enum (`text` | `image`). Text blocks are split/merged at the caret when an image is inserted
  mid-paragraph; a zero-length text block is preserved only while it holds the caret.
- **Rationale**: an ordered block list is the minimum model that renders real images in flow and
  survives future schema growth. The envelope makes migrations explicit.
- **Alternatives**: Markdown string (forbidden by the spec); single rich-text document (heavy,
  not needed); HTML (no).

## R10 — Window geometry

- **Decision**: Read `ViewportInfo.inner_rect` / `outer_rect` each frame, debounce 300 ms, store
  in points in `settings`. On startup clamp the restored rect against `NSScreen.visibleFrame`
  values from the bridge; if no saved geometry exists, centre a 1050 × 680 window.
- **Rationale**: logical points are what AppKit reports and what the spec requires; clamping in
  the bridge covers the disconnected-display case without trusting stale display IDs.
- **Alternatives**: storing pixels (wrong on Retina); trusting egui monitor info (not exposed in
  0.37's `ViewportInfo`).

## R11 — Themes and reduced motion

- **Decision**: Two hand-built `egui::Visuals` palettes (graphite/off-white, low contrast) plus
  a `ThemeMode` (`System`, `Dark`, `Light`) persisted in `settings`; system preference is read
  via `ctx.system_theme()`. Animation is permanently disabled (`style.animation_time = 0`) and
  the design uses a monospace system font, square corners, 1 px rules and a 4/8/12/16/24 spacing
  scale.
- **Rationale**: matches the terminal-like direction (no neon, no glass, no shadows) and keeps
  the theme button meaningful at all times.
- **Alternatives**: egui's stock dark/light visuals (too high contrast, pure black surfaces);
  following the system only (the spec requires an explicit toggle).

## R12 — Search

- **Decision**: In-memory filter over loaded note summaries + body text, normalizing case and
  diacritics (`é → e`) with a small hand-written fold table (no ICU dependency). Matches title
  and concatenated text blocks.
- **Rationale**: at 1 000 notes the full scan is well under the 200 ms target and avoids an
  index that would need invalidation on every keystroke.
- **Alternatives**: SQLite FTS5 (extra schema, tokenizer tuning, overkill at this scale).

## R13 — Testing strategy

- **Decision**: Pure logic and SQLite tests run with `cargo test` against `tempfile` directories,
  injecting the data root so nothing touches the real Application Support folder. `main` accepts
  `--smoke-test` to open storage, create/read a note and exit without creating a window. GUI
  behaviour is validated manually with `quickstart.md`.
- **Rationale**: satisfies the constitution's unit-test gate while acknowledging that global
  shortcuts and AppKit levels cannot be asserted in CI.
- **Alternatives**: `kittest`/AccessKit UI automation (possible later; adds harness complexity
  without covering the AppKit window level).

## R14 — PDF export

- **Decision**: `printpdf` 0.12 with an embedded subset of the macOS monospace font, A4 pages,
  a real text layer (no rasterized text), lossless image XObjects and a native `NSSavePanel`.
- **Rationale**: text stays selectable and extractable by `pdftotext` and LLM ingestion tools,
  while images keep their full resolution. printpdf writes a ToUnicode map, so copy/paste and
  text extraction work.
- **Alternatives**: rasterizing the note (unreadable for LLMs), HTML-to-PDF via a WebView
  (forbidden), base-14 PDF fonts (poor Unicode/accents), a Markdown export (not a PDF).

## R15 — Geometry of the delivered app

- **Decision**: Ship a `scripts/build-app.sh` that assembles `Logpane.app` with an
  `Info.plist` (`LSUIElement = true`, `NSHighResolutionCapable = true`,
  `LSMinimumSystemVersion = 12.0`). Unsigned local build; the script prints the ad-hoc signing
  command.
- **Rationale**: an `LSUIElement` bundle is what makes a resident, dock-less overlay feel native;
  running the raw binary still works for development.
