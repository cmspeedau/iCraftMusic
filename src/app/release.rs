//! The release check and the update install, as the shell holds them.

use super::{MusicCraftApp, UpdateStatus};
use crate::update::{ReleaseCheck, Update};

impl MusicCraftApp {
    /// Begin looking for a newer release than this build.
    ///
    /// Called once, from the binary, and deliberately not from the
    /// constructors: the release check is the one thing in the application that
    /// touches the network, and a test should not.
    pub fn check_for_release(&mut self) {
        self.release_check = Some(ReleaseCheck::start());
    }

    /// Collect the release check's answer, if it has one.
    ///
    /// Called from the frame loop. The check is dropped once it answers, so a
    /// finished request is not polled on every frame for the rest of the run.
    pub fn poll_release_check(&mut self) {
        let Some(check) = self.release_check.as_mut() else {
            return;
        };

        if let Some(update) = check.poll() {
            self.release_update = Some(update);
            self.release_check = None;
        }
    }

    /// What the release check found, once it has answered.
    #[must_use]
    pub fn release_update(&self) -> Option<&Update> {
        self.release_update.as_ref()
    }

    /// Put a release check's answer in place as though it had arrived.
    ///
    /// For tests: the real answer only ever comes from the network, which tests
    /// do not touch, so the notice and how it is drawn are checked through here.
    #[cfg(test)]
    pub(crate) fn set_release_update(&mut self, update: Update) {
        self.release_update = Some(update);
    }

    /// Take the newest release into this AppImage's place.
    ///
    /// The worker starts only when there is something to take and a file to
    /// put it in: an update is AppImage business, and a build run from a
    /// shell has no image to swap — the gear menu offers that release as a
    /// page to open instead, and never calls this with nowhere to go. Pressing
    /// twice while a download runs changes nothing: one image at a time.
    pub fn start_update(&mut self) {
        if self.update_install.is_some() {
            return;
        }

        self.install_state = None;

        let (
            Some(Update::Available {
                version,
                asset: Some(url),
                ..
            }),
            Some(target),
        ) = (self.release_update.clone(), self.app_image.clone())
        else {
            return;
        };

        self.install_version = Some(version.clone());
        self.install_state = Some(crate::update::Install::Running { done: 0, total: 0 });
        self.update_install = Some(crate::update::InstallSession::spawn(target, url, version));
    }

    /// Ask a running download to stop before its next chunk. The partial file
    /// is deleted and the image the user is running stays exactly as it was.
    pub fn cancel_update(&mut self) {
        if let Some(session) = self.update_install.as_ref() {
            session.cancel();
        }
    }

    /// Where this process's own AppImage is, if it is running from one.
    #[must_use]
    pub fn app_image_path(&self) -> Option<&std::path::Path> {
        self.app_image.as_deref()
    }

    /// The release page of the newest available release — the fallback for a
    /// build that cannot replace its own file.
    #[must_use]
    pub fn release_url(&self) -> Option<&str> {
        match self.release_update.as_ref() {
            Some(Update::Available { url, .. }) => Some(url),
            _ => None,
        }
    }

    /// Start the newly-installed image as a second process.
    ///
    /// Closing this window is the view's doing — it holds the viewport — and
    /// it is what makes the two not fight over the single-instance life of a
    /// player: the new copy is up and loading before this one lets go.
    pub fn relaunch(&self) {
        if let Some(path) = self.app_image.as_ref() {
            let _ = std::process::Command::new(path).spawn();
        }
    }

    /// The release check's answer and the installer's state, said as one
    /// thing for the centre box and the gear menu to draw.
    ///
    /// The installer outranks the check: once a download is running or has
    /// finished, that is the news, whatever the check had found before it.
    /// `None` is the silence neither has anything to show — up to date,
    /// unknown, or nothing asked.
    #[must_use]
    pub fn update_status(&self) -> Option<UpdateStatus> {
        match &self.install_state {
            Some(crate::update::Install::Running { done, total }) => {
                return Some(UpdateStatus::Downloading {
                    version: self.install_version.clone().unwrap_or_default(),
                    done: *done,
                    total: *total,
                });
            }
            Some(crate::update::Install::Installed { version }) => {
                return Some(UpdateStatus::Installed {
                    version: version.clone(),
                });
            }
            Some(crate::update::Install::Failed { reason }) => {
                return Some(UpdateStatus::Failed {
                    reason: reason.clone(),
                });
            }
            None => {}
        }

        if self.release_check.is_some() {
            return Some(UpdateStatus::Checking);
        }

        match self.release_update.as_ref() {
            Some(Update::Available { version, asset, .. }) => Some(UpdateStatus::Available {
                version: version.clone(),
                can_install: asset.is_some() && self.app_image.is_some(),
            }),
            _ => None,
        }
    }

    /// Whether an updater job is running — the frame loop's reason to ask for
    /// another frame, so a reply that arrives mid-run is drawn at once.
    #[must_use]
    pub fn update_job_running(&self) -> bool {
        self.update_install.is_some() || self.release_check.is_some()
    }

    /// Collect what the installer's thread has to say, if it has said it.
    ///
    /// Called from the frame loop. The session is dropped once it has
    /// finished, so a done install is not joined on every frame for the rest
    /// of the run; its last word stays in `install_state`.
    pub fn poll_update_install(&mut self) {
        if let Some(session) = self.update_install.as_mut()
            && let Some(progress) = session.try_progress()
        {
            self.install_state = Some(progress);
        }

        let finished = self
            .update_install
            .as_ref()
            .is_some_and(crate::update::InstallSession::is_finished);

        if finished && let Some(session) = self.update_install.take() {
            let outcome = session.join();
            match &outcome {
                crate::update::Install::Installed { version } => {
                    tracing::info!(%version, "the update is installed");
                }
                crate::update::Install::Failed { reason } => {
                    tracing::warn!(%reason, "the update did not complete");
                }
                crate::update::Install::Running { .. } => {}
            }
            self.install_state = Some(outcome);
        }
    }

    /// Put an installer's state in place as though it had happened.
    ///
    /// For tests, and for the same reason as [`MusicCraftApp::set_release_update`]:
    /// the real states come from downloading and swapping real images.
    #[cfg(test)]
    pub(crate) fn set_install(
        &mut self,
        version: Option<&str>,
        state: Option<crate::update::Install>,
    ) {
        self.install_version = version.map(str::to_owned);
        self.install_state = state;
    }

    /// Make the app believe it runs from `path`, or from no AppImage at all.
    #[cfg(test)]
    pub(crate) fn set_app_image(&mut self, path: Option<std::path::PathBuf>) {
        self.app_image = path;
    }

    /// Make the app believe a release check is pending, without a request.
    #[cfg(test)]
    pub(crate) fn set_check_pending(&mut self) {
        self.release_check = Some(ReleaseCheck::pending_for_tests());
    }
}
