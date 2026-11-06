# Feature Specification: Logpane — Instant Observation Capture

**Feature Branch**: `001-logpane-mvp`

**Created**: 2026-11-04

**Status**: Draft

**Input**: User description: "Logpane — a resident macOS tool that opens a floating note
overlay with ⌘⇧D so an observation can be typed and an inline screenshot pasted or dropped,
then saved locally at the moment it is written. Interface in English, dark and light themes,
no WebView, no network."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture an observation the moment it appears (Priority: P1)

Someone is testing an AI model, comparing tools, or hunting a bug in another application.
They notice something worth remembering, press a global shortcut, and a floating window
appears over their current work with the editor already focused. They type the observation,
press the shortcut again or `Esc`, and the window disappears. The text is already saved;
nothing was named, filed, or confirmed.

**Why this priority**: This is the product's entire promise. Without a one-gesture capture
loop that never loses text, nothing else matters. It is also the smallest slice that
delivers real value on its own.

**Independent Test**: Launch the app, switch to another application, press `⌘⇧D`, type two
paragraphs, press `Esc`, then press `⌘⇧D` again. Both paragraphs are present, the caret
behaves normally, and the app never asked to save anything.

**Acceptance Scenarios**:

1. **Given** the app is running in the background with no window visible, **When** the user
   presses the global shortcut from any application, **Then** the overlay appears above the
   current application with the caret already in the editor.
2. **Given** the overlay is visible with text typed, **When** the user presses the global
   shortcut again, **Then** the overlay hides and every typed character remains in the note.
3. **Given** the overlay is visible, **When** the user presses `Esc`, **Then** the overlay
   hides without discarding content or showing a save prompt.
4. **Given** the user hid the overlay, **When** they reopen it, **Then** the same note, its
   content, the caret position, the theme and the window geometry are restored.
5. **Given** the user clicks in another application while the overlay is open, **Then** the
   overlay stays visible until the user explicitly hides it.
6. **Given** the overlay is visible, **When** the user closes it with the header close
   control, **Then** the window hides and the process keeps running for the next shortcut.

---

### User Story 2 - Attach a screenshot inside the note (Priority: P1)

While writing an observation, the user pastes a screenshot taken moments earlier, or drags
an image file from Finder or a capture tool into the editor. The image appears inline where
they are writing, at a readable size. Days later, after restarts and after the temporary
screenshot file has been cleaned up by the system, the image is still there.

**Why this priority**: The notes are about visual experiments; text alone loses the evidence.
An image that degrades into a broken path destroys trust in the tool. This is a declared
functional blocker, and it is independently testable on top of Story 1.

**Independent Test**: Paste a macOS screenshot with `⌘V`, drag a PNG from Finder into the
editor, restart the app, then delete the original screenshot from the Desktop. All images are
still rendered in the note and no filesystem path appears anywhere in the document.

**Acceptance Scenarios**:

1. **Given** the clipboard holds a screenshot, **When** the user presses `⌘V` in the editor,
   **Then** the image is inserted as a visible block at the caret and no path is inserted.
2. **Given** the clipboard holds plain text, **When** the user presses `⌘V`, **Then** the text
   is pasted exactly as before.
3. **Given** the user drags two image files from Finder onto the editor, **When** they drop
   them, **Then** two image blocks are inserted in a predictable order at the drop location.
4. **Given** a drag payload exposing only a temporary file path or a `file://` URL, **When**
   the user drops it, **Then** the file is read immediately and imported, and the temporary
   path is never stored in the note.
5. **Given** a note contains images, **When** the app is closed and relaunched, **Then** every
   image is displayed again from permanent local storage.
6. **Given** a note contains images, **When** the original source files are deleted, **Then**
   every image remains visible.
7. **Given** the user drops a PDF or a text file onto the editor, **Then** the editor shows a
   clear "Unsupported format" message, keeps the existing text intact, and does not
   crash.
8. **Given** the user pastes text that is exactly the path of an existing valid image file,
   **Then** the app imports that image rather than inserting the raw path; text that merely
   resembles a path is never converted.

---

### User Story 3 - Keep several experiments apart (Priority: P2)

The user runs multiple experiments in a day. They create a new note per experiment, find past
ones through the sidebar list or a quick search, and delete the ones that are no longer useful.

**Why this priority**: Without several notes the tool becomes a single scratchpad and loses
its value after a day. It is independent of images and can ship as soon as text capture works.

**Independent Test**: Create three notes with distinct titles, restart the app, search for a
word that appears only in the second, open it, delete the third with confirmation, and confirm
the list and the content are consistent.

**Acceptance Scenarios**:

1. **Given** the overlay is open, **When** the user triggers "new note", **Then** a blank note
   is created, selected, and focused with an empty editor.
2. **Given** a new note, **When** the user types a first line, **Then** the note title is
   derived from that first non-empty line.
3. **Given** the user has typed a title, **When** they edit the title field, **Then** the
   custom title is kept and no longer overwritten by the first line.
4. **Given** several notes exist, **When** the user opens the list, **Then** notes are ordered
   by most recently modified first and each entry shows title, excerpt and modification time.
5. **Given** the user selects another note, **Then** the editor shows that note's content and
   the previously edited note keeps every change.
6. **Given** the user types a query, **Then** the list filters to notes whose title or text
   contains it, and clearing the query restores the full list.
7. **Given** the user asks to delete a note, **Then** a confirmation is required, and on
   confirmation the note disappears from the list.
8. **Given** only one note exists, **When** the user deletes it, **Then** a new empty note is
   created so the editor is never left without a document.

---

### User Story 4 - Timestamp observations inside a note (Priority: P2)

An experiment spans the afternoon. The user inserts the current time at the caret before each
new observation, producing a readable, chronological log without typing the time by hand.

**Why this priority**: It makes a long note scannable and it is a small, self-contained
addition to the editing experience.

**Independent Test**: Place the caret mid-paragraph, insert the time, and confirm `HH:mm — `
appears exactly at the caret with the caret placed after it; repeat with no text block focused
and confirm a new text block is created.

**Acceptance Scenarios**:

1. **Given** the caret is inside a text block, **When** the user inserts the time, **Then**
   the current local time in `HH:mm — ` format is inserted at the caret.
2. **Given** no text block has focus, **When** the user inserts the time, **Then** a new text
   block containing the timestamp is created next to the active position.
3. **Given** an image block is selected, **When** the user inserts the time, **Then** a new
   text block is created after that image instead of modifying the image.

---

### User Story 5 - Work comfortably in either theme, at a stable window (Priority: P3)

The user works at night or in daylight and switches the overlay between dark and light. They
move and resize the overlay to fit beside their main work, and it comes back exactly there
the next morning.

**Why this priority**: Comfort and predictability matter for daily use, but they do not block
the core capture loop.

**Independent Test**: Move and resize the window, switch theme, quit, relaunch, and press the
shortcut. The window returns to the same size and position and the chosen theme is active.

**Acceptance Scenarios**:

1. **Given** the overlay is visible, **When** the user clicks the theme control, **Then** the
   theme switches immediately and the control shows the active theme.
2. **Given** the user chose a theme explicitly, **When** the app restarts, **Then** that theme
   is restored rather than the system default.
3. **Given** no explicit choice has been made, **Then** the app follows the system appearance.
4. **Given** the user drags the window by its header, **Then** the window moves; dragging
   inside the editor selects text instead of moving the window.
5. **Given** the user resizes the window, **Then** it cannot go below the minimum size and the
   editor column keeps the majority of the width.
6. **Given** the user moved the window on a display that is later disconnected, **When** the
   app relaunches, **Then** the window appears fully inside a visible display.

---

### Edge Cases

- The global shortcut is already claimed by another application: the overlay must still be
  reachable another way (at minimum a menu action) and the conflict must be reported without
  crashing.
- The clipboard holds several representations at once (text + image + file URL): exactly one
  image is inserted, never duplicates.
- The clipboard holds an image in a format that cannot be decoded: an error is shown and the
  note is untouched.
- A very large image (e.g. a 5K screenshot, 20 MB) is pasted: the editor stays responsive and
  the image is displayed scaled to the column width.
- The same screenshot is pasted twice: it appears twice in the document but is stored once.
- An image block is deleted while another note still shows the same image: the other note
  keeps its image.
- A drop contains a mix of images and non-images: images are imported, others are reported
  and skipped.
- The window is resized to its minimum with the sidebar at its minimum: neither column
  collapses or overlaps.
- The active note was deleted from the database outside the app (corruption or external
  edit): the app must not crash and must fall back to a valid note or a blank one.
- The disk is full or read-only during a save: the save indicator shows an error, the text
  stays in memory, and a retry is possible without losing input.
- The system boots and the app reopens automatically with no note yet: a first blank note is
  presented on first invocation.
- Text is pasted that contains only an image path which no longer exists: it is inserted as
  text, not as a broken image.
- The user hides the overlay while an image is still importing: the import completes and the
  image is present on the next opening.

## Requirements *(mandatory)*

### Functional Requirements

**Access and window behaviour**

- **FR-001**: The application MUST keep running in the background after its window is hidden,
  so the next invocation is immediate and does not rebuild the interface.
- **FR-002**: A single global keyboard shortcut MUST show the overlay from any application and
  place keyboard focus directly in the note editor.
- **FR-003**: The same shortcut MUST hide the overlay when it is visible.
- **FR-004**: `Esc` MUST hide the overlay, except when an open dialog or an internal control
  must consume the key first.
- **FR-005**: A visible header control MUST hide the overlay, and hiding MUST NOT terminate
  the application.
- **FR-006**: The overlay MUST float above normal application windows and remain usable when
  the underlying application is in full-screen mode or on another desktop space.
- **FR-007**: The window MUST be movable only from a dedicated header area, never from the
  editor surface, so text selection is never intercepted.
- **FR-008**: The window MUST be resizable from its edges and corners, with an initial size of
  approximately 1050 × 680 points and a minimum size of approximately 720 × 460 points.
- **FR-009**: The application MUST persist window position and size and restore them on the
  next launch, using logical points rather than physical pixels.
- **FR-010**: If the saved geometry points to a display that is no longer connected, the window
  MUST be repositioned fully inside a visible display.
- **FR-011**: On opening, the application MUST restore the previously active note, its
  content, the caret position when recoverable, the theme, and the window geometry.
- **FR-012**: The editor column MUST occupy roughly 72–78 % of the window width and the note
  list roughly 22–28 %, each with a minimum usable width.
- **FR-013**: Hiding and showing the overlay MUST preserve in-memory editing state so that no
  costly reconstruction occurs.

**Notes and navigation**

- **FR-014**: Users MUST be able to create a note from a visible control and from a keyboard
  shortcut, and the new note MUST be selected and focused immediately.
- **FR-015**: A note title MUST default to the first non-empty line of its text and MUST be
  editable independently once the user changes it.
- **FR-016**: The note list MUST display, for each note, its title, an optional excerpt, and
  its last modification date and time.
- **FR-017**: The note list MUST be ordered by last modification, most recent first.
- **FR-018**: The currently displayed note MUST be visually identifiable in the list.
- **FR-019**: Selecting a note MUST replace the displayed document without opening another
  window and without losing pending changes in the previous note.
- **FR-020**: Deleting a note MUST require explicit confirmation before it is removed.
- **FR-021**: Deleting the last remaining note MUST leave a blank note available so the editor
  is never empty of a document.
- **FR-022**: The application MUST record each note's creation and last modification time
  automatically, without user input.
- **FR-023**: A search field MUST filter the note list by title and body text as the user
  types, and clearing it MUST restore the full list.
- **FR-024**: A keyboard shortcut MUST move focus to the search field, and choosing a result
  MUST open that note.

**Editing**

- **FR-025**: The editor MUST support multiline entry with natural word wrapping and a
  comfortable line height.
- **FR-026**: Users MUST be able to place the caret by clicking and select text across lines
  by dragging.
- **FR-027**: Standard copy, cut and paste shortcuts MUST work on text, including across
  applications.
- **FR-028**: Undo and redo MUST work within the current note for text edits, block insertion
  and block removal.
- **FR-029**: A visible control and a keyboard shortcut MUST insert the current local time at
  the caret in `HH:mm — ` format; when no text block is focused, a new text block MUST be
  created at the active position.
- **FR-030**: The document MUST read as one continuous vertically scrolling surface composed
  of text and image blocks in order.
- **FR-031**: Text blocks MUST remain plain text with no rich-text formatting controls.
- **FR-032**: After an image import, keyboard focus and the caret MUST remain usable, with the
  caret placed immediately after the inserted image.

**Images**

- **FR-033**: Pasting an image MUST insert a visible image block at the caret and MUST NOT
  insert a file path or any textual placeholder.
- **FR-034**: Dropping one or several image files onto the editor MUST insert the corresponding
  image blocks in a predictable order at the drop position.
- **FR-035**: The application MUST accept images delivered as raw bytes, as temporary file
  paths, as file URLs, and as the standard macOS image pasteboard representations.
- **FR-036**: Image bytes MUST be copied into permanent application storage during import, and
  the source path MUST never be retained as the durable reference.
- **FR-037**: Imported images MUST remain displayable after an application restart, a machine
  reboot, and deletion of the original source file.
- **FR-038**: Images MUST render inside the document at their original aspect ratio, scaled to
  fit the editor column with inner margin.
- **FR-039**: The displayed rendition MUST use a lightweight preview rendering, while the
  original bytes are preserved for enlarged viewing and copying.
- **FR-040**: Identical image content MUST be stored only once, while still appearing at each
  insertion point in the document.
- **FR-041**: Removing an image block from a note MUST NOT delete image data that another note
  still references.
- **FR-042**: Clicking an image MUST offer an enlarged preview, and a context menu MUST offer
  opening, copying and removing it from the document.
- **FR-043**: Import MUST show a discreet loading state, then the image; a failure MUST show an
  understandable message and leave the existing note content untouched.
- **FR-044**: Dropping a non-image file MUST be handled gracefully with a clear message and
  MUST NOT crash or corrupt the editor.
- **FR-045**: When pasted text consists solely of a path to an existing, valid image file, the
  application MUST import that image instead of inserting the path; text that only resembles
  a path MUST NOT be converted.
- **FR-046**: Image decoding MUST NOT block typing or caret movement in the editor.

**Saving and reliability**

- **FR-047**: Every real content change — text, title, block insertion or removal, metadata —
  MUST be persisted automatically without any save action from the user.
- **FR-048**: Pending changes MUST be written before the overlay is hidden, before switching
  notes, and before the application quits or is terminated normally.
- **FR-049**: The header MUST show a discreet save state (`saved`, `saving…`, `save error`) that reflects the last successfully persisted revision, not merely the presence of
  text in memory.
- **FR-050**: On a write failure the application MUST keep the content in memory, display an
  error state, and retry; it MUST NOT silently discard content.
- **FR-051**: No scenario in this specification — hiding, switching notes, restarting, or
  recovering from a write error — may lose a confirmed character.

**Appearance and privacy**

- **FR-052**: A theme control MUST remain visible in the header at all times and MUST switch
  the interface immediately between dark and light.
- **FR-053**: Both themes MUST use non-pure background surfaces, readable secondary text,
  visible caret and selection, and must avoid heavy shadows, gradients, neon and glass effects.
- **FR-054**: The first launch MUST follow the system appearance; an explicit user choice MUST
  be remembered and offered as a System → Dark → Light cycle showing the active theme.
- **FR-055**: When the system requests reduced motion or increased contrast, the interface MUST
  avoid non-essential animation and strengthen contrast accordingly.
- **FR-059**: The interface MUST use a terminal-like visual language: a monospace system font,
  square corners, one-pixel rules, low-contrast surfaces, no shadows, and a single spacing
  scale shared by the header, list, editor, insets and dialogs.
- **FR-060**: Users MUST be able to export the active note as a PDF that contains a real,
  extractable text layer and the embedded images in document order, saved through a native
  macOS save dialog.
- **FR-056**: The application MUST NOT make network requests, require an account, or collect
  analytics or telemetry.
- **FR-057**: The application MUST read only files the user explicitly pastes or drops, and
  MUST NOT index or scan other files on the machine.
- **FR-058**: Diagnostic logs MUST NOT contain note text or image content.

### Key Entities *(include if feature involves data)*

- **Note**: A single experiment or topic. Has a stable identity, a title, a creation time, a
  last modification time, a revision counter, and an ordered list of content blocks.
- **Content Block**: One ordered element of a note, either a plain-text block or an image
  block. Text blocks carry editable text; image blocks carry a reference to a stored image and
  an optional caption. Each block has a stable identity so it can be moved, undone or removed
  without ambiguity.
- **Image Asset**: The permanent copy of an imported image. Identified by its content so
  duplicates collapse into one record; carries its stored location, format, dimensions, byte
  size and import date. Shared by reference across notes.
- **Preference Setting**: A remembered user choice — theme, window geometry, active note and
  other interface state — surviving restarts.
- **Window Geometry**: The saved position and size of the overlay in logical points, together
  with enough display information to detect when the original screen is gone.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With the app already running, the overlay becomes interactive within 0.15 s of
  the shortcut in at least 95 of 100 measured attempts on the reference machine.
- **SC-002**: A user can capture a text observation from shortcut to persisted note in under
  10 seconds without touching the mouse.
- **SC-003**: 100 % of images pasted or dropped in a 20-image test set are still displayed after
  an application restart and after deletion of their source files.
- **SC-004**: Zero characters are lost across 100 hide/show cycles, 20 note switches and 10
  application restarts performed during a single editing session.
- **SC-005**: 0 raw filesystem paths appear in stored note content across the full acceptance
  test suite; every imported image is rendered as an image.
- **SC-006**: Window size and position are restored within 1 point of their previous values in
  100 % of relaunch trials on the same display configuration.
- **SC-007**: Search returns the matching note within 0.2 s for a library of 1,000 notes and
  50,000 words of total text.
- **SC-008**: Keystroke response stays under 50 ms while a 20 MB image is being imported, so no
  freeze is perceived during typing.
- **SC-009**: With the overlay hidden, average CPU usage stays below 1 % over a 5-minute idle
  observation period.
- **SC-010**: Zero outbound network connections are observed during a 30-minute session that
  exercises capture, images, search and restart.
- **SC-011**: The application completes the full acceptance suite without a crash, including
  invalid image inputs, unsupported drops, and simulated write failures.
- **SC-012**: A first-time user can locate the theme control, the new-note action and the search
  field without instructions in a usability check with 5 participants.

## Out of Scope (MVP)

Cloud sync and multi-device use; collaboration, sharing and user accounts; embedded AI,
summaries or content generation; audio and video recording; OCR of screenshots; publishing to
YouTube or editing tools; Windows and Linux builds; rich-text formatting beyond plain text,
images and timestamps; automatic Markdown export; full ZIP backup and restore; freely
configurable global shortcut; auto-hide on application deactivation; "pin" mode.

## Assumptions

- The product targets a single local user on one Mac; no sharing or permissions model is needed
  in this increment.
- The reference hardware is an Apple Silicon Mac with the M2 as the primary tuning target;
  older Intel Macs are not a supported target.
- The interface ships in English; strings are centralized so another language can be added
  later without redesigning screens.
- The default shortcut `⌘⇧D` is assumed to be free. If another application owns it, the MVP
  reports the conflict and exposes an alternative menu entry; user-configurable shortcuts come
  later.
- The overlay stays visible when the user clicks in another application and only hides on an
  explicit action (shortcut, `Esc`, or header close). Auto-hide on deactivation is deferred.
- When the caret position can be determined reliably, the image is inserted there; otherwise
  it is inserted at the end of the active text block, or at the end of the document.
- Note titles are derived from the first non-empty line until the user edits the title, after
  which the custom title is authoritative.
- Search is a case-insensitive, diacritic-insensitive substring match over title and text; no
  fuzzy ranking or indexing infrastructure is expected in this increment.
- Images are never automatically deleted in the MVP, even when no note references them;
  cleanup of orphaned assets happens later without touching referenced originals.
- Enlarged image preview is included; if it proves expensive, it may be simplified to an
  open-in-external-viewer action without changing the image import requirements.
- Timestamps are stored unambiguously and displayed in the Mac's local time zone.
- Users are expected to have a working macOS screenshot mechanism; the app does not capture the
  screen itself and therefore needs no screen-recording permission.
- All data lives on the local machine in the application's private data area; crash recovery
  beyond normal restart is limited to "last persisted revision wins".
