//! The craft adaptors the scripts fill for the craft queries: `gRecipeAdaptor` (0xAA) and
//! `gIngredientAdaptor` (0xA5), over ac2-weenie's [`Recipe`] and [`Ingredient`]. The efuns are
//! the setters and getters the recipe and ingredient builds call (`grecipeadaptor.cpp.c`,
//! `gingredientadaptor.cpp.c`); each reads its `this` at the frame's first slot and its value
//! after it, as the other natives do. Only the efuns the Legions library binds are here.

use ac2_weenie::{Ingredient, Recipe};

use super::ui_natives::{get, native, set, text_into, text_out, with, Native};
use super::{word64, Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{Context, EFuns, InterpStatus, Vm};

impl Native for Recipe {
    fn of(n: &NativeObject) -> Option<&Self> {
        match n {
            NativeObject::Recipe(x) => Some(x),
            _ => None,
        }
    }
    fn of_mut(n: &mut NativeObject) -> Option<&mut Self> {
        match n {
            NativeObject::Recipe(x) => Some(x),
            _ => None,
        }
    }
}

impl Native for Ingredient {
    fn of(n: &NativeObject) -> Option<&Self> {
        match n {
            NativeObject::Ingredient(x) => Some(x),
            _ => None,
        }
    }
    fn of_mut(n: &mut NativeObject) -> Option<&mut Self> {
        match n {
            NativeObject::Ingredient(x) => Some(x),
            _ => None,
        }
    }
}

/// A boolean argument: its low byte.
fn flag(v: u32) -> bool {
    v & 0xFF != 0
}

/// A 64-bit getter (`gRecipeAdaptor::CallEFun_GetTimeUseAgain`): the low word goes to the return
/// slot's deeper word (3), the high word to the one above it (2), as `word64` reads them.
fn get64<T: Native>(vm: &mut Vm, ctx: &mut Context, f: fn(&T) -> u64) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(v) = native::<T>(vm, this).map(f) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(3, v as u32, INT_RESULT);
    Call::new(ctx).ret(2, (v >> 32) as u32, INT_RESULT);
    InterpStatus::Success
}

pub(super) fn register(efuns: &mut EFuns) {
    // gRecipeAdaptor.
    efuns.register(0x00AA_0003, |vm, c| text_out(vm, c, |r: &Recipe| &r.name));
    efuns.register(0x00AA_0004, |vm, c| {
        text_into(vm, c, |r: &mut Recipe| &mut r.name)
    });
    efuns.register(0x00AA_0005, |vm, c| {
        text_out(vm, c, |r: &Recipe| &r.description)
    });
    efuns.register(0x00AA_0006, |vm, c| {
        text_into(vm, c, |r: &mut Recipe| &mut r.description)
    });
    efuns.register(0x00AA_0007, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.charges = v as i32)
    });
    efuns.register(0x00AA_000D, |vm, c| {
        let v = word64(c, 3);
        with::<Recipe>(vm, c, |r, _| r.time_use_again = v)
    });
    efuns.register(0x00AA_000F, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.cost = v as i32)
    });
    efuns.register(0x00AA_0011, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.shown = flag(v))
    });
    efuns.register(0x00AA_0013, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.dynamic_cost = flag(v))
    });
    efuns.register(0x00AA_0015, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.icon_did = v)
    });
    efuns.register(0x00AA_0017, |vm, c| get(vm, c, |r: &Recipe| r.did));
    efuns.register(0x00AA_0018, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.did = v)
    });
    efuns.register(0x00AA_0019, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.min_spinner = v)
    });
    efuns.register(0x00AA_001A, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.max_spinner = v)
    });
    efuns.register(0x00AA_001C, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.last_spinner = v)
    });
    efuns.register(0x00AA_001E, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.craft_skill_category = v)
    });
    efuns.register(0x00AA_0021, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.craft_skill_did = v)
    });
    efuns.register(0x00AA_0023, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.difficulty = v)
    });
    efuns.register(0x00AA_0024, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.dynamic_difficulty = flag(v))
    });
    efuns.register(0x00AA_0025, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.auto_populate = flag(v))
    });
    efuns.register(0x00AA_0026, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.max_ingredients = v)
    });
    efuns.register(0x00AA_0027, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| {
            r.has_difficulty_minus_skill_cutoff = true;
            r.difficulty_minus_skill_cutoff = v as i32;
        })
    });
    efuns.register(0x00AA_0029, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.read_only_spinner = flag(v))
    });
    efuns.register(0x00AA_002A, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.has_name_coloring = flag(v))
    });
    efuns.register(0x00AA_002B, |vm, c| {
        set(vm, c, |r: &mut Recipe, v| r.name_color = v)
    });
    // The recipe getters (`gRecipeAdaptor::CallEFun_Get*`): each returns the field its setter
    // writes, as the stored word.
    efuns.register(0x00AA_0008, |vm, c| {
        get(vm, c, |r: &Recipe| r.charges as u32)
    });
    efuns.register(0x00AA_000E, |vm, c| {
        get64(vm, c, |r: &Recipe| r.time_use_again)
    });
    efuns.register(0x00AA_0012, |vm, c| {
        get(vm, c, |r: &Recipe| u32::from(r.shown))
    });
    efuns.register(0x00AA_0014, |vm, c| {
        get(vm, c, |r: &Recipe| u32::from(r.dynamic_cost))
    });
    efuns.register(0x00AA_0016, |vm, c| get(vm, c, |r: &Recipe| r.icon_did));
    efuns.register(0x00AA_0020, |vm, c| {
        get(vm, c, |r: &Recipe| r.craft_skill_category)
    });
    efuns.register(0x00AA_0022, |vm, c| {
        get(vm, c, |r: &Recipe| r.craft_skill_did)
    });
    // gIngredientAdaptor.
    // `gIngredientAdaptor::CallEFun_SetID` (admin 0x4713D0: this at fp-4, the value at fp-8). The
    // Legions `CheckIngredient` calls it as 0x00A50002; the efun table lists only 6..9.
    efuns.register(0x00A5_0002, |vm, c| {
        set(vm, c, |i: &mut Ingredient, v| i.id = v)
    });
    efuns.register(0x00A5_0006, |vm, c| {
        text_into(vm, c, |i: &mut Ingredient| &mut i.description)
    });
    efuns.register(0x00A5_0007, |vm, c| {
        set(vm, c, |i: &mut Ingredient, v| i.did_recipe = v)
    });
    efuns.register(0x00A5_0008, |vm, c| {
        get(vm, c, |i: &Ingredient| i.did_recipe)
    });
    efuns.register(0x00A5_0009, |vm, c| {
        get(vm, c, |i: &Ingredient| u32::from(i.auto_populate))
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::native::ntype;
    use crate::vm::{op, NoData, Program};

    fn vm() -> Vm {
        let mut efuns = EFuns::new();
        super::super::register_pure(&mut efuns);
        let program = Program::from_parts(vec![op::TERMINATE], Vec::new(), Vec::new());
        Vm::new(Arc::new(program), Box::new(NoData), efuns)
    }

    /// Calls efun `id` with `returns` zeroed return slots (deepest first) and `this`; gives back
    /// the return slots' values, deepest first.
    fn call(vm: &mut Vm, id: u32, returns: usize, this: u32) -> Vec<u32> {
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        for _ in 0..returns {
            ctx.push(0, crate::vm::tag::INT).unwrap();
        }
        ctx.push(this, crate::vm::tag::REF).unwrap();
        vm.run_efun(id, &mut ctx);
        (0..returns).map(|i| ctx.data[i]).collect()
    }

    /// Each recipe getter returns the field its setter writes; the time is two words.
    #[test]
    fn recipe_getters_return_the_fields_the_setters_write() {
        let mut vm = vm();
        let recipe = Recipe {
            charges: 3,
            time_use_again: 0x0000_0001_8000_0002,
            shown: true,
            dynamic_cost: true,
            icon_did: 0x0600_1234,
            craft_skill_category: 4,
            craft_skill_did: 0x6F00_28D7,
            did: 0x6F00_1F0C,
            ..Recipe::default()
        };
        let this = vm.refs.create_native(
            ntype::RECIPE_ADAPTOR,
            NativeObject::Recipe(Box::new(recipe)),
        );
        assert_eq!(call(&mut vm, 0x00AA_0017, 1, this), vec![0x6F00_1F0C]);
        assert_eq!(call(&mut vm, 0x00AA_0008, 1, this), vec![3]);
        assert_eq!(call(&mut vm, 0x00AA_0012, 1, this), vec![1]);
        assert_eq!(call(&mut vm, 0x00AA_0014, 1, this), vec![1]);
        assert_eq!(call(&mut vm, 0x00AA_0016, 1, this), vec![0x0600_1234]);
        assert_eq!(call(&mut vm, 0x00AA_0020, 1, this), vec![4]);
        assert_eq!(call(&mut vm, 0x00AA_0022, 1, this), vec![0x6F00_28D7]);
        // The low word is the deeper return slot, the high word the one above it.
        assert_eq!(call(&mut vm, 0x00AA_000E, 2, this), vec![0x8000_0002, 0x1]);
    }

    /// Calls setter efun `id` with `value` (the argument sits below `this`, as `set` reads it).
    fn call_set(vm: &mut Vm, id: u32, value: u32, this: u32) {
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        ctx.push(value, crate::vm::tag::INT).unwrap();
        ctx.push(this, crate::vm::tag::REF).unwrap();
        vm.run_efun(id, &mut ctx);
    }

    /// `RecipeAdaptor::SetDifficultyMinusSkillCutoff` (Legions 0x005CD250): the cutoff is stored
    /// and its flag set, so the recipe reports it.
    #[test]
    fn set_difficulty_minus_skill_cutoff_stores_the_cutoff_and_sets_its_flag() {
        let mut vm = vm();
        let this = vm
            .refs
            .create_native(ntype::RECIPE_ADAPTOR, NativeObject::Recipe(Box::default()));
        assert!(
            !native::<Recipe>(&vm, this)
                .unwrap()
                .has_difficulty_minus_skill_cutoff
        );
        call_set(&mut vm, 0x00AA_0027, (-3i32) as u32, this);
        let recipe = native::<Recipe>(&vm, this).unwrap();
        assert!(recipe.has_difficulty_minus_skill_cutoff);
        assert_eq!(recipe.difficulty_minus_skill_cutoff, -3);
    }
}
