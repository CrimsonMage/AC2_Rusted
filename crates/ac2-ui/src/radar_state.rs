//! The radar's and the map notes' state (`gmRadarUI`, `gmNavigationUI`), as plain data the client
//! feeds:
//! - `gmRadarUI::AddObject` / `RemoveObject` / `RecvNotice_CreateObject`: each object's radar
//!   look (`gmCWInterface::InqRadarLook` on its script, a blip unless its shape is `0x4000000B`),
//!   and its groove level (int quality `0x3B6`), kept in `_infoArray` order (a removal moves the
//!   last entry into the gap, as `SmartArray<RadarInfo>::RemoveUnOrdered`).
//! - `gmRadarUI::Init` (`init`): every look is asked again and the ranges are read again.
//! - `gmRadarUI::UpdateGroove`: every `_grooveDelay` seconds the groove levels of the objects
//!   within range add up to the music's groove level (`SoundManager::SetGrooveLevel`).
//! - `gmNavigationUI::RecvNotice_AddMarker` / `RemoveMarker` / `ClearMarkers`: the map notes.
//!
//! The client asks the scripts (`InqRadarLook`, `InqRadarRanges`) and reads the positions; this
//! module decides what they mean.

use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_net::protocol::StringInfo;
use ac2_physics::Position;
use ac2_script::host::radar::{RadarLook, NO_BLIP};
use ac2_weenie::WeenieQualities;

/// `GrooveLevel`, the int quality an object's danger sets the music's intensity with.
const GROOVE_LEVEL: u32 = 0x3B6;
/// `gmRadarUI::_grooveDelay`'s default.
pub const GROOVE_DELAY: f64 = 2.0;
/// The marker type `gmNavigationUI` never shows (`type != 3` in its add and clear notices).
const HIDDEN_MARKER_TYPE: u32 = 3;
/// `gmNavigationUI::RecvNotice_AddMarker`: a type keeps at most this many notes (`> 9` pops).
const MAX_NOTES_PER_TYPE: usize = 10;
/// `RecvNotice_AddMarker`'s squared distance (in land cells) under which a note of the same
/// type is already there (`< 6.0`).
const NEAR_NOTE_DISTANCE_SQ: f64 = 6.0;

/// A map note (`gmNavigationUI`'s marker) as `CM_Character::SendNotice_AddMarker` sends it. The
/// navigation panel draws these, not the radar.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapMarker {
    /// The marker's type (a `CMapNoteRow`); type 3 is never shown.
    pub kind: u32,
    /// The marked object's instance id (0 for none).
    pub id: u64,
    pub cell: u32,
    /// The marker's image DID (0 for none).
    pub image: u32,
    /// The marker's text.
    pub message: StringInfo,
}

/// A change to the map notes a framework asks of the client: `CM_Character::SendNotice_AddMarker`,
/// `SendNotice_RemoveMarker`, `SendNotice_ClearMarkers` (the fellows' notes,
/// `crate::fellow_markers`).
#[derive(Debug, Clone, PartialEq)]
pub enum MarkerRequest {
    Add(MapMarker),
    Remove { kind: u32, id: u64 },
    Clear(u32),
}

/// One object on the radar (`gmRadarUI::RadarInfo`): its look, and where it is from the player.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarBlip {
    pub id: u64,
    /// The `RGBAColor` the object's script gives (r, g, b, a).
    pub color: [f32; 4],
    /// The blip's shape, `0x40000001` (point) to `0x4000000A` (face).
    pub shape: u32,
    /// The object's cell and AC origin.
    pub cell: u32,
    pub origin: [f32; 3],
    /// The object's offset in the player's frame (`Position::localtolocal`).
    pub offset: [f32; 3],
    /// Within the radar's range (`gmRadarUI::Update` draws only these).
    pub in_range: bool,
    /// The selected object (`gmRadarUI::_currentSelection`).
    pub selected: bool,
}

/// What the radar shows (`gmRadarUI`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadarFrame {
    /// `gmRadarUI::_grooveLevel`: the danger the music's groove follows.
    pub groove: i8,
    /// The range the radar shows where the player stands (indoor or outdoor).
    pub range: f32,
    /// The player's heading, which the radar's heading field turns with.
    pub heading_degrees: Option<f32>,
    /// The objects with a blip, in the radar's own order (`gmRadarUI::_infoArray`).
    pub blips: Vec<RadarBlip>,
}

/// `gmNavigationUI::RecvNotice_AddMarker` and `AddMarker` over the map notes, in arrival order:
/// type 3 is never shown; a note of a type within `sqrt(6)` land cells of a note of that type
/// already there is ignored (its cell must be on the land); the type's oldest notes go beyond
/// the limit of ten; a note for the same object in its type replaces that object's. Admin
/// 7148's rule: Legions 7160 has no separate notice handler for it.
pub fn add_map_note(markers: &mut Vec<MapMarker>, note: MapMarker) {
    if note.kind == HIDDEN_MARKER_TYPE {
        return;
    }
    if let Some(land) = ac2_physics::landdefs::gid_to_lcoord(note.cell) {
        let near = markers
            .iter()
            .filter(|m| m.kind == note.kind)
            .filter_map(|m| ac2_physics::landdefs::gid_to_lcoord(m.cell))
            .any(|(x, y)| {
                let (dx, dy) = (f64::from(land.0 - x), f64::from(land.1 - y));
                dx * dx + dy * dy < NEAR_NOTE_DISTANCE_SQ
            });
        if near {
            return;
        }
    }
    while markers.iter().filter(|m| m.kind == note.kind).count() >= MAX_NOTES_PER_TYPE {
        if let Some(at) = markers.iter().position(|m| m.kind == note.kind) {
            markers.remove(at);
        }
    }
    if note.id != 0 {
        markers.retain(|m| !(m.kind == note.kind && m.id == note.id));
    }
    markers.push(note);
}

/// The radar's and the map notes' state.
#[derive(Debug, Default)]
pub struct RadarState {
    /// `gmRadarUI::_infoArray`: the objects with a blip, in the radar's order.
    blips: Vec<(u64, RadarLook)>,
    /// The objects `AddObject` ran for, with their groove level (`_grooveyArray`; 0 for none).
    known: BTreeMap<u64, i8>,
    /// `_indoorRadarRange`, `_outdoorRadarRange`; read on the first need after each `Init`.
    ranges: Option<(f32, f32)>,
    /// `_grooveLevel`, `_nextGroove`.
    groove: i8,
    next_groove: f64,
    /// `gmNavigationUI`'s map notes, in arrival order.
    pub markers: Arc<Vec<MapMarker>>,
}

impl RadarState {
    /// `gmRadarUI::AddObject`: the object's groove level, and its blip when it has one
    /// (`add_unique`).
    pub fn add(&mut self, id: u64, groove: i8, look: Option<RadarLook>) {
        self.known.insert(id, groove);
        let Some(look) = look.filter(|l| l.shape != NO_BLIP) else {
            return;
        };
        if !self.blips.iter().any(|(b, _)| *b == id) {
            self.blips.push((id, look));
        }
    }

    /// `gmRadarUI::RemoveObject`: the groove entry and the blip go (`RemoveUnOrdered`).
    pub fn remove(&mut self, id: u64) {
        self.known.remove(&id);
        if let Some(at) = self.blips.iter().position(|(b, _)| *b == id) {
            self.blips.swap_remove(at);
        }
    }

    /// Whether `AddObject` has run for the object.
    pub fn knows(&self, id: u64) -> bool {
        self.known.contains_key(&id)
    }

    /// The objects `AddObject` ran for, with their groove levels.
    pub fn known_grooves(&self) -> impl Iterator<Item = (u64, i8)> + '_ {
        self.known.iter().map(|(id, groove)| (*id, *groove))
    }

    /// `gmRadarUI::Init`: every object's look is asked again, the ranges read again.
    pub fn init(&mut self) {
        self.known.clear();
        self.blips.clear();
        self.ranges = None;
    }

    /// The indoor and outdoor ranges, once read after the last `init`.
    pub fn ranges(&self) -> Option<(f32, f32)> {
        self.ranges
    }

    /// Stores the ranges read after `init` (`gRadarControl::CallIFun_InqRadarRanges`).
    pub fn set_ranges(&mut self, ranges: (f32, f32)) {
        self.ranges = Some(ranges);
    }

    /// `gmRadarUI::UpdateGroove`'s clock: whether the groove is due at `now`.
    pub fn groove_due(&self, now: f64) -> bool {
        now >= self.next_groove
    }

    /// `gmRadarUI::UpdateGroove`, once due: every object within range adds its groove level to
    /// the total, the range being the player's indoor or outdoor one. `members` gives each
    /// object's groove level and position (`None` when it has none). Returns the new level when
    /// it changed, for `SoundManager::SetGrooveLevel`.
    pub fn update_groove(
        &mut self,
        now: f64,
        player: &Position,
        ranges: (f32, f32),
        outside: bool,
        members: &[(i8, Option<Position>)],
    ) -> Option<i8> {
        self.next_groove = now + GROOVE_DELAY;
        let range = if outside { ranges.1 } else { ranges.0 };
        let range_sq = within_range(range);
        let mut level = 0i8;
        for (groove, at) in members {
            if *groove == 0 {
                continue;
            }
            let Some(at) = at else {
                continue;
            };
            // `Position::localtolocal`: the offset in the player's frame.
            let local = player.frame.global_to_local_vec(player.offset_to(at));
            if inside(&local, range_sq) {
                level = level.wrapping_add(*groove);
            }
        }
        if level != self.groove {
            self.groove = level;
            Some(level)
        } else {
            None
        }
    }

    /// `gmRadarUI::RecvNotice_AddMarker`: see [`add_map_note`].
    pub fn add_marker(&mut self, marker: MapMarker) {
        add_map_note(Arc::make_mut(&mut self.markers), marker);
    }

    /// `gmNavigationUI::RecvNotice_RemoveMarker` (`FlushMarkerByIID`).
    pub fn remove_marker(&mut self, kind: u32, id: u64) {
        Arc::make_mut(&mut self.markers).retain(|m| !(m.kind == kind && m.id == id));
    }

    /// `gmNavigationUI::RecvNotice_ClearMarkers` (type 3 is left alone).
    pub fn clear_markers(&mut self, kind: u32) {
        if kind != HIDDEN_MARKER_TYPE {
            Arc::make_mut(&mut self.markers).retain(|m| m.kind != kind);
        }
    }

    /// The map notes go when the player leaves the world.
    pub fn forget_markers(&mut self) {
        Arc::make_mut(&mut self.markers).clear();
    }

    /// The radar for the frame: the blips with their offsets from the player, the range the
    /// player's cell gives (`outside` picks the outdoor one), and the groove. `position_of` gives
    /// an object's position.
    pub fn frame(
        &self,
        player: Option<&Position>,
        outside: bool,
        selection: Option<u64>,
        position_of: impl Fn(u64) -> Option<Position>,
    ) -> RadarFrame {
        let (indoor, outdoor) = self.ranges.unwrap_or((0.0, 0.0));
        let range = match player {
            Some(_) if outside => outdoor,
            Some(_) => indoor,
            None => 0.0,
        };
        let range_sq = within_range(range);
        let blips = self
            .blips
            .iter()
            .filter_map(|(id, look)| {
                let at = position_of(*id)?;
                let offset = player.map(|p| p.frame.global_to_local_vec(p.offset_to(&at)));
                let (offset, in_range) = match offset {
                    Some(o) => ([o.x, o.y, o.z], inside(&o, range_sq)),
                    None => ([0.0; 3], false),
                };
                Some(RadarBlip {
                    id: *id,
                    color: look.color,
                    shape: look.shape,
                    cell: at.cell_id,
                    origin: [at.frame.origin.x, at.frame.origin.y, at.frame.origin.z],
                    offset,
                    in_range,
                    selected: selection == Some(*id),
                })
            })
            .collect();
        RadarFrame {
            groove: self.groove,
            range,
            heading_degrees: player.map(|p| p.frame.heading().degrees()),
            blips,
        }
    }
}

/// The squared radar range the radar compares against: `fVar2 * fVar2` is stored as a float
/// (`fstp DWORD`, admin 0x425740), so it is rounded to f32 before the comparison.
fn within_range(range: f32) -> f32 {
    range * range
}

/// `gmRadarUI::UpdateGroove`'s test: `x * x + y * y <= range * range` (the sum in double, the
/// squared range a float), with `local` the offset in the player's frame.
fn inside(local: &ac2_physics::Vector, range_sq: f32) -> bool {
    let (x, y) = (f64::from(local.x), f64::from(local.y));
    x * x + y * y <= f64::from(range_sq)
}

/// The groove level an object's quality gives (`CWRepository::GetInt(obj, 0x3B6, 0)`).
pub fn groove_of(qualities: &WeenieQualities) -> i8 {
    qualities.inq_int(GROOVE_LEVEL).unwrap_or(0) as i8
}

#[cfg(test)]
#[path = "radar_state_tests.rs"]
mod tests;
