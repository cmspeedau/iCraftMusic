# iCraftMusic Roadmap

## Purpose and overview

Unified roadmap for the native Rust, local-first player. v0.0–v0.10 cover the music product; v0.11–v0.15 cover the later media track. The media track is not a prerequisite for music 1.0.

Status terms are **Complete**, **Partial**, **In progress**, **Planned**, and **Unknown**. **Unknown** means no approach is committed and must not be presented as Planned.

> **Version:** 1.4.0
> **Date:** 2026-10-21
> **Status:** v0.0–v0.9 complete; v0.10 and the media track remain as scoped below

## Document history

### 1.4.0 — 2026-10-21

- Merged the former `ROADMAP-MEDIA.md` into this document and removed the separate media roadmap.
- Kept the media track's feature versions, status distinctions, gPodder reference, and delivery sequence in one place.
- Linked the new [Apple TV controls architecture](ARCHITECTURE-APPLE-TV-CONTROLS.md) as a proposal outside the committed roadmap.

### 1.3.5 — 2026-10-21

- Added the [Apple TV controls architecture](ARCHITECTURE-APPLE-TV-CONTROLS.md) reference for the living-room appliance and remote-control design.

### 1.3.4 — 2026-10-21

- Condensed redundant roadmap and README prose; refreshed metadata and cross-references.

### 1.3.3 — 2026-10-21

- Consolidated the README media status rows and condensed the surrounding roadmap prose.

### 1.3.2 — 2026-10-21

- Condensed the README release and v0.10 status section and aligned its roadmap link with the v0.10 heading.

### 1.3.1 — 2026-10-21

- Merged media feature versions into the `README.md` status table and unified roadmap links.

### 1.3.0 — 2026-10-21

- Streamlined duplicated roadmap framing and refreshed the table of contents.
- Added repository status badges to `README.md`.

### 1.2.0 — 2026-10-21

- Replaced stage numbering with feature-version identifiers and clarified the distinction from documentation releases.
- Tidied cross-references and refreshed the table of contents.

### 1.1.0 — 2026-10-21

- Marked v0.0–v0.9 complete and aligned the README status table with this roadmap.

### 1.0.0 — 2026-10-21

- Established the v1.0.0 documentation baseline and recorded the music roadmap
  (`ROADMAP.md`) together with the separate media roadmap (`ROADMAP-MEDIA.md`).
- Preserved the source stage plans, acceptance criteria, delivery sequences,
  constraints, and gPodder API reference.
- Added explicit version metadata and a generated table of contents.

This document follows the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
style for document changes.

## Feature-version mapping

Roadmap feature versions identify planned product increments; they are not documentation versions and do not correspond to Git release tags. The mapping is fixed for this roadmap revision:

| Former label | Feature version |
| --- | --- |
| 0 | v0.0 |
| 1 | v0.1 |
| 2 | v0.2 |
| 3 | v0.3 |
| 4 | v0.4 |
| 5 | v0.5 |
| 6 | v0.6 |
| 7 | v0.7 |
| 8 | v0.8 |
| 8½ | v0.8.5 |
| 9 | v0.9 |
| 10 | v0.10 |
| 11 | v0.11 |
| 12 | v0.12 |
| 13 | v0.13 |
| 14 | v0.14 |
| 15 | v0.15 |

## Table of contents

- [Purpose and overview](#purpose-and-overview)
- [Document history](#document-history)
  - [1.4.0 — 2026-10-21](#140--2026-10-21)
  - [1.3.5 — 2026-10-21](#135--2026-10-21)
  - [1.3.4 — 2026-10-21](#134--2026-10-21)
  - [1.3.3 — 2026-10-21](#133--2026-10-21)
  - [1.3.2 — 2026-10-21](#132--2026-10-21)
  - [1.3.1 — 2026-10-21](#131--2026-10-21)
  - [1.3.0 — 2026-10-21](#130--2026-10-21)
  - [1.2.0 — 2026-10-21](#120--2026-10-21)
  - [1.1.0 — 2026-10-21](#110--2026-10-21)
  - [1.0.0 — 2026-10-21](#100--2026-10-21)
- [Feature-version mapping](#feature-version-mapping)
- [Vision](#vision)
- [Core constraints](#core-constraints)
- [v0.0 — Project Foundation](#v00--project-foundation)
  - [Deliverables](#deliverables)
  - [Acceptance criteria](#acceptance-criteria)
- [v0.1 — Desktop UI Shell](#v01--desktop-ui-shell)
  - [Deliverables](#deliverables-1)
  - [Acceptance criteria](#acceptance-criteria-1)
- [v0.2 — Audio Playback Engine](#v02--audio-playback-engine)
  - [Deliverables](#deliverables-2)
  - [Acceptance criteria](#acceptance-criteria-2)
- [v0.3 — Music Library and Metadata](#v03--music-library-and-metadata)
  - [Deliverables](#deliverables-3)
  - [Acceptance criteria](#acceptance-criteria-3)
- [v0.4 — Functional Music Library UI](#v04--functional-music-library-ui)
  - [Deliverables](#deliverables-4)
  - [Acceptance criteria](#acceptance-criteria-4)
- [v0.5 — Playlists and Queue Management](#v05--playlists-and-queue-management)
  - [Deliverables](#deliverables-5)
  - [Acceptance criteria](#acceptance-criteria-5)
- [v0.6 — Metadata Editing and Library Polish](#v06--metadata-editing-and-library-polish)
  - [Deliverables](#deliverables-6)
  - [Acceptance criteria](#acceptance-criteria-6)
- [v0.7 — Settings, Reliability, and Performance](#v07--settings-reliability-and-performance)
  - [Deliverables](#deliverables-7)
  - [Acceptance criteria](#acceptance-criteria-7)
- [v0.8 — Visual Refinement and Accessibility](#v08--visual-refinement-and-accessibility)
  - [Deliverables](#deliverables-8)
  - [Acceptance criteria](#acceptance-criteria-8)
- [v0.8.5 — Match the Reference 1:1 (final gate for 1.0)](#v085--match-the-reference-11-final-gate-for-10)
  - [Deliverables](#deliverables-9)
  - [Acceptance criteria](#acceptance-criteria-9)
- [v0.9 — Release and Distribution](#v09--release-and-distribution)
  - [Deliverables](#deliverables-10)
  - [Acceptance criteria](#acceptance-criteria-10)
- [v0.10 — Optional Future Enhancements](#v010--optional-future-enhancements)
  - [Possible additions](#possible-additions)
- [Media track — v0.11 to v0.15](#media-track--v011-to-v015)
  - [Media track context](#media-track-context)
- [Why these stages exist](#why-these-stages-exist)
- [Core constraints](#core-constraints-1)
- [v0.11 — Media Library Generalisation](#v011--media-library-generalisation)
  - [Deliverables](#deliverables-11)
  - [Acceptance criteria](#acceptance-criteria-11)
- [v0.12 — Movies and TV Shows](#v012--movies-and-tv-shows)
  - [Deliverables](#deliverables-12)
  - [Acceptance criteria](#acceptance-criteria-12)
- [v0.13 — Podcasts and gPodder Synchronization](#v013--podcasts-and-gpodder-synchronization)
  - [Deliverables](#deliverables-13)
  - [Acceptance criteria](#acceptance-criteria-13)
  - [Recommended order within v0.13](#recommended-order-within-v013)
- [v0.14 — Apple iPod Synchronization](#v014--apple-ipod-synchronization)
  - [Why this is Unknown rather than Planned](#why-this-is-unknown-rather-than-planned)
  - [What is actually involved](#what-is-actually-involved)
  - [The three candidate approaches](#the-three-candidate-approaches)
  - [Deliverables, once the approach is chosen](#deliverables-once-the-approach-is-chosen)
  - [Acceptance criteria, once the approach is chosen](#acceptance-criteria-once-the-approach-is-chosen)
  - [What has to be decided before this stage can start](#what-has-to-be-decided-before-this-stage-can-start)
- [v0.15 — Rockbox Device Synchronization](#v015--rockbox-device-synchronization)
  - [Why the device is easy and the stage is not](#why-the-device-is-easy-and-the-stage-is-not)
  - [What a sync is (the research findings)](#what-a-sync-is-the-research-findings)
  - [The two references, and the licence boundary](#the-two-references-and-the-licence-boundary)
  - [Deliberately not in a first version](#deliberately-not-in-a-first-version)
  - [Deliverables, once the stage is committed](#deliverables-once-the-stage-is-committed)
  - [Acceptance criteria, once the stage is committed](#acceptance-criteria-once-the-stage-is-committed)
  - [What has to be decided before this stage can start](#what-has-to-be-decided-before-this-stage-can-start-1)
- [gPodder API reference](#gpodder-api-reference)
  - [General](#general)
  - [Endpoints](#endpoints)
  - [Episode action types](#episode-action-types)
  - [Error handling](#error-handling)
  - [What is deliberately out of scope](#what-is-deliberately-out-of-scope)
- [Recommended delivery sequence](#recommended-delivery-sequence)
- [Development rules](#development-rules)
- [Known constraints of this plan](#known-constraints-of-this-plan)

## Vision

Build a native desktop music player inspired by classic iTunes: a music library, album artwork, playlists, playback controls, search, and a polished desktop interface.

The application will be developed incrementally, with each stage delivering a working and testable result.

Scope of 1.0 is music, and only music. Films, TV shows, podcasts and device synchronisation live in the media track below and belong to later versions: nothing in the music roadmap waits for them, and nothing in 1.0 grows a control that gestures at them.

## Core constraints

    Rust-only application code.
    Native desktop application; no JavaScript application runtime.
    Local-first music library.
    Persistent library and playlist data.
    Modular architecture that supports future enhancements.
    Every stage must compile and preserve previously completed functionality.

## v0.0 — Project Foundation

Goal: Establish a reliable Rust development environment and application skeleton.

Status: Complete. Cargo workspace, module skeleton, typed error handling, tracing-based logging, on-disk settings, and formatting/linting/testing conventions are in place; see README.md for build and run instructions.

### Deliverables

    Initialize the Cargo project.
    Select and configure one Rust-compatible desktop UI framework.
    Establish the project directory structure.
    Configure error handling and logging.
    Create the initial application window.
    Add basic application settings infrastructure.
    Establish formatting, linting, and testing conventions.
    Document build and run instructions.

### Acceptance criteria

    The project builds and launches.
    The application displays a basic desktop window.
    cargo fmt --check, cargo check, and applicable tests pass.
    No application logic is written in another programming language.

## v0.1 — Desktop UI Shell

Goal: Deliver the initial iTunes-inspired visual experience.

Status: Complete. The egui/eframe shell provides a resizable window, navigation sidebar, content panel, persistent playback bar, placeholder library/playlist/settings views, empty states, and light/dark themes. Playback controls render as explicitly disabled states until v0.2 wires up audio.

### Deliverables

    Main application window with resizing support.
    Navigation sidebar.
    Central content panel.
    Persistent bottom playback bar.
    Placeholder library and playlist views.
    Consistent typography, spacing, icons, and colors.
    Initial light and dark theme support, if practical.
    Empty states for a new user's library.
    Responsive layout behavior for common window sizes.

### Acceptance criteria

    The layout is cohesive and visually consistent.
    Navigation changes the active content view.
    Playback controls have defined states, even before real audio playback is implemented.
    The UI remains usable when the window is resized.
    No placeholder control is presented as a fully working feature.

## v0.2 — Audio Playback Engine

Goal: Implement real music playback independently of the library interface.

Status: Complete. The `audio` module wraps `rodio` behind a small facade (output device, queue, transport, errors) and the UI shell drives it: the playback bar plays, pauses, seeks, changes volume, steps through the queue, shuffles and repeats, and reports the device, decoder and missing-file failures in place of the track detail. v0.2 deliberately stops at "load a file or folder, play it, keep a queue" — tag reading, folder scanning into a persisted library and playlists are v0.3.

### Deliverables

    Rust audio playback module.
    Load and play a local audio file.
    Pause and resume.
    Stop playback.
    Volume control.
    Seek and playback position reporting, where supported by the chosen backend.
    Track duration reporting.
    Previous and next track transitions.
    Playback queue data model.
    Shuffle and repeat behavior.
    Audio-device and decoder error handling.

### Acceptance criteria

    Supported audio files play correctly.
    Playback controls reflect the actual playback state.
    Track transitions do not leave the UI in a stale state.
    Unsupported files and missing files produce useful errors.
    Audio processing does not block the UI thread.

## v0.3 — Music Library and Metadata

Goal: Allow users to build and maintain a persistent local music collection.

Status: Complete. Folder scanning, metadata extraction, artwork, duplicate and missing-file handling, SQLite persistence, rescan progress, and actionable scan errors are implemented and verified.

### Deliverables

    Folder selection and recursive scanning.
    Supported-format discovery.
    Metadata extraction for title, artist, album, duration, genre, and track number when available.
    Album artwork extraction where supported.
    Track, album, and artist models.
    SQLite database and initial schema.
    Persistent track index.
    Duplicate detection.
    Missing-file detection.
    Rescan and refresh functionality.
    Scan progress and error reporting.

### Acceptance criteria

    Users can import a music folder.
    Imported tracks appear in the library.
    Repeated scans do not create unnecessary duplicate entries.
    Missing metadata does not crash the application.
    Scanning large folders does not freeze the interface.
    Removing a track from the indexed library does not delete its source file.

## v0.4 — Functional Music Library UI

Goal: Connect imported music to the desktop interface.

### Deliverables

    All Tracks view.
    Albums view with artwork.
    Artists view.
    Track table with sortable columns.
    Track selection and multi-selection where appropriate.
    Double-click or an equivalent interaction to play a track.
    Search by track title, artist, album, and other indexed fields.
    Sorting and filtering.
    Context menus for common track actions.
    Useful empty, loading, and error states.

### Acceptance criteria

    Library views reflect the database contents.
    Selecting and playing tracks updates the playback bar.
    Search returns relevant results without blocking the UI.
    Artwork and missing metadata have sensible fallbacks.
    Sorting and selection behave consistently.

Status: Complete. The Library section browses the indexed music as an All Tracks table, an album grid with cover art, and an artist list. The table sorts by any of its columns, filters as the search box is typed, supports click, ctrl-click and shift-click selection, double-click to play, and a context menu for playing, queueing and removing rows; removal only ever forgets the database row, and a later scan puts it back. Cover art is decoded once per album into an egui texture and re-decoded when the cache is full, with the album's initial standing in when a file has no artwork or it cannot be read. Empty, filtered-to-nothing, scanning and library-unavailable states each say what happened and what to do next. The search box itself is the playback bar's, at the bar's right end, rather than a field on the Library's heading: the bar is the one line on screen in every section, so the field and the query it holds are reachable from all of them, and the heading line goes back to naming the section. It is offered only while the Library is open — the query filters the Library and nothing else — and a query already typed waits in it across a change of section; a Clear button appears beside it while a query is on, and the field's trailing edge stays put whether or not the button is there. The Library's three pages moved to that same end of the bar: Tracks, Albums and Artists are three icon buttons — a musical note, a compact disc, and a person's head and shoulders — centred in the room between the cream centre box and the search field, offered only while the Library is open like the field beside them, each opening its page and the open one carrying the highlight that used to sit on the sidebar row. They left the sidebar, where they hung as three indented word-only rows under the Library entry with nothing to tell them apart and no room for an icon. Search and sorting run over the in-memory rows the database last returned, so typing never touches the database and cannot block a frame; the metadata editing, playlist and settings work they set up is v0.5 and v0.6. Each row also carries a **Plays** column — how many times that track has been started — and a heading to sort by it, so the most-played tracks can be put at the top. The count is the library's own and not the file's: nothing in an audio tag holds it, so a rescan cannot wipe it, and playing a file the library never indexed counts nothing rather than failing.

## v0.5 — Playlists and Queue Management

Goal: Support personal music organization and flexible listening.

Status: Complete. The Playlists section shows the open playlist's tracks numbered in play order, and the playlists themselves are listed in the navigation sidebar under the Playlists heading, so one can be picked without the section being open. The heading is the section's only way in: it carries no Browse entry above it, because a page whose whole content is a line asking you to pick a playlist is a step between you and the list sitting directly below the row. A playlist is made with the `+` in the status bar's left corner, and renamed, copied and deleted from the section's toolbar; deleting one asks first, and the question says what stays — the tracks, and the files behind them. Tracks reach a playlist from the Library's right-click menu, which puts the row under the pointer into an existing playlist or names a new one and puts it there; adding is one transaction, so an album cannot land half-added. Entries can be moved earlier and later from the buttons on each row, which swap the row with its neighbour, and a track can be taken out of a playlist from its own menu. Play and Add to queue run the whole playlist through the existing queue, which is also where the queue's own operations were finished in v0.2: queued tracks move and come out through the Queue page, and shuffle and repeat are the status bar's controls, at its left end beside the `+`. Shuffle and repeat are remembered between runs alongside the theme and volume, and are reapplied at startup. The playback bar itself is laid out around a cream centre box: the transport — previous · play · next, then the speaker and the volume slider, all in white — puts its three buttons at the window's left edge, on the same line as the box beside it, and centres the speaker-and-slider between those buttons and the box, so the volume sits in the middle of the empty bar rather than stacked against the buttons; the pill, centred on the window, holds two rows — the album cover, the track's title and state and the queue button across its top, and the seek row of elapsed time, bar and length across its foot. A row that grew is why the two are placed in rectangles of their own rather than stacked by the layout: the pill's height is a constant, and the text of a title and its state measures taller than the cover that sets the row, so a stacked layout pushed the seek bar out through the box's foot. Playlists are SQLite rows and survive a restart with their order intact; nothing in the playlist paths touches an audio file, and no queue operation writes playlist rows.

### Deliverables

    Create, rename, and delete playlists.
    Add tracks to playlists.
    Remove tracks from playlists.
    Reorder playlist entries.
    Display playlist contents.
    Play an entire playlist.
    Add tracks to the playback queue.
    Remove and reorder queued tracks.
    Shuffle and repeat controls integrated with the queue.
    Persist playlists in SQLite.
    Restore relevant playback preferences after restarting.

### Acceptance criteria

    Playlist changes persist between sessions.
    Queue operations do not corrupt playlist contents.
    Deleting a playlist does not delete the underlying music files.
    Shuffle and repeat modes behave predictably.
    Track transitions update the queue and interface correctly.

## v0.6 — Metadata Editing and Library Polish

Goal: Improve library maintenance and everyday usability.

Status: Complete. Selecting one track opens a metadata editor over the Library — title, artist, album, album artist, genre, track and disc numbers, and the cover — and nothing in it is written anywhere until one of its two buttons is pressed. **Save** stores the values in the database and marks the row as edited (`✱` in front of the title, with the reason on hover and in the editor); **Write to file** additionally rewrites the tags inside the audio file, and is the only path in the application that changes a user's file. A format whose tags cannot be written is refused with a message rather than written badly, and a row whose file has gone is refused too. A replacement cover is decoded before it is accepted and copied into the artwork cache under the track's own name, so a file that is not an image never becomes a cover. A scan never overwrites an edit while the file itself is unchanged; once the file *has* changed, the file wins again and the edit is dropped. Sorting ignores a leading article (`The Beatles` files under B), the table can be narrowed to the tracks whose files have vanished, and those rows can be forgotten in one step. Playback and navigation answer to the keyboard — space, `Ctrl/⌘ ←/→/↑/↓`, `Ctrl/⌘ L/P/,/1/2/3/E`, `Delete` and `Esc` — listed in Settings, and no shortcut fires while a text field has focus. Removing tracks and forgetting missing ones both ask first. Files in a music format the bundled decoder cannot read are counted and named after a scan instead of vanishing silently.

### Deliverables

    Edit supported track metadata.
    Update the database when metadata changes.
    Write tags to files only after explicit user action.
    Display embedded album artwork.
    Support manually selecting replacement artwork where practical.
    Improve sorting and filtering behavior.
    Add keyboard shortcuts for playback and navigation.
    Add confirmation for destructive library operations.
    Improve missing-file and unsupported-format handling.

### Acceptance criteria

    Metadata changes are validated and persisted correctly.
    File tag writes do not happen unexpectedly.
    Unsupported tag operations fail safely.
    Keyboard shortcuts do not interfere with normal text input.
    Existing library entries remain consistent after edits.

## v0.7 — Settings, Reliability, and Performance

Goal: Prepare the application for regular daily use.

Status: Complete. Everything the application remembers lives in one TOML file — theme, volume, watched folders and the output device — and is written as it changes; Settings → Playback now chooses the output device from the ones the backend offers, remembers it by identifier, and falls back to the system default with a message when the chosen device is gone, while watched folders can be dropped as well as added. The database is versioned, and a database written by an older build is copied beside itself (`VACUUM INTO`, so the copy is consistent even with a write-ahead log in flight) before a migration rewrites it; Settings → Diagnostics runs a `PRAGMA integrity_check` on demand, says what to do about a damaged file, and the recovery steps are written down in README.md. Diagnostics also reports the build, the schema version, the library counts, the paths (settings, data, database, log) and the output stream in use. Logging goes to stderr *and* to a bounded file under the data directory, rotated rather than grown forever, so a failure from a session that has already been closed can still be read. On the performance side the Library's visible rows are filtered and sorted once per change rather than once per question — the view asks for them several times a frame — the selection is re-checked against a set rather than a list, and the artist list is virtualised like the track table; a test over a 20,000-track library guards the whole pass. Regression tests cover a queue walked to its end, a folder rescanned without duplicating its tracks, a playlist reopened from its file, and the device, migration and integrity paths above. Packaging was prepared here and shipped by v0.9: a release profile, a desktop entry and an icon live under `packaging/`, README.md documents the release build and install, and since `v1.0.0` the release itself is the AppImage that pipeline publishes. Known limitation retained for a later enhancement: a scan can be stopped but not resumed after the application exits; this does not block v0.7 completion.

### Deliverables

    Persistent application preferences.
    Audio output configuration where supported.
    Configurable library locations.
    Database migrations and recovery considerations.
    Better cancellation and recovery for background scans.
    Improved error reporting and diagnostics.
    Performance testing with large music libraries.
    Regression tests for playback, scanning, and playlists.
    Platform-specific packaging preparation.

### Acceptance criteria

    Preferences survive application restarts.
    Large libraries remain responsive during common operations.
    Background tasks can be cancelled or safely completed where applicable.
    Database updates remain consistent.
    Common failures produce actionable messages rather than crashes.

## v0.8 — Visual Refinement and Accessibility

Goal: Deliver a cohesive, production-quality iTunes-inspired experience.

Status: Complete. egui 0.36 draws no focus indicator of its own and does not move focus on click, so the interface draws one: `theme::focus_ring` paints an accent band with a dark hairline outside it — two bands because no single colour clears the 3:1 an indicator owes every surface it lands on, while one of the two always does, which a test pins across the sidebar, both bars, the centre box and both theme panels. The ring sits on everything Tab can reach: the sidebar's entries and playlists, the track rows, album tiles and artists, the playlist's and the queue's rows, the transport, the queue button, the three page buttons, the volume slider, the search field and the status bar's four controls. It is the keyboard's mark and only the keyboard's: a click surrenders focus rather than taking the ring, and row selection is therefore `clicked_by(Primary)`, so egui's Space/Enter-as-click on a focused widget can never be read as pointer activity — tests drive real frames with Tab and Enter events and assert both what plays and what the selection does not. Enter on a ringed row is that row's double-click: it plays the track, the album, the artist's work, or the playlist from that entry. Tab and the arrow keys are egui's own focus walk, and Settings → Keyboard shortcuts now says so under the command table. Every glyph-only control — the three transport buttons, the queue's burger, the page buttons, the shuffle and repeat switches, the gear, the `+`, the playlist's reorder arrows, the queue's row buttons and the volume slider — hands its hover words to AccessKit as the node's name through `theme::accessible_name`, so a screen reader hears "Repeat: this track" where egui's default would read it a lone 🔁, and the words are the one string the pointer already reads, so the two cannot drift apart; the search field is named too and keeps its hint as the placeholder. The rest of the design system this stage refines was already kept in one place: spacing, corner radii, the 12-point floor on small text, the hover fills and the selection accent are all constants in `theme.rs`, and the existing contrast floors still hold under both themes. The window remains resizable down to the 760×520 the layout is designed to survive, with the table's columns dropping rather than squeezing. No transition animations were added: egui interpolates nothing without a per-frame animation pass, and a player that must never miss a beat is the wrong place to buy frames with decoration — loading is stated by the scan progress and the explicit empty, scanning and error states the views already carry. Verification: `cargo fmt --check` clean, `cargo clippy --all-targets` without warnings, the whole test suite passing, including the keyboard tests that Tab the ring onto a row, play it with Enter, Tab on, and confirm the second Enter takes over.

### Deliverables

    Refined spacing, typography, icons, and colors.
    Consistent visual treatment for selected and hovered items.
    Improved album and artist presentation.
    Better playback-bar layout.
    Loading indicators and subtle transition effects where appropriate.
    Keyboard navigation and focus indicators.
    Accessible control labels.
    Improved contrast and scalable layout behavior.
    Final light and dark theme refinements.

### Acceptance criteria

    All main views follow a consistent design system.
    Controls remain discoverable and understandable.
    Keyboard navigation works across major workflows.
    The application remains legible at common window sizes.
    Visual refinement does not compromise playback reliability.

## v0.8.5 — Match the Reference 1:1 (final gate for 1.0)

Goal: Make the window look like the reference player it is modelled on, component by component: iTunes 7 under Wine, resized to the same 1180×780 as ours so pixels are comparable. The half-number keeps v0.9, v0.10 and the media track's numbering stable; the position is deliberate — nothing ships as 1.0 until this stage is complete.

Status: Complete, component by component, each verified against a side-by-side capture before its commit. The measuring pipeline lives in `tools/reference/`: `grab.py` photographs a window from its X11 backing pixmap (the window never needs to be raised), `click.py` raises, real-clicks via XTEST, lowers and captures, `key.py` drives the reference's dialogs from the keyboard (Wine ignores synthetic clicks on menus and tab controls, so its Preferences opens only under `F10`, arrows, `Return`), and `resize.py` strips the `WM_NORMAL_HINTS` pin the reference uses to lock its own size. Every number quoted in this stage was read from a capture, not from memory; the keeper captures of both apps are committed under `tools/reference/captures/`, and `tools/reference/reference-ui.html` rebuilds the reference window, its menus and its Preferences dialog from those same measurements with a toggle that overlays the ruler. Captures of Store, account or sign-in surfaces are never kept. The sidebar (source list) is done — panel width and rule, heading colour, row height, icon column, the focused window's blue selection ramp and hover band, and the one disclosure triangle that actually folds PLAYLISTS — and so is the track table: 17px rows striped in the reference's order, the measured 219→187 header band closed by its (85,85,85) rule, and the selection the same flat blue as the sidebar's; on the flat Tracks page the header sits flush against the chrome rule, as the capture has it. The menu band is drawn over the window — the 226→190 gradient and the five labels, every item of which runs a real action — over the reflowed 44-point disc panel, and the status bar is the measured gradient strip with its (85,85,85) divider, hand-bevelled keycaps and counts. The Settings window is the measured sheet: 573×648, its dark title strip with a drawn close mark, tabs standing on the white pane's rule, hairlines parting the pane's groups, and an OK that closes and a Cancel that puts back what the window changed.

### Deliverables

    Capture pipeline: per-window photographs of both apps at a matched size, plus a click harness for states that only appear under a real pointer. (Complete.)
    Sidebar source list, 1:1 including selection, hover, the fold, and the smart-playlist badge. (Complete.)
    Track table: measured striping, 17px rows, gradient header band with its bottom rule, flat selected row. (Complete — first row striped, the 219→187 band closed by its (85,85,85) rule, the selection the focused window's blue, and on the flat Tracks page the band sits flush against the chrome rule, the inset the other pages keep collapsing to nothing where the capture has none.)
    Transport bar and centre chrome: measured top gradient, pale-green disc panel and border, dark rule under the chrome, toolbar strip. The centre button keeps our house disc — the Apple logo is not ours to ship. (Complete — the menu band is drawn over the window: the 226→190 gradient under the five labels, each label opening a menu whose items all run real actions, and the centre box reflowed into the 44-point disc panel the band measures, house disc and all.)
    Status bar: measured height, gradient, top divider and centred text colour. (Complete — 28 points, the 224→149 gradient over the (85,85,85) divider, the counts in their measured ink on bevelled keycaps painted under transparent widgets.)
    Settings window: take the reference dialog's layout — one window, a tab strip across the top, label-right-aligned rows with the controls, hairlines across the pane between groups, and a footer held off by a full-width rule with OK and Cancel right-aligned (measured from the live 573×648 Preferences capture). Ours keeps its own tab set and music-only content — no Podcasts, Store or device tabs are copied — but the structure becomes the reference's. (Complete — six tabs stand on the white pane's rule, the chosen one merging into it; the pane's groups are parted by the measured hairline; the footer carries the blue-ringed OK, which closes, and the Cancel, which closes and puts back the volume, crossfade and output device from the snapshot taken the moment the window opened. The Help menu's two questions open the window on the pane that answers them.)
    Every row and control painted here must also be wired: nothing is added to look right that does nothing. This rules out the reference's Movies, TV Shows, Podcasts and Store rows — 1.0 is music, so those rows arrive with the later versions that give them something to open. Equally, no Store page, account or sign-in surface is ever reproduced: the fidelity pass copies the music player and leaves the shop alone.

### Acceptance criteria

    Side-by-side captures of ours and the reference at the matched size are indistinguishable per component, apart from deliberate exceptions (the house disc mark, and content — our library is not theirs).
    Measured colours and metrics live in `src/ui/theme.rs` and are pinned by tests, so a regression fails the build instead of drifting.
    The functional suite stays green across the pass: matching the look must not break drop-to-add, playlists, the queue, or search.
    Photographs of the reference remain outside version control.
    v0.9 is complete and publishes only after this reference-fidelity gate is closed; the 1.0 release shipped after that gate.

## v0.9 — Release and Distribution

Goal: Produce a dependable installable application.

Status: Complete. The `v1.0.0` tag — pushed once the v0.8.5 gate closed — built the locked binary into `iCraftMusic-1.0.0-x86_64.AppImage` and published it as a GitHub Release; the release job checks the finished image's own payload before it will publish it, and the asset's name is the one the in-app update check finds. Clean-install and upgrade testing, the final dependency and license review, and the written release checklist are complete for the 1.0 release.

### Deliverables

    Release build configuration. (Landed: `.github/workflows/ci.yml` runs the fmt/clippy/test gate on every push and pull request, and `.github/workflows/release.yml` packs the locked release build into an AppImage and publishes it as a GitHub Release when a `v*` tag is pushed — the AppImage is the shape the in-app updater can swap, and the tag is what says the v0.8.5 gate closed. The first release it made is `v1.0.0`, published 21 October 2026.)
    Platform-specific build and packaging instructions. (Landed for the one declared platform: README.md gives the Linux/x86_64 build, the linuxdeploy AppImage bundling, and the desktop entry and icon install.)
    Application icon and version information. (Landed: `packaging/musiccraft.desktop` and the `packaging/icons/` PNG set go inside the image, and the version the About pane, the Diagnostics pane and the update check quote is the crate's own, stamped in at build time by `CARGO_PKG_VERSION`.)
    Clean-install and upgrade testing. (Complete.)
    Final dependency and license review. (Complete.)
    User documentation. (Landed: README.md is the user's book of the application — features, shortcuts, settings, storage layout, recovery steps.)
    Known-limitations documentation. (Landed: README.md says plainly what the crossfade, the equalizer and the smart-playlist rules do not yet do, and exactly what leaves the machine.)
    Release checklist and versioning strategy. (Complete; documented in VERSIONING.md and the release workflow.)

### Acceptance criteria

    A release build can be generated reproducibly.
    The application launches on each declared target platform.
    Core playback and library workflows pass release testing.
    Required runtime dependencies are documented.
    No critical known issue is hidden or misrepresented.
    The reference-fidelity gate (v0.8.5) is complete.

One part of this stage has been in the tree since before the first tag: the
application checks the releases API once at startup and shows a notice on the
status bar when a newer release than this build exists (see `src/update.rs`).
The first release gave the check its first honest answer: a build at `1.0.0`
against the published `v1.0.0` hears nothing, because the released version is
not newer than itself, and the notice will say so the moment there is a newer
one to install. It runs on its own thread, and a failure is not reported —
not knowing whether there is an update is not a fault.

## v0.10 — Optional Future Enhancements

Goal: Extend the application after the core product is stable.

### Possible additions

    Smart playlists and saved filters (the smart half is implemented — see the
    status below).
    Album detail view below the album grid, as iTunes 11 shows it (design
    agreed — see the status note below).
    Artist master–detail view, as iTunes 11 shows it (design agreed — see the
    status note below).
    Lyrics display and synchronized lyrics.
    Equalizer and audio effects.
    Crossfade (the fade half of the gapless item below).
    Ratings and listening history (play counts are counted already — see the
    Status section below).
    Playlist import and export.
    Album cover download from MusicBrainz's Cover Art Archive (implemented —
    see the status below).
    Import from an iTunes library (design agreed — see the status note below).
    External media-key integration.
    Additional audio formats and decoder backends.
    Portable mode.
    Optional device synchronization. (This one has grown into a stage of its
    own: v0.15, Rockbox device synchronization, in the media track, marked
    Unknown.)
    Optional network streaming or remote library integrations.

These features are not prerequisites for the initial release. Each must be assessed for technical complexity, dependencies, privacy, licensing, and platform support before implementation.

Status: Partial. Five of the additions are implemented: gapless playback, a
crossfade, a graphic equalizer, smart playlists, and album cover download.

Gapless playback: the queue chooses the track that follows as a track starts and
the engine hands that track's decoded audio to the output device behind the
current one, so rodio runs the two together with no silence between them; the
change-over is reported from the audio itself — a signal set the instant the last
sample is pulled — rather than noticed a frame late. It works for every track
order the queue can name ahead of time, including `Repeat one` and a repeating
`Repeat all`, and one deliberate exception is documented in README.md: an edit to
the queue in the moment before a change-over makes the player fall back to a
normal load, which a listener may hear as a short gap.

Crossfade: a length above zero blends the two tracks instead, on the engine's two
decks — the outgoing track fading out while the incoming one fades in — with
equal-power gains so the change does not dip in the middle. A length of zero
keeps the gapless hand-over. The fade is stepped on the frame clock rather than
per sample, and needs a track whose length the decoder reported.

Equalizer: a ten-band bank of peaking biquads with a preamp, an on/off switch and
named presets, applied to every track before it reaches the device. Each track
re-reads the settings as it runs, so moving a slider is heard on the track
already playing.

Smart playlists: a playlist can be filled by rules instead of by hand — match
all or any of the rules on title, artist, album, genre, track number, play
count, time or date added, optionally held to N tracks selected by most played
or recently added, which is the shape iTunes 7 taught. What a smart playlist
holds is its rules and nothing else: its tracks are whatever the library
matches each time it is read, so a track indexed later is already in the
playlist when the library is read again, and there is no membership to keep in
step. A rule set compiles to one SQL query with its values bound, so the
library answers its own question and typed words never reach the database as
SQL. A smart playlist takes no hand-added track, has no order to rearrange and
no track to take out, and the interface withdraws those operations at every
surface rather than offering and refusing them; the edit it does have — its
rules — is opened by **Edit rules…** on the section heading. The status bar's
`+` offers both kinds now, a menu where it used to be a single jump to the
name editor. Rule fields are limited on purpose to columns the library already
keeps: year and rating are not offered as rules until the ratings and iTunes
import notes below bring their columns.

Album cover download: a record whose files carry no embedded picture can be
given one from the outside, from MusicBrainz's Cover Art Archive. The ask is
always the user's — the album screen's **Download artwork** button, or a
right-click on a sleeve in the grid — and one cover is on its way at a time,
so the second ask waits while the first is out. The walk is MusicBrainz's
release search for the exact album by that exact artist, then the archive's
front image for the release chosen, preferring an official dated release
credited to the artist so a near-name cannot drop someone else's sleeve on the
record. What comes back is written where an extracted cover would be — each
track's cache file at the name the cache already uses, and the row's
`artwork_path` — so the grid, the album screen, the transport bar and the
playback bar draw it with no new code path, and no audio file is touched. It
is the second of the application's exactly two network requests, and it sends
nothing but the album and artist strings of the one record it was asked about,
under a user-agent that names the build and gives a page a human can answer
on. An `#[ignore]`d test walks the real endpoints so the contract is checked
without a test run depending on a network.

The remaining additions above are unstarted — with one in
design: the album detail view is agreed and written down below, so the session
that builds it does not have to re-argue it.

Album detail view: designed October 2026, not started. Clicking an album on the
Albums page is to open the panel iTunes 11 opens under the grid, replacing what
the click does now, which is typing the album's name into the search field and
jumping to the All Tracks table. The data comes first: tracks gain a **year**,
kept as the string the tag carries (`TYER` and `DATE` hold `2007/UK` as often as
`2007`) and displayed only as a leading four digits when there is one, added by
one schema migration, settable in the metadata editor, and summed into an
album's year by the rule the album artist already follows. The state is one
`Option` in `LibraryBrowse`, keyed by the album name the queue commands already
use: it opens on a single click, swaps to another album, toggles back from the
same one, closes on its ✕ or on Esc, and survives whatever the search field
filters the grid down to — the panel is the album's own, not a search result.
Clicking an album tile never touches the search field again; double-clicking
still plays the album, the tile's context menu keeps Play, Queue and **Show its
tracks**, and the Artists page keeps its current shape until the artist
master–detail view noted below replaces it. The panel fills the bottom of the
library's centre area under the scrolled grid: the title, a credit line of
artist, year, track count and total time, Play / Shuffle / Add to queue
buttons, and the numbered track list in two columns when the panel is wide —
split at a disc boundary when the release has more than one — where
double-clicking a row plays the album from that track. The cover sits enlarged
and blurred at the right as the backdrop, the blur a downscale and bilinear
return cached per album beside the artwork cache's cover rather than a filter
run every frame, and the panel is tinted with the theme's own colours. What
does not cross over is the screenshot's other half: there is **no “In the
Store” button** and nothing store-shaped is planned beside this panel, which
belongs to the library and not to a shop.

Artist master–detail view: designed October 2026, not started; it supersedes the
sentence above promising the Artists page is unchanged. The Artists page becomes
the two panes iTunes 11 gives it, inside the central area: a selectable,
alphabetical artist list on the left — the initial tile standing in for the
avatar, as it does for a missing cover in the album grid — and the selected
artist's discography on the right. That shape deliberately differs from the
album page's list-plus-panel-below, for the reason iTunes itself had: a
discography stack needs the height. The right pane opens with the artist's name,
an “N Albums · M Songs” line, and Play / Shuffle / Add to queue buttons; below
them, one section per album — cover at the left, “Title (1980)” as its header,
the numbered track list in two columns — and double-clicking a row plays that
album from that track. An album earns a section when the artist has credited
tracks on it, and the section lists **every** track of that album, whole
compilations included; the alternative, credited-tracks-only, reads as a
truncated album whenever anyone else plays on it. The machinery is shared with
the album view and the build order follows from that: the year layer first, then
one two-column track-list helper serving both views, then the two pages.
Selecting an artist never touches the search field; search filters the left
list only, and the selected artist's pane persists while valid, under the same
rule the album panel lives by. No Gallery, no Store, no tabs.

iTunes library import: designed October 2026, not started. A one-off migration,
not a sync: Settings → Library management offers **Import from iTunes…**, which
opens the file picker at an `iTunes Library.xml` — Apple's own documented plist,
read with one crate, no reverse-engineering and nothing that asks iTunes to
still exist. Import invents nothing and changes no file: the user has already
added the `iTunes Media` folder — that folder tree works as a scanned folder
today — and the import matches the XML's entries by path onto the indexed
tracks, applying library state only. `Play Count` goes into the existing Plays
column under `max(existing, imported)`, so playing since the last iTunes is not
lost. `Rating` lands on a new rating field at 0–10 — the unbuilt *Ratings and
listening history* addition finally has its reason, and 0–10 is the scale
Rockbox's runtime data uses too (v0.15), so one scale serves the whole
application. `Year` lands on the year the album and artist views already need,
the import doubling as a backfill where a tag lacks one; date-added follows the
same path. `Master Playlists` become playlists in order through the existing
create-or-find path, the master pseudo-playlists skipped, and the smart ones —
which speak iTunes' private query language — skipped and named rather than
silently dropped. The `Album Artist Location` paths feed the artwork cache for
albums that carry no embedded cover. The honesty rules are gPodder's, restated:
a track matching nothing is counted and named, not swallowed; a missing XML
says plainly that iTunes keeps it private until *Share XML with other applications* is ticked —
and in the newer Music apps, which generate no XML at all, that it is a
one-time **File → Library → Export Library** away — a library that is `.itl`
only is refused with the reason: the binary format is reverse-engineered, and
this project does not read it. The modern-looking alternative database is not
the relief it appears: the Music app's `Music Library.musiclibrary` package
holds `Library.musicdb` and friends whose files begin with an **`hfma`**
header — a FileMaker-lineage binary, not SQLite — undocumented, and publicly
readable only through Apple's macOS-only `iTunesLibrary` framework. That is a
second `.itl`, not an openable database; a parse failure changes nothing. Not
in the design, and on purpose: no `.itl`, no `.musicdb`, no moving or exporting
folders, no live connection to a running iTunes.

## Media track — v0.11 to v0.15

### Media track context

The music track above covers v0.0–v0.10. This section covers the later media track: v0.11 generalisation, v0.12 movies and TV, v0.13 podcasts and gPodder synchronization, and the undecided device-sync stages. None is a prerequisite for music 1.0; the shared status table is in [README.md](README.md). The [Apple TV controls architecture](ARCHITECTURE-APPLE-TV-CONTROLS.md) documents the living-room appliance, remote-control behavior, and Kodi handoff design as a proposal; it is not a committed stage, and its internal Stage 0–5 sequence is its own implementation plan rather than a renumbering of these feature versions.

v0.14 and v0.15 remain **Unknown**, not Planned: v0.14 has three candidate approaches but no choice, while v0.15 has research but no committed scope.

## Why these stages exist

Everything a film, an episode or a podcast episode needs is already in the
application, in music-shaped form:

| The library already has            | A film / episode needs                              |
| ---------------------------------- | --------------------------------------------------- |
| A row per file with a title, artist, album, genre, duration and a play count | A row per file with a title, series, season/episode, duration and a resume position |
| A scanner that walks watched folders and skips what it cannot read | The same walk, with a wider set of readable formats |
| A sortable, filterable, selectable table with artwork | The same table, with the columns the media kind actually has |
| A queue, a transport, shuffle and repeat | The same transport, playing a picture-less stream   |
| Playlists                                                        | Collections that span kinds |
| An SQLite library with a versioned schema and backups            | More tables, a migration, and the same backup rule  |

So the work is not "build three more applications". It is "generalise the one
that exists, then add the two surfaces the new rows need".

## Core constraints

These extend [the music-track constraints](#core-constraints): Rust-only and local-first; user-requested network access only; no regression of completed behavior; every stage must compile and preserve prior acceptance criteria; decoder choices must document capability and build cost; undecided work remains **Unknown**, not Planned.

---

## v0.11 — Media Library Generalisation

**Goal:** one library, one player, and three kinds of row — so that v0.12 and
13 have somewhere to put their media rather than each growing their own store.

**Status:** Planned.

This is the stage that decides whether the rest is cheap or expensive. If a film
is a second class of thing with its own scanner, its own tables and its own
transport, then the queue, the artwork cache, the search field, the selection
model and the play counts all have to be built twice more. If instead the
library holds *items* of which music, film and podcast episode are kinds, all of
that comes for free and the later stages are mostly UI.

### Deliverables

- A `MediaKind` for each row: music track, movie, TV episode, podcast episode.
- A schema migration that adds the kind and the fields the new media need
  (season, episode, series, feed URL, episode GUID, resume position), keeping
  the music columns where they are.
- The scanner split into two halves: one that walks folders and classifies what
  it finds, and one per kind that reads a file's metadata. A kind that has no
  reader yet is still indexed with what the file name says, exactly as an
  unreadable music format is counted and named today.
- The player made format-agnostic: a stream is a stream, and the transport does
  not care whether the bytes behind it are audio or a video container's audio
  track.
- One search, one sort, one selection model and one queue across all kinds, with
  the Library's pages gaining the kind filter that the columns already imply.
- The existing music tests still pass, unchanged, with the kind defaulted to
  music — that is the proof the generalisation did not move the ground.

### Acceptance criteria

- A music-only library behaves exactly as it did before the migration, and the
  pre-migration backup the schema upgrade takes is present.
- `MediaKind` is stored on every row and every existing row reads back as music.
- Searching for a term matches across kinds rather than only within the open one.
- The queue can hold a mixed list and play through it without stopping on the
  change of kind.
- Nothing in the UI claims a feature that v0.12 or 13 has not delivered yet.

---

## v0.12 — Movies and TV Shows

**Goal:** index, browse and play films and television, locally, with the same
library and the same player the music uses.

**Status:** Planned.

### Deliverables

- Containers: demuxing and playback for the formats a film actually arrives in
  (the `.mp4`/`.m4v` family first, Matroska `.mkv` after, then the legacy
  `.avi`, `.wmv` and `.mov` cases as their decoders allow).
- Video output in the existing window: a picture surface the transport can
  drive, sized to the content's aspect ratio, with the controls hiding when the
  pointer leaves and returning when it comes back.
- A film library view: a poster grid and a list, sorted by title, year, series
  or added date.
- Series grouping: a folder of episodes is one show with seasons, not forty
  films, recognised from the file name — `Show Name/S02/S02E07 …` and the
  conventions beside it — with a manual override for the names that do not fit.
- Resume: where a film or episode was stopped, remembered per item, and offered
  as "Resume" rather than a bare "Play".
- Watched state, cleared one item at a time or in a batch.
- Subtitles and audio-track selection: at minimum, choosing between the streams
  a container carries and reading the sidecar `.srt`/`.ass` files beside it.
- Artwork from the files beside the media, at a size and shape the poster grid
  asks for — reusing the cover-art cache rather than growing a second one.
- An iTunes library as a source, noted and not committed: `Movies` and
  `TV Shows` are plain folders under `iTunes Media` for the scanner as it
  already works, and the iTunes XML import noted in [the music track](#vision)
  v0.10 carries movie and TV-show rows, so that import — if built — is a
  candidate source for this track's kinds.

### Acceptance criteria

- A folder of films and a folder of a series are each imported as the right kind
  without renaming anything by hand.
- A film plays to its end, seeks, pauses, changes volume and steps to the next
  item from the same transport the music uses.
- Stopping an episode and reopening the application offers it back where it was
  left, and does not offer it once the episode has finished.
- A container the build cannot demux is counted and named rather than silently
  skipped, matching how an unplayable music format is handled.
- Playback of a video file does not block the interface, and the picture does
  not tear or stutter while the library is being scanned.
- A subtitle file in the wrong encoding is refused with a message rather than
  drawn as mojibake.

---

## v0.13 — Podcasts and gPodder Synchronization

**Goal:** subscribe to podcast feeds, download and play episodes, keep a resume
position per episode, and synchronize subscriptions and listening state with a
gPodder server — gpodder.net or any self-hosted instance — so the position on a
phone and the position on the desktop agree.

**Status:** Planned.

### Deliverables

- Feed discovery and subscription: a URL or an OPML file in, a feed that has
  been fetched, parsed and validated (RSS 2.0, Atom, and the iTunes podcast
  extensions) out, with the feed's own artwork and description read from it.
- An episode list per feed, with published date, duration, size and download
  state, sorted oldest-first or newest-first.
- Download management: fetch an episode into a directory the user chooses, show
  progress, resume an interrupted download, and delete a downloaded file without
  unsubscribing from the feed.
- Streaming playback of an episode that has not been downloaded, and the choice
  between the local file and the stream when it has.
- Resume position and played state per episode, from the item's own identity
  (feed URL plus episode GUID or media URL), so a re-fetch of the feed does not
  lose them.
- gPodder's Advanced API: device registration and captioning, subscription
  upload and download with `since` timestamps, and episode-action upload and
  retrieval. The endpoint reference is
  [below](#gpodder-api-reference).
- Credentials stored where the rest of the settings are, with the password never
  written to the log and never echoed back into the interface.
- Sync that degrades honestly: a server that is unreachable leaves the local
  state untouched and says so, and a failed sync retries rather than losing the
  local changes it was trying to send.
- OPML import and export, so a subscription list can leave and arrive without
  the server being involved at all.
- And not from iTunes: the XML import noted in [the music track](#vision)
  v0.10 lists podcast episodes as flat tracks with no feed or subscription
  state behind them, so podcast arrival stays on the feed-and-OPML path above;
  the Audiobooks rows the same XML carries belong to no stage in this roadmap
  yet.

### Acceptance criteria

- A feed URL subscribes, lists its episodes, downloads one and plays it from
  the local file.
- An episode that has not been downloaded plays as a stream, and says that is
  what it is doing.
- Stopping an episode records a position, and reopening the application offers
  the item back from that position.
- Subscriptions listened to on the desktop appear on a second client after a
  sync, and the reverse — a subscription added elsewhere arrives here.
- A sync against an unreachable server fails with a message, changes nothing
  locally, and can be retried.
- No credential, key or episode URL is written to the log.
- Importing an OPML file subscribes to every feed in it, and exporting one
  produces a file another podcast application can read.

### Recommended order within v0.13

1. Feed fetch, parse and subscribe (no server involved).
2. Episode list, download, and local playback with resume.
3. Device registration and subscription sync.
4. Episode-action sync, which is what makes resume positions travel.
5. OPML in and out, and self-hosted instances as a first-class case.

---

## v0.14 — Apple iPod Synchronization

**Status: Unknown.**

The work here is not "add a sync target". It is "write to a database that is
checksummed, on hardware that is no longer manufactured, through a C library
whose maintenance has been intermittent for years" — and the honest position is
that the approach has not been chosen yet, not that it is merely unstarted. What
follows is the research, so that whoever picks this up starts from the facts
instead of rediscovering them.

### Why this is Unknown rather than Planned

Every other stage here has an approach. v0.11's is a `MediaKind`; v0.12's
is a demuxer plus a picture surface; v0.13's is gPodder's HTTP API. This stage
has three candidate approaches, none of which is obviously right (see below), and
the choice between them decides the whole shape of the work. Marking it Planned
would mean inventing a plan to have one.

### What is actually involved

An iPod does not read the files on it — it reads a database describing them, and
since the iPod classic that database carries a cryptographic hash that the device
verifies. Adding a track means rewriting the database and recomputing that hash,
not just copying a file in.

| Model family                        | What the write needs                                                                    |
| ----------------------------------- | ---------------------------------------------------------------------------------------- |
| iPod 1G–3G, Mini 1G–2G              | A plain `iTunesDB`. No hash, no sqlite.                                                    |
| iPod 4G–5G, Nano 1G–2G              | `iTunesDB` plus a `SysInfoExtended` file                                                  |
| iPod Classic, Nano 3G/4G            | As above, plus a **hash58** computed over the database                                     |
| Nano 5G, iPhone/iPod touch (OS 2.x/3.x) | A compressed `iTunesCDB`, **hash72**, and checksummed sqlite databases plus a `.cbk` file |

The hash is the whole difficulty:

- **hash58** is derived from the device's FireWire ID, which `libgpod` reads off
  the device over SCSI/USB.
- **hash72** is *not* derivable. It needs a `HashInfo` file that only iTunes, or
  a device that has been synced by iTunes at least once, can produce. Without it
  the database is written and then silently rejected — the classic "syncs fine,
  unplug and nothing is there" failure.
- **`SysInfoExtended`** is a plist describing the device's capabilities, produced
  by libgpod's `ipod-read-sysinfo-extended` helper (installed as a HAL/udev
  callout, or run by hand).
- On Nano 5G and later, the sqlite databases must be generated and then have
  post-processing SQL run against them, which on some models is only obtainable
  from the device itself.

None of that is a library problem; it is the device's own design, and it means
supporting "iPod" is really supporting a matrix of generations with different
requirements.

### The three candidate approaches

| Approach                                              | Pros                                                                 | Cons                                                                                                              |
| ----------------------------------------------------- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **`libgpod` bindings** (FFI)                           | The formats, the hashes and the device quirks are all implemented and battle-tested by a decade of use | C dependency; `libgpod`'s own support for the newer models is partial; the maintainership is intermittent; **its licence is LGPL**, which matters for a distributed binary |
| **A pure-Rust implementation of `iTunesDB`/`iTunesCDB`** | No C dependency, and an all-Rust tree, which the project treats as a constraint | Reimplementing a checksummed binary format, and the hash72 problem does not go away — it is a hardware limitation, not a library one. Nothing else in the ecosystem has done this |
| **Do not sync, export**                                 | Honest, cheap, and never writes to the device                          | Not really iPod sync; the device is read-only to us, and the user copies files themselves                          |

A fourth possibility deserves naming and is not recommended: several older tools
sidestep the hash by restoring an older firmware or by exploiting models that do
not verify it. That is not something this project will do.

Flashing Rockbox is a different thing, and it has a stage of its own: a device
whose owner has installed Rockbox reads the filesystem rather than a verified
database, so the whole matrix above stops applying to it. Syncing such a device
is [v0.15](#v015--rockbox-device-synchronization)'s work, and that work
would only ever happen on a device the owner flashed — flashing itself is Rockbox
Utility's job, not this application's.

### Deliverables, once the approach is chosen

Provisional, and to be rewritten when it is:

- Device discovery over USB and mounting, on Linux first, with the other
  platforms named and not pretended.
- A capability probe: read the model and generation, and say plainly which
  requirements that device has and whether they are met.
- `SysInfoExtended` and — where the device allows it — `HashInfo` handling, with
  the "this device needs a HashInfo that we cannot generate; sync it once with
  iTunes, or paste your serial number into ihash.marcan.st" case surfaced to the
  user as what it is, and not as a crash.
- Database read, then write: build the new `iTunesDB`/`iTunesCDB` from the
  library, recompute the hash for the generation in hand, and write it back.
- Track and playlist mapping, including artwork in the size the device asks for.
- A dry run that reports what would change before anything is written, because
  this is the one place in the application that writes to a device rather than a
  file, and the blast radius of getting it wrong is a corrupted library on
  hardware the user cannot easily fix.
- Eject, and a refusal to write while a write is in flight.

### Acceptance criteria, once the approach is chosen

Not written yet, because acceptance criteria for an unknown approach would be
theatre. Two are already fixed regardless of the approach:

- A device whose generation this build cannot support safely is **refused with a
  message naming the generation and the reason**, never written to speculatively.
- Nothing in the interface offers to sync a device until a dry run has read that
  device successfully.

### What has to be decided before this stage can start

1. `libgpod` FFI, pure Rust, or export-only — and if FFI, whether an LGPL
   dependency is acceptable in a distributed binary, and how it is linked.
2. Which generations are in scope. "iPod" as one word hides a matrix; a first
   version that supports the un-hashed classics and refuses the rest honestly is
   a legitimate answer, and a better one than claiming all of them.
3. What happens to a user on a device that needs `HashInfo` and does not have
   one. There is no good answer here, only several bad ones, and the choice
   should be made deliberately rather than discovered in a bug report.

---

## v0.15 — Rockbox Device Synchronization

**Status: Unknown.**

v0.14 is Unknown because it does not know how it could be built. This stage is
Unknown because nobody has decided that it will be: the research is finished and
is recorded below — RockSync and iPodRocks have already paid the trial-and-error
tax on real hardware — but scope, shape and the commitment itself have not been
decided, and an uncommitted stage is Unknown, not Planned.

### Why the device is easy and the stage is not

None of v0.14's difficulty is here. A Rockbox player has no `iTunesDB`, no
hash58 or hash72, no `SysInfoExtended`; it browses the filesystem directly, and
copying files into a known layout is a class of operation this application
already performs. The pieces are already in the tree — lofty reads and writes the
tags Rockbox reads, the artwork cache already holds the covers, and the library
already knows where every file is. What is missing is not capability but the
decisions: where the first version stops, and how much of Rockbox's own data the
application ever touches.

### What a sync is (the research findings)

| Fact                     | Detail                                                                                                                                                  |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Spotting a device        | a mounted USB mass-storage device carrying a `.rockbox/` directory in its root. The firmware version is readable from the `Version:` line of `.rockbox/rockbox-info.txt` **only when that file exists**: it is written by Rockbox Utility (`utils/rbutilqt`), not by the firmware, so a hand-installed device has no version we can read and its capabilities stay unverified |
| What it plays            | the same tag formats lofty already handles; lossy and ≤CD-quality lossless copy verbatim. The references treat **16-bit / 44.1 kHz** as a ceiling, but the firmware ships a software resampler in the codec DSP chain (`lib/rbcodec/dsp/resample.c`), so whether a hi-res file plays depends on the target, its build and CPU headroom — a per-device unknown, not a firmware law. Treat hi-res as unconfirmed until tried on the owner's target |
| The filesystem           | usually FAT32: 2-second mtimes read as UTC (mishandling the clock silently forces a full on-device re-index), FAT-unsafe characters `<>:"/\|?*` replaced in every name component, names compared NFC-lowercased, and a volume token (`/<HDD0>/…`) stripped from device-side paths |
| Cover art                | one `cover.jpg` per album folder (~240–300 px, never upscaled); embedded art optionally stripped from the copied file                                    |
| Playlists                | `.m3u8`, UTF-8, device-relative paths — and this application cannot yet write playlist files for any target, which is why export lands first                |
| Identity                 | USB vendor + product + serial (`/sys/bus/usb/devices`, interface class `08`) pinned per device, so a second player never inherits the first one's sync state |
| Eject                    | Linux: `udisksctl unmount -b <blockdev>` with the block device parsed from `/proc/mounts` (undoing `\040` escapes), falling back to `umount`                 |
| The hazards              | transient write failures are `ETIMEDOUT`, `ECONNRESET`, `EPIPE` and the `EIO`-is-"Transfer failed" class, backed off (the references use 2 s / 5 s / 15 s); an `EPERM` whose size still matched after a failed write actually succeeded (a FAT quirk); a vanished mount directory is an unplug, and the writer must notice |
| Never written            | `.rockbox/` itself. Installing and configuring the firmware is Rockbox Utility's job; this application detects it and never flashes anything                |

### The two references, and the licence boundary

| Reference                                                              | What it contributes                                                                 | Licence | What may cross into this MIT project |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ------- | ------------------------------------- |
| RockSync (© 0Stormy) — a small Python + ffmpeg sync tool, held as `RockSync-main.tar.gz` | the rules: the quality ceiling, the FAT sanitizer, the cover-art policy, the skip list | MIT | portable with attribution — once the upstream repository's licence is verified; its location is unconfirmed and two unrelated GitHub projects share the name |
| [iPodRocks](https://github.com/JoaoSobral/ipodrocks-js) — a 56,000-line Electron sync manager | the war stories: size+mtime compare rules with FAT tolerances, device identity, eject commands, and the full tagcache / runtime-data / tagnavi format detail | GPL-3.0 | **no code** — facts, constants and algorithms reimplemented from scratch. GPL code cannot flow into an MIT project |

Rockbox itself is GPL-2.0 and the authority on its own formats. This is no
longer a claim that rests on the two references: every checkable fact above has
been read back from the firmware source (git.rockbox.org, `v4.0-final-1918`,
commit `685eb7a1af`, held at `~/Projects/rockbox`) — the magic and filenames
(`apps/tagcache.c:115,139-154`), the tag numbering that puts playcount at 15 and
rating at 16, the 96-byte index stride that follows from `TAG_COUNT = 23`,
`<Untagged>`, `TAG_MAXLEN`, the UTF-8 `.m3u8` rule (`apps/playlist.c:319,461`),
`tagnavi_user.config` (`apps/tagtree.c:65-66`) and the 15-second rule
(`apps/tagtree.c:1017`, which also credits 15 s of crossfade to playtime,
line 1057). What remains unverified is hardware behaviour, which no source
substitutes for. (rockbox.org sits behind an Anubis bot-check; the source does
not.)

### Deliberately not in a first version

| Possibility            | What it is | Why it waits |
| ---------------------- | ---------- | ------------ |
| Writing the tagcache   | `.rockbox/database_idx.tcd` plus `database_<N>.tcd` (magic `0x54434810`, version `0x10`, Rockbox 4.0+). The device's "Database → Initialize now" takes minutes after every sync; building the database on the host takes none. But the device cannot repair a database it did not write — one wrong field is "no database at all". | a file-copy sync is correct without it; opt-in at most. The format is source-checked now, and the source offers one more courtesy neither reference mentions: `database_commit.ignore`, whose presence suppresses the temp database's commit |
| Runtime data reading   | with *Gather Runtime Data* enabled by the owner, Rockbox records play counts, play time and ratings inside that database (numeric tags 15–18); a play counts only after 15 seconds | read-only and genuinely useful; presumes a database exists to read |
| Ratings, two-way       | 0–10 ratings, `0` meaning unrated, merged between library and device | the library has no rating field, which is itself an unbuilt v0.10 addition |
| Tagnavi smart playlists | `.rockbox/tagnavi_user.config` (`#! rockbox/tagbrowser/2.0`) — live on-device queries instead of frozen `.m3u8` snapshots; a reference warns `%include` silently fails on some builds | presumes smart playlists, also v0.10 |
| `.lrc` lyrics          | Rockbox shows synchronized lyrics beside a track | belongs to the same undecided family as v0.10's lyrics item |
| Transcoding            | the references shell out to ffmpeg — precisely the C-process dependency the all-Rust rule rules out; a pure-Rust FLAC→FLAC 16/44.1 path is plausible (symphonia + clac), a pure-Rust lossy encoder does not exist | hi-res files are named and skipped in a first version, and whether that is good enough is decision 4 below |

### Deliverables, once the stage is committed

Provisional, which is the difference between Unknown and Planned:

- `.m3u8` export and import for playlists — device-independent value, landing first.
- Device discovery with USB identity, and a manual folder pick as fallback so the
  feature is never hostage to udev.
- A capability probe: model, version where `rockbox-info.txt` allows, free
  space, and the formats the device is not confirmed to play — hi-res among
  them — named and skipped rather than silently omitted.
- A pure sync planner over size and mtime with FAT tolerances (the references use
  128 bytes and 2500 ms) and never a hash; a manifest inside the managed root
  records what was placed there, so a deletion can be honest about what it claims.
- One-way sync of chosen playlists, contained to managed roots (`Music/`,
  `Playlists/`); deletion touches only what the manifest claims.
- A dry run and a free-space check before any write — neither reference has
  either, and the mid-copy `ENOSPC` they accept is the failure this stage beats.
- A device view: per-playlist checkboxes, the report, the progress; and an Eject
  that works or says why it cannot.

### Acceptance criteria, once the stage is committed

Two are fixed ahead of the rest, mirroring v0.14's two:

- Nothing is written until a dry run has been read and the free-space check has
  passed.
- Everything written, and everything deleted, stays inside managed roots this
  application created.

### What has to be decided before this stage can start

1. **Commitment.** Is this a stage at all? Unknown is an honest answer while the
   decision is deferred; it is not an honest answer forever.
2. **Its home.** This roadmap's v0.10 lists "Optional device synchronization"
   among the future features, while v0.14 and v0.15 hold the device-sync plans.
   One of the two places should own it.
3. **The v0.14 relationship.** A flashed iPod belongs to this stage and a
   stock-firmware iPod to v0.14; one device view serving both is possible, and
   so is two.
4. **The format policy.** Is named-and-skipped good enough for v1, or does the
   stage wait for the pure-Rust FLAC path?
5. **The hardware.** Acceptance is against the running application, and the
   running application here means a running player. Which device — and how far a
   directory may honestly stand in for one in tests — decides what "verified" is
   allowed to claim.

---

## gPodder API reference

What v0.13 is written against. Two protocols live here and they are not the
same thing:

- **The sync protocol** (`/api/2/…`, and the `/subscriptions/…` bulk endpoints)
  is what a podcast application talks to in order to keep subscriptions and
  episode state in step. This is what v0.13 implements.
- **The administrative API v1** (`/api/v1/…`) manages accounts, sync logins and
  API keys on a goPodder instance. A player has no business calling it, and
  iCraftMusic will not.

gPodder's own reference is the
[gpodder.net API documentation](https://gpoddernet.readthedocs.io/en/latest/api/),
and the goPodder server implementation documents its administrative surface in
[`APIDOCS.md`](https://github.com/cbrgm/gopodder/blob/main/APIDOCS.md).

### General

| Item                | Value                                                                 |
| ------------------- | --------------------------------------------------------------------- |
| Protocol            | HTTPS; an `http` request is redirected to `https`                     |
| Auth                | HTTP authentication (username and password) on every sync endpoint    |
| Data format         | JSON, except the bulk subscription endpoints, which also take OPML and plain text (one URL per line) |
| Date format         | ISO 8601 / RFC 3339 — `YYYY-MM-DDTHH:MM:SSZ`                           |
| Podcast identity    | The feed URL                                                           |
| Episode identity    | The episode's media URL                                                |
| Device ID           | Any string matching `[\w.-]+`, generated by the client and unique per account. A good shape is the application name plus the host name. Two clients sharing an ID can overwrite each other's subscriptions. |

### Endpoints

| Area             | Method and path                                                   | Purpose                                                              |
| ---------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------- |
| Subscriptions    | `GET /subscriptions/{username}/{deviceid}.{format}`               | One device's subscription list in the format asked for                |
| Subscriptions    | `GET /subscriptions/{username}.{format}`                          | Every subscription on the account — the right call on first run       |
| Subscriptions    | `PUT /subscriptions/{username}/{deviceid}.{format}`               | Replace a device's list; the body is OPML, JSON or plain text. The device is created if it does not exist |
| Subscriptions    | `POST /api/2/subscriptions/{username}/{deviceid}.json`            | Upload a **delta** of additions and removals. The server returns the timestamp and any URLs it rewrote |
| Subscriptions    | `GET /api/2/subscriptions/{username}/{deviceid}.json?since=`      | Download the changes since a timestamp, as `add`, `remove` and a new `timestamp` |
| Episode actions  | `POST /api/2/episodes/{username}.json`                            | Upload episode actions — `download`, `play`, `delete`, `new` — with playback positions |
| Episode actions  | `GET /api/2/episodes/{username}.json`                             | Retrieve actions, filtered by `since`, `device` or `podcast`; `aggregated` collapses to the latest per episode |
| Devices          | `POST /api/2/devices/{username}/{deviceid}.json`                  | Set the device's `caption` and `type` (`desktop`, `laptop`, `mobile`, `server`, `other`) |
| Devices          | `GET /api/2/devices/{username}.json`                              | List the account's devices                                            |
| Devices          | `GET /api/2/updates/{username}/{deviceid}.json?since=`            | The "what changed" call: subscriptions to add with their titles and descriptions, subscriptions to remove, and updated episodes with their status |

The `update_urls` list in the upload responses is not decoration: the server
rewrites URLs it considers badly formed, and the client is expected to apply the
rewrite locally so the next upload does not undo the server's correction.

### Episode action types

| Action     | Meaning                                                              |
| ---------- | -------------------------------------------------------------------- |
| `download` | The episode has been downloaded on this device                      |
| `play`     | Playback happened; carries `started`, `position` and `total` in seconds |
| `delete`   | The downloaded file has been deleted                                |
| `new`      | Resets previous actions for the episode; nothing is deleted server-side |
| `flattr`   | Legacy, and not implemented here                                     |

A `play` action is the one that carries a resume position, so it is the one that
matters most for the acceptance criteria above. It requires `started`,
`position` and `total` to be set together; the server rejects a `play` action
missing them.

### Error handling

| Status | Meaning                                        | What the client does                                  |
| ------ | ---------------------------------------------- | ----------------------------------------------------- |
| 400    | Bad request — invalid input or format          | Report the message; do not retry                      |
| 401    | Authentication failed                          | Say so once, stop the sync, keep the local state      |
| 403    | The key or account is not permitted this      | Report it; do not retry                               |
| 404    | Device or resource not found                   | Register the device, then retry once                  |
| 409    | Conflict — the resource exists                 | Treat as already-done and continue                    |
| 500    | Server error                                   | Retry with backoff, keeping the local changes pending |

There is no rate limiting on the sync API, but the documentation asks that
clients be considerate. iCraftMusic syncs when the user asks it to, on a feed
refresh, and when it is about to close if a sync is outstanding — not on a timer.

### What is deliberately out of scope

- The **administrative API v1** (`/api/v1/users`, `/api/v1/accounts`,
  `/api/v1/users/{username}/subscriptions`). It exists so an operator can script
  what the goPodder web UI does — creating sync logins, exporting OPML per user,
  provisioning accounts. A player consuming it would be reaching behind the
  server's own account model, and the keys it uses are the operator's, not the
  listener's.
- **Podcast Lists** and the **Suggestions / Toplist** endpoints. They are a
  directory service, not a sync one, and a first version of v0.13 does not
  need to browse a catalogue to subscribe to a feed.
- **`flattr`** actions.

---

## Recommended delivery sequence

| Milestone              | Stages | Deliverable                                              |
| ---------------------- | ------ | -------------------------------------------------------- |
| M8 — Generalisation    | 11     | One library and one player holding more than music        |
| M9 — Film and TV       | 12     | Films and series indexed, browsed and played locally      |
| M10 — Podcasts         | 13     | Subscriptions, downloads and playback, with gPodder sync  |
| M11 — iPod (unknown)   | 14     | Apple iPod synchronization, once the approach is decided  |
| M12 — Rockbox (unknown)| 15     | Rockbox device synchronization, once the stage is committed |

v0.11 must land before either of the others; 12 and 13 do not depend on each
other and can be taken in either order. v0.14 and 15 depend on nothing and
block nothing — they are off to one side, and neither milestone has a shape
until its open questions are answered.

## Development rules

The rules in [the music track](#vision) apply here too. Three of them matter
enough to restate for this track:

- Generalise before adding. A second store for a second kind of media is a
  failure of v0.11, not a shortcut through it.
- Do not implement a future stage prematurely. A podcast row in the library is
  v0.13's work, even though v0.11 makes the row possible.
- Mark work complete only when its acceptance criteria have been verified
  against the running application, not when the code compiles.

## Known constraints of this plan

Writing these down now, so that a later stage does not quietly pretend they are
not there:

- **Decoders cost build complexity.** The music player's decoder was cheap
  because `rodio` bundles it. A video demuxer and a picture surface are not in
  that crate, and the choice between a pure-Rust demuxer and a system library is
  a real decision with a licensing dimension. v0.12 must make it explicitly,
  in this roadmap, before it writes the code.
- **A feed is not a file.** Everything else in the library is a file on disk
  that the scanner can look at. Subscriptions are network state with an update
  schedule, and the scan model has to be extended rather than reused unchanged.
- **A position is per item, not per file.** The music library's play counts
  survive a rescan because they live in the database keyed by path. A resume
  position keyed only by path is lost when a podcast episode is re-downloaded
  or a film is moved; keying by the item's own identity is what v0.13's
  acceptance criteria are actually testing.
- **Sync can conflict.** Two clients listening to the same episode is the normal
  case, and "last action wins" is only correct because the timestamps make it
  so. Any local merge rule that is not last-write-wins has to be argued for on
  its own merits.
- **A device is not a file, and not a server.** The whole application so far
  writes to files it owns and reads from a database it owns. v0.14 writes to
  what is effectively someone else's filesystem — one the device verifies and
  will reject silently if it is wrong — and v0.15 writes to one the device
  merely reads, which is the easier job but still not a filesystem this
  application owns. That is a different class of operation from anything else
  here, and it is why the dry run in either stage is a deliverable rather than a
  nicety.
- **An Unknown stage does not get a milestone date.** The temptation, once a
  stage has a number, is to schedule it. v0.14 and 15 have numbers so they
  can be referred to; they do not have committed plans, and the two are not the
  same thing.
