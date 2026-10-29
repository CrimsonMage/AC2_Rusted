//! The Tracker's list as the player sees it: each match with its distance (to the decimetre) and,
//! outdoors, its map coordinates. Distances are re-sampled at most every `SAMPLE_INTERVAL`, and a
//! new presentation is made only when a shown value changed, so the window does not churn as
//! the player walks.

use std::sync::Arc;
use std::time::Duration;

use ac2_physics::landdefs::{get_block_offset, gid_to_game_map_coords, is_outside};

use super::runtime::{MatchView, PatternView};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(250);

/// A position: its cell and the offset within it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackerPosition {
    pub cell: u32,
    pub origin: [f32; 3],
}

/// The client's answer for where a tracked object is now.
pub trait PositionQuery {
    fn position(&self, object: u64) -> Option<TrackerPosition>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentedPattern {
    pub expression: Arc<str>,
    /// The pattern as the list shows it (`Name (Exact)` for an anchored one).
    pub display: Arc<str>,
    pub exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentedMatch {
    pub object: u64,
    pub name: Arc<str>,
    pub distance_decimetres: u32,
    pub coordinates: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    pub pattern_revision: u64,
    pub match_revision: u64,
    pub pointer_revision: u64,
    pub pointer_target: Option<u64>,
    pub patterns: Arc<[PresentedPattern]>,
    pub matches: Arc<[PresentedMatch]>,
}

#[derive(Debug)]
pub(super) struct PresentationState {
    source: (u64, u64, u64),
    last_sample: Option<Duration>,
    pattern_revision: u64,
    match_revision: u64,
    pointer_target: Option<u64>,
    patterns: Arc<[PresentedPattern]>,
    matches: Arc<[PresentedMatch]>,
}

impl Default for PresentationState {
    fn default() -> Self {
        Self {
            source: (u64::MAX, u64::MAX, u64::MAX),
            last_sample: None,
            pattern_revision: 0,
            match_revision: 0,
            pointer_target: None,
            patterns: Arc::from([]),
            matches: Arc::from([]),
        }
    }
}

/// `source` is the runtime's pattern, match and pointer revisions.
#[allow(clippy::too_many_arguments)]
pub(super) fn project(
    state: &mut PresentationState,
    now: Duration,
    source: (u64, u64, u64),
    pointer_target: Option<u64>,
    patterns: &[PatternView],
    matches: &[MatchView],
    player: Option<TrackerPosition>,
    positions: &impl PositionQuery,
) -> Option<Presentation> {
    let patterns_changed = state.source.0 != source.0;
    let source_matches_changed = state.source.1 != source.1;
    let pointer_changed = state.source.2 != source.2;
    let sample_due = state
        .last_sample
        .is_none_or(|last| now < last || now.saturating_sub(last) >= SAMPLE_INTERVAL);
    let sample = source_matches_changed || sample_due;
    let matches_changed = sample && !same_rows(matches, &state.matches, player, positions);
    if sample {
        state.source.1 = source.1;
        state.last_sample = Some(now);
    }
    if !patterns_changed && !matches_changed && !pointer_changed {
        return None;
    }
    if patterns_changed {
        state.patterns = patterns
            .iter()
            .map(|p| PresentedPattern {
                expression: Arc::clone(&p.expression),
                display: display(&p.expression, p.exact),
                exact: p.exact,
            })
            .collect::<Vec<_>>()
            .into();
        state.source.0 = source.0;
        state.pattern_revision = state.pattern_revision.wrapping_add(1).max(1);
    }
    if matches_changed {
        state.matches = matches
            .iter()
            .filter_map(|found| row(found, player, positions))
            .collect::<Vec<_>>()
            .into();
        state.match_revision = state.match_revision.wrapping_add(1).max(1);
    }
    if pointer_changed {
        state.source.2 = source.2;
        state.pointer_target = pointer_target;
    }
    Some(Presentation {
        pattern_revision: state.pattern_revision,
        match_revision: state.match_revision,
        pointer_revision: state.source.2,
        pointer_target: state.pointer_target,
        patterns: Arc::clone(&state.patterns),
        matches: Arc::clone(&state.matches),
    })
}

fn display(expression: &str, exact: bool) -> Arc<str> {
    if exact {
        Arc::from(format!(
            "{} (Exact)",
            expression.trim_start_matches('^').trim_end_matches('$')
        ))
    } else {
        Arc::from(expression)
    }
}

/// Whether the shown rows would stay as they are (the matches without a position are hidden).
fn same_rows(
    source: &[MatchView],
    current: &[PresentedMatch],
    player: Option<TrackerPosition>,
    positions: &impl PositionQuery,
) -> bool {
    let mut current = current.iter();
    for found in source {
        let Some((distance, coordinates)) = measure(found, player, positions) else {
            continue;
        };
        let Some(shown) = current.next() else {
            return false;
        };
        if shown.object != found.object
            || shown.name != found.name
            || shown.distance_decimetres != distance
            || shown.coordinates.as_deref() != coordinates.as_deref()
        {
            return false;
        }
    }
    current.next().is_none()
}

fn row(
    found: &MatchView,
    player: Option<TrackerPosition>,
    positions: &impl PositionQuery,
) -> Option<PresentedMatch> {
    let (distance_decimetres, coordinates) = measure(found, player, positions)?;
    Some(PresentedMatch {
        object: found.object,
        name: Arc::clone(&found.name),
        distance_decimetres,
        coordinates: coordinates.map(Arc::from),
    })
}

fn measure(
    found: &MatchView,
    player: Option<TrackerPosition>,
    positions: &impl PositionQuery,
) -> Option<(u32, Option<String>)> {
    let player = player?;
    let target = positions.position(found.object)?;
    // `LandDefs::get_block_offset` between the blocks, plus the offsets within them.
    let block = get_block_offset(player.cell, target.cell);
    let offset = [
        f64::from(block.x) + f64::from(target.origin[0]) - f64::from(player.origin[0]),
        f64::from(block.y) + f64::from(target.origin[1]) - f64::from(player.origin[1]),
        f64::from(block.z) + f64::from(target.origin[2]) - f64::from(player.origin[2]),
    ];
    let decimetres = offset.iter().map(|v| v * v).sum::<f64>().sqrt() * 10.0;
    (decimetres.is_finite() && decimetres <= f64::from(u32::MAX))
        .then(|| (decimetres.round() as u32, coordinates_text(target.cell)))
}

/// A land cell's map coordinates as the radar writes them (`12.3N, 45.6E`); `None` indoors.
pub fn coordinates_text(cell: u32) -> Option<String> {
    if !is_outside(cell) {
        return None;
    }
    let (east, north) = gid_to_game_map_coords(cell)?;
    let ns = if north < 0.0 { 'S' } else { 'N' };
    let ew = if east < 0.0 { 'W' } else { 'E' };
    Some(format!("{:.1}{ns}, {:.1}{ew}", north.abs(), east.abs()))
}
