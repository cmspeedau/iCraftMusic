Roadmap — Rust Desktop Music Player
Vision

Build a native desktop music player inspired by classic iTunes: a music library, album artwork, playlists, playback controls, search, and a polished desktop interface.

The application will be developed incrementally, with each stage delivering a working and testable result.

Core constraints

    Rust-only application code.
    Native desktop application; no JavaScript application runtime.
    Local-first music library.
    Persistent library and playlist data.
    Modular architecture that supports future enhancements.
    Every stage must compile and preserve previously completed functionality.

Stage 0 — Project Foundation

Goal: Establish a reliable Rust development environment and application skeleton.

Status: Complete. Cargo workspace, module skeleton, typed error handling, tracing-based logging, on-disk settings, and formatting/linting/testing conventions are in place; see README.md for build and run instructions.
Deliverables

    Initialize the Cargo project.
    Select and configure one Rust-compatible desktop UI framework.
    Establish the project directory structure.
    Configure error handling and logging.
    Create the initial application window.
    Add basic application settings infrastructure.
    Establish formatting, linting, and testing conventions.
    Document build and run instructions.

Acceptance criteria

    The project builds and launches.
    The application displays a basic desktop window.
    cargo fmt --check, cargo check, and applicable tests pass.
    No application logic is written in another programming language.

Stage 1 — Desktop UI Shell

Goal: Deliver the initial iTunes-inspired visual experience.

Status: Complete. The egui/eframe shell provides a resizable window, navigation sidebar, content panel, persistent playback bar, placeholder library/playlist/settings views, empty states, and light/dark themes. Playback controls render as explicitly disabled states until Stage 2 wires up audio.
Deliverables

    Main application window with resizing support.
    Navigation sidebar.
    Central content panel.
    Persistent bottom playback bar.
    Placeholder library and playlist views.
    Consistent typography, spacing, icons, and colors.
    Initial light and dark theme support, if practical.
    Empty states for a new user's library.
    Responsive layout behavior for common window sizes.

Acceptance criteria

    The layout is cohesive and visually consistent.
    Navigation changes the active content view.
    Playback controls have defined states, even before real audio playback is implemented.
    The UI remains usable when the window is resized.
    No placeholder control is presented as a fully working feature.

Stage 2 — Audio Playback Engine

Goal: Implement real music playback independently of the library interface.

Status: Complete. The `audio` module wraps `rodio` behind a small facade (output device, queue, transport, errors) and the UI shell drives it: the playback bar plays, pauses, seeks, changes volume, steps through the queue, shuffles and repeats, and reports the device, decoder and missing-file failures in place of the track detail. Stage 2 deliberately stops at "load a file or folder, play it, keep a queue" — tag reading, folder scanning into a persisted library and playlists are Stage 3.

Deliverables

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

Acceptance criteria

    Supported audio files play correctly.
    Playback controls reflect the actual playback state.
    Track transitions do not leave the UI in a stale state.
    Unsupported files and missing files produce useful errors.
    Audio processing does not block the UI thread.

Stage 3 — Music Library and Metadata

Goal: Allow users to build and maintain a persistent local music collection.
Deliverables

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

Acceptance criteria

    Users can import a music folder.
    Imported tracks appear in the library.
    Repeated scans do not create unnecessary duplicate entries.
    Missing metadata does not crash the application.
    Scanning large folders does not freeze the interface.
    Removing a track from the indexed library does not delete its source file.

Stage 4 — Functional Music Library UI

Goal: Connect imported music to the desktop interface.
Deliverables

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

Acceptance criteria

    Library views reflect the database contents.
    Selecting and playing tracks updates the playback bar.
    Search returns relevant results without blocking the UI.
    Artwork and missing metadata have sensible fallbacks.
    Sorting and selection behave consistently.

Status: Complete. The Library section browses the indexed music as an All Tracks table, an album grid with cover art, and an artist list. The table sorts by any of its columns, filters as the search box is typed, supports click, ctrl-click and shift-click selection, double-click to play, and a context menu for playing, queueing and removing rows; removal only ever forgets the database row, and a later scan puts it back. Cover art is decoded once per album into an egui texture and re-decoded when the cache is full, with the album's initial standing in when a file has no artwork or it cannot be read. Empty, filtered-to-nothing, scanning and library-unavailable states each say what happened and what to do next. Search and sorting run over the in-memory rows the database last returned, so typing never touches the database and cannot block a frame; the metadata editing, playlist and settings work they set up is Stage 5 and Stage 6.

Stage 5 — Playlists and Queue Management

Goal: Support personal music organization and flexible listening.

Status: Complete. The Playlists section lists every playlist by name with its track count and running time, and shows the open one's tracks numbered in play order. A playlist is made, renamed, copied and deleted from the section's toolbar; deleting one asks first, and the question says what stays — the tracks, and the files behind them. Tracks reach a playlist from the Library's right-click menu, which puts the row under the pointer into an existing playlist or names a new one and puts it there; adding is one transaction, so an album cannot land half-added. Entries can be moved earlier and later from the buttons on each row, which swap the row with its neighbour, and a track can be taken out of a playlist from its own menu. Play and Add to queue run the whole playlist through the existing queue, which is also where the queue's own operations were finished in Stage 2: queued tracks move and come out through the Queue page, and shuffle and repeat are the playback bar's controls. Shuffle and repeat are remembered between runs alongside the theme and volume, and are reapplied at startup. Playlists are SQLite rows and survive a restart with their order intact; nothing in the playlist paths touches an audio file, and no queue operation writes playlist rows.

Deliverables

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

Acceptance criteria

    Playlist changes persist between sessions.
    Queue operations do not corrupt playlist contents.
    Deleting a playlist does not delete the underlying music files.
    Shuffle and repeat modes behave predictably.
    Track transitions update the queue and interface correctly.

Stage 6 — Metadata Editing and Library Polish

Goal: Improve library maintenance and everyday usability.

Status: Complete. Selecting one track opens a metadata editor over the Library — title, artist, album, album artist, genre, track and disc numbers, and the cover — and nothing in it is written anywhere until one of its two buttons is pressed. **Save** stores the values in the database and marks the row as edited (`✱` in front of the title, with the reason on hover and in the editor); **Write to file** additionally rewrites the tags inside the audio file, and is the only path in the application that changes a user's file. A format whose tags cannot be written is refused with a message rather than written badly, and a row whose file has gone is refused too. A replacement cover is decoded before it is accepted and copied into the artwork cache under the track's own name, so a file that is not an image never becomes a cover. A scan never overwrites an edit while the file itself is unchanged; once the file *has* changed, the file wins again and the edit is dropped. Sorting ignores a leading article (`The Beatles` files under B), the table can be narrowed to the tracks whose files have vanished, and those rows can be forgotten in one step. Playback and navigation answer to the keyboard — space, `Ctrl/⌘ ←/→/↑/↓`, `Ctrl/⌘ L/P/,/1/2/3/E`, `Delete` and `Esc` — listed in Settings, and no shortcut fires while a text field has focus. Removing tracks and forgetting missing ones both ask first. Files in a music format the bundled decoder cannot read are counted and named after a scan instead of vanishing silently.

Deliverables

    Edit supported track metadata.
    Update the database when metadata changes.
    Write tags to files only after explicit user action.
    Display embedded album artwork.
    Support manually selecting replacement artwork where practical.
    Improve sorting and filtering behavior.
    Add keyboard shortcuts for playback and navigation.
    Add confirmation for destructive library operations.
    Improve missing-file and unsupported-format handling.

Acceptance criteria

    Metadata changes are validated and persisted correctly.
    File tag writes do not happen unexpectedly.
    Unsupported tag operations fail safely.
    Keyboard shortcuts do not interfere with normal text input.
    Existing library entries remain consistent after edits.

Stage 7 — Settings, Reliability, and Performance

Goal: Prepare the application for regular daily use.

Status: Complete. Everything the application remembers lives in one TOML file — theme, volume, watched folders and the output device — and is written as it changes; Settings → Playback now chooses the output device from the ones the backend offers, remembers it by identifier, and falls back to the system default with a message when the chosen device is gone, while watched folders can be dropped as well as added. The database is versioned, and a database written by an older build is copied beside itself (`VACUUM INTO`, so the copy is consistent even with a write-ahead log in flight) before a migration rewrites it; Settings → Diagnostics runs a `PRAGMA integrity_check` on demand, says what to do about a damaged file, and the recovery steps are written down in README.md. Diagnostics also reports the build, the schema version, the library counts, the paths (settings, data, database, log) and the output stream in use. Logging goes to stderr *and* to a bounded file under the data directory, rotated rather than grown forever, so a failure from a session that has already been closed can still be read. On the performance side the Library's visible rows are filtered and sorted once per change rather than once per question — the view asks for them several times a frame — the selection is re-checked against a set rather than a list, and the artist list is virtualised like the track table; a test over a 20,000-track library guards the whole pass. Regression tests cover a queue walked to its end, a folder rescanned without duplicating its tracks, a playlist reopened from its file, and the device, migration and integrity paths above. Packaging is prepared but not finished, which is Stage 9's job: a release profile, a desktop entry and an icon live under `packaging/`, and README.md documents the release build and install. Still missing, and deliberately left to a later stage: a scan can be stopped but not resumed after the application exits, and there is no installer or release artifact yet.

Deliverables

    Persistent application preferences.
    Audio output configuration where supported.
    Configurable library locations.
    Database migrations and recovery considerations.
    Better cancellation and recovery for background scans.
    Improved error reporting and diagnostics.
    Performance testing with large music libraries.
    Regression tests for playback, scanning, and playlists.
    Platform-specific packaging preparation.

Acceptance criteria

    Preferences survive application restarts.
    Large libraries remain responsive during common operations.
    Background tasks can be cancelled or safely completed where applicable.
    Database updates remain consistent.
    Common failures produce actionable messages rather than crashes.

Stage 8 — Visual Refinement and Accessibility

Goal: Deliver a cohesive, production-quality iTunes-inspired experience.
Deliverables

    Refined spacing, typography, icons, and colors.
    Consistent visual treatment for selected and hovered items.
    Improved album and artist presentation.
    Better playback-bar layout.
    Loading indicators and subtle transition effects where appropriate.
    Keyboard navigation and focus indicators.
    Accessible control labels.
    Improved contrast and scalable layout behavior.
    Final light and dark theme refinements.

Acceptance criteria

    All main views follow a consistent design system.
    Controls remain discoverable and understandable.
    Keyboard navigation works across major workflows.
    The application remains legible at common window sizes.
    Visual refinement does not compromise playback reliability.

Stage 9 — Release and Distribution

Goal: Produce a dependable installable application.
Deliverables

    Release build configuration.
    Platform-specific build and packaging instructions.
    Application icon and version information.
    Clean-install and upgrade testing.
    Final dependency and license review.
    User documentation.
    Known-limitations documentation.
    Release checklist and versioning strategy.

Acceptance criteria

    A release build can be generated reproducibly.
    The application launches on each declared target platform.
    Core playback and library workflows pass release testing.
    Required runtime dependencies are documented.
    No critical known issue is hidden or misrepresented.

Stage 10 — Optional Future Enhancements

Goal: Extend the application after the core product is stable.

Possible additions:

    Smart playlists and saved filters.
    Lyrics display and synchronized lyrics.
    Equalizer and audio effects.
    Gapless playback and crossfade.
    Ratings, play counts, and listening history.
    Playlist import and export.
    External media-key integration.
    Additional audio formats and decoder backends.
    Portable mode.
    Optional device synchronization.
    Optional network streaming or remote library integrations.

These features are not prerequisites for the initial release. Each must be assessed for technical complexity, dependencies, privacy, licensing, and platform support before implementation.
Recommended Delivery Sequence
Milestone	Stages	Deliverable
M1 — Foundation	0–1	Launchable desktop UI
M2 — Playback	2	Working audio player
M3 — Library	3–4	Indexed and searchable music collection
M4 — Organization	5–6	Persistent playlists and metadata editing
M5 — Stability	7	Reliable daily-use application
M6 — Release	8–9	Polished, packaged desktop application
M7 — Expansion	10	Optional advanced capabilities
Development Rules

    Complete stages in order unless a dependency requires otherwise.
    Keep every stage in a compilable state.
    Implement real functionality before adding decorative complexity.
    Keep application code exclusively in Rust.
    Write tests alongside features rather than postponing all testing.
    Avoid implementing future-stage features prematurely.
    Update this roadmap when requirements change.
    Mark work complete only when its acceptance criteria have been verified.

Definition of Done

A feature is complete when its implementation, error handling, tests, documentation, and integration with existing functionality are complete.

The project is ready for its initial release when the core library, playback, search, playlists, persistence, and primary desktop workflows are stable and packaged for the declared target platforms.
Recommended project layout

Keep both files at the repository root so your coding agent can use the instructions and roadmap throughout development.
text

rust-music-player/
├── AGENT.md
├── ROADMAP.md
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .gitignore
├── migrations/
├── assets/
│   └── icons/
├── tests/
└── src/
    ├── main.rs
    ├── app.rs
    ├── audio/
    ├── library/
    ├── playlists/
    ├── database/
    ├── search/
    └── ui/

One important recommendation: Start with Stage 0 and Stage 1 only. Establish the Rust UI framework and get the iTunes-style layout working before introducing audio playback, metadata scanning, or database complexity. That gives you a stable foundation and keeps each delivery stage manageable.
