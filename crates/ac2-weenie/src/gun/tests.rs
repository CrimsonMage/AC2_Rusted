//! Tests of the GUN classes. Expected values come from the retail functions as the spec and the
//! decompile give them (`decompiled-pdb/gundll/spec/0c_client.md` and its adjudication), never
//! from this port's output.

use ac2_gun::message::{GunQueueMessage, Message, MessageBody, ResultHeader};
use ac2_gun::message_id as id;
use ac2_gun::test_support::{Call, TestGunHandle, TestLoader};
use ac2_gun::{Context, Hresult, MessageId, UserStatus};
use ac2_net::protocol::GunChatExtraInfo;

use super::client::GunClient;
use super::interface::*;
use super::options::GunOptions;
use super::output::GunOutput;
use super::shared::shared;
use crate::friends::FriendInfo;
use std::rc::Rc;

pub(super) fn queued(message: Message) -> GunQueueMessage {
    GunQueueMessage {
        flags: 4,
        gmid: 1,
        param: 0,
        cb: 0,
        message,
    }
}

pub(super) fn event(id: MessageId, body: MessageBody) -> GunQueueMessage {
    queued(Message {
        inited: true,
        id,
        result: None,
        body,
    })
}

pub(super) fn result(id: MessageId, ctx: Context, hr: u32, body: MessageBody) -> GunQueueMessage {
    queued(Message {
        inited: true,
        id,
        result: Some(ResultHeader {
            context: ctx,
            hr: Hresult(hr),
        }),
        body,
    })
}

pub(super) struct Rig {
    pub handle: TestGunHandle,
    pub client: Rc<GunClient>,
}

pub(super) fn gun_options(server: &str) -> GunOptions {
    GunOptions {
        gun_server: server.to_owned(),
        ..GunOptions::default()
    }
}

pub(super) fn rig(options: GunOptions, primer: &str) -> Rig {
    let handle = TestGunHandle::new();
    let loader = TestLoader::new(handle.gun());
    let client = GunClient::new(options, false, primer, 0x0409, Box::new(loader));
    client.init();
    Rig { handle, client }
}

/// A rig whose connect completed and whose player exists, outputs drained.
pub(super) fn connected_rig() -> Rig {
    let rig = rig(gun_options("gun.example"), "");
    shared().set_player(true);
    assert!(matches!(
        rig.client.connect(&|| 0, true),
        super::client::ConnectPoll::Pending
    ));
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    assert!(matches!(
        rig.client.connect(&|| 0, true),
        super::client::ConnectPoll::Ready(_)
    ));
    shared().take_outputs();
    rig.handle.clear_calls();
    rig
}

fn blob_for(flags: u32, letters: &str) -> ac2_gun::TagBlob {
    let bytes = GunChatExtraInfo::new(flags, letters.to_owned()).encode();
    ac2_gun::TagBlob {
        cb_size: bytes.len() as u32,
        data: Some(bytes),
    }
}

#[test]
fn options_have_the_constructors_defaults() {
    let o = GunOptions::default();
    assert_eq!(o.gun_server, "");
    assert_eq!(o.gun_port, 2310);
    assert_eq!(o.gun_client_dll, "");
    assert!(!o.use_gun_ticket);
    assert_eq!(o.gun_ticket_override, "");
    assert_eq!(
        o.migration_url,
        "https://acbm.turbinegames.com/IISAcBillingMigration/IISAcBillingMigration.dll?ac2"
    );
    assert_eq!((o.migration1_timeout, o.migration2_timeout), (8000, 8000));
    assert!(
        !o.gun_enabled(),
        "GUN is off exactly when gunserver is empty"
    );
}

#[test]
fn options_are_stored_as_the_command_line_table_does() {
    let mut o = GunOptions::default();
    for (name, value) in [
        ("gunserver", Some("gun.example")),
        ("gunport", Some("4096")),
        ("gunclientdll", Some("Other.dll")),
        ("gunticket", None),
        ("gunticketoverride", Some("ticketText")),
        ("migrationurl", Some("http://m")),
        ("migration1Timeout", Some("0x10")),
        ("migration2Timeout", Some("012")),
    ] {
        assert!(GunOptions::takes_value(name).is_some());
        o.apply(name, value).unwrap();
    }
    assert_eq!(o.gun_server, "gun.example");
    assert_eq!(o.gun_port, 4096);
    assert_eq!(o.gun_client_dll, "Other.dll");
    assert!(o.use_gun_ticket);
    assert_eq!(o.gun_ticket_override, "ticketText");
    assert_eq!(o.migration_url, "http://m");
    // strtol base 0: hex and octal.
    assert_eq!((o.migration1_timeout, o.migration2_timeout), (16, 10));
    assert!(o.gun_enabled());
    assert_eq!(GunOptions::takes_value("gunticket"), Some(false));
    assert_eq!(GunOptions::takes_value("nope"), None);
}

/// `-gunport` is `sscanf("%hu")` (Legions 0x0060A0B0): decimal only, text after the digits is
/// ignored, no digits is an option error.
#[test]
fn gunport_is_read_as_sscanf_hu() {
    use super::options::scan_hu;
    assert_eq!(scan_hu("2310"), Some(2310));
    assert_eq!(scan_hu("  012"), Some(12));
    assert_eq!(scan_hu("0x910"), Some(0), "%hu stops at the x");
    assert_eq!(scan_hu("77abc"), Some(77));
    assert_eq!(scan_hu("65537"), Some(1), "16 bits");
    assert_eq!(scan_hu("-1"), Some(65535));
    assert_eq!(scan_hu("abc"), None);
    assert_eq!(scan_hu(""), None);
    let mut o = GunOptions::default();
    assert!(o.apply("gunport", Some("abc")).is_err());
    assert_eq!(o.gun_port, 2310);
}

#[test]
fn tchat_and_tchatdll_belong_to_the_turbine_client() {
    use super::options::TurbineClientOptions;
    let mut o = TurbineClientOptions::default();
    assert_eq!(TurbineClientOptions::takes_value("tchat"), Some(false));
    assert_eq!(TurbineClientOptions::takes_value("tchatdll"), Some(true));
    assert_eq!(GunOptions::takes_value("tchat"), None);
    o.apply("tchat", None).unwrap();
    o.apply("tchatdll", Some("Chat.dll")).unwrap();
    assert!(o.tchat);
    assert_eq!(o.tchat_dll, "Chat.dll");
}

#[test]
fn gunticket_needs_a_gunserver() {
    let mut o = GunOptions::default();
    assert!(o.validate().is_ok());
    o.apply("gunticket", None).unwrap();
    assert_eq!(
        o.validate(),
        Err("You must specify a gun server in order to use GUN ticketing")
    );
    o.apply("gunserver", Some("s")).unwrap();
    assert!(o.validate().is_ok());
}

#[test]
fn hear_tell_appends_the_meta_letters_to_the_teller() {
    let rig = connected_rig();
    let gun = rig.client.turbine_gun().unwrap();
    let extra = blob_for(0x20, "ms");
    rig.handle.queue_message(event(
        id::USER_SPEAK_EVENT,
        MessageBody::UserSpeak {
            user_name: "Bob".into(),
            kind: 0,
            text: "hi there".into(),
            extra: Some(extra),
        },
    ));
    gun.check_for_messages();
    assert_eq!(
        shared().take_outputs(),
        [GunOutput::HearTell {
            teller: "Bob[ms]".into(),
            message: "hi there".into(),
            flags: 0x20,
        }]
    );
}

#[test]
fn hear_tell_without_a_usable_blob_has_no_suffix_and_flags_zero() {
    let rig = connected_rig();
    let gun = rig.client.turbine_gun().unwrap();
    for extra in [
        Some(ac2_gun::TagBlob::default()),
        Some(ac2_gun::TagBlob {
            cb_size: 4,
            data: Some(vec![1, 0, 0, 0]),
        }),
        Some(blob_for(7, "")),
    ] {
        rig.handle.queue_message(event(
            id::USER_SPEAK_EVENT,
            MessageBody::UserSpeak {
                user_name: "Bob".into(),
                kind: 0,
                text: "x".into(),
                extra,
            },
        ));
    }
    gun.check_for_messages();
    let out = shared().take_outputs();
    let tells: Vec<_> = out
        .iter()
        .map(|o| match o {
            GunOutput::HearTell { teller, flags, .. } => (teller.as_str(), *flags),
            other => panic!("{other:?}"),
        })
        .collect();
    // Short blobs fail to unpack (under 8 bytes): flags 0. An empty meta string: no suffix, but
    // the unpacked weenie flags are handed on.
    assert_eq!(tells, [("Bob", 0), ("Bob", 0), ("Bob", 7)]);
}

#[test]
fn room_speech_appends_the_meta_letters_to_the_speaker() {
    let rig = connected_rig();
    let gun = rig.client.turbine_gun().unwrap();
    gun.set_world_name("Legions");
    rig.handle.queue_message(event(
        id::ROOM_SPEAK_EVENT,
        MessageBody::RoomSpeak {
            room_id: 9,
            user_name: "Ann@Legions".into(),
            kind: 0,
            text: "hello".into(),
            extra: Some(blob_for(3, "fn")),
        },
    ));
    gun.check_for_messages();
    assert_eq!(
        shared().take_outputs(),
        [GunOutput::EventHearFromRoom {
            speaker: "Ann[fn]".into(),
            message: "hello".into(),
            room: 9,
            flags: 3,
        }]
    );
}

#[test]
fn friend_events_become_script_events_with_local_names() {
    let rig = connected_rig();
    let gun = rig.client.turbine_gun().unwrap();
    gun.set_world_name("Legions");
    let status = |n: &str, m: u32, l: &str| UserStatus {
        user_name: n.into(),
        mode: m,
        location: l.into(),
    };
    rig.handle.queue_message(event(
        id::FRIEND_START_EVENT,
        MessageBody::FriendStart(vec![
            status("Al@Legions", 1, "Town"),
            status("Bo@Other", 0, ""),
        ]),
    ));
    rig.handle.queue_message(event(
        id::FRIEND_STATUS_EVENT,
        MessageBody::FriendStatus {
            user_name: "Al@Legions".into(),
            mode: 4,
        },
    ));
    // A start with no friends is dropped by RcvFriendsEvent.
    rig.handle.queue_message(event(
        id::FRIEND_START_EVENT,
        MessageBody::FriendStart(vec![]),
    ));
    gun.check_for_messages();
    let f = |n: &str, l: &str, m| FriendInfo {
        name: n.into(),
        location: l.into(),
        mode: m,
    };
    assert_eq!(
        shared().take_outputs(),
        [
            GunOutput::EventFriendStart(vec![f("Al", "Town", 1), f("Bo@Other", "", 0)]),
            GunOutput::EventFriendStatus(f("Al", "", 4)),
        ]
    );
}

#[test]
fn friend_add_routes_to_the_gun_and_the_result_reaches_the_script() {
    let rig = connected_rig();
    rig.client.turbine_gun().unwrap().set_world_name("Legions");
    assert_eq!(request_gun_friend_add("Al"), 1);
    let calls = rig.handle.calls();
    // LocalToGlobalName appends @World.
    let Call::FriendAdd(name, ctx) = calls[0].clone() else {
        panic!("{calls:?}");
    };
    assert_eq!(name, "Al@Legions");
    rig.handle.queue_message(result(
        id::FRIEND_ADD_RESULT,
        ctx,
        0xC112_1009,
        MessageBody::None,
    ));
    rig.client.turbine_gun().unwrap().check_for_messages();
    // ZT_E_ALREADY_IN_LIST -> 0xC740.
    assert_eq!(
        shared().take_outputs(),
        [GunOutput::ResultFriendAdd {
            result: 0xC740,
            name: "Al".into()
        }]
    );
}

#[test]
fn results_for_the_other_requests() {
    let rig = connected_rig();
    let gun = rig.client.turbine_gun().unwrap();
    assert_eq!(request_gun_get_friends_list(1), 1);
    assert_eq!(request_gun_get_ignored_list(0), 1);
    assert_eq!(request_gun_ignored_add("Cy"), 1);
    assert_eq!(request_gun_call_create(3, "World", "Dee", "help"), 1);
    assert_eq!(request_gun_send_to_user("Eve", "psst", 5), 1);
    let ctxs: Vec<Context> = rig
        .handle
        .calls()
        .into_iter()
        .filter_map(|c| match c {
            Call::GetFriendsList(c)
            | Call::GetIgnoredList(c)
            | Call::IgnoredAdd(_, c)
            | Call::SendToUserEx { context: c, .. } => Some(c),
            Call::CallCreate { context, .. } => Some(context),
            _ => None,
        })
        .collect();
    assert_eq!(ctxs.len(), 5);
    let queue = |m| rig.handle.queue_message(m);
    queue(result(
        id::GET_FRIENDS_LIST_RESULT,
        ctxs[0],
        0,
        MessageBody::GetFriendsList(vec![UserStatus {
            user_name: "Fy".into(),
            mode: 1,
            location: "".into(),
        }]),
    ));
    queue(result(
        id::GET_IGNORED_LIST_RESULT,
        ctxs[1],
        0,
        MessageBody::GetIgnoredList(vec!["Gi".into()]),
    ));
    queue(result(
        id::IGNORED_ADD_RESULT,
        ctxs[2],
        0xC112_1401,
        MessageBody::None,
    ));
    queue(result(
        id::CALL_CREATE_RESULT,
        ctxs[3],
        0,
        MessageBody::CallCreate {
            queue_id: 11,
            queue_pos: 2,
        },
    ));
    queue(result(
        id::SEND_TO_USER_EX_RESULT,
        ctxs[4],
        0x8000_000A,
        MessageBody::None,
    ));
    gun.check_for_messages();
    let f = FriendInfo {
        name: "Fy".into(),
        location: String::new(),
        mode: 1,
    };
    assert_eq!(
        shared().take_outputs(),
        [
            GunOutput::ResultGetFriendsList {
                result: 0,
                user_requested: 1,
                list: vec![f]
            },
            GunOutput::ResultGetIgnoredList {
                result: 0,
                user_requested: 0,
                list: vec!["Gi".into()]
            },
            GunOutput::ResultIgnoredAdd {
                result: 0xC73E,
                name: "Cy".into()
            },
            GunOutput::ResultCallCreate {
                result: 0,
                call_type: 3,
                target: "Dee".into(),
                description: "help".into(),
                queue_id: 11,
                queue_pos: 2
            },
            GunOutput::ResultSendToUser {
                user: "Eve".into(),
                message: "psst".into(),
                result: 0xC73C
            },
        ]
    );
}

#[test]
fn requests_return_zero_without_a_turbine_gun() {
    let _rig = rig(GunOptions::default(), "");
    assert!(shared().turbine_gun().is_none());
    assert_eq!(request_gun_friend_add("a"), 0);
    assert_eq!(request_gun_friend_delete("a"), 0);
    assert_eq!(request_gun_get_friends_list(1), 0);
    assert_eq!(request_gun_ignored_add("a"), 0);
    assert_eq!(request_gun_ignored_delete("a"), 0);
    assert_eq!(request_gun_get_ignored_list(1), 0);
    assert_eq!(request_gun_call_create(1, "w", "t", "d"), 0);
    assert_eq!(request_gun_send_to_user("a", "b", 0), 0);
    assert_eq!(request_gun_set_state_flags(1), 0);
    assert_eq!(request_gun_set_location(1, ""), 0);
    // SendToRoom: 0 for room 0 or an empty message, else the left-over arg2 (never 0).
    assert_eq!(request_gun_send_to_room(0, "m", 0), 0);
    assert_eq!(request_gun_send_to_room(5, "", 0), 0);
    assert_ne!(request_gun_send_to_room(5, "m", 0), 0);
}

#[test]
fn send_to_user_needs_a_user_and_a_message() {
    let rig = connected_rig();
    assert_eq!(request_gun_send_to_user("", "m", 0), 0);
    assert_eq!(request_gun_send_to_user("u", "", 0), 0);
    assert!(rig.handle.calls().is_empty());
}

#[test]
fn send_to_room_picks_turbine_chat_or_gun() {
    let rig = connected_rig();
    shared().set_player_name("Sam [ms]");
    // GUN: RequestSendToRoomEx with the packed GunChatExtraInfo (flags 0x44, letters "ms").
    assert_eq!(request_gun_send_to_room(7, "yo", 0x44), 1);
    let expected = GunChatExtraInfo::new(0x44, "ms".into()).encode();
    // [u16 1][u16 0][u32 0x44] then the munged string: 12 bytes for an empty string.
    assert_eq!(&expected[..8], &[1, 0, 0, 0, 0x44, 0, 0, 0]);
    let calls = rig.handle.calls();
    let Call::SendToRoomEx {
        room_id,
        message,
        blob,
        ..
    } = &calls[0]
    else {
        panic!("{calls:?}");
    };
    assert_eq!((*room_id, message.as_str()), (7, "yo"));
    assert_eq!(blob, &expected);
    assert!(shared().take_turbine_sends().is_empty());
    // Turbine chat (a character set with m_fUseTurbineChat): the chat client, not GUN.
    rig.client.recv_notice_character_set(true);
    rig.handle.clear_calls();
    // No Turbine chat client yet (not in the world): CSendToTurbineRoomByID returns 0.
    shared().set_chat_client(false);
    assert_eq!(request_gun_send_to_room(7, "yo", 0x44), 0);
    assert!(shared().take_turbine_sends().is_empty());
    shared().set_chat_client(true);
    assert_eq!(request_gun_send_to_room(7, "yo", 0x44), 1);
    assert!(rig.handle.calls().is_empty());
    let sends = shared().take_turbine_sends();
    assert_eq!(sends.len(), 1);
    assert_eq!((sends[0].room, sends[0].text.as_str()), (7, "yo"));
    assert_eq!(sends[0].extra, expected);
    // The chat reply answers the context: OnSendToRoomByIDResult -> Result_SendToRoom, the
    // result converted (ZT_E_INVALID_ROOM 0xC1121800 -> 0xC742).
    shared().register_turbine_context(55, &sends[0]);
    shared().answer_turbine_send(55, 0xC112_1800);
    assert_eq!(
        shared().take_outputs(),
        [GunOutput::ResultSendToRoom {
            message: "yo".into(),
            room: 7,
            result: 0xC742
        }]
    );
    // Answered once.
    shared().answer_turbine_send(55, 0);
    assert!(shared().take_outputs().is_empty());
}

#[test]
fn extra_info_for_an_empty_name_is_twelve_bytes() {
    let _rig = connected_rig();
    shared().set_player_name("");
    let blob = chat_extra_blob(1);
    assert_eq!(blob.cb_size, 12);
    assert_eq!(&blob.bytes()[..8], &[1, 0, 0, 0, 1, 0, 0, 0]);
}

#[test]
fn state_flags_and_location_use_the_friends_interface() {
    let rig = connected_rig();
    assert_eq!(request_gun_set_state_flags(2), 1);
    assert_eq!(request_gun_set_location(4, "Here"), 1);
    let calls = rig.handle.calls();
    assert!(matches!(
        calls[0],
        Call::Flags {
            op: ac2_gun::FlagOperation::Set,
            which: ac2_gun::FlagWord::State,
            bits: 2,
            ..
        }
    ));
    assert!(matches!(&calls[1], Call::SetLocation { mode: 4, location, .. } if location == "Here"));
}

#[test]
fn a_failing_request_hresult_gives_zero() {
    let rig = connected_rig();
    rig.handle.script("friend_add", Hresult(0x8000_4005));
    assert_eq!(request_gun_friend_add("a"), 0);
}

#[test]
fn connect_events_become_notices() {
    use ac2_gun::ct::{CtConnectEvent, EConnectEvent};
    let _rig = connected_rig();
    let ev = super::handlers::CCliConnectEvent::new();
    for e in [
        EConnectEvent::Connect,
        EConnectEvent::Disconnect,
        EConnectEvent::Undef,
        EConnectEvent::ConnectFailed,
    ] {
        ev.handle_connect_event(e);
    }
    // Only the connection-lost notice has a receiver outside the gmClient (0x469 has none,
    // CONNECT_FAILED sends nothing).
    assert_eq!(
        shared().take_outputs(),
        [
            GunOutput::Notice(super::output::GunNotice::ConnectionLost),
            GunOutput::Notice(super::output::GunNotice::ConnectionLost)
        ]
    );
}

#[test]
fn handlers_do_nothing_without_a_player() {
    let rig = connected_rig();
    shared().set_player(false);
    assert_eq!(request_gun_friend_add("a"), 1);
    let Call::FriendAdd(_, ctx) = rig.handle.calls()[0].clone() else {
        panic!()
    };
    rig.handle
        .queue_message(result(id::FRIEND_ADD_RESULT, ctx, 0, MessageBody::None));
    rig.client.turbine_gun().unwrap().check_for_messages();
    assert!(shared().take_outputs().is_empty());
}

#[test]
fn meta_letters_are_taken_between_the_brackets() {
    use super::chat_extra::retrieve_meta_letters;
    assert_eq!(retrieve_meta_letters("Sword"), ("Sword".into(), "".into()));
    assert_eq!(
        retrieve_meta_letters("Sword  [ms]"),
        ("Sword".into(), "ms".into())
    );
    assert_eq!(
        retrieve_meta_letters("A [x] tail"),
        ("A".into(), "x".into())
    );
}
