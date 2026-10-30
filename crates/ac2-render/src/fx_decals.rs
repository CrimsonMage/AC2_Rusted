//! `FXManager::DoDecalFX` and its callers: an FX node with a `DecalFX` projects a decal where its
//! emitter stands. The node's waveforms give the decal's size, its rotation and the radius its
//! position is scattered over (`Waveform::GetValue` at client time); its origin is offset from the
//! emitter, and the decal starts on the ground there (`DecalEffectManager::CreateDecalEffect`).
//! `DoTargettedFX` makes it with the node's gate (the streak part's gate); `DoStartupFX` makes it
//! for every node of a startup FX, with no gate. The decal then lives on its own: stopping the FX
//! does not end it (`DecalEffect` has no emitter). The calls queue here and the scene takes them
//! ([`FxDecals::take_calls`]).
//!
//! Not ported here: a decal on a dungeon cell (`DecalEffect::ConstructDungeonDecal` needs the cell's
//! polygons; such a node makes no decal).

use std::collections::BTreeSet;

use ac2_assets::{Ac2DecalFx, Ac2FxObject, Ac2ParticleAttachment, Ac2VisualFxParticles};
use ac2_core::{DataId, Random};
use ac2_math::Vec3;

use crate::decal::DecalSpec;
use crate::fx_streaks::{node_plays, StreakContext};

/// The `0x0100` cell-id bound: a cell below it is a landblock cell (`LandDefs::first_envcell_id`).
const FIRST_ENVCELL_ID: u32 = 0x100;

/// The world-space origin of landblock `cell`'s corner: the block's east and north offsets in
/// metres (the same origin the near terrain meshes use).
fn block_origin(cell: u32, block_size: f32) -> [f32; 2] {
    [
        ((cell >> 24) & 0xFF) as f32 * block_size,
        ((cell >> 16) & 0xFF) as f32 * block_size,
    ]
}

/// Where an emitter stands for a decal: its cell, its block-local origin and the world heading
/// of its local y axis (`Frame::fl2gv`'s second row, the vector `DoDecalFX` turns).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterPlace {
    pub cell: u32,
    pub origin: Vec3,
    pub forward: Vec3,
}

/// The decal `DoDecalFX` makes for `decal` at an emitter in `cell` standing at `origin` (its
/// block-local position) heading `forward`, at client time `time`; `None` where retail makes
/// none (a dungeon cell, or no cell at all).
///
/// The order of the work follows `FXManager::DoDecalFX`: the waveforms are read at `time`;
/// the position is the emitter's plus the node's origin; with a radius the decal is scattered
/// in the horizontal plane (two `RollDice(-1, 1)` for the direction, then `RollDice(0, 1)` for
/// its distance); the direction is the emitter's forward vector (`fl2gv[3..6]`, the y axis of its
/// frame, which retail reads at offset 0x30 of the `Position` copy) turned by the rotation about
/// the vertical axis, and normalised.
pub fn decal_spec(
    decal: &Ac2DecalFx,
    cell: u32,
    origin: Vec3,
    forward: Vec3,
    block_size: f32,
    time: f64,
    random: &mut Random,
) -> Option<DecalSpec> {
    if cell == 0 || (cell & 0xFFFF) >= FIRST_ENVCELL_ID {
        return None;
    }
    let size = decal.size.value(time);
    let rotation_deg = decal.rotation.value(time);
    let radius = decal.position_radius.value(time);
    let angle = f64::from(rotation_deg) * 0.017_453_292;

    let mut local = [
        origin.x + decal.origin.x,
        origin.y + decal.origin.y,
        origin.z + decal.origin.z,
    ];
    if radius > 0.0 {
        // `vRandomDir.y` is the first roll and `.x` the second.
        let y = f64::from(random.roll_dice_f32(-1.0, 1.0));
        let x = f64::from(random.roll_dice_f32(-1.0, 1.0));
        let length = (x * x + y * y).sqrt();
        if length > 0.0 {
            let distance = f64::from(random.roll_dice_f32(0.0, 1.0)) * f64::from(radius);
            local[0] += (x / length * distance) as f32;
            local[1] += (y / length * distance) as f32;
        }
    }

    let [east, north] = block_origin(cell, block_size);
    let world = Vec3::new(east + local[0], north + local[1], local[2]);
    // `Quaternion::RotationAxis(z, angle)` applied to the forward vector, evaluated in f64.
    let (sin, cos) = angle.sin_cos();
    let (fx, fy, fz) = (
        f64::from(forward.x),
        f64::from(forward.y),
        f64::from(forward.z),
    );
    let turned = Vec3::new(
        (fx * cos - fy * sin) as f32,
        (fx * sin + fy * cos) as f32,
        fz as f32,
    );
    let direction = turned.normalize_or_zero();
    Some(DecalSpec {
        material: DataId::new(decal.material),
        lifespan: decal.lifespan,
        size,
        direction,
        color: decal.color,
        draw_once: false,
        origin: world,
    })
}

/// The FX decal state: the FX whose decal nodes have run, the decals not yet taken by the
/// scene, and `DecalEffects` (off makes none; `DoDecalFX` checks the preference first).
#[derive(Debug, Default)]
pub struct FxDecals {
    /// The FX that have started on an emitter (guards the once-per-start rule the streak part
    /// shares: `DoTargettedFX`'s `m_current_fx`).
    started: BTreeSet<(u64, u32)>,
    calls: Vec<DecalSpec>,
    disabled: bool,
}

impl FxDecals {
    /// `FXManager::DoTargettedFX`'s decal part for an FX starting on `emitter`, standing in
    /// `at` (the emitter's cell, block-local origin and heading). Each node that plays (the same gate as
    /// the streak part) with a decal makes it, in order. A running FX starts nothing again.
    pub fn start(
        &mut self,
        emitter: u64,
        fx: u32,
        table: &Ac2VisualFxParticles,
        at: Option<EmitterPlace>,
        ctx: &StreakContext<'_>,
        block_size: f32,
        time: f64,
        random: &mut Random,
    ) {
        if !self.started.insert((emitter, fx)) {
            return;
        }
        let Some(nodes) = table.get_on(fx, ctx.terrain) else {
            return;
        };
        let is_player = ctx.player == Some(emitter);
        let playing = nodes.iter().filter(|node| {
            node_plays(&node.object.gate, is_player, ctx.night) && node.object.decal.is_some()
        });
        for node in playing {
            self.make(&node.object, at, block_size, time, random);
        }
    }

    /// `FXManager::DoStartupFX`'s decal parts: every node of a startup FX with a decal makes it
    /// (no gate). Called once per object, when its startup FX run.
    pub fn start_startup(
        &mut self,
        nodes: &[Ac2ParticleAttachment],
        at: Option<EmitterPlace>,
        block_size: f32,
        time: f64,
        random: &mut Random,
    ) {
        for node in nodes {
            self.make(&node.object, at, block_size, time, random);
        }
    }

    fn make(
        &mut self,
        object: &Ac2FxObject,
        at: Option<EmitterPlace>,
        block_size: f32,
        time: f64,
        random: &mut Random,
    ) {
        if self.disabled {
            return;
        }
        let (Some(decal), Some(place)) = (object.decal.as_ref(), at) else {
            return;
        };
        let spec = decal_spec(
            decal,
            place.cell,
            place.origin,
            place.forward,
            block_size,
            time,
            random,
        );
        if let Some(spec) = spec {
            self.calls.push(spec);
        }
    }

    /// `StopFX`: the FX can start (and make its decals) again. The decals already made stay.
    pub fn stop(&mut self, emitter: u64, fx: u32) {
        self.started.remove(&(emitter, fx));
    }

    /// The emitter left the world: its running FX are forgotten, so they can start again.
    pub fn forget_emitter(&mut self, emitter: u64) {
        self.started.retain(|(id, _)| *id != emitter);
    }

    /// The decals the FX made since the scene last took them, in order.
    pub fn take_calls(&mut self) -> Vec<DecalSpec> {
        std::mem::take(&mut self.calls)
    }

    /// `Render::m_Prefs.DecalEffects`: off makes no decal.
    pub fn set_enabled(&mut self, on: bool) {
        self.disabled = !on;
    }

    /// Whether the FX make decals (`DecalEffects` is on).
    pub fn enabled(&self) -> bool {
        !self.disabled
    }
}

#[cfg(test)]
#[path = "fx_decals_tests.rs"]
mod tests;
