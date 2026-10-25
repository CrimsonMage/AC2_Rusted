//! `gGameplayOptionsProfile` (`gGameplayOptionsProfile.cpp`, efuns 0x9B0001-0x9B0056 as the
//! Legions efun switch binds them): the options profile's bits, filters, chat tables, aliases
//! and shortcuts. `this` is the profile; every wrapper crashes on an undefined `this`. Results
//! are tagged 4, floats too. The container copies are in `options_tables`.
//!
//! `Export` / `Import` (0x9B002B / 0x9B002C) write and read a file in the client's folder and
//! are not bound.

use ac2_net::protocol::{self as net, ShortcutInfo};

use super::{word, Call, INT_RESULT};
use crate::native::{option_bits as b, word_get, word_set, GameplayOptions, NativeObject};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};

/// The bit efuns: (get id, set id, mask, how the set efun reads its flag).
const BITS: [(u32, u32, u32, Flag); 16] = [
    (0x10, 0x0F, b::SHOW_SHORTCUT_NUMBERS, Flag::Int),
    (0x12, 0x11, b::SHOW_DAMAGE_TEXT, Flag::Int),
    (0x14, 0x13, b::SHOW_DAMAGE_TEXT_OTHER, Flag::Int),
    (0x1A, 0x19, b::SHOW_RADAR_COORDS, Flag::Int),
    (0x24, 0x23, b::AUTO_TARGET, Flag::Int),
    (0x26, 0x25, b::ACCEPT_ALLEGIANCE_REQUEST, Flag::Int),
    (0x28, 0x27, b::ACCEPT_FELLOWSHIP_REQUEST, Flag::Int),
    (0x2A, 0x29, b::ACCEPT_TRADE_REQUEST, Flag::Int),
    (0x33, 0x32, b::SHOW_OBJECT_NAMES, Flag::Int),
    (0x41, 0x40, b::BYPASS_GOLD_MELTING_CONFIRMATION, Flag::Byte),
    (0x48, 0x47, b::SHORTCUT_LOCKED, Flag::Int),
    (0x4A, 0x49, b::SHOW_PK_DAMAGE, Flag::Int),
    (0x4E, 0x4D, b::SHORTCUT_INSTA_SCROLL, Flag::Int),
    (0x4C, 0x4B, b::LATENCY_INDICATOR_GRAPH, Flag::Byte),
    (0x50, 0x4F, b::HIDE_FELLOWS_ON_MAP, Flag::InvertedByte),
    (0x56, 0x55, b::HIDE_PET_UI, Flag::InvertedByte),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Flag {
    /// The int argument (non-zero sets).
    Int,
    /// Its low byte.
    Byte,
    /// Its low byte, inverted (the bit means hidden).
    InvertedByte,
}

const BASE: u32 = 0x009B_0000;

pub(super) fn register(efuns: &mut EFuns) {
    for (get, set, _, _) in BITS {
        efuns.register(BASE | get, get_bit);
        efuns.register(BASE | set, set_bit);
    }
    let words: [(u32, EFunFn); 30] = [
        (0x01, set_alias),
        (0x02, inq_alias),
        (0x05, |vm, c| unit(vm, c, |_, _, p| p.aliases.flush())),
        (0x06, remove_alias),
        (0x07, build_alias_list),
        (0x08, set_shortcut),
        (0x09, inq_shortcut),
        (0x0C, init_shortcut_array),
        (0x0D, |vm, c| {
            set_word(vm, c, |p, v| p.which_shortcut_set = v)
        }),
        (0x0E, |vm, c| get_word(vm, c, |p| p.which_shortcut_set)),
        (0x15, |vm, c| {
            set_word(vm, c, |p, v| p.damage_text_range_other = f32::from_bits(v))
        }),
        (0x16, |vm, c| {
            get_word(vm, c, |p| p.damage_text_range_other.to_bits())
        }),
        (0x1B, |vm, c| set_word(vm, c, |p, v| p.radar_mask |= v)),
        (0x1C, |vm, c| set_word(vm, c, |p, v| p.radar_mask &= !v)),
        (0x1D, |vm, c| set_word(vm, c, |p, v| p.radar_mask = v)),
        (0x1E, |vm, c| get_word(vm, c, |p| p.radar_mask)),
        (0x1F, |vm, c| {
            filter(vm, c, |old, mask| old.map_or(mask, |o| o | mask))
        }),
        (0x20, remove_filter),
        (0x21, |vm, c| filter(vm, c, |_, mask| mask)),
        (0x22, |vm, c| lookup(vm, c, |p| &p.filters, 0)),
        (0x2D, |vm, c| get_word(vm, c, |p| p.bit_field)),
        (0x2E, |vm, c| set_word(vm, c, |p, v| p.bit_field = v)),
        (0x38, |vm, c| store(vm, c, |p| &mut p.font_sizes)),
        (0x39, |vm, c| store(vm, c, |p| &mut p.font_colors)),
        (0x3A, |vm, c| lookup(vm, c, |p| &p.font_sizes, 0)),
        (0x3B, |vm, c| lookup(vm, c, |p| &p.font_colors, 0)),
        (0x44, |vm, c| lookup(vm, c, |p| &p.popup_flags, 0)),
        (0x45, set_chat_popup_flag),
        (0x46, |vm, c| store(vm, c, |p| &mut p.opacities)),
        (0x53, |vm, c| store(vm, c, |p| &mut p.window_text_types)),
    ];
    for (id, f) in words {
        efuns.register(BASE | id, f);
    }
    efuns.register(BASE | 0x51, |vm, c| {
        set_word(vm, c, |p, v| p.shortcut_height = v)
    });
    efuns.register(BASE | 0x52, |vm, c| get_word(vm, c, |p| p.shortcut_height));
    // Legions 0x00472330 -> 0x005BFD50: a window without a text type answers 0x10.
    efuns.register(BASE | 0x54, |vm, c| {
        lookup(vm, c, |p| &p.window_text_types, 0x10)
    });
    super::options_tables::register(efuns);
}

/// Runs `f` on `this` as a profile (taken out of its heap object for the call); crashes on
/// an undefined or non-profile `this`.
pub(super) fn with_profile(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut Vm, &mut Context, &mut GameplayOptions),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Ok(object) = vm.refs.heap_mut(this) else {
        return vm.bad_this(this);
    };
    let mut native = object.native.take();
    let ok = match &mut native {
        Some(NativeObject::GameplayOptions(p)) => {
            f(vm, ctx, p);
            true
        }
        _ => false,
    };
    if let Ok(object) = vm.refs.heap_mut(this) {
        object.native = native;
    }
    if ok {
        InterpStatus::Success
    } else {
        vm.bad_this(this)
    }
}

fn ret(ctx: &mut Context, n: usize, value: u32) {
    Call::new(ctx).ret(n, value, INT_RESULT);
}

fn unit(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut Vm, &mut Context, &mut GameplayOptions),
) -> InterpStatus {
    with_profile(vm, ctx, f)
}

fn get_word(vm: &mut Vm, ctx: &mut Context, f: fn(&GameplayOptions) -> u32) -> InterpStatus {
    with_profile(vm, ctx, |_, c, p| ret(c, 2, f(p)))
}

fn set_word(vm: &mut Vm, ctx: &mut Context, f: fn(&mut GameplayOptions, u32)) -> InterpStatus {
    let v = word(ctx, 2);
    with_profile(vm, ctx, |_, _, p| f(p, v))
}

fn bit_entry(vm: &Vm) -> (u32, u32, u32, Flag) {
    let id = vm.current_efun() & 0xFFFF;
    BITS.iter()
        .copied()
        .find(|&(g, s, _, _)| g == id || s == id)
        .expect("registered bit efun")
}

/// `TestBitMask(mask)` (inverted for the "show" efuns of hidden bits).
fn get_bit(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (_, _, mask, flag) = bit_entry(vm);
    with_profile(vm, ctx, |_, c, p| {
        let on = p.test_bit_mask(mask);
        ret(c, 2, u32::from(on != (flag == Flag::InvertedByte)));
    })
}

/// `SetBitMask(flag, mask)`.
fn set_bit(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (_, _, mask, flag) = bit_entry(vm);
    let v = word(ctx, 2);
    let on = match flag {
        Flag::Int => v != 0,
        Flag::Byte => v & 0xFF != 0,
        Flag::InvertedByte => v & 0xFF == 0,
    };
    with_profile(vm, ctx, |_, _, p| {
        p.set_bit_mask(on, mask);
    })
}

/// `GetFilter` / `GetChatFontSize` / `GetChatFontColor` / `GetChatPopupFlag` (key, ret).
fn lookup(
    vm: &mut Vm,
    ctx: &mut Context,
    table: fn(&GameplayOptions) -> &crate::native::ScriptHash,
    default: u32,
) -> InterpStatus {
    let key = word(ctx, 2);
    with_profile(vm, ctx, |_, c, p| {
        ret(c, 3, word_get(table(p), key).unwrap_or(default));
    })
}

/// `SetChatFontSize` / `SetChatFontColor` / `SetWindowOpacity` / Legions 0x005C0490 (key,
/// value, ret 1): `HashTable::set`.
fn store(
    vm: &mut Vm,
    ctx: &mut Context,
    table: fn(&mut GameplayOptions) -> &mut crate::native::ScriptHash,
) -> InterpStatus {
    let (key, value) = (word(ctx, 2), word(ctx, 3));
    with_profile(vm, ctx, |_, c, p| {
        word_set(table(p), key, value);
        ret(c, 4, 1);
    })
}

/// `SetChatPopupFlag(textType, flag)`: the flag's low byte.
fn set_chat_popup_flag(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (key, flag) = (word(ctx, 2), word(ctx, 3) & 0xFF);
    with_profile(vm, ctx, |_, c, p| {
        word_set(&mut p.popup_flags, key, flag);
        ret(c, 4, 1);
    })
}

/// `AddFilter` / `SetFilter(window, bitmask)`: the new mask from the old one, ret 1.
fn filter(vm: &mut Vm, ctx: &mut Context, f: fn(Option<u32>, u32) -> u32) -> InterpStatus {
    let (window, mask) = (word(ctx, 2), word(ctx, 3));
    with_profile(vm, ctx, |_, c, p| {
        let old = word_get(&p.filters, window);
        p.set_filter(window, f(old, mask));
        ret(c, 4, 1);
    })
}

/// `RemoveFilter`: a window without a filter is left alone (still 1).
fn remove_filter(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (window, mask) = (word(ctx, 2), word(ctx, 3));
    with_profile(vm, ctx, |_, c, p| {
        if let Some(old) = word_get(&p.filters, window) {
            p.set_filter(window, old & !mask);
        }
        ret(c, 4, 1);
    })
}

fn text(vm: &Vm, reference: u32) -> Vec<u16> {
    vm.wstring(reference).unwrap_or(&[]).to_vec()
}

/// `SetAlias(alias, text)`: `HashTable::set`, ret 1.
fn set_alias(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (alias, value) = (text(vm, word(ctx, 2)), text(vm, word(ctx, 3)));
    with_profile(vm, ctx, |_, c, p| {
        p.aliases.set(alias, value);
        ret(c, 4, 1);
    })
}

/// `InqAlias(alias, out text)`: the out string takes the alias's text when it exists.
fn inq_alias(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let alias = text(vm, word(ctx, 2));
    let out = word(ctx, 3);
    with_profile(vm, ctx, |vm, c, p| {
        let found = p.aliases.find(&alias).map(<[u16]>::to_vec);
        if let Some(value) = &found {
            let target = vm.read_through(c, out).unwrap_or(u32::MAX);
            if let Some(NativeObject::WString(s)) = vm.native_mut(target) {
                *s = value.clone();
            }
        }
        ret(c, 4, u32::from(found.is_some()));
    })
}

/// `RemoveAlias(alias)`: whether it was there.
fn remove_alias(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let alias = text(vm, word(ctx, 2));
    with_profile(vm, ctx, |_, c, p| {
        let removed = p.aliases.erase(&alias);
        ret(c, 3, u32::from(removed));
    })
}

/// `BuildAliasList`: "alias [%s] == [%s]\n" per alias in table order, as a new `wpstring`.
fn build_alias_list(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_profile(vm, ctx, |vm, c, p| {
        let mut list = Vec::new();
        for (alias, value) in p.aliases.iter() {
            list.extend("alias [".encode_utf16());
            list.extend_from_slice(alias);
            list.extend("] == [".encode_utf16());
            list.extend_from_slice(value);
            list.extend("]\n".encode_utf16());
        }
        let r = vm.new_wstring(list);
        Call::new(c).ret(2, r, tag::REF);
    })
}

/// The `ShortcutInfo` a reference holds (`ShortcutInfo()` for anything else).
pub(super) fn shortcut_of(vm: &Vm, reference: u32) -> ShortcutInfo {
    match vm.native(reference) {
        Some(NativeObject::Wire(w)) => match &**w {
            net::NativeObject::ShortcutInfo(s) => s.clone(),
            _ => ShortcutInfo::Empty(0),
        },
        _ => ShortcutInfo::Empty(0),
    }
}

/// `SetShortcut(index, shortcut)`: 0 past the array's end.
fn set_shortcut(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let index = word(ctx, 2) as usize;
    let shortcut = shortcut_of(vm, word(ctx, 3));
    with_profile(vm, ctx, |_, c, p| {
        let ok = match p.shortcuts.get_mut(index) {
            Some(slot) => {
                *slot = shortcut;
                true
            }
            None => false,
        };
        ret(c, 4, u32::from(ok));
    })
}

/// `InqShortcut(index, out shortcut)`: the out `ShortcutInfo` takes the entry.
fn inq_shortcut(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (index, out) = (word(ctx, 2) as usize, word(ctx, 3));
    with_profile(vm, ctx, |vm, c, p| {
        let found = p.shortcuts.get(index).cloned();
        let target = vm.read_through(c, out).unwrap_or(u32::MAX);
        if let (Some(s), Some(NativeObject::Wire(w))) = (&found, vm.native_mut(target)) {
            **w = net::NativeObject::ShortcutInfo(s.clone());
        }
        ret(c, 4, u32::from(found.is_some()));
    })
}

/// `InitShortcutArray(size)`: `size` empty shortcuts (none for a size below 1), ret 1.
fn init_shortcut_array(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let size = word(ctx, 2) as i32;
    with_profile(vm, ctx, |_, c, p| {
        p.shortcuts = vec![ShortcutInfo::Empty(0); size.max(0) as usize];
        ret(c, 3, 1);
    })
}
