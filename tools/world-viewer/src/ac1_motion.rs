//! The panel's "AC1 motion" toggle: the AC2 player is animated by an AC1 motion table instead of
//! its own, retargeted onto its skeleton (`ac1_assets::retarget`). The walker's movement picks
//! the AC1 cycle as AC1's `CMotionInterp` would (ready, run, walk back, side step, turn, fall);
//! the cycle's part frames turn the AC2 bones the body table pairs, and the bones no pair covers
//! keep the AC2 animation. An AC2 body without a pairing keeps its own motion.

use std::collections::HashMap;
use std::sync::Arc;

use ac1_assets::retarget::{BonePair, Retarget, HUMAN};
use ac1_assets::{Ac1Dats, MotionClip};
use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_client::PlayerFrame;
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_physics::PartPose;

use crate::walk::WalkInput;

/// AC1 `MotionCommand`s the walker's movement maps to.
mod motion {
    pub const READY: u32 = 0x4100_0003;
    pub const WALK_BACKWARD: u32 = 0x4500_0006;
    pub const RUN_FORWARD: u32 = 0x4400_0007;
    pub const TURN_RIGHT: u32 = 0x6500_000D;
    pub const SIDE_STEP_RIGHT: u32 = 0x6500_000F;
    pub const FALLING: u32 = 0x4000_0015;
}

/// An AC1 body for an AC2 race: its Setup, its motion table and the bone pairs.
struct Body {
    setup: u32,
    motion_table: u32,
    pairs: &'static [BonePair],
}

/// The AC1 player body each AC2 race and sex takes: AC1's own heritage of that race, as AC1's
/// `CharGen` (`0x0E000002`) lists it (`SexCG`'s Setup and motion table). AC1's playable humans,
/// Lugians, Tumeroks and Empyreans all share the human's part layout, and AC2's humans, Lugians
/// and Tumeroks share its bone names, so one pairing serves them all.
fn body_for(race: u32, sex: u32) -> Option<Body> {
    let (setup, motion_table) = ac1_body(race, sex)?;
    Some(Body {
        setup,
        motion_table,
        pairs: HUMAN,
    })
}

/// The AC1 body Setup and motion table of AC2 race `race` and sex `sex` (`body_for`).
pub fn ac1_body(race: u32, sex: u32) -> Option<(u32, u32)> {
    const MALE: u32 = 0x1000;
    let male = sex == MALE;
    Some(match (race, male) {
        // Human (Aluvian and the other human heritages).
        (0x1, true) => (0x0200_0001, 0x0900_0001),
        (0x1, false) => (0x0200_004E, 0x0900_0001),
        // Lugian.
        (0x2, true) => (0x0200_1A18, 0x0900_0216),
        (0x2, false) => (0x0200_1A12, 0x0900_020E),
        // Tumerok.
        (0x4, true) => (0x0200_1A1A, 0x0900_0214),
        (0x4, false) => (0x0200_1A14, 0x0900_0213),
        // Empyrean.
        (0xA, true) => (0x0200_1A16, 0x0900_020B),
        (0xA, false) => (0x0200_1A10, 0x0900_020A),
        _ => return None,
    })
}

/// One race's retarget and its AC1 cycles.
struct Rig {
    retarget: Retarget,
    motion_table: u32,
    parts: usize,
    cycles: HashMap<u32, Option<MotionClip>>,
}

/// The AC1 motion of the player.
pub struct Ac1Motion {
    dats: Ac1Dats,
    store: Arc<ResourceStore>,
    /// Each AC2 visual's rig (`None`: no AC1 body or no retarget for it).
    rigs: HashMap<u32, Option<Rig>>,
    /// The cycle playing and when it started.
    playing: Option<(u32, f64)>,
}

impl Ac1Motion {
    pub fn new(dats: Ac1Dats, store: Arc<ResourceStore>) -> Self {
        Self {
            dats,
            store,
            rigs: HashMap::new(),
            playing: None,
        }
    }

    fn rig(&mut self, visual: u32, race: u32, sex: u32) -> Option<&mut Rig> {
        if !self.rigs.contains_key(&visual) {
            let rig = self.build(visual, race, sex);
            if rig.is_none() {
                eprintln!(
                    "world-viewer: no AC1 motion for visual {visual:#010x} (race {race:#x}, sex {sex:#x})"
                );
            }
            self.rigs.insert(visual, rig);
        }
        self.rigs.get_mut(&visual)?.as_mut()
    }

    /// The AC1 body of `race` and `sex` retargeted onto AC2 visual `visual`'s root skeleton.
    pub fn retarget(&mut self, visual: u32, race: u32, sex: u32) -> Option<&Retarget> {
        self.rig(visual, race, sex).map(|rig| &rig.retarget)
    }

    /// The race's AC1 body retargeted onto the root skeleton of the AC2 visual.
    fn build(&self, visual: u32, race: u32, sex: u32) -> Option<Rig> {
        let body = body_for(race, sex)?;
        let model = ac2_render::object::load_object_model(
            &self.store,
            DataId::new(visual),
            &ac2_render::object::RenderSettings::default(),
        )
        .ok()??;
        let group = model.skeleton.as_ref()?.groups.first()?.setup;
        let setup = self
            .store
            .load_cached_packed::<Ac2Setup>(TypedResourceRef::new(DatFileKind::Portal, group, 0))
            .ok()??;
        let ac1_ref = self.dats.rest_pose(body.setup).ok()?;
        let joints = self
            .dats
            .limb_joints(body.setup, body.motion_table, body.pairs, &ac1_ref)
            .unwrap_or_default();
        let retarget = Retarget::new(&setup, &ac1_ref, body.pairs, &joints)?;
        Some(Rig {
            retarget,
            motion_table: body.motion_table,
            parts: ac1_ref.len(),
            cycles: HashMap::new(),
        })
    }

    /// The AC1 cycle the walker's movement plays, and whether it runs backwards (AC1 plays the
    /// left turns and side steps as the right ones at negative speed).
    fn pick(input: &WalkInput, contact: bool) -> (u32, bool) {
        if !contact {
            (motion::FALLING, false)
        } else if input.forward > 0.0 {
            (motion::RUN_FORWARD, false)
        } else if input.forward < 0.0 {
            (motion::WALK_BACKWARD, false)
        } else if input.strafe != 0.0 {
            (motion::SIDE_STEP_RIGHT, input.strafe < 0.0)
        } else if input.turn != 0.0 {
            // A positive turn is to the left.
            (motion::TURN_RIGHT, input.turn > 0.0)
        } else {
            (motion::READY, false)
        }
    }

    /// Poses `player` (of AC2 race `race` and sex `sex`) with the AC1 cycle its movement picks
    /// at `time`.
    pub fn apply(
        &mut self,
        player: &mut PlayerFrame,
        race: u32,
        sex: u32,
        input: &WalkInput,
        time: f64,
    ) -> bool {
        let Some(visual) = player.visual else {
            return false;
        };
        let (mut wanted, reversed) = Self::pick(input, player.contact);
        let dats = self.dats.clone();
        let mut playing = self.playing;
        let Some(rig) = self.rig(visual, race, sex) else {
            return false;
        };
        let cycle = |motion: u32, rig: &mut Rig| {
            rig.cycles
                .entry(motion)
                .or_insert_with(|| {
                    dats.motion_cycle(rig.motion_table, motion, rig.parts)
                        .ok()
                        .flatten()
                })
                .is_some()
        };
        if !cycle(wanted, rig) {
            wanted = motion::READY;
            if !cycle(wanted, rig) {
                return false;
            }
        }
        let key = wanted | u32::from(reversed) << 31;
        let started = match playing {
            Some((current, started)) if current == key => started,
            _ => {
                playing = Some((key, time));
                time
            }
        };
        let Some(Some(clip)) = rig.cycles.get(&wanted) else {
            return false;
        };
        let t = time - started;
        let parts: Vec<Ac2Transform> = clip
            .looped(if reversed { -t } else { t })
            .iter()
            .map(PartPose::transform)
            .collect();
        let animation: Vec<Ac2Transform> = player.pose.iter().map(PartPose::transform).collect();
        player.pose = rig
            .retarget
            .pose(&parts, &animation)
            .into_iter()
            .map(|t| PartPose {
                offset: t.offset.to_array(),
                rotation: [t.rotation.w, t.rotation.x, t.rotation.y, t.rotation.z],
                scale: t.scale.to_array(),
            })
            .collect();
        self.playing = playing;
        true
    }
}
