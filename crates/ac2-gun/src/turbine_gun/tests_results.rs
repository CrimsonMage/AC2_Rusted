//! Tests of the result and event handlers and of `RcvResult`'s context handling
//! (`0c_client.md` sections 3.2-3.4).

use std::rc::Rc;

use super::tests::*;
use crate::ct::*;
use crate::interfaces::{Context, TagBlob};
use crate::message::{ChatRoomAttrs, Message, MessageBody, ResultHeader, UserStatus};
use crate::message_id as id;
use crate::zt::*;
use crate::TurbineGun;
use crate::{Hresult, MessageId};

fn result(id: MessageId, ctx: Context, hr: Hresult, body: MessageBody) -> Message {
    Message {
        inited: true,
        id,
        result: Some(ResultHeader { context: ctx, hr }),
        body,
    }
}

fn uninited(id: MessageId, ctx: Context) -> Message {
    Message {
        inited: false,
        ..result(id, ctx, Hresult::S_OK, MessageBody::None)
    }
}

fn status(name: &str, mode: u32, location: &str) -> UserStatus {
    UserStatus {
        user_name: name.into(),
        mode,
        location: location.into(),
    }
}

#[test]
fn rcv_result_stores_hr_counts_the_event_calls_the_handler_and_releases() {
    // RcvResult (admin 0x00605900 / Legions 0x005C67E0)
    let rig = Rig::connected();
    let ctx = rig.probe("fa");
    assert!(rig.gun.request_friend_add("Birch", Some(ctx.clone())));
    let cx = match &rig.handle.calls()[0] {
        crate::test_support::Call::FriendAdd(_, c) => *c,
        other => panic!("{other:?}"),
    };
    let message = result(id::FRIEND_ADD_RESULT, cx, ZT_E_NOT_FOUND, MessageBody::None);
    assert_eq!(rig.gun.rcv_result(&message), Hresult::S_OK);
    assert_eq!(ctx.h_result(), ZT_E_NOT_FOUND);
    assert_eq!(ctx.num_events_handled(), 1);
    assert_eq!(rig.log(), ["fa:friend_add"]);
    // released: the entry is gone, the result is used exactly once
    assert_eq!(rig.gun.pending_context_count(), 0);
    assert_eq!(Rc::strong_count(&ctx), 1);
    assert_eq!(rig.gun.rcv_result(&message), Hresult::S_OK);
    assert_eq!(ctx.num_events_handled(), 1);
    assert_eq!(rig.log().len(), 1);
}

#[test]
fn rcv_result_routes_by_family_and_still_releases_for_an_unrouted_id() {
    let rig = Rig::connected();
    let ctx = rig.probe("x");
    let cx = rig
        .gun
        .register_context(PendingContext::FriendAdd(ctx.clone()));
    // 0x1000 (connect results) and 0x6000 have no handler: ZT_E_INTERNAL, context released
    let message = result(id::INIT_RESULT, cx, Hresult::S_OK, MessageBody::None);
    assert_eq!(rig.gun.rcv_result(&message), ZT_E_INTERNAL);
    assert_eq!(rig.gun.pending_context_count(), 0);
    assert_eq!(ctx.num_events_handled(), 1);
    assert_eq!(
        rig.gun.rcv_result(&result(
            MessageId(0x6200),
            Context(0),
            Hresult::S_OK,
            MessageBody::None
        )),
        ZT_E_INTERNAL
    );
}

#[test]
fn an_uninited_result_neither_touches_nor_releases_its_context() {
    let rig = Rig::connected();
    let ctx = rig.probe("x");
    let cx = rig
        .gun
        .register_context(PendingContext::FriendAdd(ctx.clone()));
    // prologue: hr = status = ZT_E_NOT_INITED
    assert_eq!(
        rig.gun.rcv_result(&uninited(id::FRIEND_ADD_RESULT, cx)),
        ZT_E_NOT_INITED
    );
    assert_eq!(rig.gun.pending_context_count(), 1);
    assert_eq!(ctx.num_events_handled(), 0);
    assert!(rig.log().is_empty());
}

#[test]
fn friends_results() {
    // RcvFriendsResult (admin 0x00605140 / Legions 0x005C6020)
    let rig = Rig::connected();
    let p = rig.probe("p");
    let list = vec![status("A", 1, "x"), status("B", 0, "")];
    for (rid, body, ctx) in [
        (
            id::GET_FRIENDS_LIST_RESULT,
            MessageBody::GetFriendsList(list.clone()),
            PendingContext::GetFriendsList(p.clone()),
        ),
        (
            id::GET_IGNORED_LIST_RESULT,
            MessageBody::GetIgnoredList(vec!["n1".into(), "n2".into()]),
            PendingContext::GetIgnoredList(p.clone()),
        ),
        (
            id::FRIEND_ADD_RESULT,
            MessageBody::None,
            PendingContext::FriendAdd(p.clone()),
        ),
        (
            id::FRIEND_DELETE_RESULT,
            MessageBody::None,
            PendingContext::FriendDelete(p.clone()),
        ),
        (
            id::IGNORED_ADD_RESULT,
            MessageBody::None,
            PendingContext::IgnoredAdd(p.clone()),
        ),
        (
            id::IGNORED_DELETE_RESULT,
            MessageBody::None,
            PendingContext::IgnoredDelete(p.clone()),
        ),
        (
            id::SET_LOCATION_RESULT,
            MessageBody::None,
            PendingContext::Location(p.clone()),
        ),
        (
            id::FLAGS_RESULT,
            MessageBody::Flags {
                which_flag: 1,
                flags: 0x18,
            },
            PendingContext::Flags(p.clone()),
        ),
    ] {
        let cx = rig.gun.register_context(ctx);
        assert_eq!(
            rig.gun.rcv_result(&result(rid, cx, Hresult::S_OK, body)),
            Hresult::S_OK
        );
    }
    assert_eq!(
        rig.log(),
        [
            "p:friends_list 2",
            "p:ignored_list [\"n1\", \"n2\"]",
            "p:friend_add",
            "p:friend_delete",
            "p:ignored_add",
            "p:ignored_delete",
            "p:location",
            "p:flags 1 24"
        ]
    );
}

#[test]
fn friends_lists_and_flags_read_nothing_when_hr_is_not_zero() {
    let rig = Rig::connected();
    let p = rig.probe("p");
    let cx = rig
        .gun
        .register_context(PendingContext::GetFriendsList(p.clone()));
    rig.gun.rcv_result(&result(
        id::GET_FRIENDS_LIST_RESULT,
        cx,
        ZT_E_INTERNAL,
        MessageBody::GetFriendsList(vec![status("A", 1, "")]),
    ));
    let cx = rig.gun.register_context(PendingContext::Flags(p.clone()));
    rig.gun.rcv_result(&result(
        id::FLAGS_RESULT,
        cx,
        ZT_E_INTERNAL,
        MessageBody::Flags {
            which_flag: 1,
            flags: 2,
        },
    ));
    // the handler still runs, with count 0 / flags 0 0, and the status stays 0
    assert_eq!(rig.log(), ["p:friends_list 0", "p:flags 0 0"]);
}

#[test]
fn friends_result_without_a_context_and_the_ids_that_do_nothing_return_the_status() {
    let rig = Rig::connected();
    for rid in [
        id::FRIEND_ADD_RESULT,
        id::WHERE_IS_RESULT,
        id::ACCEPT_ADD_FRIEND_RESULT,
        id::GET_ID_FROM_NAME_RESULT,
        id::GET_NAME_FROM_ID_RESULT,
        MessageId(0x420c),
    ] {
        assert_eq!(
            rig.gun
                .rcv_friends_result(&result(rid, Context(0), Hresult::S_OK, MessageBody::None)),
            Hresult::S_OK,
            "{rid:?}"
        );
    }
    // unlike chat, an unknown friends id is not ZT_E_INTERNAL; uninited gives NOT_INITED
    assert_eq!(
        rig.gun
            .rcv_friends_result(&uninited(MessageId(0x4299), Context(0))),
        ZT_E_NOT_INITED
    );
    assert!(rig.log().is_empty());
}

#[test]
fn chat_results() {
    // RcvChatResult (admin 0x00604CF0 / Legions 0x005C5BD0)
    let rig = Rig::connected();
    let p = rig.probe("p");
    let join = MessageBody::JoinRoom {
        room_id: 5,
        server_id: 9,
    };
    let create = MessageBody::CreateRoom {
        room_id: 6,
        server_id: 8,
    };
    for (rid, body, ctx) in [
        (
            id::JOIN_ROOM_RESULT,
            join.clone(),
            PendingContext::JoinRoom(p.clone()),
        ),
        (
            id::CREATE_ROOM_RESULT,
            create,
            PendingContext::CreateRoom(p.clone()),
        ),
        (
            id::LEAVE_ROOM_RESULT,
            MessageBody::None,
            PendingContext::LeaveRoom(p.clone()),
        ),
        (
            id::LEAVE_ALL_ROOMS_RESULT,
            MessageBody::None,
            PendingContext::LeaveAllRooms(p.clone()),
        ),
        (
            id::SEND_TO_ROOM_RESULT,
            MessageBody::None,
            PendingContext::SendToRoom(p.clone()),
        ),
        (
            id::SEND_TO_ROOM_EX_RESULT,
            MessageBody::None,
            PendingContext::SendToRoom(p.clone()),
        ),
        (
            id::SEND_TO_USER_RESULT,
            MessageBody::None,
            PendingContext::SendToUser(p.clone()),
        ),
        (
            id::SEND_TO_USER_EX_RESULT,
            MessageBody::None,
            PendingContext::SendToUser(p.clone()),
        ),
        (
            id::CREATE_PERSISTENT_CHAT_CHANNEL_RESULT,
            MessageBody::None,
            PendingContext::CreatePersistentChatChannel(p.clone()),
        ),
        (
            id::DELETE_PERSISTENT_CHAT_CHANNEL_RESULT,
            MessageBody::None,
            PendingContext::DeletePersistentChatChannel(p.clone()),
        ),
    ] {
        let cx = rig.gun.register_context(ctx);
        assert_eq!(
            rig.gun.rcv_result(&result(rid, cx, ZT_E_NOT_FOUND, body)),
            Hresult::S_OK,
            "{rid:?}"
        );
    }
    assert_eq!(
        rig.log(),
        [
            // hr != 0: (roomId, serverId) = (0, 0), the handler still runs
            "p:join 0 0",
            "p:create 0 0",
            "p:leave_room",
            "p:leave_all",
            "p:send_to_room",
            "p:send_to_room",
            "p:send_to_user",
            "p:send_to_user",
            // 0x5212: ctx->m_hResult = hr first, then +0xC(0)
            "p:create_channel 0 hr=0xc1121c04",
            "p:delete_channel"
        ]
    );
    // with hr == 0 the ids are read
    let cx = rig
        .gun
        .register_context(PendingContext::JoinRoom(p.clone()));
    rig.gun
        .rcv_result(&result(id::JOIN_ROOM_RESULT, cx, Hresult::S_OK, join));
    assert_eq!(rig.log().last().map(String::as_str), Some("p:join 5 9"));
}

#[test]
fn chat_result_ids_that_only_print_or_are_not_in_the_table() {
    let rig = Rig::connected();
    let rooms = MessageBody::GetListOfRooms {
        index: 0,
        total: 1,
        rooms: vec![ChatRoomAttrs {
            name: "r".into(),
            mode: 1,
            ..Default::default()
        }],
    };
    for (rid, body) in [
        (id::GET_LIST_OF_ROOMS_RESULT, rooms),
        (id::GET_USERS_IN_ROOM_RESULT, MessageBody::None),
        (id::GET_SQUELCH_LIST_RESULT, MessageBody::None),
        (id::SQUELCH_ADD_RESULT, MessageBody::None),
        (id::SQUELCH_DELETE_RESULT, MessageBody::None),
        (id::GET_ROOM_OWNER_RESULT, MessageBody::None),
        (id::ASSIGN_OWNERSHIP_RESULT, MessageBody::None),
        (id::SET_ROOM_MODE_RESULT, MessageBody::None),
        (id::SET_ROOM_TOPIC_RESULT, MessageBody::None),
        (id::INVITE_USER_RESULT, MessageBody::None),
        (id::KICK_OUT_USER_RESULT, MessageBody::None),
        (id::USER_SAY_RESULT, MessageBody::None),
    ] {
        assert_eq!(
            rig.gun
                .rcv_chat_result(&result(rid, Context(0), Hresult::S_OK, body)),
            Hresult::S_OK,
            "{rid:?}"
        );
    }
    // the table's default entry: 0x5214-0x521B and above 0x521D
    for rid in [0x5214, 0x521b, 0x521e, 0x5300] {
        assert_eq!(
            rig.gun.rcv_chat_result(&result(
                MessageId(rid),
                Context(0),
                Hresult::S_OK,
                MessageBody::None
            )),
            ZT_E_INTERNAL,
            "{rid:#x}"
        );
    }
    assert_eq!(
        rig.gun
            .rcv_chat_result(&uninited(id::JOIN_ROOM_RESULT, Context(0))),
        ZT_E_NOT_INITED
    );
}

#[test]
fn a_context_of_the_wrong_class_is_not_called() {
    let rig = Rig::connected();
    let p = rig.probe("p");
    let cx = rig.gun.register_context(PendingContext::FriendAdd(p));
    rig.gun.rcv_result(&result(
        id::SEND_TO_ROOM_RESULT,
        cx,
        Hresult::S_OK,
        MessageBody::None,
    ));
    assert!(rig.log().is_empty());
    assert_eq!(rig.gun.pending_context_count(), 0);
}

#[test]
fn accounts_news_match_and_support_results() {
    let rig = Rig::connected();
    rig.gun.logged_in.set(true);
    // RcvAccountsResult: 0x2200 clears m_fLoggedIn; the prologue status
    assert_eq!(
        rig.gun.rcv_accounts_result(&result(
            MessageId(0x2201),
            Context(0),
            Hresult::S_OK,
            MessageBody::None
        )),
        Hresult::S_OK
    );
    assert!(rig.gun.is_logged_in());
    assert_eq!(
        rig.gun.rcv_accounts_result(&result(
            id::LOGIN_RESULT,
            Context(0),
            Hresult::S_OK,
            MessageBody::None
        )),
        Hresult::S_OK
    );
    assert!(!rig.gun.is_logged_in());
    assert_eq!(
        rig.gun
            .rcv_accounts_result(&uninited(id::LOGIN_RESULT, Context(0))),
        ZT_E_NOT_INITED
    );
    assert_eq!(
        rig.gun.rcv_news_result(&result(
            id::GET_NEWS_RESULT,
            Context(0),
            Hresult::S_OK,
            MessageBody::None
        )),
        Hresult::S_OK
    );
    assert_eq!(
        rig.gun
            .rcv_news_result(&uninited(id::GET_NEWS_RESULT, Context(0))),
        ZT_E_NOT_INITED
    );
    assert_eq!(
        rig.gun
            .rcv_match_result(&uninited(id::GET_PAGE_RESULT, Context(0))),
        ZT_E_NOT_INITED
    );
    // RcvSupportResult: 0x10200 -> ctx(queueId, queuePos); status 0 always
    let p = rig.probe("p");
    let cx = rig
        .gun
        .register_context(PendingContext::CallCreate(p.clone()));
    assert_eq!(
        rig.gun.rcv_result(&result(
            id::CALL_CREATE_RESULT,
            cx,
            Hresult::S_OK,
            MessageBody::CallCreate {
                queue_id: 7,
                queue_pos: 3
            }
        )),
        Hresult::S_OK
    );
    let cx = rig
        .gun
        .register_context(PendingContext::CallCreate(p.clone()));
    rig.gun.rcv_result(&result(
        id::CALL_CREATE_RESULT,
        cx,
        ZT_E_INTERNAL,
        MessageBody::CallCreate {
            queue_id: 7,
            queue_pos: 3,
        },
    ));
    assert_eq!(rig.log(), ["p:call_create 7 3", "p:call_create 0 0"]);
    assert_eq!(
        rig.gun
            .rcv_support_result(&uninited(id::CALL_CREATE_RESULT, Context(0))),
        Hresult::S_OK
    );
    assert_eq!(
        rig.gun.rcv_support_result(&result(
            id::CALL_GET_STATUS_RESULT,
            Context(0),
            Hresult::S_OK,
            MessageBody::None
        )),
        Hresult::S_OK
    );
}

fn auth(rig: &Rig, rid: MessageId, hr: Hresult, body: MessageBody) -> (Hresult, Rc<Probe>) {
    let p = rig.probe("t");
    let cx = rig.gun.register_context(PendingContext::Ticket(p.clone()));
    (rig.gun.rcv_result(&result(rid, cx, hr, body)), p)
}

#[test]
fn authenticate_results_map_hr_to_the_ticket_result() {
    // RcvAuthenticateResult (admin 0x006054D0 / Legions 0x005C63B0)
    let rig = Rig::connected();
    for (rid, hr) in [
        (id::CHECK_TICKET_RESULT, 0u32),
        (id::PRESENT_TICKET_RESULT, 0),
        (id::PRESENT_TICKET_RESULT, 0xC112080C),
        (id::PRESENT_TICKET_RESULT, 0xC1121005),
        (id::PRESENT_TICKET_RESULT, 0xC1121006),
        (id::PRESENT_TICKET_RESULT, 0xC1120003),
        (id::CHECK_TICKET_RESULT, 0xC112080B),
    ] {
        let (status, _) = auth(&rig, rid, Hresult(hr), MessageBody::None);
        assert_eq!(status, Hresult::S_OK);
    }
    assert_eq!(
        rig.log(),
        [
            "t:auth 0x0",
            "t:auth 0x0",
            "t:auth 0xc112080c",
            "t:auth 0xc1121005",
            "t:auth 0xc1121006",
            "t:auth 0xc1121006",
            "t:auth 0xc1121006"
        ]
    );
}

#[test]
fn release_ticket_result_uses_the_third_slot() {
    let rig = Rig::connected();
    for hr in [0xC112080Bu32, 0, 0xC112080C] {
        auth(
            &rig,
            id::RELEASE_TICKET_RESULT,
            Hresult(hr),
            MessageBody::None,
        );
    }
    assert_eq!(
        rig.log(),
        [
            "t:release 0xc112080b",
            "t:release 0x0",
            "t:release 0xc1121006"
        ]
    );
}

#[test]
fn get_ticket_result_copies_the_ticket_then_reports_success() {
    let rig = Rig::connected();
    let ticket = TagBlob {
        cb_size: 3,
        data: Some(vec![1, 2, 3]),
    };
    let (status, p) = auth(
        &rig,
        id::GET_TICKET_RESULT,
        Hresult::S_OK,
        MessageBody::GetTicket(Some(ticket)),
    );
    assert_eq!(status, Hresult::S_OK);
    assert_eq!(rig.log(), ["t:arrived 0x0"]);
    let copy = p.get_ticket_blob().unwrap();
    assert_eq!((copy.cb_size, copy.bytes()), (3, &[1u8, 2, 3][..]));
    // hr != 0: FAILURE, nothing copied
    let (_, p) = auth(
        &rig,
        id::GET_TICKET_RESULT,
        ZT_E_INTERNAL,
        MessageBody::None,
    );
    assert_eq!(
        rig.log().last().map(String::as_str),
        Some("t:arrived 0xc1121006")
    );
    assert!(p.get_ticket_blob().is_none());
}

#[test]
fn authenticate_ids_without_a_slot() {
    let rig = Rig::connected();
    // 0xF205 returns the prologue status, 0xF204 and the rest ZT_E_INTERNAL
    assert_eq!(
        auth(
            &rig,
            id::SET_GAME_SERVER_RESULT,
            Hresult::S_OK,
            MessageBody::None
        )
        .0,
        Hresult::S_OK
    );
    assert_eq!(
        auth(&rig, id::BOOT_USER_RESULT, Hresult::S_OK, MessageBody::None).0,
        ZT_E_INTERNAL
    );
    assert_eq!(
        auth(&rig, MessageId(0xf300), Hresult::S_OK, MessageBody::None).0,
        ZT_E_INTERNAL
    );
    assert!(rig.log().is_empty());
    assert_eq!(
        rig.gun
            .rcv_authenticate_result(&uninited(id::PRESENT_TICKET_RESULT, Context(0))),
        ZT_E_NOT_INITED
    );
}

#[test]
fn persona_results() {
    let rig = Rig::connected();
    let p = rig.probe("p");
    for (rid, body) in [
        (
            id::ENTER_PERSONA_RESULT,
            MessageBody::EnterPersona {
                kind: 2,
                server_id: 4,
            },
        ),
        (id::LEAVE_PERSONA_RESULT, MessageBody::None),
        (id::CREATE_PERSONA_RESULT, MessageBody::None),
        (id::DELETE_PERSONA_RESULT, MessageBody::None),
        (
            id::GET_PERSONA_LIST_RESULT,
            MessageBody::GetPersonaList(vec!["a".into()]),
        ),
    ] {
        let cx = rig.gun.register_context(PendingContext::Persona(p.clone()));
        assert_eq!(
            rig.gun.rcv_result(&result(rid, cx, Hresult::S_OK, body)),
            Hresult::S_OK
        );
    }
    assert_eq!(
        rig.log(),
        [
            "p:enter_persona 2 4",
            "p:leave_persona",
            "p:create_persona",
            "p:delete_persona",
            "p:persona_list [\"a\"]"
        ]
    );
    assert_eq!(
        rig.gun.rcv_persona_result(&result(
            MessageId(0x8205),
            Context(0),
            Hresult::S_OK,
            MessageBody::None
        )),
        ZT_E_INTERNAL
    );
}

#[test]
fn friend_start_with_no_statuses_is_dropped() {
    // RcvFriendsEvent (admin 0x00606C20 / Legions 0x005C7AC0)
    let rig = Rig::connected().with_world();
    let l = rig.probe("f");
    rig.gun.register_for_friends_events(Some(l.clone()));
    let status_a = status("Birch@Legions", 1, "Hall");
    assert_eq!(
        rig.gun.rcv_friends_event(&event_with(
            id::FRIEND_START_EVENT,
            MessageBody::FriendStart(vec![])
        )),
        Hresult::S_OK
    );
    assert!(rig.log().is_empty());
    assert_eq!(l.num_events_handled(), 0);
    rig.gun.rcv_friends_event(&event_with(
        id::FRIEND_START_EVENT,
        MessageBody::FriendStart(vec![status_a]),
    ));
    // the names are handed over as the event has them: stripping is the CCli handler's job
    assert_eq!(rig.log(), ["f:start [\"Birch@Legions\"]"]);
    assert_eq!(l.num_events_handled(), 1);
}

#[test]
fn friend_status_strips_the_world_name_regardless_of_the_chat_flag() {
    let rig = Rig::connected().with_world();
    let l = rig.probe("f");
    rig.gun.register_for_friends_events(Some(l));
    TurbineGun::set_strip_world_name_from_room_chat_events(false);
    rig.gun.rcv_friends_event(&event_with(
        id::FRIEND_STATUS_EVENT,
        MessageBody::FriendStatus {
            user_name: "Birch@Legions".into(),
            mode: 9,
        },
    ));
    TurbineGun::set_strip_world_name_from_room_chat_events(true);
    assert_eq!(rig.log(), ["f:status Birch 9"]);
    // 0x430E, 0x430F and unknown ids are ignored
    for rid in [
        id::FRIEND_REQUEST_EVENT,
        id::FRIEND_ADD_COMPLETE_EVENT,
        MessageId(0x4310),
    ] {
        assert_eq!(rig.gun.rcv_friends_event(&event(rid)), Hresult::S_OK);
    }
    assert_eq!(rig.log().len(), 1);
}

#[test]
fn chat_events_route_and_strip_only_the_enter_leave_and_room_speak_users() {
    // RcvChatEvent (admin 0x00606710 / Legions 0x005C75B0)
    let rig = Rig::connected().with_world();
    let older = rig.probe("old");
    let newer = rig.probe("new");
    rig.gun.register_for_chat_events(Some(older));
    rig.gun.register_for_chat_events(Some(newer.clone()));
    let extra = TagBlob {
        cb_size: 2,
        data: Some(vec![1, 2]),
    };
    let events = [
        (
            id::ROOM_ENTER_EVENT,
            MessageBody::RoomEnter {
                room_id: 3,
                user_name: "Birch@Legions".into(),
            },
        ),
        (
            id::ROOM_LEAVE_EVENT,
            MessageBody::RoomLeave {
                room_id: 3,
                user_name: "Oak@Other".into(),
            },
        ),
        (
            id::ROOM_SPEAK_EVENT,
            MessageBody::RoomSpeak {
                room_id: 3,
                user_name: "Birch@Legions".into(),
                kind: 99,
                text: "hi".into(),
                extra: Some(extra.clone()),
            },
        ),
        (
            id::USER_SPEAK_EVENT,
            MessageBody::UserSpeak {
                user_name: "Birch@Legions".into(),
                kind: 1,
                text: "psst".into(),
                extra: None,
            },
        ),
        (
            id::KICKED_OUT_EVENT,
            MessageBody::KickedOut {
                room_id: 3,
                reason: "bye".into(),
            },
        ),
        (
            id::RECEIVE_OWNERSHIP_EVENT,
            MessageBody::ReceiveOwnership { room_id: 3 },
        ),
        (
            id::INVITED_EVENT,
            MessageBody::Invited {
                user_name: "ignored".into(),
                room_name: "room".into(),
                invitation: "join".into(),
            },
        ),
        (id::USER_SAY_EVENT, MessageBody::None),
    ];
    for (eid, body) in events {
        assert_eq!(
            rig.gun.rcv_chat_event(&event_with(eid, body)),
            Hresult::S_OK
        );
    }
    let log = rig.log();
    // newest listener first; 7 events x 2 listeners, UserSay not handled
    assert_eq!(log.len(), 14);
    assert_eq!(log[0], "new:enter 3 Birch");
    assert_eq!(log[1], "old:enter 3 Birch");
    assert_eq!(log[2], "new:leave 3 Oak@Other");
    assert_eq!(log[4], "new:hear_room 3 Birch hi Some([1, 2])");
    assert_eq!(log[6], "new:hear_user Birch@Legions psst None");
    assert_eq!(log[8], "new:kicked 3 bye");
    assert_eq!(log[10], "new:owner 3");
    assert_eq!(log[12], "new:invited room join");
    assert_eq!(newer.num_events_handled(), 7);
}

#[test]
fn chat_events_keep_the_user_when_world_names_are_not_stripped() {
    let rig = Rig::connected().with_world();
    let l = rig.probe("c");
    rig.gun.register_for_chat_events(Some(l));
    TurbineGun::set_strip_world_name_from_room_chat_events(false);
    rig.gun.rcv_chat_event(&event_with(
        id::ROOM_ENTER_EVENT,
        MessageBody::RoomEnter {
            room_id: 1,
            user_name: "Birch@Legions".into(),
        },
    ));
    TurbineGun::set_strip_world_name_from_room_chat_events(true);
    assert_eq!(rig.log(), ["c:enter 1 Birch@Legions"]);
}

#[test]
fn handlers_for_uninited_events_do_nothing() {
    // open: retail reads uninitialised stack values here
    let rig = Rig::connected();
    let l = rig.probe("c");
    rig.gun.register_for_chat_events(Some(l));
    let mut ev = event_with(
        id::ROOM_ENTER_EVENT,
        MessageBody::RoomEnter {
            room_id: 1,
            user_name: "x".into(),
        },
    );
    ev.inited = false;
    assert_eq!(rig.gun.rcv_chat_event(&ev), Hresult::S_OK);
    assert!(rig.log().is_empty());
}
