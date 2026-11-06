# Changelog

All notable changes are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to
[Semantic Versioning](https://semver.org/).

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
