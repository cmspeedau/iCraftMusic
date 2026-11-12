# Agent Instructions

These instructions apply to all work in this repository. Follow any more specific instructions found in a nested `AGENTS.md` for the files being changed.

## Project context

- iCraftMusic is a local-first desktop media library for music, movies, TV shows, and podcasts, with planned device syncing.
- The application must be implemented **only in Rust, using egui for the UI and eframe as the application framework**. Treat this as a firm requirement, not an initial preference or a replaceable technology direction. Target macOS, Windows, and Linux.
- Do not introduce another programming language, UI toolkit, or application framework for application functionality. Build scripts and platform packaging/integration tooling may use platform-appropriate tools only when necessary, and must not replace or duplicate the Rust/egui/eframe application.
- Consult `README.md`, `ROADMAP.md`, and `PRODUCT-TECHNICAL-PLAN.md` before changing product behavior or architecture. Treat roadmap capabilities as plans unless implementation and tests verify them.

## Before changing code

- Inspect the repository structure, relevant source, tests, configuration, and documentation; do not assume files or tooling exist.
- Trace the affected code paths and follow established project conventions.
- Keep changes focused, preserve unrelated user work, and avoid broad refactors or new dependencies without a clear need.
- Ask for clarification only when ambiguity materially affects the implementation or user-visible behavior.

## Implementation standards

- Prefer simple, maintainable Rust with explicit types and clear module boundaries.
- Implement the desktop UI with egui within the eframe application. Keep UI concerns separate from persistence, scanning, playback, and device protocol logic while keeping the application in Rust.
- Handle errors explicitly and provide useful, safe feedback. Do not silently swallow errors or add placeholder functionality.
- Validate untrusted input at boundaries and avoid exposing private paths, media details, or credentials unnecessarily.
- Preserve compatibility and existing behavior unless the task calls for a deliberate change.

## Media and data safety

- Imports must not move, rename, transcode, or modify source media by default.
- Treat metadata edits as library-only unless an explicit, reviewed write-back feature is requested.
- Never perform destructive source-library or device operations implicitly. Use clear intent, preview where practical, and recovery guidance.
- Do not overwrite existing media during restore or sync without explicit user confirmation.
- Do not claim support for formats, codecs, hardware acceleration, platforms, or devices without verified evidence.

## Verification

- Inspect existing tests before changing behavior and add tests for new behavior and important edge cases.
- Run relevant tests, formatting, lint, type checks, or builds using the repository's existing documented tools when practical.
- Report commands and outcomes accurately. If checks cannot be run, state why and what remains unverified.
- Review the final diff for unrelated changes and regressions.

## Documentation

- Update documentation when changing user-visible behavior, setup, configuration, architecture, or support claims.
- Keep `ROADMAP.md` and `PRODUCT-TECHNICAL-PLAN.md` aligned with actual decisions, and distinguish goals from implemented capabilities.
- Preserve the requirement that the application is written only in Rust with egui and eframe in any updated project guidance.

## Git and external actions

- Do not create, publish, deploy, install, migrate, or delete resources unless explicitly requested.
- Do not commit or push unless specifically requested.
- Never add secrets, credentials, or personal media data to the repository.
