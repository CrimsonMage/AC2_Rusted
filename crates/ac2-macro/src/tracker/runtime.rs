//! The Tracker's state: the compiled patterns, the objects already seen (so each is announced
//! once), the matches, and the object the arrow points at. Every change returns the effects for
//! the client to carry out.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use fancy_regex::Regex;

use super::codec::{decode_patterns, encode_patterns};
use super::persistence::{initial_plan, PatternRead};
use super::presentation::{
    coordinates_text, project, PositionQuery, Presentation, PresentationState, TrackerPosition,
};

/// One object the client knows of: its instance id, name, whether its container is known (and
/// which), and its cell. Objects inside a container (a pack, a corpse) are not tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservedObject<'a> {
    pub object: u64,
    pub name: Option<&'a str>,
    pub containment_known: bool,
    pub container: Option<u64>,
    pub cell: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternView {
    pub expression: Arc<str>,
    pub exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchView {
    pub object: u64,
    pub name: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackerView {
    pub pattern_revision: u64,
    pub match_revision: u64,
    pub patterns: Arc<[PatternView]>,
    pub matches: Arc<[MatchView]>,
}

/// What the client does for the Tracker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Write the pattern list to the Tracker's file.
    SavePatterns(Arc<[u8]>),
    /// Announce a match in chat: its name (a link that selects it) and, outdoors, its map
    /// coordinates (a link that points the arrow there).
    Detected {
        object: u64,
        name: Arc<str>,
        at: Option<Arc<str>>,
    },
    /// A line of Tracker news in chat (a reload, a bad pattern, a file error).
    Notice(Arc<str>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackerUpdate {
    pub changed: bool,
    pub effects: Vec<Effect>,
}

impl TrackerUpdate {
    fn merge(&mut self, mut other: Self) {
        self.changed |= other.changed;
        self.effects.append(&mut other.effects);
    }

    fn notice(text: String) -> Self {
        Self {
            changed: false,
            effects: vec![Effect::Notice(Arc::from(text))],
        }
    }
}

#[derive(Debug)]
struct CompiledPattern {
    expression: Arc<str>,
    regex: Regex,
}

#[derive(Debug)]
pub struct Tracker {
    initialized: bool,
    patterns: Vec<CompiledPattern>,
    found: Vec<MatchView>,
    /// Objects already looked at, so a match is announced once per object.
    seen: BTreeSet<u64>,
    last_catalog_revision: Option<u64>,
    pattern_revision: u64,
    match_revision: u64,
    pointer_revision: u64,
    pointer_target: Option<u64>,
    pattern_view: Arc<[PatternView]>,
    match_view: Arc<[MatchView]>,
    presentation: PresentationState,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            initialized: false,
            patterns: Vec::new(),
            found: Vec::new(),
            seen: BTreeSet::new(),
            last_catalog_revision: None,
            pattern_revision: 0,
            match_revision: 0,
            pointer_revision: 0,
            pointer_target: None,
            pattern_view: Arc::from([]),
            match_view: Arc::from([]),
            presentation: PresentationState::default(),
        }
    }
}

impl Tracker {
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn view(&self) -> TrackerView {
        TrackerView {
            pattern_revision: self.pattern_revision,
            match_revision: self.match_revision,
            patterns: Arc::clone(&self.pattern_view),
            matches: Arc::clone(&self.match_view),
        }
    }

    /// The object the arrow points at, if a match was picked.
    pub const fn pointer_target(&self) -> Option<u64> {
        self.pointer_target
    }

    /// The first load: the Tracker's own list, else an older one (copied over). The objects
    /// already around are matched and announced now; after that only newcomers are.
    pub fn load_initial<'a>(
        &mut self,
        canonical: PatternRead,
        legacy: PatternRead,
        catalog_revision: u64,
        objects: impl IntoIterator<Item = ObservedObject<'a>>,
    ) -> TrackerUpdate {
        let plan = initial_plan(canonical, legacy);
        let mut update = self.reload(plan.bytes.as_deref(), catalog_revision, objects);
        if let Some(bytes) = plan.migrate {
            update.effects.push(Effect::SavePatterns(bytes));
        }
        if let Some(warning) = plan.warning {
            update.effects.push(Effect::Notice(warning));
        }
        update
    }

    /// Reload from a file the player named: every object is matched again.
    pub fn reload_source<'a>(
        &mut self,
        source: PatternRead,
        source_label: &str,
        catalog_revision: u64,
        objects: impl IntoIterator<Item = ObservedObject<'a>>,
    ) -> TrackerUpdate {
        let bytes = match source {
            PatternRead::Missing => None,
            PatternRead::Bytes(bytes) => Some(bytes),
            PatternRead::Failed(error) => {
                return TrackerUpdate::notice(format!(
                    "[Tracker] Could not reload the pattern list: {error}"
                ));
            }
        };
        let mut update = self.reload(bytes.as_deref(), catalog_revision, objects);
        update.effects.insert(
            0,
            Effect::Notice(Arc::from(format!(
                "[Tracker] Reloading the pattern list from {source_label}"
            ))),
        );
        update
    }

    /// The client could not write the pattern file.
    pub fn save_failed(&self, error: &str) -> TrackerUpdate {
        TrackerUpdate::notice(format!(
            "[Tracker] Could not save the pattern list: {error}"
        ))
    }

    /// The list's rows with distances and coordinates, re-sampled at most four times a second;
    /// `None` when nothing the player would see changed.
    pub fn project(
        &mut self,
        now: Duration,
        player: Option<TrackerPosition>,
        positions: &impl PositionQuery,
    ) -> Option<Presentation> {
        project(
            &mut self.presentation,
            now,
            (
                self.pattern_revision,
                self.match_revision,
                self.pointer_revision,
            ),
            self.pointer_target,
            &self.pattern_view,
            &self.match_view,
            player,
            positions,
        )
    }

    /// The player selected something else: an arrow the Tracker set lets go. Selecting never
    /// sets the arrow; only picking a match does.
    pub fn observe_selection(&mut self, selected: Option<u64>) -> bool {
        if self.pointer_target.is_none() || self.pointer_target == selected {
            return false;
        }
        self.set_pointer(None);
        true
    }

    fn reload<'a>(
        &mut self,
        bytes: Option<&[u8]>,
        catalog_revision: u64,
        objects: impl IntoIterator<Item = ObservedObject<'a>>,
    ) -> TrackerUpdate {
        self.initialized = true;
        let objects: Vec<_> = objects.into_iter().collect();
        self.last_catalog_revision = Some(catalog_revision);
        self.seen = objects.iter().map(|o| o.object).collect();
        self.replace_patterns(bytes.unwrap_or_default());
        self.clear_matches();
        let mut update = TrackerUpdate {
            changed: true,
            effects: Vec::new(),
        };
        for index in 0..self.patterns.len() {
            update.merge(self.scan_with_pattern(index, &objects));
        }
        update
    }

    /// The objects the client knows now (`catalog_revision` changes when they do): gone ones
    /// leave the list, newcomers are matched once.
    pub fn reconcile<'a>(
        &mut self,
        catalog_revision: u64,
        objects: impl IntoIterator<Item = ObservedObject<'a>>,
    ) -> TrackerUpdate {
        if self.last_catalog_revision == Some(catalog_revision) {
            return TrackerUpdate::default();
        }
        self.last_catalog_revision = Some(catalog_revision);
        let objects: Vec<_> = objects.into_iter().collect();
        let live: BTreeSet<u64> = objects.iter().map(|o| o.object).collect();
        let mut update = TrackerUpdate::default();
        if self.pointer_target.is_some_and(|t| !live.contains(&t)) {
            self.set_pointer(None);
            update.changed = true;
        }
        self.seen.retain(|o| live.contains(o));
        let before = self.found.len();
        self.found.retain(|found| {
            objects
                .iter()
                .any(|o| o.object == found.object && o.containment_known && o.container.is_none())
        });
        if self.found.len() != before {
            self.rebuild_match_view();
            update.changed = true;
        }
        for object in objects {
            if self.seen.contains(&object.object) || !object.containment_known {
                continue;
            }
            let Some(name) = object.name else {
                continue;
            };
            self.seen.insert(object.object);
            if object.container.is_none() && self.matches_any(name) {
                update.merge(self.record_match(object));
            }
        }
        update
    }

    /// Adds a pattern (`exact` anchors it to the whole name) and matches it at once.
    pub fn add_pattern<'a>(
        &mut self,
        expression: &str,
        exact: bool,
        objects: impl IntoIterator<Item = ObservedObject<'a>>,
    ) -> TrackerUpdate {
        if expression.is_empty() {
            return TrackerUpdate::default();
        }
        let expression = if exact {
            format!("^{}$", expression.trim_matches(['^', '$']))
        } else {
            expression.to_owned()
        };
        let pattern = match compile(&expression) {
            Ok(pattern) => pattern,
            Err(error) => {
                return TrackerUpdate::notice(format!("[Tracker] Invalid pattern: {error}"))
            }
        };
        self.patterns.push(pattern);
        self.rebuild_pattern_view();
        let objects: Vec<_> = objects.into_iter().collect();
        let mut update = self.scan_with_pattern(self.patterns.len() - 1, &objects);
        update.changed = true;
        update
            .effects
            .push(Effect::SavePatterns(self.encoded_patterns().into()));
        update
    }

    /// Adds the selected object's name, literally.
    pub fn add_selected<'a>(
        &mut self,
        name: Option<&str>,
        objects: impl IntoIterator<Item = ObservedObject<'a>>,
    ) -> TrackerUpdate {
        name.map_or_else(TrackerUpdate::default, |name| {
            self.add_pattern(&regex::escape(name), false, objects)
        })
    }

    pub fn remove_pattern(&mut self, index: usize) -> TrackerUpdate {
        if index >= self.patterns.len() {
            return TrackerUpdate::default();
        }
        self.patterns.remove(index);
        self.rebuild_pattern_view();
        TrackerUpdate {
            changed: true,
            effects: vec![Effect::SavePatterns(self.encoded_patterns().into())],
        }
    }

    /// Drops a match from the list (it is not announced again while it stays around).
    pub fn dismiss_match(&mut self, object: u64) -> bool {
        let before = self.found.len();
        self.found.retain(|found| found.object != object);
        if self.found.len() == before {
            return false;
        }
        self.rebuild_match_view();
        true
    }

    /// Picks a match: the arrow points at it, and the object the client should select is
    /// returned (`None` for an object not in the list).
    pub fn select_match(&mut self, object: u64) -> Option<u64> {
        if !self.found.iter().any(|f| f.object == object) {
            return None;
        }
        if self.pointer_target != Some(object) {
            self.set_pointer(Some(object));
        }
        Some(object)
    }

    fn set_pointer(&mut self, target: Option<u64>) {
        self.pointer_target = target;
        self.pointer_revision = self.pointer_revision.wrapping_add(1).max(1);
    }

    /// A list whose patterns stop at the first that does not compile, as Virindi Sense loads.
    fn replace_patterns(&mut self, bytes: &[u8]) {
        self.patterns.clear();
        for expression in decode_patterns(bytes) {
            let Ok(pattern) = compile(&expression) else {
                break;
            };
            self.patterns.push(pattern);
        }
        self.rebuild_pattern_view();
    }

    fn clear_matches(&mut self) {
        if self.found.is_empty() && self.match_view.is_empty() {
            return;
        }
        self.found.clear();
        self.rebuild_match_view();
    }

    fn scan_with_pattern(&mut self, index: usize, objects: &[ObservedObject<'_>]) -> TrackerUpdate {
        let mut update = TrackerUpdate::default();
        for object in objects.iter().copied() {
            let Some(name) = object.name else {
                continue;
            };
            if !object.containment_known
                || object.container.is_some()
                || self.found.iter().any(|f| f.object == object.object)
            {
                continue;
            }
            if matches(&self.patterns[index], name) {
                update.merge(self.record_match(object));
            }
        }
        update
    }

    fn matches_any(&self, name: &str) -> bool {
        self.patterns.iter().any(|p| matches(p, name))
    }

    fn record_match(&mut self, object: ObservedObject<'_>) -> TrackerUpdate {
        let name: Arc<str> = Arc::from(object.name.unwrap_or_default());
        self.found.push(MatchView {
            object: object.object,
            name: Arc::clone(&name),
        });
        self.rebuild_match_view();
        TrackerUpdate {
            changed: true,
            effects: vec![Effect::Detected {
                object: object.object,
                name,
                at: object.cell.and_then(coordinates_text).map(Arc::from),
            }],
        }
    }

    fn rebuild_pattern_view(&mut self) {
        self.pattern_revision = self.pattern_revision.wrapping_add(1).max(1);
        self.pattern_view = self
            .patterns
            .iter()
            .map(|p| PatternView {
                expression: Arc::clone(&p.expression),
                exact: is_exact(&p.expression),
            })
            .collect::<Vec<_>>()
            .into();
    }

    fn rebuild_match_view(&mut self) {
        self.match_revision = self.match_revision.wrapping_add(1).max(1);
        self.match_view = self.found.clone().into();
    }

    fn encoded_patterns(&self) -> Vec<u8> {
        encode_patterns(self.patterns.iter().map(|p| p.expression.as_ref()))
    }
}

/// A pattern compiled case-insensitively; the error is the regex library's message.
fn compile(expression: &str) -> Result<CompiledPattern, String> {
    Ok(CompiledPattern {
        expression: Arc::from(expression),
        regex: Regex::new(&format!("(?i:{expression})")).map_err(|e| e.to_string())?,
    })
}

fn matches(pattern: &CompiledPattern, name: &str) -> bool {
    pattern.regex.is_match(name).unwrap_or(false)
}

fn is_exact(expression: &str) -> bool {
    expression.starts_with('^') && expression.ends_with('$')
}
