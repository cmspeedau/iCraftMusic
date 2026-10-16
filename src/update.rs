//! Whether a newer iCraftMusic has been published.
//!
//! The player is offline by design, and this is the single exception: one GET
//! of the releases API for the mirror on GitHub, asking for the newest published
//! release's tag. Nothing about the library, the listening history or the
//! machine is sent — the request carries a user-agent that names the build and
//! nothing else — and the reply is read for one string, the version.
//!
//! The request runs on its own thread and hands its answer back through a
//! channel, so a slow or unreachable network never holds up a frame. Until the
//! answer arrives the state is [`Update::Unknown`], and a check that fails is
//! left [`Update::Unknown`] too rather than turned into an error the user has to
//! read: not knowing whether there is an update is not a fault.

use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// The version this build carries, from `Cargo.toml` at compile time.
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// How long the whole request may take, from DNS to the last byte of the reply.
///
/// The thread is not waited on, so this is not needed to keep a frame moving;
/// it is here so a request against a network that never answers ends by itself
/// rather than leaving the check's thread alive for the run.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// The API call that names the newest published release.
///
/// GitHub answers with the repository's latest release as JSON, or 404 while
/// the repository has none; either way the only field read is `tag_name`.
const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/cmspeedau/iCraftMusic/releases/latest";

/// The page a release tag lives on, for the notice to link to.
const RELEASE_PAGE: &str = "https://github.com/cmspeedau/iCraftMusic/releases/tag";

/// What the release check knows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Update {
    /// The check has not answered yet, or it could not be made.
    #[default]
    Unknown,
    /// This build is the newest published release.
    UpToDate,
    /// A newer release than this build has been published.
    Available {
        /// The release's tag, e.g. `v0.2.0`.
        version: String,
        /// The release's page, for the notice to open.
        url: String,
    },
}

/// A release check running on its own thread.
pub struct ReleaseCheck {
    /// The answer, once the thread has one.
    receiver: Receiver<Update>,
}

impl ReleaseCheck {
    /// Start the check.
    ///
    /// The request runs on a worker thread so the frame loop never waits on the
    /// network; the answer is collected by [`ReleaseCheck::poll`].
    #[must_use]
    pub fn start() -> Self {
        let (sender, receiver) = mpsc::channel();

        // A failed spawn leaves `sender` dropped, so `poll` simply never
        // answers — the same as a check that has not finished.
        let _ = std::thread::Builder::new()
            .name("musiccraft-update".to_owned())
            .spawn(move || {
                let _ = sender.send(look_up());
            });

        Self { receiver }
    }

    /// The answer, if the thread has one; `None` until it does.
    ///
    /// Called once per frame, and cheap: it only takes what the thread has
    /// already put in the channel.
    pub fn poll(&mut self) -> Option<Update> {
        self.receiver.try_recv().ok()
    }
}

/// Ask GitHub for the newest published release and compare it with this build.
fn look_up() -> Update {
    let mut response = match ureq::get(LATEST_RELEASE_API)
        .config()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .header("User-Agent", user_agent())
        .header("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(response) => response,
        // A repository with no published release answers 404. There is nothing
        // newer to offer, which is what "up to date" means here rather than a
        // check that could not be made.
        Err(ureq::Error::StatusCode(404)) => return Update::UpToDate,
        Err(_) => return Update::Unknown,
    };

    let Ok(body) = response.body_mut().read_to_string() else {
        return Update::Unknown;
    };

    let Some(tag) = tag_name(&body) else {
        // No tag in the reply: there is no release to compare against, which is
        // what a repository with no published releases looks like.
        return Update::UpToDate;
    };

    if newer(&tag, CURRENT) {
        Update::Available {
            url: format!("{RELEASE_PAGE}/{tag}"),
            version: tag,
        }
    } else {
        Update::UpToDate
    }
}

/// The `tag_name` a release JSON body carries.
///
/// A deliberately small reader rather than a JSON dependency: the application
/// needs one plain string out of one stable field, and a parser for that is a
/// few lines with a test over a real reply, where pulling in a second crate to
/// read it would weigh more than the read.
fn tag_name(body: &str) -> Option<String> {
    let needle = "\"tag_name\"";
    let rest = &body[body.find(needle)? + needle.len()..];
    let rest = &rest[rest.find(':')? + 1..];
    let rest = &rest[rest.find('"')? + 1..];
    let end = rest.find('"')?;

    Some(rest[..end].to_owned())
}

/// Whether `candidate` names a later version than `current`.
///
/// Compared component by component on the numbers a version is made of, with a
/// leading `v` and any pre-release suffix dropped: `v0.2.0` is later than
/// `0.1.0`, and `1.2.0-beta.1` is read as `1.2.0`. This is the check that
/// decides whether to point the user at a release, so a build must never come
/// out newer than itself.
fn newer(candidate: &str, current: &str) -> bool {
    let candidate = components(candidate);
    let current = components(current);

    for index in 0..candidate.len().max(current.len()) {
        let left = candidate.get(index).copied().unwrap_or(0);
        let right = current.get(index).copied().unwrap_or(0);

        if left != right {
            return left > right;
        }
    }

    false
}

/// The numeric components of a version: `v1.2.0-beta.1` is `[1, 2, 0]`.
fn components(version: &str) -> Vec<u64> {
    let core = version.trim_start_matches(['v', 'V']);
    let core = core.split(['-', '+']).next().unwrap_or(core);

    core.split('.')
        .filter_map(|part| part.parse().ok())
        .collect()
}

/// The user-agent every request carries.
///
/// GitHub's API refuses a request without one, and naming the build keeps it
/// honest: nothing but the name and version is sent.
fn user_agent() -> &'static str {
    static AGENT: OnceLock<String> = OnceLock::new();

    AGENT.get_or_init(|| format!("iCraftMusic/{CURRENT}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_newer_release_is_later_than_this_build() {
        assert!(newer("v0.2.0", "0.1.0"));
        assert!(newer("1.0.0", "0.9.9"));
        assert!(newer("0.1.1", "0.1.0"));
        assert!(newer("0.1.0.1", "0.1.0"));
    }

    #[test]
    fn this_build_is_not_newer_than_itself() {
        assert!(!newer(CURRENT, CURRENT));
        assert!(!newer("v0.1.0", "0.1.0"));
        assert!(!newer("0.1.0", "v0.1.0"));
    }

    #[test]
    fn an_older_release_is_not_newer() {
        assert!(!newer("0.0.9", "0.1.0"));
        assert!(!newer("1.2.0", "1.3.0"));
    }

    #[test]
    fn a_pre_release_reads_as_the_version_it_is_built_on() {
        // `1.2.0-beta.1` is not offered as newer than the `1.2.0` it precedes.
        assert!(!newer("1.2.0-beta.1", "1.2.0"));
        assert_eq!(components("1.2.0-beta.1"), vec![1, 2, 0]);
    }

    #[test]
    fn the_tag_is_read_out_of_a_release_reply() {
        // The shape of GitHub's answer, trimmed to the part that matters.
        let body = r#"{
            "url": "https://api.github.com/repos/cmspeedau/iCraftMusic/releases/1",
            "tag_name": "v0.2.0",
            "name": "Stage 9",
            "draft": false
        }"#;

        assert_eq!(tag_name(body).as_deref(), Some("v0.2.0"));
    }

    #[test]
    fn a_reply_with_no_tag_names_no_release() {
        assert_eq!(tag_name(r#"{ "message": "Not Found" }"#), None);
        assert_eq!(tag_name(""), None);
    }

    #[test]
    fn a_check_that_has_not_answered_yet_knows_nothing() {
        assert_eq!(Update::default(), Update::Unknown);
    }
}
