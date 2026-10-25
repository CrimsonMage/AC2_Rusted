//! The natives scripts fill for store, craft and damage-text notices: `gUIShop` (0x380A),
//! `gUIShopSale` (0x380B), `gCraftSkillAdaptor` (0x33A5), `gDamageTextBlob` (0x97) and the
//! `gRGBAColor` (0x64) a damage text takes, over ac2-weenie's plain types. Every wrapper
//! crashes on an undefined `this`; 64-bit values come high word first.
//! `gUIShopSale::SetItemIconDesc` (an `IconDesc`) is not bound.

use ac2_net::protocol::StringInfo;
use ac2_weenie::{CraftSkill, DamageText, Shop, ShopSale};

use super::{word, word64, Call, INT_RESULT};
use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};
use crate::wire::{string_info_from_net, string_info_to_net};

pub(super) fn register(efuns: &mut EFuns) {
    // gUIShop.
    efuns.register(0x380A_0001, |vm, c| {
        text_into(vm, c, |s: &mut Shop| &mut s.name)
    });
    efuns.register(0x380A_0002, |vm, c| {
        text_into(vm, c, |s: &mut Shop| &mut s.desc)
    });
    efuns.register(0x380A_000A, |vm, c| {
        text_into(vm, c, |s: &mut Shop| &mut s.sorter)
    });
    efuns.register(0x380A_0003, |vm, c| {
        set(vm, c, |s: &mut Shop, v| s.portrait = v)
    });
    efuns.register(0x380A_0007, |vm, c| {
        set(vm, c, |s: &mut Shop, v| s.position = v as i32)
    });
    efuns.register(0x380A_0008, |vm, c| {
        set(vm, c, |s: &mut Shop, v| s.store_size = v as i32)
    });
    efuns.register(0x380A_0009, |vm, c| set(vm, c, |s: &mut Shop, v| s.id = v));
    efuns.register(0x380A_0004, add_sale);
    // gUIShopSale.
    efuns.register(0x380B_0001, |vm, c| {
        let id = word64(c, 3);
        with::<ShopSale>(vm, c, |s, _| s.item = id)
    });
    efuns.register(0x380B_0003, |vm, c| {
        text_into(vm, c, |s: &mut ShopSale| &mut s.item_desc)
    });
    efuns.register(0x380B_0008, |vm, c| {
        text_into(vm, c, |s: &mut ShopSale| &mut s.cost_desc)
    });
    efuns.register(0x380B_0004, |vm, c| {
        set(vm, c, |s: &mut ShopSale, v| s.cost_icon = v)
    });
    efuns.register(0x380B_0005, |vm, c| {
        set(vm, c, |s: &mut ShopSale, v| {
            s.cost_quantity = f32::from_bits(v)
        })
    });
    efuns.register(0x380B_0006, |vm, c| {
        set(vm, c, |s: &mut ShopSale, v| s.stock = v as i32)
    });
    efuns.register(0x380B_0009, |vm, c| {
        set(vm, c, |s: &mut ShopSale, v| s.ordinal = v)
    });
    efuns.register(0x380B_000A, |vm, c| {
        with::<ShopSale>(vm, c, |s, _| s.status = 1)
    });
    efuns.register(0x380B_000B, |vm, c| {
        with::<ShopSale>(vm, c, |s, _| s.status = 2)
    });
    efuns.register(0x380B_000C, |vm, c| {
        with::<ShopSale>(vm, c, |s, _| s.status = 3)
    });
    efuns.register(0x380B_000E, |vm, c| {
        set(vm, c, |s: &mut ShopSale, v| s.icon_did = v)
    });
    efuns.register(0x380B_0010, |vm, c| {
        set(vm, c, |s: &mut ShopSale, v| s.item_did = v)
    });
    // gCraftSkillAdaptor.
    efuns.register(0x33A5_0005, |vm, c| {
        text_out(vm, c, |s: &CraftSkill| &s.name)
    });
    efuns.register(0x33A5_0006, |vm, c| {
        text_into(vm, c, |s: &mut CraftSkill| &mut s.name)
    });
    efuns.register(0x33A5_0007, |vm, c| {
        text_out(vm, c, |s: &CraftSkill| &s.description)
    });
    efuns.register(0x33A5_0008, |vm, c| {
        text_into(vm, c, |s: &mut CraftSkill| &mut s.description)
    });
    efuns.register(0x33A5_0009, |vm, c| get(vm, c, |s: &CraftSkill| s.did));
    efuns.register(0x33A5_000A, |vm, c| {
        set(vm, c, |s: &mut CraftSkill, v| s.did = v)
    });
    efuns.register(0x33A5_000B, |vm, c| get(vm, c, |s: &CraftSkill| s.level));
    efuns.register(0x33A5_000C, |vm, c| {
        set(vm, c, |s: &mut CraftSkill, v| s.level = v)
    });
    efuns.register(0x33A5_000D, |vm, c| get(vm, c, |s: &CraftSkill| s.icon));
    efuns.register(0x33A5_000E, |vm, c| {
        set(vm, c, |s: &mut CraftSkill, v| s.icon = v)
    });
    efuns.register(0x33A5_000F, get_cost);
    efuns.register(0x33A5_0010, |vm, c| {
        let cost = word64(c, 3);
        with::<CraftSkill>(vm, c, |s, _| s.cost_to_raise = cost)
    });
    efuns.register(0x33A5_0011, |vm, c| {
        get(vm, c, |s: &CraftSkill| s.raw_level)
    });
    efuns.register(0x33A5_0012, |vm, c| {
        set(vm, c, |s: &mut CraftSkill, v| s.raw_level = v)
    });
    efuns.register(0x33A5_0013, |vm, c| {
        get(vm, c, |s: &CraftSkill| s.xp_mod.to_bits())
    });
    efuns.register(0x33A5_0014, |vm, c| {
        set(vm, c, |s: &mut CraftSkill, v| s.xp_mod = f32::from_bits(v))
    });
    // gDamageTextBlob.
    efuns.register(0x0097_0001, |vm, c| {
        let text = String::from_utf16_lossy(vm.wstring(word(c, 2)).unwrap_or(&[]));
        with::<DamageText>(vm, c, |d, _| d.text = text)
    });
    efuns.register(0x0097_0002, |vm, c| {
        let this = Call::new(c).this();
        let Some(text) = native::<DamageText>(vm, this).map(|d| d.text.clone()) else {
            return vm.bad_this(this);
        };
        let r = vm.new_wstring(text.encode_utf16().collect());
        Call::new(c).ret(2, r, tag::REF);
        InterpStatus::Success
    });
    efuns.register(0x0097_0003, |vm, c| {
        let color = match vm.native(word(c, 2)) {
            Some(NativeObject::Color(rgba)) => *rgba,
            _ => [0.0; 4],
        };
        with::<DamageText>(vm, c, |d, _| d.color = color)
    });
    efuns.register(0x0097_0004, |vm, c| {
        let this = Call::new(c).this();
        let Some(color) = native::<DamageText>(vm, this).map(|d| d.color) else {
            return vm.bad_this(this);
        };
        let r = vm
            .refs
            .create_native(ntype::RGBA_COLOR, NativeObject::Color(color));
        Call::new(c).ret(2, r, tag::REF);
        InterpStatus::Success
    });
    efuns.register(0x0097_0005, |vm, c| {
        set(vm, c, |d: &mut DamageText, v| {
            d.duration = f32::from_bits(v)
        })
    });
    efuns.register(0x0097_0006, |vm, c| {
        get(vm, c, |d: &DamageText| d.duration.to_bits())
    });
    efuns.register(0x0097_0007, |vm, c| {
        set(vm, c, |d: &mut DamageText, v| d.scale = f32::from_bits(v))
    });
    efuns.register(0x0097_0008, |vm, c| {
        get(vm, c, |d: &DamageText| d.scale.to_bits())
    });
    // gRGBAColor.
    efuns.register(0x0064_0001, |vm, c| {
        let rgba = [2, 3, 4, 5].map(|n| f32::from_bits(word(c, n)));
        with_color(vm, c, |col| *col = rgba)
    });
    efuns.register(0x0064_0002, |vm, c| {
        // `Attenuate(f)`: r, g and b scaled.
        let f = f64::from(f32::from_bits(word(c, 2)));
        with_color(vm, c, |col| {
            for v in &mut col[..3] {
                *v = (f * f64::from(*v)) as f32;
            }
        })
    });
    efuns.register(0x0064_0003, |vm, c| {
        let a = f32::from_bits(word(c, 2));
        with_color(vm, c, |col| col[3] = a)
    });
    efuns.register(0x0064_0004, |vm, c| {
        let v = word(c, 2);
        with_color(vm, c, |col| *col = from_color32(v))
    });
    efuns.register(0x0064_0005, |vm, c| {
        let this = Call::new(c).this();
        let Some(rgba) = (match vm.native(this) {
            Some(NativeObject::Color(rgba)) => Some(*rgba),
            _ => None,
        }) else {
            return vm.bad_this(this);
        };
        Call::new(c).ret(2, to_color32(rgba), INT_RESULT);
        InterpStatus::Success
    });
}

/// `RGBAColor::SetColor32`: each byte times `0.003921569` (a, r, g, b from the top).
pub fn from_color32(v: u32) -> [f32; 4] {
    let k = f64::from(0.003_921_569_f32);
    let c = |shift: u32| (f64::from((v >> shift) & 0xFF) * k) as f32;
    [c(16), c(8), c(0), c(24)]
}

/// `RGBAColor::GetColor32`: each channel times 255, truncated (`_ftol2`).
pub fn to_color32(rgba: [f32; 4]) -> u32 {
    let b = |v: f32| (f64::from(v) * 255.0) as i64 as u32;
    let [r, g, bl, a] = rgba;
    b(bl) | ((b(a) << 8 | b(r)) << 8 | b(g)) << 8
}

/// A native the script holds, by Rust type.
pub(super) trait Native: Sized {
    fn of(n: &NativeObject) -> Option<&Self>;
    fn of_mut(n: &mut NativeObject) -> Option<&mut Self>;
}

macro_rules! native_kind {
    ($t:ty, $v:ident) => {
        impl Native for $t {
            fn of(n: &NativeObject) -> Option<&Self> {
                match n {
                    NativeObject::$v(x) => Some(&**x),
                    _ => None,
                }
            }
            fn of_mut(n: &mut NativeObject) -> Option<&mut Self> {
                match n {
                    NativeObject::$v(x) => Some(&mut **x),
                    _ => None,
                }
            }
        }
    };
}

native_kind!(Shop, Shop);
native_kind!(ShopSale, ShopSale);
native_kind!(CraftSkill, CraftSkill);
native_kind!(DamageText, DamageText);

pub(super) fn native<T: Native>(vm: &Vm, r: u32) -> Option<&T> {
    T::of(vm.native(r)?)
}

pub(super) fn with<T: Native>(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut T, &mut Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this).and_then(T::of_mut) {
        Some(t) => {
            f(t, ctx);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

fn with_color(vm: &mut Vm, ctx: &mut Context, f: impl FnOnce(&mut [f32; 4])) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::Color(rgba)) => {
            f(rgba);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

pub(super) fn set<T: Native>(vm: &mut Vm, ctx: &mut Context, f: fn(&mut T, u32)) -> InterpStatus {
    let v = word(ctx, 2);
    with::<T>(vm, ctx, |t, _| f(t, v))
}

pub(super) fn get<T: Native>(vm: &mut Vm, ctx: &mut Context, f: fn(&T) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(v) = native::<T>(vm, this).map(f) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(2, v, INT_RESULT);
    InterpStatus::Success
}

/// The `StringInfo` setters (`StringInfo::operator=`).
pub(super) fn text_into<T: Native>(
    vm: &mut Vm,
    ctx: &mut Context,
    f: fn(&mut T) -> &mut StringInfo,
) -> InterpStatus {
    let text = match vm.native(word(ctx, 2)) {
        Some(NativeObject::StringInfo(s)) => string_info_to_net(s),
        _ => StringInfo::default(),
    };
    with::<T>(vm, ctx, |t, _| *f(t) = text)
}

/// The `StringInfo` getters: a copy as a new native.
pub(super) fn text_out<T: Native>(
    vm: &mut Vm,
    ctx: &mut Context,
    f: fn(&T) -> &StringInfo,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(text) = native::<T>(vm, this).map(|t| string_info_from_net(f(t))) else {
        return vm.bad_this(this);
    };
    let r = vm
        .refs
        .create_native(ntype::STRING_INFO, NativeObject::StringInfo(text));
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

/// `GetCostToRaise`: 64-bit, high word in slot 2, low in slot 3 (tag 0x84).
fn get_cost(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(cost) = native::<CraftSkill>(vm, this).map(|s| s.cost_to_raise) else {
        return vm.bad_this(this);
    };
    let high_tag = ctx.arg_tag(2).unwrap_or(tag::UNINIT);
    let mut call = Call::new(ctx);
    call.ret(3, cost as u32, tag::BOOL | tag::LONG);
    call.ret(2, (cost >> 32) as u32, high_tag);
    InterpStatus::Success
}

/// `UIShop::AddSale` (`List<UIShopSale>::insert_sort`): before the first sale with a higher
/// ordinal.
fn add_sale(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let sale = native::<ShopSale>(vm, word(ctx, 2))
        .cloned()
        .unwrap_or_default();
    with::<Shop>(vm, ctx, |s, _| {
        let at = s
            .sales
            .iter()
            .position(|n| sale.ordinal < n.ordinal)
            .unwrap_or(s.sales.len());
        s.sales.insert(at, sale);
    })
}
