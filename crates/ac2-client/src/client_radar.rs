//! The radar (`gmRadarUI`) and the map notes (`gmNavigationUI`) the UI reads:
//! - `gmRadarUI::AddObject` / `RemoveObject` / `RecvNotice_CreateObject`: each object's radar
//!   look, `gmCWInterface::InqRadarLook` on its script (a blip unless its shape is `0x4000000B`),
//!   and its groove level (int quality `0x3B6`), kept in `_infoArray` order (a removal moves the
//!   last entry into the gap, as `SmartArray<RadarInfo>::RemoveUnOrdered`).
//! - `gmRadarUI::RecvNotice_ChangeRadarLook` (the player's: `Init`, every look again) and
//!   `RecvNotice_ReloadOptions`.
//! - `gRadarControl::CallIFun_InqRadarRanges`: the indoor and outdoor ranges, read once per `Init`.
//! - `gmRadarUI::UpdateGroove`: every `_grooveDelay` seconds the groove levels of the objects
//!   within range add up to the music's groove level (`SoundManager::SetGrooveLevel`).
//! - `gmNavigationUI::RecvNotice_AddMarker` / `RemoveMarker` / `ClearMarkers`: the map notes.

use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_audio::is_outside;
use ac2_physics::Position;
use ac2_script::host::radar::{self, RadarLook, NO_BLIP};
use ac2_script::vm::InterpStatus;
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use crate::frame::{MapMarker, RadarBlip};

/// `GrooveLevel`, the int quality an object's danger sets the music's intensity with.
const GROOVE_LEVEL: u32 = 0x3B6;
/// `gmRadarUI::_grooveDelay`'s default.
const GROOVE_DELAY: f64 = 2.0;
/// The marker type `gmNavigationUI` never shows (`type != 3` in its add and clear notices).
const HIDDEN_MARKER_TYPE: u32 = 3;
/// `gmNavigationUI::RecvNotice_AddMarker`: a type keeps at most this many notes (`> 9` pops).
const MAX_NOTES_PER_TYPE: usize = 10;
/// `RecvNotice_AddMarker`'s squared distance (in land cells) under which a note of the same
/// type is already there (`< 6.0`).
const NEAR_NOTE_DISTANCE_SQ: f64 = 6.0;

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

/// The radar's and the map notes' state in the client.
#[derive(Debug, Default)]
pub(crate) struct RadarState {
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
    pub(crate) markers: Arc<Vec<MapMarker>>,
}

impl RadarState {
    /// `gmRadarUI::AddObject`: the object's groove level, and its blip when it has one
    /// (`add_unique`).
    fn add(&mut self, id: u64, groove: i8, look: Option<RadarLook>) {
        self.known.insert(id, groove);
        let Some(look) = look.filter(|l| l.shape != NO_BLIP) else {
            return;
        };
        if !self.blips.iter().any(|(b, _)| *b == id) {
            self.blips.push((id, look));
        }
    }

    /// `gmRadarUI::RemoveObject`: the groove entry and the blip go (`RemoveUnOrdered`).
    fn remove(&mut self, id: u64) {
        self.known.remove(&id);
        if let Some(at) = self.blips.iter().position(|(b, _)| *b == id) {
            self.blips.swap_remove(at);
        }
    }

    /// `gmRadarUI::Init`: every object's look is asked again, the ranges read again.
    fn init(&mut self) {
        self.known.clear();
        self.blips.clear();
        self.ranges = None;
    }

    /// `gmNavigationUI::RecvNotice_AddMarker`: see [`add_map_note`].
    fn add_marker(&mut self, marker: MapMarker) {
        add_map_note(Arc::make_mut(&mut self.markers), marker);
    }

    /// `gmNavigationUI::RecvNotice_RemoveMarker` (`FlushMarkerByIID`).
    fn remove_marker(&mut self, kind: u32, id: u64) {
        Arc::make_mut(&mut self.markers).retain(|m| !(m.kind == kind && m.id == id));
    }

    /// `gmNavigationUI::RecvNotice_ClearMarkers` (type 3 is left alone).
    fn clear_markers(&mut self, kind: u32) {
        if kind != HIDDEN_MARKER_TYPE {
            Arc::make_mut(&mut self.markers).retain(|m| m.kind != kind);
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
fn groove_of(qualities: &ac2_weenie::WeenieQualities) -> i8 {
    qualities.inq_int(GROOVE_LEVEL).unwrap_or(0) as i8
}

#[cfg(test)]
#[path = "client_radar_tests.rs"]
mod tests;

impl Client {
    /// The radar's part of the frame's events: the notices (looks, map notes), the objects
    /// that left, then the looks the scripts can now answer. Returns what the scripts produced
    /// meanwhile, for the frame's events.
    pub(crate) fn radar_use_time(&mut self, events: &[SessionEvent]) -> Vec<SessionEvent> {
        // `gmRadarUI::Init` reads the ranges as the radar comes up.
        self.radar_ranges();
        for event in events {
            match event {
                SessionEvent::Notice(notice) => self.radar_notice(notice),
                SessionEvent::ObjectRemoved(id) => self.radar.remove(*id),
                SessionEvent::LeftWorld | SessionEvent::Disconnected(_) => {
                    self.radar.init();
                    Arc::make_mut(&mut self.radar.markers).clear();
                }
                _ => {}
            }
        }
        self.radar_looks()
    }

    fn radar_notice(&mut self, notice: &GameplayNotice) {
        match notice {
            // `gmRadarUI::RecvNotice_ChangeRadarLook`: the player's look means `Init`.
            GameplayNotice::ChangeRadarLook(id) if Some(*id) == self.session.player() => {
                self.radar.init();
            }
            GameplayNotice::ChangeRadarLook(id) => self.radar.remove(*id),
            GameplayNotice::ReloadOptions => self.radar.init(),
            GameplayNotice::AddMarker {
                kind,
                id,
                cell,
                image,
                message,
            } if self.session.player().is_some() => self.radar.add_marker(MapMarker {
                kind: *kind,
                id: *id,
                cell: *cell,
                image: *image,
                message: message.clone(),
            }),
            GameplayNotice::RemoveMarker { kind, id } => self.radar.remove_marker(*kind, *id),
            GameplayNotice::ClearMarkers(kind) => self.radar.clear_markers(*kind),
            _ => {}
        }
    }

    /// `gRadarControl::CallIFun_InqRadarRanges`, read once after each `Init`; without a script
    /// VM the ranges are 0.
    fn radar_ranges(&mut self) -> (f32, f32) {
        if let Some(ranges) = self.radar.ranges {
            return ranges;
        }
        let Some(scripts) = self.scripts.as_mut() else {
            return (0.0, 0.0);
        };
        let ranges = radar::inq_radar_ranges(scripts.host.vm_mut(), 0.0, 0.0).unwrap_or((0.0, 0.0));
        self.radar.ranges = Some(ranges);
        ranges
    }

    /// `gmRadarUI::AddObject` for each object whose process runs and that `AddObject` has not
    /// run for yet. A look that waits for data is asked again on a later frame.
    fn radar_looks(&mut self) -> Vec<SessionEvent> {
        let mut produced = Vec::new();
        let Some(scripts) = self.scripts.as_mut() else {
            return produced;
        };
        let pending: Vec<(u64, i8)> = self
            .session
            .objects()
            .iter()
            .filter(|o| !self.radar.known.contains_key(&o.id))
            .filter(|o| scripts.host.instance(o.id).is_some())
            .map(|o| (o.id, groove_of(&o.qualities)))
            .collect();
        for (id, groove) in pending {
            let Some(scripts) = self.scripts.as_mut() else {
                break;
            };
            match scripts.host.inq_radar_look(id) {
                Ok((outcome, look)) => {
                    if outcome.status != InterpStatus::Interrupted {
                        self.radar.add(id, groove, look);
                    }
                    self.script_outcome(id, None, Ok(outcome), &mut produced);
                }
                Err(error) => {
                    self.radar.add(id, groove, None);
                    if let Some(scripts) = self.scripts.as_mut() {
                        scripts
                            .problems
                            .push(format!("{id:#018x} InqRadarLook: {error:?}"));
                    }
                }
            }
        }
        produced
    }

    /// The object's position: the player's, or the remote's.
    fn object_position(&self, id: u64) -> Option<Position> {
        match self.player.as_ref() {
            Some(player) if player.id == id => Some(player.position()),
            _ => self.remotes.get(id).and_then(|r| r.position()),
        }
    }

    /// `gmRadarUI::UpdateGroove`: every `_grooveDelay` seconds, the groove levels of the objects
    /// within range (the player's indoor or outdoor range) add up to the groove level, which
    /// goes to the music when it changes: `SoundManager::SetGrooveLevel` forwards it to the
    /// playing style, whose next measure chooses its pattern at the new level.
    pub(crate) fn groove_use_time(&mut self, now: f64) {
        if now < self.radar.next_groove {
            return;
        }
        let Some(player) = self.player.as_ref().map(|p| p.position()) else {
            return;
        };
        self.radar.next_groove = now + GROOVE_DELAY;
        let (indoor, outdoor) = self.radar_ranges();
        let range = if is_outside(player.cell_id) {
            outdoor
        } else {
            indoor
        };
        let range_sq = within_range(range);
        let mut level = 0i8;
        for (id, groove) in &self.radar.known {
            if *groove == 0 {
                continue;
            }
            let Some(at) = self.object_position(*id) else {
                continue;
            };
            // `Position::localtolocal`: the offset in the player's frame.
            let local = player.frame.global_to_local_vec(player.offset_to(&at));
            if inside(&local, range_sq) {
                level = level.wrapping_add(*groove);
            }
        }
        if level != self.radar.groove {
            self.radar.groove = level;
            if let Some(audio) = self.audio.as_mut() {
                audio.manager.set_groove_level(i32::from(level));
            }
        }
    }

    /// The radar for [`crate::ClientFrame::radar`].
    pub(crate) fn radar_frame(&self) -> crate::frame::RadarFrame {
        let player = self.player.as_ref().map(|p| p.position());
        let selection = self.selection_frame().map(|s| s.id);
        let (indoor, outdoor) = self.radar.ranges.unwrap_or((0.0, 0.0));
        let range = player.as_ref().map_or(0.0, |p| {
            if is_outside(p.cell_id) {
                outdoor
            } else {
                indoor
            }
        });
        let range_sq = within_range(range);
        let blips = self
            .radar
            .blips
            .iter()
            .filter_map(|(id, look)| {
                let at = self.object_position(*id)?;
                let offset = player
                    .as_ref()
                    .map(|p| p.frame.global_to_local_vec(p.offset_to(&at)));
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
        crate::frame::RadarFrame {
            groove: self.radar.groove,
            range,
            heading_degrees: player.map(|p| p.frame.heading().degrees()),
            blips,
        }
    }
}
