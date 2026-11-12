# iCraftMusic Product and Technical Foundation Plan

**Status:** Reviewed planning baseline. This document defines intended direction, not implemented or verified capabilities. Update decisions as prototypes and technical investigations provide evidence.

## Product direction

Build a polished, modern desktop manager for a personal collection of music, movies, TV shows, and podcasts, with playlists/collections and future iPod and Rockbox syncing. Use the approachable all-in-one library organization users recognize from iTunes, while keeping the application local-first, transparent, and protective of media files. No cloud, streaming, or remote-hosting service is implied.

### User experience

- Provide distinct, coherent views for Music, Movies, TV Shows, and Podcasts, plus playlists and collections.
- Support global search with an explicit scope, useful empty/no-results/loading/error states, and responsive browsing for large libraries.
- Present device connection status clearly; do not imply device support, identity, or storage details that have not been detected.
- Keep keyboard navigation, accessibility, and clear state feedback in the design from the start.
- Prototype the main navigation, media-specific browsing hierarchies, search behavior, and connected/disconnected device states before implementation. Use sample content only; the prototype is not a claim of functioning media playback or device integration.

## Technical direction

- **Platforms:** target desktop releases for macOS, Windows, and Linux.
- **Language and UI:** Rust, egui, and eframe.
- **Architecture:** keep core library behavior and platform-specific integrations behind clear boundaries. The UI should not own persistence, media scanning, playback, or device protocol logic. Select concrete storage, scanning, playback, metadata, and device libraries through documented technical evaluation before adopting them.
- **Packaging:** Linux is planned as AppImage with zsync metadata. macOS and Windows packaging, signing, and update details remain open decisions. Planned packaging is not a claim of available release automation.

## Media and metadata policy

- Target broad music/audio format support, but determine exact format compatibility separately for import, playback, metadata reading/writing, and each platform before claiming support.
- Target H.264/AVC and H.265/HEVC for movies and TV, with hardware-accelerated decoding when available and a feasible software fallback. Report actual capabilities and limitations; do not promise GPU support independent of backend, OS, and hardware.
- Apply the audio policy to podcast episodes; define feed ingestion and podcast-specific metadata separately.
- Preserve embedded source metadata by default. Establish precedence for embedded tags, user edits, and any external metadata provider; external providers and automatic tag writing are not assumed.
- Publish a tested compatibility matrix as the playback and metadata stack is selected. All formats and acceleration capabilities above are goals, not current support claims.

## Library storage, originals, and backup

- Importing adds media to the library without moving, renaming, transcoding, or modifying original files by default.
- An optional, clearly explained managed-media folder may copy new imports. Never silently relocate existing files; make referenced-in-place versus copied behavior visible and consistent.
- Keep the library database, preferences, playlists, history, and app-managed artwork separate from media. Track media paths and provide missing-file detection and user-directed relinking.
- Treat metadata changes as library-only by default. Any future write-back to media files must be an explicit action with a clear change preview and practical recovery of prior values.
- Document backup, export, restore, and migration for the database and app-managed data. Clearly distinguish these from backing up the media itself; include media only when the user explicitly chooses it or backs it up separately.
- Validate backups and avoid overwriting or deleting existing media during restore without explicit confirmation.

## Delivery sequence

1. **Foundation:** contribution/coding guidelines and issue templates; tested format/metadata policy; UX prototype and interaction/state specification; reviewed architecture and package investigations.
2. **Core library MVP:** application shell, local-folder import, scan/index/refresh with diagnostics, media-specific browsing and search, metadata/artwork handling, playlists/collections, persistence, and backup/export. Protect originals by default and test core library failure cases.
3. **Playback and polish:** select a playback stack based on the compatibility matrix; add media-appropriate playback, queue, history/progress where supported, accessibility, and large-library performance work.
4. **Library care and audio quality:** smart playlists and review-first health tools; then document and implement audio properties and optional audiophile playback capabilities only where verified by the selected backends.
5. **Device syncing:** define a capability-based device abstraction and previewable, recoverable sync flow; validate using simulated devices before targeting specific hardware.
6. **iPod/Rockbox and maturity:** research supported models and workflows, implement and test documented compatibility, and add release automation and platform packaging after platform-specific decisions.

Detailed feature sequencing and acceptance milestones remain in [ROADMAP.md](ROADMAP.md).

## Open decisions and review gates

These decisions require prototype, compatibility, or implementation evidence and are intentionally not guessed here:

- Exact audio/video/container/codec and metadata read/write compatibility by operation and platform; playback backend and hardware-acceleration coverage.
- Metadata source/provider selection and conflict-resolution rules.
- Concrete database, media scanning, artwork/cache, and library migration/backup formats.
- macOS and Windows packaging, signing, distribution, and update strategy.
- Supported iPod models, Rockbox workflows, and device detection/sync capabilities.
- UX prototype review and usability feedback before UI implementation.

Record the rationale and user-visible consequences when resolving these decisions. Avoid representing planned targets as implemented features.

## Scope and safety

This is a local personal-media library plan. Cloud accounts, streaming, remote hosting, and automatic destructive library/device cleanup are out of scope unless separately evaluated and approved. Any operation that could alter source files, restore over existing data, or remove device content requires clear user intent, a preview where practical, and useful recovery guidance.
