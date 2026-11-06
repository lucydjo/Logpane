# Security policy

Logpane is a local-only macOS application. It makes no network requests, has no account system
and stores notes and images under `~/Library/Application Support/io.github.lucydjo.logpane/`.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use the repository's private
**Security advisory** feature (Security → Advisories → Report a vulnerability) and include:

- affected version or commit,
- macOS version and chip,
- reproduction steps,
- impact (data exposure, code execution, denial of service).

You will get an acknowledgement within 7 days. Fixes are released as patch versions.

## Scope

In scope: path handling in the image importer, SQLite usage, AppKit bridge, pasteboard and
drag-and-drop handling, data-directory permissions.

Out of scope: issues that require an attacker to already have local code execution or access to
the user's account.
