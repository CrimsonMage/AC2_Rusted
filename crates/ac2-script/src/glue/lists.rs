//! The list and array glue (`gAList`, `gLList`, `gRList`, `gAArray`, `gLArray`, `gRArray` and
//! the list iterators `gAListIter`, `gLListIter`, `gRListIter`).
//!
//! The families share one implementation: the element kind comes from the `this` object's native
//! type (`A` 32-bit, `L` 64-bit, `R` a reference) and decides how many slots a value takes and
//! how out parameters are tagged (4, 0x84 on the first slot, 1). Lists are `List<T>` (node
//! lists: front/back pushes, positional `push`, last-match `remove`); arrays are `SmartArray<T>`
//! (`erase` moves the last element into the hole).

use super::Call;
use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFuns, Fault, InterpStatus, Vm};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    A,
    L,
    R,
}

impl Kind {
    fn of(native_type: u32) -> Self {
        match native_type {
            ntype::L_LIST | ntype::L_ARRAY | ntype::L_LIST_ITER => Self::L,
            ntype::R_LIST | ntype::R_ARRAY | ntype::R_LIST_ITER => Self::R,
            _ => Self::A,
        }
    }

    pub(super) fn out_tag(self) -> u8 {
        match self {
            Self::A => tag::BOOL,
            Self::L => tag::BOOL | tag::LONG,
            Self::R => tag::REF,
        }
    }
}

/// Registers every list, array and list-iterator efun by its retail name.
pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let Some((class, method)) = name.split_once("::CallEFun_") else {
            continue;
        };
        let f: Option<crate::vm::EFunFn> = match class {
            "gAList" | "gLList" | "gRList" => list_method(method),
            "gAArray" | "gLArray" | "gRArray" => array_method(method),
            "gAListIter" | "gLListIter" | "gRListIter" => iter_method(method),
            _ => None,
        };
        if let Some(f) = f {
            efuns.register(id, f);
        }
    }
}

/// The value `start` slots down (64-bit values take two, low word deeper) and the next slot.
fn value(ctx: &Context, start: usize, kind: Kind) -> (u64, usize) {
    if kind == Kind::L {
        (super::word64(ctx, start + 1), start + 2)
    } else {
        (u64::from(super::word(ctx, start)), start + 1)
    }
}

/// Runs `f` on `this` as a list or array (`index != Undef_ReferenceID` otherwise).
fn with_list(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut Vec<u64>, Kind, &mut Context) -> Result<u32, Fault>,
) -> (InterpStatus, u32) {
    let this = Call::new(ctx).this();
    let Ok(object) = vm.refs.heap_mut(this) else {
        return (vm.bad_this(this), 0);
    };
    let kind = Kind::of(object.native_type);
    match &mut object.native {
        Some(NativeObject::List(list)) => match f(&mut list.items, kind, ctx) {
            Ok(r) => (InterpStatus::Success, r),
            Err(fault) => {
                vm.last_fault = Some(fault);
                (InterpStatus::Crash, 0)
            }
        },
        _ => (vm.bad_this(this), 0),
    }
}

/// Writes an out parameter through reference `out`: the slot's tag always, the value if any.
pub(super) fn write_out(vm: &mut Vm, ctx: &mut Context, out: u32, value: Option<u64>, kind: Kind) {
    let (low, high) = value.map_or((None, None), |v| (Some(v as u32), Some((v >> 32) as u32)));
    let current = vm.read_through(ctx, out).unwrap_or(0);
    vm.write_through(ctx, out, low.unwrap_or(current), kind.out_tag());
    if kind == Kind::L {
        if let (Some(high), Ok(Some(slot))) = (high, vm.refs.locate(out, 1)) {
            if let Ok((_, t)) = vm.refs.read(ctx, slot) {
                let _ = vm.refs.write(ctx, slot, high, t);
            }
        }
    }
}

fn ret(ctx: &mut Context, slot: usize, value: u32) {
    Call::new(ctx).ret(slot, value, tag::BOOL);
}

macro_rules! list_op {
    ($name:ident, |$items:ident, $kind:ident, $ctx:ident| $body:expr) => {
        fn $name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
            with_list(vm, ctx, |$items, $kind, $ctx| $body).0
        }
    };
}

fn list_method(method: &str) -> Option<crate::vm::EFunFn> {
    Some(match method {
        "push_front" | "push_head" | "insert_front" | "insert_head" => push_front,
        "push_back" | "push_tail" | "insert_back" | "insert_tail" => push_back,
        "push_unique_front" | "push_unique_head" | "insert_unique_front" | "insert_unique_head" => {
            push_unique_front
        }
        "push_unique_back" | "push_unique_tail" | "insert_unique_back" | "insert_unique_tail" => {
            push_unique_back
        }
        "insert" | "push" => push_at,
        "pop_front" | "pop_head" | "remove_front" | "remove_head" => pop_front,
        "pop_back" | "pop_tail" | "remove_back" | "remove_tail" => pop_back,
        "peek_front" | "peek_head" => peek_front,
        "peek_back" | "peek_tail" => peek_back,
        "erase_front" | "erase_head" => erase_front,
        "erase_back" | "erase_tail" => erase_back,
        "remove" => remove_last,
        "contains" => contains,
        "get_position" => get_position,
        "flush" => flush,
        "empty" => empty,
        "num_elements" => num_elements,
        "inq_data" => inq_data,
        "insert_sort_int" => insert_sort_int,
        "insert_sort_float" => insert_sort_float,
        "insert_sort_ulint" => insert_sort_ulint,
        "get_begin" => get_begin,
        "get_reverse" => get_reverse,
        "get_end" => get_end,
        "clone" => clone,
        _ => return None,
    })
}

list_op!(push_front, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    items.insert(0, v);
    ret(ctx, next, 1);
    Ok(1)
});

list_op!(push_back, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    items.push(v);
    ret(ctx, next, 1);
    Ok(1)
});

list_op!(push_unique_front, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    let added = !items.contains(&v);
    if added {
        items.insert(0, v);
    }
    ret(ctx, next, u32::from(added));
    Ok(0)
});

list_op!(push_unique_back, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    let added = !items.contains(&v);
    if added {
        items.push(v);
    }
    ret(ctx, next, u32::from(added));
    Ok(0)
});

// `List::push(pos, data)`: before the element at `pos`, or at the end when `pos` is the count.
list_op!(push_at, |items, kind, ctx| {
    let pos = Call::new(ctx).word(2) as usize;
    let (v, next) = value(ctx, 3, kind);
    let ok = pos <= items.len();
    if ok {
        items.insert(pos, v);
    }
    ret(ctx, next, u32::from(ok));
    Ok(0)
});

fn pop_front(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    take_value(vm, ctx, true, true)
}
fn pop_back(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    take_value(vm, ctx, false, true)
}
fn peek_front(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    take_value(vm, ctx, true, false)
}
fn peek_back(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    take_value(vm, ctx, false, false)
}

/// Pops or peeks the front or back element into the out reference at slot 2 (result slot 3).
fn take_value(vm: &mut Vm, ctx: &mut Context, front: bool, remove: bool) -> InterpStatus {
    let out = Call::new(ctx).word(2);
    let mut taken = None;
    let mut list_kind = Kind::A;
    let (status, _) = with_list(vm, ctx, |items, kind, _| {
        list_kind = kind;
        taken = if front { items.first() } else { items.last() }.copied();
        if remove && taken.is_some() {
            if front {
                items.remove(0);
            } else {
                items.pop();
            }
        }
        Ok(0)
    });
    if status != InterpStatus::Success {
        return status;
    }
    write_out(vm, ctx, out, taken, list_kind);
    ret(ctx, 3, u32::from(taken.is_some()));
    InterpStatus::Success
}

list_op!(erase_front, |items, _kind, ctx| {
    let ok = !items.is_empty();
    if ok {
        items.remove(0);
    }
    ret(ctx, 2, u32::from(ok));
    Ok(0)
});

list_op!(erase_back, |items, _kind, ctx| {
    let ok = items.pop().is_some();
    ret(ctx, 2, u32::from(ok));
    Ok(0)
});

// `gAList::CallEFun_remove`: the last matching element.
list_op!(remove_last, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    let found = items.iter().rposition(|x| *x == v);
    if let Some(i) = found {
        items.remove(i);
    }
    ret(ctx, next, u32::from(found.is_some()));
    Ok(0)
});

list_op!(contains, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    let found = items.contains(&v);
    ret(ctx, next, u32::from(found));
    Ok(0)
});

list_op!(get_position, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    let at = items
        .iter()
        .position(|x| *x == v)
        .map_or(u32::MAX, |i| i as u32);
    ret(ctx, next, at);
    Ok(0)
});

list_op!(flush, |items, _kind, _ctx| {
    items.clear();
    Ok(0)
});

list_op!(empty, |items, _kind, ctx| {
    ret(ctx, 2, u32::from(items.is_empty()));
    Ok(0)
});

list_op!(num_elements, |items, _kind, ctx| {
    ret(ctx, 2, items.len() as u32);
    Ok(0)
});

/// `BaseList::inq_data(pos, out)` / `SmartArray` query: element `pos` into the out reference.
fn inq_data(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (pos, out) = (call.word(2) as usize, call.word(3));
    let mut found = None;
    let mut list_kind = Kind::A;
    let (status, _) = with_list(vm, ctx, |items, kind, _| {
        list_kind = kind;
        found = items.get(pos).copied();
        Ok(0)
    });
    if status != InterpStatus::Success {
        return status;
    }
    write_out(vm, ctx, out, found, list_kind);
    ret(ctx, 4, u32::from(found.is_some()));
    InterpStatus::Success
}

/// `BaseList::insert_sort_*`: before the first element the value is less than, else at the end.
fn insert_sorted(items: &mut Vec<u64>, v: u64, less: impl Fn(u64, u64) -> bool) {
    let at = items
        .iter()
        .position(|&x| less(v, x))
        .unwrap_or(items.len());
    items.insert(at, v);
}

list_op!(insert_sort_int, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    insert_sorted(items, v, |a, b| (a as u32 as i32) < (b as u32 as i32));
    ret(ctx, next, 1);
    Ok(0)
});

list_op!(insert_sort_float, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    let f = |x: u64| f32::from_bits(x as u32);
    insert_sorted(items, v, |a, b| f(a) < f(b));
    ret(ctx, next, 1);
    Ok(0)
});

list_op!(insert_sort_ulint, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    insert_sorted(items, v, |a, b| a < b);
    ret(ctx, next, 1);
    Ok(0)
});

/// `BaseList::get_begin/get_reverse/get_end`: a new iterator native on this list.
fn make_iter(vm: &mut Vm, ctx: &mut Context, at: fn(usize) -> Option<usize>) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Ok(object) = vm.refs.heap(this) else {
        return vm.bad_this(this);
    };
    let len = match &object.native {
        Some(NativeObject::List(l)) => l.items.len(),
        _ => return vm.bad_this(this),
    };
    let iter_type = object.native_type + 1;
    let r = vm.refs.create_native(
        iter_type,
        NativeObject::ListIter {
            list: this,
            pos: at(len),
        },
    );
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

fn get_begin(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    make_iter(vm, ctx, |len| (len > 0).then_some(0))
}
fn get_reverse(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    make_iter(vm, ctx, |len| len.checked_sub(1))
}
fn get_end(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    make_iter(vm, ctx, |_| None)
}

/// `List::operator=` into the list the out reference holds; result 1.
fn clone(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, out) = (call.this(), call.word(2));
    let items = match vm.native(this) {
        Some(NativeObject::List(l)) => l.items.clone(),
        _ => return vm.bad_this(this),
    };
    let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
    if let Some(NativeObject::List(l)) = vm.native_mut(target) {
        l.items = items;
    }
    ret(ctx, 3, 1);
    InterpStatus::Success
}

fn array_method(method: &str) -> Option<crate::vm::EFunFn> {
    Some(match method {
        "SetNElements" => set_n_elements,
        "push" => push_back_array,
        "push_unique" => push_unique_back,
        "get" => array_get,
        "query" => inq_data,
        "set" => array_set,
        "pop" => pop_back,
        "erase" => array_erase,
        "contains" => contains,
        "flush" => flush,
        "empty" => empty,
        "num_elements" => num_elements,
        _ => return None,
    })
}

// `SmartArray::grow` + `m_num = n`; new elements read as 0.
list_op!(set_n_elements, |items, _kind, ctx| {
    let n = Call::new(ctx).word(2) as usize;
    items.resize(n, 0);
    ret(ctx, 3, 1);
    Ok(0)
});

list_op!(push_back_array, |items, kind, ctx| {
    let (v, next) = value(ctx, 2, kind);
    items.push(v);
    ret(ctx, next, 1);
    Ok(0)
});

// `get(i)`: unchecked in retail; out of range crashes the context here.
list_op!(array_get, |items, kind, ctx| {
    let i = Call::new(ctx).word(2) as usize;
    let v = *items.get(i).ok_or(Fault::BadReference(i as u32))?;
    let mut call = Call::new(ctx);
    if kind == Kind::L {
        call.ret(4, v as u32, kind.out_tag());
        call.ret(3, (v >> 32) as u32, kind.out_tag());
    } else {
        call.ret(3, v as u32, kind.out_tag());
    }
    Ok(0)
});

list_op!(array_set, |items, kind, ctx| {
    let i = Call::new(ctx).word(2) as usize;
    let (v, next) = value(ctx, 3, kind);
    let ok = i < items.len();
    if ok {
        items[i] = v;
    }
    ret(ctx, next, u32::from(ok));
    Ok(0)
});

// `erase(i)`: the last element moves into the hole.
list_op!(array_erase, |items, _kind, ctx| {
    let i = Call::new(ctx).word(2) as usize;
    let ok = i < items.len();
    if ok {
        items.swap_remove(i);
    }
    ret(ctx, 3, u32::from(ok));
    Ok(0)
});

fn iter_method(method: &str) -> Option<crate::vm::EFunFn> {
    Some(match method {
        "next" => iter_next,
        "prev" => iter_prev,
        "data" => iter_data,
        "remove" => iter_remove,
        "insert" => iter_insert,
        "equals" => iter_equals,
        "notdone" => iter_notdone,
        _ => return None,
    })
}

/// `this` as an iterator: (list reference, position, element kind).
fn iter_of(vm: &Vm, ctx: &Context) -> Option<(u32, Option<usize>, Kind)> {
    let this = super::word(ctx, 1);
    let object = vm.refs.heap(this).ok()?;
    match object.native {
        Some(NativeObject::ListIter { list, pos }) => {
            Some((list, pos, Kind::of(object.native_type)))
        }
        _ => None,
    }
}

fn set_iter_pos(vm: &mut Vm, ctx: &Context, pos: Option<usize>) {
    let this = super::word(ctx, 1);
    if let Some(NativeObject::ListIter { pos: p, .. }) = vm.native_mut(this) {
        *p = pos;
    }
}

fn list_len(vm: &Vm, list: u32) -> usize {
    match vm.native(list) {
        Some(NativeObject::List(l)) => l.items.len(),
        _ => 0,
    }
}

fn iter_next(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((list, pos, _)) = iter_of(vm, ctx) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let len = list_len(vm, list);
    set_iter_pos(vm, ctx, pos.and_then(|p| (p + 1 < len).then_some(p + 1)));
    InterpStatus::Success
}

fn iter_prev(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((_, pos, _)) = iter_of(vm, ctx) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    set_iter_pos(vm, ctx, pos.and_then(|p| p.checked_sub(1)));
    InterpStatus::Success
}

/// The current element; a finished iterator dereferences null in retail (crash here).
fn iter_data(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((list, pos, kind)) = iter_of(vm, ctx) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let v = match (vm.native(list), pos) {
        (Some(NativeObject::List(l)), Some(p)) => l.items.get(p).copied(),
        _ => None,
    };
    let Some(v) = v else {
        return vm.fault(Fault::BadReference(list));
    };
    let mut call = Call::new(ctx);
    if kind == Kind::L {
        call.ret(3, v as u32, kind.out_tag());
        call.ret(2, (v >> 32) as u32, kind.out_tag());
    } else {
        call.ret(2, v as u32, kind.out_tag());
    }
    InterpStatus::Success
}

/// Removes the current element; the iterator moves to the next one.
fn iter_remove(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((list, pos, _)) = iter_of(vm, ctx) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let mut removed = false;
    if let (Some(NativeObject::List(l)), Some(p)) = (vm.native_mut(list), pos) {
        if p < l.items.len() {
            l.items.remove(p);
            removed = true;
        }
    }
    let len = list_len(vm, list);
    set_iter_pos(vm, ctx, pos.filter(|&p| p < len));
    ret(ctx, 2, u32::from(removed));
    InterpStatus::Success
}

/// Inserts before the current element (at the end when finished); the iterator keeps its node.
fn iter_insert(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((list, pos, kind)) = iter_of(vm, ctx) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let (v, next) = value(ctx, 2, kind);
    let mut ok = false;
    if let Some(NativeObject::List(l)) = vm.native_mut(list) {
        let at = pos.unwrap_or(l.items.len()).min(l.items.len());
        l.items.insert(at, v);
        ok = true;
    }
    if ok {
        set_iter_pos(vm, ctx, pos.map(|p| p + 1));
    }
    ret(ctx, next, u32::from(ok));
    InterpStatus::Success
}

fn iter_equals(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((list, pos, _)) = iter_of(vm, ctx) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let other = Call::new(ctx).word(2);
    let same = match vm.native(other) {
        Some(NativeObject::ListIter { list: l, pos: p }) => {
            // Retail compares node pointers: two finished iterators are equal on any list.
            *p == pos && (pos.is_none() || *l == list)
        }
        _ => false,
    };
    ret(ctx, 3, u32::from(same));
    InterpStatus::Success
}

fn iter_notdone(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((_, pos, _)) = iter_of(vm, ctx) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    ret(ctx, 2, u32::from(pos.is_some()));
    InterpStatus::Success
}
