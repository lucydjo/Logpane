# UI Contract — Logpane MVP

This contract fixes the user-visible behaviours and the visual language the Rust code must
implement. User-visible literals live in `src/strings.rs`; the struct keeps another language easy to
add without touching layout code.

## Visual language

Terminal-like, deliberately restrained:

- **Typography**: one monospace system font (SF Mono, Menlo fallback) for every text style;
  body 13.5 px, buttons 12.5, small labels 10.5, headings 15.5.
- **Geometry**: no rounded corners anywhere. Every surface, button, input, menu, modal and image
  is square.
- **Rules**: one-pixel borders and separators. No drop shadows, no gradients, no glass, no blur.
- **Contrast**: surfaces sit only a few values apart; secondary text stays readable but never
  competes with content. There is a single accent colour for focus, active rows and hover
  underlines.
- **Labels**: lowercase; actions are wrapped in brackets (`[ + new note ]`, `[ cancel ]`).
  macOS-native menus keep title case.
- **Spacing**: one scale — `4, 8, 12, 16, 24` (`SPACE_XS…XL`). Header, sidebar, editor insets,
  rows, buttons, menus and modals all use it; no ad-hoc values.
- **Fixed sizes**: header 40 px, control 28 px, list row 50 px, action bar 30 px, border 1 px.
- **Images**: rendered square, inside a 1 px rule, at their original aspect ratio.
- **Selection**: text selection is enabled only inside `TextEdit` widgets. Labels and list rows
  are not selectable and never show an I-beam cursor.

## Layout

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ logpane   [saved]                         ⌘⇧D   theme:dark        [x]   │  40 px
├──────────────────────────────────────────────────────┬───────────────────┤
│ Test GPT-4o vs Claude                                │ notes             │
│ november 4, 2026 · modified 10:19                    │ [ search…       ] │
│                                                      │ [ + new note    ] │
│ 09:15 — Première observation…                        │                   │
│                                                      │ today             │
│ ┌──────────────────────────────────────────────────┐ │ ┃ Test GPT-4o     │
│ │                 inline image                     │ │ ┃ 10:19           │
│ └──────────────────────────────────────────────────┘ │ ────────────────  │
│                                                      │                   │
│ [ + time ] [ + image ]                  [ delete ]   │                   │
└──────────────────────────────────────────────────────┴───────────────────┘
      editor ≈ 75 %                                        sidebar ≈ 25 %
```

- Header 40 px, theme control and hide control always visible, save badge never clickable.
- Editor column min width 420 px, sidebar 240–340 px (25 % of the window, clamped), 1 px
  separator between them.
- Window min 720 × 460 pt, default 1050 × 680 pt.
- Only the header drags the window. Editor, sidebar, images and action bar never do.
- Edge and corner resizing is native AppKit (borderless window with `Resizable`).

## Spacing scale

| Token | Value | Used for |
|---|---|---|
| `SPACE_XS` | 4 | inside-inset padding, menu margins, tight gaps |
| `SPACE_S` | 8 | button padding, row padding, block gaps |
| `SPACE_M` | 12 | header and sidebar insets, section gaps |
| `SPACE_L` | 16 | editor top inset, modal padding |
| `SPACE_XL` | 24 | editor left/right insets |

## Colours

| Token | Dark | Light |
|---|---|---|
| background | `#121314` | `#F5F5F2` |
| panel | `#151617` | `#FAFAF8` |
| raised surface | `#191B1C` | `#EFEFEB` |
| primary text | `#D2D3CF` | `#25272A` |
| secondary text | `#85888A` | `#676A6E` |
| separator | `#232527` | `#E2E2DD` |
| accent | `#8FA6B8` | `#5A7187` |
| selection | `#26313A` | `#DCE4EA` |
| hover | `#1C1E1F` | `#ECECE7` |
| danger | `#C98B80` | `#A75C50` |
| scrollbar | `#2D3032` | `#C9CAC5` |

No pure black or pure white large surfaces, no heavy contrast, no decorative colour.

## Shortcuts

| Action | Binding | Scope |
|---|---|---|
| Show / hide overlay | `⌘⇧D` | global, registered once |
| Hide overlay | `Esc` | local, ignored while a modal is open |
| New note | `⌘N` | local |
| Focus search | `⌘K` | local |
| Insert time | `⌘⇧;` | local, inserts `HH:mm — ` |
| Insert image from clipboard | `⌘V` | local, image-only; text falls through |
| Undo / redo | `⌘Z` / `⌘⇧Z` | local, native text widget |
| Hide (alternative) | `⌘W` | local |
| Quit | `⌘Q` or menu-bar item | local |

Rules:

- Global and local bindings must never both fire for `⌘⇧D`.
- While a modal is open, `Esc` closes the modal only.
- If macOS refuses `⌘⇧D`, the app falls back to `⌘⇧J` and shows it in the header. A conflict with
  another app is not detected.

## Paste decision table (`⌘V`)

| Clipboard content | Behaviour |
|---|---|
| PNG representation | import bytes → image block, remove the textual paste event |
| TIFF representation | import bytes → image block |
| File URL to an existing image | read bytes immediately → image block |
| Plain text | untouched: native text paste |
| Plain text that is exactly a path to a valid image | import that image; never insert the path |
| Plain text that only looks like a path | native text paste |
| Several image representations | exactly one block, best representation (PNG > TIFF > URL) |
| Undecodable image | `! could not import this image`, note unchanged |

## Drop contract

- Accept drops over the editor and the image area.
- Each dropped image becomes one block, in payload order, after the block under the pointer
  (or at the end of the active block when there is no hover position).
- Non-image entries are skipped with `unsupported format: <name>`; the drop never panics.
- Mixed drops import valid images and report skipped ones.

## State machine

```text
Hidden ──⌘⇧D / menu──▶ Visible(focused editor)
Visible ──⌘⇧D / Esc / [x] / ⌘W──▶ flush ▶ Hidden
Visible + delete requested ──▶ Modal ▶ confirm ▶ delete ▶ Visible
Visible + import in progress ──▶ Visible(importing image…) ▶ Visible(image)
Any state + write failure ──▶ [save error] (retry, data kept in memory)
```

Save badge states: `[saving…]` while a revision is queued or in flight, `[saved]` once the
writer acknowledges the latest revision, `[save error]` after a failed write. The badge is
never `[saved]` before the acknowledgement.

## Strings (v1)

| Key | Value |
|---|---|
| `app_name` | logpane |
| `saved` | saved |
| `saving` | saving… |
| `save_error` | save error |
| `search_placeholder` | search… |
| `new_note` | [ + new note ] |
| `notes` | notes |
| `insert_time` | [ + time ] |
| `insert_image` | [ + image ] |
| `empty_title` | new note |
| `empty_state` | start writing your observation… |
| `confirm_delete_title` | delete this note? |
| `confirm_delete_body` | this cannot be undone. |
| `cancel` | [ cancel ] |
| `delete` | [ delete ] |
| `unsupported_format` | unsupported format |
| `image_error` | could not import this image |
| `image_missing` | image not found |
| `image_loading` | importing image… |
| `theme_system` | system |
| `theme_dark` | dark |
| `theme_light` | light |
| `hotkey_conflict` | ⌘⇧D shortcut unavailable |
| `today` | today |
| `yesterday` | yesterday |
| `modified_at` | modified |
| `no_results` | no matching notes |
| `preview` | preview |
| `open` | [ open ] |
| `remove` | [ remove from note ] |
| `delete_note` | [ delete note ] |
| `export_pdf` | [ export pdf ] |
| `exported` | exported |
| `export_failed` | export failed |
| `read_only_notice` | ! content could not be read — read only |
| `close_button` | [x] |
| `image_hint` | copy an image then press ⌘V, or drop a file. |
| `menu_show` | Show Logpane |
| `menu_quit` | Quit Logpane |

Native menu: `Show Logpane`, `Quit Logpane`.

## Menus, insets and modals

- Context menus: square, 1 px rule, `SPACE_XS` padding, no shadow, items in lowercase.
- Notices and read-only banners: square raised inset with a 1 px rule and an exclamation mark.
- Action bar: `[ + time ] [ + image ] [ export pdf ]` on the left, `[ delete note ]` on the right.
- PDF export uses the native save panel and writes A4 pages with a text layer and lossless images.
- Delete modal: square panel, 1 px rule, `SPACE_L` padding, title and body left-aligned,
  `[ cancel ]` and `[ delete ]` right-aligned (danger colour).
- Preview modal: square panel, image inside a 1 px rule, `[ open ]` action and dimensions.
- All dialogs block interaction behind them and close on backdrop click or `Esc`.

## Empty and error states

- No notes at all → create one blank note before the first render.
- Missing asset file → `! image not found` inset (never the path).
- Unknown `schema_version` → read-only note with a warning inset.
- Hotkey registration failure → persistent header notice.

## Accessibility and permissions

- No Accessibility permission; the global hotkey must work without it.
- No Screen Recording permission: the app never captures the screen.
- Caret and selection must remain visible in both themes; secondary text contrast stays at or
  above 4.5:1 on its surface.
- Animation is disabled; reduced-motion and increased-contrast preferences are respected by
  construction (no motion, strong text contrast).
