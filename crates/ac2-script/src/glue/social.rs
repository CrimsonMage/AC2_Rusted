//! The fellowship and allegiance adaptors' efuns (`gFellowAdaptor` 0x93,
//! `gFellowshipAdaptor` 0x94, `gAllegianceDataAdaptor` 0x82, `gAllegianceProfileAdaptor`
//! 0x83): setters the scripts fill an adaptor with before a `CM_Fellowship` /
//! `CM_Allegiance` notice hands it to the UI. Every wrapper crashes on an undefined `this`;
//! 64-bit ids come high word first; the `online` and officer flags are the argument's low byte.

use ac2_net::protocol::StringInfo;
use ac2_weenie::{AllegianceMember, AllegianceProfile, FellowInfo, FellowshipInfo};

use super::{word, word64, Call, INT_RESULT};
use crate::native::{social, NativeObject};
use crate::vm::{Context, EFuns, InterpStatus, Vm};
use crate::wire::string_info_to_net;

pub(super) fn register(efuns: &mut EFuns) {
    // gFellowAdaptor: one member.
    efuns.register(0x0093_0001, |vm, c| fellow_name(vm, c));
    for (id, _) in FELLOW_WORDS {
        efuns.register(0x0093_0000 | id, |vm, c| {
            let id = vm.current_efun() & 0xFF;
            let v = word(c, 2);
            let set = FELLOW_WORDS.iter().find(|(i, _)| *i == id).map(|(_, f)| *f);
            with_fellow(vm, c, |f, _| set.expect("registered")(f, v))
        });
    }
    // gFellowshipAdaptor.
    efuns.register(0x0094_0001, add_fellow);
    efuns.register(0x0094_0002, |vm, c| {
        let name = vm.wstring(word(c, 2)).unwrap_or(&[]).to_vec();
        with_fellowship(vm, c, |f| f.name = String::from_utf16_lossy(&name))
    });
    efuns.register(0x0094_0003, |vm, c| {
        let leader = word64(c, 3);
        with_fellowship(vm, c, |f| f.leader = leader)
    });
    efuns.register(0x0094_0004, |vm, c| {
        let v = word(c, 2) != 0;
        with_fellowship(vm, c, |f| f.social = v)
    });
    efuns.register(0x0094_0007, |vm, c| {
        let v = word(c, 2) as i32;
        with_fellowship(vm, c, |f| f.looting_method = v)
    });
    // gAllegianceDataAdaptor: one member.
    efuns.register(0x0082_0001, |vm, c| {
        let id = word64(c, 3);
        with_member(vm, c, |m| m.id = id)
    });
    efuns.register(0x0082_0002, |vm, c| {
        let s = string_info(vm, word(c, 2));
        with_member(vm, c, |m| m.name = s)
    });
    efuns.register(0x0082_0003, |vm, c| {
        let s = string_info(vm, word(c, 2));
        with_member(vm, c, |m| m.title = s)
    });
    efuns.register(0x0082_0004, |vm, c| {
        let xp = word64(c, 3);
        with_member(vm, c, |m| m.xp_passed = xp)
    });
    for (id, _) in MEMBER_WORDS {
        efuns.register(0x0082_0000 | id, |vm, c| {
            let id = vm.current_efun() & 0xFF;
            let v = word(c, 2);
            let set = MEMBER_WORDS.iter().find(|(i, _)| *i == id).map(|(_, f)| *f);
            with_member(vm, c, |m| set.expect("registered")(m, v))
        });
    }
    // gAllegianceProfileAdaptor.
    efuns.register(0x0083_0001, |vm, c| set_rank(vm, c, Rank::Monarch));
    efuns.register(0x0083_0002, |vm, c| set_rank(vm, c, Rank::Patron));
    efuns.register(0x0083_0003, |vm, c| set_rank(vm, c, Rank::Member));
    efuns.register(0x0083_0004, add_vassal);
    efuns.register(0x0083_0005, |vm, c| {
        let v = word(c, 2);
        with_profile(vm, c, |p, _| p.total = v)
    });
    efuns.register(0x0083_0006, |vm, c| {
        let s = string_info(vm, word(c, 2));
        with_profile(vm, c, |p, _| p.motd = s)
    });
    efuns.register(0x0083_0007, |vm, c| {
        let s = string_info(vm, word(c, 2));
        with_profile(vm, c, |p, _| p.name = s)
    });
    efuns.register(0x0083_0008, |vm, c| {
        let v = word(c, 2);
        with_profile(vm, c, |p, _| p.faction = v)
    });
    // `HashSet<InstanceID>::add`.
    efuns.register(0x0083_0009, |vm, c| {
        let id = word64(c, 3);
        with_profile(vm, c, |p, _| {
            if !p.officers.contains(&id) {
                p.officers.push(id);
            }
        })
    });
}

const FELLOW_WORDS: [(u32, fn(&mut FellowInfo, u32)); 8] = [
    (0x02, |f, v| f.level = v),
    (0x03, |f, v| f.health = v),
    (0x04, |f, v| f.max_health = v),
    (0x05, |f, v| f.vigor = v),
    (0x06, |f, v| f.max_vigor = v),
    (0x07, |f, v| f.cell = v),
    (0x08, |f, v| f.pk_damage = v),
    (0x09, |f, v| f.pk_vigor_loss = v),
];

const MEMBER_WORDS: [(u32, fn(&mut AllegianceMember, u32)); 6] = [
    (0x05, |m, v| m.level = v),
    (0x06, |m, v| m.gender = v),
    (0x07, |m, v| m.species = v),
    (0x08, |m, v| m.officer = v & 0xFF != 0),
    (0x09, |m, v| m.faction = v),
    (0x0A, |m, v| m.rank = v),
];

/// A `StringInfo` argument as ac2-net's (`StringInfo::operator=`).
fn string_info(vm: &Vm, reference: u32) -> StringInfo {
    match vm.native(reference) {
        Some(NativeObject::StringInfo(s)) => string_info_to_net(s),
        _ => StringInfo::default(),
    }
}

fn this_or_crash<T>(
    vm: &mut Vm,
    ctx: &mut Context,
    get: fn(&mut NativeObject) -> Option<&mut T>,
    f: impl FnOnce(&mut T, &mut Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this).and_then(get) {
        Some(t) => {
            f(t, ctx);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

fn with_fellow(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut FellowInfo, &mut Context),
) -> InterpStatus {
    this_or_crash(
        vm,
        ctx,
        |n| match n {
            NativeObject::Fellow(f) => Some(&mut **f),
            _ => None,
        },
        f,
    )
}

fn with_fellowship_ctx(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut FellowshipInfo, &mut Context),
) -> InterpStatus {
    this_or_crash(
        vm,
        ctx,
        |n| match n {
            NativeObject::Fellowship(f) => Some(&mut **f),
            _ => None,
        },
        f,
    )
}

fn with_fellowship(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut FellowshipInfo),
) -> InterpStatus {
    with_fellowship_ctx(vm, ctx, |t, _| f(t))
}

fn with_member(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut AllegianceMember),
) -> InterpStatus {
    this_or_crash(
        vm,
        ctx,
        |n| match n {
            NativeObject::AllegianceData(m) => Some(&mut **m),
            _ => None,
        },
        |t, _| f(t),
    )
}

fn with_profile(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut AllegianceProfile, &mut Context),
) -> InterpStatus {
    this_or_crash(
        vm,
        ctx,
        |n| match n {
            NativeObject::AllegianceProfile(p) => Some(&mut **p),
            _ => None,
        },
        f,
    )
}

/// `gFellowAdaptor::SetName(StringInfo)`.
fn fellow_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let s = string_info(vm, word(ctx, 2));
    with_fellow(vm, ctx, |f, _| f.name = s)
}

/// `FellowshipAdaptor::AddFellow(id, fellow)`: words 2-3 the id (high word first), word 4
/// the `FellowAdaptor`; whether it was added.
fn add_fellow(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = word64(ctx, 3);
    let info = match vm.native(word(ctx, 4)) {
        Some(NativeObject::Fellow(f)) => (**f).clone(),
        _ => social::fellow(),
    };
    with_fellowship_ctx(vm, ctx, |f, c| {
        let added = social::add_fellow(f, id, info);
        Call::new(c).ret(5, u32::from(added), INT_RESULT);
    })
}

enum Rank {
    Monarch,
    Patron,
    Member,
}

fn member_of(vm: &Vm, reference: u32) -> AllegianceMember {
    match vm.native(reference) {
        Some(NativeObject::AllegianceData(m)) => (**m).clone(),
        _ => social::allegiance_member(),
    }
}

/// `SetMonarch` / `SetPatron` / `SetMember(data, online)`.
fn set_rank(vm: &mut Vm, ctx: &mut Context, rank: Rank) -> InterpStatus {
    let m = member_of(vm, word(ctx, 2));
    let online = word(ctx, 3) & 0xFF != 0;
    with_profile(vm, ctx, |p, _| match rank {
        Rank::Monarch => (p.monarch, p.monarch_online) = (m, online),
        Rank::Patron => (p.patron, p.patron_online) = (m, online),
        Rank::Member => (p.member, p.member_online) = (m, online),
    })
}

/// `AddVassal(data, online)`: appended to both lists, ret 1.
fn add_vassal(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let m = member_of(vm, word(ctx, 2));
    let online = word(ctx, 3) & 0xFF != 0;
    with_profile(vm, ctx, |p, c| {
        p.vassals.push((m, online));
        Call::new(c).ret(4, 1, INT_RESULT);
    })
}
