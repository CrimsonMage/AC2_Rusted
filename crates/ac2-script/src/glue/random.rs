//! `gRandom` (`Random::RollDice`, `RollPercentile`, `RollGaussianDice`, `RollSkewedNumberEx`)
//! and `gRandomSelectionTable_Int` (`RandomSelectionTable<unsigned long>`: a weight tree laid
//! out like a heap, chosen without replacement).
//!
//! Every roll draws from [`Vm::random`]. Float arguments and results are the f32 bits in a slot;
//! retail tags the results 4 like ints.

use super::lists::{write_out, Kind};
use super::{Call, INT_RESULT};
use crate::native::{NativeObject, RandomSelectionTable};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x005F_0001, roll_percentile);
    efuns.register(0x005F_0002, roll_dice);
    efuns.register(0x005F_0003, roll_dice_int);
    efuns.register(0x005F_0004, roll_gaussian_dice);
    efuns.register(0x005F_0005, roll_skewed_number);
    efuns.register(0x005F_0006, roll_skewed_number_ex);
    efuns.register(0x0060_0001, |vm, ctx| add_entry(vm, ctx, true));
    efuns.register(0x0060_0002, |vm, ctx| add_entry(vm, ctx, false));
    efuns.register(0x0060_0003, initialize_weights);
    efuns.register(0x0060_0004, clear);
    efuns.register(0x0060_0005, done);
    efuns.register(0x0060_0006, start_over);
    efuns.register(0x0060_0007, choose);
}

fn f(ctx: &Context, n: usize) -> f32 {
    f32::from_bits(super::word(ctx, n))
}

fn ret_f32(ctx: &mut Context, n: usize, v: f32) -> InterpStatus {
    Call::new(ctx).ret(n, v.to_bits(), INT_RESULT);
    InterpStatus::Success
}

/// `Random::RollPercentile`: `1 - ftol(rand() * -100)`, 1..=100.
fn roll_percentile(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = 1 - (vm.random.rand() * -100.0) as i64 as i32;
    Call::new(ctx).ret(2, v as u32, INT_RESULT);
    InterpStatus::Success
}

/// `Random::RollDice(float, float)`.
fn roll_dice(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = vm.random.roll_dice_f32(f(ctx, 2), f(ctx, 3));
    ret_f32(ctx, 4, v)
}

/// `Random::RollDice(long, long)`.
fn roll_dice_int(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (lo, hi) = (super::word(ctx, 2) as i32, super::word(ctx, 3) as i32);
    let v = vm.random.roll_dice(lo, hi);
    Call::new(ctx).ret(4, v as u32, INT_RESULT);
    InterpStatus::Success
}

/// `Random::RollGaussianDice(mean, stddev)`: Leva's ratio-of-uniforms normal deviate.
pub(crate) fn gaussian(random: &mut ac2_core::Random, mean: f32, stddev: f32) -> f32 {
    loop {
        let u = random.rand() as f32;
        let v = ((f64::from(random.rand() as f32) - 0.5) * 1.7156) as f32;
        let x = f64::from(u) - 0.449871;
        let y = f64::from(v.abs()) + 0.386595;
        let q = (x * x + (y * 0.196 - x * 0.25472) * y) as f32;
        let accept = q < 0.27597
            || (q <= 0.27846 && {
                let u = f64::from(u);
                u.ln() * u * u * -4.0 >= f64::from(v) * f64::from(v)
            });
        if accept {
            return (f64::from(v) / f64::from(u) * f64::from(stddev) + f64::from(mean)) as f32;
        }
    }
}

fn roll_gaussian_dice(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = gaussian(&mut vm.random, f(ctx, 2), f(ctx, 3));
    ret_f32(ctx, 4, v)
}

/// `RollSkewedNumberEx(lower, upper, median, stddev)`: a Gaussian roll around `median`
/// (stddev 0 means half the distance from `median` to `upper`); after two rolls outside the
/// bounds, `median`.
fn skewed(random: &mut ac2_core::Random, lower: f32, upper: f32, median: f32, stddev: f32) -> f32 {
    let stddev = if stddev == 0.0 {
        ((f64::from(upper) - f64::from(median)).abs() * 0.5) as f32
    } else {
        stddev
    };
    for _ in 0..2 {
        let v = gaussian(random, median, stddev);
        if !(v < lower || upper < v) {
            return v;
        }
    }
    median
}

fn roll_skewed_number(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = skewed(&mut vm.random, f(ctx, 2), f(ctx, 3), f(ctx, 4), 0.0);
    ret_f32(ctx, 5, v)
}

fn roll_skewed_number_ex(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = skewed(&mut vm.random, f(ctx, 2), f(ctx, 3), f(ctx, 4), f(ctx, 5));
    ret_f32(ctx, 6, v)
}

/// Runs `g` on `this` as a `RandomSelectionTable_Int`.
fn with_table(
    vm: &mut Vm,
    ctx: &mut Context,
    g: impl FnOnce(&mut RandomSelectionTable),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::RandomSelectionTable(t)) => {
            g(t);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// Adds `w` to the subtree weights of `i`'s ancestors.
fn add_up(unused: &mut [(u32, u32, u32)], mut i: usize, w: u32) {
    while i != 0 {
        i = (i - 1) >> 1;
        unused[i].2 = unused[i].2.wrapping_add(w);
    }
}

/// `AddEntry(object, weight)` (`AddEntry_Internal`: append, then add the weight up the tree)
/// and `FastAddEntry` (append only; `InitializeWeights` fixes the tree later). Weights that
/// are not positive are ignored.
fn add_entry(vm: &mut Vm, ctx: &mut Context, update: bool) -> InterpStatus {
    let call = Call::new(ctx);
    let (object, weight) = (call.word(2), call.word(3));
    if weight as i32 <= 0 {
        return with_table(vm, ctx, |_| {});
    }
    with_table(vm, ctx, |t| {
        t.unused.push((object, weight, 0));
        if update {
            let i = t.unused.len() - 1;
            t.unused[i].2 = weight;
            add_up(&mut t.unused, i, weight);
        }
    })
}

/// `InitializeWeights_Internal`: each subtree weight is the node's weight plus its children's.
fn init_weights(unused: &mut [(u32, u32, u32)], i: usize) {
    unused[i].2 = unused[i].1;
    for child in [2 * i + 1, 2 * i + 2] {
        if child < unused.len() {
            init_weights(unused, child);
            unused[i].2 = unused[i].2.wrapping_add(unused[child].2);
        }
    }
}

fn initialize_weights(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_table(vm, ctx, |t| {
        if !t.unused.is_empty() {
            init_weights(&mut t.unused, 0);
        }
    })
}

fn clear(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_table(vm, ctx, |t| {
        t.unused.clear();
        t.used.clear();
    })
}

/// `Done`: no unused entries left (a byte result).
fn done(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let empty = match vm.native(this) {
        Some(NativeObject::RandomSelectionTable(t)) => t.unused.is_empty(),
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(2, u32::from(empty), INT_RESULT);
    InterpStatus::Success
}

/// `RandomSelectionTable::StartOver`: every used entry goes back through `AddEntry_Internal`.
fn start_over(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_table(vm, ctx, |t| {
        for (object, weight, _) in std::mem::take(&mut t.used) {
            t.unused.push((object, weight, weight));
            let i = t.unused.len() - 1;
            add_up(&mut t.unused, i, weight);
        }
    })
}

/// `RandomSelectionTable::MoveToUsedList(i)`: the last entry takes `i`'s place.
fn move_to_used(t: &mut RandomSelectionTable, i: usize) {
    t.used.push(t.unused[i]);
    let last = t.unused.len() - 1;
    let last_weight = t.unused[last].1;
    let diff = last_weight.wrapping_sub(t.unused[i].1);
    let mut j = last;
    while j != 0 {
        j = (j - 1) >> 1;
        t.unused[j].2 = t.unused[j].2.wrapping_sub(last_weight);
    }
    t.unused[i].1 = last_weight;
    t.unused[i].0 = t.unused[last].0;
    if diff != 0 {
        t.unused[i].2 = t.unused[i].2.wrapping_add(diff);
        add_up(&mut t.unused, i, diff);
    }
    t.unused.pop();
}

/// `RandomSelectionTable::Choose`: roll in [1, total weight], walk down the tree, move the
/// entry to the used list.
fn pick(t: &mut RandomSelectionTable, random: &mut ac2_core::Random) -> Option<u32> {
    let root = *t.unused.first()?;
    let mut sum = root.1;
    let mut at = 0usize;
    let roll = random.roll_dice(1, root.2 as i32) as u32;
    while sum < roll {
        let left = 2 * at + 1;
        let left_total = t.unused.get(left).map_or(0, |e| e.2).wrapping_add(sum);
        sum = if left_total < roll {
            at = left + 1;
            left_total.wrapping_add(t.unused.get(at).map_or(0, |e| e.1))
        } else {
            at = left;
            sum.wrapping_add(t.unused.get(at).map_or(0, |e| e.1))
        };
        if at >= t.unused.len() {
            return None;
        }
    }
    let object = t.unused[at].0;
    move_to_used(t, at);
    Some(object)
}

/// `gRandomSelectionTable_Int::CallEFun_Choose(out value)`.
fn choose(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, out) = (call.this(), call.word(2));
    let mut random = vm.random.clone();
    let picked = match vm.native_mut(this) {
        Some(NativeObject::RandomSelectionTable(t)) => pick(t, &mut random),
        _ => return vm.bad_this(this),
    };
    vm.random = random;
    write_out(vm, ctx, out, picked.map(u64::from), Kind::A);
    Call::new(ctx).ret(3, u32::from(picked.is_some()), INT_RESULT);
    InterpStatus::Success
}
