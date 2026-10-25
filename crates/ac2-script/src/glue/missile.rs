//! `gMissileParameters`: the `MissileParameters` setters (`MissileParameters::Set*` in
//! `physobj/missile/MissileParameters.cpp`): each stores its arguments and sets its group's
//! pack bit.

use super::Call;
use crate::native::{MissileParameters, NativeObject};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

/// A setter: the fields it writes, each from an argument word (`Some(n)`, word n) or a
/// constant (`None` with the value), and the pack bit it sets.
struct Setter {
    id: u32,
    bit: u32,
    writes: &'static [(&'static str, Arg)],
}

#[derive(Clone, Copy)]
enum Arg {
    Word(usize),
    Const(u32),
}

use Arg::{Const, Word};

const ACCEL: [&str; 6] = [
    "max_forward_accel",
    "max_decel",
    "max_left_accel",
    "max_right_accel",
    "max_upward_accel",
    "max_downward_accel",
];

const SETTERS: &[Setter] = &[
    // SetSpeedLimits(max, min)
    Setter {
        id: 0x01,
        bit: 0x1,
        writes: &[("max_velocity", Word(2)), ("min_velocity", Word(3))],
    },
    // SetMaxAccelerationUniform(max)
    Setter {
        id: 0x02,
        bit: 0x2,
        writes: &[
            (ACCEL[0], Word(2)),
            (ACCEL[1], Word(2)),
            (ACCEL[2], Word(2)),
            (ACCEL[3], Word(2)),
            (ACCEL[4], Word(2)),
            (ACCEL[5], Word(2)),
        ],
    },
    // SetMaxAcceleration(forward, decel, left, right, up, down)
    Setter {
        id: 0x03,
        bit: 0x2,
        writes: &[
            (ACCEL[0], Word(2)),
            (ACCEL[1], Word(3)),
            (ACCEL[2], Word(4)),
            (ACCEL[3], Word(5)),
            (ACCEL[4], Word(6)),
            (ACCEL[5], Word(7)),
        ],
    },
    // SetTotalFuel(fuel, report_when_out)
    Setter {
        id: 0x04,
        bit: 0x4,
        writes: &[
            ("total_fuel", Word(2)),
            ("uses_fuel", Const(1)),
            ("report_if_no_fuel", Word(3)),
        ],
    },
    // SetUseNoFuel
    Setter {
        id: 0x05,
        bit: 0x4,
        writes: &[("uses_fuel", Const(0))],
    },
    // SetTotalLifespan(lifespan)
    Setter {
        id: 0x06,
        bit: 0x8,
        writes: &[
            ("missile_lifespan", Word(2)),
            ("missile_has_lifespan", Const(1)),
        ],
    },
    // SetHasNoLifespan
    Setter {
        id: 0x07,
        bit: 0x8,
        writes: &[("missile_has_lifespan", Const(0))],
    },
    // SetDragFactor(drag)
    Setter {
        id: 0x08,
        bit: 0x10,
        writes: &[("drag", Word(2))],
    },
    // SetAffectedByGravity(gravity)
    Setter {
        id: 0x09,
        bit: 0x20,
        writes: &[("affected_by_gravity", Word(2))],
    },
    // SetFollowGroundType(follow, height)
    Setter {
        id: 0x0A,
        bit: 0x80,
        writes: &[
            ("ground_follow_type", Word(2)),
            ("height_above_ground", Word(3)),
        ],
    },
    // SetConserveFuel(conserve, min_catchup_speed, consider_remaining_time)
    Setter {
        id: 0x0B,
        bit: 0x1000,
        writes: &[
            ("conserve_fuel", Word(2)),
            ("min_catchup_speed", Word(3)),
            ("consider_remaining_time", Word(4)),
        ],
    },
    // SetMaintainDistance(distance)
    Setter {
        id: 0x0C,
        bit: 0x200,
        writes: &[("maintain_distance", Word(2)), ("use_maint_dist", Const(1))],
    },
    // SetMissileAsHoming
    Setter {
        id: 0x0D,
        bit: 0x200,
        writes: &[("use_maint_dist", Const(0))],
    },
    // SetMissileReaction(react_time_check, min_react_time, lead_target)
    Setter {
        id: 0x0E,
        bit: 0x400,
        writes: &[
            ("reaction_time_check", Word(2)),
            ("min_reaction_time", Word(3)),
            ("correct_for_target_velocity", Word(4)),
        ],
    },
    // SetTurnMethod(turn)
    Setter {
        id: 0x0F,
        bit: 0x800,
        writes: &[("tight_turn_method", Word(2))],
    },
    // UseEnvironmentalMaximumVelocities(use) (retail sets the conserve-fuel bit, 0x1000)
    Setter {
        id: 0x10,
        bit: 0x1000,
        writes: &[("use_env_max_velocities", Word(2))],
    },
    // SetAlignToPath: SetAlignType(0, 0.0)
    Setter {
        id: 0x11,
        bit: 0x2000,
        writes: &[("align_type", Const(0)), ("spin_rate", Const(0))],
    },
    // SetSpinRate(rate): SetAlignType(1, rate)
    Setter {
        id: 0x12,
        bit: 0x2000,
        writes: &[("align_type", Const(1)), ("spin_rate", Word(2))],
    },
    // SetToNoAlign: SetAlignType(2, 0.0)
    Setter {
        id: 0x13,
        bit: 0x2000,
        writes: &[("align_type", Const(2)), ("spin_rate", Const(0))],
    },
    // SetMissileSticky(sticky)
    Setter {
        id: 0x14,
        bit: 0x4000,
        writes: &[("sticks_upon_collide", Word(2))],
    },
    // SetCanLand(land)
    Setter {
        id: 0x15,
        bit: 0x8000,
        writes: &[("can_land", Word(2))],
    },
    // SetDetonatesOnMiss(detonates)
    Setter {
        id: 0x16,
        bit: 0x10000,
        writes: &[("detonates_on_miss", Word(2))],
    },
    // SetDetonationProximity(proximity, chance)
    Setter {
        id: 0x17,
        bit: 0x20000,
        writes: &[
            ("detonation_proximity", Word(2)),
            ("detonation_chance", Word(3)),
        ],
    },
];

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        if !name.starts_with("gMissileParameters::") {
            continue;
        }
        if SETTERS.iter().any(|s| 0x0046_0000 | s.id == id) {
            efuns.register(id, set);
        }
    }
}

/// Applies the setter of the running efun ([`Vm::current_efun`]).
fn set(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = vm.current_efun() & 0xFFFF;
    let Some(setter) = SETTERS.iter().find(|s| s.id == id) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let call = Call::new(ctx);
    let this = call.this();
    let values: Vec<(&str, u32)> = setter
        .writes
        .iter()
        .map(|(name, arg)| {
            let v = match *arg {
                Word(n) => call.word(n),
                Const(c) => c,
            };
            (*name, v)
        })
        .collect();
    match vm.native_mut(this) {
        Some(NativeObject::MissileParameters(p)) => {
            apply(p, &values, setter.bit);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

fn apply(p: &mut MissileParameters, values: &[(&str, u32)], bit: u32) {
    for (name, v) in values {
        p.set(name, *v);
    }
    p.bitfield |= bit;
}
