//! `gRadarControl`: the native side's call into the `RadarControl` singleton script for the
//! radar's ranges, which `gmRadarUI` reads for the music's groove.

use crate::vm::refs::{flags, Target};
use crate::vm::{tag, Context, FunctionId, Vm};

const SYSTEM: &str = "RadarControl";
/// `RadarControl::InqRadarRanges`.
const INQ_RADAR_RANGES: u32 = 0x00A9_0001;

/// `gRadarControl::CallIFun_InqRadarRanges(&indoor, &outdoor)`: the outdoor then the indoor
/// float, a zero return slot, the stack addresses of the two, the instance; the script writes
/// the ranges through the addresses. `(indoor, outdoor)`, starting from the given values.
pub fn inq_radar_ranges(vm: &mut Vm, indoor: f32, outdoor: f32) -> Option<(f32, f32)> {
    let pid = vm.program.package_id(SYSTEM)?;
    let instance = vm.singleton_package(pid);
    let mut ctx = Context::new(0, Some(instance));
    let outdoor_slot = ctx.sp;
    ctx.push(outdoor.to_bits(), tag::BOOL).ok()?;
    let indoor_slot = ctx.sp;
    ctx.push(indoor.to_bits(), tag::BOOL).ok()?;
    ctx.push(0, tag::INT).ok()?;
    for slot in [outdoor_slot, indoor_slot] {
        let blob = flags::KIND_VALUE | ((slot as u32 & 0xFFFF) << flags::OFFSET_SHIFT);
        let address = vm.refs.add(blob, Target::Ref(u32::MAX));
        ctx.push(address, tag::REF).ok()?;
    }
    ctx.push(instance, tag::REF).ok()?;
    let pkg = vm.package_id(instance).ok()?;
    vm.last_fault = None;
    vm.call_ifun(&mut ctx, pkg, FunctionId(INQ_RADAR_RANGES));
    let read = |ctx: &Context, slot| ctx.slot(slot).ok().map(|(v, _)| f32::from_bits(v));
    Some((read(&ctx, indoor_slot)?, read(&ctx, outdoor_slot)?))
}
