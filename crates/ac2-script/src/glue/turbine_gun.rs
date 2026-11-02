//! `gTurbineGunUtils` (`gturbinegunutils.cpp`): the world name and the `@World` suffix the HUD
//! scripts read, from `TurbineGun`'s names (`S_pTurbineGun`; empty without one).

use super::Call;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x200A_0001, get_world_name);
    efuns.register(0x200A_0002, get_world_suffix);
}

/// A new string of `text` (`CopyNativeObject` type 0x76, `CreateNativeObject`), returned in the
/// slot below `this` with tag 1.
fn return_string(vm: &mut Vm, ctx: &mut Context, text: &str) -> InterpStatus {
    let wide: Vec<u16> = text.encode_utf16().collect();
    let reference = vm.new_wstring(wide);
    Call::new(ctx).ret(2, reference, tag::REF);
    InterpStatus::Success
}

/// `gTurbineGunUtils::CallEFun_GetWorldName` (admin 0x0046EA20, Legions 0x0046E8C0, EFun
/// 0x200A0001): `TurbineGunUtils::GetWorldName()` = `S_pTurbineGun->m_worldName`, empty while
/// `S_pTurbineGun` is NULL or before the world-name notice.
fn get_world_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    return_string(vm, ctx, &ac2_gun::turbine_gun_utils::get_world_name())
}

/// `gTurbineGunUtils::CallEFun_GetWorldSuffix` (admin 0x0046EAA0, Legions 0x0046E940, EFun
/// 0x200A0002): `TurbineGunUtils::GetWorldSuffix()`, `@World`.
fn get_world_suffix(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    return_string(vm, ctx, &ac2_gun::turbine_gun_utils::get_world_suffix())
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

    fn call(vm: &mut Vm, efun: u32) -> Vec<u16> {
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        ctx.push(0, tag::INT).unwrap();
        ctx.push(u32::MAX, tag::REF).unwrap();
        assert_eq!(vm.run_efun(efun, &mut ctx), InterpStatus::Success);
        assert_eq!(ctx.tags[0], tag::REF);
        vm.wstring(ctx.data[0]).unwrap().to_vec()
    }

    #[test]
    fn the_efuns_answer_from_the_turbine_gun_names() {
        let mut vm = vm();
        // No TurbineGun yet: S_pTurbineGun is NULL, both are the empty string.
        assert!(call(&mut vm, 0x200A_0001).is_empty());
        assert!(call(&mut vm, 0x200A_0002).is_empty());
        let gun = ac2_gun::TurbineGun::new(0x0409);
        gun.set_world_name("Legions");
        let wide = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        assert_eq!(call(&mut vm, 0x200A_0001), wide("Legions"));
        assert_eq!(call(&mut vm, 0x200A_0002), wide("@Legions"));
    }
}
