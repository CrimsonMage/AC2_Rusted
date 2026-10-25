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
