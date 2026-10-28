//! `gAppearanceTableAdaptor` (the adaptor over `AppearanceTable`, `AppearanceTableAdaptor.cpp`):
//! the appearance keys' modifier intensities, readable from scripts.

use super::{out_native, Call};
use crate::native::{NativeObject, ScriptList};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x000E_0002, inq_appearance_key_mod_list);
}

/// `gAppearanceTableAdaptor::CallEFun_InqAppearanceKeyModList(did, key, list)` ->
/// `AppearanceTableAdaptor::InqAppearanceKeyModList`: the `AppearanceTable` (DB type 0x40000022)
/// `did` is loaded and the script's list (an `AList` of floats, copied in and back as
/// `gCharacterGenSystem::AListToValueList` / `ValueListToAList` do) is flushed and filled with the
/// intensities the table and its base tables hold for `key`, ascending. The answer is 1; it is
/// 0, the list untouched, when the table is missing. A cache-only host loads the table on its
/// worker and the call runs again.
fn inq_appearance_key_mod_list(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (did, key, list) = {
        let call = Call::new(ctx);
        (call.word(2), call.word(3), call.word(4))
    };
    let target = out_native(vm, ctx, list);
    let mods = vm.data().appearance_key_mods(did, key);
    if let Some(mods) = &mods {
        match vm.native_mut(target) {
            Some(NativeObject::List(l)) => {
                *l = ScriptList {
                    wide: false,
                    items: mods.iter().map(|v| u64::from(v.to_bits())).collect(),
                };
            }
            _ => return vm.bad_this(target),
        }
    }
    Call::new(ctx).ret(5, u32::from(mods.is_some()), tag::INT);
    InterpStatus::Success
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::host::ScriptHost;
    use crate::native::ntype;
    use crate::vm::{Program, ScriptData};

    /// A table `0x20000001` holding the intensities 0.5 and 0.25 (already ascending) for key 7.
    struct Tables;

    impl ScriptData for Tables {
        fn wstate(&self, _did: u32) -> Option<crate::vm::StateRef> {
            None
        }

        fn appearance_key_mods(&self, did: u32, key: u32) -> Option<Arc<Vec<f32>>> {
            (did == 0x2000_0001).then(|| Arc::new(if key == 7 { vec![0.25, 0.5] } else { vec![] }))
        }
    }

    fn inq(did: u32, key: u32) -> (u32, Vec<f32>) {
        let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
        let mut host = ScriptHost::new(Arc::new(program), Box::new(Tables));
        let vm = host.vm_mut();
        let list = vm.refs.create_native(
            ntype::A_LIST,
            NativeObject::List(ScriptList {
                wide: false,
                items: vec![9],
            }),
        );
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        for _ in 0..2 {
            ctx.push(0, tag::INT).unwrap();
        }
        ctx.push(list, tag::REF).unwrap();
        ctx.push(key, tag::INT).unwrap();
        ctx.push(did, tag::INT).unwrap();
        ctx.push(0, tag::REF).unwrap();
        let fp = ctx.sp;
        assert_eq!(vm.run_efun(0x000E_0002, &mut ctx), InterpStatus::Success);
        let answer = ctx.data[fp - 5];
        let Some(NativeObject::List(l)) = vm.native(list) else {
            panic!("list");
        };
        (
            answer,
            l.items.iter().map(|v| f32::from_bits(*v as u32)).collect(),
        )
    }

    #[test]
    fn the_list_takes_the_keys_intensities() {
        assert_eq!(inq(0x2000_0001, 7), (1, vec![0.25, 0.5]));
        // A key the table lacks empties the list (the adaptor flushes it first).
        assert_eq!(inq(0x2000_0001, 8), (1, vec![]));
    }

    #[test]
    fn a_missing_table_answers_zero_and_leaves_the_list() {
        assert_eq!(inq(0x2000_0002, 7), (0, vec![f32::from_bits(9)]));
    }
}
