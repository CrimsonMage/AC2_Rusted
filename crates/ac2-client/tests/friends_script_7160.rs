//! The friends list with the player's script on the VM (`AC2_DAT_DIR`). The script's own GUN
//! requests (the login's `RequestGUN_*` efuns) are the exact values the VM queues; the UI's add and
//! delete are the client's GUN requests (C++, not the script); the GUN service is the double's
//! [`FriendsBook`], whose answers come back as the script's `Result_FriendAdd` and
//! `Result_FriendDelete` IFuns, which raise the `CM_Friends` notices. The list itself has no answer
//! path in Legions (see the catalog notes), and the friends panel is not ported.

mod common;

use std::sync::Arc;

use ac2_client::{Client, FrameInput, FriendEvent, FriendResult};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::{ChatRequest, ScriptHost};
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{FriendInfo, GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{
    FriendsBook, Spawn, GUN_ALREADY_IN_LIST, GUN_DONE, GUN_INVALID_USER, GUN_NOT_IN_LIST, PLAYER,
};

const NAME: &str = "Maria";
/// The directory of players a friend can be.
const DIRECTORY: [&str; 3] = ["Maria", "Tomas", "Ilse"];

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

/// The UI's add goes to the GUN service; the service's new-friend answer (done) is delivered as
/// `Result_FriendAdd`, which raises the message and `FriendAdded` with the name.
#[test]
fn friend_add_is_answered_by_the_gun_service() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    client.take_gun_requests();
    client.friend_add(NAME);
    let requests = client.take_gun_requests();
    assert_eq!(requests, vec![ChatRequest::FriendAdd(NAME.to_owned())]);
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    let ChatRequest::FriendAdd(name) = &requests[0] else {
        panic!("{requests:?}");
    };
    let gun = book.add(name);
    assert_eq!(gun, GUN_DONE);
    let raised = notices(client.gun_friend_result(FriendResult::Added {
        gun,
        name: name.clone(),
    }));
    assert_eq!(raised.len(), 2, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
    assert_eq!(raised[1], GameplayNotice::FriendAdded(NAME.to_owned()));
    assert_eq!(book.names(), vec![NAME.to_owned()]);
    assert!(client.take_script_problems().is_empty());
}

/// A duplicate add is refused by the service (`ZT_E_ALREADY_IN_LIST`, converted to weenie 0xC740):
/// the script raises its error message and no `FriendAdded`.
#[test]
fn duplicate_friend_is_refused_with_a_message() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    assert_eq!(book.add(NAME), GUN_DONE);
    let gun = book.add(NAME);
    assert_eq!(gun, GUN_ALREADY_IN_LIST);
    let raised = notices(client.gun_friend_result(FriendResult::Added {
        gun,
        name: NAME.to_owned(),
    }));
    assert_eq!(raised.len(), 1, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
    assert_eq!(book.names(), vec![NAME.to_owned()], "the list is unchanged");
    assert!(client.take_script_problems().is_empty());
}

/// An add of a name the directory does not hold is refused (`ZT_E_INVALID_USER`): the message only.
#[test]
fn unknown_friend_is_refused_with_a_message() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    let gun = book.add("Nobody");
    assert_eq!(gun, GUN_INVALID_USER);
    let raised = notices(client.gun_friend_result(FriendResult::Added {
        gun,
        name: "Nobody".to_owned(),
    }));
    assert_eq!(raised.len(), 1, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
    assert!(book.friends().is_empty());
}

/// The UI's delete goes to the GUN service; its success is delivered as `Result_FriendDelete`,
/// which raises the message and `FriendRemoved`.
#[test]
fn friend_delete_is_answered_by_the_gun_service() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    book.add(NAME);
    client.take_gun_requests();
    client.friend_delete(NAME);
    let requests = client.take_gun_requests();
    assert_eq!(requests, vec![ChatRequest::FriendDelete(NAME.to_owned())]);
    let ChatRequest::FriendDelete(name) = &requests[0] else {
        panic!("{requests:?}");
    };
    let gun = book.remove(name);
    assert_eq!(gun, GUN_DONE);
    let raised = notices(client.gun_friend_result(FriendResult::Removed {
        gun,
        name: name.clone(),
    }));
    assert_eq!(raised.len(), 2, "{raised:?}");
    assert_eq!(raised[1], GameplayNotice::FriendRemoved(NAME.to_owned()));
    assert!(book.friends().is_empty());
    assert!(client.take_script_problems().is_empty());
}

/// A delete of a name not on the list is refused (`ZT_E_NOT_IN_LIST`): the message only.
#[test]
fn delete_of_a_non_friend_is_refused_with_a_message() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    let gun = book.remove(NAME);
    assert_eq!(gun, GUN_NOT_IN_LIST);
    let raised = notices(client.gun_friend_result(FriendResult::Removed {
        gun,
        name: NAME.to_owned(),
    }));
    assert_eq!(raised.len(), 1, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
}

/// A friend's status (the GUN `FriendStatus` event): the script reads the `TurbineUserStatus`
/// (name, location, mode through its efuns) and raises `FriendStatus` with it.
#[test]
fn friend_status_event_raises_the_status() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    book.add("Tomas");
    assert!(book.set_status("Tomas", "Yaraq", 2));
    let friend = book.friends()[0].clone();
    let raised = notices(client.gun_friend_event(FriendEvent::Status(FriendInfo {
        name: friend.name.clone(),
        location: friend.location.clone(),
        mode: friend.mode,
    })));
    assert_eq!(raised.len(), 2, "{raised:?}");
    assert_eq!(
        raised[1],
        GameplayNotice::FriendStatus(FriendInfo {
            name: "Tomas".to_owned(),
            location: "Yaraq".to_owned(),
            mode: 2,
        })
    );
    assert!(client.take_script_problems().is_empty());
}

/// The friends list (the GUN `FriendStart` event) runs the script's start body, which reads each
/// status and keeps it in the player's list, but raises no list notice in Legions: only the
/// script's message. The panel takes the list from the event (see the catalog notes).
#[test]
fn friend_start_event_raises_the_script_message_only() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let list = vec![
        FriendInfo {
            name: "Tomas".to_owned(),
            location: "Holtburg".to_owned(),
            mode: 1,
        },
        FriendInfo {
            name: "Ilse".to_owned(),
            location: String::new(),
            mode: 0,
        },
    ];
    let raised = notices(client.gun_friend_event(FriendEvent::Start(list)));
    assert_eq!(raised.len(), 1, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
    assert!(client.take_script_problems().is_empty());
}

/// Without the GUN service the friend requests are not queued (`gmCWInterface::RequestGUN_*`
/// answer 0 when `TurbineGun::S_pTurbineGun` is NULL): the UI's add, delete, list and status are
/// refused, and the friends list stays empty.
#[test]
fn friend_requests_are_not_queued_without_the_gun_service() {
    let (mut client, _server) = entered(spawn());
    assert!(!client.friend_add(NAME));
    assert!(!client.friend_delete(NAME));
    assert!(!client.friends_list(true));
    assert!(!client.friend_status(2));
    assert!(client.take_gun_requests().is_empty());
    assert!(client.take_friend_events().is_empty());
}

/// The GUN list and status events reach the friends page (`take_friend_events`), in order: the
/// list fills the page's rows, then a status updates one row. The page's requests go to the GUN
/// service: a delete, the status's flags and an added friend's list request.
#[test]
fn gun_events_reach_the_page_and_its_requests_go_to_gun() {
    let Some((mut client, _server)) = with_gun() else {
        return;
    };
    let friend = |name: &str, location: &str, mode: u32| FriendInfo {
        name: name.to_owned(),
        location: location.to_owned(),
        mode,
    };
    let list = vec![friend("Tomas", "Holtburg", 1), friend("Ilse", "", 0)];
    client.gun_friend_event(FriendEvent::Start(list.clone()));
    client.gun_friend_event(FriendEvent::Status(friend("Tomas", "Yaraq", 2)));
    assert_eq!(
        client.take_friend_events(),
        vec![
            FriendEvent::Start(list),
            FriendEvent::Status(friend("Tomas", "Yaraq", 2)),
        ]
    );
    assert!(client.take_friend_events().is_empty(), "taken once");

    // The login's own GUN lists are queued first; the page's requests come after them.
    client.take_gun_requests();
    assert!(client.friend_delete("Ilse"));
    assert!(client.friend_status(2));
    assert!(client.friends_list(true));
    assert_eq!(
        client.take_gun_requests(),
        vec![
            ChatRequest::FriendDelete("Ilse".to_owned()),
            ChatRequest::SetLocation {
                flags: 2,
                location: String::new(),
            },
            ChatRequest::GetFriendsList {
                user_requested: true
            },
        ],
    );
    client.gun_friend_result(FriendResult::Removed {
        gun: GUN_DONE,
        name: "Ilse".to_owned(),
    });
    assert!(client.take_script_problems().is_empty());
}
