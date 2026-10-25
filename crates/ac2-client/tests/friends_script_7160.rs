//! The friends list with the player's script on the VM (`AC2_DAT_DIR`). The script's own GUN
//! requests (the login's `RequestGUN_*` efuns) are the exact values the VM queues; the UI's add and
//! delete are the client's GUN requests (C++, not the script); the GUN service is the double's
//! [`FriendsBook`], whose answers come back as the script's `Result_FriendAdd` and
//! `Result_FriendDelete` IFuns, which raise the `CM_Friends` notices. The list itself has no answer
//! path in Legions (see the catalog notes), and the friends panel is not ported.

mod common;

use std::sync::Arc;

use ac2_client::{Client, FrameInput, FriendResult};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::{ChatRequest, ScriptHost};
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{FriendsBook, Spawn, PLAYER};

const NAME: &str = "Maria";

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The script with the player attached and the GUN service on (the host's chat services are
/// reset by the attach, so the service is set after it), after the login's idle frames.
fn with_gun() -> Option<(Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    client.set_gun_service(true);
    // The login state went out before the VM was attached: it is sent again so the script's
    // login (which queues the GUN lists) runs.
    server.borrow_mut().send_init_player();
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    Some((client, server))
}

/// The notices in a result's events, in order.
fn notices(events: Vec<SessionEvent>) -> Vec<GameplayNotice> {
    events
        .into_iter()
        .filter_map(|event| match event {
            SessionEvent::Notice(notice) => Some(notice),
            _ => None,
        })
        .collect()
}

/// The login's GUN requests from the script, exactly as the VM queues them: the ignore list and
/// the friends list (not the user's), then the state flags.
#[test]
fn login_queues_the_script_gun_lists() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    assert_eq!(
        client.take_gun_requests(),
        vec![
            ChatRequest::GetIgnoredList {
                user_requested: false
            },
            ChatRequest::GetFriendsList {
                user_requested: false
            },
            ChatRequest::SetStateFlags(1),
        ]
    );
    assert!(client.take_script_problems().is_empty());
}

/// The UI's add goes to the GUN service; the service's new-friend answer (success) is delivered
/// as `Result_FriendAdd`, which raises `FriendAdded` with the name.
#[test]
fn friend_add_is_answered_by_the_gun_service() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    client.take_gun_requests();
    client.friend_add(NAME);
    let requests = client.take_gun_requests();
    assert_eq!(requests, vec![ChatRequest::FriendAdd(NAME.to_owned())]);
    let mut book = FriendsBook::default();
    let ChatRequest::FriendAdd(name) = &requests[0] else {
        panic!("{requests:?}");
    };
    let result = book.add(name).expect("a new friend is answered");
    let events = client.gun_friend_result(FriendResult::Added {
        result,
        name: name.clone(),
    });
    // The script's own message first (a `DisplayStringInfo`), then the notice of the add.
    let raised = notices(events);
    assert_eq!(raised.len(), 2, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
    assert_eq!(raised[1], GameplayNotice::FriendAdded(NAME.to_owned()));
    assert_eq!(book.names(), [NAME.to_owned()]);
    assert!(client.take_script_problems().is_empty());
}

/// A second add of the same name is refused by the service, which answers nothing: no result
/// reaches the script, so no notice.
#[test]
fn duplicate_friend_is_not_answered() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::default();
    book.add(NAME);
    client.take_gun_requests();
    client.friend_add(NAME);
    let requests = client.take_gun_requests();
    let ChatRequest::FriendAdd(name) = &requests[0] else {
        panic!("{requests:?}");
    };
    assert_eq!(book.add(name), None, "the service refuses a duplicate");
    assert_eq!(book.names(), [NAME.to_owned()], "the list is unchanged");
}

/// The UI's delete goes to the GUN service; its success is delivered as `Result_FriendDelete`,
/// which raises `FriendRemoved`.
#[test]
fn friend_delete_is_answered_by_the_gun_service() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::default();
    book.add(NAME);
    client.take_gun_requests();
    client.friend_delete(NAME);
    let requests = client.take_gun_requests();
    assert_eq!(requests, vec![ChatRequest::FriendDelete(NAME.to_owned())]);
    let ChatRequest::FriendDelete(name) = &requests[0] else {
        panic!("{requests:?}");
    };
    let result = book.remove(name).expect("a listed friend is answered");
    let events = client.gun_friend_result(FriendResult::Removed {
        result,
        name: name.clone(),
    });
    let raised = notices(events);
    assert_eq!(raised.len(), 2, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
    assert_eq!(raised[1], GameplayNotice::FriendRemoved(NAME.to_owned()));
    assert!(book.names().is_empty());
    assert!(client.take_script_problems().is_empty());
}

/// Without the GUN service the friend requests are not queued by the script (the VM's own
/// requests answer 0), and the friends list stays empty.
#[test]
fn friend_requests_are_not_queued_without_the_vm() {
    let (mut client, _server) = entered(spawn());
    client.friend_add(NAME);
    assert!(client.take_gun_requests().is_empty());
}
