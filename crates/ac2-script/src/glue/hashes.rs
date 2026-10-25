//! The hash glue: `gAAHash`, `gALHash`, `gARHash`, `gLAHash`, `gLLHash`, `gLRHash`, `gNAHash`,
//! `gNRHash`, the hash sets `gAHashSet`, `gLAHashSet`, and their iterators (`BaseHash`,
//! `BaseHashSet`, `BaseHashIter`, `BaseHashSetIter` over `IntrusiveHashTable`).
//!
//! The first letter is the key (`A` 32-bit, `L` 64-bit taking two slots with the low word
//! deeper, `N` a reference hashed by its native value), the second the value (`A`, `L`, `R` a
//! reference). Arguments follow `this` in order and the result is the slot below the last.

use super::lists::{write_out, Kind};
use super::{Call, INT_RESULT};
use crate::glue::string_text;
use crate::native::{ntype, HashPos, NativeKeys, NativeObject, RefKey, ScriptHash, StringInfo};
use crate::vm::{tag, Context, EFunFn, EFuns, Fault, InterpStatus, ReferenceTable, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let Some((class, method)) = name.split_once("::CallEFun_") else {
            continue;
        };
        let is_hash = matches!(
            class,
            "gAAHash"
                | "gALHash"
                | "gARHash"
                | "gLAHash"
                | "gLLHash"
                | "gLRHash"
                | "gNAHash"
                | "gNRHash"
                | "gAHashSet"
                | "gLAHashSet"
        );
        let is_iter = class.ends_with("HashIter") || class.ends_with("HashSetIter");
        let f = if is_hash {
            hash_method(method)
        } else if is_iter && !class.contains("Multi") {
            iter_method(method)
        } else {
            None
        };
        if let Some(f) = f {
            efuns.register(id, f);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyKind {
    A,
    L,
    N,
}

/// The key and value kinds of a hash type (`None` value for a set).
fn kinds(hash_type: u32) -> (KeyKind, Option<Kind>) {
    match hash_type {
        ntype::AA_HASH => (KeyKind::A, Some(Kind::A)),
        ntype::AL_HASH => (KeyKind::A, Some(Kind::L)),
        ntype::AR_HASH => (KeyKind::A, Some(Kind::R)),
        ntype::LA_HASH => (KeyKind::L, Some(Kind::A)),
        ntype::LL_HASH => (KeyKind::L, Some(Kind::L)),
        ntype::LR_HASH => (KeyKind::L, Some(Kind::R)),
        ntype::NA_HASH => (KeyKind::N, Some(Kind::A)),
        ntype::NR_HASH => (KeyKind::N, Some(Kind::R)),
        ntype::LA_HASH_SET => (KeyKind::L, None),
        _ => (KeyKind::A, None),
    }
}

/// The key starting `start` slots down and the slot after it.
fn key_at(ctx: &Context, start: usize, kind: KeyKind) -> (RefKey, usize) {
    match kind {
        KeyKind::A => (RefKey::Value(u64::from(super::word(ctx, start))), start + 1),
        KeyKind::L => (RefKey::Value(super::word64(ctx, start + 1)), start + 2),
        KeyKind::N => (
            RefKey::Ref {
                id: super::word(ctx, start),
                cached_hash: 0,
                use_cache: 0,
            },
            start + 1,
        ),
    }
}

fn value_at(ctx: &Context, start: usize, kind: Kind) -> (u64, usize) {
    if kind == Kind::L {
        (super::word64(ctx, start + 1), start + 2)
    } else {
        (u64::from(super::word(ctx, start)), start + 1)
    }
}

/// `PStringBase<unsigned short>::compute_hash` (up to the first NUL).
fn compute_hash(text: &[u16]) -> u32 {
    let mut h: u32 = 0;
    for &c in text.iter().take_while(|&&c| c != 0) {
        h = h.wrapping_mul(16).wrapping_add(u32::from(c));
        if h & 0xF000_0000 != 0 {
            h = ((h & 0xF000_0000) >> 24 ^ h) & 0x0FFF_FFFF;
        }
    }
    if h == u32::MAX {
        0xFFFF_FFFE
    } else {
        h
    }
}

fn until_nul(text: &[u16]) -> &[u16] {
    let end = text.iter().position(|&c| c == 0).unwrap_or(text.len());
    &text[..end]
}

/// `StringInfo::operator%`.
fn string_info_hash(info: &StringInfo, n: u32) -> u32 {
    match &info.literal {
        Some(text) => compute_hash(text) % n,
        None => (info.table_id % n + info.string_id % n) % n,
    }
}

/// `VM::HashNativeObject` / `VM::IsNativeEqual` through `CNativeFactory`: `StringInfo` and
/// `wpstring` hash and compare by value; every other native hashes to 0 and equals nothing.
impl NativeKeys for ReferenceTable {
    fn hash(&self, id: u32, n: u32) -> u32 {
        let native = self.heap(id).ok().and_then(|o| o.native.as_ref());
        match native {
            Some(NativeObject::StringInfo(info)) => string_info_hash(info, n),
            Some(NativeObject::WString(s)) => compute_hash(s) % n,
            _ => 0,
        }
    }

    fn equal(&self, a: u32, b: u32) -> bool {
        let native = |id| self.heap(id).ok().and_then(|o| o.native.as_ref());
        match (native(a), native(b)) {
            (Some(NativeObject::StringInfo(x)), Some(NativeObject::StringInfo(y))) => {
                a == b || string_text::equal(x, y)
            }
            (Some(NativeObject::WString(x)), Some(NativeObject::WString(y))) => {
                until_nul(x) == until_nul(y)
            }
            _ => false,
        }
    }
}

/// Runs `f` on `this` as a hash, with the reference table for native keys. The table is taken
/// out of its heap object for the call (a hash used as its own key hashes to 0 either way).
fn with_hash<R>(
    vm: &mut Vm,
    this: u32,
    f: impl FnOnce(&mut ScriptHash, &ReferenceTable, u32) -> R,
) -> Option<R> {
    let object = vm.refs.heap_mut(this).ok()?;
    let hash_type = object.native_type;
    let mut native = object.native.take();
    let result = match &mut native {
        Some(NativeObject::Hash(hash)) => Some(f(hash, &vm.refs, hash_type)),
        _ => None,
    };
    if let Ok(object) = vm.refs.heap_mut(this) {
        object.native = native;
    }
    result
}

fn bad(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    vm.bad_this(this)
}

/// `add` / `set` (and `HashSet::add`): key, value, result.
fn insert(vm: &mut Vm, ctx: &mut Context, replace: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(hash_type) = vm.refs.heap(this).ok().map(|o| o.native_type) else {
        return vm.bad_this(this);
    };
    let (kk, vk) = kinds(hash_type);
    let (key, next) = key_at(ctx, 2, kk);
    let (value, ret) = vk.map_or((0, next), |k| value_at(ctx, next, k));
    let done = with_hash(vm, this, |h, keys, _| {
        if replace {
            h.set(key, value, keys)
        } else {
            h.add(key, value, keys)
        }
    });
    match done {
        Some(ok) => {
            Call::new(ctx).ret(ret, u32::from(ok), INT_RESULT);
            InterpStatus::Success
        }
        None => bad(vm, ctx),
    }
}

fn add(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    insert(vm, ctx, false)
}

fn set(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    insert(vm, ctx, true)
}

/// `find(key, out value)` and `remove(key, out value)`: the out slot is tagged whether or not
/// the key is found.
fn lookup(vm: &mut Vm, ctx: &mut Context, take: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(hash_type) = vm.refs.heap(this).ok().map(|o| o.native_type) else {
        return vm.bad_this(this);
    };
    let (kk, vk) = kinds(hash_type);
    let vk = vk.unwrap_or(Kind::A);
    let (key, next) = key_at(ctx, 2, kk);
    let out = super::word(ctx, next);
    let found = with_hash(vm, this, |h, keys, _| {
        if take {
            h.remove(&key, keys)
        } else {
            h.find(&key, keys)
        }
    });
    let Some(found) = found else {
        return bad(vm, ctx);
    };
    write_out(vm, ctx, out, found, vk);
    Call::new(ctx).ret(next + 1, u32::from(found.is_some()), INT_RESULT);
    InterpStatus::Success
}

fn find(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    lookup(vm, ctx, false)
}

fn remove(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    lookup(vm, ctx, true)
}

/// `contains(key)` and `erase(key)`.
fn keyed(vm: &mut Vm, ctx: &mut Context, erase: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(hash_type) = vm.refs.heap(this).ok().map(|o| o.native_type) else {
        return vm.bad_this(this);
    };
    let (key, ret) = key_at(ctx, 2, kinds(hash_type).0);
    let hit = with_hash(vm, this, |h, keys, _| {
        if erase {
            h.remove(&key, keys).is_some()
        } else {
            h.contains(&key, keys)
        }
    });
    match hit {
        Some(hit) => {
            Call::new(ctx).ret(ret, u32::from(hit), INT_RESULT);
            InterpStatus::Success
        }
        None => bad(vm, ctx),
    }
}

fn contains(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    keyed(vm, ctx, false)
}

fn erase(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    keyed(vm, ctx, true)
}

/// A method with no arguments returning an int in word 2.
fn query(
    vm: &mut Vm,
    ctx: &mut Context,
    f: fn(&mut ScriptHash, &ReferenceTable) -> u32,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match with_hash(vm, this, |h, keys, _| f(h, keys)) {
        Some(v) => {
            Call::new(ctx).ret(2, v, INT_RESULT);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

macro_rules! hash_query {
    ($name:ident, |$h:ident, $keys:ident| $body:expr) => {
        fn $name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
            query(vm, ctx, |$h, $keys| {
                let _ = &$keys;
                $body
            })
        }
    };
}

hash_query!(num_elements, |h, keys| h.num_elements());
hash_query!(num_buckets, |h, keys| h.num_buckets());
hash_query!(empty, |h, keys| u32::from(h.num_elements() == 0));
hash_query!(too_big, |h, keys| u32::from(h.too_big()));
hash_query!(too_small, |h, keys| u32::from(h.too_small()));
hash_query!(grow, |h, keys| u32::from(h.grow(keys)));
hash_query!(shrink, |h, keys| u32::from(h.shrink(keys)));
hash_query!(optimize_size, |h, keys| u32::from(h.optimize_size(keys)));

fn flush(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    match with_hash(vm, this, |h, _, _| h.flush()) {
        Some(()) => InterpStatus::Success,
        None => vm.bad_this(this),
    }
}

/// `resize(n)`: `resize_internal(lower_bound(g_bucketSizes, n))`.
fn resize(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, n) = (call.this(), call.word(2));
    match with_hash(vm, this, |h, keys, _| h.resize(n, keys)) {
        Some(ok) => {
            Call::new(ctx).ret(3, u32::from(ok), INT_RESULT);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// `BaseHash::get_begin` / `get_end`: a new iterator (`hash type + 1`) holding `this`.
fn make_iter(vm: &mut Vm, ctx: &mut Context, begin: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let found = with_hash(vm, this, |h, _, t| {
        (if begin { h.begin() } else { h.end() }, t)
    });
    let Some((pos, hash_type)) = found else {
        return vm.bad_this(this);
    };
    let r = vm
        .refs
        .create_native(hash_type + 1, NativeObject::HashIter { hash: this, pos });
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

fn get_begin(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    make_iter(vm, ctx, true)
}

fn get_end(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    make_iter(vm, ctx, false)
}

fn hash_method(method: &str) -> Option<EFunFn> {
    Some(match method {
        "add" => add,
        "set" => set,
        "erase" => erase,
        "find" => find,
        "remove" => remove,
        "contains" => contains,
        "num_elements" => num_elements,
        "num_buckets" => num_buckets,
        "empty" => empty,
        "get_begin" => get_begin,
        "get_end" => get_end,
        "flush" => flush,
        "grow" => grow,
        "shrink" => shrink,
        "too_big" => too_big,
        "too_small" => too_small,
        "resize" => resize,
        "optimize_size" => optimize_size,
        _ => return None,
    })
}

/// `this` as an iterator: its hash, position and the hash's type.
fn iter_of(vm: &Vm, this: u32) -> Option<(u32, HashPos, u32)> {
    let object = vm.refs.heap(this).ok()?;
    match object.native {
        Some(NativeObject::HashIter { hash, pos }) => Some((hash, pos, object.native_type - 1)),
        _ => None,
    }
}

fn hash_ref(vm: &Vm, hash: u32) -> Option<&ScriptHash> {
    match vm.native(hash)? {
        NativeObject::Hash(h) => Some(h),
        _ => None,
    }
}

fn set_pos(vm: &mut Vm, this: u32, pos: HashPos) {
    if let Some(NativeObject::HashIter { pos: p, .. }) = vm.native_mut(this) {
        *p = pos;
    }
}

/// `HashIterator::next`; past the end retail reads through a null element (crash here).
fn iter_next(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some((hash, pos, _)) = iter_of(vm, this) else {
        return vm.bad_this(this);
    };
    match hash_ref(vm, hash).and_then(|h| h.next(pos)) {
        Some(next) => {
            set_pos(vm, this, next);
            InterpStatus::Success
        }
        None => vm.fault(Fault::BadReference(hash)),
    }
}

/// `IntrusiveHashIterator::remove`: to the next element, then unlink the current one.
fn iter_remove(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some((hash, pos, _)) = iter_of(vm, this) else {
        return vm.bad_this(this);
    };
    let next = match vm.native_mut(hash) {
        Some(NativeObject::Hash(h)) => h.remove_at(pos),
        _ => None,
    };
    match next {
        Some(next) => {
            set_pos(vm, this, next);
            InterpStatus::Success
        }
        None => vm.fault(Fault::BadReference(hash)),
    }
}

/// `key` / `data`: the current element's key or value (64-bit: low word deeper, tag 0x84 on
/// it). Retail dereferences a null element at the end (crash here).
fn iter_get(vm: &mut Vm, ctx: &mut Context, want_key: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some((hash, pos, hash_type)) = iter_of(vm, this) else {
        return vm.bad_this(this);
    };
    let Some(node) = hash_ref(vm, hash).and_then(|h| h.node(pos)) else {
        return vm.fault(Fault::BadReference(hash));
    };
    let (kk, vk) = kinds(hash_type);
    let (value, kind) = if want_key {
        let v = match node.key {
            RefKey::Value(v) => v,
            RefKey::Ref { id, .. } => u64::from(id),
        };
        let kind = match kk {
            KeyKind::A => Kind::A,
            KeyKind::L => Kind::L,
            KeyKind::N => Kind::R,
        };
        (v, kind)
    } else {
        (node.value, vk.unwrap_or(Kind::A))
    };
    if kind == Kind::L {
        let high_tag = ctx.arg_tag(2).unwrap_or(tag::UNINIT);
        let mut call = Call::new(ctx);
        call.ret(3, value as u32, kind.out_tag());
        call.ret(2, (value >> 32) as u32, high_tag);
    } else {
        Call::new(ctx).ret(2, value as u32, kind.out_tag());
    }
    InterpStatus::Success
}

fn iter_key(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    iter_get(vm, ctx, true)
}

fn iter_data(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    iter_get(vm, ctx, false)
}

/// `equals(other)`: the same element (both at an end counts).
fn iter_equals(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, other) = (call.this(), call.word(2));
    let (Some((_, a, _)), Some((_, b, _))) = (iter_of(vm, this), iter_of(vm, other)) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(3, u32::from(a.node == b.node), INT_RESULT);
    InterpStatus::Success
}

fn iter_notdone(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some((_, pos, _)) = iter_of(vm, this) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(2, u32::from(pos.node.is_some()), INT_RESULT);
    InterpStatus::Success
}

fn iter_method(method: &str) -> Option<EFunFn> {
    Some(match method {
        "next" => iter_next,
        "key" => iter_key,
        "data" => iter_data,
        "equals" => iter_equals,
        "notdone" => iter_notdone,
        "remove" => iter_remove,
        _ => return None,
    })
}
