# iCraftMusic Roadmap

This roadmap is a starting point for planning. Priorities and sequencing may change as the project’s platform, architecture, and community needs become clearer. “Done” means the milestone has a usable, documented outcome—not just a placeholder.

## Guiding principles

- Keep the experience polished, approachable, and responsive for large libraries.
- Protect users’ media and metadata; make destructive actions explicit and recoverable where practical.
- Prefer open, documented formats and avoid locking users into a service.
- Treat device compatibility as a first-class feature, with safe transfer and clear diagnostics.
- Ship incremental, testable milestones and document supported platforms and formats.

## Phase 0 — Define the foundation

- [x] Decide target desktop platforms and application architecture.
  - Target **macOS, Windows, and Linux** desktop releases.
  - Use **Rust** for the application, with **egui** for the UI and **eframe** as the application framework.
  - Keep platform-specific integrations behind clear boundaries so platform behavior can be handled without changing the cross-platform UI and core library logic.
  - Package Linux releases as **AppImage** with **zsync** metadata; macOS and Windows release packaging details remain to be specified.
- [ ] Establish project contribution guidelines, coding standards, and issue templates.
- [ ] Document initial supported media formats and metadata sources.
  - **Music:** target broad format support across common and less common audio formats. Define and publish the exact import, playback, tagging, and codec support matrix before implementation; format support may differ by operation and platform.
  - **Movies and TV:** target H.264/AVC and H.265/HEVC video, with GPU-accelerated decoding where supported by the operating system, hardware, and playback backend. Provide a compatible software-decoding fallback where feasible and report when hardware acceleration is unavailable.
  - **Podcasts:** use the music/audio format support policy for episode media; document feed and metadata handling separately.
  - Identify metadata/tag sources and rules for conflicting or missing values, including preserving embedded metadata and avoiding source-file edits unless explicitly requested.
  - Treat these as planned targets, not claims of currently implemented support; publish a tested format and hardware-acceleration compatibility matrix as the playback stack is selected.
- [x] Define data-storage and backup principles, including how original files are handled.
  - Provide an iTunes-like library experience: users can add media to a central library and organize it into media-type views, playlists, and collections.
  - By default, importing **adds media to the library without moving, renaming, transcoding, or modifying the original files**. Offer an explicit, understandable option to copy imported media into a user-selected managed media folder; never silently relocate existing files.
  - Keep the library database, preferences, playlists, play history, and app-managed artwork/cache in the application’s library data, separate from media files. Store references to media paths and detect missing or moved files; support a user-directed relink workflow.
  - Make the managed media folder optional and user-configurable. Explain whether files are referenced in place or copied into the managed folder during import, and preserve that choice consistently.
  - Treat metadata edits as library-only by default. If writing tags/artwork into media files is supported later, make it an explicit per-action choice, show what will change, and preserve/recover prior values where practical.
  - Provide a documented library backup/export and restore/migration path covering the database, playlists, settings, and app-managed artwork; clarify that media files are included only when the user chooses a managed-media backup or copies them separately. Avoid implying that backing up the database alone protects the media.
  - Ensure backup and restore do not overwrite or delete existing media without explicit confirmation; validate backups and provide clear recovery guidance.
- [x] Create a UX prototype for library browsing, search, and device status.
  - Provide a low-fidelity, platform-neutral desktop prototype that can be translated into the Rust/egui/eframe application; use sample content only, with no real media or device integration implied.
  - Show persistent navigation for **Music, Movies, TV Shows, Podcasts**, and playlists/collections, with the selected library view, clear hierarchy, and a useful empty-library state.
  - Include a global search field with an explicit scope (current view or entire library), a results state spanning relevant media types, and clear no-results behavior.
  - Define the main browsing layouts: music albums/tracks with artwork fallback; movie posters; TV shows with season/episode hierarchy; podcasts with show/episode hierarchy.
  - Include a persistent device-status area with distinct disconnected and connected states. Connected state should summarize device identity and available/used storage when known; unsupported or unavailable details must be communicated rather than guessed.
  - Specify representative navigation and search interactions, keyboard focus/order, loading and error states, and responsive behavior for narrower windows; keep destructive device actions out of the prototype’s implicit flows.
  - Deliver annotated wireframes or equivalent prototype screens plus a short interaction/state specification, and review them before implementation.
- [x] Review the proposed foundation decisions and publish the resulting product and technical plan.
  - See [PRODUCT-TECHNICAL-PLAN.md](PRODUCT-TECHNICAL-PLAN.md) for the reviewed planning baseline, delivery sequence, and open decisions. It describes intended direction, not implemented or verified capabilities.

**Milestone:** A reviewed product and technical plan that can guide a first implementation, including platform/architecture decisions, initial format and metadata policy, data-safety principles, contribution basics, and a core library/device UX prototype.

## Phase 1 — Core library MVP

- [ ] Build the application shell and responsive library interface.
- [ ] Import local folders without moving or modifying original media by default.
- [ ] Scan, index, and refresh a collection, with useful progress and error reporting.
- [ ] Browse and search music, movies, TV shows, and podcasts.
- [ ] Display and edit core metadata and artwork; define safe conflict handling.
- [ ] Add playlists or collections and basic library organization.
- [ ] Persist the library between sessions and provide a documented backup/export path.
- [ ] Add automated tests for scanning, metadata, persistence, and common failure cases.

**Milestone:** Users can build and manage a dependable local media library.

## Phase 2 — Playback and polish

- [ ] Add playback appropriate to supported media types and platforms.
- [ ] Provide queue, playback controls, and useful library views.
- [ ] Add an **Up Next** queue so users can manage upcoming playback independently of saved playlists.
- [ ] Track favorites/ratings and play history, with clear controls for viewing and clearing history.
- [ ] Add useful shortcuts such as Recently Added and Continue Listening/Watching, where playback progress is supported.
- [ ] Add an optional **Cover Flow** album-browsing view, with artwork navigation, keyboard support, and a useful fallback for albums without artwork.
- [ ] Improve accessibility, keyboard navigation, localization readiness, and empty/error states.
- [ ] Validate performance with large libraries and improve indexing/search as needed.
- [ ] Document supported formats, known limitations, and troubleshooting.

**Milestone:** A polished day-to-day media browsing and playback experience.

## Phase 3 — Playlists and library care

- [ ] Support **smart playlists** built from user-defined rules (for example, genre, rating, date added, or play count), with predictable updates as library items change.
- [ ] Add library health tools to identify missing files, duplicates, and incomplete metadata; provide review and confirmation before applying fixes.
- [ ] Add playlist and metadata import/export in documented, portable formats where practical.

**Milestone:** Users can organize collections flexibly and maintain them without risking unintended changes.

## Phase 4 — Audiophile library and playback

- [ ] Document supported lossless and high-resolution audio formats, sample rates, bit depths, and metadata/tagging limitations.
- [ ] Display audio properties (codec, lossless/lossy status, sample rate, bit depth, channel count, and bitrate) and make them searchable/filterable.
- [ ] Provide optional **bit-perfect playback** where the operating system, audio device, and output backend support it; clearly indicate when system mixing, resampling, or other processing prevents bit-perfect output.
- [ ] Offer configurable output device selection and sample-rate handling, with safe fallbacks and transparent status rather than claiming unsupported exclusive-mode behavior.
- [ ] Add optional ReplayGain/R128 track and album loudness analysis/playback normalization, disabled or configurable by the listener and without altering source files.
- [ ] Support gapless playback and configurable crossfade behavior, ensuring gapless mode does not introduce a crossfade.
- [ ] Add a configurable **10-band equalizer** for music playback, with per-band controls, a flat/reset option, and saved presets; make it clear when EQ processing is enabled since it affects bit-perfect output.
- [ ] Provide a waveform/spectrum visualizer as an optional, non-invasive display feature.

**Milestone:** Listeners can understand their files and configure high-quality playback with honest, device-dependent capability reporting.

## Phase 5 — Device syncing foundation

- [ ] Define a device capability model and a safe sync workflow.
- [ ] Detect and identify supported devices, showing connection and storage status.
- [ ] Preview sync changes before applying them; handle insufficient space and interruptions.
- [ ] Keep device changes separate from the source library and avoid deleting device content unexpectedly.
- [ ] Add sync profiles so users can choose playlists and media types to sync; show an estimate/preview of required space and proposed changes.
- [ ] Add sync logs, recovery guidance, and tests using simulated devices.

**Milestone:** A reliable, transparent sync engine with a tested device abstraction.

## Phase 6 — iPod and Rockbox support

- [ ] Research and document supported iPod models, connection modes, and limitations.
- [ ] Implement and test iPod syncing, including playlists and artwork where supported.
- [ ] Research Rockbox-compatible device and database workflows.
- [ ] Implement Rockbox syncing, including playlists and metadata where supported.
- [ ] Publish a compatibility matrix and device-specific troubleshooting guide.

**Milestone:** Users can sync to explicitly documented iPod and Rockbox configurations.

## Phase 7 — Expand and mature

- [ ] Improve podcast workflows, such as episode status and organization, based on user needs.
- [ ] Consider additional metadata providers, subtitle/artwork support, and portable players.
- [ ] Add migration tools for common library formats where practical.
- [ ] Establish release automation, platform packaging, and a stable support process.
- [ ] Prioritize features from user feedback and maintain the compatibility matrix.

**Milestone:** A stable, maintainable project with a clear release and support lifecycle.

## Out of scope until explicitly planned

Cloud accounts, streaming services, remote hosting, and automatic destructive library/device cleanup are not promised by this roadmap. Any such capability should be evaluated separately, with privacy, security, and data-loss risks documented first.

## How to contribute

Open an issue to discuss a feature, platform, format, or device before beginning substantial implementation. Bug reports should include the application version, operating system, relevant media/device details, and steps to reproduce—without sharing private media or personal data.
