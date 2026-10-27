//! `gTurbineGunUtils` (`gturbinegunutils.cpp`): the world name the HUD scripts show.

use super::Call;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x200A_0001, get_world_name);
}

/// `gTurbineGunUtils::CallEFun_GetWorldName`: a new string of the world's name (empty before the
/// world-name notice arrives), returned in the slot below `this`.
fn get_world_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let name: Vec<u16> = vm.world_name.encode_utf16().collect();
    let reference = vm.new_wstring(name);
    Call::new(ctx).ret(2, reference, tag::REF);
    InterpStatus::Success
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::glue::register_pure;
    use crate::vm::{tag, Context, EFuns, InterpStatus, NoData, Program, Vm};

    fn vm() -> Vm {
        let mut efuns = EFuns::new();
        register_pure(&mut efuns);
        let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
        Vm::new(Arc::new(program), Box::new(NoData), efuns)
    }

    #[test]
    fn the_efun_returns_the_world_name_as_a_string() {
        let mut vm = vm();
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        ctx.push(0, tag::INT).unwrap();
        ctx.push(u32::MAX, tag::REF).unwrap();
        vm.world_name = "Legions".to_owned();
        assert_eq!(vm.run_efun(0x200A_0001, &mut ctx), InterpStatus::Success);
        assert_eq!(ctx.tags[0], tag::REF);
        let text: Vec<u16> = "Legions".encode_utf16().collect();
        assert_eq!(vm.wstring(ctx.data[0]).unwrap(), text.as_slice());
    }
}
