//! The friends list with the player's script on the VM (`AC2_DAT_DIR`). The script's own GUN
//! requests (the login's `RequestGUN_*` efuns) reach `TurbineGun` (here over the recording
//! `TestGun`); the UI's add and delete are the C++ `RequestGUN_*` calls; the GUN service's answers
//! are the double's [`FriendsBook`] values delivered as what `CCli*Result` raises
//! ([`GunOutput`]), which the client runs as the script's `Result_FriendAdd` and
//! `Result_FriendDelete` IFuns, raising the `CM_Friends` notices.

mod common;

use std::sync::Arc;

use ac2_client::{Client, FrameInput, FriendEvent};
use ac2_core::DataId;
use ac2_gun::test_support::{Call, TestGunHandle};
use ac2_net::protocol::ScriptRequest;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::gun::{shared, GunOutput};
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
fn with_gun() -> Option<(Client, Shared, TestGunHandle)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let names = DatScriptData::new(dataset.clone(), store.clone());
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let handle = connected_turbine_gun();
    let (mut client, server) = entered(spawn());
    client
        .session_mut()
        .set_string_tables(ac2_client::string_table_ids(&names));
    client
        .session_mut()
        .set_chat_commands(ac2_client::chat_commands(&names));
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    // The login state went out before the VM was attached: it is sent again so the script's
    // login (which queues the GUN lists) runs.
    server.borrow_mut().send_init_player();
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    Some((client, server, handle))
}

/// A connected `TurbineGun` over the recording double, as `S_pTurbineGun`.
fn connected_turbine_gun() -> TestGunHandle {
    use ac2_gun::message::{GunQueueMessage, Message, MessageBody};
    let handle = TestGunHandle::new();
    let gun = std::rc::Rc::new(ac2_gun::TurbineGun::new(0x0409));
    let mut loader = ac2_gun::test_support::TestLoader::new(handle.gun());
    assert!(gun.init("", &mut loader));
    assert!(gun.request_connect("gun", 2310, 0, "1.0", "K", 0));
    handle.queue_message(GunQueueMessage {
        flags: 4,
        gmid: 1,
        param: 0,
        cb: 0,
        message: Message {
            inited: true,
            id: ac2_gun::message_id::CONNECT_EVENT,
            result: None,
            body: MessageBody::None,
        },
    });
    gun.check_for_messages();
    shared().set_turbine_gun(Some(gun));
    handle.clear_calls();
    handle
}

/// What a `CCli*Result` / `CCli*Event` raised, carried out by the next frame.
fn deliver(client: &mut Client, output: GunOutput) -> Vec<SessionEvent> {
    shared().push(output);
    client.use_time(&FrameInput::default()).unwrap()
}

fn weenie(gun: u32) -> u32 {
    ac2_gun::ct::convert_to_weenie_result(ac2_gun::Hresult(gun))
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
    let Some((mut client, _server, handle)) = with_gun() else {
        return;
    };
    let calls = handle.calls();
    assert!(matches!(calls[0], Call::GetIgnoredList(_)), "{calls:?}");
    assert!(matches!(calls[1], Call::GetFriendsList(_)), "{calls:?}");
    assert!(matches!(calls[2], Call::Flags { bits: 1, .. }));
    assert_eq!(calls.len(), 3);
    assert!(client.take_script_problems().is_empty());
}

/// The UI's add goes to the GUN service; the service's new-friend answer (done) is delivered as
/// `Result_FriendAdd`, which raises the message and `FriendAdded` with the name.
#[test]
fn friend_add_is_answered_by_the_gun_service() {
    let Some((mut client, _server, handle)) = with_gun() else {
        return;
    };
    handle.clear_calls();
    assert!(client.friend_add(NAME));
    let calls = handle.calls();
    assert!(
        matches!(&calls[0], Call::FriendAdd(n, _) if n == NAME),
        "{calls:?}"
    );
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    let name = NAME.to_owned();
    let gun = book.add(&name);
    assert_eq!(gun, GUN_DONE);
    let raised = notices(deliver(
        &mut client,
        GunOutput::ResultFriendAdd {
            result: weenie(gun),
            name: name.clone(),
        },
    ));
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
    let Some((mut client, _server, _handle)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    assert_eq!(book.add(NAME), GUN_DONE);
    let gun = book.add(NAME);
    assert_eq!(gun, GUN_ALREADY_IN_LIST);
    let raised = notices(deliver(
        &mut client,
        GunOutput::ResultFriendAdd {
            result: weenie(gun),
            name: NAME.to_owned(),
        },
    ));
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
    let Some((mut client, _server, _handle)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    let gun = book.add("Nobody");
    assert_eq!(gun, GUN_INVALID_USER);
    let raised = notices(deliver(
        &mut client,
        GunOutput::ResultFriendAdd {
            result: weenie(gun),
            name: "Nobody".to_owned(),
        },
    ));
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
    let Some((mut client, _server, handle)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    book.add(NAME);
    handle.clear_calls();
    assert!(client.friend_delete(NAME));
    let calls = handle.calls();
    assert!(
        matches!(&calls[0], Call::FriendDelete(n, _) if n == NAME),
        "{calls:?}"
    );
    let name = NAME.to_owned();
    let gun = book.remove(&name);
    assert_eq!(gun, GUN_DONE);
    let raised = notices(deliver(
        &mut client,
        GunOutput::ResultFriendDelete {
            result: weenie(gun),
            name: name.clone(),
        },
    ));
    assert_eq!(raised.len(), 2, "{raised:?}");
    assert_eq!(raised[1], GameplayNotice::FriendRemoved(NAME.to_owned()));
    assert!(book.friends().is_empty());
    assert!(client.take_script_problems().is_empty());
}

/// A delete of a name not on the list is refused (`ZT_E_NOT_IN_LIST`): the message only.
#[test]
fn delete_of_a_non_friend_is_refused_with_a_message() {
    let Some((mut client, _server, _handle)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    let gun = book.remove(NAME);
    assert_eq!(gun, GUN_NOT_IN_LIST);
    let raised = notices(deliver(
        &mut client,
        GunOutput::ResultFriendDelete {
            result: weenie(gun),
            name: NAME.to_owned(),
        },
    ));
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
    let Some((mut client, _server, _handle)) = with_gun() else {
        return;
    };
    let mut book = FriendsBook::with_directory(&DIRECTORY);
    book.add("Tomas");
    assert!(book.set_status("Tomas", "Yaraq", 2));
    let friend = book.friends()[0].clone();
    let raised = notices(deliver(
        &mut client,
        GunOutput::EventFriendStatus(FriendInfo {
            name: friend.name.clone(),
            location: friend.location.clone(),
            mode: friend.mode,
        }),
    ));
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
    let Some((mut client, _server, _handle)) = with_gun() else {
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
    let raised = notices(deliver(&mut client, GunOutput::EventFriendStart(list)));
    assert_eq!(raised.len(), 1, "{raised:?}");
    assert!(matches!(
        raised[0],
        GameplayNotice::DisplayStringInfo { .. }
    ));
    assert!(client.take_script_problems().is_empty());
}

/// Without a `TurbineGun` the friend requests answer 0 (`gmCWInterface::RequestGUN_*` with
/// `S_pTurbineGun` NULL): the UI's add, delete, list and status are refused.
#[test]
fn friend_requests_are_refused_without_a_turbine_gun() {
    let (mut client, _server) = entered(spawn());
    shared().set_turbine_gun(None);
    assert!(!client.friend_add(NAME));
    assert!(!client.friend_delete(NAME));
    assert!(!client.friends_list(true));
    assert!(!client.friend_status(2));
    assert!(client.take_friend_events().is_empty());
}

/// The GUN list and status events reach the friends page (`take_friend_events`), in order: the
/// list fills the page's rows, then a status updates one row. The page's requests go to the GUN
/// service: a delete, the status's flags and an added friend's list request.
#[test]
fn gun_events_reach_the_page_and_its_requests_go_to_gun() {
    let Some((mut client, _server, handle)) = with_gun() else {
        return;
    };
    let friend = |name: &str, location: &str, mode: u32| FriendInfo {
        name: name.to_owned(),
        location: location.to_owned(),
        mode,
    };
    let list = vec![friend("Tomas", "Holtburg", 1), friend("Ilse", "", 0)];
    deliver(&mut client, GunOutput::EventFriendStart(list.clone()));
    deliver(
        &mut client,
        GunOutput::EventFriendStatus(friend("Tomas", "Yaraq", 2)),
    );
    assert_eq!(
        client.take_friend_events(),
        vec![
            FriendEvent::Start(list),
            FriendEvent::Status(friend("Tomas", "Yaraq", 2)),
        ]
    );
    assert!(client.take_friend_events().is_empty(), "taken once");

    handle.clear_calls();
    assert!(client.friend_delete("Ilse"));
    assert!(client.friend_status(2));
    assert!(client.friends_list(true));
    let calls = handle.calls();
    assert!(
        matches!(&calls[0], Call::FriendDelete(n, _) if n == "Ilse"),
        "{calls:?}"
    );
    assert!(
        matches!(&calls[1], Call::SetLocation { mode: 2, location, .. } if location.is_empty())
    );
    assert!(matches!(calls[2], Call::GetFriendsList(_)));
    deliver(
        &mut client,
        GunOutput::ResultFriendDelete {
            result: weenie(GUN_DONE),
            name: "Ilse".to_owned(),
        },
    );
    assert!(client.take_script_problems().is_empty());
}

/// The friends status drop-down's emote is `gmCWInterface::DoEmote`, the player's script's IFun
/// 0xB400EC with the emote's `StringInfo`: the status's flags go to GUN, and the script runs the
/// IFun (it raises no problem), where the emote's request reaches the server as the script
/// sends it.
#[test]
fn the_status_emote_runs_the_scripts_do_emote() {
    let Some((mut client, server, handle)) = with_gun() else {
        return;
    };
    let before = server.borrow().script_requests.len();
    let request = ac2_client::Command::Friends(ac2_ui::FriendsRequest::Status {
        flags: 2,
        emote: Some("ID_EmoteAFKCmd"),
    });
    let input = FrameInput {
        commands: vec![request],
        ..FrameInput::default()
    };
    step(&mut client, &server, &input);
    for _ in 0..4 {
        step(&mut client, &server, &FrameInput::default());
    }
    assert!(
        handle.calls().iter().any(
            |c| matches!(c, Call::SetLocation { mode: 2, location, .. } if location.is_empty())
        ),
        "{:?}",
        handle.calls()
    );
    assert!(client.take_script_problems().is_empty());
    // The script sends the emote the status names (`ID_EmoteAFKCmd`'s emote id in the chat table).
    let requests = server.borrow().script_requests[before..].to_vec();
    let afk = client
        .session_mut()
        .chat()
        .commands()
        .emote("afk")
        .expect("the away emote");
    assert!(
        requests.contains(&ScriptRequest::Emote(afk)),
        "{requests:?}"
    );
}
