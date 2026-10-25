//! `ggmCharGenResult` (`gmCharGenResult`, native 0x9E) and `ggmRaceSexInfo` (ac2-net's
//! `GmRaceSexInfo`, native 0xA2). The calls that ask the `CharacterGenSystem` script (admin
//! avatar, validity, starting position) go through [`crate::host::chargen`].

use ac2_net::protocol::{GmRaceSexInfo, NativeObject as Wire};

use super::{word, Call, INT_RESULT};
use crate::host::chargen;
use crate::native::{
    bucket_size_index, ntype, CharGenResult, NativeObject, PlainKeys, Position, RefKey, ScriptHash,
};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let f = match name.split_once("::CallEFun_") {
            Some(("ggmCharGenResult", m)) => cgr_method(m),
            Some(("ggmRaceSexInfo", m)) => race_sex_method(m),
            _ => None,
        };
        if let Some(f) = f {
            efuns.register(id, f);
        }
    }
}

fn cgr(vm: &Vm, r: u32) -> Option<CharGenResult> {
    match vm.native(r)? {
        NativeObject::CharGenResult(c) => Some((**c).clone()),
        _ => None,
    }
}

fn edit(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut CharGenResult, &Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::CharGenResult(c)) => {
            f(c, ctx);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// A setter that returns 1 in slot 3 (`SetClassType`, `SetImplement`).
fn set_ret(vm: &mut Vm, ctx: &mut Context, f: fn(&mut CharGenResult, u32)) -> InterpStatus {
    let v = word(ctx, 2);
    let status = edit(vm, ctx, |c, _| f(c, v));
    Call::new(ctx).ret(3, 1, INT_RESULT);
    status
}

/// A word result in slot 2.
fn get(vm: &mut Vm, ctx: &mut Context, f: fn(&CharGenResult) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    match cgr(vm, this) {
        Some(c) => {
            Call::new(ctx).ret(2, f(&c), INT_RESULT);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// `gmCharGenResult::SanityCheck`: 0xCB without a name or entity, else the script's
/// `IsValid`.
fn sanity_check(vm: &mut Vm, c: &CharGenResult) -> u32 {
    if c.name.is_empty() || c.entity_did == 0 {
        return 0xCB;
    }
    chargen::is_valid(vm, c).unwrap_or(0)
}

/// `gmCharGenResult::GetStartingPosition`: the start area (or the race and class's first valid
/// one) and its position.
fn starting_position(vm: &mut Vm, c: &CharGenResult) -> Position {
    let mut area = c.start_area;
    if area == 0 {
        if let Some((_, areas)) = chargen::valid_starting_areas(vm, c.race, c.class) {
            area = areas.first().copied().unwrap_or(0);
        }
    }
    match chargen::starting_position(vm, area) {
        Some((_, p)) => p,
        None => match NativeObject::new_empty(ntype::POSITION) {
            NativeObject::Position(p) => p,
            _ => unreachable!(),
        },
    }
}

fn cgr_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "GetRace" => |vm, c| get(vm, c, |g| g.race),
        "GetSex" => |vm, c| get(vm, c, |g| g.sex),
        "GetClassType" => |vm, c| get(vm, c, |g| g.class),
        "GetImplement" => |vm, c| get(vm, c, |g| g.implement),
        "GetStartArea" => |vm, c| get(vm, c, |g| g.start_area),
        "SetRace" => |vm, c| edit(vm, c, |g, c| g.race = word(c, 2)),
        "SetSex" => |vm, c| edit(vm, c, |g, c| g.sex = word(c, 2)),
        "SetStartArea" => |vm, c| edit(vm, c, |g, c| g.start_area = word(c, 2)),
        "SetClassType" => |vm, c| set_ret(vm, c, |g, v| g.class = v),
        "SetImplement" => |vm, c| set_ret(vm, c, |g, v| g.implement = v),
        "Clear" => |vm, c| edit(vm, c, |g, _| g.clear()),
        "GetPhysTypeValue" => |vm, c| {
            let this = Call::new(c).this();
            let Some(g) = cgr(vm, this) else {
                return vm.bad_this(this);
            };
            let v = g.phys_type_value(word(c, 2));
            Call::new(c).ret(3, v.to_bits(), INT_RESULT);
            InterpStatus::Success
        },
        "SetPhysTypeValue" => |vm, c| {
            let (t, v) = (word(c, 2), f32::from_bits(word(c, 3)));
            let status = edit(vm, c, |g, _| g.set_phys_type_value(t, v));
            Call::new(c).ret(4, 1, INT_RESULT);
            status
        },
        "GetName" => |vm, c| {
            let this = Call::new(c).this();
            let Some(g) = cgr(vm, this) else {
                return vm.bad_this(this);
            };
            let r = vm.new_wstring(g.name);
            Call::new(c).ret(2, r, tag::REF);
            InterpStatus::Success
        },
        "SetName" => |vm, c| {
            let name = vm
                .wstring(word(c, 2))
                .map(<[u16]>::to_vec)
                .unwrap_or_default();
            edit(vm, c, |g, _| g.name = name)
        },
        "SetStartingPosition" => |vm, c| {
            let Some(NativeObject::Position(p)) = vm.native(word(c, 2)).cloned() else {
                return vm.bad_this(word(c, 2));
            };
            edit(vm, c, |g, _| g.start_pos = p)
        },
        "GetStartingPosition" => |vm, c| {
            let this = Call::new(c).this();
            let Some(g) = cgr(vm, this) else {
                return vm.bad_this(this);
            };
            let p = starting_position(vm, &g);
            let r = vm
                .refs
                .create_native(ntype::POSITION, NativeObject::Position(p));
            Call::new(c).ret(2, r, tag::REF);
            InterpStatus::Success
        },
        "SanityCheck" | "VerifyCharGenResult" => |vm, c| {
            let this = Call::new(c).this();
            let Some(g) = cgr(vm, this) else {
                return vm.bad_this(this);
            };
            let v = sanity_check(vm, &g);
            Call::new(c).ret(2, v, INT_RESULT);
            InterpStatus::Success
        },
        // `gmCharGenResult::IsAdmin`: the entity is the admin avatar.
        "IsAdmin" => |vm, c| {
            let this = Call::new(c).this();
            let Some(g) = cgr(vm, this) else {
                return vm.bad_this(this);
            };
            let admin = chargen::avatar_entity_did(vm, true).unwrap_or(0);
            Call::new(c).ret(2, u32::from(g.entity_did == admin), INT_RESULT);
            InterpStatus::Success
        },
        // `gmCharGenResult::SetAdmin(bAdmin)`: the admin or the player avatar entity.
        "SetAdmin" => |vm, c| {
            let admin = word(c, 2) != 0;
            let did = chargen::avatar_entity_did(vm, admin).unwrap_or(0);
            edit(vm, c, |g, _| g.entity_did = did)
        },
        _ => return None,
    })
}

fn race_sex_mut(vm: &mut Vm, r: u32) -> Option<&mut GmRaceSexInfo> {
    match vm.native_mut(r)? {
        NativeObject::Wire(w) => match &mut **w {
            Wire::GmRaceSexInfo(info) => Some(info),
            _ => None,
        },
        _ => None,
    }
}

fn race_sex(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut GmRaceSexInfo, &Context) -> Option<(usize, u32)>,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let result = match race_sex_mut(vm, this) {
        Some(info) => f(info, ctx),
        None => return vm.bad_this(this),
    };
    if let Some((slot, v)) = result {
        Call::new(ctx).ret(slot, v, INT_RESULT);
    }
    InterpStatus::Success
}

/// `gmRaceSexInfo::gmRaceSexInfo`: zero DIDs and an empty 47-bucket table.
pub(crate) fn empty_race_sex_info() -> GmRaceSexInfo {
    GmRaceSexInfo {
        phys_obj_did: 0,
        apr_file_did: 0,
        music_file_did: 0,
        obj_in_loc_bucket_index: bucket_size_index(crate::native::DEFAULT_BUCKETS) as u8,
        obj_in_loc: Vec::new(),
    }
}

/// `gmRaceSexInfo::PutObjInLoc(key, value)`: `HashTable::add` on `_objInLocHash`.
fn put_obj_in_loc(info: &mut GmRaceSexInfo, key: u32, value: u32) -> bool {
    let items = info
        .obj_in_loc
        .iter()
        .map(|(k, v)| (RefKey::Value(u64::from(*k)), u64::from(*v)))
        .collect();
    let mut hash = ScriptHash::from_packed(info.obj_in_loc_bucket_index, items);
    let added = hash.add(RefKey::Value(u64::from(key)), u64::from(value), &PlainKeys);
    info.obj_in_loc = hash
        .nodes()
        .filter_map(|n| match n.key {
            RefKey::Value(k) => Some((k as u32, n.value as u32)),
            _ => None,
        })
        .collect();
    added
}

fn race_sex_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "SetPhysObj" => |vm, c| {
            race_sex(vm, c, |i, c| {
                i.phys_obj_did = word(c, 2);
                None
            })
        },
        "SetPileAppearance" => |vm, c| {
            race_sex(vm, c, |i, c| {
                i.apr_file_did = word(c, 2);
                None
            })
        },
        "SetMusic" => |vm, c| {
            race_sex(vm, c, |i, c| {
                i.music_file_did = word(c, 2);
                None
            })
        },
        "GetPhysObj" => |vm, c| race_sex(vm, c, |i, _| Some((2, i.phys_obj_did))),
        "GetPileAppearance" => |vm, c| race_sex(vm, c, |i, _| Some((2, i.apr_file_did))),
        "GetMusic" => |vm, c| race_sex(vm, c, |i, _| Some((2, i.music_file_did))),
        "PutObjInLoc" => |vm, c| {
            race_sex(vm, c, |i, c| {
                Some((4, u32::from(put_obj_in_loc(i, word(c, 2), word(c, 3)))))
            })
        },
        // `gmRaceSexInfo::Verify`: every DID set and no zero in the location table.
        "Verify" => |vm, c| {
            race_sex(vm, c, |i, _| {
                let ok = i.phys_obj_did != 0
                    && i.obj_in_loc.iter().all(|(_, v)| *v != 0)
                    && i.apr_file_did != 0
                    && i.music_file_did != 0;
                Some((2, u32::from(ok)))
            })
        },
        _ => return None,
    })
}
