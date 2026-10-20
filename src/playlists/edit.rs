//! The playlist editors' model: what creating, renaming and editing rules
//! start from, and what they refuse.
//!
//! Creating and renaming a playlist are the same edit — the user types a name
//! and the library either gains a playlist or renames one — so both go through
//! [`PlaylistDraft`]. Editing what fills a smart playlist is a wider edit — a
//! name *and* rules — so it has its own [`SmartDraft`]. Everything here is
//! plain data with no database handle, so every rule below (a blank name is
//! refused, the list of names a copy can take, a half-written rule cannot be
//! saved) is testable without a window and without a library.

use crate::library::PlaylistId;
use crate::playlists::PlaylistSummary;
use crate::playlists::smart::SmartRules;

/// What the playlist editor is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistEditKind {
    /// Make a new playlist with the typed name.
    Create,
    /// Give an existing playlist the typed name.
    Rename(PlaylistId),
    /// Copy an existing playlist under the typed name.
    Duplicate(PlaylistId),
}

impl PlaylistEditKind {
    /// The heading shown above the name field.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Create => "New playlist",
            Self::Rename(_) => "Rename playlist",
            Self::Duplicate(_) => "Duplicate playlist",
        }
    }

    /// The label on the button that goes ahead.
    #[must_use]
    pub fn confirm_label(self) -> &'static str {
        match self {
            Self::Create => "Create",
            Self::Rename(_) => "Rename",
            Self::Duplicate(_) => "Duplicate",
        }
    }

    /// The playlist being changed, when one is.
    #[must_use]
    pub fn target(self) -> Option<PlaylistId> {
        match self {
            Self::Create => None,
            Self::Rename(id) | Self::Duplicate(id) => Some(id),
        }
    }
}

/// A playlist name being typed, with the reason the last attempt failed.
///
/// The failure is a message rather than a typed error because it is shown
/// beside the field: the two things that can go wrong — an empty name and one
/// already taken — differ only in the sentence the user reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistDraft {
    kind: PlaylistEditKind,
    name: String,
    error: Option<String>,
}

impl PlaylistDraft {
    /// Start a draft for `kind`, with the existing name filled in where there
    /// is one so a rename begins from what the playlist is called now.
    #[must_use]
    pub fn new(kind: PlaylistEditKind, existing: Option<&PlaylistSummary>) -> Self {
        let name = match kind {
            // A duplicate of "Evening" starts as "Evening copy", which is a
            // name the user can keep as-is rather than one they must think of.
            PlaylistEditKind::Duplicate(_) => existing
                .map(|playlist| format!("{} copy", playlist.display_name()))
                .unwrap_or_default(),
            PlaylistEditKind::Rename(_) => existing
                .map(|playlist| playlist.display_name().to_owned())
                .unwrap_or_default(),
            PlaylistEditKind::Create => String::new(),
        };

        Self {
            kind,
            name,
            error: None,
        }
    }

    /// What the editor is doing.
    #[must_use]
    pub fn kind(&self) -> PlaylistEditKind {
        self.kind
    }

    /// The name as typed.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The name, for the view's text field.
    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    /// Why the last attempt failed, if it did.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Say why the last attempt failed.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.error = Some(reason.into());
    }

    /// Forget the last failure.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// The name to store, with the surrounding whitespace taken off.
    ///
    /// # Errors
    ///
    /// Returns the message for a blank name, which is the one failure the
    /// editor can spot on its own; a name that is already taken is the
    /// database's answer, and reaches the draft through
    /// [`PlaylistDraft::fail`].
    pub fn validated_name(&self) -> Result<String, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("a playlist needs a name".to_owned());
        }

        Ok(name.to_owned())
    }
}

/// What the smart-playlist editor is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartDraftKind {
    /// Make a new smart playlist with the typed name and rules.
    Create,
    /// Replace an existing smart playlist's rules — and its name, since the
    /// editor carries the field and renaming is the same question there too.
    EditRules(PlaylistId),
}

impl SmartDraftKind {
    /// The heading on the window.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Create => "New smart playlist",
            Self::EditRules(_) => "Edit smart playlist",
        }
    }

    /// The label on the button that commits the edit.
    #[must_use]
    pub fn confirm_label(self) -> &'static str {
        match self {
            Self::Create => "Create",
            Self::EditRules(_) => "Save",
        }
    }

    /// The playlist being edited, if any.
    #[must_use]
    pub fn target(self) -> Option<PlaylistId> {
        match self {
            Self::Create => None,
            Self::EditRules(id) => Some(id),
        }
    }
}

/// The smart-playlist editor's model: a name and the rules that would fill it.
///
/// The rules themselves — fields, operators, values, and the SQL they compile
/// to — live in [`crate::playlists::smart`]; the draft adds the two things an
/// editor needs and nothing else: what it is about, and why the last attempt
/// was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartDraft {
    kind: SmartDraftKind,
    name: String,
    rules: SmartRules,
    error: Option<String>,
}

impl SmartDraft {
    /// Start a draft for `kind`. A creation begins with the empty rule set
    /// and no name; an edit begins with the playlist's own name and rules, so
    /// what is on screen is what is stored until something changes it.
    #[must_use]
    pub fn new(kind: SmartDraftKind, name: &str, rules: SmartRules) -> Self {
        Self {
            kind,
            name: name.to_owned(),
            rules,
            error: None,
        }
    }

    /// What the editor is doing.
    #[must_use]
    pub fn kind(&self) -> SmartDraftKind {
        self.kind
    }

    /// The name as typed.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The name, for the view's text field.
    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    /// The rules as edited so far.
    #[must_use]
    pub fn rules(&self) -> &SmartRules {
        &self.rules
    }

    /// The rules, for the view's rule rows.
    pub fn rules_mut(&mut self) -> &mut SmartRules {
        &mut self.rules
    }

    /// Why the last attempt failed, if it did.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Say why the last attempt failed.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.error = Some(reason.into());
    }

    /// The name and rules to store, checked as a whole.
    ///
    /// # Errors
    ///
    /// Returns the message for a blank name or a rule that cannot compile —
    /// the failures the editor can spot on its own. A name that is already
    /// taken is the database's answer, and reaches the draft through
    /// [`SmartDraft::fail`].
    pub fn validated(&self) -> Result<(String, SmartRules), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("a smart playlist needs a name".to_owned());
        }

        if let Some(problem) = self.rules.problem() {
            return Err(problem.to_owned());
        }

        Ok((name.to_owned(), self.rules.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playlists::smart::{Rule, RuleField};
    use std::time::Duration;

    fn summary(name: &str) -> PlaylistSummary {
        PlaylistSummary {
            id: PlaylistId(7),
            name: name.to_owned(),
            track_count: 3,
            duration: Some(Duration::from_secs(180)),
            smart: false,
        }
    }

    #[test]
    fn a_new_playlist_starts_with_an_empty_name() {
        let draft = PlaylistDraft::new(PlaylistEditKind::Create, None);

        assert_eq!(draft.name(), "");
        assert!(draft.error().is_none());
        assert_eq!(draft.kind().target(), None);
    }

    #[test]
    fn a_rename_starts_from_the_name_it_has_now() {
        let draft = PlaylistDraft::new(
            PlaylistEditKind::Rename(PlaylistId(7)),
            Some(&summary("Evening")),
        );

        assert_eq!(draft.name(), "Evening");
        assert_eq!(draft.kind().target(), Some(PlaylistId(7)));
        assert_eq!(draft.kind().title(), "Rename playlist");
    }

    #[test]
    fn a_duplicate_starts_from_a_name_that_is_not_taken() {
        let draft = PlaylistDraft::new(
            PlaylistEditKind::Duplicate(PlaylistId(7)),
            Some(&summary("Evening")),
        );

        assert_eq!(draft.name(), "Evening copy");
        assert_eq!(draft.kind().confirm_label(), "Duplicate");
    }

    #[test]
    fn a_blank_name_is_refused_with_a_sentence() {
        let mut draft = PlaylistDraft::new(PlaylistEditKind::Create, None);

        assert!(draft.validated_name().is_err());

        *draft.name_mut() = "   ".to_owned();
        let message = draft.validated_name().expect_err("a blank name is refused");

        assert!(message.contains("name"), "{message}");
    }

    #[test]
    fn a_name_is_trimmed_before_it_is_stored() {
        let mut draft = PlaylistDraft::new(PlaylistEditKind::Create, None);
        *draft.name_mut() = "  Late Evening  ".to_owned();

        assert_eq!(draft.validated_name().expect("valid"), "Late Evening");
    }

    #[test]
    fn a_failure_can_be_shown_and_taken_back() {
        let mut draft = PlaylistDraft::new(PlaylistEditKind::Create, None);
        assert!(draft.error().is_none());

        draft.fail("`Evening` is already taken");
        assert_eq!(draft.error(), Some("`Evening` is already taken"));

        draft.clear_error();
        assert!(draft.error().is_none());
    }

    #[test]
    fn every_kind_has_a_title_and_a_button() {
        for kind in [
            PlaylistEditKind::Create,
            PlaylistEditKind::Rename(PlaylistId(1)),
            PlaylistEditKind::Duplicate(PlaylistId(1)),
        ] {
            assert!(!kind.title().is_empty(), "{kind:?} has no title");
            assert!(!kind.confirm_label().is_empty(), "{kind:?} has no button");
        }
    }

    #[test]
    fn a_smart_draft_refuses_a_blank_name_and_a_half_written_rule() {
        let mut draft = SmartDraft::new(SmartDraftKind::Create, "", SmartRules::default());
        assert_eq!(
            draft.validated().err().as_deref(),
            Some("a smart playlist needs a name")
        );

        *draft.name_mut() = "Recent".to_owned();
        draft.rules_mut().rules.push(Rule::new(RuleField::Artist));
        assert_eq!(
            draft.validated().err().as_deref(),
            Some("needs some text to match against"),
            "an unfinished rule is refused with the rule's own words"
        );

        draft.rules_mut().rules[0].value = "Squid".to_owned();
        let (name, rules) = draft.validated().expect("complete now");
        assert_eq!(name, "Recent");
        assert_eq!(rules.rules.len(), 1);
    }

    #[test]
    fn a_smart_draft_starts_from_the_rules_the_playlist_already_has() {
        let rules = SmartRules {
            match_all: false,
            rules: vec![Rule::text(RuleField::Genre, "Dub")],
            limit: None,
        };

        let draft = SmartDraft::new(
            SmartDraftKind::EditRules(PlaylistId(7)),
            "Dubwise",
            rules.clone(),
        );

        assert_eq!(draft.name(), "Dubwise");
        assert_eq!(draft.rules(), &rules);
        assert_eq!(draft.kind().target(), Some(PlaylistId(7)));
        assert_eq!(draft.kind().title(), "Edit smart playlist");
        assert_eq!(draft.kind().confirm_label(), "Save");
    }
}
