# Roadmap — Media Beyond Music

The second roadmap. [ROADMAP.md](ROADMAP.md) covers the music player, stages 0
to 10; this file carries stages **11 to 13**, which turn MusicCraft from a music
player into a media player: movies, TV shows, and podcasts with gPodder
synchronization.

It is deliberately separate. The music stages are finished and their acceptance
criteria were verified against the running application; the stages below are
unstarted, and keeping them in one file made it look as though the music work
was still open. The two files share one status table in
[README.md](README.md).

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
  must not be able to tell that Stages 11–13 landed, except that the scan got
  slower in proportion to how much more it is being asked to look at.
- Every stage compiles, keeps the previous stages' acceptance criteria passing,
  and is verified against the running application before it is marked complete.
- No new decoder is added without saying in this file what it is, what it can
  read, and what it costs to build.

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

## gPodder API reference

What Stage 13 is written against. Two protocols live here and they are not the
same thing:

- **The sync protocol** (`/api/2/…`, and the `/subscriptions/…` bulk endpoints)
  is what a podcast application talks to in order to keep subscriptions and
  episode state in step. This is what Stage 13 implements.
- **The administrative API v1** (`/api/v1/…`) manages accounts, sync logins and
  API keys on a goPodder instance. A player has no business calling it, and
  MusicCraft will not.

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
clients be considerate. MusicCraft syncs when the user asks it to, on a feed
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

Stage 11 must land before either of the others; 12 and 13 do not depend on each
other and can be taken in either order.

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
