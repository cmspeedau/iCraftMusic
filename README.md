# MusicCraft

A native desktop music player inspired by classic iTunes, written entirely in Rust.
The crate is named `musiccraft`; the repository is `openmusicstreaming`.

MusicCraft is built incrementally against [ROADMAP.md](ROADMAP.md). The current
milestone (M1 — Foundation) is complete: a launchable desktop UI shell with
navigation, an empty-state library, and a persistent playback bar whose controls
are intentionally disabled until the audio engine lands in Stage 2.

## Status

| Stage | Description        | State    |
| ----- | ------------------ | -------- |
| 0     | Project Foundation | Complete |
| 1     | Desktop UI Shell   | Complete |
| 2+    | See ROADMAP.md     | Planned  |

Known limitations (Stage 1): playback controls are rendered as disabled
placeholders, and the library, playlists, and settings views show empty/placeholder
content. No audio, metadata, or persistence beyond application settings exists yet.
These are deliberate and tracked in the roadmap, not presented as finished features.

## Prerequisites

- Stable Rust toolchain (install via [rustup](https://rustup.rs)).
- A desktop session with GPU/OpenGL support (the UI uses `egui`/`eframe`).
- On Linux, the usual `eframe` system libraries, for example:

  ```sh
  # Debian/Ubuntu
  sudo apt install libxkbcommon-dev libgtk-3-dev libgl1-mesa-dev
  # Fedora
  sudo dnf install libxkbcommon-devel gtk3-devel mesa-libGL-devel
  ```

  Exact package names vary by distribution.

## Build

```sh
cargo build            # debug build
cargo build --release  # optimized build
```

## Run

```sh
cargo run
```

The window opens with the navigation sidebar, content area, and playback bar.

## Quality checks

Run these before considering any change complete:

```sh
cargo fmt --check      # formatting
cargo check            # compile check
cargo clippy           # lints
cargo test             # unit tests
```

## Configuration

User settings are stored as TOML in the platform configuration directory
(`config.toml` under the OS-provided config path). A missing or malformed file
falls back to defaults rather than failing; save errors are surfaced in the top
bar instead of crashing.

## Project layout

```
src/
├── main.rs            # binary entry point
├── lib.rs             # library root
├── app.rs             # application state and eframe glue
├── config.rs          # settings model and persistence
├── error.rs           # typed errors (thiserror) and Result alias
├── logging.rs         # tracing subscriber setup
└── ui/
    ├── mod.rs         # shell composition
    ├── navigation.rs  # sidebar sections
    ├── theme.rs       # light/dark theming
    ├── libraryview.rs # library view (placeholder)
    ├── nowplaying.rs  # playback bar
    ├── playlistview.rs# playlist view (placeholder)
    └── settingsview.rs# settings view
```

## License

See [LICENSE](LICENSE).
