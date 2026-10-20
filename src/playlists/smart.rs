//! The rules that fill a smart playlist.
//!
//! A smart playlist holds no membership rows: it holds a rule set, and its
//! tracks are whatever the library matches right now. That is what makes it
//! *live* — a track scanned ten minutes later appears the moment the library
//! is read again — and it is why nothing here needs the window: every rule is
//! plain data, and a rule set compiles to one SQL query with bound parameters,
//! so the library answers its own question rather than every row being pulled
//! out and filtered in Rust.
//!
//! A rule may only speak about a column the library actually stores — title,
//! artist, album, genre, track number, play count, time, date added — because
//! a rule about a field that does not exist could only ever be a dead control.
//! Ratings and year arrive in the roadmap's iTunes import and ratings stages;
//! when their columns arrive, the rule fields can grow with them.
//!
//! ```
//! # use musiccraft::playlists::{Rule, RuleField, SmartRules};
//! let mut rules = SmartRules::default();
//! rules.rules.push(Rule::text(RuleField::Artist, "Squid"));
//!
//! let (sql, _) = rules.select_query("id FROM tracks");
//! assert!(sql.contains("WHERE"));
//! ```

use serde::{Deserialize, Serialize};

/// The columns a smart playlist may be filled from, in the order the editor
/// lists them.
///
/// Every variant names a column of the `tracks` table; nothing here can refer
/// to a field the library does not keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleField {
    Title,
    Artist,
    Album,
    Genre,
    TrackNumber,
    PlayCount,
    /// Compared in whole seconds, because that is how a listener thinks.
    DurationSeconds,
    DateAdded,
}

/// What a field's values are like, which decides the operators it can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Compared as words.
    Text,
    /// Compared as whole numbers.
    Number,
    /// Date added, which only answers "within the last N days".
    Days,
}

/// How a rule compares a field with its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleOperator {
    Is,
    IsNot,
    Contains,
    BeginsWith,
    GreaterThan,
    LessThan,
    /// Date added only: the value is a number of days.
    WithinLastDays,
}

/// Which field's values a rule compares, so a field's operators are one
/// answer rather than a table per variant.
pub fn field_kind(field: RuleField) -> FieldKind {
    match field {
        RuleField::Title | RuleField::Artist | RuleField::Album | RuleField::Genre => {
            FieldKind::Text
        }
        RuleField::TrackNumber | RuleField::PlayCount | RuleField::DurationSeconds => {
            FieldKind::Number
        }
        RuleField::DateAdded => FieldKind::Days,
    }
}

impl RuleField {
    /// The words the editor shows for the field.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::Album => "Album",
            Self::Genre => "Genre",
            Self::TrackNumber => "Track Number",
            Self::PlayCount => "Play Count",
            Self::DurationSeconds => "Time (seconds)",
            Self::DateAdded => "Date Added",
        }
    }

    /// The column the rule reads. Private to this module's queries: nothing
    /// outside composes SQL from these names.
    fn column(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Artist => "artist",
            Self::Album => "album",
            Self::Genre => "genre",
            Self::TrackNumber => "track_number",
            Self::PlayCount => "play_count",
            Self::DurationSeconds => "duration_ms",
            Self::DateAdded => "added_at",
        }
    }

    /// What the field's values are like.
    #[must_use]
    pub fn kind(self) -> FieldKind {
        field_kind(self)
    }

    /// Every field, for the editor's menu.
    pub const ALL: &'static [Self] = &[
        Self::Title,
        Self::Artist,
        Self::Album,
        Self::Genre,
        Self::TrackNumber,
        Self::PlayCount,
        Self::DurationSeconds,
        Self::DateAdded,
    ];
}

impl RuleOperator {
    /// The words the editor shows, in the middle of a rule's sentence.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Is => "is",
            Self::IsNot => "is not",
            Self::Contains => "contains",
            Self::BeginsWith => "begins with",
            Self::GreaterThan => "is greater than",
            Self::LessThan => "is less than",
            Self::WithinLastDays => "is within the last",
        }
    }

    /// Whether this operator can follow a field, which is what stops the
    /// editor offering "genre begins greater than".
    #[must_use]
    pub fn applies_to(self, field: RuleField) -> bool {
        match field.kind() {
            FieldKind::Text => matches!(
                self,
                Self::Is | Self::IsNot | Self::Contains | Self::BeginsWith
            ),
            FieldKind::Number => matches!(
                self,
                Self::Is | Self::IsNot | Self::GreaterThan | Self::LessThan
            ),
            FieldKind::Days => self == Self::WithinLastDays,
        }
    }

    /// The operator a freshly chosen field starts with.
    #[must_use]
    pub fn default_for(field: RuleField) -> Self {
        match field.kind() {
            FieldKind::Text | FieldKind::Number => Self::Is,
            FieldKind::Days => Self::WithinLastDays,
        }
    }

    /// Every operator that can follow `field`, for the editor's menu.
    #[must_use]
    pub fn for_field(field: RuleField) -> Vec<Self> {
        [
            Self::Is,
            Self::IsNot,
            Self::Contains,
            Self::BeginsWith,
            Self::GreaterThan,
            Self::LessThan,
            Self::WithinLastDays,
        ]
        .into_iter()
        .filter(|operator| operator.applies_to(field))
        .collect()
    }
}

/// One rule: a field, an operator, and the value typed beside them.
///
/// The value stays what the user typed — a string — and is checked when the
/// rule set is saved rather than on every keystroke; a half-typed number is a
/// work in progress, not an error to be shouted about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    /// The column the rule reads.
    pub field: RuleField,
    /// How the column is compared.
    pub operator: RuleOperator,
    /// The value, as typed.
    pub value: String,
}

impl Rule {
    /// A rule about a text field, with the operator that text takes.
    #[must_use]
    pub fn text(field: RuleField, value: impl Into<String>) -> Self {
        Self {
            field,
            operator: RuleOperator::default_for(field),
            value: value.into(),
        }
    }

    /// A rule with a fresh field's defaults and an empty value, which is what
    /// the editor's `+` adds.
    #[must_use]
    pub fn new(field: RuleField) -> Self {
        Self::text(field, String::new())
    }

    /// Why this rule cannot fill a playlist yet, in the words the editor
    /// shows. `None` means it is ready.
    #[must_use]
    pub fn problem(&self) -> Option<&'static str> {
        let value = self.value.trim();

        match self.field.kind() {
            FieldKind::Text if value.is_empty() => Some("needs some text to match against"),
            FieldKind::Number if value.parse::<i64>().is_err() => Some("needs a whole number"),
            FieldKind::Number if value.parse::<i64>().expect("checked") < 0 => {
                Some("cannot be negative")
            }
            FieldKind::Days if value.parse::<i64>().is_err() => Some("needs a number of days"),
            FieldKind::Days if value.parse::<i64>().expect("checked") < 1 => {
                Some("needs at least one day")
            }
            _ => None,
        }
    }

    /// The rule's SQL fragment with its bound parameters, or `None` while the
    /// value cannot be used. The fragments are fixed text chosen from the
    /// field and operator; only values are bound, so typed text never reaches
    /// the database as SQL.
    #[must_use]
    pub(crate) fn compile(&self) -> Option<(String, Vec<rusqlite::types::Value>)> {
        use rusqlite::types::Value as SqlValue;

        let value = self.value.trim();

        // Times are stored in milliseconds but asked about in the whole
        // seconds a listener counts, so that column arrives already reduced to
        // seconds — with unknown times as `-1`, which no usable rule value
        // can equal and every "is not" rule accepts.
        let column = if self.field == RuleField::DurationSeconds {
            "COALESCE(duration_ms / 1000, -1)"
        } else {
            self.field.column()
        };

        let fragment = match (self.field.kind(), self.operator) {
            // An unknown artist or album reads as no text at all, so "is not"
            // is true of it and "is" matches only tracks that say so.
            (FieldKind::Text, RuleOperator::Is) => {
                format!("COALESCE({}, '') = ?", self.field.column())
            }
            (FieldKind::Text, RuleOperator::IsNot) => {
                format!("COALESCE({}, '') <> ?", self.field.column())
            }
            (FieldKind::Text, RuleOperator::Contains) => {
                format!("{} LIKE ? ESCAPE '\\'", self.field.column())
            }
            (FieldKind::Text, RuleOperator::BeginsWith) => {
                format!("{} LIKE ? ESCAPE '\\'", self.field.column())
            }
            // A track without a number or a time has no answer to give, so it
            // matches no comparison — except "is not", which counts unknown
            // as not-this, the way the text rules count no-text as not-that.
            (FieldKind::Number, RuleOperator::Is) => format!("{column} = ?"),
            (FieldKind::Number, RuleOperator::IsNot) => format!("COALESCE({column}, -1) <> ?"),
            (FieldKind::Number, RuleOperator::GreaterThan) => format!("{column} > ?"),
            (FieldKind::Number, RuleOperator::LessThan) => format!("{column} < ?"),
            // Date added is kept as SQLite's own text, so a bound "-N days"
            // is exactly what `datetime('now', ?)` wants.
            (FieldKind::Days, RuleOperator::WithinLastDays) => {
                "added_at >= datetime('now', ?)".to_owned()
            }
            _ => return None,
        };

        let parameter = match self.field.kind() {
            FieldKind::Text => {
                let escaped = like_escape(value);
                let pattern = match self.operator {
                    RuleOperator::Contains => format!("%{escaped}%"),
                    RuleOperator::BeginsWith => format!("{escaped}%"),
                    _ => value.to_owned(),
                };
                SqlValue::Text(pattern)
            }
            FieldKind::Number => SqlValue::Integer(value.parse::<i64>().ok()?),
            FieldKind::Days => SqlValue::Text(format!("-{} days", value.parse::<i64>().ok()?)),
        };

        Some((fragment, vec![parameter]))
    }

    /// Move a rule onto a new field: the operator the new field starts with,
    /// and the typed value only if the new field can still read it — a name
    /// put onto Play Count loses its value because a count cannot be a name,
    /// while words moved between text fields stay exactly as typed.
    pub fn set_field(&mut self, field: RuleField) {
        if self.field == field {
            return;
        }

        self.field = field;
        self.operator = RuleOperator::default_for(field);

        let usable = match field.kind() {
            FieldKind::Text => true,
            FieldKind::Number | FieldKind::Days => self.value.trim().parse::<i64>().is_ok(),
        };

        if !usable {
            self.value.clear();
        }
    }
}

/// Make typed text safe to match with `LIKE`, by escaping the characters
/// `LIKE` itself listens for.
fn like_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for character in text.chars() {
        if matches!(character, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(character);
    }

    escaped
}

/// What a limited smart playlist keeps when more tracks match than it can
/// hold, and what it drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LimitOrder {
    MostPlayed,
    RecentlyAdded,
}

impl LimitOrder {
    /// The words the editor shows beside "selected by".
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::MostPlayed => "most played",
            Self::RecentlyAdded => "recently added",
        }
    }

    /// Every option, for the editor's menu.
    pub const ALL: &'static [Self] = &[Self::MostPlayed, Self::RecentlyAdded];

    /// The order that picks which tracks the limit keeps. The `id` break is
    /// there so two tracks with the same count or the same second added are
    /// kept or dropped the same way every time the playlist is read.
    fn order_by(self) -> &'static str {
        match self {
            Self::MostPlayed => "play_count DESC, id",
            Self::RecentlyAdded => "added_at DESC, id",
        }
    }
}

/// "Limit to N tracks selected by …", which is how a smart playlist stays a
/// size someone can listen to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limit {
    /// How many tracks the playlist holds at most.
    pub count: i64,
    /// Which matches the limit keeps.
    pub order: LimitOrder,
}

impl Limit {
    /// Why the limit cannot work yet, in the words the editor shows.
    #[must_use]
    pub fn problem(&self) -> Option<&'static str> {
        if self.count < 1 {
            Some("limit needs at least one track")
        } else {
            None
        }
    }
}

/// A smart playlist's whole definition: which rules, how they combine, and an
/// optional limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SmartRules {
    /// Whether every rule must match or only one.
    pub match_all: bool,
    /// The rules, in the order the editor lists them.
    pub rules: Vec<Rule>,
    /// "Limit to N", when ticked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<Limit>,
}

impl Default for SmartRules {
    /// No rules and `match all`, which is the definition that fills the new
    /// editor: with nothing written yet every track matches, and the editor
    /// says so plainly.
    fn default() -> Self {
        Self {
            match_all: true,
            rules: Vec::new(),
            limit: None,
        }
    }
}

/// The order a smart playlist shows its tracks in: the library's own sort,
/// which is also how the album view orders its tracks.
const DISPLAY_ORDER: &str = "COALESCE(artist, '') COLLATE NOCASE, \
                             COALESCE(album, '') COLLATE NOCASE, \
                             COALESCE(disc_number, 1), COALESCE(track_number, 0), \
                             title COLLATE NOCASE";

/// The track columns, in the order the row mappers read them — the same list
/// the library repository selects.
pub(crate) const TRACK_COLUMNS: &str = "id, path, title, artist, album, album_artist, \
    genre, track_number, disc_number, duration_ms, artwork_path, size_bytes, modified_at, \
    missing, tags_dirty, play_count";

impl SmartRules {
    /// Whether every rule must match. `false` is "any".
    #[must_use]
    pub fn matches_all(&self) -> bool {
        self.match_all
    }

    /// Why the rule set cannot fill a playlist yet — the first rule's or the
    /// limit's complaint, whichever comes first. `None` means it is ready to
    /// save.
    #[must_use]
    pub fn problem(&self) -> Option<&'static str> {
        for rule in &self.rules {
            if let Some(problem) = rule.problem() {
                return Some(problem);
            }
        }

        self.limit.as_ref().and_then(Limit::problem)
    }

    /// The rules' SQL, joined by `AND` or `OR` and wrapped so `OR` rules bind
    /// the way they read. `None` when a rule's value cannot compile, which
    /// [`SmartRules::problem`] has already explained; no rules at all means
    /// the whole library, so `where_clause` stays `None` and the queries skip
    /// the `WHERE`.
    fn conditions(&self) -> Option<(String, Vec<rusqlite::types::Value>)> {
        let mut fragments = Vec::new();
        let mut values = Vec::new();

        for rule in &self.rules {
            let (fragment, parameters) = rule.compile()?;
            fragments.push(fragment);
            values.extend(parameters);
        }

        if fragments.is_empty() {
            return Some((String::new(), values));
        }

        let joiner = if self.match_all { " AND " } else { " OR " };
        Some((fragments.join(joiner), values))
    }

    /// The full `SELECT` for a set of columns: the where-clause, the limit if
    /// there is one, and the library's own display order. Returns the SQL with
    /// its bound values in placeholder order.
    ///
    /// A limited playlist picks its tracks in the limit's order inside a
    /// sub-select and is re-sorted outside it, which is what keeps "limit 25
    /// by most played" both *the right 25* and *in reading order*.
    #[must_use]
    pub fn select_query(&self, columns: &str) -> (String, Vec<rusqlite::types::Value>) {
        // Rules that cannot compile are not run with the condition left out —
        // that would hand back the whole library. They match nothing, which is
        // what a half-written rule set honestly means until it is saved.
        let Some((conditions, mut values)) = self.conditions() else {
            return (format!("SELECT {columns} FROM tracks WHERE 0"), Vec::new());
        };

        let where_clause = (!conditions.is_empty()).then(|| format!(" WHERE {conditions}"));

        match &self.limit {
            Some(limit) => {
                let sql = format!(
                    "SELECT {columns} FROM (SELECT {columns} FROM tracks{} \
                     ORDER BY {} LIMIT ?) ORDER BY {DISPLAY_ORDER}",
                    where_clause.as_deref().unwrap_or(""),
                    limit.order.order_by(),
                );
                values.push(rusqlite::types::Value::Integer(limit.count));
                (sql, values)
            }
            None => (
                format!(
                    "SELECT {columns} FROM tracks{} ORDER BY {DISPLAY_ORDER}",
                    where_clause.as_deref().unwrap_or(""),
                ),
                values,
            ),
        }
    }

    /// The same query as [`SmartRules::select_query`], counting what it would
    /// return: the sidebar's count and total time come from here, so a smart
    /// playlist's numbers describe its rules rather than membership rows it
    /// never had.
    #[must_use]
    pub fn totals_query(&self) -> (String, Vec<rusqlite::types::Value>) {
        let Some((conditions, mut values)) = self.conditions() else {
            return (
                "SELECT COUNT(*), SUM(duration_ms) FROM tracks WHERE 0".to_owned(),
                Vec::new(),
            );
        };

        let where_clause = (!conditions.is_empty()).then(|| format!(" WHERE {conditions}"));

        match &self.limit {
            Some(limit) => {
                let sql = format!(
                    "SELECT COUNT(*), SUM(duration_ms) FROM \
                     (SELECT duration_ms FROM tracks{} ORDER BY {} LIMIT ?)",
                    where_clause.as_deref().unwrap_or(""),
                    limit.order.order_by(),
                );
                values.push(rusqlite::types::Value::Integer(limit.count));
                (sql, values)
            }
            None => (
                format!(
                    "SELECT COUNT(*), SUM(duration_ms) FROM tracks{}",
                    where_clause.as_deref().unwrap_or(""),
                ),
                values,
            ),
        }
    }

    /// The definition as the text stored in the database.
    ///
    /// # Errors
    ///
    /// Returns [`toml::ser::Error`] — unreachable in practice, as everything
    /// here is plain data.
    pub fn to_definition(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    /// Read a stored definition back.
    ///
    /// # Errors
    ///
    /// Returns [`toml::de::Error`] when the text is not a rule set this build
    /// understands.
    pub fn from_definition(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use crate::database::LibraryRepository;
    use crate::library::Track;

    /// A library holding exactly the tracks a rule needs to be tried against,
    /// with their identifiers.
    fn fixture() -> LibraryRepository {
        let mut repository = LibraryRepository::open_in_memory().expect("in-memory repository");

        let mut tracks = vec![];

        let mut squid = Track::from_path("/music/charuk.mp3");
        squid.title = "Charuk".to_owned();
        squid.artist = Some("Squid".to_owned());
        squid.album = Some("Bright Green Field".to_owned());
        squid.genre = Some("Post-punk".to_owned());
        squid.track_number = Some(1);
        squid.duration = Some(Duration::from_secs_f64(299.6));
        tracks.push(squid);

        let mut fields = Track::from_path("/music/fields.mp3");
        fields.title = "Fields of the Field".to_owned();
        fields.artist = Some("Broadcast".to_owned());
        fields.album = Some("The Noise Made by People".to_owned());
        fields.genre = Some("Pop".to_owned());
        fields.track_number = Some(3);
        fields.duration = Some(Duration::from_secs(301));
        tracks.push(fields);

        let mut unknown = Track::from_path("/music/untagged.wav");
        unknown.title = "Untitled".to_owned();
        unknown.duration = Some(Duration::from_secs(120));
        tracks.push(unknown);

        for track in &tracks {
            repository.upsert_track(track).expect("insert");
        }

        // Play counts are what playing leaves behind, not what a tag says —
        // the repository keeps them off the upsert path — so the fixture
        // plays them in through the same door the player uses.
        for (path, plays) in [("/music/charuk.mp3", 7u64), ("/music/fields.mp3", 2u64)] {
            let stored = repository
                .track_by_path(path)
                .expect("lookup")
                .expect("stored");
            let id = stored.id.expect("identifier");
            for _ in 0..plays {
                repository.record_play(id).expect("play");
            }
        }

        repository
    }

    /// Run a rule set's own query through SQLite and return the titles.
    fn matching(repository: &LibraryRepository, rules: &SmartRules) -> Vec<String> {
        let (sql, values) = rules.select_query(TRACK_COLUMNS);
        let mut statement = repository
            .connection()
            .prepare(&sql)
            .expect("the query runs");
        let rows = statement
            .query_map(rusqlite::params_from_iter(values.iter()), |row| {
                row.get::<_, String>(2)
            })
            .expect("the query answers");

        rows.collect::<rusqlite::Result<Vec<_>>>().expect("rows")
    }

    fn rules(rules: Vec<Rule>) -> SmartRules {
        SmartRules {
            match_all: true,
            rules,
            limit: None,
        }
    }

    #[test]
    fn an_artist_rule_finds_the_tracks_it_names() {
        let mut one = Rule::text(RuleField::Artist, "Squid");
        one.operator = RuleOperator::Is;

        assert_eq!(matching(&fixture(), &rules(vec![one])), vec!["Charuk"]);
    }

    #[test]
    fn contains_finds_the_words_inside_a_title() {
        let mut contains = Rule::text(RuleField::Title, "Field");
        contains.operator = RuleOperator::Contains;

        assert_eq!(
            matching(&fixture(), &rules(vec![contains])),
            vec!["Fields of the Field"]
        );
    }

    #[test]
    fn begins_with_respects_the_start_of_a_word() {
        let mut begins = Rule::text(RuleField::Title, "Fields");
        begins.operator = RuleOperator::BeginsWith;

        assert_eq!(
            matching(&fixture(), &rules(vec![begins])),
            vec!["Fields of the Field"]
        );
    }

    #[test]
    fn typed_wildcards_are_matched_as_themselves() {
        // "100%" as a title is the title, not "anything ending in a percent
        // followed by anything" — the LIKE wildcards have to be escaped.
        let mut wildcard = Rule::text(RuleField::Title, "100%");
        wildcard.operator = RuleOperator::Contains;

        assert!(matching(&fixture(), &rules(vec![wildcard])).is_empty());
    }

    #[test]
    fn is_not_counts_a_track_with_no_artist_as_not_the_one_named() {
        let mut not = Rule::text(RuleField::Artist, "Squid");
        not.operator = RuleOperator::IsNot;

        let found = matching(&fixture(), &rules(vec![not]));
        assert_eq!(found.len(), 2, "Broadcast and the untagged track");
        assert!(
            found.contains(&"Untitled".to_owned()),
            "an unknown artist is not Squid"
        );
    }

    #[test]
    fn play_count_and_time_are_compared_as_numbers() {
        let mut played = Rule::new(RuleField::PlayCount);
        played.operator = RuleOperator::GreaterThan;
        played.value = "3".to_owned();

        assert_eq!(matching(&fixture(), &rules(vec![played])), vec!["Charuk"]);

        let mut shorter = Rule::new(RuleField::DurationSeconds);
        shorter.operator = RuleOperator::LessThan;
        shorter.value = "300".to_owned();

        let found = matching(&fixture(), &rules(vec![shorter]));
        assert_eq!(
            found,
            vec!["Untitled", "Charuk"],
            "299.6 and 120 are under 300, and the track with no artist sorts first"
        );
    }

    #[test]
    fn a_track_without_a_number_answers_no_number_rule() {
        let mut number = Rule::new(RuleField::TrackNumber);
        number.operator = RuleOperator::GreaterThan;
        number.value = "0".to_owned();

        assert_eq!(matching(&fixture(), &rules(vec![number])).len(), 2);
    }

    #[test]
    fn all_and_any_combine_the_rules_they_hold() {
        let punk = Rule::text(RuleField::Genre, "Post-punk");
        let mut loud = Rule::new(RuleField::PlayCount);
        loud.operator = RuleOperator::GreaterThan;
        loud.value = "1".to_owned();

        let both = SmartRules {
            match_all: true,
            rules: vec![punk, loud],
            limit: None,
        };
        assert_eq!(matching(&fixture(), &both), vec!["Charuk"]);

        let either = SmartRules {
            match_all: false,
            ..both.clone()
        };
        let found = matching(&fixture(), &either);
        assert_eq!(
            found.len(),
            2,
            "Charuk matches both rules, Fields matches the play count, the untagged track neither"
        );

        let narrow = SmartRules {
            match_all: true,
            rules: vec![
                Rule::text(RuleField::Genre, "Pop"),
                Rule::text(RuleField::Artist, "Squid"),
            ],
            limit: None,
        };
        assert!(matching(&fixture(), &narrow).is_empty(), "no track is both");
    }

    #[test]
    fn no_rules_at_all_is_the_whole_library() {
        assert_eq!(matching(&fixture(), &SmartRules::default()).len(), 3);
    }

    #[test]
    fn date_added_finds_what_the_scan_just_brought_in() {
        let mut recent = Rule::new(RuleField::DateAdded);
        recent.operator = RuleOperator::WithinLastDays;
        recent.value = "7".to_owned();

        // Everything was indexed moments ago, so a week's window holds it all.
        assert_eq!(matching(&fixture(), &rules(vec![recent])).len(), 3);
    }

    #[test]
    fn a_limit_keeps_the_tracks_its_order_prefers_in_reading_order() {
        let limit = SmartRules {
            match_all: true,
            rules: vec![],
            limit: Some(Limit {
                count: 2,
                order: LimitOrder::MostPlayed,
            }),
        };

        // The two most played are Charuk (7) and Fields (2) — Untitled's zero
        // is dropped — and they come back sorted by artist, not by count.
        assert_eq!(
            matching(&fixture(), &limit),
            vec!["Fields of the Field", "Charuk"],
            "the limit picks by plays, the playlist lists by artist"
        );
    }

    #[test]
    fn totals_count_what_the_query_returns_including_the_limit() {
        let limit = SmartRules {
            match_all: true,
            rules: vec![],
            limit: Some(Limit {
                count: 1,
                order: LimitOrder::MostPlayed,
            }),
        };

        let (sql, values) = limit.totals_query();
        let (count, seconds): (i64, Option<i64>) = fixture()
            .connection()
            .query_row(&sql, rusqlite::params_from_iter(values.iter()), |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("the totals query runs");

        assert_eq!(count, 1, "the limit holds the sidebar to one track");
        assert!(seconds.expect("Charuk has a duration") > 299_000);
    }

    #[test]
    fn an_unusable_value_is_a_problem_not_a_query() {
        let mut number = Rule::new(RuleField::PlayCount);
        number.value = "lots".to_owned();
        assert_eq!(number.problem(), Some("needs a whole number"));
        assert_eq!(number.compile(), None);

        let empty = Rule::new(RuleField::Artist);
        assert_eq!(empty.problem(), Some("needs some text to match against"));

        let combined = rules(vec![Rule::text(RuleField::Artist, "Squid"), empty]);
        assert_eq!(
            combined.problem(),
            Some("needs some text to match against"),
            "the rule set passes the rule's own words on"
        );

        let limited = SmartRules {
            limit: Some(Limit {
                count: 0,
                order: LimitOrder::MostPlayed,
            }),
            ..SmartRules::default()
        };
        assert_eq!(limited.problem(), Some("limit needs at least one track"));
    }

    #[test]
    fn a_definition_survives_the_round_trip_through_storage_text() {
        let original = SmartRules {
            match_all: false,
            rules: vec![
                Rule::text(RuleField::Artist, "Squid"),
                Rule::new(RuleField::DateAdded),
            ],
            limit: Some(Limit {
                count: 25,
                order: LimitOrder::RecentlyAdded,
            }),
        };

        let text = original.to_definition().expect("serialisable");
        let read = SmartRules::from_definition(&text).expect("readable back");

        assert_eq!(read, original);
    }

    #[test]
    fn changing_a_field_moves_the_rule_to_that_fields_kind() {
        let mut rule = Rule::text(RuleField::Artist, "Squid");
        rule.set_field(RuleField::PlayCount);
        assert_eq!(
            rule.operator,
            RuleOperator::Is,
            "a count takes a count's operator"
        );
        assert!(rule.value.is_empty(), "a name is not a count");

        let mut number = Rule::new(RuleField::PlayCount);
        number.value = "12".to_owned();
        number.set_field(RuleField::Title);
        assert_eq!(number.value, "12", "a count reads as words, so it carries");
        assert_eq!(number.operator, RuleOperator::Is);

        number.set_field(RuleField::DateAdded);
        assert_eq!(
            number.value, "12",
            "a count carries as a number of days too, and date added answers only that"
        );
        assert_eq!(number.operator, RuleOperator::WithinLastDays);

        let mut kept = Rule::new(RuleField::Album);
        kept.value = "Side".to_owned();
        kept.set_field(RuleField::Genre);
        assert_eq!(kept.value, "Side", "words carry across text fields");
    }

    #[test]
    fn every_field_has_the_operators_its_kind_supports() {
        for field in RuleField::ALL {
            let operators = RuleOperator::for_field(*field);
            assert!(!operators.is_empty(), "{field:?} has nothing to ask");
            for operator in operators {
                assert!(operator.applies_to(*field));
            }
        }

        assert!(
            RuleOperator::for_field(RuleField::DateAdded) == vec![RuleOperator::WithinLastDays],
            "date added answers exactly one question"
        );
    }

    #[test]
    fn a_compiled_query_binds_its_values_rather_than_writing_them_in() {
        // The words a user typed must never become SQL. The classic attempt:
        // a value that would close a string and add its own clause.
        let clever = rules(vec![Rule::text(
            RuleField::Artist,
            "x'); DROP TABLE tracks; --",
        )]);

        let (sql, values) = clever.select_query(TRACK_COLUMNS);
        assert!(
            !sql.contains("DROP TABLE"),
            "the value never reaches the SQL: {sql}"
        );
        assert_eq!(values.len(), 1, "the value is a bound parameter");

        let repository = fixture();
        matching(&repository, &clever);
        let still_there: i64 = repository
            .connection()
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .expect("the table is still there");
        assert_eq!(still_there, 3);
    }
}
