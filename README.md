# iCraftMusic

[![CI](https://github.com/cmspeedau/iCraftMusic/actions/workflows/ci.yml/badge.svg)](https://github.com/cmspeedau/iCraftMusic/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/github/license/cmspeedau/iCraftMusic)](https://github.com/cmspeedau/iCraftMusic/blob/main/LICENSE)
[![Latest tag](https://img.shields.io/github/v/tag/cmspeedau/iCraftMusic)](https://github.com/cmspeedau/iCraftMusic/tags)
[![Last commit](https://img.shields.io/github/last-commit/cmspeedau/iCraftMusic)](https://github.com/cmspeedau/iCraftMusic/commits/main)
[![Top language](https://img.shields.io/github/languages/top/cmspeedau/iCraftMusic)](https://github.com/cmspeedau/iCraftMusic)
[![Repo size](https://img.shields.io/github/repo-size/cmspeedau/iCraftMusic)](https://github.com/cmspeedau/iCraftMusic)
[![Issues](https://img.shields.io/github/issues/cmspeedau/iCraftMusic)](https://github.com/cmspeedau/iCraftMusic/issues)

A native Rust desktop player in the idiom of [iTunes 7](#itunes-7-and-the-source-list): a local library with artwork, rule-based playlists, queue, search, and a classic source-list layout. The crate is `musiccraft`; the project is [iCraftMusic](https://github.com/cmspeedau/iCraftMusic). There is no JavaScript runtime or web view. [ROADMAP.md](ROADMAP.md) is authoritative for feature scope and status.

## Status

| Area                            | Feature version | State    | Notes |
| ------------------------------- | --------------- | -------- | ----- |
| Project Foundation               | v0.0            | Complete | |
| Desktop UI Shell                 | v0.1            | Complete | |
| Audio Playback Engine            | v0.2            | Complete | |
| Music Library and Metadata       | v0.3            | Complete | |
| Functional Music Library UI      | v0.4            | Complete | |
| Playlists and Queue Management   | v0.5            | Complete | |
| Metadata Editing and Library Polish | v0.6         | Complete | |
| Settings, Reliability, Performance | v0.7         | Complete | |
| Visual Refinement and Accessibility | v0.8         | Complete | |
| Match the Reference 1:1 (1.0 gate) | v0.8.5       | Complete | |
| Release and Distribution         | v0.9            | Complete | |
| Optional Future Enhancements     | v0.10           | Partial  | |
| Media Library Generalisation     | v0.11           | Planned  | |
| Movies                           | v0.12           | Planned  | Container demux, artwork, per-title grouping for a series |
| TV shows                         | v0.12           | Planned  | Season and episode numbers, remembered next-episode position |
| Podcasts                         | v0.13           | Planned  | RSS/Atom feed parsing, episode download, resume position |
| gPodder sync                     | v0.13           | Planned  | Subscription and episode-action sync with gpodder.net or a self-hosted instance |
| Apple iPod sync                  | v0.14           | Unknown  | Writing the device's checksummed database; approach undecided |
| Rockbox device sync              | v0.15           | Unknown  | Managed file copy plus `.m3u8` to Rockbox players; researched, uncommitted |

The v0.8.5 1.0 gate is **Complete**, including the measured iTunes 7 sidebar, track table, menu band, disc panel, status bar, and Settings sheet. The `v1.0.0` release is also complete: its locked AppImage was published through GitHub Release after release, dependency, licence, and checklist review.

v0.10 is **Partial**: gapless playback, crossfade, a 10-band equalizer, SQL-backed smart playlists, and MusicBrainz cover downloads are built; remaining items are not started. See [ROADMAP.md](ROADMAP.md#v010--optional-future-enhancements).

## What it plays

A scan indexes the music formats the bundled decoder can read. Everything else
is left alone: a text file, a cover image and a playlist in a watched folder are
not counted as music, and a music format this build cannot decode is counted and
named after a scan rather than passed over in silence.

### Music

| Format                    | Extensions              | Play | Read tags | Write tags | Tag block                                    |
| ------------------------- | ----------------------- | ---- | --------- | ---------- | -------------------------------------------- |
| MP3 (MPEG-1/2 Layer III)  | `.mp3`                  | Yes  | Yes       | Yes        | ID3v2                                        |
| FLAC                      | `.flac`                 | Yes  | Yes       | Yes        | Vorbis Comments                              |
| WAV                       | `.wav` `.wave`          | Yes  | Yes       | Yes        | ID3v2                                        |
| Ogg Vorbis                | `.ogg` `.oga`           | Yes  | Yes       | Yes        | Vorbis Comments                              |
| AAC / MP4 family          | `.m4a` `.mp4` `.aac`    | Yes  | Yes       | Yes        | MP4 `ilst` (`.m4a`, `.mp4`), ID3v2 (`.aac`)  |

Every format above reads a title, artist, album, album artist, genre, track and
disc number, duration and embedded cover art, and every one of them accepts a
write. A write is what **Write to file** in the metadata editor does, and it is
the only path in the application that changes a file; a file whose tags cannot
be written is refused with a message rather than written badly.

Recognised as music but not playable by this build, so a scan counts them and
names the formats:

| Extensions                                                                                            |
| ----------------------------------------------------------------------------------------------------- |
| `.aif` `.aiff` `.alac` `.ape` `.dff` `.dsf` `.mka` `.mpc` `.opus` `.ra` `.tak` `.tta` `.wma` `.wv`    |

Nothing else is a media file as far as iCraftMusic is concerned.

Roadmap cross-check: v0.12 covers movies and TV, v0.13 covers podcasts and gPodder synchronization, and v0.14/v0.15 remain **Unknown**. See [ROADMAP.md](ROADMAP.md#v012--movies-and-tv-shows), [ROADMAP.md](ROADMAP.md#v013--podcasts-and-gpodder-synchronization), [ROADMAP.md](ROADMAP.md#v014--apple-ipod-synchronization), and [ROADMAP.md](ROADMAP.md#v015--rockbox-device-synchronization).

## The source list and the three pages

The left panel is a source list in the old sense: under the **LIBRARY** heading
sits **Music**, and under a **PLAYLISTS** heading — which folds away with its
own disclosure triangle — the playlists themselves, the open one marked by a
full-bleed slate-to-shade bar with a single darker row seating it at the top.
Clicking a playlist opens the Playlists page on it; there is no separate
Playlists row because the list under the heading already is that page.

Music browses the same rows three ways. The three page buttons — a note, a jewel
case, a person — sit on the playback bar, and `Ctrl/⌘ 1`, `2` and `3` are the
keystrokes for the same choices.

**All Tracks** — the whole library as one sortable, filterable table.

**Albums** — one tile per album, with the cover the scanner cached, the artist
and the track count. A click opens the album: the page becomes the record —
cover art with the album's name and artist at the left, its tracks listed in
track-number order at the right, where a row plays from that song and can be
dragged onto a playlist. Pressing the jewel-case button again (or `Esc`) falls
back to the covers.

**Artists** — the artist list; a click marks a row and stays on the page.
Double-click plays the artist, and **Show their tracks** is in the right-click
menu.

Music comes in three ways: **Add files…** and **Add folder…** open the
desktop's choosers from the gear at the right of the status bar, and folders or
files can be dragged onto the window. The drag is the one way the desktop can
thwart: on a Wayland session the windowing toolkit receives no file drops, so
there the drag needs the window run through XWayland with **`--x11`** (the two
chooser rows work everywhere). A copy adds files by asking the desktop to place
them in the **Music home** — the first watched folder is the media home, and it
is where copies land — while files already somewhere else in the library are
indexed where they stand.

## Current functionality

| Area                  | What works                                                                                                          |
| --------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Playback              | Play, pause, stop, seek, volume, previous and next; device, decoder and missing-file failures reported in place of the track detail |
| Gapless playback      | The queue lines up the next track's audio behind the current one, so consecutive tracks change over with no silence between them; the change-over is reported from the audio, not a frame late |
| Queue                 | Move a track earlier or later, remove it, clear the list, jump to any entry; shuffle and repeat                       |
| Music — All Tracks    | Sortable table (title, artist, album, genre, duration, plays) with click, ctrl-click and shift-click selection        |
| Music — Albums        | Album grid with cover art, decoded once per album and re-decoded when the texture cache fills; clicking a sleeve opens the album's screen — cover, name and artist beside its track list |
| Music — Artists       | Artist list, virtualised like the track table                                                                        |
| Search                | Live filtering as the query is typed, over the rows already in memory; typing never touches the database              |
| Playlists             | Create, rename, copy and delete; add tracks from the Music list's context menu or by dragging a track row (or the whole selection) onto a playlist's row in the sidebar; reorder entries; play the whole list, or from any row by double-click |
| Smart playlists       | Rules on the fields the library really holds, matched all-or-any, with a limit by most-played or recently-added; membership is a compiled query, so the list is never stale and a duplicate copies the rules, not today's answer |
| Metadata editing      | Title, artist, album, album artist, genre, track and disc numbers, and the cover; save to the database or write the tags into the file |
| Play counts           | A **Plays** column, counted by the library rather than by the file, so a rescan does not reset it                     |
| Sorting and filtering | A leading article is ignored (`The Beatles` files under B); the table can be narrowed to tracks whose files have vanished |
| Keyboard              | Space, `Ctrl/⌘ ←/→/↑/↓`, `Ctrl/⌘ L/P/,/1/2/3/E`, `Delete` and `Esc`; Tab and the arrow keys walk a visible focus ring, and Enter acts on the row it rests on — the table, the albums, the artists, a playlist — as a double-click would; listed in Settings, and none fires while a text field has focus |
| Accessibility         | Every focusable control shows the ring, and every glyph-only button and slider gives a screen reader the words the pointer gets as hover text (`Repeat: this track`, not `🔁`) through AccessKit |
| Settings              | A window of its own (gear or `Ctrl/⌘ ,`, never a page that replaces the music): Library management (watched folders, rescan), Playback (volume, crossfade, output device), the keyboard table, Storage, Diagnostics and About, each change persisted as it happens        |
| Equalizer             | A window of its own from the gear: ten band gains, a preamp and presets, able to stay open beside the music so a band is trimmed by ear                                                             |
| Diagnostics           | Build, schema version, library counts, exact paths, the audio stream in use, and an on-demand `PRAGMA integrity_check` |
| Recovery              | A versioned database copied beside itself (`VACUUM INTO`) before a migration rewrites it                             |
| Packaging             | Release profile, desktop entry and icon set under `packaging/`; `linuxdeploy` bundles them with the release binary into a single-file AppImage |
| Updates               | One check of the releases API at startup; when a newer release than this build exists, the centre box says so and the gear menu carries the row that installs it — which swaps the running file only when the application is running from an AppImage, and downloads nothing until it is asked |
| Cover art download    | The album screen (and a right-click on any sleeve in the grid) asks MusicBrainz's Cover Art Archive for that record's front image — one at a time, only when asked, never automatically; the fetched cover lands in the same cache as extracted art and no audio file is touched |

Known limitations: a scan can be stopped but is not resumed after the application
exits. Editing a track's details keeps them in the library until the file changes
or the tags are written into the file itself. A file played without being indexed
(or dragged onto the window from outside the library) has no row to count a play
against. Gapless playback covers every track order the queue can name before the
current track ends, but an edit to the queue in the moment before a change-over —
removing or moving the track that was lined up — makes the player follow the
queue with a normal load, which can be heard as a short gap; this is the one case
where a change is not seamless. A crossfade is stepped on the frame clock rather
than per sample, so a fade is smooth but not exact; it also needs a track whose
length the decoder reported, and a track shorter than the fade finishes before
the fade does. The equalizer has ten fixed bands and one preset shape per band.
Smart playlists rule over the fields the database has columns for; year and
rating join the rule fields the day those columns exist. The theme follows the
system's light/dark setting — `theme` can be set in the settings file, but there
is no control for it in the interface yet. Only two requests ever leave the
machine, and only when the user asks — the update check, which sends nothing
but this build's own name and version to the releases API, and a cover
download, which sends nothing but the album and artist strings of the one
record it was asked about to MusicBrainz. The check stays silent unless the
latest release is genuinely newer than this build, and a released copy never
hears about itself. Nothing about the library, the listening history or the
machine is sent. These are deliberate and tracked in the roadmap, not
presented as finished features.

## iTunes 7, and the source list

The reference is **iTunes 7** — the October 2006 release that introduced the
source list. That is what the left panel copies, measured pixel-for-pixel off a
running copy: slate capital group headings with the fold triangle on the
playlists, the full-bleed selection bar (pale slate deepening to shade, seated
by a darker row of pixels at its top), a paler one under the pointer's row, a
note or a ruled card or a lensed badge beside each name, and a near-black hairline
where the panel meets the page. The rest of the window
follows the same year's idiom: the bevelled, slightly glossy chrome of the
centre box, the rounded groove of the progress bar, the jewel-case page icons.
What the era really stands for, and what this project follows, is the shape of
the thing — a local library you own, browsed rather than fed to you.

## Prerequisites

- Stable Rust toolchain (install via [rustup](https://rustup.rs)), and a C
  compiler — SQLite is compiled in, and that is the one crate with C in it.
- On Linux, the audio backend links ALSA at build time:

  ```sh
  # Debian/Ubuntu
  sudo apt install libasound2-dev pkg-config
  # Fedora
  sudo dnf install alsa-lib-devel pkgconf-pkg-config
  ```

  Everything else the UI needs — the display server and OpenGL — is opened at
  run time, so the only run-time requirement is a desktop session with a
  driver that speaks OpenGL 3.3 or newer.

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

The window opens with the source list, the content page, and the playback bar.

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

What is stored: the theme (which follows the system unless the file says
otherwise), the volume, the folders the library watches — the first of them is
the Music home where copies are placed — the output device iCraftMusic plays
through (`output_device`, omitted entirely when the system default is wanted),
whether playback shuffles and repeats, the crossfade length, and the equalizer.
Changing any of them writes the file at the end of the frame.

The library database, the cached cover art and the log live in the platform's
data directory, apart from the settings:

| Path (Linux)                                        | What it is                     |
| --------------------------------------------------- | ------------------------------ |
| `~/.config/icraftmusic/config.toml`                 | preferences                    |
| `~/.local/share/icraftmusic/library.sqlite`         | the indexed library            |
| `~/.local/share/icraftmusic/logs/icraftmusic.log`   | this run's log (and the one before it) |

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
# 1. Stop iCraftMusic, then move the database aside (the cover-art cache and the
#    music files are untouched).
mv ~/.local/share/icraftmusic/library.sqlite* /tmp/library-broken/

# 2. Start iCraftMusic and press Rescan under Settings → Library management.
#    Every watched folder is indexed again from the files themselves.
```

Upgrading the schema never happens silently on top of your only copy: before a
migration rewrites a database written by an older build, iCraftMusic writes a
consistent copy beside it with SQLite's `VACUUM INTO`:

```
library.sqlite.pre-vN.bak
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
for size in 16 32 48 64 128 256 512; do
    install -Dm644 packaging/icons/hicolor/${size}x${size}/apps/musiccraft.png \
        ~/.local/share/icons/hicolor/${size}x${size}/apps/musiccraft.png
done
update-desktop-database ~/.local/share/applications  # optional, for MimeType
```

`packaging/` holds the parts a distribution package would install:

- `musiccraft.desktop` — the desktop entry (name, categories, MIME types, and
  the `StartupWMClass` that ties the window to its launcher icon).
- `icons/hicolor/…/apps/musiccraft.png` — the artwork at the seven sizes an
  icon theme asks for, cut to its rounded tile, so the corners are transparent
  rather than white canvas. The window wears the 256px rendition itself — it is
  embedded in the binary — so the launcher, the taskbar and the desktop entry
  show the same picture.

The release profile in `Cargo.toml` builds one stripped binary with link-time
optimisation. A published release comes with the `v1.0.0` tag and everything
after it: pushing the tag is the deliberate act, and `.github/workflows/release.yml`
builds the locked binary into an AppImage and publishes that as a GitHub Release.
The AppImage is the shape the in-app updater installs: it looks at
the latest GitHub release for an asset named after this machine's architecture
(`-x86_64.AppImage`), and installs by renaming that file over the running
AppImage, which is a swap only an AppImage copy can make.

### Bundling the AppImage

The same pieces, bundled: [linuxdeploy](https://github.com/linuxdeploy/linuxdeploy)
assembles the directory and compresses it into one file.

```sh
cargo build --release
VERSION=1.0.0 ARCH=x86_64 ./linuxdeploy-x86_64.AppImage \
    --appdir "$PWD/iCraftMusic.AppDir" \
    -e target/release/musiccraft \
    -d packaging/musiccraft.desktop \
    -i packaging/icons/hicolor/256x256/apps/musiccraft.png \
    -i packaging/icons/hicolor/512x512/apps/musiccraft.png \
    --output appimage
# → iCraftMusic-1.0.0-x86_64.AppImage
```

(The absolute `--appdir` matters: the tools are AppImages themselves, and a
relative directory is looked up from wherever they unpacked themselves to run
— which is also why the finished file can land in the home directory rather
than beside the AppDir. Look for it with `ls iCraftMusic-*.AppImage`.)

The bundle carries exactly one executable, the desktop entry, and the two
largest renditions of the icon set — linuxdeploy points the AppDir's own icon
at the smaller of the pair, which is what `.DirIcon` becomes, and a theme
scales the rest down from there. That is deliberate: linuxdeploy's rules keep
`libc`, ALSA and the rest of the system libraries on the host, and the player
opens the display server and OpenGL at run time anyway, so there is nothing
else to bundle. The file's name ends the way the in-app updater expects
(`…-x86_64.AppImage`), so a bundled image is a GitHub release asset as-is —
which is exactly what the release job publishes on a `v*` tag.

Two honest caveats, both ordinary:

- The image inherits the build machine's glibc. Built on a current distro it
  runs on current distros; an image meant for older machines must be compiled
  on an older base, since shipping `libc` inside the app is not a thing that
  works.
- A machine whose session has no FUSE still runs it: append
  `--appimage-extract-and-run`, which unpacks to a temporary directory, runs,
  and cleans up.

## Project layout

```
ROADMAP.md             # unified roadmap: music and media versions v0.0–v0.15
ARCHITECTURE-APPLE-TV-CONTROLS.md  # living-room appliance controls design (proposal)
CHANGELOG.md           # documentation changelog
VERSIONING.md          # documentation versioning and tag guidance
packaging/             # desktop entry and icon set — shared by a manual install and the AppImage

src/
├── main.rs            # binary entry point
├── lib.rs             # library root
├── app/               # application state, methods grouped by the surface they serve
│   ├── mod.rs         #   the struct, the constructors, the eframe glue
│   ├── windows.rs     #   which panes are open: section, queue, Settings, equalizer, gear
│   ├── release.rs     #   the update check and the install, as the shell holds them
│   ├── playlists.rs   #   opened playlists and their editing
│   ├── playback.rs    #   the transport, the queue, and how tracks enter them
│   ├── artwork.rs     #   the cover download's shell state: pending, busy, notice
│   ├── library.rs     #   watched folders, scans, imports, copies, the media home
│   ├── editor.rs      #   the track tag editor
│   ├── confirm.rs     #   the one pending confirmation the views may ask for
│   ├── settings.rs    #   theme, volume, equalizer, output device, diagnostics
│   └── tests.rs       #   the window-level behaviour tests
├── config.rs          # settings model and persistence
├── diagnostics.rs     # what the installation looks like, and the integrity report
├── error.rs           # typed errors (thiserror) and Result alias
├── fixtures.rs        # the sample library the tests share
├── logging.rs         # tracing subscriber: stderr and a rotated log file
├── update.rs          # the releases check and the AppImage swap
├── audio/             # output device, queue, transport, equalizer
├── library/           # models, scanning, tag reading and writing, browse state, cover fetching
├── database/          # SQLite schema and repository
├── playlists/         # playlists, their membership, and smart rules
├── search/            # pure query, sort and filtering over library rows
└── ui/
    ├── mod.rs         # shell composition
    ├── navigation.rs  # the source list: sections and the playlist list
    ├── theme.rs       # light/dark theming, the focus ring, and accessible names
    ├── artwork.rs     # the texture cache behind the album grid
    ├── libraryview.rs # the Music pages: tracks, albums, artists
    ├── editview.rs    # the metadata editor
    ├── dialog.rs      # confirmation for destructive library actions
    ├── shortcuts.rs   # playback and navigation keys
    ├── nowplaying.rs  # playback bar
    ├── queueview.rs   # the queue popover
    ├── playlistview.rs# the open playlist and its tracks
    ├── playlistedit.rs# the playlist name editor
    ├── smartplaylist.rs# the smart playlist rule editor
    ├── settingsview.rs# the Settings window and its panes
    ├── equalizerview.rs# the Equalizer window
    └── statusbar.rs   # the bar across the foot of the window, and the gear
```

The [Apple TV controls architecture](ARCHITECTURE-APPLE-TV-CONTROLS.md) is a proposal for a living-room appliance build. It remains outside the committed v0.0–v0.15 roadmap and makes no claim of support until its target and integration decisions are verified.

## License

See [LICENSE](LICENSE).
