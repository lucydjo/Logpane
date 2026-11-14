# TODO — Logpane

Last updated: 2026-11-14 (0.2.0: tints, Markdown export, pins, keyboard navigation, icon, releases)

## Status

Alpha. The application works and the source is ready to publish; no signed or notarised binary is
distributed. Interactive acceptance is signed off (see below).

## Open-source preparation

Done in the publication pass:

- Git repository initialised, source-only commit, `target/`, `Logpane.app`, `.specify/` and
  `.opencode/` excluded.
- MIT `LICENSE`, `THIRD_PARTY_NOTICES.md`, `Cargo.toml` metadata and a real MSRV (`1.95`).
- `rust-toolchain.toml`, `rustfmt.toml`, macOS CI (fmt, clippy, tests, release build).
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`, issue and PR templates.
- README rewritten around verified behaviour, with screenshots, status banner, troubleshooting
  and licence sections.
- Public copy of the governance rules in `docs/constitution.md`.
- Release script derives the version from `Cargo.toml`; local ad-hoc signing only.
- Screenshot/debug hook disabled in release builds.
- Save failures are propagated (writer opened before the thread starts, send results checked,
  flush result checked); future schema versions are refused instead of written.
- Image imports are capped (40 MB, 40 MP, 12 000 px side) and the original is never decoded on the
  UI thread as a thumbnail fallback.
- PDF export (A4, real text layer, Flate-compressed images, native save panel).
- Note list is re-sorted after edits; multi-file drops keep their order; paste only intercepts
  images when a text block is focused.
- A trailing editable text block is preserved after an image inserted at the end.
- Local data uses `0700` directories and `0600` files.
- User-visible literals centralised in `src/strings.rs`; bindings and global key codes
  centralised in `src/shortcuts.rs` (small bracket/quote formatting wrappers remain in the UI).

## Verified automatically

- `cargo test --locked`: 100 tests green (block model, tints, search, assets, persistence, Markdown and PDF export, migrations).
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `cargo build --release --locked`: arm64 binary produced.
- `./target/debug/logpane --smoke-test`: database created, note written and read back.
- UI verified through internal captures (light, dark, modal) with generic demo data; images are
  committed under `docs/screenshots/`.
- Menu-bar item installed at startup (`NSStatusItem`, “Show Logpane” / “Quit Logpane”).
- Native resizing confirmed at runtime (`window configured for native resizing resizable=true`).
- Non-editor labels are not selectable (`selectable_labels = false`).

## Validated manually on the target Mac

All validated by the maintainer on 2026-11-06.

- [x] Resize from edges and corners with the mouse (native AppKit).
- [x] Menu-bar icon visible; “Show” opens the overlay, “Quit” terminates the process.
- [x] `⌘⇧D` opens the overlay from Safari, Chrome, an IDE, Finder and a full-screen app.
- [x] The same shortcut hides the overlay; `Esc` hides without losing text.
- [x] Caret is in the editor on every open.
- [x] Typing, click-to-place caret, multi-line selection, copy/paste.
- [x] `⌘Z` / `⌘⇧Z` on text.
- [x] `⌘V` of a macOS screenshot shows the real image, only when the editor is focused.
- [x] Paste an image copied from Safari.
- [x] Drag a PNG/JPEG from Finder; drag several files and confirm the order.
- [x] Drag a capture exposed as a temporary path or `file://`.
- [x] Paste an image at the end of a note and keep typing below it.
- [x] Restart the app: text and images are still present.
- [x] Delete source files: images remain visible.
- [x] Move/resize the window, relaunch: geometry restored.
- [x] Disconnect the display used last, relaunch: window is brought back on screen.
- [x] Dark/light theme restored at launch.
- [x] Idle CPU < 1 % (Activity Monitor, overlay hidden).
- [x] No network sockets (`lsof -i -p $(pgrep logpane)`).
- [x] No Accessibility or Screen Recording permission prompt.

## Known limitations

1. **Structural undo**: `⌘Z` undoes text only; a removed image has its own `[ undo ]`, an inserted one does not.
2. **HEIC/HEIF** is not decoded.
3. **Enlarged preview** uses the 1600 × 1200 thumbnail, not the original.
4. **Export**: PDF and Markdown export exist; ZIP backup and confirmed orphan-asset cleanup are future work.
5. **Shortcut not configurable**: `⌘⇧J` is used only if macOS refuses `⌘⇧D`; a conflict with another
   app is not detected.
6. **No notarised build**. Bundles and releases are ad-hoc signed only.
7. **Accessibility**: list rows expose an accessible label, but full keyboard traversal and
   VoiceOver coverage are incomplete.
8. **No persistent log**: diagnostics go to stderr when launched from a terminal.
9. **No auto-hide** on deactivation and no “pin” mode.
10. **Clipboard images over 40 MB** are rejected with a message.

## Measurements (release build, target M2)

These numbers come from manual runs, not from an automated benchmark:

| Measurement | Value |
|---|---|
| Cold start → first frame → capture → exit | 1.57 s (includes the system font, OpenGL init, capture and shutdown) |
| Storage smoke test (no UI) | 0.32 s |
| Idle CPU, overlay hidden (`ps` average over 10 s) | 0.4–0.6 % (including startup) |
| Resident memory (RSS), 4 s after launch | 93 MB hidden, 141 MB with the window open |
| Release binary, arm64, stripped | 9.9 MB (0.2.0) |
| Overlay show latency from the resident process (target ≤ 150 ms) | not measured interactively |

## Next steps

1. Validate the 0.2.0 interactions by hand on the target Mac (see below), then tag `v0.2.0`.
2. Add structural undo for image insertion.
3. Add a UI integration test (`kittest`) for typing, paste focus and drop order.
4. Notarisation when a Developer ID is available; a Homebrew cask once a release exists.
5. Confirmed orphan-asset cleanup; configurable global shortcut.

## 0.2.0 — to validate manually

Verified so far by automated tests and by captures of the debug build only.

- [ ] `⌥⌘↑` / `⌥⌘↓`, and `↑` `↓` `⏎` `Esc` in the search field.
- [ ] `⌘⇧P`, `⌘⇧C`, `⌘⇧L`, `⌘/` from a focused text block.
- [ ] `[ export md ]` through the native save panel, with and without images.
- [ ] Remove an image, then `[ undo ]`.
- [ ] Typing feel and caret position in a long note with tints.
- [ ] `./scripts/build-app.sh --dist`: icon shown in Finder, `.dmg` opens.
