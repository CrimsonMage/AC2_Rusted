//! `gGameplayOptionsProfile`'s container efuns: the profile's tables to and from the script's
//! hashes and arrays through a temporary C++ table (`AAHashToChatPopupFlags`,
//! `ChatPopupFlagsToAAHash`, `NRHashToStrStrHash`, `StrStrHashToNRHash`,
//! `RArrayToShortcutInfoArray`, `ShortcutInfoArrayToRArray`). Each copy flushes its target and
//! adds the source's elements in the source's order, each at the head of its bucket, so the
//! orders the scripts see follow retail's.

use ac2_net::protocol::{self as net, ShortcutInfo};

use super::hashes::with_hash;
use super::options::{shortcut_of, with_profile};
use super::{word, Call, INT_RESULT};
use crate::native::{
    assign_ui_locations, ntype, word_assign, AliasTable, GameplayOptions, NativeObject, RefKey,
    ScriptHash, DEFAULT_BUCKETS,
};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

const BASE: u32 = 0x009B_0000;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(BASE | 0x03, set_alias_table);
    efuns.register(BASE | 0x04, inq_alias_table);
    efuns.register(BASE | 0x0A, set_shortcut_array);
    efuns.register(BASE | 0x0B, inq_shortcut_array);
    efuns.register(BASE | 0x17, set_ui_save_locations);
    efuns.register(BASE | 0x18, |vm, c| get_ui_save_locations(vm, c, true));
    efuns.register(BASE | 0x31, |vm, c| get_ui_save_locations(vm, c, false));
    efuns.register(BASE | 0x3E, get_chat_options);
    efuns.register(BASE | 0x3F, set_chat_options);
    efuns.register(BASE | 0x42, |vm, c| {
        let out = word(c, 2);
        with_profile(vm, c, |vm, c, p| fill_out(vm, c, out, &p.opacities))
    });
    efuns.register(BASE | 0x43, |vm, c| {
        let from = read_words(vm, word(c, 2));
        with_profile(vm, c, |_, _, p| word_assign(&mut p.opacities, &from))
    });
}

/// The chat tables `Get/SetChatOptions` move, in argument order (Legions 0x00473660 /
/// 0x00473960: filters, sizes, colours, the window text types, popup flags).
fn chat_tables(p: &mut GameplayOptions) -> [&mut ScriptHash; 5] {
    [
        &mut p.filters,
        &mut p.font_sizes,
        &mut p.font_colors,
        &mut p.window_text_types,
        &mut p.popup_flags,
    ]
}

/// A script word hash as the temporary C++ table the glue builds from it (47 buckets).
fn read_words(vm: &Vm, reference: u32) -> ScriptHash {
    let mut temp = ScriptHash::with_buckets(DEFAULT_BUCKETS);
    if let Some(NativeObject::Hash(h)) = vm.native(reference) {
        word_assign(&mut temp, h);
    }
    temp
}

/// `ChatPopupFlagsToAAHash` after copying the profile's table into a temporary: the script
/// hash behind the out parameter `out` gets `src`'s elements.
fn fill_out(vm: &mut Vm, ctx: &mut Context, out: u32, src: &ScriptHash) {
    let mut temp = ScriptHash::with_buckets(DEFAULT_BUCKETS);
    word_assign(&mut temp, src);
    let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
    with_hash(vm, target, |h, keys, _| {
        h.flush();
        for n in temp.nodes() {
            h.add(n.key, n.value, keys);
        }
    });
}

fn set_chat_options(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let from: Vec<ScriptHash> = (2..7).map(|n| read_words(vm, word(ctx, n))).collect();
    with_profile(vm, ctx, |_, _, p| {
        for (table, src) in chat_tables(p).into_iter().zip(&from) {
            word_assign(table, src);
        }
    })
}

fn get_chat_options(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let outs: Vec<u32> = (2..7).map(|n| word(ctx, n)).collect();
    with_profile(vm, ctx, |vm, c, p| {
        for (table, out) in chat_tables(p).into_iter().zip(outs) {
            fill_out(vm, c, out, table);
        }
    })
}

/// `NRHashToStrStrHash`: the script's wpstring -> wpstring hash as an alias table
/// (`HashTable::add`, first key wins).
fn read_aliases(vm: &Vm, reference: u32) -> AliasTable {
    let mut table = AliasTable::with_buckets(DEFAULT_BUCKETS);
    if let Some(NativeObject::Hash(h)) = vm.native(reference) {
        for n in h.nodes() {
            let key = match n.key {
                RefKey::Ref { id, .. } => id,
                RefKey::Value(v) => v as u32,
            };
            let text = |r: u32| vm.wstring(r).unwrap_or(&[]).to_vec();
            table.add(text(key), text(n.value as u32));
        }
    }
    table
}

/// `SetAliasTable(NRHash)`.
fn set_alias_table(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let from = read_aliases(vm, word(ctx, 2));
    with_profile(vm, ctx, |_, _, p| p.aliases.assign(&from))
}

/// `InqAliasTable(out NRHash)`, ret 1: `StrStrHashToNRHash` makes a new `wpstring` for each
/// key and text and adds them keyed by `ReferenceIDWrapper(key)`.
fn inq_alias_table(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let out = word(ctx, 2);
    with_profile(vm, ctx, |vm, c, p| {
        let mut temp = AliasTable::with_buckets(DEFAULT_BUCKETS);
        temp.assign(&p.aliases);
        let target = vm.read_through(c, out).unwrap_or(u32::MAX);
        let pairs: Vec<(u32, u32)> = temp
            .iter()
            .map(|(k, v)| (k.to_vec(), v.to_vec()))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(k, v)| (vm.new_wstring(k), vm.new_wstring(v)))
            .collect();
        with_hash(vm, target, |h, keys, _| {
            h.flush();
            for (k, v) in pairs {
                let key = RefKey::Ref {
                    id: k,
                    cached_hash: 0,
                    use_cache: 0,
                };
                h.add(key, u64::from(v), keys);
            }
        });
        Call::new(c).ret(3, 1, INT_RESULT);
    })
}

/// `SetShortcutArray(RArray)`: `RArrayToShortcutInfoArray`, then `SetShortcutArray`.
fn set_shortcut_array(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let items: Vec<ShortcutInfo> = match vm.native(word(ctx, 2)) {
        Some(NativeObject::List(l)) => l.items.clone(),
        _ => Vec::new(),
    }
    .into_iter()
    .map(|r| shortcut_of(vm, r as u32))
    .collect();
    with_profile(vm, ctx, |_, _, p| p.shortcuts = items)
}

/// `InqShortcutArray(out RArray)`, ret 1: `ShortcutInfoArrayToRArray` empties the array and
/// appends a new `ShortcutInfo` per shortcut.
fn inq_shortcut_array(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let out = word(ctx, 2);
    with_profile(vm, ctx, |vm, c, p| {
        let refs: Vec<u64> = p
            .shortcuts
            .iter()
            .map(|s| {
                let native =
                    NativeObject::Wire(Box::new(net::NativeObject::ShortcutInfo(s.clone())));
                u64::from(vm.refs.create_native(ntype::SHORTCUT_INFO, native))
            })
            .collect();
        let target = vm.read_through(c, out).unwrap_or(u32::MAX);
        if let Some(NativeObject::List(l)) = vm.native_mut(target) {
            l.items = refs;
        }
        Call::new(c).ret(3, 1, INT_RESULT);
    })
}

fn ui_locations_of(vm: &Vm, reference: u32) -> Option<net::UiSaveLocations> {
    match vm.native(reference) {
        Some(NativeObject::Wire(w)) => match &**w {
            net::NativeObject::UiSaveLocations(u) => Some(u.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// `SetUISaveLocations(UISaveLocations)`: `UISaveLocations::operator=`.
fn set_ui_save_locations(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let from = ui_locations_of(vm, word(ctx, 2));
    with_profile(vm, ctx, |_, _, p| {
        if let Some(from) = &from {
            assign_ui_locations(&mut p.ui_locations, from);
        }
    })
}

/// `GetUISaveLocations(out)` / `InqUISaveLocations(out)` (the latter returns 1).
fn get_ui_save_locations(vm: &mut Vm, ctx: &mut Context, inq: bool) -> InterpStatus {
    let out = word(ctx, 2);
    with_profile(vm, ctx, |vm, c, p| {
        let target = vm.read_through(c, out).unwrap_or(u32::MAX);
        if let Some(NativeObject::Wire(w)) = vm.native_mut(target) {
            if let net::NativeObject::UiSaveLocations(dst) = &mut **w {
                assign_ui_locations(dst, &p.ui_locations);
            }
        }
        if inq {
            Call::new(c).ret(3, 1, INT_RESULT);
        }
    })
}
