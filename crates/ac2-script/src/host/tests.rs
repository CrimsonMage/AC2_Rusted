//! The host over the Legions library (`AC2_DAT_DIR`) and over a synthetic program.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_net::protocol::{script_function, PackValue, ScriptMessage, SelectionInfo};

use super::*;
use crate::data::DatScriptData;
use crate::vm::NoData;
use crate::{load_wlib, LEGIONS_WLIB_DID};

fn legions_host() -> Option<ScriptHost> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Program::from_stream(&wlib.stream);
    Some(ScriptHost::new(
        Arc::new(program),
        Box::new(DatScriptData::new(dataset, store)),
    ))
}

#[test]
fn dispatch_pushes_retail_frames_and_reports_unknown_objects() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    assert_eq!(
        host.dispatch(7, 0x8212_002C, &[]),
        Err(HostError::UnknownObject(7))
    );
    assert!(host.take_server_calls().is_empty());
}

/// The player's script (`CPlayer`, a fresh instance: its class-default WState is in the older
/// layout retail cannot load) takes server calls: `WM_Player` 0x2C (UpdateSelectionInfo) runs
/// to the end, and `WM_Communication` 0x11 raises `DisplayStringInfo` before it reaches a
/// member the server's state would have set.
#[test]
fn the_player_script_takes_server_calls_and_raises_notices() {
    let Some(mut host) = legions_host() else {
        return;
    };
    let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
    assert!(host.create_weenie(0x5000_0001, pid, 0));
    host.set_player(0x5000_0001);
    let info = SelectionInfo {
        kind: SelectionInfo::CREATURE,
        cur_health: 40,
        pk_damage: 0,
        max_health: 50,
        cur_vigor: 7,
        pk_vigor_loss: 0,
        max_vigor: 9,
    };
    let message = ScriptMessage::UpdateSelectionInfo { info, id: 0x77 };
    let outcome = host
        .dispatch(0x5000_0001, message.function(), &message.arguments())
        .unwrap();
    assert_eq!(outcome.status, InterpStatus::Terminated);
    let text = ac2_net::protocol::StringInfo {
        string_id: 0,
        table_id: 0,
        literal: Some("Hello".into()),
        variables: Vec::new(),
    };
    let message = ScriptMessage::DisplayStringInfo {
        function: script_function::DISPLAY_STRING_INFO,
        text: text.clone(),
        kind: 1,
    };
    host.dispatch(0x5000_0001, message.function(), &message.arguments())
        .unwrap();
    assert_eq!(
        host.take_notices(),
        vec![GameplayNotice::DisplayStringInfo { kind: 1, text }]
    );
    assert!(host.take_dropped().is_empty());
}

/// A script that packs every value kind and sends it: the call comes out in wire form, the
/// package as its packed graph.
#[test]
fn pack_buffer_and_send_server_evt_reach_the_outbox() {
    use crate::native::{ntype, NativeObject};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let buffer = vm.refs.create_native(
        ntype::PACK_BUFFER,
        NativeObject::new_empty(ntype::PACK_BUFFER),
    );
    let text = vm.new_wstring("hi".encode_utf16().collect());
    let call = |vm: &mut Vm, id: u32, args: &[u32], this: u32| {
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        ctx.push(0, tag::INT).unwrap();
        for &a in args.iter().rev() {
            ctx.push(a, tag::INT).unwrap();
        }
        ctx.push(this, tag::REF).unwrap();
        assert_eq!(vm.run_efun(id, &mut ctx), InterpStatus::Success);
    };
    call(vm, 0x0050_0002, &[7], buffer); // PackInt
    call(vm, 0x0050_0003, &[1, 2], buffer); // PackLInt (high word first)
    call(vm, 0x0050_0004, &[1.5f32.to_bits()], buffer); // PackFloat
    call(vm, 0x0050_0001, &[text], buffer); // PackPackage
    call(vm, 0x007F_0036, &[0x8212_0016, buffer], u32::MAX); // SendServerEvt
    let calls = host.take_server_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].function, 0x8212_0016);
    assert!(!calls[0].encrypt);
    assert_eq!(
        calls[0].args[..3],
        [
            PackValue::Int(7),
            PackValue::LongInt(0x1_0000_0002),
            PackValue::Float(1.5),
        ]
    );
    assert_eq!(
        calls[0].args[3].as_native(),
        Some(&ac2_net::protocol::NativeObject::WString("hi".into()))
    );
}

/// Runs efun `id` with `this` and `args` (word 2 first) over a frame with a result slot.
fn run_efun(vm: &mut Vm, id: u32, args: &[u32], this: u32) -> InterpStatus {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    ctx.push(0, tag::INT).unwrap();
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    vm.run_efun(id, &mut ctx)
}

/// `CM_*` notices without an ac2-weenie type come out decoded by their signature; ones with a
/// type come out typed.
#[test]
fn notices_decode_by_signature() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let obj = vm.new_wstring(Vec::new());
    vm.owners.insert(obj, 0x5000_0001);
    // CM_Character::SendNotice_UpdateSkillPanel(obj, ulong64, ulong, ulong).
    let args = [obj, 0x1, 0x2, 3, 4];
    assert_eq!(
        run_efun(vm, 0x0085_0005, &args, u32::MAX),
        InterpStatus::Success
    );
    // CM_Inventory::SendNotice_UpdateInventory(obj, code, itemID, location).
    let args = [obj, 0x4000_0001, 0, 9, 2];
    assert_eq!(
        run_efun(vm, 0x008B_0001, &args, u32::MAX),
        InterpStatus::Success
    );
    assert_eq!(
        host.take_script_notices(),
        vec![ScriptNotice {
            name: "CM_Character::SendNotice_UpdateSkillPanel",
            object: 0x5000_0001,
            args: vec![
                NoticeArg::Long(0x1_0000_0002),
                NoticeArg::Int(3),
                NoticeArg::Int(4)
            ],
        }]
    );
    assert_eq!(
        host.take_notices(),
        vec![GameplayNotice::UpdateInventory {
            code: 0x4000_0001,
            item: 9,
            location: 2
        }]
    );
}

/// `RequestGUN_SendToRoom` goes out only through a chat service; with none, retail returns the
/// room id and sends nothing. GUN-only requests need GUN.
#[test]
fn chat_requests_follow_the_available_services() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let this = vm.new_wstring(Vec::new());
    let text = vm.new_wstring("hi".encode_utf16().collect());
    run_efun(vm, 0x00B4_0033, &[5, text, 0x10], this);
    run_efun(vm, 0x00B4_0037, &[1], this);
    assert!(host.take_chat_requests().is_empty());
    host.set_chat_services(ChatServices {
        turbine_chat: true,
        gun: false,
    });
    let vm = host.vm_mut();
    run_efun(vm, 0x00B4_0033, &[5, text, 0x10], this);
    run_efun(vm, 0x00B4_0037, &[1], this);
    assert_eq!(
        run_efun(vm, 0x00B4_0033, &[5, text, 0], u32::MAX),
        InterpStatus::Crash
    );
    assert_eq!(
        host.take_chat_requests(),
        vec![ChatRequest::SendToRoom {
            room: 5,
            text: "hi".into(),
            flags: 0x10
        }]
    );
}

/// `gCWRepository::SetInt` / `AdjInt` write the weenie's quality table; without qualities
/// (`m_qualities == NULL`) nothing is stored.
#[test]
fn repository_efuns_write_the_weenie_qualities() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let this = host.vm_mut().new_wstring(Vec::new());
    host.vm_mut().owners.insert(this, 0x5000_0001);
    run_efun(host.vm_mut(), 0x0081_0006, &[0x14, 7], this);
    assert!(host.qualities(0x5000_0001).is_none());
    host.set_qualities(0x5000_0001, Default::default());
    run_efun(host.vm_mut(), 0x0081_0006, &[0x14, 7], this);
    // AdjInt(stat, delta, default): 7 + 5.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x14, 5, 0], this);
    let q = host.qualities(0x5000_0001).unwrap();
    assert_eq!(q.ints.get(&0x14), Some(&12));
}

/// The `CharacterGenSystem` singleton answers the creation screens' questions from its WState.
#[test]
fn the_character_gen_system_answers_from_its_state() {
    use crate::host::chargen;
    use crate::native::CharGenResult;
    let Some(mut host) = legions_host() else {
        return;
    };
    let vm = host.vm_mut();
    let (ret, races) = chargen::race_sex_data(vm).expect("race/sex data");
    eprintln!("race/sex data {ret}: {races:x?} fault {:?}", vm.last_fault);
    let (ret, infos) = chargen::race_sex_info_data(vm).expect("race/sex info");
    eprintln!(
        "race/sex info {ret}: {} entries, first {:x?}",
        infos.len(),
        infos.first()
    );
    let player = chargen::avatar_entity_did(vm, false);
    let admin = chargen::avatar_entity_did(vm, true);
    eprintln!("avatar entity DIDs: player {player:x?} admin {admin:x?}");
    let Some((race, sexes)) = races.first().cloned() else {
        panic!("no races");
    };
    let sex = sexes.first().copied().unwrap_or(0);
    let mut cgr = CharGenResult {
        race,
        sex,
        ..CharGenResult::default()
    };
    let r = chargen::set_defaults(vm, &mut cgr);
    eprintln!(
        "defaults {r:?}: {:x?} fault {:?}",
        (
            cgr.race,
            cgr.sex,
            cgr.class,
            cgr.entity_did,
            cgr.phys_type_values()
        ),
        vm.last_fault
    );
    let areas = chargen::valid_starting_areas(vm, cgr.race, cgr.class);
    eprintln!("starting areas {areas:x?}");
    if let Some((_, areas)) = &areas {
        if let Some(area) = areas.first() {
            eprintln!(
                "starting position {:?}",
                chargen::starting_position(vm, *area)
            );
        }
    }
    let phys = chargen::phys_obj(vm, race, sex);
    eprintln!("phys obj {phys:x?}");
    assert!(!races.is_empty());
    let pairs: usize = races.iter().map(|(_, s)| s.len()).sum();
    assert_eq!(infos.len(), pairs);
    assert!(player.is_some_and(|d| d != 0) && admin.is_some_and(|d| d != 0));
    assert_eq!(r, Some(1));
    assert!(!cgr.phys_type_values().is_empty());
    assert!(phys.is_some_and(|d| d != 0));
}

#[test]
#[ignore]
fn probe_player_init() {
    let Some(mut host) = legions_host() else {
        return;
    };
    let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
    let state = host.vm().data().wstate(0x7100_110F);
    eprintln!(
        "0x7100110F: {:?}",
        state.as_ref().map(|s| (
            s.pid,
            s.references
                .as_ref()
                .map(|r| r.ids.len())
                .map_err(|e| e.context)
        ))
    );
    eprintln!(
        "db type of 0x4700110F: {:x?}",
        host.vm().data().db_type(0x4700_110F)
    );
    assert!(host.create_weenie(0x5000_0001, pid, 0x4700_110F));
    host.set_player(0x5000_0001);
    host.set_qualities(0x5000_0001, Default::default());
    let args = vec![PackValue::Int(0); 14];
    let outcome = host
        .dispatch(0x5000_0001, script_function::INIT_PLAYER, &args)
        .unwrap();
    eprintln!(
        "INIT_PLAYER: {outcome:?} ({:?}); notices {:?}; script notices {:?}; calls {:?}",
        outcome.fault.as_ref().and_then(|f| match f {
            Fault::UnboundEFun(id) => crate::glue::efun_name(*id),
            _ => None,
        }),
        host.take_notices(),
        host.take_script_notices()
            .iter()
            .map(|n| n.name)
            .collect::<Vec<_>>(),
        host.take_server_calls()
    );
}

/// Prints every bound efun (`cargo test -p ac2-script list_bound_efuns -- --ignored`).
#[test]
#[ignore]
fn list_bound_efuns() {
    let efuns = host_efuns();
    for (id, name) in crate::glue::efun_table() {
        if efuns.is_bound(id) {
            println!("BOUND {id:08x} {name}");
        }
    }
}

/// A UI call (`ggmCWInterface::CallIFun_*`) runs on the object's process and reads back its
/// return slot: here the method returns its int argument.
#[test]
fn ui_ifun_calls_read_back_the_return_slot() {
    use crate::vm::op;
    let w = |code: u32, t: u8, imm: i32| code | u32::from(t) << 8 | (imm as u32) << 16;
    let code = vec![
        // 0: initializer
        w(op::PUSH_FRAME, 0, 0),
        op::POP_FRAME,
        // 2: method 1 (this, arg): ret = arg
        w(op::PUSH_FRAME, 0, 0),
        w(op::LOAD, 0, -2),
        w(op::PUSHV, 2, -3),
        op::STORE,
        op::POP,
        op::POP_FRAME,
    ];
    let mut program = Program::from_parts(code, vec![vec![0, 1]], vec![vec![0, 8]]);
    program.package_sizes = vec![1];
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    assert!(host.create_weenie(9, 0, 0));
    let result = host
        .call_ifun(9, FunctionId::new(0, 1, false).0, &[IFunArg::Int(41)])
        .unwrap();
    assert_eq!(result.outcome.status, InterpStatus::Terminated);
    assert_eq!(result.retval, 41);
    assert_eq!(host.call_ifun(8, 0, &[]), Err(HostError::UnknownObject(8)));
}
