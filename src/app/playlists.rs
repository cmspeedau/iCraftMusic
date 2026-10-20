//! Opened playlists, playlist editing and smart-playlist drafts.

use super::{MusicCraftApp, playlist_conflict};
use crate::error::AppError;
use crate::library::{PlaylistId, Track, TrackId};
use crate::playlists::{
    PlaylistDraft, PlaylistEditKind, PlaylistSummary, Playlists, SmartDraft, SmartDraftKind,
    SmartRules,
};

impl MusicCraftApp {
    /// Every playlist the library holds, for the sidebar.
    #[must_use]
    pub fn playlists(&self) -> &[PlaylistSummary] {
        &self.playlists
    }

    /// The playlist the Playlists section has open.
    #[must_use]
    pub fn selected_playlist(&self) -> Option<PlaylistId> {
        self.selected_playlist
    }

    /// The open playlist's tracks, in play order.
    #[must_use]
    pub fn playlist_tracks(&self) -> &[Track] {
        &self.playlist_tracks
    }

    /// The open playlist's name and totals.
    #[must_use]
    pub fn selected_playlist_summary(&self) -> Option<&PlaylistSummary> {
        let open = self.selected_playlist?;

        self.playlists.iter().find(|playlist| playlist.id == open)
    }

    /// Why the last playlist action failed, ready to show and dismiss.
    #[must_use]
    pub fn playlist_error(&self) -> Option<&str> {
        self.playlist_error.as_deref()
    }

    /// Dismiss the last playlist error.
    pub fn dismiss_playlist_error(&mut self) {
        self.playlist_error = None;
    }

    /// Show a playlist in the Playlists section, reading its tracks back.
    ///
    /// Returns whether the playlist was found, so the caller can tell an empty
    /// playlist from one that has just been deleted in another window.
    pub fn open_playlist(&mut self, id: PlaylistId) -> bool {
        let Some(library) = self.library.as_mut() else {
            return false;
        };

        match Playlists::new(library.repository_mut()).playlist(id) {
            Ok(Some(playlist)) => {
                self.selected_playlist = Some(id);
                self.playlist_tracks = playlist.tracks;
                true
            }
            Ok(None) => {
                self.playlist_error = Some("that playlist is no longer in the library".to_owned());
                false
            }
            Err(error) => {
                self.playlist_error = Some(error.to_string());
                false
            }
        }
    }

    /// Close the Playlists section's open playlist.
    pub fn close_playlist(&mut self) {
        self.selected_playlist = None;
        self.playlist_tracks.clear();
    }

    /// The name editor, when a playlist is being created or renamed.
    #[must_use]
    pub fn playlist_draft(&self) -> Option<&PlaylistDraft> {
        self.playlist_draft.as_ref()
    }

    /// The name editor, for the window's text field.
    pub fn playlist_draft_mut(&mut self) -> Option<&mut PlaylistDraft> {
        self.playlist_draft.as_mut()
    }

    /// Start creating a playlist, or renaming the one that is open.
    pub fn start_playlist_edit(&mut self, kind: PlaylistEditKind) {
        let existing = kind
            .target()
            .and_then(|id| self.playlists.iter().find(|playlist| playlist.id == id))
            .cloned();

        self.playlist_draft = Some(PlaylistDraft::new(kind, existing.as_ref()));
        self.playlist_error = None;
    }

    /// Start a new playlist and put `tracks` in it once it has a name.
    ///
    /// This is the Library's "New playlist…" entry: the user picks tracks
    /// first, then names the playlist, which is the order the menu offers them.
    pub fn start_playlist_from_tracks(&mut self, tracks: &[TrackId]) {
        self.playlist_pending = tracks.to_vec();

        let existing = None;
        self.playlist_draft = Some(PlaylistDraft::new(PlaylistEditKind::Create, existing));
        self.playlist_error = None;
    }

    /// Close the name editor without changing anything.
    pub fn cancel_playlist_edit(&mut self) {
        self.playlist_draft = None;
        self.playlist_pending.clear();
    }

    /// Carry out whatever the name editor was opened for.
    ///
    /// A name already in use is the database's refusal rather than the draft's,
    /// so the message it returns is shown beside the field instead of replacing
    /// the window: the user's typed name is still there to correct.
    pub fn confirm_playlist_edit(&mut self) {
        let Some(draft) = self.playlist_draft.clone() else {
            return;
        };

        let name = match draft.validated_name() {
            Ok(name) => name,
            Err(message) => {
                if let Some(draft) = self.playlist_draft.as_mut() {
                    draft.fail(message);
                }
                return;
            }
        };

        let Some(library) = self.library.as_mut() else {
            return;
        };

        // What the edit asks the library to do, worked out before it is done so
        // a failure leaves the draft exactly as the user typed it.
        let outcome = {
            let mut playlists = Playlists::new(library.repository_mut());

            match draft.kind() {
                PlaylistEditKind::Create => playlists.create(&name).map(Some),
                PlaylistEditKind::Rename(id) => playlists.rename(id, &name).map(|()| None),
                PlaylistEditKind::Duplicate(id) => playlists.duplicate(id, &name).map(Some),
            }
        };

        match outcome {
            Ok(created) => {
                self.playlist_draft = None;
                self.playlist_error = None;

                // A new or copied playlist is the one the user wants to look at,
                // so the section moves to it rather than staying where it was.
                let pending = std::mem::take(&mut self.playlist_pending);
                if let (Some(id), false) = (created, pending.is_empty()) {
                    self.add_tracks_to_playlist_id(id, &pending);
                }

                if let Err(error) = self.refresh() {
                    self.playlist_error = Some(error.to_string());
                } else if let Some(id) = created {
                    self.open_playlist(id);
                }
            }
            Err(error) => {
                if let Some(draft) = self.playlist_draft.as_mut() {
                    draft.fail(playlist_conflict(&error, &name));
                }
            }
        }
    }

    /// The rule editor, when a smart playlist is being made or its rules are
    /// being edited.
    #[must_use]
    pub fn smart_draft(&self) -> Option<&SmartDraft> {
        self.smart_draft.as_ref()
    }

    /// The rule editor, for the window's name field and rule rows.
    pub fn smart_draft_mut(&mut self) -> Option<&mut SmartDraft> {
        self.smart_draft.as_mut()
    }

    /// Open the rule editor to make a new smart playlist.
    pub fn start_smart_playlist_create(&mut self) {
        self.smart_draft = Some(SmartDraft::new(
            SmartDraftKind::Create,
            "",
            SmartRules::default(),
        ));
        self.playlist_error = None;
    }

    /// Open the rule editor with a smart playlist's own name and rules, so
    /// what is on screen is what is stored until something changes it.
    ///
    /// Returns whether the editor opened: a playlist deleted in another window
    /// or a hand-made one with no rules to edit says so through the usual
    /// error rather than opening an editor over nothing.
    pub fn start_smart_playlist_edit(&mut self, id: PlaylistId) -> bool {
        let outcome = match self.library.as_mut() {
            Some(library) => {
                let playlists = Playlists::new(library.repository_mut());

                match playlists.rules(id) {
                    Ok(Some(rules)) => match playlists.summary(id) {
                        Ok(Some(summary)) => Ok((summary.name, rules)),
                        Ok(None) => Err("that playlist is no longer in the library".to_owned()),
                        Err(error) => Err(error.to_string()),
                    },
                    Ok(None) => Err("only a smart playlist has rules to edit".to_owned()),
                    Err(error) => Err(error.to_string()),
                }
            }
            None => Err("the library is not open".to_owned()),
        };

        match outcome {
            Ok((name, rules)) => {
                self.smart_draft =
                    Some(SmartDraft::new(SmartDraftKind::EditRules(id), &name, rules));
                self.playlist_error = None;
                true
            }
            Err(message) => {
                self.playlist_error = Some(message);
                false
            }
        }
    }

    /// Close the rule editor without changing anything.
    pub fn cancel_smart_edit(&mut self) {
        self.smart_draft = None;
    }

    /// Carry out whatever the rule editor was opened for: make the smart
    /// playlist, or give the one it names its new rules.
    ///
    /// Saving an edit recomputes the open playlist's tracks through the usual
    /// refresh — there is no membership to update, which is the whole point.
    pub fn confirm_smart_edit(&mut self) {
        let Some(draft) = self.smart_draft.clone() else {
            return;
        };

        let (name, rules) = match draft.validated() {
            Ok(ready) => ready,
            Err(message) => {
                if let Some(draft) = self.smart_draft.as_mut() {
                    draft.fail(message);
                }
                return;
            }
        };

        let Some(library) = self.library.as_mut() else {
            return;
        };

        let outcome = {
            let mut playlists = Playlists::new(library.repository_mut());

            match draft.kind() {
                SmartDraftKind::Create => playlists.create_smart(&name, &rules).map(Some),
                SmartDraftKind::EditRules(id) => playlists
                    .rename(id, &name)
                    .and_then(|()| playlists.set_rules(id, &rules))
                    .map(|()| None),
            }
        };

        match outcome {
            Ok(created) => {
                self.smart_draft = None;
                self.playlist_error = None;

                if let Err(error) = self.refresh() {
                    self.playlist_error = Some(error.to_string());
                } else if let Some(id) = created {
                    self.open_playlist(id);
                }
            }
            Err(error) => {
                if let Some(draft) = self.smart_draft.as_mut() {
                    draft.fail(playlist_conflict(&error, &name));
                }
            }
        }
    }

    /// Add tracks to a playlist, creating it under `name` if it is new.
    ///
    /// This is what the Library's "Add to playlist" menu calls, so the same
    /// entry can put the selection into an existing playlist or into one named
    /// on the spot. A smart playlist of that name refuses the tracks — the menu
    /// normally hides it, so this is the second line of defence — because its
    /// tracks come from its rules, and there is nothing to add them to.
    ///
    /// Returns how many tracks were added, so the caller can say what happened.
    pub fn add_tracks_to_playlist(&mut self, name: &str, tracks: &[TrackId]) -> usize {
        let Some(library) = self.library.as_mut() else {
            return 0;
        };

        let added = {
            let mut playlists = Playlists::new(library.repository_mut());

            match playlists
                .create_or_find(name)
                .and_then(|id| playlists.add_tracks(id, tracks))
            {
                Ok((_, added)) => added,
                Err(AppError::PlaylistIsSmart(_)) => {
                    self.playlist_error = Some(format!(
                        "`{name}` is a smart playlist — its tracks come from its rules"
                    ));
                    0
                }
                Err(error) => {
                    self.playlist_error = Some(error.to_string());
                    0
                }
            }
        };

        if added > 0
            && let Err(error) = self.refresh()
        {
            self.playlist_error = Some(error.to_string());
        }

        added
    }

    /// Add tracks to an existing playlist by identifier.
    ///
    /// The name-taking [`MusicCraftApp::add_tracks_to_playlist`] is what the
    /// menu uses when the playlist may not exist yet; this is the half of it
    /// that runs once the playlist does — including when a drag of tracks
    /// lands on a playlist's row in the sidebar, where the playlist is named
    /// by nothing but its position.
    pub fn add_tracks_to_playlist_id(&mut self, id: PlaylistId, tracks: &[TrackId]) -> usize {
        let Some(library) = self.library.as_mut() else {
            return 0;
        };

        match Playlists::new(library.repository_mut()).add_tracks(id, tracks) {
            Ok((_, added)) => added,
            Err(error) => {
                self.playlist_error = Some(error.to_string());
                0
            }
        }
    }

    /// Swap two entries of the open playlist, keeping the rest in order.
    ///
    /// Returns whether the playlist changed, which is `false` for a move that
    /// goes nowhere or a playlist that is not open.
    pub fn move_in_playlist(&mut self, from: usize, to: usize) -> bool {
        let Some(id) = self.selected_playlist else {
            return false;
        };

        let Some(library) = self.library.as_mut() else {
            return false;
        };

        let moved = match Playlists::new(library.repository_mut()).move_track(id, from, to) {
            Ok(moved) => moved,
            Err(error) => {
                self.playlist_error = Some(error.to_string());
                false
            }
        };

        if moved {
            self.reload_playlist_tracks();
        }

        moved
    }

    /// Take a track out of the open playlist.
    ///
    /// Returns whether a row was removed. Only the first occurrence goes: a
    /// playlist is allowed to hold the same track twice.
    pub fn remove_from_playlist(&mut self, track: TrackId) -> bool {
        let Some(id) = self.selected_playlist else {
            return false;
        };

        let Some(library) = self.library.as_mut() else {
            return false;
        };

        let removed = match Playlists::new(library.repository_mut()).remove_track(id, track) {
            Ok(removed) => removed,
            Err(error) => {
                self.playlist_error = Some(error.to_string());
                false
            }
        };

        if removed {
            // The playlist's totals have changed, so the sidebar's count is
            // reloaded with the tracks rather than left showing the old one.
            if let Err(error) = self.refresh() {
                self.playlist_error = Some(error.to_string());
            }
        }

        removed
    }

    /// Take the open playlist out of the library.
    ///
    /// The deletion is asked for through [`MusicCraftApp::ask_to_delete_playlist`]
    /// rather than called directly, so the files it points at are never touched
    /// without the user having said so.
    pub fn delete_playlist(&mut self, id: PlaylistId) -> bool {
        let Some(library) = self.library.as_mut() else {
            return false;
        };

        let deleted = match Playlists::new(library.repository_mut()).delete(id) {
            Ok(deleted) => deleted,
            Err(error) => {
                self.playlist_error = Some(error.to_string());
                false
            }
        };

        if deleted {
            if self.selected_playlist == Some(id) {
                self.close_playlist();
            }

            if let Err(error) = self.refresh() {
                self.playlist_error = Some(error.to_string());
            }
        }

        deleted
    }

    /// Play the open playlist from the top.
    ///
    /// Returns how many tracks were queued, which is zero for an empty one.
    pub fn play_playlist(&mut self) -> usize {
        let tracks = self.playlist_tracks.clone();

        self.play_tracks(&tracks, 0)
    }

    /// Add the open playlist to the end of the queue.
    pub fn queue_playlist(&mut self) -> usize {
        let tracks = self.playlist_tracks.clone();

        self.queue_tracks(&tracks)
    }

    /// Read the open playlist's tracks back out of the database.
    pub(super) fn reload_playlist_tracks(&mut self) {
        let Some(id) = self.selected_playlist else {
            self.playlist_tracks.clear();
            return;
        };

        let Some(library) = self.library.as_mut() else {
            self.playlist_tracks.clear();
            return;
        };

        match Playlists::new(library.repository_mut()).playlist(id) {
            Ok(Some(playlist)) => self.playlist_tracks = playlist.tracks,
            Ok(None) => {
                self.playlist_tracks.clear();
                self.selected_playlist = None;
            }
            Err(error) => {
                self.playlist_tracks.clear();
                self.playlist_error = Some(error.to_string());
            }
        }
    }

    /// How many playlists the database holds.
    ///
    /// Read straight from the library rather than kept in memory: the
    /// Diagnostics screen is the only caller, and it is not worth caching a
    /// number that is asked for once in a while.
    pub(super) fn playlist_count(&self) -> usize {
        self.library
            .as_ref()
            .and_then(|library| library.repository().playlist_count().ok())
            .unwrap_or(0)
    }
}
