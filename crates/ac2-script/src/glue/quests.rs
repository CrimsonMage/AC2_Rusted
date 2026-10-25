//! `ggmQuestInfo` (0xA00001-0xA00009) and `ggmQuestInfoList` (0xA10001-0xA10007): the quests
//! the player's script keeps (`gmQuestInfo`, `gmQuestInfoList`, `List<gmQuestInfo>`), over
//! ac2-net's `GmQuestInfo`. Every wrapper crashes on an undefined `this`. `DumpW` (a debug
//! dump) is not bound.

use ac2_net::protocol::{GmQuestInfo, NativeObject as Wire, StringInfo};

use super::{word, word64, Call, INT_RESULT};
use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};
use crate::wire::{string_info_from_net, string_info_to_net};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x00A0_0001, |vm, c| get_word(vm, c, |q| q.id));
    efuns.register(0x00A0_0002, |vm, c| get_text(vm, c, |q| &q.name));
    efuns.register(0x00A0_0003, |vm, c| get_text(vm, c, |q| &q.journal));
    efuns.register(0x00A0_0005, init);
    efuns.register(0x00A0_0006, |vm, c| get_text(vm, c, |q| &q.description));
    efuns.register(0x00A0_0007, |vm, c| {
        get_word(vm, c, |q| u32::from(q.play_fx_on_update))
    });
    efuns.register(0x00A0_0009, |vm, c| get_word(vm, c, |q| q.status));
    efuns.register(0x00A1_0001, push_tail);
    efuns.register(0x00A1_0002, update_quest_info);
    efuns.register(0x00A1_0003, find);
    efuns.register(0x00A1_0004, erase);
    efuns.register(0x00A1_0005, pop_head);
    efuns.register(0x00A1_0006, |vm, c| {
        let this = Call::new(c).this();
        let Some(n) = list(vm, this).map(|l| l.len() as u32) else {
            return vm.bad_this(this);
        };
        Call::new(c).ret(2, n, INT_RESULT);
        InterpStatus::Success
    });
}

fn quest(vm: &Vm, reference: u32) -> Option<&GmQuestInfo> {
    match vm.native(reference)? {
        NativeObject::Wire(w) => match &**w {
            Wire::GmQuestInfo(q) => Some(q),
            _ => None,
        },
        _ => None,
    }
}

fn quest_mut(vm: &mut Vm, reference: u32) -> Option<&mut GmQuestInfo> {
    match vm.native_mut(reference)? {
        NativeObject::Wire(w) => match &mut **w {
            Wire::GmQuestInfo(q) => Some(q),
            _ => None,
        },
        _ => None,
    }
}

fn list(vm: &Vm, reference: u32) -> Option<&Vec<GmQuestInfo>> {
    match vm.native(reference)? {
        NativeObject::Wire(w) => match &**w {
            Wire::GmQuestInfoList(l) => Some(l),
            _ => None,
        },
        _ => None,
    }
}

fn list_mut(vm: &mut Vm, reference: u32) -> Option<&mut Vec<GmQuestInfo>> {
    match vm.native_mut(reference)? {
        NativeObject::Wire(w) => match &mut **w {
            Wire::GmQuestInfoList(l) => Some(l),
            _ => None,
        },
        _ => None,
    }
}

fn get_word(vm: &mut Vm, ctx: &mut Context, f: fn(&GmQuestInfo) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(v) = quest(vm, this).map(f) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(2, v, INT_RESULT);
    InterpStatus::Success
}

/// The `StringInfo` getters: a copy as a new native (`VM::CopyNativeObject(0x6A)`).
fn get_text(vm: &mut Vm, ctx: &mut Context, f: fn(&GmQuestInfo) -> &StringInfo) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(text) = quest(vm, this).map(|q| string_info_from_net(f(q))) else {
        return vm.bad_this(this);
    };
    let r = vm
        .refs
        .create_native(ntype::STRING_INFO, NativeObject::StringInfo(text));
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

fn text_arg(vm: &Vm, reference: u32) -> StringInfo {
    match vm.native(reference) {
        Some(NativeObject::StringInfo(s)) => string_info_to_net(s),
        _ => StringInfo::default(),
    }
}

/// `gmQuestInfo::Init(questID, name, description, icon, phase, journal, status, challenge,
/// bestowal, maxedOut, expires, secondsRemaining, done, secondsUntilRetry, playFX)`; the
/// doubles take two words, high word first.
fn init(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let f = |n: usize| f64::from_bits(word64(ctx, n));
    let q = GmQuestInfo {
        id: word(ctx, 2),
        name: text_arg(vm, word(ctx, 3)),
        description: text_arg(vm, word(ctx, 4)),
        icon: word(ctx, 5),
        phase: word(ctx, 6),
        journal: text_arg(vm, word(ctx, 7)),
        status: word(ctx, 8),
        challenge_level: word(ctx, 9) as i32,
        bestowal: f(11),
        maxed_out: word(ctx, 12) != 0,
        expires: word(ctx, 13) != 0,
        seconds_remaining: f(15),
        done: f(17),
        seconds_until_retry: f(19),
        play_fx_on_update: word(ctx, 20) != 0,
    };
    let this = Call::new(ctx).this();
    match quest_mut(vm, this) {
        Some(slot) => {
            *slot = q;
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// Runs `f` on `this` as a quest list; `f` gives the result for `slot`.
fn with_list(
    vm: &mut Vm,
    ctx: &mut Context,
    slot: usize,
    f: impl FnOnce(&mut Vec<GmQuestInfo>) -> u32,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(l) = list_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let v = f(l);
    Call::new(ctx).ret(slot, v, INT_RESULT);
    InterpStatus::Success
}

/// `List<gmQuestInfo>::push_back`.
fn push_tail(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let q = quest(vm, word(ctx, 2)).cloned().unwrap_or_default();
    with_list(vm, ctx, 3, |l| {
        l.push(q);
        1
    })
}

/// `List<gmQuestInfo>::insert_sort`: before the first quest bestowed earlier (newest first).
fn insert_sort(l: &mut Vec<GmQuestInfo>, q: GmQuestInfo) {
    let at = l
        .iter()
        .position(|n| q.bestowal > n.bestowal)
        .unwrap_or(l.len());
    l.insert(at, q);
}

/// `gmQuestInfoList::UpdateQuestInfo(quest, out bNew)`: the quest with the same id updated
/// (`gmQuestInfo::Update`), else inserted sorted; `bNew` says which.
fn update_quest_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let q = quest(vm, word(ctx, 2)).cloned().unwrap_or_default();
    let out = word(ctx, 3);
    let mut new = false;
    let status = with_list(vm, ctx, 4, |l| {
        match l.iter_mut().find(|n| n.id == q.id) {
            Some(n) => *n = q,
            None => {
                insert_sort(l, q);
                new = true;
            }
        }
        1
    });
    if status == InterpStatus::Success {
        vm.write_through(ctx, out, u32::from(new), tag::BOOL);
    }
    status
}

/// `gmQuestInfoList::Find(questID, out quest)`: the quest copied out when there; always 1.
fn find(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (id, out) = (word(ctx, 2), word(ctx, 3));
    let this = Call::new(ctx).this();
    let Some(found) = list(vm, this).map(|l| l.iter().find(|q| q.id == id).cloned()) else {
        return vm.bad_this(this);
    };
    if let Some(found) = found {
        let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
        if let Some(q) = quest_mut(vm, target) {
            *q = found;
        }
    }
    Call::new(ctx).ret(4, 1, INT_RESULT);
    InterpStatus::Success
}

/// `gmQuestInfoList::Erase(quest)`: the last quest with its id removed.
fn erase(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = quest(vm, word(ctx, 2)).map_or(0, |q| q.id);
    with_list(vm, ctx, 3, |l| match l.iter().rposition(|q| q.id == id) {
        Some(at) => {
            l.remove(at);
            1
        }
        None => 0,
    })
}

/// `List<gmQuestInfo>::pop_front(out quest)`.
fn pop_head(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let out = word(ctx, 2);
    let mut head = None;
    let status = with_list(vm, ctx, 3, |l| {
        if l.is_empty() {
            return 0;
        }
        head = Some(l.remove(0));
        1
    });
    if let Some(head) = head {
        let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
        if let Some(q) = quest_mut(vm, target) {
            *q = head;
        }
    }
    status
}
