//! Whether a newer iCraftMusic has been published, and how it is taken.
//!
//! The player is offline by design, and this and the cover fetch in
//! [`crate::library::artworkdl`] are the only two exceptions — each starts
//! from a named user action and neither fetches on the library's own
//! initiative. The
//! check is one GET of the releases API for the mirror on GitHub, asking for
//! the newest published release's tag. Nothing about the library, the
//! listening history or the machine is sent — the request carries a
//! user-agent that names the build and nothing else — and the reply is read
//! for a version and one download URL.
//!
//! Downloading is a second request, and it is never automatic: it happens
//! when the user presses the update in the gear menu. Only an AppImage run
//! can install one — the runtime says which file it is running from, and the
//! update swaps a new image into that file's place.
//!
//! The work runs on its own thread and hands its answer back through a
//! channel, so a slow or unreachable network never holds up a frame. Until
//! the answer arrives the state is [`Update::Unknown`], and a check that
//! fails is left [`Update::Unknown`] too rather than turned into an error the
//! user has to read: not knowing whether there is an update is not a fault.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, OnceLock};
use std::thread::JoinHandle;
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
/// the repository has none; from the reply only `tag_name` and the assets'
/// download URLs are read.
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
        /// Where to fetch this platform's AppImage, if the release carries
        /// one. Without it the update can only be pointed at, not installed:
        /// the notice says "see the release" rather than "update".
        asset: Option<String>,
    },
}

/// The download-and-swap of a newer AppImage, as the interface sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Install {
    /// Bytes are on their way: `done` of `total`, or `total` `0` until the
    /// server has said how big the file is — the bar slides until then, the
    /// way it does while a scan's walk is still counting.
    Running { done: u64, total: u64 },
    /// The running AppImage has been replaced. The new version is on disk
    /// under the same name; a restart is what runs it.
    Installed { version: String },
    /// Nothing was swapped: the download stopped or the swap failed, and the
    /// running file is exactly what it was.
    Failed { reason: String },
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

    /// For tests: a check that stands pending and never answers. The real
    /// state comes only from a live request, which tests do not make.
    #[cfg(test)]
    pub(crate) fn pending_for_tests() -> Self {
        let (sender, receiver) = mpsc::channel();
        // The sender is dropped, so `poll` never answers and the pending
        // state is the whole of what this one is.
        drop(sender);
        Self { receiver }
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
            asset: asset_url(&body),
            version: tag,
        }
    } else {
        Update::UpToDate
    }
}

/// The download URL of this platform's AppImage among a release's assets.
///
/// Walks the reply's `assets` array — GitHub's asset objects carry the
/// asset's own file name as the URL's last path part, so the URL alone says
/// what it is — and keeps the one whose name ends in this architecture's
/// `.AppImage`. The `.zsync` sibling is deliberately not chosen: it is the
/// delta recipe, and reading it is the `zsync` tool's job, not something to
/// fetch into a file as if it were the image.
fn asset_url(body: &str) -> Option<String> {
    let Ok(reply) = serde_json::from_str::<serde_json::Value>(body) else {
        return None;
    };
    let arch = format!("-{}.AppImage", std::env::consts::ARCH);

    let mut plain = None;

    for asset in reply.get("assets")?.as_array()? {
        let Some(url) = asset
            .get("browser_download_url")
            .and_then(|value| value.as_str())
        else {
            continue;
        };

        let name = url.rsplit('/').next().unwrap_or("");
        if name.ends_with(&arch) {
            return Some(url.to_owned());
        }
        if name.ends_with(".AppImage") && plain.is_none() {
            plain = Some(url.to_owned());
        }
    }

    plain
}

/// The running AppImage's own file, if this build is running from one.
///
/// The AppImage runtime sets `APPIMAGE` to the file it mounted, so the
/// variable being set is both the proof and the path. Started from a shell as
/// a plain binary — every development run — it is unset, and an update has
/// nothing to swap.
#[must_use]
pub fn app_image() -> Option<std::path::PathBuf> {
    std::env::var_os("APPIMAGE").map(std::path::PathBuf::from)
}

/// The `tag_name` a release JSON body carries, and nothing else.
///
/// A body that is not JSON at all — a proxy page, an empty 204 — reads as no
/// tag, which is the same "there is nothing published" that a hand-written
/// scan would have given.
fn tag_name(body: &str) -> Option<String> {
    Some(
        serde_json::from_str::<serde_json::Value>(body)
            .ok()?
            .get("tag_name")?
            .as_str()?
            .to_owned(),
    )
}

/// Whether `candidate` names a later version than `current`.
///
/// Compared with the ordering Cargo itself uses — this crate's `Version` —
/// once a leading `v` is dropped: `v0.2.0` is later than `0.1.0`, and a
/// pre-release sorts before the release it precedes, so `1.2.0-beta.1` is
/// older than `1.2.0` and newer than `1.1.0`. A tag that is not a version at
/// all is never advertised: this check points the user at a download, and it
/// only ever points at one it has actually read.
fn newer(candidate: &str, current: &str) -> bool {
    let (Ok(candidate), Ok(current)) = (parse_version(candidate), parse_version(current)) else {
        return false;
    };

    candidate > current
}

/// A release tag read as a version: `v0.2.0` parses as `0.2.0`.
fn parse_version(tag: &str) -> Result<semver::Version, semver::Error> {
    tag.trim_start_matches(['v', 'V']).parse()
}

/// The user-agent every request carries.
///
/// GitHub's API refuses a request without one, and naming the build keeps it
/// honest: nothing but the name and version is sent.
fn user_agent() -> &'static str {
    static AGENT: OnceLock<String> = OnceLock::new();

    AGENT.get_or_init(|| format!("iCraftMusic/{CURRENT}"))
}

/// How much has to land between two progress reports, in bytes.
///
/// A frame polls the channel cheaply, but there is nothing to draw between
/// quarter-megabytes, and a small buffer over a fast line would mean hundreds
/// of messages for one bar that moves once.
const REPORT_STEP: u64 = 128 * 1024;

/// Download the release's AppImage and swap it in over the running one.
///
/// The new image is written beside the old under its own name, made
/// executable, and only then does the swap begin: the old file is renamed
/// aside, the new one renamed into its place, and the aside file deleted.
/// Every step is a rename or a file that only this function made, so any
/// crash leaves either the old image or the new one at the path — never
/// nothing. The process already running keeps running from the inode the
/// rename unlinked; that is the whole trick, and it is why no restart dance
/// is needed beyond starting the file again.
///
/// Reports through `observer` as bytes land, and stops early — leaving the
/// running file untouched — when `cancel` is set.
fn install_with_cancel(
    target: &Path,
    url: &str,
    version: &str,
    cancel: &AtomicBool,
    observer: &mut dyn FnMut(&Install),
) -> Install {
    let staging = sibling(target, ".new");
    // A previous attempt that died leaves a `.new` and perhaps an `.old`;
    // neither is anything worth keeping, and `staging` cannot be reused
    // while one is in the way.
    let _ = std::fs::remove_file(&staging);
    let _ = std::fs::remove_file(sibling(target, ".old"));

    let outcome = fetch(target, url, &staging, cancel, observer);

    match outcome {
        Ok(()) => match swap_into_place(&staging, target) {
            Ok(()) => Install::Installed {
                version: version.to_owned(),
            },
            Err(error) => Install::Failed {
                reason: format!("could not put {version} in place: {error}"),
            },
        },
        Err(reason) => Install::Failed { reason },
    }
}

/// Fetch `url` into `staging`, reporting progress and watching for cancel.
fn fetch(
    target: &Path,
    url: &str,
    staging: &Path,
    cancel: &AtomicBool,
    observer: &mut dyn FnMut(&Install),
) -> Result<(), String> {
    let mut response = ureq::get(url)
        .config()
        .build()
        .header("User-Agent", user_agent())
        .call()
        .map_err(|error| format!("could not reach the release: {error}"))?;

    // No deadline over the whole exchange: an image is tens of megabytes and
    // a patient line is not a hung one. What ends a stuck download is the
    // cancel, checked between every chunk.
    let total = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    let running = Install::Running { done: 0, total };
    observer(&running);

    let mut file = std::fs::File::create(staging).map_err(|error| {
        let directory = target.parent().unwrap_or(target);
        format!("could not write beside {directory:?}: {error}")
    })?;

    let mut reader = response.body_mut().as_reader();
    let mut buffer = vec![0u8; 64 * 1024];
    let mut done: u64 = 0;
    let mut last_report = 0;

    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = std::fs::remove_file(staging);
            return Err("the update was cancelled".to_owned());
        }

        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                file.write_all(&buffer[..read])
                    .map_err(|error| format!("the download could not be saved: {error}"))?;
                done += read as u64;
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => {
                let _ = std::fs::remove_file(staging);
                return Err(format!("the download stopped: {error}"));
            }
        }

        if done - last_report >= REPORT_STEP {
            let running = Install::Running { done, total };
            observer(&running);
            last_report = done;
        }
    }

    drop(file);

    // An image is only an image if it can be executed; the mode comes across
    // the wire as nothing, so it is set here, before the swap.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(staging, std::fs::Permissions::from_mode(0o755)).map_err(
            |error| format!("the downloaded image could not be made executable: {error}"),
        )?;
    }

    Ok(())
}

/// Move `staging` into `target`'s place, keeping the old file until the new
/// one is in.
///
/// On any failure the old file is put back before the error is returned, so
/// the path the user launched always names a whole image.
fn swap_into_place(staging: &Path, target: &Path) -> std::io::Result<()> {
    let retired = sibling(target, ".old");

    // Retiring the old image is the one move with no rollback to be had; if
    // it cannot be made, nothing has moved and the error simply travels up.
    std::fs::rename(target, &retired)?;

    if let Err(error) = std::fs::rename(staging, target) {
        // Put back exactly what was taken, so a failed update is a failed
        // update and not a vanished application.
        let _ = std::fs::rename(&retired, target);
        return Err(error);
    }

    // Unlinking the old image cannot hurt the process already running from
    // it: its file lives on through the open handle until it exits.
    let _ = std::fs::remove_file(&retired);
    Ok(())
}

/// `path` with `suffix` appended to its file name.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    name.push(suffix);
    path.with_file_name(name)
}

/// A byte count as the bar's wording says it: `1.4 MB`, `980 KB`, `312 B`.
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const MEBIBYTE: u64 = 1024 * 1024;

    if bytes >= MEBIBYTE {
        format!("{:.1} MB", bytes as f64 / MEBIBYTE as f64)
    } else if bytes >= 1024 {
        format!("{} KB", (bytes + 512) / 1024)
    } else {
        format!("{bytes} B")
    }
}

/// An AppImage install running on a worker thread.
///
/// Identical in shape to [`crate::library::importer::CopySession`]: progress
/// arrives on a channel for the interface to draw, and [`InstallSession::join`]
/// returns the outcome once [`InstallSession::is_finished`] has said so.
pub struct InstallSession {
    handle: JoinHandle<Install>,
    progress: Receiver<Install>,
    cancel: Arc<AtomicBool>,
}

impl InstallSession {
    /// Start the worker that installs `url`'s image over `target`.
    #[must_use]
    pub fn spawn(target: PathBuf, url: String, version: String) -> Self {
        let (sender, progress) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);

        let handle = std::thread::Builder::new()
            .name("musiccraft-update-install".to_owned())
            .spawn(move || {
                let mut sender = Some(sender);
                install_with_cancel(&target, &url, &version, &worker_cancel, &mut |install| {
                    forward(&mut sender, install);
                })
            })
            // A failed spawn leaves no worker, and the session is never
            // started: the caller sees the same nothing as before the click.
            .expect("a thread to install the update");

        Self {
            handle,
            progress,
            cancel,
        }
    }

    /// Take the most recent progress snapshot, if the worker has sent one.
    #[must_use]
    pub fn try_progress(&self) -> Option<Install> {
        let mut latest = None;
        while let Ok(install) = self.progress.try_recv() {
            latest = Some(install);
        }
        latest
    }

    /// Ask the worker to stop before the next chunk lands.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Whether the worker has finished, without waiting for it.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    /// Wait for the worker to finish and return what became of the update.
    ///
    /// # Panics
    ///
    /// Panics if the worker thread panicked; that can only happen if a bug
    /// bypasses the error handling inside the install.
    #[must_use]
    pub fn join(self) -> Install {
        match self.handle.join() {
            Ok(outcome) => outcome,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}

/// Forward a progress snapshot, ignoring a receiver that has gone away.
fn forward(sender: &mut Option<Sender<Install>>, install: &Install) {
    if let Some(sender) = sender {
        let _ = sender.send(install.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_newer_release_is_later_than_this_build() {
        assert!(newer("v0.2.0", "0.1.0"));
        assert!(newer("1.0.0", "0.9.9"));
        assert!(newer("0.1.1", "0.1.0"));
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
    fn a_pre_release_sorts_where_semver_puts_it() {
        // Not offered as newer than the `1.2.0` it precedes, but newer than
        // the `1.1.0` it is a pre-release after; and the release, once out,
        // is newer than its own pre-release.
        assert!(!newer("1.2.0-beta.1", "1.2.0"));
        assert!(newer("1.2.0-beta.1", "1.1.0"));
        assert!(newer("1.2.0", "1.2.0-beta.1"));
    }

    #[test]
    fn a_tag_that_is_not_a_version_is_never_advertised() {
        assert!(!newer("nightly", "0.1.0"));
        assert!(!newer("0.2.0", "not-a-version"));
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

    /// The release's assets, as one real reply carries them: the image, its
    /// delta recipe, and the checksums file. Only the image is something to
    /// download into a file and run — and when two architectures are
    /// published, this machine's is the one taken.
    #[test]
    fn this_machines_appimage_is_read_out_of_a_release_reply() {
        let arch = std::env::consts::ARCH;
        let body = format!(
            r#"{{
            "tag_name": "v0.2.0",
            "assets": [
                {{
                    "name": "iCraftMusic-0.2.0-aarch64.AppImage",
                    "browser_download_url": "https://github.com/cmspeedau/iCraftMusic/releases/download/v0.2.0/iCraftMusic-0.2.0-aarch64.AppImage"
                }},
                {{
                    "name": "iCraftMusic-0.2.0-{arch}.AppImage.zsync",
                    "browser_download_url": "https://github.com/cmspeedau/iCraftMusic/releases/download/v0.2.0/iCraftMusic-0.2.0-{arch}.AppImage.zsync"
                }},
                {{
                    "name": "iCraftMusic-0.2.0-{arch}.AppImage",
                    "browser_download_url": "https://github.com/cmspeedau/iCraftMusic/releases/download/v0.2.0/iCraftMusic-0.2.0-{arch}.AppImage"
                }}
            ]
        }}"#
        );

        let url = asset_url(&body).expect("the image is among the assets");
        assert!(url.ends_with(&format!("-{arch}.AppImage")));
        assert!(
            !url.ends_with(".zsync"),
            "the delta recipe was taken for the image"
        );
    }

    /// A release that names no architecture publishes one image for every
    /// machine; it is still the one to take.
    #[test]
    fn an_appimage_without_an_architecture_in_its_name_is_taken_as_fallback() {
        let body = r#"{
            "assets": [
                {"name": "notes.txt", "browser_download_url": "https://e/notes.txt"},
                {"name": "iCraftMusic.AppImage", "browser_download_url": "https://e/iCraftMusic.AppImage"}
            ]
        }"#;

        assert_eq!(
            asset_url(body).as_deref(),
            Some("https://e/iCraftMusic.AppImage")
        );
    }

    #[test]
    fn a_release_with_nothing_downloadable_offers_no_image() {
        assert_eq!(asset_url(r#"{ "tag_name": "v0.2.0" }"#), None);
        assert_eq!(asset_url(""), None);
    }

    #[test]
    fn the_swap_puts_the_new_image_in_the_old_ones_place() {
        let directory = scratch("swap-ok");
        let target = directory.join("iCraftMusic.AppImage");
        let staging = directory.join("iCraftMusic.AppImage.new");
        std::fs::write(&target, b"the running image").expect("write target");
        std::fs::write(&staging, b"the new image").expect("write staging");

        swap_into_place(&staging, &target).expect("the swap of two files is plain work");

        assert_eq!(
            std::fs::read(&target).expect("the path names a file"),
            b"the new image",
            "the new image is not in the old one's place"
        );
        assert!(!staging.exists(), "the staging file was left behind");
        assert!(
            !directory.join("iCraftMusic.AppImage.old").exists(),
            "the retired image was left lying around"
        );

        let _ = std::fs::remove_dir_all(&directory);
    }

    /// A swap that cannot finish puts the running image back: an update that
    /// fails must fail with the application still on the path the user
    /// launched, not with a hole where it was.
    #[test]
    fn a_swap_that_cannot_finish_leaves_the_running_image_in_place() {
        let directory = scratch("swap-fails");
        let target = directory.join("iCraftMusic.AppImage");
        std::fs::write(&target, b"the running image").expect("write target");

        let missing = directory.join("iCraftMusic.AppImage.new");
        assert!(swap_into_place(&missing, &target).is_err());

        assert_eq!(
            std::fs::read(&target).expect("the path names a file"),
            b"the running image",
            "the failed swap left the path empty"
        );

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn byte_counts_read_as_the_bar_says_them() {
        assert_eq!(format_bytes(312), "312 B");
        assert_eq!(format_bytes(2 * 1024), "2 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1_572_864), "1.5 MB");
    }

    /// Scratch for the swap tests, in the house's shared temporary place.
    fn scratch(tag: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("musiccraft-update-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("a scratch directory");
        directory
    }
}
