//! `ggmCWInterface::CallIFun_*` getters and requests not covered elsewhere, run on the player's
//! script VM (`AC2_DAT_DIR`): each call is one wrapper frame and must terminate cleanly.

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, FrameInput};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::interface_calls::InterfaceCalls;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{Spawn, State};

fn logged_in() -> Option<(Client, Rc<RefCell<State>>)> {
    logged_in_owning(ac2_client::SCRIPT_VM_FUNCTIONS)
}

fn logged_in_owning(owned: &[u32]) -> Option<(Client, Rc<RefCell<State>>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    client.attach_scripts(ScriptHost::new(program, Box::new(data)), owned);
    step(&mut client, &server, &FrameInput::default());
    server.borrow_mut().send_init_player();
    for _ in 0..6 {
        step(&mut client, &server, &FrameInput::default());
    }
    Some((client, server))
}

#[test]
fn the_option_getters_and_selection_calls_terminate_on_the_player_script() {
    let Some((mut client, _server)) = logged_in() else {
        return;
    };
    assert!(client.accept_trade_request().is_ok());
    assert!(client.accept_fellowship_request().is_ok());
    assert!(client.accept_allegiance_request().is_ok());
    assert!(client.auto_target_option().is_ok());
    assert!(client.show_damage_text().is_ok());
    assert!(client.show_radar_coords_option().is_ok());
    assert!(client.which_shortcut_set().is_ok());
    assert!(client.shortcut_height().is_ok());
    assert!(client.damage_text_range_other().is_ok());
    assert!(client.available_experience().is_ok());
    // Nothing is selected at login.
    assert_eq!(client.selection(), Ok(0));
    assert_eq!(client.has_something_selected(), Ok(0));
    assert!(client.is_fellowship_member(0x1234).is_ok_and(|m| !m));
}

#[test]
fn the_history_and_alias_walkers_answer_through_the_script() {
    let Some((mut client, _server)) = logged_in() else {
        return;
    };
    assert!(client.reset_retell_history_index().is_ok());
    assert!(client.reset_reply_history_index().is_ok());
    assert!(client.inq_alias("nosuchalias").is_ok());
    assert!(client
        .inq_history_item(ac2_script::host::interface_calls::INQ_PREV_REPLY_HISTORY_ITEM)
        .is_ok());
}

/// The lifecycle IFuns (`gCWInterface::CallIFun_HandleEnterWorld` and kin) run on the player's
/// process once the VM owns them; a call the VM does not own is left to the stand-ins.
#[test]
fn the_lifecycle_ifuns_run_on_the_player_script_when_the_vm_owns_them() {
    use ac2_script::host::lifecycle::lifecycle_ifun::*;
    use ac2_script::host::lifecycle::ScriptLifecycle;
    use ac2_script::host::port::ScriptPort;
    let owned: Vec<u32> = ac2_client::SCRIPT_VM_FUNCTIONS
        .iter()
        .copied()
        .chain([
            HANDLE_ENTER_WORLD,
            HANDLE_LEAVE_WORLD,
            HANDLE_POST_LEAVE_WORLD,
        ])
        .collect();
    let Some((mut client, _server)) = logged_in_owning(&owned) else {
        return;
    };
    let player = client.player_id().expect("a player");
    let _ = client.take_script_problems();
    let mut out = Vec::new();
    for function in [
        HANDLE_ENTER_WORLD,
        HANDLE_LEAVE_WORLD,
        HANDLE_POST_LEAVE_WORLD,
    ] {
        client.lifecycle_ifun(player, function, &mut out);
    }
    assert_eq!(client.take_script_problems(), Vec::<String>::new());
    // A call the VM does not own is not run: the destruction IFun was not handed over.
    client.lifecycle_ifun(player, HANDLE_DESTRUCTION, &mut out);
    assert_eq!(client.take_script_problems(), Vec::<String>::new());
}

/// Every remaining getter and request of the player's interface runs its wrapper frame to the
/// end on the player's script (the options, the examination panel, the chat history, the GUN
/// results the script hears).
#[test]
fn the_other_wrappers_end_cleanly_on_the_player_script() {
    use ac2_net::protocol::{ExaminationRequest, StringInfo};
    use ac2_script::host::interface_calls::*;
    use ac2_script::host::port::ScriptPort;
    let Some((mut client, _server)) = logged_in() else {
        return;
    };
    assert!(client.accept_allegiance_request().is_ok());
    assert!(client.bypass_gold_melting_confirmation_dlg().is_ok());
    assert!(client.latency_indicator_graph().is_ok());
    assert!(client.radar_mask().is_ok());
    assert!(client.shortcut_insta_scroll().is_ok());
    assert!(client.shortcut_locked().is_ok());
    assert!(client.show_damage_text_other().is_ok());
    assert!(client.show_fellows_on_map().is_ok());
    assert!(client.show_object_names_option().is_ok());
    assert!(client.show_pk_damage().is_ok());
    assert!(client.show_pet_ui().is_ok());
    assert!(client.show_shortcut_numbers().is_ok());
    assert!(client.should_auto_target().is_ok());
    assert!(client.last_selection().is_ok());
    assert!(client.set_which_shortcut_set(0).is_ok());
    assert!(client.register_commands().is_ok());
    assert!(client.stop_examination().is_ok());
    assert!(client.start_examination().is_ok());
    assert!(client
        .set_examination_request(ExaminationRequest::default())
        .is_ok());
    assert!(client.update_open(0x1234, true).is_ok());
    assert!(client.do_assisted_tell("Birch").is_ok());
    assert!(client.gun_handle_reauthentication().is_ok());
    assert!(client.result_send_to_user("Birch", "hello", 0).is_ok());
    assert!(client.result_send_to_room("hello", 1, 0).is_ok());
    assert!(client.result_ignored_add(0, "Birch").is_ok());
    assert!(client.result_ignored_delete(0, "Birch").is_ok());
    assert!(client
        .result_call_create(0, 1, "Birch", "help", 2, 3)
        .is_ok());
    assert!(client
        .hear_tell(7, StringInfo::default(), StringInfo::default(), 0)
        .is_ok());
    assert!(client
        .gun_hear_from_room(StringInfo::default(), "hi", 1, 0)
        .is_ok());
    // A melee hit (skill 6, `AttackResult` 0x40000008): the script fills the info it is handed,
    // the target's reaction among it (`Damage_Hit1`).
    match client.fx_and_behavior_info(6, 0, 0x4000_0008) {
        Ok(Some(ac2_script::native::NativeObject::FxAndBehavior(info))) => {
            assert_eq!(info.target_behavior.map(|b| b.behavior), Some(0x4100_007C))
        }
        other => panic!("the script fills the FXAndBehaviorInfo: {other:?}"),
    }
    let player = client.player_id().expect("a player");
    // Without a missile weapon the script reads an undefined reference and the call faults (the
    // answer is an `Err`, not a panic); a real launch passes a wielded launcher's context.
    let _ = client.missile_info(0, 0, player, player);
    assert!(client
        .inq_history_item(INQ_NEXT_RETELL_HISTORY_ITEM)
        .is_ok());
    assert!(client
        .inq_history_item(INQ_PREV_RETELL_HISTORY_ITEM)
        .is_ok());
    assert!(client.inq_history_item(INQ_NEXT_REPLY_HISTORY_ITEM).is_ok());
}
