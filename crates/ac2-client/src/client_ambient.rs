//! The client's ambient sound and music intensity:
//! - `WorldRender::UpdateAmbientSounds`: once the player has moved `m_fSoundUpdateDistance`
//!   (`AmbientSound::DoSoundUpdate`), the land points of the blocks around
//!   (`LScapeRender::AddCellSounds`) and the visible interior cells (`CellRender::AddCellSounds`,
//!   sound type 2) weigh the environments, and `AmbientSound::UseTime` runs every frame.
//! - `gmRadarUI::UpdateGroove`: every `_grooveDelay` seconds the groove levels (int quality
//!   0x3B6) of the objects within radar range add up to the music's groove level.
//!
//! The region's sound data and each block's points are read on the loader thread.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ac2_audio::{AmbientData, AmbientSound, Emitter};
use ac2_physics::{CellSource, Frame, Position, Quaternion, Vector};
use ac2_world::Ac2AmbientSoundPoint;

use crate::client::Client;
use crate::loader::Request;

/// `CEnvCell::m_ambientSoundType`: every Legions interior cell's.
const ENV_CELL_SOUND_TYPE: u32 = 2;
/// `GrooveLevel`, the int quality an object's danger sets the music's intensity with.
const GROOVE_LEVEL: u32 = 0x3B6;
/// `gmRadarUI::_grooveDelay`'s default.
const GROOVE_DELAY: f64 = 2.0;

/// What the loader read for the ambient sound.
#[derive(Debug, Clone)]
pub enum AmbientLoaded {
    Data(Option<Arc<AmbientData>>),
    Points(u32, Vec<Ac2AmbientSoundPoint>),
}

/// The ambient sound's state in the client.
#[derive(Debug, Default)]
pub(crate) struct ClientAmbient {
    pub(crate) sound: AmbientSound,
    data_requested: bool,
    points: HashMap<u32, Vec<Ac2AmbientSoundPoint>>,
    points_requested: HashSet<u32>,
    /// `m_posPlayerOld`.
    moved_from: Option<Position>,
    /// `gmRadarUI::_grooveLevel`, `_nextGroove`.
    groove: i8,
    next_groove: f64,
    /// `gRadarControl::CallIFun_InqRadarRanges`: indoor, outdoor.
    radar_ranges: Option<(f32, f32)>,
}

fn is_outside(cell: u32) -> bool {
    cell & 0xFFFF < 0x100
}

impl Client {
    pub(crate) fn ambient_loaded(&mut self, loaded: AmbientLoaded) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        match loaded {
            AmbientLoaded::Data(data) => audio.ambient.sound.set_data(data),
            AmbientLoaded::Points(block, points) => {
                audio.ambient.points.insert(block, points);
                // New points: weigh again on the next update.
                audio.ambient.moved_from = None;
            }
        }
    }

    /// `WorldRender::UpdateAmbientSounds` and `AmbientSound::UseTime` for the frame.
    pub(crate) fn ambient_use_time(&mut self, now: f64) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        let ambient = &mut audio.ambient;
        if !ambient.data_requested {
            ambient.data_requested = true;
            self.loader.request(Request::AmbientData);
        }
        let Some(player) = self
            .player
            .as_ref()
            .filter(|p| p.in_world)
            .map(|p| p.position())
        else {
            return;
        };
        // The blocks around the player (the landscape the renderer keeps).
        let (bx, by) = (
            (player.cell_id >> 24) as i32,
            ((player.cell_id >> 16) & 0xFF) as i32,
        );
        for dx in -1..=1 {
            for dy in -1..=1 {
                let (x, y) = (bx + dx, by + dy);
                if !(0..=0xFF).contains(&x) || !(0..=0xFF).contains(&y) {
                    continue;
                }
                let block = ((x as u32) << 24) | ((y as u32) << 16);
                if ambient.points_requested.insert(block) {
                    self.loader.request(Request::AmbientPoints(block));
                }
            }
        }
        // `AmbientSound::DoSoundUpdate`.
        let update_distance = f64::from(ambient.sound.update_distance);
        let moved = ambient
            .moved_from
            .as_ref()
            .is_none_or(|old| player.distance_squared(old) >= update_distance * update_distance);
        if moved && ambient.sound.enabled() {
            ambient.moved_from = Some(player);
            let player_outside = is_outside(player.cell_id);
            ambient.sound.change_cell_start();
            // `CellRender::AddCellSounds`: the visible interior cells; the landscape too
            // unless the player is in a cell that cannot see outside.
            let mut landscape = true;
            if !player_outside {
                if let Some(cell) = self.terrain.env_cell(player.cell_id) {
                    landscape = cell.seen_outside;
                    let visible =
                        std::iter::once(cell.cell_id).chain(cell.stab_list.iter().copied());
                    for id in visible.collect::<Vec<_>>() {
                        if let Some(other) = self.terrain.env_cell(id) {
                            let o = player.offset_to(&other.position);
                            ambient.sound.add_cell_sound(
                                ENV_CELL_SOUND_TYPE,
                                [o.x, o.y, o.z],
                                false,
                                player_outside,
                            );
                        }
                    }
                }
            }
            if landscape {
                // `LScapeRender::AddCellSounds`.
                for (block, points) in &ambient.points {
                    for point in points {
                        let [x, y, z] = point.origin;
                        let at = Position::new(
                            block | 0xFFFF,
                            Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY),
                        );
                        let o = player.offset_to(&at);
                        ambient.sound.add_cell_sound(
                            point.sound_type,
                            [o.x, o.y, o.z],
                            true,
                            player_outside,
                        );
                    }
                }
            }
            ambient.sound.change_cell_end(now, &mut audio.manager);
        }
        ambient.sound.use_time(now, &mut audio.manager);
        self.groove_use_time(now);
    }

    /// `gmRadarUI::UpdateGroove`.
    fn groove_use_time(&mut self, now: f64) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        let ambient = &mut audio.ambient;
        if now < ambient.next_groove {
            return;
        }
        ambient.next_groove = now + GROOVE_DELAY;
        if ambient.radar_ranges.is_none() {
            ambient.radar_ranges = self.scripts.as_mut().and_then(|scripts| {
                ac2_script::host::radar::inq_radar_ranges(scripts.host.vm_mut(), 0.0, 0.0)
            });
        }
        let Some(player) = self.player.as_ref().filter(|p| p.in_world) else {
            return;
        };
        let position = player.position();
        let (indoor, outdoor) = ambient.radar_ranges.unwrap_or((0.0, 0.0));
        let range = f64::from(if is_outside(position.cell_id) {
            outdoor
        } else {
            indoor
        });
        let mut level = 0i8;
        for object in self.session.objects().iter() {
            let groove = object.qualities.inq_int(GROOVE_LEVEL).unwrap_or(0) as i8;
            if groove == 0 {
                continue;
            }
            let Some(at) = self.remotes.get(object.id).and_then(|r| r.position()) else {
                continue;
            };
            // `Position::localtolocal`: the offset in the player's frame, level distance.
            let local = position.frame.global_to_local_vec(position.offset_to(&at));
            let (x, y) = (f64::from(local.x), f64::from(local.y));
            if x * x + y * y <= range * range {
                level = level.wrapping_add(groove);
            }
        }
        if level != ambient.groove {
            ambient.groove = level;
            audio.manager.set_groove_level(i32::from(level));
        }
    }

    /// The dominant ambient environment's `EnvInfo` (its reverb), once the cells were weighed.
    pub fn ambient_environment(&self) -> Option<u32> {
        self.audio.as_ref()?.ambient.sound.environment()
    }

    /// The ambient emitters from the listener (the viewer the ambient sound places around).
    pub(crate) fn ambient_emitters(&self, listener: &Position) -> HashMap<u64, Emitter> {
        let Some(audio) = self.audio.as_ref() else {
            return HashMap::new();
        };
        audio
            .ambient
            .sound
            .emitters()
            .into_iter()
            .map(|(id, [x, y, z])| {
                let local = listener.frame.global_to_local_vec(Vector::new(x, y, z));
                let emitter = Emitter {
                    offset: [local.x, local.y, local.z],
                    velocity: [0.0; 3],
                    muted: false,
                };
                (id, emitter)
            })
            .collect()
    }
}
