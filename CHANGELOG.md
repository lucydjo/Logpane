# Changelog

All notable changes are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to
[Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-11-14

### Added

- Editor tints: leading `HH:MM —` timestamps, `#tags`, `` `code` ``, links, `# headings`,
  `> quotes` and `- [ ]` / `- [x]` tasks. Notes stay plain text; nothing is rewritten.
- Markdown export (`<title>.md` plus a `<title>.assets/` folder) and `⌘⇧C` to copy the note as
  Markdown.
- Pinned notes (`⌘⇧P` or right-click), kept on top of the list.
- Keyboard navigation: `⌥⌘↑` / `⌥⌘↓` between notes; in search, `↑` `↓` move, `⏎` opens and
  `Esc` leaves the search before hiding the overlay.
- Search highlights the match in the list and moves the excerpt to it; the footer shows how many
  notes match.
- Shortcut sheet (`⌘/` or `[?]`), and the essential bindings shown in a blank note.
- `[ undo ]` after removing an image from a note.
- Word and image counts under the title; image count in the note list; size and format shown
  when hovering an image.
- `⌘⇧L` cycles the theme.
- App icon, and `./scripts/build-app.sh --dist` to produce a `.dmg` and a `.zip`.
- Release workflow: pushing a `v*` tag publishes the packaged app with checksums.
- `examples/seed_demo.rs` and `scripts/screenshots.sh` to regenerate the documentation images.

### Fixed

- A scrolled note list could paint its rows over the search field.

## [0.1.0] - 2026-11-06

### Added

- Resident macOS overlay toggled with `⌘⇧D`, with native floating window behaviour.
- Block-based notes: plain-text blocks and inline images.
- Image import from the clipboard, Finder drag-and-drop, temporary paths and `file://` URLs;
  content-addressed storage with SHA-256, atomic writes and thumbnails.
- Immediate SQLite persistence (WAL), debounced saves, flush on hide/switch/quit.
- Multiple notes, search, timestamps, dark/light themes, restored window geometry.
- Menu-bar icon with “Show Logpane” and “Quit Logpane”.
- PDF export of the current note, with selectable/extractable text and embedded images.
- Terminal-like visual language: monospace system font, square corners, 1 px rules,
  low-contrast surfaces and a single spacing scale.

### Known limitations

- Text undo only; image-block undo is not implemented.
- HEIC is not decoded.
- Emoji are not displayed, and appear as `□` in PDF exports.
- No Markdown export, ZIP backup, cloud sync or configurable shortcut.
- No signed or notarised binary is distributed; build locally.
