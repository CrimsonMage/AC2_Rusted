//! Engine structures the glue edits, in their run-time form: `MissileParameters` (every
//! field, constructor values first), `RandomSelectionTable<unsigned long>`, and
//! `BehaviorParams` (ac2-net's struct, with retail's `SetDefaults` and `get_pack_bitfield`).
//! The byte codecs are ac2-net's; [`crate::wire`] converts.

pub use ac2_net::protocol::BehaviorParams;

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

/// `MissileParameters::MissileParameters`'s values, in member order (floats as their bits).
const MISSILE_DEFAULTS: [(&str, u32); 35] = [
    ("max_velocity", 0x4248_0000),
    ("min_velocity", 0x3F80_0000),
    ("max_forward_accel", 0x3F80_0000),
    ("max_decel", 0x3F80_0000),
    ("max_left_accel", 0x3F80_0000),
    ("max_right_accel", 0x3F80_0000),
    ("max_upward_accel", 0x3F80_0000),
    ("max_downward_accel", 0x3F80_0000),
    ("total_fuel", 0),
    ("uses_fuel", 0),
    ("report_if_no_fuel", 1),
    ("missile_lifespan", 0),
    ("missile_has_lifespan", 0),
    ("drag", 0),
    ("affected_by_gravity", 0),
    ("affected_by_wind", 0),
    ("ground_follow_type", 0),
    ("height_above_ground", 0x3FC0_0000),
    ("tight_turn_method", 2),
    ("conserve_fuel", 0),
    ("consider_remaining_time", 0),
    ("min_catchup_speed", 0x4000_0000),
    ("maintain_distance", 0x40A0_0000),
    ("use_maint_dist", 0),
    ("reaction_time_check", 0x3F00_0000),
    ("min_reaction_time", 0x3F80_0000),
    ("correct_for_target_velocity", 1),
    ("use_env_max_velocities", 1),
    ("align_type", 0),
    ("spin_rate", 0),
    ("sticks_upon_collide", 0),
    ("can_land", 0),
    ("detonates_on_miss", 0),
    ("detonation_proximity", 0),
    ("detonation_chance", 0),
];

/// `MissileParameters`: the bitfield and every field by `m_*` name, each a 32-bit word (a
/// float's bits, an int or a bool, as the field is); unpacking starts from the constructor's
/// values and overwrites the packed groups.
#[derive(Debug, Clone, PartialEq)]
pub struct MissileParameters {
    pub bitfield: u32,
    pub fields: Vec<(&'static str, u32)>,
}

impl Default for MissileParameters {
    /// `MissileParameters::MissileParameters`.
    fn default() -> Self {
        Self {
            bitfield: 0,
            fields: MISSILE_DEFAULTS.to_vec(),
        }
    }
}

impl MissileParameters {
    /// The run-time object from its packed form: the constructor's values, then each set
    /// group's words in stream order (`MissileParameters::StreamPack`).
    pub fn from_packed(bitfield: u32, words: &[u32]) -> Self {
        let mut p = Self {
            bitfield,
            ..Self::default()
        };
        let mut words = words.iter();
        for (bit, names) in MISSILE_GROUPS {
            if bitfield & bit != 0 {
                for name in names {
                    if let Some(v) = words.next() {
                        p.set(name, *v);
                    }
                }
            }
        }
        p
    }

    /// The words `MissileParameters::StreamPack` writes for the set groups.
    pub fn packed_words(&self) -> Vec<u32> {
        MISSILE_GROUPS
            .iter()
            .filter(|(bit, _)| self.bitfield & bit != 0)
            .flat_map(|(_, names)| names.iter().map(|n| self.field(n).unwrap_or(0)))
            .collect()
    }

    /// A field's word.
    pub fn field(&self, name: &str) -> Option<u32> {
        self.fields.iter().find(|(n, _)| *n == name).map(|f| f.1)
    }

    /// Sets a field's word (names as in `MISSILE_GROUPS`).
    pub fn set(&mut self, name: &str, value: u32) {
        if let Some(f) = self.fields.iter_mut().find(|(n, _)| *n == name) {
            f.1 = value;
        }
    }
}

/// `BehaviorParams::SetDefaults` (`BehaviorParams::DEFAULTS`): time scale 1, one hold cycle,
/// send event on, everything else zero (`INVALID_IID` targets).
pub fn behavior_defaults() -> BehaviorParams {
    BehaviorParams {
        time_scale: 1.0,
        hold_cycles: 1,
        send_event: 1,
        ..BehaviorParams::default()
    }
}

/// `BehaviorParams::get_pack_bitfield`: a bit for each field that differs from the defaults
/// (the impulse by 0.0002 or more on an axis).
pub fn behavior_pack_bitfield(p: &BehaviorParams) -> u32 {
    let d = behavior_defaults();
    let mut bits = 0;
    let mut set = |bit: u32, differs: bool| {
        if differs {
            bits |= bit;
        }
    };
    set(0x1, p.behavior != d.behavior);
    set(0x2, p.mode != d.mode);
    set(0x4, p.time_scale != d.time_scale);
    set(0x8, p.early_callback != d.early_callback);
    set(0x10, p.hold_cycles != d.hold_cycles);
    set(0x20, p.moveto_cancels != d.moveto_cancels);
    set(0x40, p.camera_target_flag != d.camera_target_flag);
    set(0x80, p.camera_parent != d.camera_parent);
    set(0x4_0000, p.camera_hold != d.camera_hold);
    set(0x8_0000, p.camera_restore != d.camera_restore);
    set(0x20_0000, p.camera_target != d.camera_target);
    set(0x100, p.fade_children != d.fade_children);
    set(0x400, p.propagate != d.propagate);
    set(0x800, p.lock_actions != d.lock_actions);
    set(0x1000, p.fx_script != d.fx_script);
    set(0x10_0000, p.fx != d.fx);
    set(0x200, p.send_event != d.send_event);
    set(0x2000, p.target != d.target);
    set(0x1_0000, p.vdesc_to_clone != d.vdesc_to_clone);
    let moved = (0..3).any(|i| (f64::from(p.impulse[i]) - f64::from(d.impulse[i])).abs() >= 0.0002);
    set(0x4000, moved);
    set(0x2_0000, p.leave_idle_alone != d.leave_idle_alone);
    set(0x8000, p.destroy_on_completion != d.destroy_on_completion);
    set(0x80_0000, p.weenie_emote != d.weenie_emote);
    set(0x40_0000, p.context != d.context);
    bits
}

/// `RandomSelectionTable<unsigned long>::StreamPack`: the unused then the used elements, each
/// `ElementStruct::StreamPack` (object, weight, subtree weight).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RandomSelectionTable {
    /// `m_unusedElements`: (`m_object`, `m_weight`, `m_subTreeWeight`), a heap-ordered weight
    /// tree (children of i at 2i+1, 2i+2).
    pub unused: Vec<(u32, u32, u32)>,
    /// `m_usedElements`.
    pub used: Vec<(u32, u32, u32)>,
}
