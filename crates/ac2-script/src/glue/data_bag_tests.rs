//! `data_bags`, `engine_misc` and the keyframe name setters through the efun calling convention.

use std::sync::Arc;

use super::*;
use crate::native::{ntype, NativeObject};
use crate::vm::{NoData, Program};

fn vm() -> Vm {
    let mut efuns = EFuns::new();
    register_pure(&mut efuns);
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    Vm::new(Arc::new(program), Box::new(NoData), efuns)
}

fn id(name: &str) -> u32 {
    efun_table()
        .find(|(_, n)| *n == name)
        .map(|(id, _)| id)
        .unwrap_or_else(|| panic!("no efun {name}"))
}

/// Calls efun `name` with `returns` return slots, `args` (first argument first, a 64-bit argument
/// as its two words high first) and `this`; the return slots come back deepest first.
fn call(
    vm: &mut Vm,
    name: &str,
    returns: usize,
    args: &[u32],
    this: u32,
) -> (InterpStatus, Vec<(u32, u8)>) {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    for _ in 0..returns {
        ctx.push(0, tag::INT).unwrap();
    }
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    let status = vm.run_efun(id(name), &mut ctx);
    let rets = (0..returns).map(|i| (ctx.data[i], ctx.tags[i])).collect();
    (status, rets)
}

fn native(vm: &mut Vm, native_type: u32) -> u32 {
    vm.refs
        .create_native(native_type, NativeObject::new_empty(native_type))
}

#[test]
fn link_requirements_store_and_return_their_words() {
    let mut vm = vm();
    let l = native(&mut vm, ntype::LINK_REQUIREMENTS);
    for (setter, getter, value) in [
        ("SetInputLink", "IsInputLink", 1),
        ("SetLinkType", "GetLinkType", 7),
        ("SetNumLinks", "GetNumLinks", 3),
        ("SetRequired", "IsRequired", 1),
        ("SetNumReqLinks", "GetNumReqLinks", 2),
    ] {
        let set = format!("gLinkRequirements::CallEFun_{setter}");
        let get = format!("gLinkRequirements::CallEFun_{getter}");
        assert_eq!(call(&mut vm, &set, 0, &[value], l).0, InterpStatus::Success);
        let (status, rets) = call(&mut vm, &get, 1, &[], l);
        assert_eq!(status, InterpStatus::Success);
        assert_eq!(rets[0], (value, tag::BOOL), "{getter}");
    }
    // An undefined `this` crashes the script.
    let set = "gLinkRequirements::CallEFun_SetLinkType";
    assert_eq!(call(&mut vm, set, 0, &[1], u32::MAX).0, InterpStatus::Crash);
}

#[test]
fn missile_targeting_parameters_set_their_group_bits() {
    let mut vm = vm();
    let t = native(&mut vm, ntype::MISSILE_TARGETING_PARAMETERS);
    let offset = vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([1.0, 2.0, 3.0]));
    // SetTargetID(id, height, use_scale, offset): the 64-bit id is passed high word first.
    let args = [0x0000_0005, 0x0000_0009, 1.5f32.to_bits(), 1, offset];
    let name = "gMissileTargetingParameters::CallEFun_SetTargetID";
    assert_eq!(call(&mut vm, name, 0, &args, t).0, InterpStatus::Success);
    let name = "gMissileTargetingParameters::CallEFun_SetInitialSpeed";
    call(&mut vm, name, 0, &[30.0f32.to_bits()], t);
    let name = "gMissileTargetingParameters::CallEFun_SetAlwaysStartTowardTarget";
    call(&mut vm, name, 0, &[], t);
    let Some(NativeObject::MissileTargeting(p)) = vm.native(t) else {
        panic!("not a targeting native");
    };
    assert_eq!(p.target_id, 0x0000_0005_0000_0009);
    assert_eq!((p.target_height_adjustment, p.use_target_scale), (1.5, 1));
    assert_eq!(p.target_offset, [1.0, 2.0, 3.0]);
    assert_eq!(p.initial_velocity, 30.0);
    assert_eq!(p.bitfield, 0x1 | 0x4 | 0x400);
}

#[test]
fn missile_info_copies_the_parameters_it_is_given() {
    let mut vm = vm();
    let info = native(&mut vm, ntype::MISSILE_INFO);
    let targeting = native(&mut vm, ntype::MISSILE_TARGETING_PARAMETERS);
    call(
        &mut vm,
        "gMissileTargetingParameters::CallEFun_SetNoFireIfTooFar",
        0,
        &[],
        targeting,
    );
    let calls: [(&str, u32); 5] = [
        ("gMissileInfo::CallEFun_SetEntityDID", 0x0200_0001),
        ("gMissileInfo::CallEFun_SetHoldingLocation", 4),
        ("gMissileInfo::CallEFun_SetThrown", 1),
        ("gMissileInfo::CallEFun_SetClone", 0x100),
        ("gMissileInfo::CallEFun_SetTargetingParameters", targeting),
    ];
    for (name, arg) in calls {
        assert_eq!(
            call(&mut vm, name, 0, &[arg], info).0,
            InterpStatus::Success
        );
    }
    // The copy does not follow later changes to the source.
    call(
        &mut vm,
        "gMissileTargetingParameters::CallEFun_SetStartTowardTarget",
        0,
        &[],
        targeting,
    );
    let Some(NativeObject::MissileInfo(i)) = vm.native(info) else {
        panic!("not a missile info");
    };
    assert_eq!((i.entity_did, i.holding_location), (0x0200_0001, 4));
    assert!(i.thrown);
    assert!(!i.clone, "SetClone takes the argument's low byte only");
    assert_eq!(i.targeting_params.bitfield, 0x100);
    // A wrong native as the parameters crashes.
    let bad = call(
        &mut vm,
        "gMissileInfo::CallEFun_SetCollisionParameters",
        0,
        &[info],
        info,
    );
    assert_eq!(bad.0, InterpStatus::Crash);
}

#[test]
fn fx_and_behavior_info_keeps_ids_and_behaviour_copies() {
    let mut vm = vm();
    let fx = native(&mut vm, ntype::FX_AND_BEHAVIOR_INFO);
    let behavior = native(&mut vm, ntype::BEHAVIOR_PARAMS);
    for (name, arg) in [
        ("SetAttackerFX", 11),
        ("SetTargetFX", 12),
        ("SetWeaponFX", 13),
        ("SetTargetBehavior", behavior),
        ("SetWeaponBehavior", behavior),
    ] {
        let name = format!("gFXAndBehaviorInfo::CallEFun_{name}");
        assert_eq!(call(&mut vm, &name, 0, &[arg], fx).0, InterpStatus::Success);
    }
    let Some(NativeObject::FxAndBehavior(f)) = vm.native(fx) else {
        panic!("not an FXAndBehaviorInfo");
    };
    assert_eq!((f.attacker_fx, f.target_fx, f.weapon_fx), (11, 12, 13));
    assert!(f.target_behavior.is_some() && f.weapon_behavior.is_some());
}

#[test]
fn keyframe_track_by_name_sets_the_bit_even_when_the_name_does_not_resolve() {
    let mut vm = vm();
    let k = native(&mut vm, ntype::GM_KEYFRAME);
    let name = vm.new_wstring("movie.avi".encode_utf16().collect());
    for efun in [
        "SetVideoTrackByName",
        "SetAudioTrackByName",
        "SetVoiceTrackByName",
    ] {
        let efun = format!("ggmKeyframe::CallEFun_{efun}");
        assert_eq!(call(&mut vm, &efun, 0, &[name], k).0, InterpStatus::Success);
    }
    let Some(NativeObject::Keyframe(kf)) = vm.native(k) else {
        panic!("not a keyframe");
    };
    assert_eq!(
        kf.defined_tracks
            & (ac2_weenie::TRACK_VIDEO | ac2_weenie::TRACK_AUDIO | ac2_weenie::TRACK_VOICE),
        ac2_weenie::TRACK_VIDEO | ac2_weenie::TRACK_AUDIO | ac2_weenie::TRACK_VOICE
    );
    assert_eq!((kf.video, kf.audio, kf.voice), (0, 0, 0));
}

#[test]
fn shell_commands_split_at_separators_and_lowercase() {
    let mut vm = vm();
    let this = native(&mut vm, ntype::VECTOR);
    let cmd = vm.new_wstring("Attack; Duel,,  Yell ".encode_utf16().collect());
    // AddShellCmd(cmd, function, final) -> 1.
    let (status, rets) = call(
        &mut vm,
        "gCWInterface::CallEFun_AddShellCmd",
        1,
        &[cmd, 0x8212_0001, 0],
        this,
    );
    assert_eq!(status, InterpStatus::Success);
    assert_eq!(rets[0].0, 1);
    let words: Vec<String> = vm
        .shell
        .commands
        .iter()
        .map(|c| String::from_utf16_lossy(&c.command))
        .collect();
    assert_eq!(words, ["attack", "duel", "yell"]);
    // The same word again keeps one entry and takes the new function; help is a separate table.
    let cmd = vm.new_wstring("ATTACK".encode_utf16().collect());
    call(
        &mut vm,
        "gCWInterface::CallEFun_AddShellCmd",
        1,
        &[cmd, 0x8212_0002, 0],
        this,
    );
    call(
        &mut vm,
        "gCWInterface::CallEFun_AddShellHelp",
        1,
        &[cmd, 0x8212_0003],
        this,
    );
    assert_eq!(vm.shell.commands.len(), 3);
    assert_eq!(vm.shell.commands[0].function, 0x8212_0002);
    assert_eq!(vm.shell.help.len(), 1);
}

#[test]
fn timer_returns_the_host_clock_as_a_double() {
    let mut vm = vm();
    vm.cur_time = 12.5;
    let (status, rets) = call(&mut vm, "gTimer::CallEFun_get_time", 2, &[], u32::MAX);
    assert_eq!(status, InterpStatus::Success);
    let bits = u64::from(rets[0].0) | u64::from(rets[1].0) << 32;
    assert_eq!(f64::from_bits(bits), 12.5);
    assert_eq!(rets[0].1, tag::BOOL | tag::LONG);
}

#[test]
fn debug_print_string_takes_a_wpstring_and_crashes_on_a_bad_reference() {
    let mut vm = vm();
    let text = vm.new_wstring("hello".encode_utf16().collect());
    let (status, _) = call(
        &mut vm,
        "gdebug::CallEFun_PrintString",
        0,
        &[text],
        u32::MAX,
    );
    assert_eq!(status, InterpStatus::Success);
    let (status, _) = call(
        &mut vm,
        "gdebug::CallEFun_PrintString",
        0,
        &[u32::MAX],
        u32::MAX,
    );
    assert_eq!(status, InterpStatus::Crash);
}

#[test]
fn sub_data_ids_collect_a_scene_keyframes_ids_unless_filtered() {
    let mut vm = vm();
    let scene = native(&mut vm, ntype::GM_SCENE_INFO);
    if let Some(NativeObject::SceneInfo(s)) = vm.native_mut(scene) {
        s.keyframes.push(ac2_weenie::Keyframe {
            video: 0x0600_0001,
            audio: 0x0A00_0002,
            voice: 0,
            ..Default::default()
        });
    }
    let set = native(&mut vm, ntype::QUALIFIED_DATA_ID_ARRAY);
    let name = "ggmSceneInfo::CallEFun_GetSubDataIDs";
    // The filter is the type `0x40000008`: the track ids are left out.
    assert_eq!(
        call(&mut vm, name, 0, &[0x4000_0008, set], scene).0,
        InterpStatus::Success
    );
    let Some(NativeObject::DataIds(ids)) = vm.native(set) else {
        panic!("not a data id array");
    };
    assert!(ids.is_empty());
    assert_eq!(
        call(&mut vm, name, 0, &[0, set], scene).0,
        InterpStatus::Success
    );
    let Some(NativeObject::DataIds(ids)) = vm.native(set) else {
        panic!("not a data id array");
    };
    let dids: Vec<u32> = ids.iter().map(|&(_, d)| d).collect();
    assert_eq!(dids, [0x0600_0001, 0x0A00_0002]);
}

#[test]
fn string_info_sub_data_ids_add_the_table_of_a_non_literal_string() {
    let mut vm = vm();
    let info = vm.refs.create_native(
        ntype::STRING_INFO,
        NativeObject::StringInfo(crate::native::StringInfo {
            string_id: 5,
            table_id: 0x2300_0001,
            ..Default::default()
        }),
    );
    let literal = vm.refs.create_native(
        ntype::STRING_INFO,
        NativeObject::StringInfo(crate::native::StringInfo {
            literal: Some(vec![65]),
            table_id: 0x2300_0002,
            ..Default::default()
        }),
    );
    let set = native(&mut vm, ntype::QUALIFIED_DATA_ID_ARRAY);
    let name = "gStringInfo::CallEFun_GetSubDataIDs";
    call(&mut vm, name, 0, &[0, set], info);
    call(&mut vm, name, 0, &[0, set], literal);
    let Some(NativeObject::DataIds(ids)) = vm.native(set) else {
        panic!("not a data id array");
    };
    let dids: Vec<u32> = ids.iter().map(|&(_, d)| d).collect();
    assert_eq!(dids, [0x2300_0001]);
}
