# Quickstart — Logpane MVP

## Prerequisites

- Apple Silicon Mac (M2 reference), macOS 12 or later
- Rust stable (validated with 1.99.1) and Xcode command-line tools (`xcode-select --install`)
- No other toolchain, no Node, no WebView runtime

## Build and test

```bash
cd /path/to/Logpane
cargo build                 # debug build, first build ~2-4 min (SQLite is bundled)
cargo test                  # model, search, asset pipeline, persistence
cargo build --release       # deliverable binary at target/release/logpane
```

Headless sanity check (no window, uses a temporary data root):

```bash
./target/debug/logpane --smoke-test
# expected: "smoke test ok: storage=<path> notes=<n>" and exit code 0
```

## Run

Development (terminal process, window starts hidden):

```bash
cargo run
```

Then press `⌘⇧D` from any application. Press it again, or `Esc`, to hide.

Packaged app (dock-less resident app):

```bash
./scripts/build-app.sh
open ./Logpane.app
```

The bundle sets `LSUIElement = true`, so there is no Dock icon and no menu bar takeover; use
`⌘⇧D` or the `Logpane` status item to show the window.

Data lives in `~/Library/Application Support/io.github.lucydjo.logpane/`. Delete that folder to reset.

## Manual acceptance (must run on the target Mac)

These cannot be asserted by `cargo test`; record results in `TODO.md`.

### Shortcut and overlay

1. Open Safari, press `⌘⇧D` → overlay appears above Safari, caret already in the editor.
2. Type two paragraphs, press `⌘⇧D` → overlay hides; press `⌘⇧D` again → both paragraphs and
   the caret are back.
3. Press `Esc` while typing → overlay hides, no text lost.
4. Repeat 10 times from Finder, an IDE and a full-screen app; the shortcut never stops working.
5. Click Safari while the overlay is open → the overlay stays visible (MVP behaviour).

### Editing

6. Click mid-word, drag across three lines, copy, paste elsewhere: standard macOS behaviour.
7. `⌘Z` / `⌘⇧Z` undo and redo text edits (image-block undo is not implemented yet).
8. `⌘N` creates a note; the title follows the first line until you edit the title field.
9. `⌘K` focuses search; typing filters the list; `Esc` hides the overlay.
10. `⌘⇧;` inserts `HH:mm — ` at the caret.

### Images (blocking)

11. Take a screenshot (`⌘⇧4`), press `⌘V` in the editor → the real image appears at the caret;
    no `/tmp/...`, no `file://`, no `![`.
12. Copy an image from Safari, `⌘V` → image appears.
13. Drag a PNG and a JPEG from Finder → both images appear in drop order.
14. Drag a capture from a tool that only exposes a temporary path → image appears.
15. Quit the app, relaunch, press `⌘⇧D` → every image is still there.
16. Delete the original screenshot files from the Desktop, relaunch → images still there.
17. Paste the same screenshot twice → two blocks, one file in `assets/`.
18. Drop a PDF → `Unsupported format`, text untouched, no crash.

### PDF export

19. Open a note with text and images, click `[ export pdf ]`, save it, open the PDF: the text is
    selectable, the images are visible, and `pdftotext file.pdf -` returns the note text.
20. Export a note with a very long body and confirm that pages break without clipping.

### Persistence and window

21. Type without saving, quit with `⌘Q`, relaunch → text is present.
22. Switch notes five times while typing → no text is lost.
23. Move and resize the window, quit, relaunch → same position and size (±1 pt).
24. Disconnect the external display used for the last position, relaunch → the window is fully
    visible on the built-in display.
25. Switch to the light theme, quit, relaunch → the light theme is restored.
26. Hide the overlay, watch Activity Monitor for 5 minutes → CPU near 0 %.

### Privacy

27. Run `lsof -i -p $(pgrep logpane)` during a session → no network sockets.
28. Run from a terminal with `RUST_LOG=logpane=debug` and confirm the output contains no note text or image content.
29. Confirm macOS never asked for Accessibility or Screen Recording.

## Automated coverage

| Area | Test file |
|---|---|
| Block model, split/merge at caret, title derivation | `tests/document_model.rs` |
| Case/diacritic-insensitive search and ranking | `tests/search.rs` |
| SHA-256 dedup, atomic asset write, thumbnail generation, path rejection | `tests/asset_import.rs` |
| Schema creation, save/load round-trip, revision monotonicity, note_assets projection | `tests/persistence.rs` |

## Troubleshooting

- `⌘⇧D` does nothing: another app owns it; use the Logpane status item, quit the conflicting
  app, or change the binding in a future version (not configurable in the MVP).
- Window invisible after a monitor change: the app clamps to the visible frame on launch; if it
  still fails, delete `window_geometry` from the `settings` table.
- Images do not appear: check `~/Library/Application Support/io.github.lucydjo.logpane/assets/` for a
  `<sha256>.<ext>` file and the `assets` table for the matching row.
