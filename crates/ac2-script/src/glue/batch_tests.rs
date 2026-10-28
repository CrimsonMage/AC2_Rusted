//! `gPropertyCollection` / `gPropertyGroup`, `gGameTime`, `gCollisionParameters`' mask efuns and
//! `gEtherealPhysicsType` / `gEtherealWeenieType` through the efun calling convention.

use std::sync::Arc;

use super::*;
use crate::host::game_time::{GameDate, GameTimeRequest, GameTimeState};
use crate::vm::{NoData, Program};
use ac2_db::{BaseProperty, BasePropertyValue};

fn vm() -> Vm {
    let mut efuns = EFuns::new();
    register_pure(&mut efuns);
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    Vm::new(Arc::new(program), Box::new(NoData), efuns)
}

fn id(name: &str) -> u32 {
    efun_table().find(|(_, n)| *n == name).unwrap().0
}

/// Calls efun `name` with `ret_slots` return slots, `args` and `this`; gives back the status and
/// the return slots (first slot first).
fn call(
    vm: &mut Vm,
    name: &str,
    args: &[u32],
    this: u32,
    ret_slots: usize,
) -> (InterpStatus, Vec<u32>) {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    for _ in 0..ret_slots {
        ctx.push(0, tag::INT).unwrap();
    }
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    let status = vm.run_efun(id(name), &mut ctx);
    (status, ctx.data[..ret_slots].to_vec())
}

fn word(vm: &mut Vm, name: &str, args: &[u32], this: u32) -> u32 {
    let (status, ret) = call(vm, name, args, this, 1);
    assert_eq!(status, InterpStatus::Success, "{name}");
    ret[0]
}

fn property(name: u32, group: u32, value: i32) -> BaseProperty {
    BaseProperty {
        name,
        value_type: 0x4000_0002,
        group,
        value: Some(BasePropertyValue::Integer(value)),
    }
}

fn property_ref(vm: &mut Vm, p: BaseProperty) -> u32 {
    vm.refs.create_native(
        ntype::BASE_PROPERTY,
        NativeObject::BaseProperty(Box::new(p)),
    )
}

fn new_native(vm: &mut Vm, native_type: u32) -> u32 {
    vm.refs
        .create_native(native_type, NativeObject::new_empty(native_type))
}

#[test]
fn a_collection_adds_properties_into_their_groups_and_walks_them() {
    let mut vm = vm();
    let collection = new_native(&mut vm, ntype::PROPERTY_COLLECTION);
    let a = property_ref(&mut vm, property(0x10, 7, 1));
    let b = property_ref(&mut vm, property(0x11, 7, 2));
    let c = property_ref(&mut vm, property(0x12, 8, 3));
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_AddProperty",
            &[a],
            collection
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_AddProperty",
            &[b],
            collection
        ),
        1
    );
    // The same name again is refused.
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_AddProperty",
            &[a],
            collection
        ),
        0
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_AddOptionalProperty",
            &[c],
            collection
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_GetNumGroups",
            &[],
            collection
        ),
        2
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_ContainsGroup",
            &[8],
            collection
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_ContainsGroup",
            &[9],
            collection
        ),
        0
    );
    // Group 0 holds two required properties, read back through a group native.
    let group = new_native(&mut vm, ntype::PROPERTY_GROUP);
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_InqGroupAt",
            &[0, group],
            collection
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_InqGroupAt",
            &[2, group],
            collection
        ),
        0,
        "past the end the out group stays"
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_GetNumProperties",
            &[],
            group
        ),
        2
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_GetNumOptionalProperties",
            &[],
            group
        ),
        0
    );
    let out = new_native(&mut vm, ntype::BASE_PROPERTY);
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_InqPropertyAt",
            &[1, out],
            group
        ),
        1
    );
    let Some(NativeObject::BaseProperty(p)) = vm.native(out) else {
        panic!("a BaseProperty");
    };
    assert_eq!(p.name, 0x11);
    assert_eq!(p.value, Some(BasePropertyValue::Integer(2)));
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_InqPropertyAt",
            &[2, out],
            group
        ),
        0
    );
    // The optional property sits in group 8's optional list.
    assert_eq!(
        word(
            &mut vm,
            "gPropertyCollection::CallEFun_InqGroup",
            &[8, group],
            collection
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_ContainsOptionalProperty",
            &[0x12],
            group
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_ContainsProperty",
            &[0x12],
            group
        ),
        0
    );
}

#[test]
fn a_group_updates_and_removes_properties() {
    let mut vm = vm();
    let group = new_native(&mut vm, ntype::PROPERTY_GROUP);
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_SetGroupName",
            &[5],
            group
        ),
        1
    );
    let a = property_ref(&mut vm, property(0x10, 5, 1));
    let a2 = property_ref(&mut vm, property(0x10, 5, 9));
    let b = property_ref(&mut vm, property(0x11, 5, 2));
    assert_eq!(
        word(&mut vm, "gPropertyGroup::CallEFun_AddProperty", &[a], group),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_AddProperty",
            &[a2],
            group
        ),
        0
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_UpdateProperty",
            &[a2],
            group
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_UpdateProperty",
            &[b],
            group
        ),
        1
    );
    let out = new_native(&mut vm, ntype::BASE_PROPERTY);
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_InqProperty",
            &[0x10, out],
            group
        ),
        1
    );
    let Some(NativeObject::BaseProperty(p)) = vm.native(out) else {
        panic!("a BaseProperty");
    };
    assert_eq!(p.value, Some(BasePropertyValue::Integer(9)));
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_RemoveProperty",
            &[0x10],
            group
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_RemoveProperty",
            &[0x10],
            group
        ),
        0
    );
    assert_eq!(
        word(
            &mut vm,
            "gPropertyGroup::CallEFun_GetNumProperties",
            &[],
            group
        ),
        1
    );
    // A name the enum lacks answers 0 and leaves the out string empty.
    let name = vm.new_wstring(vec![1, 2]);
    let (status, ret) = call(
        &mut vm,
        "gPropertyGroup::CallEFun_InqPropertyGroupNameW",
        &[name],
        group,
        1,
    );
    assert_eq!(status, InterpStatus::Success);
    assert_eq!(ret[0], 0);
    assert_eq!(vm.wstring(name), Some(&[][..]));
}

#[test]
fn an_undefined_this_crashes_a_property_efun() {
    let mut vm = vm();
    let (status, _) = call(
        &mut vm,
        "gPropertyCollection::CallEFun_GetNumGroups",
        &[],
        u32::MAX,
        1,
    );
    assert_eq!(status, InterpStatus::Crash);
}

fn state() -> GameTimeState {
    GameTimeState {
        night: true,
        time_of_day: 0.25,
        day_length: 7200.0,
        days_per_year: 360,
        year: 3,
        day_of_year: 40,
        date: GameDate {
            year: 3,
            day: 5,
            year_spec: 0x10,
            season: 0x20,
            time: 0x30,
        },
    }
}

#[test]
fn the_game_time_getters_read_the_clock_the_host_set() {
    let mut vm = vm();
    let name = |n: &str| format!("gGameTime::CallEFun_{n}");
    assert_eq!(
        word(&mut vm, &name("IsNight"), &[], 0),
        0,
        "no game time answers 0"
    );
    vm.game_time = Some(state());
    assert_eq!(word(&mut vm, &name("IsNight"), &[], 0), 1);
    assert_eq!(
        word(&mut vm, &name("GetDayLength"), &[], 0),
        7200.0_f32.to_bits()
    );
    assert_eq!(word(&mut vm, &name("GetDayInYear"), &[], 0), 40);
    assert_eq!(word(&mut vm, &name("GetYearLength"), &[], 0), 360);
    assert_eq!(word(&mut vm, &name("GetYear"), &[], 0), 3);
    let (status, ret) = call(&mut vm, &name("GetTimeOfDay"), &[], 0, 2);
    assert_eq!(status, InterpStatus::Success);
    let bits = u64::from(ret[0]) | u64::from(ret[1]) << 32;
    assert_eq!(f64::from_bits(bits), 0.25);
}

#[test]
fn the_game_time_changes_queue_requests_for_the_client() {
    let mut vm = vm();
    let name = |n: &str| format!("gGameTime::CallEFun_{n}");
    assert_eq!(
        word(
            &mut vm,
            &name("AlterTimeOfDayImmediate"),
            &[0.5_f32.to_bits()],
            0
        ),
        0
    );
    vm.game_time = Some(state());
    assert_eq!(
        word(
            &mut vm,
            &name("AlterTimeOfDayImmediate"),
            &[0.5_f32.to_bits()],
            0
        ),
        1
    );
    assert_eq!(
        word(
            &mut vm,
            &name("TemporalDilation"),
            &[0.0002_f32.to_bits()],
            0
        ),
        0
    );
    assert_eq!(
        word(&mut vm, &name("TemporalDilation"), &[2.0_f32.to_bits()], 0),
        1
    );
    assert_eq!(
        vm.game_time_requests,
        [
            GameTimeRequest::AlterTimeOfDay(0.5),
            GameTimeRequest::TemporalDilation(2.0)
        ]
    );
}

#[test]
fn the_current_date_and_time_strings_carry_the_clock_variables() {
    let mut vm = vm();
    let date = new_native(&mut vm, ntype::STRING_INFO);
    let time = new_native(&mut vm, ntype::STRING_INFO);
    let name = "gGameTime::CallEFun_InqCurrentGameDateTimeString";
    assert_eq!(word(&mut vm, name, &[date, time], 0), 0);
    vm.game_time = Some(state());
    assert_eq!(word(&mut vm, name, &[date, time], 0), 1);
    let Some(NativeObject::StringInfo(info)) = vm.native(date) else {
        panic!("a StringInfo");
    };
    let token = |s: &str| super::wpstring::tokenize_wide(&s.encode_utf16().collect::<Vec<_>>());
    assert_eq!(info.string_id, token("ID_DateString"));
    let ids: Vec<u32> = info.variables.iter().map(|v| v.id).collect();
    assert_eq!(
        ids,
        [
            token("YEAR"),
            token("YEARSPEC"),
            token("DAY"),
            token("SEASON")
        ]
    );
    let Some(NativeObject::StringInfo(info)) = vm.native(time) else {
        panic!("a StringInfo");
    };
    assert_eq!(info.string_id, token("ID_TimeString"));
    assert_eq!(info.variables.len(), 1);
}

#[test]
fn physics_type_natives_set_get_and_edit_the_words() {
    let mut vm = vm();
    let params = new_native(&mut vm, ntype::COLLISION_PARAMETERS);
    let mask = new_native(&mut vm, ntype::ETHEREAL_PHYSICS_TYPE);
    let ethereal = "gEtherealPhysicsType::CallEFun_SetToEthereal";
    let (status, _) = call(&mut vm, ethereal, &[], mask, 0);
    assert_eq!(status, InterpStatus::Success);
    let (status, _) = call(
        &mut vm,
        "gCollisionParameters::CallEFun_SetPhysicsType",
        &[mask],
        params,
        0,
    );
    assert_eq!(status, InterpStatus::Success);
    let got = word(
        &mut vm,
        "gCollisionParameters::CallEFun_GetPhysicsType",
        &[],
        params,
    );
    assert_eq!(
        vm.native(got),
        Some(&NativeObject::EtherealPhysicsType(
            ac2_physics::PhysicsTypeMask::from_bits(u64::MAX)
        ))
    );
    // The movement set was not touched.
    let moved = word(
        &mut vm,
        "gCollisionParameters::CallEFun_GetPhysicsMovementType",
        &[],
        params,
    );
    assert_eq!(
        vm.native(moved),
        Some(&NativeObject::EtherealPhysicsType(
            ac2_physics::PhysicsTypeMask::default()
        ))
    );
    // A property the table lacks changes nothing and answers 0.
    assert_eq!(
        word(
            &mut vm,
            "gEtherealPhysicsType::CallEFun_SetPropertyEthereal",
            &[0x7EAD_BEEF],
            mask
        ),
        0
    );
}

#[test]
fn the_dword_setters_write_one_word_and_sign_extend_the_low_one() {
    let mut vm = vm();
    let params = new_native(&mut vm, ntype::COLLISION_PARAMETERS);
    let set = |vm: &mut Vm, name: &str, value: u32| {
        let (status, _) = call(
            vm,
            &format!("gCollisionParameters::CallEFun_{name}"),
            &[value],
            params,
            0,
        );
        assert_eq!(status, InterpStatus::Success);
    };
    let read = |vm: &mut Vm| match vm.native(params) {
        Some(NativeObject::CollisionParameters(p)) => p.sets.movement_physics().bits(),
        _ => panic!("parameters"),
    };
    set(&mut vm, "SetMovementHighDWord", 0x1234);
    assert_eq!(read(&mut vm), 0x1234_0000_0000);
    set(&mut vm, "SetMovementLowDWord", 0x8000_0001);
    // The low word is replaced; the high word gains the sign extension.
    assert_eq!(read(&mut vm), 0xFFFF_FFFF_8000_0001);
    set(&mut vm, "SetMovementHighDWord", 5);
    assert_eq!(read(&mut vm), 0x0000_0005_8000_0001);
}

#[test]
fn weenie_type_natives_track_masks_and_lists() {
    let mut vm = vm();
    let params = new_native(&mut vm, ntype::COLLISION_PARAMETERS);
    let types = new_native(&mut vm, ntype::ETHEREAL_WEENIE_TYPE);
    let (status, _) = call(
        &mut vm,
        "gEtherealWeenieType::CallEFun_SetToSolid",
        &[],
        types,
        0,
    );
    assert_eq!(status, InterpStatus::Success);
    // The 64-bit id 0x1_0000_0042: high word at word 2, low at word 3.
    assert_eq!(
        word(
            &mut vm,
            "gEtherealWeenieType::CallEFun_AddToEtherealList",
            &[1, 0x42],
            types
        ),
        1
    );
    let (status, _) = call(
        &mut vm,
        "gCollisionParameters::CallEFun_SetWeenieMovementType",
        &[types],
        params,
        0,
    );
    assert_eq!(status, InterpStatus::Success);
    let got = word(
        &mut vm,
        "gCollisionParameters::CallEFun_GetWeenieMovementType",
        &[],
        params,
    );
    let Some(NativeObject::EtherealWeenieType(held)) = vm.native(got) else {
        panic!("an EtherealWeenieType");
    };
    assert!(held.in_ethereal_list(0x1_0000_0042));
    assert_eq!(held.mask(), u64::MAX);
}

#[test]
fn a_registered_shell_word_finds_its_script_function() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = crate::host::ScriptHost::new(Arc::new(program), Box::new(NoData));
    let words: Vec<u16> = "Where;Loc, here".encode_utf16().collect();
    host.vm_mut().shell.add(false, &words, 0x0102_0003);
    assert_eq!(host.shell_command("where"), Some(0x0102_0003));
    assert_eq!(host.shell_command("LOC"), Some(0x0102_0003));
    assert_eq!(host.shell_command("here"), Some(0x0102_0003));
    assert_eq!(host.shell_command("there"), None);
}
