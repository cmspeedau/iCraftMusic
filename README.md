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
| 7     | Settings, Reliability, Performance | Complete |
| 8+    | See ROADMAP.md                    | Planned  |

Known limitations: a scan indexes `mp3`, `flac`, `wav`,
`ogg`, `m4a`, `mp4` and `aac` files; music in any other format is counted and
named after a scan but cannot be played or tagged. Editing a track's details in
the library keeps them there until the file changes or the tags are written into
the file itself. A scan can be stopped but is not resumed after the application
exits. There is no installer or release artifact yet; packaging is prepared
under `packaging/` (see below), not finished. These are deliberate and tracked
in the roadmap, not presented as finished features.

The All Tracks table has a **Plays** column: how many times each track has been
started. It is counted by the library, not by the file — nothing in an audio tag
carries it, so a rescan does not reset it — and the heading sorts by it like any
other column (low to high first, click again for most-played first). A file
played without being indexed (the play field, or the command line) has no row to
count against and silently counts nothing.

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

- The folder chooser in Settings → Library management is the desktop's own file
  dialog. On Linux that is `xdg-desktop-portal`, which every KDE/GNOME session
  already runs; if no portal is available the button reports the failure rather
  than opening nothing. The path field beside it always works.

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

What is stored: the theme, the volume, the folders the library watches, and the
output device MusicCraft plays through (`output_device`, omitted entirely when
the system default is wanted). Changing any of them writes the file at the end
of the frame.

The library database, the cached cover art and the log live in the platform's
data directory, apart from the settings:

| Path (Linux)                                        | What it is                     |
| --------------------------------------------------- | ------------------------------ |
| `~/.config/musiccraft/config.toml`                   | preferences                    |
| `~/.local/share/musiccraft/library.sqlite`           | the indexed library            |
| `~/.local/share/musiccraft/logs/musiccraft.log`      | this run's log (and the one before it) |

Settings → Diagnostics shows the exact paths on your machine, the schema version
of the database, the counts, the audio stream in use, and a button that checks
the database. Set `RUST_LOG=musiccraft=debug` to make the log talkative.

## Library data and recovery

The database is a cache of what the files on disk say: removing a row never
deletes a file, and a rescan rebuilds what is missing. Nothing in the
application damages your music.

If the library misbehaves, Settings → Diagnostics → **Check the library
database** runs SQLite's own `PRAGMA integrity_check` and reports what it found
without changing anything. To rebuild from scratch:

```sh
# 1. Stop MusicCraft, then move the database aside (the cover-art cache and the
#    music files are untouched).
mv ~/.local/share/musiccraft/library.sqlite* /tmp/library-broken/

# 2. Start MusicCraft and press Rescan under Settings → Library management.
#    Every watched folder is indexed again from the files themselves.
```

Upgrading the schema never happens silently on top of your only copy: before a
migration rewrites a database written by an older build, MusicCraft writes a
consistent copy beside it with SQLite's `VACUUM INTO`:

```
library.sqlite.pre-v2.bak
```

If an upgrade goes wrong, that file is the library as the previous version left
it — move it back to `library.sqlite` and start the older build.

## Packaging

A release build and what a package needs:

```sh
cargo build --release
# → target/release/musiccraft

install -Dm755 target/release/musiccraft ~/.local/bin/musiccraft
install -Dm644 packaging/musiccraft.desktop \
    ~/.local/share/applications/musiccraft.desktop
install -Dm644 packaging/musiccraft.svg \
    ~/.local/share/icons/hicolor/scalable/apps/musiccraft.svg
update-desktop-database ~/.local/share/applications  # optional, for MimeType
```

`packaging/` holds the parts a distribution package would install:

- `musiccraft.desktop` — the desktop entry (name, categories, MIME types, and
  the `StartupWMClass` that ties the window to its launcher icon).
- `musiccraft.svg` — the icon, in plain SVG so a packager can rasterise whatever
  sizes the icon theme asks for.

The release profile in `Cargo.toml` builds one stripped binary with link-time
optimisation. There is no installer, and no artifact is published yet — that is
Stage 9's job; what is here is the preparation for it.

## Project layout

```
src/
├── main.rs            # binary entry point
├── lib.rs             # library root
├── app.rs             # application state and eframe glue
├── config.rs          # settings model and persistence
├── diagnostics.rs     # what the installation looks like, and the integrity report
├── error.rs           # typed errors (thiserror) and Result alias
├── logging.rs         # tracing subscriber: stderr and a rotated log file
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
    ├── playlistview.rs# the Playlists section
    ├── playlistedit.rs# the playlist name editor
    ├── settingsview.rs# settings view
    └── statusbar.rs   # the bar across the foot of the window
```

## License

See [LICENSE](LICENSE).
