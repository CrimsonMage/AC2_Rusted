//! `MissileParameters::CopyInto` and `MissileTargetingParameters::CopyInto`: the merge that
//! `CMissileManager::Initialize`, `Retarget` and `ClientRetarget` use. Only the groups the
//! source's bitfield names are copied, and the group's bit is set on the destination.

use crate::missile::{MissileParameters, MissileTargetingParameters};

impl MissileParameters {
    /// `MissileParameters::CopyInto`: copies each group whose bit is set in `self.bitfield` into
    /// `dst` and sets that bit there. The groups follow the retail order, including its second
    /// test of bit `0x1000` (the environment-velocity group) after the conserve-fuel group.
    /// Always true.
    pub fn copy_into(&self, dst: &mut MissileParameters) -> bool {
        let bf = self.bitfield;
        let set = |bit: u32, dst: &mut MissileParameters| dst.bitfield |= bit;
        if bf & 1 != 0 {
            dst.max_velocity = self.max_velocity;
            dst.min_velocity = self.min_velocity;
            set(1, dst);
        }
        if bf & 2 != 0 {
            dst.max_forward_accel = self.max_forward_accel;
            dst.max_decel = self.max_decel;
            dst.max_left_accel = self.max_left_accel;
            dst.max_right_accel = self.max_right_accel;
            dst.max_upward_accel = self.max_upward_accel;
            dst.max_downward_accel = self.max_downward_accel;
            set(2, dst);
        }
        if bf & 4 != 0 {
            dst.total_fuel = self.total_fuel;
            dst.uses_fuel = self.uses_fuel;
            dst.report_if_no_fuel = self.report_if_no_fuel;
            set(4, dst);
        }
        if bf & 8 != 0 {
            dst.missile_lifespan = self.missile_lifespan;
            dst.missile_has_lifespan = self.missile_has_lifespan;
            set(8, dst);
        }
        if bf & 0x10 != 0 {
            dst.drag = self.drag;
            set(0x10, dst);
        }
        if bf & 0x20 != 0 {
            dst.affected_by_gravity = self.affected_by_gravity;
            set(0x20, dst);
        }
        if bf & 0x40 != 0 {
            dst.affected_by_wind = self.affected_by_wind;
            set(0x40, dst);
        }
        if bf & 0x80 != 0 {
            dst.ground_follow_type = self.ground_follow_type;
            dst.height_above_ground = self.height_above_ground;
            set(0x80, dst);
        }
        if bf & 0x1000 != 0 {
            dst.conserve_fuel = self.conserve_fuel;
            dst.min_catchup_speed = self.min_catchup_speed;
            dst.consider_remaining_time = self.consider_remaining_time;
            set(0x1000, dst);
        }
        if bf & 0x200 != 0 {
            dst.maintain_distance = self.maintain_distance;
            dst.use_maint_dist = self.use_maint_dist;
            set(0x200, dst);
        }
        if bf & 0x400 != 0 {
            dst.reaction_time_check = self.reaction_time_check;
            dst.min_reaction_time = self.min_reaction_time;
            dst.correct_for_target_velocity = self.correct_for_target_velocity;
            set(0x400, dst);
        }
        if bf & 0x800 != 0 {
            dst.tight_turn_method = self.tight_turn_method;
            set(0x800, dst);
        }
        if bf & 0x1000 != 0 {
            dst.use_env_max_velocities = self.use_env_max_velocities;
            set(0x1000, dst);
        }
        if bf & 0x2000 != 0 {
            dst.align_type = self.align_type;
            dst.spin_rate = self.spin_rate;
            set(0x2000, dst);
        }
        if bf & 0x4000 != 0 {
            dst.sticks_upon_collide = self.sticks_upon_collide;
            set(0x4000, dst);
        }
        if bf & 0x8000 != 0 {
            dst.can_land = self.can_land;
            set(0x8000, dst);
        }
        if bf & 0x10000 != 0 {
            dst.detonates_on_miss = self.detonates_on_miss;
            set(0x10000, dst);
        }
        if bf & 0x20000 != 0 {
            dst.detonation_proximity = self.detonation_proximity;
            dst.detonation_chance = self.detonation_chance;
            set(0x20000, dst);
        }
        true
    }
}

impl MissileTargetingParameters {
    /// `MissileTargetingParameters::CopyInto`: the target is either an object (bit 1: id, height
    /// adjustment, scale flag and offset; clears bit 2) or a position (bit 2; clears bit 1); the
    /// start is either the source object (bit `0x10`; clears 8) or a base position (bit 8;
    /// clears `0x10`). Bits 4, `0x80`, `0x200` and `0x800` bring their scalar; `0x100`, `0x40`
    /// and `0x400` carry only their bit. Always true.
    pub fn copy_into(&self, dst: &mut MissileTargetingParameters) -> bool {
        let bf = self.bitfield;
        if bf & 1 != 0 {
            dst.target_id = self.target_id;
            dst.target_height_adjustment = self.target_height_adjustment;
            dst.use_target_scale = self.use_target_scale;
            dst.target_offset = self.target_offset;
            dst.bitfield = dst.bitfield & !2 | 1;
        } else if bf & 2 != 0 {
            dst.target_pos = self.target_pos;
            dst.bitfield = dst.bitfield & !1 | 2;
        }
        if bf & 4 != 0 {
            dst.initial_velocity = self.initial_velocity;
            dst.bitfield |= 4;
        }
        if bf & 0x80 != 0 {
            dst.initial_pitch = self.initial_pitch;
            dst.bitfield |= 0x80;
        }
        if bf & 0x10 != 0 {
            dst.source_id = self.source_id;
            dst.source_height_adjustment = self.source_height_adjustment;
            dst.use_source_scale = self.use_source_scale;
            dst.source_offset = self.source_offset;
            dst.bitfield = dst.bitfield & !8 | 0x10;
        } else if bf & 8 != 0 {
            dst.base_pos = self.base_pos;
            dst.bitfield = dst.bitfield & !0x10 | 8;
        }
        if bf & 0x100 != 0 {
            dst.bitfield |= 0x100;
        }
        if bf & 0x200 != 0 {
            dst.missile_range = self.missile_range;
            dst.bitfield |= 0x200;
        }
        if bf & 0x40 != 0 {
            dst.bitfield |= 0x40;
        }
        if bf & 0x400 != 0 {
            dst.bitfield |= 0x400;
        }
        if bf & 0x800 != 0 {
            dst.missile_detonation_chance = self.missile_detonation_chance;
            dst.bitfield |= 0x800;
        }
        true
    }
}
