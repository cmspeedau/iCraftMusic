//! The asks the interface cannot answer for itself: one pending confirmation at a time.

use super::MusicCraftApp;
use crate::library::PlaylistId;
use crate::ui::dialog::Confirmation;

impl MusicCraftApp {
    /// Ask the user to confirm removing the selected tracks.
    pub fn ask_to_remove_selection(&mut self) {
        let count = self.browse.selection().len();
        if count > 0 {
            self.pending = Some(Confirmation::RemoveSelection { count });
        }
    }

    /// Ask the user to confirm forgetting every track whose file has vanished.
    pub fn ask_to_forget_missing(&mut self) {
        let count = self.browse.missing_count();
        if count > 0 {
            self.pending = Some(Confirmation::ForgetMissing { count });
        }
    }

    /// Ask the user before taking a playlist out of the library.
    ///
    /// Asked rather than done, for the same reason removing tracks is: the
    /// playlist is something the user built, and it cannot be got back with a
    /// scan the way a track can.
    pub fn ask_to_delete_playlist(&mut self, id: PlaylistId) {
        let Some(summary) = self
            .playlists
            .iter()
            .find(|playlist| playlist.id == id)
            .cloned()
        else {
            return;
        };

        self.pending = Some(Confirmation::DeletePlaylist {
            id,
            name: summary.name.clone(),
            count: summary.track_count,
        });
    }

    /// The destructive action waiting to be confirmed, if there is one.
    #[must_use]
    pub fn pending_confirmation(&self) -> Option<Confirmation> {
        self.pending.clone()
    }

    /// Drop the pending action without doing it.
    pub fn cancel_confirmation(&mut self) {
        self.pending = None;
    }

    /// Carry out the action the user agreed to.
    ///
    /// Both arms remove rows and nothing else: no audio file is deleted by any
    /// path in this application, and both say so before they run.
    pub fn confirm_pending(&mut self) {
        match self.pending.take() {
            Some(Confirmation::RemoveSelection { .. }) => {
                self.remove_selection_from_library();
            }
            Some(Confirmation::ForgetMissing { .. }) => {
                self.forget_missing();
            }
            Some(Confirmation::DeletePlaylist { id, .. }) => {
                self.delete_playlist(id);
            }
            None => {}
        }
    }
}
