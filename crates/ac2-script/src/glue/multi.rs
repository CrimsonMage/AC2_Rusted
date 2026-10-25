//! The multi-hash and multi-map glue (`gAAMultiHash` ... `gLRMultiHash`, `gAAMultiMap` ...
//! `gLLMultiMap` and their iterators) over [`ScriptMultiHash`] / [`ScriptMultiMap`].
//!
//! The first letter is the key (`A` 32-bit, `L` 64-bit in two slots, low word deeper), the
//! second the value (`A`, `L`, `R`). Iterator efuns past the end read a null node in retail
//! (a crash here).

use super::lists::Kind;
use super::{Call, INT_RESULT};
use crate::native::{NativeObject, ScriptMultiHash, ScriptMultiMap};
use crate::vm::{tag, Context, EFunFn, EFuns, Fault, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let Some((class, method)) = name.split_once("::CallEFun_") else {
            continue;
        };
        let f = if class.ends_with("MultiHash") || class.ends_with("MultiMap") {
            container_method(method)
        } else if class.ends_with("MultiHashIter") || class.ends_with("MultiMapIter") {
            iter_method(method)
        } else {
            None
        };
        if let Some(f) = f {
            efuns.register(id, f);
        }
    }
}

/// The key and value kinds of a multi-container type.
fn kinds(t: u32) -> (Kind, Kind) {
    match t {
        0x03 | 0x3691 => (Kind::A, Kind::A),
        0x0C | 0x3693 => (Kind::A, Kind::L),
        0x13 => (Kind::A, Kind::R),
        0x34 | 0x3695 => (Kind::L, Kind::A),
        0x3D | 0x3697 => (Kind::L, Kind::L),
        _ => (Kind::L, Kind::R),
    }
}

/// The value of `kind` starting `start` slots down, and the slot after it.
fn value_at(ctx: &Context, start: usize, kind: Kind) -> (u64, usize) {
    if kind == Kind::L {
        (super::word64(ctx, start + 1), start + 2)
    } else {
        (u64::from(super::word(ctx, start)), start + 1)
    }
}

/// A result of `kind` whose first slot is `n` (64-bit: tag 0x84 on the low word only).
fn ret_value(ctx: &mut Context, n: usize, v: u64, kind: Kind) {
    if kind == Kind::L {
        let high_tag = ctx.arg_tag(n).unwrap_or(tag::UNINIT);
        let mut call = Call::new(ctx);
        call.ret(n + 1, v as u32, kind.out_tag());
        call.ret(n, (v >> 32) as u32, high_tag);
    } else {
        Call::new(ctx).ret(n, v as u32, kind.out_tag());
    }
}

enum Multi<'a> {
    Hash(&'a mut ScriptMultiHash),
    Map(&'a mut ScriptMultiMap),
}

/// Runs `f` on `this` as a multi-hash or multi-map with its type.
fn with_this<R>(vm: &mut Vm, this: u32, f: impl FnOnce(Multi<'_>, u32) -> R) -> Option<R> {
    let object = vm.refs.heap_mut(this).ok()?;
    let t = object.native_type;
    match object.native.as_mut()? {
        NativeObject::MultiHash(h) => Some(f(Multi::Hash(h), t)),
        NativeObject::MultiMap(m) => Some(f(Multi::Map(m), t)),
        _ => None,
    }
}

fn new_iter(vm: &mut Vm, container: u32, t: u32, node: Option<u32>) -> u32 {
    vm.refs
        .create_native(t + 1, NativeObject::MultiIter { container, node })
}

/// Methods that take the key in word 2.
fn keyed(vm: &mut Vm, ctx: &mut Context, op: &str) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(t) = vm.refs.heap(this).ok().map(|o| o.native_type) else {
        return vm.bad_this(this);
    };
    let (kk, vk) = kinds(t);
    let (key, next) = value_at(ctx, 2, kk);
    let value = value_at(ctx, next, vk).0;
    enum Out {
        None,
        Int(u32),
        Iter(Option<u32>),
    }
    let out = with_this(vm, this, |m, _| match (m, op) {
        (Multi::Hash(h), "add") => {
            h.add(key, value);
            Out::None
        }
        (Multi::Map(m), "insert") => {
            m.insert(key, value);
            Out::None
        }
        (Multi::Hash(h), "remove") => Out::Int(h.remove(key)),
        (Multi::Map(m), "remove") => {
            m.remove(key);
            Out::None
        }
        (Multi::Hash(h), "count") => Out::Int(h.count(key)),
        (Multi::Map(m), "count") => Out::Int(m.count(key)),
        (Multi::Hash(h), "find") => Out::Iter(h.find(key)),
        (Multi::Map(m), "find") => Out::Iter(m.find(key)),
        (Multi::Hash(h), "find_last") => Out::Iter(h.equal_range_end(key)),
        // `BaseMultiMap::find_last` returns `equal_range(key).first`, the lower bound.
        (Multi::Map(m), "find_last" | "lower_bound") => Out::Iter(m.lower_bound(key)),
        (Multi::Map(m), "upper_bound") => Out::Iter(m.upper_bound(key)),
        _ => Out::None,
    });
    match out {
        None => vm.bad_this(this),
        Some(Out::None) => InterpStatus::Success,
        Some(Out::Int(n)) => {
            Call::new(ctx).ret(next, n, INT_RESULT);
            InterpStatus::Success
        }
        Some(Out::Iter(node)) => {
            let r = new_iter(vm, this, t, node);
            Call::new(ctx).ret(next, r, tag::REF);
            InterpStatus::Success
        }
    }
}

fn container_method(method: &str) -> Option<EFunFn> {
    Some(match method {
        "add" => |vm, c| keyed(vm, c, "add"),
        "insert" => |vm, c| keyed(vm, c, "insert"),
        "remove" => |vm, c| keyed(vm, c, "remove"),
        "count" => |vm, c| keyed(vm, c, "count"),
        "find" => |vm, c| keyed(vm, c, "find"),
        "find_last" => |vm, c| keyed(vm, c, "find_last"),
        "lower_bound" => |vm, c| keyed(vm, c, "lower_bound"),
        "upper_bound" => |vm, c| keyed(vm, c, "upper_bound"),
        "clear" => |vm, c| {
            let this = Call::new(c).this();
            match with_this(vm, this, |m, _| match m {
                Multi::Hash(h) => h.clear(),
                Multi::Map(m) => m.entries.clear(),
            }) {
                Some(()) => InterpStatus::Success,
                None => vm.bad_this(this),
            }
        },
        "size" | "empty" => size_or_empty,
        "get_begin" => |vm, c| make_iter(vm, c, true),
        "get_end" => |vm, c| make_iter(vm, c, false),
        "erase" => erase,
        _ => return None,
    })
}

/// `get_begin` / `get_end`: a new iterator (`type + 1`) in word 2.
fn make_iter(vm: &mut Vm, ctx: &mut Context, begin: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let found = with_this(vm, this, |m, t| {
        let first = match m {
            Multi::Hash(h) => h.begin(),
            Multi::Map(m) => m.begin(),
        };
        (first, t)
    });
    let Some((first, t)) = found else {
        return vm.bad_this(this);
    };
    let r = new_iter(vm, this, t, if begin { first } else { None });
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

/// `size` / `empty` (word 2).
fn size_or_empty(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let empty = super::efun_name(vm.current_efun()).is_some_and(|n| n.ends_with("empty"));
    let n = with_this(vm, this, |m, _| match m {
        Multi::Hash(h) => h.len(),
        Multi::Map(m) => m.entries.len() as u32,
    });
    let Some(n) = n else {
        return vm.bad_this(this);
    };
    let v = if empty { u32::from(n == 0) } else { n };
    Call::new(ctx).ret(2, v, INT_RESULT);
    InterpStatus::Success
}

/// `erase(iterator)`.
fn erase(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, iter) = (call.this(), call.word(2));
    let node = match vm.native(iter) {
        Some(NativeObject::MultiIter { node, .. }) => *node,
        _ => return vm.bad_this(iter),
    };
    let Some(id) = node else {
        return vm.fault(Fault::BadReference(iter));
    };
    match with_this(vm, this, |m, _| match m {
        Multi::Hash(h) => h.erase(id),
        Multi::Map(m) => m.erase(id),
    }) {
        Some(_) => InterpStatus::Success,
        None => vm.bad_this(this),
    }
}

/// `this` as an iterator: (container, node, container type).
fn iter_of(vm: &Vm, this: u32) -> Option<(u32, Option<u32>, u32)> {
    let object = vm.refs.heap(this).ok()?;
    match object.native {
        Some(NativeObject::MultiIter { container, node }) => {
            Some((container, node, object.native_type - 1))
        }
        _ => None,
    }
}

fn set_node(vm: &mut Vm, this: u32, to: Option<u32>) {
    if let Some(NativeObject::MultiIter { node, .. }) = vm.native_mut(this) {
        *node = to;
    }
}

/// `next` / `prev`.
fn step(vm: &mut Vm, ctx: &mut Context, forward: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some((container, node, _)) = iter_of(vm, this) else {
        return vm.bad_this(this);
    };
    let to = match (vm.native(container), node) {
        (Some(NativeObject::MultiHash(h)), Some(id)) if forward => h.next(id),
        (Some(NativeObject::MultiMap(m)), at) => {
            Some(if forward { m.next(at) } else { m.prev(at) })
        }
        _ => None,
    };
    match to {
        Some(to) => {
            set_node(vm, this, to);
            InterpStatus::Success
        }
        None => vm.fault(Fault::BadReference(container)),
    }
}

/// `key` / `data` of the current element (word 2).
fn get(vm: &mut Vm, ctx: &mut Context, want_key: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some((container, node, t)) = iter_of(vm, this) else {
        return vm.bad_this(this);
    };
    let found = node.and_then(|id| match vm.native(container)? {
        NativeObject::MultiHash(h) => h.node(id).map(|n| (n.key, n.value)),
        NativeObject::MultiMap(m) => m.node(id).map(|n| (n.key, n.value)),
        _ => None,
    });
    let Some((key, value)) = found else {
        return vm.fault(Fault::BadReference(container));
    };
    let (kk, vk) = kinds(t);
    if want_key {
        ret_value(ctx, 2, key, kk);
    } else {
        ret_value(ctx, 2, value, vk);
    }
    InterpStatus::Success
}

fn iter_method(method: &str) -> Option<EFunFn> {
    Some(match method {
        "next" => |vm, c| step(vm, c, true),
        "prev" => |vm, c| step(vm, c, false),
        "key" => |vm, c| get(vm, c, true),
        "data" => |vm, c| get(vm, c, false),
        "equals" => |vm, c| {
            let call = Call::new(c);
            let (this, other) = (call.this(), call.word(2));
            let (Some((_, a, _)), Some((_, b, _))) = (iter_of(vm, this), iter_of(vm, other)) else {
                return vm.bad_this(this);
            };
            Call::new(c).ret(3, u32::from(a == b), INT_RESULT);
            InterpStatus::Success
        },
        _ => return None,
    })
}
