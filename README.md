# iCraftMusic

A sleek, modern media manager for your personal collection of music, movies, TV shows, and podcasts—with syncing support for iPod and Rockbox devices.

## Vision

Bring the familiar, all-in-one library experience of iTunes up to date. iCraftMusic organizes your music, movies, TV shows, and podcasts in one application, with clear media-specific views, polished browsing, and straightforward device syncing—without losing sight of your personal collection.

## Experience

- **One library, tailored views:** Browse each media type in the way that suits it, while keeping everything together in a single application.
- **Familiar organization:** Find your way around a collection with library sections, metadata, artwork, playlists, and search.
- **Modern feel:** Reimagine the convenience of iTunes with a clean, responsive interface designed for today’s devices and large libraries.
- **Your media, your devices:** Manage a local collection and take favorites with you using iPod and Rockbox-compatible players.

## Technology direction

The application will be written in **Rust**, with **egui** for the UI library and **eframe** as the application framework.

## Planned capabilities

- Organize and browse music, movies, TV shows, and podcasts.
- Import local media and keep titles, artwork, playlists, and library details tidy.
- Sync to supported iPod and Rockbox devices.
- Package desktop releases for **macOS**, **Windows**, and **Linux**. Linux releases are planned as **AppImage** downloads, with **zsync** metadata to support incremental updates.

## Project status

The initial Rust/egui/eframe desktop shell is implemented, with navigation for the four planned media sections and a music playback prototype with an Up Next queue. Users can add files or folders for music, movies, TV shows, and podcasts; reorder/remove queued items; and play, pause, stop, seek, or skip audio. Drag-and-drop of files and folders is supported and targets the currently selected section. The library and queue are persisted to the platform config directory between sessions, and a backup can be written on demand. Audio metadata (duration, sample rate, channels, codec) can be read via Symphonia. Importing files does not move or modify them. Folders are tracked as scan roots and can be refreshed with progress reporting. Metadata display/editing in the UI, video playback, and device integration are not implemented yet. Decoder features are enabled for MP3, FLAC, WAV/PCM, Ogg/Vorbis, and MP4/AAC; actual file and platform compatibility remains to be tested. See [ROADMAP.md](ROADMAP.md) and [PRODUCT-TECHNICAL-PLAN.md](PRODUCT-TECHNICAL-PLAN.md) for scope and verification status. Packaging targets above describe intended release formats; release automation and update behavior have not yet been implemented.

Run the desktop application with `cargo run`.

## Contributing

Ideas, issue reports, and contributions are welcome. Please open an issue to discuss significant changes before starting implementation.
