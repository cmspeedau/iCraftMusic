# MusicCraft

A native desktop music player inspired by classic iTunes, written entirely in Rust.
The crate is named `musiccraft`; the repository is `openmusicstreaming`.

MusicCraft is built incrementally against [ROADMAP.md](ROADMAP.md), which is the
authority on what each stage delivers and on what is still missing.

## Status

| Stage | Description                       | State    |
| ----- | --------------------------------- | -------- |
| 0     | Project Foundation                | Complete |
| 1     | Desktop UI Shell                  | Complete |
| 2     | Audio Playback Engine             | Complete |
| 3     | Music Library and Metadata        | Complete |
| 4     | Functional Music Library UI       | Complete |
| 5     | Playlists and Queue Management    | Complete |
| 6     | Metadata Editing and Library Polish | Complete |
| 7+    | See ROADMAP.md                    | Planned  |

Known limitations: the Playlists section is still a placeholder — the queue
(`☰` in the playback bar) is the only way to arrange what plays, and playlists
are not yet creatable from the interface. A scan indexes `mp3`, `flac`, `wav`,
`ogg`, `m4a`, `mp4` and `aac` files; music in any other format is counted and
named after a scan but cannot be played or tagged. Editing a track's details in
the library keeps them there until the file changes or the tags are written into
the file itself. These are deliberate and tracked in the roadmap, not presented
as finished features.

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
├── audio/             # output device, queue, transport
├── library/           # models, scanning, tag reading and writing, browse state
├── database/          # SQLite schema and repository
├── playlists/         # playlists and their membership
├── search/            # pure query, sort and filtering over library rows
└── ui/
    ├── mod.rs         # shell composition
    ├── navigation.rs  # sidebar sections
    ├── theme.rs       # light/dark theming
    ├── libraryview.rs # library view: tracks, albums, artists
    ├── editview.rs    # the metadata editor
    ├── dialog.rs      # confirmation for destructive library actions
    ├── shortcuts.rs   # playback and navigation keys
    ├── nowplaying.rs  # playback bar
    ├── queueview.rs   # the queue popover
    ├── playlistview.rs# playlist view (placeholder)
    ├── settingsview.rs# settings view
    └── statusbar.rs   # the bar across the foot of the window
```

## License

See [LICENSE](LICENSE).
