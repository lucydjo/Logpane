# Contributing to Logpane

Thanks for taking the time to look at the project. It is a small native macOS app written in
Rust; the fastest way to help is a focused bug report or a small, tested pull request.

## Before you start

- macOS on Apple Silicon is the only supported target.
- Read `README.md` for the build and data-location details.
- Read `docs/constitution.md`; it records the non-negotiable rules (native stack, image pipeline, persistence, local-only).
- For anything larger than a bug fix, open an issue first so the scope can be agreed.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo run
```

The app starts hidden. Press `⌘⇧D`, or run `cargo run -- --show` to see it immediately.

## Pull requests

- Keep changes focused; one concern per pull request.
- Add or update tests for model, storage, search and asset changes.
- Do not introduce network access, telemetry, a WebView, Electron or Tauri.
- Keep AppKit calls and new `unsafe` inside `src/platform/macos/`.
- Describe the manual macOS check you performed (hotkey, paste, drag-and-drop, restart).
- Never commit `target/`, `Logpane.app`, `.DS_Store` or local data directories.

## Commit messages

Short imperative subject, optional body explaining the why. Examples:

```text
fix: keep the trailing text block after an image
docs: document the local data directory
```

## Security

Do not open a public issue for a vulnerability. See `SECURITY.md`.
