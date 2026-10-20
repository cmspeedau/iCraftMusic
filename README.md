# iCraftMusic

A native desktop player dressed in the idiom of [iTunes 7](#itunes-7-and-the-source-list),
written entirely in Rust: a local library, album artwork, playlists that fill
themselves from rules, a queue, a search field that is always a keystroke away,
and one window with the source list on the left, the page in the middle, the
player across the top and a status strip along the foot.

**Written entirely in Rust** — every line of application code, from the audio
engine to the tag writer to the UI. The only non-Rust things in the tree are the
desktop entry and the icon a Linux package would install. There is no JavaScript
runtime and no web view anywhere in it.

The crate is named `musiccraft`.
[iCraftMusic](https://github.com/cmspeedau/iCraftMusic) — it is a music player,
and the name on the tin says so.

iCraftMusic is built incrementally against [ROADMAP.md](ROADMAP.md), which is the
authority on what each stage delivers and on what is still missing.
[ROADMAP-MEDIA.md](ROADMAP-MEDIA.md) is the second roadmap: movies, TV shows and
podcasts, with gPodder synchronization — and two kinds of device synchronization,
Apple iPod and [Rockbox](https://www.rockbox.org/)-running players, whose
approaches are not yet decided and which that file marks **Unknown** rather than
pretending otherwise.

## Status

| Stage | Description                         | State    |
| ----- | ----------------------------------- | -------- |
| 0     | Project Foundation                  | Complete |
| 1     | Desktop UI Shell                    | Complete |
| 2     | Audio Playback Engine               | Complete |
| 3     | Music Library and Metadata          | Complete |
| 4     | Functional Music Library UI         | Complete |
| 5     | Playlists and Queue Management      | Complete |
| 6     | Metadata Editing and Library Polish | Complete |
| 7     | Settings, Reliability, Performance  | Complete |
| 8     | Visual Refinement and Accessibility | Complete |
| 9     | Release and Distribution            | Planned  |
| 10    | Optional Future Enhancements        | Partial  |

Stage 10 collects features that are optional rather than prerequisites, and
four of them have been built: **gapless playback**, **crossfade**, a **graphic
equalizer** and **smart playlists**. Consecutive tracks whose order the queue
already knows change over with no silence between them, because the track that
follows is handed to the audio device before the current one ends and the
hand-over is reported from the audio itself. A crossfade length above zero
blends the two instead, overlapping them by that many seconds on the engine's
two decks. The equalizer is a ten-band peaking-filter bank with a preamp and
presets, applied to every track as it plays. A smart playlist stores rules —
clauses on title, artist, album, genre, track number, play count, length and
date added, matched all-or-any with an optional limit by most-played or
recently-added — and the application turns those rules into one SQL query every
time the playlist is read, so the list fills itself and stays current: a track
that begins matching the moment it joins the library is already in the list the
next time it is opened. The other additions the roadmap lists are not started.
See [ROADMAP.md](ROADMAP.md#stage-10--optional-future-enhancements) for the
detail.

Beyond music, five further stages are recorded in
[ROADMAP-MEDIA.md](ROADMAP-MEDIA.md):

| Stage | Description                          | State   |
| ----- | ------------------------------------ | ------- |
| 11    | Media Library Generalisation         | Planned |
| 12    | Movies and TV Shows                  | Planned |
| 13    | Podcasts and gPodder Synchronization | Planned |
| 14    | Apple iPod Synchronization           | Unknown |
| 15    | Rockbox Device Synchronization       | Unknown |

The two roadmaps share one foundation: Stage 11 generalises the library and the
player so that a film, an episode and a podcast episode are rows beside a song
rather than a second application, and Stages 12 and 13 then add the surfaces
those rows need.

**Unknown** is a state of its own, not a shy way of writing Planned. Stages 11 to
13 each have an approach that has been thought through and only the work left;
Stages 14 and 15 do not, for opposite reasons. Stage 14 cannot yet decide: an
iPod verifies a checksummed database against a hash that, on the newer
generations, cannot be derived without a file only iTunes can produce — so
writing to the device is a different class of operation from anything else in
this application, and its three candidate approaches have no obvious winner.
Stage 15 has decided nothing because it is scope that is undecided, not method: a
player running Rockbox reads the filesystem directly, so syncing one is a managed
file copy — `.m3u8` playlists and a `cover.jpg` per album — and what remains open
is how far the application reaches into Rockbox's own databases, and whether the
stage is built at all. [ROADMAP-MEDIA.md](ROADMAP-MEDIA.md) records the research
and the open questions for both rather than inventing plans to have them.

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

### Planned

| Media         | Stage | State   | Notes                                                             |
| ------------- | ----- | ------- | ----------------------------------------------------------------- |
| Movies        | 12    | Planned | Container demux, artwork, per-title grouping for a series          |
| TV shows      | 12    | Planned | Season and episode numbers, remembered next-episode position       |
| Podcasts      | 13    | Planned | RSS/Atom feed parsing, episode download, resume position           |
| gPodder sync  | 13    | Planned | Subscription and episode-action sync with gpodder.net or a self-hosted instance |
| Apple iPod sync | 14  | Unknown | Writing the device's checksummed database; approach undecided      |
| Rockbox device sync | 15 | Unknown | Managed file copy plus `.m3u8` to Rockbox players; researched, uncommitted |

The roadmap for these is [ROADMAP-MEDIA.md](ROADMAP-MEDIA.md), which also carries
the gPodder API reference those stages are written against.

The two **Unknown** entries are listed as such for different reasons. Apple iPod
sync has three candidate approaches and no decision. An iPod reads a database of
what is on it, and since the iPod classic that database is verified against a
cryptographic hash — so a sync means rewriting the database and recomputing the
hash, and on the newer generations that hash depends on a `HashInfo` file only
iTunes can generate. Three approaches are on the table (bindings to the C
`libgpod`, a from-scratch Rust implementation of the format, or export-only and
never write to the device); the trade-offs, the generation-by-generation
requirements and the open questions are written down in
[ROADMAP-MEDIA.md](ROADMAP-MEDIA.md#stage-14--apple-ipod-synchronization).

Rockbox device sync is the reverse case: the research is done and the commitment
is what is missing. A player running Rockbox browses the filesystem instead of a
verified database, so a sync there is a managed file copy — `.m3u8` playlists, a
`cover.jpg` per album, the same tags lofty already reads — with no `iTunesDB`, no
hashes and no C library. Undecided is the first version's scope (files,
playlists and cover art, with hi-res files the target is not confirmed to play
named and skipped), whether the application ever reaches into Rockbox's own
databases for play counts and ratings, and whether the stage is built at all.
The findings, and the licence boundary that keeps the larger of the two
open-source references (GPL-3.0, reference-only here) out of this MIT project,
are written down in
[ROADMAP-MEDIA.md](ROADMAP-MEDIA.md#stage-15--rockbox-device-synchronization).

## The source list and the three pages

The left panel is a source list in the old sense: under the **LIBRARY** heading
sits **Music**, and under a **PLAYLISTS** heading the playlists themselves, the
open one marked by the blue bar that runs the panel's whole width. Clicking a
playlist opens the Playlists page on it; there is no separate Playlists row
because the list under the heading already is that page.

Music browses the same rows three ways. The three page buttons — a note, a jewel
case, a person — sit on the playback bar, and `Ctrl/⌘ 1`, `2` and `3` are the
keystrokes for the same choices.

**All Tracks** — the whole library as one sortable, filterable table.

**Albums** — one tile per album, with the cover the scanner cached, the artist
and the track count.

**Artists** — the artist list, which narrows Music to one name when a row is
picked, the same way clicking an album tile does.

Music comes in two ways, both from the gear at the right of the status bar:
**Add files…** and **Add folder…** open the desktop's choosers, and folders or
files can be dropped onto the window. A copy adds files by asking the desktop
to place them in the **Music home** — the first watched folder is the media
home, and it is where copies land — while files already somewhere else in the
library are indexed where they stand.

## Current functionality

| Area                  | What works                                                                                                          |
| --------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Playback              | Play, pause, stop, seek, volume, previous and next; device, decoder and missing-file failures reported in place of the track detail |
| Gapless playback      | The queue lines up the next track's audio behind the current one, so consecutive tracks change over with no silence between them; the change-over is reported from the audio, not a frame late |
| Queue                 | Move a track earlier or later, remove it, clear the list, jump to any entry; shuffle and repeat                       |
| Music — All Tracks    | Sortable table (title, artist, album, genre, duration, plays) with click, ctrl-click and shift-click selection        |
| Music — Albums        | Album grid with cover art, decoded once per album and re-decoded when the texture cache fills                         |
| Music — Artists       | Artist list, virtualised like the track table                                                                        |
| Search                | Live filtering as the query is typed, over the rows already in memory; typing never touches the database              |
| Playlists             | Create, rename, copy and delete; add tracks from the Music list's context menu; reorder entries; play the whole list, or from any row by double-click |
| Smart playlists       | Rules on the fields the library really holds, matched all-or-any, with a limit by most-played or recently-added; membership is a compiled query, so the list is never stale and a duplicate copies the rules, not today's answer |
| Metadata editing      | Title, artist, album, album artist, genre, track and disc numbers, and the cover; save to the database or write the tags into the file |
| Play counts           | A **Plays** column, counted by the library rather than by the file, so a rescan does not reset it                     |
| Sorting and filtering | A leading article is ignored (`The Beatles` files under B); the table can be narrowed to tracks whose files have vanished |
| Keyboard              | Space, `Ctrl/⌘ ←/→/↑/↓`, `Ctrl/⌘ L/P/,/1/2/3/E`, `Delete` and `Esc`; Tab and the arrow keys walk a visible focus ring, and Enter acts on the row it rests on — the table, the albums, the artists, a playlist — as a double-click would; listed in Settings, and none fires while a text field has focus |
| Accessibility         | Every focusable control shows the ring, and every glyph-only button and slider gives a screen reader the words the pointer gets as hover text (`Repeat: this track`, not `🔁`) through AccessKit |
| Settings              | Library management (watched folders, rescan), Playback (volume, crossfade, output device), the Equalizer, the keyboard table, Storage, Diagnostics and About, each change persisted as it happens |
| Diagnostics           | Build, schema version, library counts, exact paths, the audio stream in use, and an on-demand `PRAGMA integrity_check` |
| Recovery              | A versioned database copied beside itself (`VACUUM INTO`) before a migration rewrites it                             |
| Packaging             | Release profile, desktop entry and SVG icon prepared under `packaging/`                                              |
| Updates               | One check of the releases API at startup; when a newer release than this build exists, the centre box says so and the gear menu carries the row that installs it — which swaps the running file only when the application is running from an AppImage, and downloads nothing until it is asked |

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
is no control for it in the interface yet. There is no published release yet:
the update check is the only network request the application makes — it sends
nothing but this build's own name and version to the releases API — and while no
release has been published it stays silent, no notice at all. Nothing about the
library, the listening history or the machine is sent. These are deliberate and
tracked in the roadmap, not presented as finished features.

## iTunes 7, and the source list

The reference is **iTunes 7** — the October 2006 release that introduced the
source list. That is what the left panel copies: grey capital group headings,
the flat blue bar with its lit top edge across the open row, a paler one under
the pointer's row, and a note or a disc beside each name. The rest of the window
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
optimisation. There is no published release yet — that is Stage 9's job; what is
here is the preparation for it. The in-app updater is ready for it: it looks at
the latest GitHub release for an asset named after this machine's architecture
(`-x86_64.AppImage`), and installs by renaming that file over the running
AppImage, which is a swap only an AppImage copy can make.

## Project layout

```
ROADMAP.md             # music: stages 0–10
ROADMAP-MEDIA.md       # movies, TV shows, podcasts and devices: stages 11–15

src/
├── main.rs            # binary entry point
├── lib.rs             # library root
├── app.rs             # application state and eframe glue
├── config.rs          # settings model and persistence
├── diagnostics.rs     # what the installation looks like, and the integrity report
├── error.rs           # typed errors (thiserror) and Result alias
├── fixtures.rs        # the sample library the tests share
├── logging.rs         # tracing subscriber: stderr and a rotated log file
├── update.rs          # the releases check and the AppImage swap
├── audio/             # output device, queue, transport, equalizer
├── library/           # models, scanning, tag reading and writing, browse state
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
    ├── settingsview.rs# settings view
    └── statusbar.rs   # the bar across the foot of the window, and the gear
```

## License

See [LICENSE](LICENSE).
