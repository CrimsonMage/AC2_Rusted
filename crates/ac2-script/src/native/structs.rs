//! Portal engine structures held as native objects: `Position`, `MissileParameters`,
//! `BehaviorParams`, `RandomSelectionTable<unsigned long>` (`RandomSelectionTable_Int`).

use crate::unpack::{Reader, UnpackResult};

/// `Position::UnPack`: the cell id, then `AFrame::UnPack` (origin, then the quaternion as
/// packed: w, x, y, z).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub objcell_id: u32,
    pub origin: [f32; 3],
    pub rotation: [f32; 4],
}

impl Position {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        Ok(Self {
            objcell_id: r.u32()?,
            origin: [r.f32()?, r.f32()?, r.f32()?],
            rotation: [r.f32()?, r.f32()?, r.f32()?, r.f32()?],
        })
    }
}

/// `MissileParameters::StreamPack`: a bitfield, then for each set bit its fields, in this
/// order (verified against the admin disassembly; bit 0x1000 guards two groups, 0x100 none).
const MISSILE_GROUPS: [(u32, &[&str]); 18] = [
    (0x1, &["max_velocity", "min_velocity"]),
    (
        0x2,
        &[
            "max_forward_accel",
            "max_decel",
            "max_left_accel",
            "max_right_accel",
            "max_upward_accel",
            "max_downward_accel",
        ],
    ),
    (0x4, &["total_fuel", "uses_fuel", "report_if_no_fuel"]),
    (0x8, &["missile_lifespan", "missile_has_lifespan"]),
    (0x10, &["drag"]),
    (0x20, &["affected_by_gravity"]),
    (0x40, &["affected_by_wind"]),
    (0x80, &["ground_follow_type", "height_above_ground"]),
    (
        0x1000,
        &[
            "conserve_fuel",
            "consider_remaining_time",
            "min_catchup_speed",
        ],
    ),
    (0x200, &["maintain_distance", "use_maint_dist"]),
    (
        0x400,
        &[
            "reaction_time_check",
            "min_reaction_time",
            "correct_for_target_velocity",
        ],
    ),
    (0x800, &["tight_turn_method"]),
    (0x1000, &["use_env_max_velocities"]),
    (0x2000, &["align_type", "spin_rate"]),
    (0x4000, &["sticks_upon_collide"]),
    (0x8000, &["can_land"]),
    (0x10000, &["detonates_on_miss"]),
    (0x20000, &["detonation_proximity", "detonation_chance"]),
];

/// `MissileParameters`: the bitfield and the packed fields by `m_*` name (each a 32-bit word:
/// a float, an int or a bool, as the field is).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MissileParameters {
    pub bitfield: u32,
    pub fields: Vec<(&'static str, u32)>,
}

impl MissileParameters {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let bitfield = r.u32()?;
        let mut fields = Vec::new();
        for (bit, names) in MISSILE_GROUPS {
            if bitfield & bit != 0 {
                for name in names {
                    fields.push((*name, r.u32()?));
                }
            }
        }
        Ok(Self { bitfield, fields })
    }

    /// A packed field's word, if its group was packed.
    pub fn field(&self, name: &str) -> Option<u32> {
        self.fields.iter().find(|(n, _)| *n == name).map(|f| f.1)
    }
}

/// `BehaviorParams` with `BehaviorParams::SetDefaults` values for the fields not packed.
#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorParams {
    /// The pack bitfield (`BehaviorParams::get_pack_bitfield`).
    pub bitfield: u32,
    pub behavior_id: u32,
    pub time_scale: f32,
    pub mode_id: u32,
    pub hold_cycles: u32,
    pub fx_script_id: u32,
    pub fx_id: u32,
    pub target_id: u64,
    pub camera_target_id: u64,
    pub camera_behavior: u32,
    pub vdesc_to_clone: u32,
    pub cloned_apr_id: u32,
    /// `m_clonedAppAprHash` (`PHashTable<unsigned long,float>`).
    pub cloned_app_apr_hash: Vec<(u32, f32)>,
    pub impulse: [f32; 3],
    pub early_callback: u32,
    pub move_to_cancels: u32,
    pub camera_parent: u32,
    pub camera_target: u32,
    pub camera_hold: u32,
    pub camera_restore: u32,
    pub fade_children: u32,
    pub propagate: u32,
    pub lock_actions: u32,
    pub send_event: u32,
    pub leave_idle_alone: u32,
    pub destroy_on_completion: u32,
    pub weenie_emote_id: u32,
    pub context_id: u32,
}

impl Default for BehaviorParams {
    /// `BehaviorParams::SetDefaults`.
    fn default() -> Self {
        Self {
            bitfield: 0,
            behavior_id: 0,
            time_scale: 1.0,
            mode_id: 0,
            hold_cycles: 1,
            fx_script_id: 0,
            fx_id: 0,
            // `INVALID_IID`.
            target_id: 0,
            camera_target_id: 0,
            camera_behavior: 0,
            vdesc_to_clone: 0,
            cloned_apr_id: 0,
            cloned_app_apr_hash: Vec::new(),
            impulse: [0.0; 3],
            early_callback: 0,
            move_to_cancels: 0,
            camera_parent: 0,
            camera_target: 0,
            camera_hold: 0,
            camera_restore: 0,
            fade_children: 0,
            propagate: 0,
            lock_actions: 0,
            send_event: 1,
            leave_idle_alone: 0,
            destroy_on_completion: 0,
            weenie_emote_id: 0,
            context_id: 0,
        }
    }
}

impl BehaviorParams {
    /// `BehaviorParams::StreamPack` (unpack).
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let bits = r.u32()?;
        let mut p = Self {
            bitfield: bits,
            ..Self::default()
        };
        let has = |bit: u32| bits & bit != 0;
        if has(0x1) {
            p.behavior_id = r.u32()?;
        }
        if has(0x4) {
            p.time_scale = r.f32()?;
        }
        if has(0x2) {
            p.mode_id = r.u32()?;
        }
        if has(0x10) {
            p.hold_cycles = r.u32()?;
        }
        if has(0x1000) {
            p.fx_script_id = r.u32()?;
        }
        if has(0x10_0000) {
            p.fx_id = r.u32()?;
        }
        if has(0x2000) {
            p.target_id = r.u64()?;
        }
        if has(0x20_0000) {
            p.camera_target_id = r.u64()?;
            p.camera_behavior = r.u32()?;
        }
        if has(0x1_0000) {
            p.vdesc_to_clone = r.u32()?;
            p.cloned_apr_id = r.u32()?;
            let n = r.hash_count(8, "cloned appearance count")?;
            p.cloned_app_apr_hash = (0..n)
                .map(|_| Ok((r.u32()?, r.f32()?)))
                .collect::<UnpackResult<_>>()?;
        }
        if has(0x4000) {
            p.impulse = [r.f32()?, r.f32()?, r.f32()?];
        }
        for (bit, field) in [
            (0x8, &mut p.early_callback),
            (0x20, &mut p.move_to_cancels),
            (0x80, &mut p.camera_parent),
            (0x40, &mut p.camera_target),
            (0x4_0000, &mut p.camera_hold),
            (0x8_0000, &mut p.camera_restore),
            (0x100, &mut p.fade_children),
            (0x400, &mut p.propagate),
            (0x800, &mut p.lock_actions),
            (0x200, &mut p.send_event),
            (0x2_0000, &mut p.leave_idle_alone),
            (0x8000, &mut p.destroy_on_completion),
            (0x80_0000, &mut p.weenie_emote_id),
            (0x40_0000, &mut p.context_id),
        ] {
            if has(bit) {
                *field = r.u32()?;
            }
        }
        Ok(p)
    }
}

/// `RandomSelectionTable<unsigned long>::StreamPack`: the unused then the used elements, each
/// `ElementStruct::StreamPack` (object, weight, subtree weight).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RandomSelectionTable {
    pub unused: Vec<(u32, f32, f32)>,
    pub used: Vec<(u32, f32, f32)>,
}

impl RandomSelectionTable {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let read = |r: &mut Reader<'_>| -> UnpackResult<Vec<(u32, f32, f32)>> {
            let n = r.count(12, "random selection element count")?;
            (0..n).map(|_| Ok((r.u32()?, r.f32()?, r.f32()?))).collect()
        };
        Ok(Self {
            unused: read(r)?,
            used: read(r)?,
        })
    }
}
