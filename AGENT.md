Agent Instructions — Rust Music Player
1. Project Objective

Build a polished desktop music player inspired by the design and usability of classic iTunes.

The application should provide a local music library, music playback, playlists, search, metadata management, album artwork, and a refined desktop interface.

All application code MUST be written in Rust.
2. Mandatory Language Rules

    Rust is the only permitted programming language for application logic.
    All application features, business logic, state management, data processing, and integrations MUST be implemented in Rust.
    Do not write application logic in JavaScript, TypeScript, Python, Go, C#, C++, or any other programming language.
    Do not introduce Node.js, Python scripts, or other language-based runtimes as part of the development workflow.
    Do not use external scripts written in other programming languages.
    Configuration files, documentation, database schemas, and build manifests are permitted in their appropriate formats.
    Use declarative UI markup or styling only when required by the chosen Rust-compatible UI framework. Do not use markup to implement application logic.
    Any native dependencies required by Rust libraries must be integrated through supported Rust crates and documented system dependencies.

3. Technology Stack

Prefer the following Rust ecosystem, subject to compatibility and project requirements:

    Language: Rust, using stable Rust.
    Desktop UI: Slint or egui. Choose one primary framework and keep it consistent.
    Audio playback: rodio or another suitable Rust audio playback library.
    Metadata: lofty for reading and editing supported audio tags.
    Database: SQLite through rusqlite or sqlx.
    Serialization: serde.
    File system: std::fs and Rust filesystem libraries.
    Logging: tracing and tracing-subscriber.
    Error handling: thiserror for library errors and anyhow where appropriate in application boundaries.
    Testing: Rust's built-in test framework, supplemented by suitable Rust testing crates.

Before introducing a dependency, verify its maintenance status, license, platform compatibility, and suitability.

Do not add a web frontend or a JavaScript-based desktop shell.
4. Architecture

Organize the application into clearly separated Rust modules.

Suggested structure:

src/ main.rs app.rs config.rs error.rs audio/ mod.rs player.rs queue.rs output.rs library/ mod.rs scanner.rs track.rs metadata.rs artwork.rs playlists/ mod.rs manager.rs database/ mod.rs schema.rs repository.rs search/ mod.rs index.rs ui/ mod.rs navigation.rs libraryview.rs nowplaying.rs playlistview.rs settingsview.rs

The exact structure may evolve, but responsibilities must remain clearly separated.
Architectural rules

    Keep UI rendering separate from audio playback.
    Keep database operations separate from UI components.
    Represent tracks, albums, artists, playlists, and playback state with explicit Rust types.
    Centralize playback state and queue management.
    Avoid global mutable state.
    Prefer ownership, borrowing, and explicit state transitions.
    Use message passing or appropriate synchronization when coordinating background tasks.
    Keep long-running library scans and metadata operations off the UI thread.
    Avoid unnecessary abstraction and premature optimization.
    Make modules testable without requiring the full desktop application to run.

5. Product Requirements
Music library

    Import music from user-selected folders.
    Scan supported audio formats recursively.
    Extract available metadata, including title, artist, album, genre, duration, and track number.
    Store indexed library information in SQLite.
    Detect previously imported files and avoid unnecessary duplicates.
    Handle missing files, malformed tags, and unsupported formats safely.
    Refresh library information when files change.

Audio playback

    Play, pause, resume, and stop playback.
    Support previous and next track.
    Provide seek and volume controls.
    Display elapsed and total playback time.
    Support queue management.
    Support shuffle and repeat modes.
    Handle end-of-track transitions and playback errors.
    Keep playback state consistent with the visible interface.

Library interface

    Provide a sidebar for library navigation and playlists.
    Provide a central track, album, or artist view.
    Provide a persistent playback bar.
    Include album artwork where available.
    Support sortable track columns.
    Include search and filtering.
    Support dark and light themes where practical.

Playlists

    Create, rename, and delete playlists.
    Add and remove tracks.
    Reorder playlist entries.
    Persist playlists between sessions.
    Support saving and restoring the playback queue where appropriate.

Persistence

    Store library indexes, application settings, playlists, and relevant playback preferences.
    Use database transactions for multi-step updates.
    Provide database migrations when the schema changes.
    Never delete user music files when removing library entries.
    Never overwrite user metadata without an explicit action.

6. Design Guidelines

The application should evoke classic iTunes without copying proprietary branding or assets.

    Use a restrained desktop layout with a navigation sidebar, content area, and playback controls.
    Establish clear visual hierarchy through spacing, typography, and alignment.
    Make album artwork visually prominent without overwhelming track information.
    Keep playback controls accessible at all times.
    Support keyboard navigation and sensible keyboard shortcuts.
    Provide empty, loading, error, and success states.
    Ensure controls have clear interactive states.
    Avoid unnecessary animations, excessive gradients, and decorative UI elements that reduce usability.
    Maintain consistent spacing, component sizing, and iconography.
    Design for resizing and common desktop display sizes.

7. Code Quality

    Use cargo fmt to format Rust code.
    Use cargo clippy to identify common errors and improve code quality.
    Run cargo check after meaningful changes.
    Run relevant tests before considering a stage complete.
    Use explicit types and meaningful names.
    Avoid unexplained constants and duplicated logic.
    Handle errors explicitly; do not silently discard failures.
    Do not use unwrap() or expect() in production paths unless the invariant is documented and justified.
    Keep functions focused and modules cohesive.
    Document public APIs and non-obvious design decisions.
    Do not leave placeholder implementations while claiming a feature is complete.
    Never claim a command or test succeeded unless it was actually executed successfully.

8. Dependency Management

    Use Cargo for Rust dependencies and builds.
    Keep Cargo.toml and Cargo.lock consistent with the project.
    Prefer a small, well-maintained dependency set.
    Avoid duplicate libraries serving the same purpose without a documented reason.
    Record necessary native libraries and platform-specific requirements.
    Verify licenses and redistribution requirements before shipping dependencies.
    Do not add dependencies solely to avoid writing a small, straightforward Rust implementation.

9. Security and Data Integrity

    Validate imported paths and external metadata.
    Handle malformed audio files without crashing the application.
    Avoid executing imported files or treating metadata as trusted instructions.
    Use parameterized SQL queries.
    Protect database integrity through transactions and appropriate constraints.
    Avoid destructive file operations.
    Keep application settings and library data in platform-appropriate locations.
    Do not transmit library contents or listening history over the network unless a future feature explicitly requires it and the user consents.

10. Development Workflow

For every implementation task:

    Inspect the existing project structure and relevant Rust modules.
    Identify the smallest coherent change that satisfies the task.
    Follow the existing architecture unless there is a documented reason to change it.
    Implement the feature in Rust.
    Add or update tests for relevant behavior.
    Run formatting, compilation checks, linting, and applicable tests.
    Resolve regressions before proceeding.
    Update documentation when architecture, commands, or behavior changes.

Do not implement multiple unrelated stages in one change unless explicitly requested.
11. Stage Completion Requirements

A stage is complete only when:

    Its specified functionality is implemented.
    Its code follows the Rust-only application-code rule.
    The project compiles for the intended target.
    Relevant automated tests pass.
    Errors and edge cases are handled appropriately.
    Existing functionality remains intact.
    The corresponding roadmap items are updated.
    Any known limitations are documented honestly.

12. Final Rule

Rust is the sole application programming language.

All implementation decisions must prioritize reliability, maintainability, user data integrity, and a cohesive desktop music-player experience.

When requirements are ambiguous, choose the simplest reliable Rust-native solution that preserves the established architecture and the staged roadmap.
