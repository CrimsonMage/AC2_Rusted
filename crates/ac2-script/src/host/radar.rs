//! The radar's script calls: `gRadarControl::CallIFun_InqRadarRanges` (the `RadarControl`
//! singleton's ranges, which `gmRadarUI` reads for the music's groove) and
//! `gmCWInterface::InqRadarLook` (an object's blip colour and shape, `gmRadarUI::AddObject`).

use super::{DispatchOutcome, HostError, ScriptHost};
use crate::native::{ntype, NativeObject};
use crate::vm::refs::{flags, Target};
use crate::vm::{tag, Context, FunctionId, InterpStatus, Vm};

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

/// `ggmCWInterface::CallIFun_InqRadarLook` (`0xB400C4`).
const INQ_RADAR_LOOK: u32 = 0x00B4_00C4;
/// `gmRadarUI::AddObject`: a blip with this shape is no blip (`info.shape != 0x4000000b`).
pub const NO_BLIP: u32 = 0x4000_000B;

/// What `gmCWInterface::InqRadarLook` gives an object for the radar: its colour and blip shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarLook {
    /// `RGBAColor` (r, g, b, a).
    pub color: [f32; 4],
    /// The blip shape, `0x40000001` (point) to `0x4000000A` (face).
    pub shape: u32,
}

impl ScriptHost {
    /// `gmCWInterface::InqRadarLook` on `id`'s process: `CallIFun_InqRadarLook(process, &color,
    /// &shape, &retval)`, the colour an `RGBAColor` native (`gRGBAColor`), the shape a word
    /// out parameter, the return slot 0. The outcome first: a call that stopped for data waits
    /// like any other, and its look is not read; the look is `None` when the call returns 0.
    pub fn inq_radar_look(
        &mut self,
        id: u64,
    ) -> Result<(DispatchOutcome, Option<RadarLook>), HostError> {
        let instance = self
            .processes
            .get(&id)
            .ok_or(HostError::UnknownObject(id))?
            .instance;
        let cid = self.next_context;
        self.next_context += 1;
        let mut ctx = Context::new(cid, Some(instance));
        ctx.can_yield = true;
        let bad = |_| HostError::BadArguments;
        // The out parameters, last deepest: the shape word, then the colour's reference.
        let shape_slot = ctx.sp;
        ctx.push(0, tag::BOOL).map_err(bad)?;
        let color = self
            .vm
            .refs
            .create_native(ntype::RGBA_COLOR, NativeObject::Color([0.0; 4]));
        let color_slot = ctx.sp;
        ctx.push(color, tag::REF).map_err(bad)?;
        let ret_slot = ctx.sp;
        ctx.push(0, tag::INT).map_err(bad)?;
        // Then the parameters' addresses (the shape, then the colour: colour is the first
        // parameter, so it sits nearest the instance), and the instance.
        for slot in [shape_slot, color_slot] {
            let blob = flags::KIND_VALUE | ((slot as u32 & 0xFFFF) << flags::OFFSET_SHIFT);
            let address = self.vm.refs.add(blob, Target::Ref(u32::MAX));
            ctx.push(address, tag::REF).map_err(bad)?;
        }
        ctx.push(instance, tag::REF).map_err(bad)?;
        self.vm.last_fault = None;
        let pkg = self.vm.package_id(instance).unwrap_or(u32::MAX);
        let status = self.vm.call_ifun(&mut ctx, pkg, FunctionId(INQ_RADAR_LOOK));
        let read = |ctx: &Context, slot| ctx.slot(slot).ok().map(|(v, _)| v);
        let retval = read(&ctx, ret_slot).unwrap_or(0);
        let shape = read(&ctx, shape_slot).unwrap_or(0);
        let rgba = read(&ctx, color_slot).and_then(|r| match self.vm.native(r) {
            Some(NativeObject::Color(c)) => Some(*c),
            _ => None,
        });
        let outcome = self.finish(id, ctx, status);
        let look = (outcome.status == InterpStatus::Terminated && retval != 0).then(|| RadarLook {
            color: rgba.unwrap_or([0.0; 4]),
            shape,
        });
        Ok((outcome, look))
    }
}
