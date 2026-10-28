//! `gCWRepository` (`gCWRepository.cpp`): a weenie's qualities as its script sees them
//! (`CWRepository` -> `m_qualities`, a `CBaseQualities`), over ac2-weenie's `WeenieQualities`
//! that the host keeps per object ([`Vm::repositories`]).
//!
//! `this` is the weenie's script instance; its process owner names the object. Without
//! qualities (`m_qualities == NULL`) Inq/Set/Remove give 0 and Get gives the default, as
//! `CWRepository` does. Reads look in the quality tables; the `gmWeenieDesc` fields and the
//! class defaults (`CBaseQualities::m_defaults`) are ac2-weenie's to add (see the coord file).

use std::collections::BTreeMap;

use ac2_net::protocol::QualityValue;
use ac2_weenie::{DescKind, WeenieQualities};

use super::lists::{write_out, Kind};
use super::{narrow, word, word64, Call, INT_RESULT};
use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};
use crate::wire::{string_info_from_net, string_info_to_net};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        if let Some(("gCWRepository", m)) = name.split_once("::CallEFun_") {
            if let Some(f) = method(m) {
                efuns.register(id, f);
            }
        }
    }
}

/// The value kinds, by the efun name's suffix.
#[derive(Clone, Copy, PartialEq, Eq)]
enum K {
    Int,
    LongInt,
    Bool,
    Float,
    String,
    DataId,
    InstanceId,
    Position,
    Timestamp,
    StringInfo,
    PackageId,
}

impl K {
    fn of(name: &str) -> Option<(Self, &str)> {
        // Longest suffix first ("LongInt" before "Int", "StringInfo" before "String").
        const NAMES: [(&str, K); 12] = [
            ("LongInt", K::LongInt),
            ("StringInfo", K::StringInfo),
            ("StringW", K::String),
            ("String", K::String),
            ("InstanceID", K::InstanceId),
            ("PackageID", K::PackageId),
            ("DataID", K::DataId),
            ("Timestamp", K::Timestamp),
            ("Position", K::Position),
            ("Float", K::Float),
            ("Bool", K::Bool),
            ("Int", K::Int),
        ];
        NAMES
            .iter()
            .find_map(|(s, k)| name.strip_suffix(s).map(|op| (*k, op)))
    }

    /// Slots a value of this kind takes in the frame (natives are one reference).
    fn wide(self) -> bool {
        matches!(self, K::LongInt | K::InstanceId | K::Timestamp)
    }

    fn out_kind(self) -> Kind {
        if self.wide() {
            Kind::L
        } else {
            Kind::A
        }
    }
}

/// A stored value: a word (`u64` for the 64-bit kinds) or a native.
#[derive(Clone)]
enum V {
    Word(u64),
    Native(NativeObject),
}

pub(super) fn owner_qualities(vm: &mut Vm, this: u32) -> Option<&mut WeenieQualities> {
    let id = *vm.owners.get(&this)?;
    vm.repositories.get_mut(&id)
}

/// `CBaseQualities::Inq*`: the weenie description's fields first (`gmWeenieDesc::Query*Stat`),
/// then the quality tables.
fn get_value(q: &WeenieQualities, k: K, id: u32) -> Option<V> {
    let w = |v: Option<u64>| v.map(V::Word);
    match k {
        K::Int => w(q.inq_int(id).map(|v| u64::from(v as u32))),
        K::Bool => w(q.inq_bool(id).map(|v| u64::from(v as u32))),
        K::Float => w(q.inq_float(id).map(|v| u64::from(v.to_bits()))),
        K::DataId => w(q.inq_data_id(id).map(u64::from)),
        K::PackageId => w(q.package_ids.get(&id).map(|v| u64::from(*v))),
        K::LongInt => w(q.long_ints.get(&id).map(|v| *v as u64)),
        K::InstanceId => w(q.inq_instance_id(id)),
        K::Timestamp => w(q.timestamps.get(&id).map(|v| v.to_bits())),
        K::String => q.strings.get(&id).map(|s| {
            V::Native(NativeObject::WString(
                s.iter().map(|b| u16::from(*b)).collect(),
            ))
        }),
        K::Position => q
            .positions
            .get(&id)
            .map(|p| V::Native(NativeObject::Position(*p))),
        K::StringInfo => q
            .inq_string_info(id)
            .map(|s| V::Native(NativeObject::StringInfo(string_info_from_net(&s)))),
    }
}

/// `CBaseQualities::Set*`: a description field goes into the weenie description, anything else
/// into its table ([`WeenieQualities::set`]). Package ids have no quality value and keep their
/// own table.
fn set_value(q: &mut WeenieQualities, k: K, id: u32, v: &V) -> bool {
    let value = match (k, v) {
        (K::PackageId, V::Word(x)) => {
            _ = q.package_ids.insert(id, *x as u32);
            return true;
        }
        (K::Int, V::Word(x)) => QualityValue::Int(*x as u32 as i32),
        (K::Bool, V::Word(x)) => QualityValue::Bool(*x as u32 as i32),
        (K::Float, V::Word(x)) => QualityValue::Float(f32::from_bits(*x as u32)),
        (K::DataId, V::Word(x)) => QualityValue::DataId(*x as u32),
        (K::LongInt, V::Word(x)) => QualityValue::LongInt(*x as i64),
        (K::InstanceId, V::Word(x)) => QualityValue::InstanceId(*x),
        (K::Timestamp, V::Word(x)) => QualityValue::Timestamp(f64::from_bits(*x)),
        (K::String, V::Native(NativeObject::WString(s))) => QualityValue::String(narrow(s)),
        (K::Position, V::Native(NativeObject::Position(p))) => QualityValue::Position(*p),
        (K::StringInfo, V::Native(NativeObject::StringInfo(s))) => {
            QualityValue::StringInfo(Box::new(string_info_to_net(s)))
        }
        _ => return false,
    };
    q.set(id, &value);
    true
}

/// `CBaseQualities::Remove*`: a description field goes first (`gmWeenieDesc::Remove*`), then
/// the table.
fn remove_value(q: &mut WeenieQualities, k: K, id: u32) -> bool {
    fn rm<T>(m: &mut BTreeMap<u32, T>, id: u32) -> bool {
        m.remove(&id).is_some()
    }
    let desc = match k {
        K::Int => Some(DescKind::Int),
        K::Bool => Some(DescKind::Bool),
        K::Float => Some(DescKind::Float),
        K::DataId => Some(DescKind::DataId),
        K::InstanceId => Some(DescKind::InstanceId),
        K::StringInfo => Some(DescKind::StringInfo),
        _ => None,
    };
    if desc.is_some_and(|kind| q.remove_in_weenie_desc(id, kind)) {
        return true;
    }
    match k {
        K::Int => rm(&mut q.ints, id),
        K::Bool => rm(&mut q.bools, id),
        K::Float => rm(&mut q.floats, id),
        K::DataId => rm(&mut q.dids, id),
        K::PackageId => rm(&mut q.package_ids, id),
        K::LongInt => rm(&mut q.long_ints, id),
        K::InstanceId => rm(&mut q.iids, id),
        K::Timestamp => rm(&mut q.timestamps, id),
        K::String => rm(&mut q.strings, id),
        K::Position => rm(&mut q.positions, id),
        K::StringInfo => rm(&mut q.string_infos, id),
    }
}

/// `CBaseQualities::AdjInt`'s own table: the value the table holds for `id`.
fn table_value(q: &WeenieQualities, k: K, id: u32) -> Option<V> {
    match k {
        K::Int => q.ints.get(&id).map(|v| V::Word(u64::from(*v as u32))),
        K::Float => q.floats.get(&id).map(|v| V::Word(u64::from(v.to_bits()))),
        K::LongInt => q.long_ints.get(&id).map(|v| V::Word(*v as u64)),
        _ => None,
    }
}

/// Stores an adjusted value in the table it was read from (`CBaseQualities::AdjInt` never
/// writes the description).
fn adj_store(q: &mut WeenieQualities, k: K, id: u32, new: u64) {
    match k {
        K::Int => _ = q.ints.insert(id, new as u32 as i32),
        K::Float => _ = q.floats.insert(id, f32::from_bits(new as u32)),
        K::LongInt => _ = q.long_ints.insert(id, new as i64),
        _ => {}
    }
}

/// `CBaseQualities::m_defaults` of the object `this` names: the class defaults at its qualities
/// DID. Without a DID, or without a record, the object has none.
fn class_defaults(vm: &Vm, this: u32) -> Option<std::sync::Arc<WeenieQualities>> {
    let id = *vm.owners.get(&this)?;
    let did = vm.repositories.get(&id)?.did;
    if did == 0 {
        return None;
    }
    vm.data().class_defaults(did)
}

/// The argument of kind `k` starting at slot `n` and the slot after it.
fn arg(vm: &Vm, ctx: &Context, k: K, n: usize) -> (V, usize) {
    if k.wide() {
        (V::Word(word64(ctx, n + 1)), n + 2)
    } else if matches!(k, K::String | K::Position | K::StringInfo) {
        let native = vm
            .native(word(ctx, n))
            .cloned()
            .unwrap_or(NativeObject::Unbound(0));
        (V::Native(native), n + 1)
    } else {
        (V::Word(u64::from(word(ctx, n))), n + 1)
    }
}

/// Writes a result of kind `k` into slot `n` (a native becomes a new object).
fn ret(vm: &mut Vm, ctx: &mut Context, k: K, n: usize, v: V) {
    match v {
        V::Word(x) if k.wide() => {
            let high_tag = ctx.arg_tag(n).unwrap_or(tag::UNINIT);
            let mut call = Call::new(ctx);
            call.ret(n + 1, x as u32, tag::BOOL | tag::LONG);
            call.ret(n, (x >> 32) as u32, high_tag);
        }
        V::Word(x) => Call::new(ctx).ret(n, x as u32, INT_RESULT),
        V::Native(native) => {
            let t = match native {
                NativeObject::WString(_) => ntype::WPSTRING,
                NativeObject::Position(_) => ntype::POSITION,
                _ => ntype::STRING_INFO,
            };
            let r = vm.refs.create_native(t, native);
            Call::new(ctx).ret(n, r, tag::REF);
        }
    }
}

/// `CWRepository::Inq*(stype, out value)`: 1 and the value when present.
fn inq(vm: &mut Vm, ctx: &mut Context, k: K) -> InterpStatus {
    let (this, id, out) = (word(ctx, 1), word(ctx, 2), word(ctx, 3));
    let found = owner_qualities(vm, this).and_then(|q| get_value(q, k, id));
    match &found {
        Some(V::Native(native)) => {
            // The out reference holds a native of the right type; it is overwritten.
            if let Some(target) = vm.read_through(ctx, out) {
                if let Some(slot) = vm.native_mut(target) {
                    *slot = native.clone();
                }
            }
            let current = vm.read_through(ctx, out).unwrap_or(0);
            vm.write_through(ctx, out, current, tag::REF);
        }
        Some(V::Word(x)) => write_out(vm, ctx, out, Some(*x), k.out_kind()),
        None if matches!(k, K::String | K::Position | K::StringInfo) => {}
        None => write_out(vm, ctx, out, None, k.out_kind()),
    }
    Call::new(ctx).ret(4, u32::from(found.is_some()), INT_RESULT);
    InterpStatus::Success
}

/// `CWRepository::Get*` / `GetRaw*(stype, default)`.
fn get(vm: &mut Vm, ctx: &mut Context, k: K) -> InterpStatus {
    let (this, id) = (word(ctx, 1), word(ctx, 2));
    let (default, next) = arg(vm, ctx, k, 3);
    let v = owner_qualities(vm, this)
        .and_then(|q| get_value(q, k, id))
        .unwrap_or(default);
    ret(vm, ctx, k, next, v);
    InterpStatus::Success
}

/// `CWRepository::Set*(stype, value)`: 1 when stored.
fn set(vm: &mut Vm, ctx: &mut Context, k: K) -> InterpStatus {
    let (this, id) = (word(ctx, 1), word(ctx, 2));
    let (v, next) = arg(vm, ctx, k, 3);
    let ok = owner_qualities(vm, this).is_some_and(|q| set_value(q, k, id, &v));
    Call::new(ctx).ret(next, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

/// `CWRepository::Remove*` / `Does*Exist(stype)`.
fn remove_or_exists(vm: &mut Vm, ctx: &mut Context, k: K, remove: bool) -> InterpStatus {
    let (this, id) = (word(ctx, 1), word(ctx, 2));
    let hit = owner_qualities(vm, this).is_some_and(|q| {
        if remove {
            remove_value(q, k, id)
        } else {
            get_value(q, k, id).is_some()
        }
    });
    Call::new(ctx).ret(3, u32::from(hit), INT_RESULT);
    InterpStatus::Success
}

/// `CBaseQualities::AdjInt` / `AdjLongInt` / `AdjFloat` (and the `AndTruncate` forms with
/// `TruncateInt`'s wrap and clamp): the new value, or the default without qualities.
fn adjust(vm: &mut Vm, ctx: &mut Context, k: K, truncate: bool) -> InterpStatus {
    let (this, id) = (word(ctx, 1), word(ctx, 2));
    let (V::Word(delta), n) = arg(vm, ctx, k, 3) else {
        return vm.bad_this(this);
    };
    let (V::Word(default), n) = arg(vm, ctx, k, n) else {
        return vm.bad_this(this);
    };
    let (min, max, n) = if truncate {
        let (V::Word(min), n) = arg(vm, ctx, k, n) else {
            return vm.bad_this(this);
        };
        let (V::Word(max), n) = arg(vm, ctx, k, n) else {
            return vm.bad_this(this);
        };
        (Some(min), Some(max), n)
    } else {
        (None, None, n)
    };
    let defaults = class_defaults(vm, this);
    let result = owner_qualities(vm, this).map(|q| {
        // `CBaseQualities::AdjInt` works on the quality's table only; a quality the table lacks
        // starts from the class default (`m_defaults`, `InqInt`), and the default's value is the
        // old value the truncation clamps against.
        let old =
            table_value(q, k, id).or_else(|| defaults.as_deref().and_then(|d| get_value(d, k, id)));
        let new = match k {
            K::Float => {
                let o = old.as_ref().map_or(0.0, |v| match v {
                    V::Word(x) => f32::from_bits(*x as u32),
                    V::Native(_) => 0.0,
                });
                let d = f32::from_bits(delta as u32);
                let mut v = (f64::from(o) + f64::from(d)) as f32;
                if let (Some(lo), Some(hi)) = (min, max) {
                    let (lo, hi) = (f32::from_bits(lo as u32), f32::from_bits(hi as u32));
                    v = v.min(hi).max(lo);
                }
                u64::from(v.to_bits())
            }
            K::LongInt => {
                let o = match &old {
                    Some(V::Word(x)) => *x as i64,
                    _ => 0,
                };
                let d = delta as i64;
                let mut v = o.wrapping_add(d);
                if let (Some(lo), Some(hi)) = (min, max) {
                    let (lo, hi) = (lo as i64, hi as i64);
                    if old.is_some() {
                        if (d < 0 && o < v) || (d >= 0 && v < o) {
                            v = if d < 0 { lo } else { hi };
                        }
                        v = v.min(hi).max(lo);
                    }
                }
                v as u64
            }
            _ => {
                let o = match &old {
                    Some(V::Word(x)) => *x as u32 as i32,
                    _ => 0,
                };
                let d = delta as u32 as i32;
                let mut v = o.wrapping_add(d);
                if let (Some(lo), Some(hi)) = (min, max) {
                    let (lo, hi) = (lo as u32 as i32, hi as u32 as i32);
                    // `TruncateInt` applies to a value that was there; a new one is stored as
                    // given.
                    if old.is_some() {
                        if (d < 0 && o < v) || (d >= 0 && v < o) {
                            v = if d < 0 { lo } else { hi };
                        }
                        v = v.min(hi).max(lo);
                    }
                }
                u64::from(v as u32)
            }
        };
        adj_store(q, k, id, new);
        new
    });
    ret(vm, ctx, k, n, V::Word(result.unwrap_or(default)));
    InterpStatus::Success
}

/// `CWRepository::GetID`: the weenie's instance id (64-bit in slots 2-3).
fn get_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = vm.owners.get(&word(ctx, 1)).copied().unwrap_or(0);
    ret(vm, ctx, K::InstanceId, 2, V::Word(id));
    InterpStatus::Success
}

fn method(m: &str) -> Option<EFunFn> {
    if m == "GetID" {
        return Some(get_id);
    }
    let (op, k) = if let Some(rest) = m.strip_prefix("AdjAndTruncate") {
        ("AdjAndTruncate", rest)
    } else if let Some(rest) = m.strip_suffix("Exist").and_then(|r| r.strip_prefix("Does")) {
        ("Does", rest)
    } else {
        let (k, op) = K::of(m)?;
        return dispatch(op, k);
    };
    let (k, _) = K::of(k)?;
    dispatch(op, k)
}

/// One glue function per (operation, kind): the kind comes from [`Vm::current_efun`]'s name.
fn dispatch(op: &str, _k: K) -> Option<EFunFn> {
    Some(match op {
        "Inq" => |vm, c| with_kind(vm, c, inq),
        "Get" | "GetRaw" => |vm, c| with_kind(vm, c, get),
        "Set" => |vm, c| with_kind(vm, c, set),
        "Remove" => |vm, c| with_kind(vm, c, |vm, c, k| remove_or_exists(vm, c, k, true)),
        "Does" => |vm, c| with_kind(vm, c, |vm, c, k| remove_or_exists(vm, c, k, false)),
        "Adj" => |vm, c| with_kind(vm, c, |vm, c, k| adjust(vm, c, k, false)),
        "AdjAndTruncate" => |vm, c| with_kind(vm, c, |vm, c, k| adjust(vm, c, k, true)),
        _ => return None,
    })
}

fn with_kind(
    vm: &mut Vm,
    ctx: &mut Context,
    f: fn(&mut Vm, &mut Context, K) -> InterpStatus,
) -> InterpStatus {
    let name = super::efun_name(vm.current_efun()).unwrap_or("");
    let m = name.split_once("::CallEFun_").map_or("", |(_, m)| m);
    let kind = m
        .strip_suffix("Exist")
        .and_then(|r| K::of(r))
        .or_else(|| K::of(m))
        .map(|(k, _)| k);
    match kind {
        Some(k) => f(vm, ctx, k),
        None => vm.bad_this(word(ctx, 1)),
    }
}
