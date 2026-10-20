//! The shell's side of the cover download: one record at a time, asked for
//! by name, answered on the frame after the worker finishes.
//!
//! The fetching itself lives in [`crate::library::artworkdl`]; this file is
//! where its answer becomes library state — the picture written into every
//! track's cache file, the database told what each track's cover now is, and
//! the album's screen told that the cover it draws is one the archive sent.

use super::{ArtworkNotice, MusicCraftApp};
use crate::library::artworkdl::{ArtworkRequest, ArtworkSession};

impl MusicCraftApp {
    /// Fetch this album's cover from the Cover Art Archive.
    ///
    /// One download runs at a time; asking for a second while one is running
    /// is refused in the open — the button is disabled while any fetch is up,
    /// and this says so rather than queueing silently.
    pub fn download_album_artwork(&mut self, album: &str) {
        if self.artwork_job.is_some() {
            return;
        }

        let artist = self
            .browse()
            .albums()
            .iter()
            .find(|card| card.album.name == album)
            .and_then(|card| card.album.album_artist.clone())
            .or_else(|| {
                self.album_tracks(album)
                    .iter()
                    .find_map(|track| track.album_artist.clone())
            })
            .unwrap_or_else(|| album.to_owned());

        self.artwork_notice = None;
        self.artwork_pending = Some(album.to_owned());
        self.artwork_job = Some(ArtworkSession::spawn(ArtworkRequest {
            artist,
            album: album.to_owned(),
        }));
    }

    /// The album whose cover is being fetched, while one is.
    #[must_use]
    pub fn album_artwork_pending(&self) -> Option<&str> {
        self.artwork_job
            .as_ref()
            .map(|_| self.artwork_pending.as_deref().unwrap_or_default())
    }

    /// Whether any cover download is running, for the buttons that must wait.
    #[must_use]
    pub fn album_artwork_busy(&self) -> bool {
        self.artwork_job.is_some()
    }

    /// What the last finished cover download said.
    #[must_use]
    pub fn artwork_notice(&self) -> Option<&ArtworkNotice> {
        self.artwork_notice.as_ref()
    }

    /// Take the cover answer away, when the view has shown it.
    pub fn dismiss_artwork_notice(&mut self) {
        self.artwork_notice = None;
    }

    /// Collect a finished cover download, once per frame.
    pub(crate) fn poll_album_artwork(&mut self) {
        if !self
            .artwork_job
            .as_ref()
            .is_some_and(ArtworkSession::is_finished)
        {
            return;
        }

        let job = self
            .artwork_job
            .take()
            .expect("a job was checked and found finished");
        let album = self.artwork_pending.take().unwrap_or_default();

        match job.join() {
            Ok(image) => {
                self.store_album_artwork(&album, &image);
                self.refresh().ok();
                self.artwork_notice = Some(ArtworkNotice::Found { album });
            }
            Err(reason) => self.artwork_notice = Some(ArtworkNotice::Missing { album, reason }),
        }
    }

    /// Write a fetched cover where the album's tracks keep their covers.
    ///
    /// Each track of the album gets the picture at its own cache name — the
    /// same shape the tagger's own writing makes, so the views need to know
    /// nothing new — and the database rows follow. Returns how many tracks
    /// ended up with the cover.
    pub(super) fn store_album_artwork(&mut self, album: &str, image: &[u8]) -> usize {
        let directory = self.artwork_directory();
        let mut stored = 0;

        for mut track in self.album_tracks(album) {
            let Ok(path) = crate::library::artwork::store(&directory, &track.path, image) else {
                continue;
            };

            self.artwork.forget(&path);

            track.artwork_path = Some(path);
            stored += 1;

            if let Some(library) = self.library.as_mut()
                && let Some(id) = track.id
                && let Err(error) = library
                    .repository_mut()
                    .set_artwork(id, track.artwork_path.as_deref())
            {
                tracing::warn!(
                    "the cached cover for `{}` could not be remembered: {error}",
                    track.path.display()
                );
            }
        }

        stored
    }
}
