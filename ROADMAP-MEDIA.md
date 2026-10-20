# Roadmap — Media Beyond Music

The second roadmap. [ROADMAP.md](ROADMAP.md) covers the music player, stages 0
to 10; this file carries stages **11 to 15**, which turn iCraftMusic from a music
player into a media player: movies, TV shows, and podcasts with gPodder
synchronization — and, with their approaches still undecided, Apple iPod
synchronization and Rockbox device synchronization. All of them are later
versions: 1.0 is music and only music, and no stage here is a prerequisite for
it.

It is deliberately separate. The music stages are finished and their acceptance
criteria were verified against the running application; the stages below are
unstarted, and keeping them in one file made it look as though the music work
was still open. The two files share one status table in
[README.md](README.md).

Stages 14 and 15 are marked **Unknown** rather than Planned. Everything up to
Stage 13 has an approach that has been thought through; neither device
synchronization stage does, and writing either down as though it did would be the
exact thing this file exists to avoid. The two are Unknown for opposite reasons:
Stage 14 cannot yet choose between its three candidate approaches; Stage 15 has
finished its research and has not committed to building any of it.

## Why these stages exist

Everything a film, an episode or a podcast episode needs is already in the
application, in music-shaped form:

| The library already has            | A film / episode needs                              |
| ---------------------------------- | --------------------------------------------------- |
| A row per file with a title, artist, album, genre, duration and a play count | A row per file with a title, series, season/episode, duration and a resume position |
| A scanner that walks watched folders and skips what it cannot read | The same walk, with a wider set of readable formats |
| A sortable, filterable, selectable table with artwork | The same table, with the columns the media kind actually has |
| A queue, a transport, shuffle and repeat | The same transport, playing a picture-less stream   |
| Playlists                                                        | Collections that span kinds |
| An SQLite library with a versioned schema and backups            | More tables, a migration, and the same backup rule  |

So the work is not "build three more applications". It is "generalise the one
that exists, then add the two surfaces the new rows need".

## Core constraints

These extend the constraints in [ROADMAP.md](ROADMAP.md) and do not replace them.

- Application code stays Rust-only, with no JavaScript runtime and no web view.
- A local-first library. Network access happens only where the user asked for it
  — a feed refresh, a download, a sync — and never as a background fact of
  running the application.
- Nothing already working may stop working. A user who only ever plays music
  must not be able to tell that Stages 11–14 landed, except that the scan got
  slower in proportion to how much more it is being asked to look at.
- Every stage compiles, keeps the previous stages' acceptance criteria passing,
  and is verified against the running application before it is marked complete.
- No new decoder is added without saying in this file what it is, what it can
  read, and what it costs to build.
- Work whose approach is not yet decided is marked **Unknown**, not Planned. A
  stage is Planned when its approach has been chosen and only the work is left;
  an Unknown stage says so in the status itself rather than pretending the
  research is done.

---

## Stage 11 — Media Library Generalisation

**Goal:** one library, one player, and three kinds of row — so that Stages 12 and
13 have somewhere to put their media rather than each growing their own store.

**Status:** Planned.

This is the stage that decides whether the rest is cheap or expensive. If a film
is a second class of thing with its own scanner, its own tables and its own
transport, then the queue, the artwork cache, the search field, the selection
model and the play counts all have to be built twice more. If instead the
library holds *items* of which music, film and podcast episode are kinds, all of
that comes for free and the later stages are mostly UI.

### Deliverables

- A `MediaKind` for each row: music track, movie, TV episode, podcast episode.
- A schema migration that adds the kind and the fields the new media need
  (season, episode, series, feed URL, episode GUID, resume position), keeping
  the music columns where they are.
- The scanner split into two halves: one that walks folders and classifies what
  it finds, and one per kind that reads a file's metadata. A kind that has no
  reader yet is still indexed with what the file name says, exactly as an
  unreadable music format is counted and named today.
- The player made format-agnostic: a stream is a stream, and the transport does
  not care whether the bytes behind it are audio or a video container's audio
  track.
- One search, one sort, one selection model and one queue across all kinds, with
  the Library's pages gaining the kind filter that the columns already imply.
- The existing music tests still pass, unchanged, with the kind defaulted to
  music — that is the proof the generalisation did not move the ground.

### Acceptance criteria

- A music-only library behaves exactly as it did before the migration, and the
  pre-migration backup the schema upgrade takes is present.
- `MediaKind` is stored on every row and every existing row reads back as music.
- Searching for a term matches across kinds rather than only within the open one.
- The queue can hold a mixed list and play through it without stopping on the
  change of kind.
- Nothing in the UI claims a feature that Stage 12 or 13 has not delivered yet.

---

## Stage 12 — Movies and TV Shows

**Goal:** index, browse and play films and television, locally, with the same
library and the same player the music uses.

**Status:** Planned.

### Deliverables

- Containers: demuxing and playback for the formats a film actually arrives in
  (the `.mp4`/`.m4v` family first, Matroska `.mkv` after, then the legacy
  `.avi`, `.wmv` and `.mov` cases as their decoders allow).
- Video output in the existing window: a picture surface the transport can
  drive, sized to the content's aspect ratio, with the controls hiding when the
  pointer leaves and returning when it comes back.
- A film library view: a poster grid and a list, sorted by title, year, series
  or added date.
- Series grouping: a folder of episodes is one show with seasons, not forty
  films, recognised from the file name — `Show Name/S02/S02E07 …` and the
  conventions beside it — with a manual override for the names that do not fit.
- Resume: where a film or episode was stopped, remembered per item, and offered
  as "Resume" rather than a bare "Play".
- Watched state, cleared one item at a time or in a batch.
- Subtitles and audio-track selection: at minimum, choosing between the streams
  a container carries and reading the sidecar `.srt`/`.ass` files beside it.
- Artwork from the files beside the media, at a size and shape the poster grid
  asks for — reusing the cover-art cache rather than growing a second one.
- An iTunes library as a source, noted and not committed: `Movies` and
  `TV Shows` are plain folders under `iTunes Media` for the scanner as it
  already works, and the iTunes XML import noted in [ROADMAP.md](ROADMAP.md)
  Stage 10 carries movie and TV-show rows, so that import — if built — is a
  candidate source for this file's kinds.

### Acceptance criteria

- A folder of films and a folder of a series are each imported as the right kind
  without renaming anything by hand.
- A film plays to its end, seeks, pauses, changes volume and steps to the next
  item from the same transport the music uses.
- Stopping an episode and reopening the application offers it back where it was
  left, and does not offer it once the episode has finished.
- A container the build cannot demux is counted and named rather than silently
  skipped, matching how an unplayable music format is handled.
- Playback of a video file does not block the interface, and the picture does
  not tear or stutter while the library is being scanned.
- A subtitle file in the wrong encoding is refused with a message rather than
  drawn as mojibake.

---

## Stage 13 — Podcasts and gPodder Synchronization

**Goal:** subscribe to podcast feeds, download and play episodes, keep a resume
position per episode, and synchronize subscriptions and listening state with a
gPodder server — gpodder.net or any self-hosted instance — so the position on a
phone and the position on the desktop agree.

**Status:** Planned.

### Deliverables

- Feed discovery and subscription: a URL or an OPML file in, a feed that has
  been fetched, parsed and validated (RSS 2.0, Atom, and the iTunes podcast
  extensions) out, with the feed's own artwork and description read from it.
- An episode list per feed, with published date, duration, size and download
  state, sorted oldest-first or newest-first.
- Download management: fetch an episode into a directory the user chooses, show
  progress, resume an interrupted download, and delete a downloaded file without
  unsubscribing from the feed.
- Streaming playback of an episode that has not been downloaded, and the choice
  between the local file and the stream when it has.
- Resume position and played state per episode, from the item's own identity
  (feed URL plus episode GUID or media URL), so a re-fetch of the feed does not
  lose them.
- gPodder's Advanced API: device registration and captioning, subscription
  upload and download with `since` timestamps, and episode-action upload and
  retrieval. The endpoint reference is
  [below](#gpodder-api-reference).
- Credentials stored where the rest of the settings are, with the password never
  written to the log and never echoed back into the interface.
- Sync that degrades honestly: a server that is unreachable leaves the local
  state untouched and says so, and a failed sync retries rather than losing the
  local changes it was trying to send.
- OPML import and export, so a subscription list can leave and arrive without
  the server being involved at all.
- And not from iTunes: the XML import noted in [ROADMAP.md](ROADMAP.md)
  Stage 10 lists podcast episodes as flat tracks with no feed or subscription
  state behind them, so podcast arrival stays on the feed-and-OPML path above;
  the Audiobooks rows the same XML carries belong to no stage in either file
  yet.

### Acceptance criteria

- A feed URL subscribes, lists its episodes, downloads one and plays it from
  the local file.
- An episode that has not been downloaded plays as a stream, and says that is
  what it is doing.
- Stopping an episode records a position, and reopening the application offers
  the item back from that position.
- Subscriptions listened to on the desktop appear on a second client after a
  sync, and the reverse — a subscription added elsewhere arrives here.
- A sync against an unreachable server fails with a message, changes nothing
  locally, and can be retried.
- No credential, key or episode URL is written to the log.
- Importing an OPML file subscribes to every feed in it, and exporting one
  produces a file another podcast application can read.

### Recommended order within the stage

1. Feed fetch, parse and subscribe (no server involved).
2. Episode list, download, and local playback with resume.
3. Device registration and subscription sync.
4. Episode-action sync, which is what makes resume positions travel.
5. OPML in and out, and self-hosted instances as a first-class case.

---

## Stage 14 — Apple iPod Synchronization

**Status: Unknown.**

The work here is not "add a sync target". It is "write to a database that is
checksummed, on hardware that is no longer manufactured, through a C library
whose maintenance has been intermittent for years" — and the honest position is
that the approach has not been chosen yet, not that it is merely unstarted. What
follows is the research, so that whoever picks this up starts from the facts
instead of rediscovering them.

### Why this is Unknown rather than Planned

Every other stage here has an approach. Stage 11's is a `MediaKind`; Stage 12's
is a demuxer plus a picture surface; Stage 13's is gPodder's HTTP API. This stage
has three candidate approaches, none of which is obviously right (see below), and
the choice between them decides the whole shape of the work. Marking it Planned
would mean inventing a plan to have one.

### What is actually involved

An iPod does not read the files on it — it reads a database describing them, and
since the iPod classic that database carries a cryptographic hash that the device
verifies. Adding a track means rewriting the database and recomputing that hash,
not just copying a file in.

| Model family                        | What the write needs                                                                    |
| ----------------------------------- | ---------------------------------------------------------------------------------------- |
| iPod 1G–3G, Mini 1G–2G              | A plain `iTunesDB`. No hash, no sqlite.                                                    |
| iPod 4G–5G, Nano 1G–2G              | `iTunesDB` plus a `SysInfoExtended` file                                                  |
| iPod Classic, Nano 3G/4G            | As above, plus a **hash58** computed over the database                                     |
| Nano 5G, iPhone/iPod touch (OS 2.x/3.x) | A compressed `iTunesCDB`, **hash72**, and checksummed sqlite databases plus a `.cbk` file |

The hash is the whole difficulty:

- **hash58** is derived from the device's FireWire ID, which `libgpod` reads off
  the device over SCSI/USB.
- **hash72** is *not* derivable. It needs a `HashInfo` file that only iTunes, or
  a device that has been synced by iTunes at least once, can produce. Without it
  the database is written and then silently rejected — the classic "syncs fine,
  unplug and nothing is there" failure.
- **`SysInfoExtended`** is a plist describing the device's capabilities, produced
  by libgpod's `ipod-read-sysinfo-extended` helper (installed as a HAL/udev
  callout, or run by hand).
- On Nano 5G and later, the sqlite databases must be generated and then have
  post-processing SQL run against them, which on some models is only obtainable
  from the device itself.

None of that is a library problem; it is the device's own design, and it means
supporting "iPod" is really supporting a matrix of generations with different
requirements.

### The three candidate approaches

| Approach                                              | Pros                                                                 | Cons                                                                                                              |
| ----------------------------------------------------- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **`libgpod` bindings** (FFI)                           | The formats, the hashes and the device quirks are all implemented and battle-tested by a decade of use | C dependency; `libgpod`'s own support for the newer models is partial; the maintainership is intermittent; **its licence is LGPL**, which matters for a distributed binary |
| **A pure-Rust implementation of `iTunesDB`/`iTunesCDB`** | No C dependency, and an all-Rust tree, which the project treats as a constraint | Reimplementing a checksummed binary format, and the hash72 problem does not go away — it is a hardware limitation, not a library one. Nothing else in the ecosystem has done this |
| **Do not sync, export**                                 | Honest, cheap, and never writes to the device                          | Not really iPod sync; the device is read-only to us, and the user copies files themselves                          |

A fourth possibility deserves naming and is not recommended: several older tools
sidestep the hash by restoring an older firmware or by exploiting models that do
not verify it. That is not something this project will do.

Flashing Rockbox is a different thing, and it has a stage of its own: a device
whose owner has installed Rockbox reads the filesystem rather than a verified
database, so the whole matrix above stops applying to it. Syncing such a device
is [Stage 15](#stage-15--rockbox-device-synchronization)'s work, and that work
would only ever happen on a device the owner flashed — flashing itself is Rockbox
Utility's job, not this application's.

### Deliverables, once the approach is chosen

Provisional, and to be rewritten when it is:

- Device discovery over USB and mounting, on Linux first, with the other
  platforms named and not pretended.
- A capability probe: read the model and generation, and say plainly which
  requirements that device has and whether they are met.
- `SysInfoExtended` and — where the device allows it — `HashInfo` handling, with
  the "this device needs a HashInfo that we cannot generate; sync it once with
  iTunes, or paste your serial number into ihash.marcan.st" case surfaced to the
  user as what it is, and not as a crash.
- Database read, then write: build the new `iTunesDB`/`iTunesCDB` from the
  library, recompute the hash for the generation in hand, and write it back.
- Track and playlist mapping, including artwork in the size the device asks for.
- A dry run that reports what would change before anything is written, because
  this is the one place in the application that writes to a device rather than a
  file, and the blast radius of getting it wrong is a corrupted library on
  hardware the user cannot easily fix.
- Eject, and a refusal to write while a write is in flight.

### Acceptance criteria, once the approach is chosen

Not written yet, because acceptance criteria for an unknown approach would be
theatre. Two are already fixed regardless of the approach:

- A device whose generation this build cannot support safely is **refused with a
  message naming the generation and the reason**, never written to speculatively.
- Nothing in the interface offers to sync a device until a dry run has read that
  device successfully.

### What has to be decided before this stage can start

1. `libgpod` FFI, pure Rust, or export-only — and if FFI, whether an LGPL
   dependency is acceptable in a distributed binary, and how it is linked.
2. Which generations are in scope. "iPod" as one word hides a matrix; a first
   version that supports the un-hashed classics and refuses the rest honestly is
   a legitimate answer, and a better one than claiming all of them.
3. What happens to a user on a device that needs `HashInfo` and does not have
   one. There is no good answer here, only several bad ones, and the choice
   should be made deliberately rather than discovered in a bug report.

---

## Stage 15 — Rockbox Device Synchronization

**Status: Unknown.**

Stage 14 is Unknown because it does not know how it could be built. This stage is
Unknown because nobody has decided that it will be: the research is finished and
is recorded below — RockSync and iPodRocks have already paid the trial-and-error
tax on real hardware — but scope, shape and the commitment itself have not been
decided, and an uncommitted stage is Unknown, not Planned.

### Why the device is easy and the stage is not

None of Stage 14's difficulty is here. A Rockbox player has no `iTunesDB`, no
hash58 or hash72, no `SysInfoExtended`; it browses the filesystem directly, and
copying files into a known layout is a class of operation this application
already performs. The pieces are already in the tree — lofty reads and writes the
tags Rockbox reads, the artwork cache already holds the covers, and the library
already knows where every file is. What is missing is not capability but the
decisions: where the first version stops, and how much of Rockbox's own data the
application ever touches.

### What a sync is (the research findings)

| Fact                     | Detail                                                                                                                                                  |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Spotting a device        | a mounted USB mass-storage device carrying a `.rockbox/` directory in its root. The firmware version is readable from the `Version:` line of `.rockbox/rockbox-info.txt` **only when that file exists**: it is written by Rockbox Utility (`utils/rbutilqt`), not by the firmware, so a hand-installed device has no version we can read and its capabilities stay unverified |
| What it plays            | the same tag formats lofty already handles; lossy and ≤CD-quality lossless copy verbatim. The references treat **16-bit / 44.1 kHz** as a ceiling, but the firmware ships a software resampler in the codec DSP chain (`lib/rbcodec/dsp/resample.c`), so whether a hi-res file plays depends on the target, its build and CPU headroom — a per-device unknown, not a firmware law. Treat hi-res as unconfirmed until tried on the owner's target |
| The filesystem           | usually FAT32: 2-second mtimes read as UTC (mishandling the clock silently forces a full on-device re-index), FAT-unsafe characters `<>:"/\|?*` replaced in every name component, names compared NFC-lowercased, and a volume token (`/<HDD0>/…`) stripped from device-side paths |
| Cover art                | one `cover.jpg` per album folder (~240–300 px, never upscaled); embedded art optionally stripped from the copied file                                    |
| Playlists                | `.m3u8`, UTF-8, device-relative paths — and this application cannot yet write playlist files for any target, which is why export lands first                |
| Identity                 | USB vendor + product + serial (`/sys/bus/usb/devices`, interface class `08`) pinned per device, so a second player never inherits the first one's sync state |
| Eject                    | Linux: `udisksctl unmount -b <blockdev>` with the block device parsed from `/proc/mounts` (undoing `\040` escapes), falling back to `umount`                 |
| The hazards              | transient write failures are `ETIMEDOUT`, `ECONNRESET`, `EPIPE` and the `EIO`-is-"Transfer failed" class, backed off (the references use 2 s / 5 s / 15 s); an `EPERM` whose size still matched after a failed write actually succeeded (a FAT quirk); a vanished mount directory is an unplug, and the writer must notice |
| Never written            | `.rockbox/` itself. Installing and configuring the firmware is Rockbox Utility's job; this application detects it and never flashes anything                |

### The two references, and the licence boundary

| Reference                                                              | What it contributes                                                                 | Licence | What may cross into this MIT project |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ------- | ------------------------------------- |
| RockSync (© 0Stormy) — a small Python + ffmpeg sync tool, held as `RockSync-main.tar.gz` | the rules: the quality ceiling, the FAT sanitizer, the cover-art policy, the skip list | MIT | portable with attribution — once the upstream repository's licence is verified; its location is unconfirmed and two unrelated GitHub projects share the name |
| [iPodRocks](https://github.com/JoaoSobral/ipodrocks-js) — a 56,000-line Electron sync manager | the war stories: size+mtime compare rules with FAT tolerances, device identity, eject commands, and the full tagcache / runtime-data / tagnavi format detail | GPL-3.0 | **no code** — facts, constants and algorithms reimplemented from scratch. GPL code cannot flow into an MIT project |

Rockbox itself is GPL-2.0 and the authority on its own formats. This is no
longer a claim that rests on the two references: every checkable fact above has
been read back from the firmware source (git.rockbox.org, `v4.0-final-1918`,
commit `685eb7a1af`, held at `~/Projects/rockbox`) — the magic and filenames
(`apps/tagcache.c:115,139-154`), the tag numbering that puts playcount at 15 and
rating at 16, the 96-byte index stride that follows from `TAG_COUNT = 23`,
`<Untagged>`, `TAG_MAXLEN`, the UTF-8 `.m3u8` rule (`apps/playlist.c:319,461`),
`tagnavi_user.config` (`apps/tagtree.c:65-66`) and the 15-second rule
(`apps/tagtree.c:1017`, which also credits 15 s of crossfade to playtime,
line 1057). What remains unverified is hardware behaviour, which no source
substitutes for. (rockbox.org sits behind an Anubis bot-check; the source does
not.)

### Deliberately not in a first version

| Possibility            | What it is | Why it waits |
| ---------------------- | ---------- | ------------ |
| Writing the tagcache   | `.rockbox/database_idx.tcd` plus `database_<N>.tcd` (magic `0x54434810`, version `0x10`, Rockbox 4.0+). The device's "Database → Initialize now" takes minutes after every sync; building the database on the host takes none. But the device cannot repair a database it did not write — one wrong field is "no database at all". | a file-copy sync is correct without it; opt-in at most. The format is source-checked now, and the source offers one more courtesy neither reference mentions: `database_commit.ignore`, whose presence suppresses the temp database's commit |
| Runtime data reading   | with *Gather Runtime Data* enabled by the owner, Rockbox records play counts, play time and ratings inside that database (numeric tags 15–18); a play counts only after 15 seconds | read-only and genuinely useful; presumes a database exists to read |
| Ratings, two-way       | 0–10 ratings, `0` meaning unrated, merged between library and device | the library has no rating field, which is itself an unbuilt Stage 10 addition |
| Tagnavi smart playlists | `.rockbox/tagnavi_user.config` (`#! rockbox/tagbrowser/2.0`) — live on-device queries instead of frozen `.m3u8` snapshots; a reference warns `%include` silently fails on some builds | presumes smart playlists, also Stage 10 |
| `.lrc` lyrics          | Rockbox shows synchronized lyrics beside a track | belongs to the same undecided family as Stage 10's lyrics item |
| Transcoding            | the references shell out to ffmpeg — precisely the C-process dependency the all-Rust rule rules out; a pure-Rust FLAC→FLAC 16/44.1 path is plausible (symphonia + clac), a pure-Rust lossy encoder does not exist | hi-res files are named and skipped in a first version, and whether that is good enough is decision 4 below |

### Deliverables, once the stage is committed

Provisional, which is the difference between Unknown and Planned:

- `.m3u8` export and import for playlists — device-independent value, landing first.
- Device discovery with USB identity, and a manual folder pick as fallback so the
  feature is never hostage to udev.
- A capability probe: model, version where `rockbox-info.txt` allows, free
  space, and the formats the device is not confirmed to play — hi-res among
  them — named and skipped rather than silently omitted.
- A pure sync planner over size and mtime with FAT tolerances (the references use
  128 bytes and 2500 ms) and never a hash; a manifest inside the managed root
  records what was placed there, so a deletion can be honest about what it claims.
- One-way sync of chosen playlists, contained to managed roots (`Music/`,
  `Playlists/`); deletion touches only what the manifest claims.
- A dry run and a free-space check before any write — neither reference has
  either, and the mid-copy `ENOSPC` they accept is the failure this stage beats.
- A device view: per-playlist checkboxes, the report, the progress; and an Eject
  that works or says why it cannot.

### Acceptance criteria, once the stage is committed

Two are fixed ahead of the rest, mirroring Stage 14's two:

- Nothing is written until a dry run has been read and the free-space check has
  passed.
- Everything written, and everything deleted, stays inside managed roots this
  application created.

### What has to be decided before this stage can start

1. **Commitment.** Is this a stage at all? Unknown is an honest answer while the
   decision is deferred; it is not an honest answer forever.
2. **Its home.** ROADMAP.md's Stage 10 lists "Optional device synchronization"
   among the future features, and this file holds the device-sync stages. One of
   the two should own it.
3. **The Stage 14 relationship.** A flashed iPod belongs to this stage and a
   stock-firmware iPod to Stage 14; one device view serving both is possible, and
   so is two.
4. **The format policy.** Is named-and-skipped good enough for v1, or does the
   stage wait for the pure-Rust FLAC path?
5. **The hardware.** Acceptance is against the running application, and the
   running application here means a running player. Which device — and how far a
   directory may honestly stand in for one in tests — decides what "verified" is
   allowed to claim.

---

## gPodder API reference

What Stage 13 is written against. Two protocols live here and they are not the
same thing:

- **The sync protocol** (`/api/2/…`, and the `/subscriptions/…` bulk endpoints)
  is what a podcast application talks to in order to keep subscriptions and
  episode state in step. This is what Stage 13 implements.
- **The administrative API v1** (`/api/v1/…`) manages accounts, sync logins and
  API keys on a goPodder instance. A player has no business calling it, and
  iCraftMusic will not.

gPodder's own reference is the
[gpodder.net API documentation](https://gpoddernet.readthedocs.io/en/latest/api/),
and the goPodder server implementation documents its administrative surface in
[`APIDOCS.md`](https://github.com/cbrgm/gopodder/blob/main/APIDOCS.md).

### General

| Item                | Value                                                                 |
| ------------------- | --------------------------------------------------------------------- |
| Protocol            | HTTPS; an `http` request is redirected to `https`                     |
| Auth                | HTTP authentication (username and password) on every sync endpoint    |
| Data format         | JSON, except the bulk subscription endpoints, which also take OPML and plain text (one URL per line) |
| Date format         | ISO 8601 / RFC 3339 — `YYYY-MM-DDTHH:MM:SSZ`                           |
| Podcast identity    | The feed URL                                                           |
| Episode identity    | The episode's media URL                                                |
| Device ID           | Any string matching `[\w.-]+`, generated by the client and unique per account. A good shape is the application name plus the host name. Two clients sharing an ID can overwrite each other's subscriptions. |

### Endpoints

| Area             | Method and path                                                   | Purpose                                                              |
| ---------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------- |
| Subscriptions    | `GET /subscriptions/{username}/{deviceid}.{format}`               | One device's subscription list in the format asked for                |
| Subscriptions    | `GET /subscriptions/{username}.{format}`                          | Every subscription on the account — the right call on first run       |
| Subscriptions    | `PUT /subscriptions/{username}/{deviceid}.{format}`               | Replace a device's list; the body is OPML, JSON or plain text. The device is created if it does not exist |
| Subscriptions    | `POST /api/2/subscriptions/{username}/{deviceid}.json`            | Upload a **delta** of additions and removals. The server returns the timestamp and any URLs it rewrote |
| Subscriptions    | `GET /api/2/subscriptions/{username}/{deviceid}.json?since=`      | Download the changes since a timestamp, as `add`, `remove` and a new `timestamp` |
| Episode actions  | `POST /api/2/episodes/{username}.json`                            | Upload episode actions — `download`, `play`, `delete`, `new` — with playback positions |
| Episode actions  | `GET /api/2/episodes/{username}.json`                             | Retrieve actions, filtered by `since`, `device` or `podcast`; `aggregated` collapses to the latest per episode |
| Devices          | `POST /api/2/devices/{username}/{deviceid}.json`                  | Set the device's `caption` and `type` (`desktop`, `laptop`, `mobile`, `server`, `other`) |
| Devices          | `GET /api/2/devices/{username}.json`                              | List the account's devices                                            |
| Devices          | `GET /api/2/updates/{username}/{deviceid}.json?since=`            | The "what changed" call: subscriptions to add with their titles and descriptions, subscriptions to remove, and updated episodes with their status |

The `update_urls` list in the upload responses is not decoration: the server
rewrites URLs it considers badly formed, and the client is expected to apply the
rewrite locally so the next upload does not undo the server's correction.

### Episode action types

| Action     | Meaning                                                              |
| ---------- | -------------------------------------------------------------------- |
| `download` | The episode has been downloaded on this device                      |
| `play`     | Playback happened; carries `started`, `position` and `total` in seconds |
| `delete`   | The downloaded file has been deleted                                |
| `new`      | Resets previous actions for the episode; nothing is deleted server-side |
| `flattr`   | Legacy, and not implemented here                                     |

A `play` action is the one that carries a resume position, so it is the one that
matters most for the acceptance criteria above. It requires `started`,
`position` and `total` to be set together; the server rejects a `play` action
missing them.

### Error handling

| Status | Meaning                                        | What the client does                                  |
| ------ | ---------------------------------------------- | ----------------------------------------------------- |
| 400    | Bad request — invalid input or format          | Report the message; do not retry                      |
| 401    | Authentication failed                          | Say so once, stop the sync, keep the local state      |
| 403    | The key or account is not permitted this      | Report it; do not retry                               |
| 404    | Device or resource not found                   | Register the device, then retry once                  |
| 409    | Conflict — the resource exists                 | Treat as already-done and continue                    |
| 500    | Server error                                   | Retry with backoff, keeping the local changes pending |

There is no rate limiting on the sync API, but the documentation asks that
clients be considerate. iCraftMusic syncs when the user asks it to, on a feed
refresh, and when it is about to close if a sync is outstanding — not on a timer.

### What is deliberately out of scope

- The **administrative API v1** (`/api/v1/users`, `/api/v1/accounts`,
  `/api/v1/users/{username}/subscriptions`). It exists so an operator can script
  what the goPodder web UI does — creating sync logins, exporting OPML per user,
  provisioning accounts. A player consuming it would be reaching behind the
  server's own account model, and the keys it uses are the operator's, not the
  listener's.
- **Podcast Lists** and the **Suggestions / Toplist** endpoints. They are a
  directory service, not a sync one, and a first version of Stage 13 does not
  need to browse a catalogue to subscribe to a feed.
- **`flattr`** actions.

---

## Recommended delivery sequence

| Milestone              | Stages | Deliverable                                              |
| ---------------------- | ------ | -------------------------------------------------------- |
| M8 — Generalisation    | 11     | One library and one player holding more than music        |
| M9 — Film and TV       | 12     | Films and series indexed, browsed and played locally      |
| M10 — Podcasts         | 13     | Subscriptions, downloads and playback, with gPodder sync  |
| M11 — iPod (unknown)   | 14     | Apple iPod synchronization, once the approach is decided  |
| M12 — Rockbox (unknown)| 15     | Rockbox device synchronization, once the stage is committed |

Stage 11 must land before either of the others; 12 and 13 do not depend on each
other and can be taken in either order. Stages 14 and 15 depend on nothing and
block nothing — they are off to one side, and neither milestone has a shape
until its open questions are answered.

## Development rules

The rules in [ROADMAP.md](ROADMAP.md) apply here too. Three of them matter
enough to restate for this file:

- Generalise before adding. A second store for a second kind of media is a
  failure of Stage 11, not a shortcut through it.
- Do not implement a future stage prematurely. A podcast row in the library is
  Stage 13's work, even though Stage 11 makes the row possible.
- Mark work complete only when its acceptance criteria have been verified
  against the running application, not when the code compiles.

## Known constraints of this plan

Writing these down now, so that a later stage does not quietly pretend they are
not there:

- **Decoders cost build complexity.** The music player's decoder was cheap
  because `rodio` bundles it. A video demuxer and a picture surface are not in
  that crate, and the choice between a pure-Rust demuxer and a system library is
  a real decision with a licensing dimension. Stage 12 must make it explicitly,
  in this file, before it writes the code.
- **A feed is not a file.** Everything else in the library is a file on disk
  that the scanner can look at. Subscriptions are network state with an update
  schedule, and the scan model has to be extended rather than reused unchanged.
- **A position is per item, not per file.** The music library's play counts
  survive a rescan because they live in the database keyed by path. A resume
  position keyed only by path is lost when a podcast episode is re-downloaded
  or a film is moved; keying by the item's own identity is what Stage 13's
  acceptance criteria are actually testing.
- **Sync can conflict.** Two clients listening to the same episode is the normal
  case, and "last action wins" is only correct because the timestamps make it
  so. Any local merge rule that is not last-write-wins has to be argued for on
  its own merits.
- **A device is not a file, and not a server.** The whole application so far
  writes to files it owns and reads from a database it owns. Stage 14 writes to
  what is effectively someone else's filesystem — one the device verifies and
  will reject silently if it is wrong — and Stage 15 writes to one the device
  merely reads, which is the easier job but still not a filesystem this
  application owns. That is a different class of operation from anything else
  here, and it is why the dry run in either stage is a deliverable rather than a
  nicety.
- **An Unknown stage does not get a milestone date.** The temptation, once a
  stage has a number, is to schedule it. Stages 14 and 15 have numbers so they
  can be referred to; they do not have committed plans, and the two are not the
  same thing.
