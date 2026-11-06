# Logpane — project

## Promise

Record an observation at the exact moment it appears, in seconds, without opening a project,
creating a file or deciding how to organize anything.

`⌘⇧D` opens an already-initialized overlay, the caret is in the editor, a screenshot can be
pasted or dropped, and everything is saved locally with no “Save” button.

## V1 scope

Included: resident macOS app, floating overlay that can be moved and resized, multiple notes
with search, multiline editor, inline images (clipboard, drag-and-drop, temporary paths and
`file://`), immediate saving, dark/light themes, restored preferences and geometry, menu-bar
status item, PDF export with a text layer and images.

Excluded: cloud sync, collaboration, accounts, embedded AI, audio/video, OCR, publishing,
Windows/Linux, rich formatting, Markdown export, ZIP backup, configurable shortcut, auto-hide on
deactivation, pin mode.

## Reference documents

- Full specification: `specs/001-logpane-mvp/spec.md`
- Technical plan: `specs/001-logpane-mvp/plan.md`
- Data model: `specs/001-logpane-mvp/data-model.md`
- UI and storage contracts: `specs/001-logpane-mvp/contracts/`
- Manual validation: `specs/001-logpane-mvp/quickstart.md`
- Non-negotiable rules: `docs/constitution.md`
- Progress and limitations: `TODO.md`

## Structural rules

1. No image is represented by a path: bytes are copied into content-addressed storage before any
   database reference exists.
2. Every real change produces a persisted revision; the write queue is flushed before hiding,
   switching notes and quitting.
3. The stack is native Rust (egui/eframe plus a minimal AppKit bridge); no WebView, no network.
4. All AppKit calls stay in `src/platform/macos/`, and so does all `unsafe` except the file lock in
   `src/instance.rs`.
5. A macOS behaviour is only considered done after validation on a real Apple Silicon Mac; the
   interactive scenarios are listed in `quickstart.md`.
6. Labels and controls outside text editors are not selectable and never intercept clicks.
7. The visual language is terminal-like: monospace system font, square corners, 1 px rules,
   low-contrast surfaces, no shadows, and one shared spacing scale.
