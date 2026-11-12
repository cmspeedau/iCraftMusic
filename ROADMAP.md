# iCraftMusic Roadmap

This roadmap is a starting point for planning. Priorities and sequencing may change as the project’s platform, architecture, and community needs become clearer. “Done” means the milestone has a usable, documented outcome—not just a placeholder.

## Guiding principles

- Keep the experience polished, approachable, and responsive for large libraries.
- Protect users’ media and metadata; make destructive actions explicit and recoverable where practical.
- Prefer open, documented formats and avoid locking users into a service.
- Treat device compatibility as a first-class feature, with safe transfer and clear diagnostics.
- Ship incremental, testable milestones and document supported platforms and formats.

## Phase 0 — Define the foundation

- [ ] Decide target desktop platforms and application architecture.
- [ ] Establish project contribution guidelines, coding standards, and issue templates.
- [ ] Document initial supported media formats and metadata sources.
- [ ] Define data-storage and backup principles, including how original files are handled.
- [ ] Create a UX prototype for library browsing, search, and device status.

**Milestone:** A reviewed product and technical plan that can guide a first implementation.

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
