//! Fetching the cover a record's files do not carry.
//!
//! The sources are the two open halves of MusicBrainz: the release search,
//! which answers a name with an identifier, and the Cover Art Archive, which
//! hangs pictures off that identifier. Neither needs an account, and the only
//! thing either asks for is that a request name the application sending it —
//! which [`user_agent`] does, down to a repository page a human can write to.
//!
//! Nothing here fetches on the library's own initiative. Every request starts
//! at one record with one button pressed, the same bargain the release check
//! makes (see [`crate::update`]): the player is offline until it is asked.
//!
//! The bytes land in the player's own artwork cache, at the file name each
//! track's cover already uses, so a fetched cover is drawn exactly where an
//! extracted one would be — the fetch writes cache files and the library's
//! idea of each track's cover, and touches no audio file.

use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::Value;

use crate::error::{AppError, Result};

/// The MusicBrainz release search, which the archive's pictures are keyed by.
const SEARCH_ENDPOINT: &str = "https://musicbrainz.org/ws/2/release";

/// The Cover Art Archive's front images, addressed by release identifier.
const ARCHIVE_ENDPOINT: &str = "https://coverartarchive.org/release";

/// How long one request may take, DNS to last byte.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// The largest body accepted for a cover, generous by an order of magnitude
/// over any front image either endpoint publishes.
const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;

/// One record, as the user named it and as the library credits it.
#[derive(Clone, Debug)]
pub struct ArtworkRequest {
    /// The album to look up.
    pub album: String,
    /// The artist the album is credited to.
    pub artist: String,
}

/// The user-agent every request carries.
///
/// MusicBrainz asks that automated callers name themselves and say how to be
/// contacted; this build's name and the project's repository page are both
/// true and neither costs anything to keep accurate.
fn user_agent() -> String {
    format!(
        "iCraftMusic/{} (+https://github.com/cmspeedau/iCraftMusic)",
        env!("CARGO_PKG_VERSION")
    )
}

/// The release-search URL asking for this exact album by this artist.
#[must_use]
pub fn search_url(album: &str, artist: &str) -> String {
    let query = format!("release:\"{album}\" AND artist:\"{artist}\"");

    format!(
        "{SEARCH_ENDPOINT}?query={}&fmt=json",
        percent_encode(&query)
    )
}

/// The URL of a release's front image at the archive.
#[must_use]
pub fn front_url(release_id: &str) -> String {
    format!("{ARCHIVE_ENDPOINT}/{release_id}/front-500")
}

/// How a title or credit is compared: trimmed, lower-cased, runs of whitespace
/// collapsed to one space. The archive's data and a file tag's differ in
/// exactly the ways that survive this — casing, padding, wrapped credits —
/// and in none that should be ignored.
fn normalise(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Percent-encodes everything a query value may not carry verbatim.
fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(*byte));
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }

    encoded
}

/// The artist a release result is credited to, joined across its parts.
///
/// `artist-credit` arrives as an array of named parts and, in older replies,
/// as a bare string; both shapes mean the same credit.
fn credit(release: &Value) -> Option<String> {
    match release.get("artist-credit")? {
        Value::String(name) => Some(name.clone()),
        Value::Array(parts) => {
            let joined = parts
                .iter()
                .filter_map(|part| match part {
                    Value::String(name) => Some(name.as_str()),
                    object => object.get("name")?.as_str(),
                })
                .collect::<Vec<_>>()
                .join(" ");

            Some(joined)
        }
        _ => None,
    }
}

/// Which result to believe: the release whose title *is* the album asked for,
/// preferring one credited to the artist asked for and, among those, an
/// officially released copy with a date.
///
/// An exact-title rule, not a best-score rule: the search's own ranking
/// answers fuzzy matches with near-misses, and putting someone else's cover
/// on a record is worse than finding nothing.
#[must_use]
pub fn pick_release(reply: &Value, album: &str, artist: &str) -> Option<String> {
    let wanted_title = normalise(album);
    let wanted_artist = normalise(artist);

    let matches_title = reply
        .get("releases")?
        .as_array()?
        .iter()
        .filter(|release| {
            release
                .get("title")
                .and_then(Value::as_str)
                .is_some_and(|title| normalise(title) == wanted_title)
                && release
                    .get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| !id.is_empty())
        })
        .collect::<Vec<_>>();

    let credited = matches_title
        .iter()
        .filter(|release| {
            credit(release).is_some_and(|names| {
                let names = normalise(&names);
                names.contains(&wanted_artist) || wanted_artist.contains(&names)
            })
        })
        .copied()
        .collect::<Vec<_>>();

    let pool = if credited.is_empty() {
        matches_title
    } else {
        credited
    };

    let dated = pool.iter().find(|release| {
        release.get("status").and_then(Value::as_str) == Some("Official")
            && release.get("date").and_then(Value::as_str).is_some()
    });
    let chosen = dated.or_else(|| {
        pool.iter()
            .find(|release| release.get("date").and_then(Value::as_str).is_some())
    })?;

    Some(chosen.get("id")?.as_str()?.to_owned())
}

/// One request pair, end to end: search, pick, fetch the front.
///
/// `get` is the HTTP the fetch runs on, injectable so the whole walk can be
/// tested without a network.
///
/// # Errors
///
/// [`AppError::CoverArt`] when the search refuses, the reply is not the JSON
/// it promises, no release answers to the record, or the archive's front is
/// not a picture.
pub fn fetch(request: &ArtworkRequest, get: &dyn Fn(&str) -> Result<Vec<u8>>) -> Result<Vec<u8>> {
    let reply = get(&search_url(&request.album, &request.artist))?;
    let Ok(reply) = serde_json::from_slice::<Value>(&reply) else {
        return Err(AppError::CoverArt {
            reason: format!(
                "the release search for {:?} did not answer with JSON",
                request.album
            ),
        });
    };

    let Some(release_id) = pick_release(&reply, &request.album, &request.artist) else {
        return Err(AppError::CoverArt {
            reason: format!(
                "no release named {:?} by {:?} in MusicBrainz",
                request.album, request.artist
            ),
        });
    };

    let image = get(&front_url(&release_id))?;
    if super::artwork::extension_for(&image) == super::artwork::FALLBACK_EXTENSION {
        return Err(AppError::CoverArt {
            reason: "the archive's front image is not a picture this player can read".to_owned(),
        });
    }

    Ok(image)
}

/// The real `get`: one ureq request, named, timed, and bounded.
fn http_get(url: &str) -> Result<Vec<u8>> {
    let mut response = ureq::get(url)
        .config()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .header("User-Agent", &user_agent())
        .header("Accept", "*/*")
        .call()
        .map_err(|error| AppError::CoverArt {
            reason: format!("{url} did not answer: {error}"),
        })?;

    let bytes = response
        .body_mut()
        .read_to_vec()
        .map_err(|error| AppError::CoverArt {
            reason: format!("{url} could not be read: {error}"),
        })?;

    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(AppError::CoverArt {
            reason: format!("{url} offered more bytes than a cover should be"),
        });
    }

    Ok(bytes)
}

/// One cover fetch running on its own thread.
///
/// The same shape as the copy and scan sessions: the window never waits, the
/// answer is there when [`ArtworkSession::is_finished`] says so, and a worker
/// that dies leaves an honest failure rather than a silent nothing.
pub struct ArtworkSession {
    handle: JoinHandle<std::result::Result<Vec<u8>, String>>,
}

impl ArtworkSession {
    /// Queue `request` on a worker thread of its own.
    #[must_use]
    pub fn spawn(request: ArtworkRequest) -> Self {
        let retry = request.clone();
        let handle = thread::Builder::new()
            .name("cover-art".to_owned())
            .spawn(move || fetch(&request, &http_get).map_err(|error| error.to_string()))
            .unwrap_or_else(|_| {
                thread::spawn(move || fetch(&retry, &http_get).map_err(|error| error.to_string()))
            });

        Self { handle }
    }

    /// Whether the answer is ready.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    /// The answer: the picture's bytes, or why there is no picture.
    ///
    /// # Panics
    ///
    /// Never — a panicked worker is reported as a failure.
    pub fn join(self) -> std::result::Result<Vec<u8>, String> {
        self.handle
            .join()
            .unwrap_or_else(|_| Err("the cover worker did not answer".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A search reply with three `Pet Sounds` entries: one wrong artist,
    /// one right artist with a date, and one right artist that is a bootleg.
    fn reply() -> Value {
        serde_json::json!({
            "releases": [
                {"id": "aaa", "title": "Pet Sounds", "artist-credit": [{"name": "The Beatles"}], "status": "Official", "date": "1965"},
                {"id": "bbb", "title": " pet  sounds ", "artist-credit": [{"name": "The"}, {"name": "Beach Boys"}], "status": "Official", "date": "1966-07-20"},
                {"id": "ccc", "title": "Pet Sounds", "artist-credit": "The Beach Boys", "status": "Bootleg"}
            ]
        })
    }

    #[test]
    fn a_search_asks_for_the_exact_title_and_credit() {
        let url = search_url("Pet Sounds", "The Beach Boys");

        assert!(url.starts_with("https://musicbrainz.org/ws/2/release?query="));
        assert!(url.contains("release%3A%22Pet%20Sounds%22"));
        assert!(url.contains("artist%3A%22The%20Beach%20Boys%22"));
        assert!(url.ends_with("&fmt=json"));
    }

    #[test]
    fn the_picked_release_is_the_credited_official_and_dated_one() {
        let picked = pick_release(&reply(), "Pet Sounds", "The Beach Boys");

        assert_eq!(picked.as_deref(), Some("bbb"));
    }

    #[test]
    fn without_a_credited_match_any_exact_title_is_taken() {
        let picked = pick_release(&reply(), "Pet Sounds", "Nobody At All");

        assert_eq!(picked.as_deref(), Some("aaa"));
    }

    #[test]
    fn a_record_no_result_matches_is_nothing_to_take() {
        assert_eq!(
            pick_release(&reply(), "Smiley Smile", "The Beach Boys"),
            None
        );
        assert_eq!(
            pick_release(&serde_json::json!({}), "Pet Sounds", "any"),
            None
        );
        assert_eq!(
            pick_release(
                &serde_json::json!({"releases": [{"id": "", "title": "Pet Sounds"}]}),
                "Pet Sounds",
                "x"
            ),
            None
        );
    }

    #[test]
    fn the_front_image_is_addressed_by_release() {
        assert_eq!(
            front_url("abc-123"),
            "https://coverartarchive.org/release/abc-123/front-500"
        );
    }

    /// The whole walk over a fake network that knows both endpoints.
    #[test]
    fn a_cover_is_searched_picked_and_fetched() {
        let found = fetch(
            &ArtworkRequest {
                album: "Pet Sounds".to_owned(),
                artist: "The Beach Boys".to_owned(),
            },
            &|url| {
                if url.starts_with(SEARCH_ENDPOINT) {
                    Ok(serde_json::to_vec(&reply()).expect("the fixture is JSON"))
                } else {
                    assert!(url.ends_with("bbb/front-500"), "wrong release asked: {url}");
                    Ok(b"\x89PNG a picture".to_vec())
                }
            },
        )
        .expect("the walk finds the picture");

        assert_eq!(found, b"\x89PNG a picture");
    }

    #[test]
    fn a_record_the_archive_has_no_picture_for_fails_honestly() {
        let error = fetch(
            &ArtworkRequest {
                album: "Pet Sounds".to_owned(),
                artist: "The Beach Boys".to_owned(),
            },
            &|url| {
                if url.starts_with(SEARCH_ENDPOINT) {
                    Ok(b"this page is not json".to_vec())
                } else {
                    Ok(Vec::new())
                }
            },
        )
        .expect_err("a reply that is not JSON is not a cover");

        assert!(error.to_string().contains("did not answer with JSON"));
    }

    #[test]
    fn bytes_that_are_not_a_picture_are_not_a_cover() {
        let error = fetch(
            &ArtworkRequest {
                album: "Pet Sounds".to_owned(),
                artist: "The Beach Boys".to_owned(),
            },
            &|url| {
                if url.starts_with(SEARCH_ENDPOINT) {
                    Ok(serde_json::to_vec(&reply()).expect("the fixture is JSON"))
                } else {
                    Ok(b"an html page, not an image".to_vec())
                }
            },
        )
        .expect_err("bytes with no image header are not a cover");

        assert!(error.to_string().contains("not a picture"));
    }

    #[test]
    fn a_network_that_refuses_is_reported_as_said() {
        let error = fetch(
            &ArtworkRequest {
                album: "Any".to_owned(),
                artist: "One".to_owned(),
            },
            &|_| {
                Err(AppError::CoverArt {
                    reason: "the search is offline".to_owned(),
                })
            },
        )
        .expect_err("a refused request is a failure");

        assert!(error.to_string().contains("the search is offline"));
    }

    #[test]
    fn normalising_what_titles_are_compared_on() {
        assert_eq!(normalise("  Pet   SOUNDS \n boys"), "pet sounds boys");
    }

    /// The one test that speaks to the real archive, kept honest and kept
    /// manual: it needs a network, and the network is not something a test
    /// run should depend on. `cargo test -- --ignored` runs it on purpose.
    #[test]
    #[ignore = "sends real requests to MusicBrainz and the Cover Art Archive"]
    fn a_real_record_comes_back_with_a_real_cover() {
        let image = fetch(
            &ArtworkRequest {
                album: "Pet Sounds".to_owned(),
                artist: "The Beach Boys".to_owned(),
            },
            &http_get,
        )
        .expect("the archive has Pet Sounds");

        assert!(
            super::super::artwork::extension_for(&image)
                != super::super::artwork::FALLBACK_EXTENSION,
            "the front of a famous record should be a picture"
        );
    }
}
