//! The ambient sound's driver (`WorldRender::UpdateAmbientSounds`): once the listener has moved
//! `m_fSoundUpdateDistance` (`AmbientSound::DoSoundUpdate`), the land points of the blocks around
//! (`LScapeRender::AddCellSounds`) and the visible interior cells (`CellRender::AddCellSounds`,
//! sound type 2) weigh the environments, and `AmbientSound::UseTime` runs every frame. The world
//! answers through [`SoundSpace`]; the block data the loader reads arrives through `set_data` and
//! `set_points`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ambient::AmbientSound;
use crate::ambient_data::AmbientData;
use crate::manager::SoundManager;
use crate::playing::Emitter;
use crate::space::SoundSpace;

/// `CEnvCell::m_ambientSoundType`: every Legions interior cell's.
const ENV_CELL_SOUND_TYPE: u32 = 2;

/// A landblock's ambient sound point (`CLandBlock::CreateAmbientSoundPoints`): its sound type
/// (a `CSoundDesc` key) and its origin, block-local.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbientPoint {
    pub sound_type: u32,
    pub origin: [f32; 3],
}

/// What the driver needs read on the loader thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientRequest {
    /// The region's sound data (`CRegionDesc::GetCurrentSoundInfo`).
    Data,
    /// A landblock's points, by its block cell id.
    Points(u32),
}

/// `CEnvCell`'s interior cells are the ids past the outdoor ones: an outdoor cell has a low id
/// (`cell & 0xFFFF < 0x100`).
pub fn is_outside(cell: u32) -> bool {
    cell & 0xFFFF < 0x100
}

/// The ambient sound's state: the sound itself, the data and points it has, and the listener
/// place it last weighed (`m_posPlayerOld`).
#[derive(Debug)]
pub struct AmbientDriver<P> {
    pub sound: AmbientSound,
    data_requested: bool,
    points: HashMap<u32, Vec<AmbientPoint>>,
    points_requested: HashSet<u32>,
    moved_from: Option<P>,
}

impl<P> Default for AmbientDriver<P> {
    fn default() -> Self {
        Self {
            sound: AmbientSound::default(),
            data_requested: false,
            points: HashMap::new(),
            points_requested: HashSet::new(),
            moved_from: None,
        }
    }
}

impl<P: Copy> AmbientDriver<P> {
    pub fn set_data(&mut self, data: Option<Arc<AmbientData>>) {
        self.sound.set_data(data);
    }

    /// A landblock's points arrived: they weigh on the next update.
    pub fn set_points(&mut self, block: u32, points: Vec<AmbientPoint>) {
        self.points.insert(block, points);
        self.moved_from = None;
    }

    /// `WorldRender::UpdateAmbientSounds` and `AmbientSound::UseTime` for the frame. `listener`
    /// is the player's place when it is in the world. The reads it needs are pushed to
    /// `requests` (each once).
    pub fn use_time<S: SoundSpace<Place = P>>(
        &mut self,
        space: &S,
        listener: Option<P>,
        now: f64,
        manager: &mut SoundManager,
        requests: &mut Vec<AmbientRequest>,
    ) {
        if !self.data_requested {
            self.data_requested = true;
            requests.push(AmbientRequest::Data);
        }
        let Some(player) = listener else {
            return;
        };
        let cell = space.cell_id(player);
        // The blocks around the player (the landscape the renderer keeps).
        let (bx, by) = ((cell >> 24) as i32, ((cell >> 16) & 0xFF) as i32);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let (x, y) = (bx + dx, by + dy);
                if !(0..=0xFF).contains(&x) || !(0..=0xFF).contains(&y) {
                    continue;
                }
                let block = ((x as u32) << 24) | ((y as u32) << 16);
                if self.points_requested.insert(block) {
                    requests.push(AmbientRequest::Points(block));
                }
            }
        }
        // `AmbientSound::DoSoundUpdate`.
        let update_distance = f64::from(self.sound.update_distance);
        let moved = self.moved_from.as_ref().is_none_or(|old| {
            space.distance_squared(player, *old) >= update_distance * update_distance
        });
        if moved && self.sound.enabled() {
            self.moved_from = Some(player);
            let player_outside = is_outside(cell);
            self.sound.change_cell_start();
            // `CellRender::AddCellSounds`: the visible interior cells; the landscape too
            // unless the player is in a cell that cannot see outside.
            let mut landscape = true;
            if !player_outside {
                if let Some(env) = space.env_cell(cell) {
                    landscape = env.seen_outside;
                    let visible = std::iter::once(env.cell_id).chain(env.stab_list.iter().copied());
                    for id in visible {
                        if let Some(other) = space.env_cell(id) {
                            let offset = space.offset(player, other.place);
                            self.sound.add_cell_sound(
                                ENV_CELL_SOUND_TYPE,
                                offset,
                                false,
                                player_outside,
                            );
                        }
                    }
                }
            }
            if landscape {
                // `LScapeRender::AddCellSounds`.
                for (block, points) in &self.points {
                    for point in points {
                        let at = space.point(block | 0xFFFF, point.origin);
                        let offset = space.offset(player, at);
                        self.sound
                            .add_cell_sound(point.sound_type, offset, true, player_outside);
                    }
                }
            }
            self.sound.change_cell_end(now, manager);
        }
        self.sound.use_time(now, manager);
    }

    /// Every ambient emitter from the listener (`AmbientSound::emitters` in the listener's frame).
    pub fn emitters_from<S: SoundSpace<Place = P>>(
        &self,
        space: &S,
        listener: P,
    ) -> HashMap<u64, Emitter> {
        self.sound
            .emitters()
            .into_iter()
            .map(|(id, global)| {
                let emitter = Emitter {
                    offset: space.listener_local(listener, global),
                    velocity: [0.0; 3],
                    muted: false,
                };
                (id, emitter)
            })
            .collect()
    }
}
