//! The wire text of every client-used request, built from the `RenderUnicode` templates of
//! `0a_receive_messages.md` Part 2, sent through the `IGun*` interfaces and a fake DP8; plus the
//! argument checks of the `CGun::X` bodies in the decompile.

use super::fakes::*;
use crate::interfaces::*;
use crate::zt::*;
use crate::Hresult;

const CX: u32 = 0x2f4a10;

fn ctx() -> Context {
    Context(CX)
}

/// An initialised, connecting `CGun` (the connection exists, which is all the request interfaces test).
fn connected() -> Rig {
    let r = rig();
    assert_eq!(
        r.gun.init(&APP_GUID, "1.0", 1033, "XXXX-YYYY-ZZZZ", 0),
        Hresult::S_OK
    );
    assert_eq!(r.gun.connect("gun.example.net", 8610), Hresult::E_PENDING);
    r
}

fn sent_one(r: &Rig, hr: Hresult) -> String {
    assert_eq!(hr, Hresult::S_OK);
    assert_eq!(r.dp8.sent_count(), 1);
    r.dp8.sent_text(0)
}

#[test]
fn friends_requests() {
    type Call = Box<dyn Fn(&crate::gundll::CGun) -> Hresult>;
    let cases: Vec<(&str, Call, &str)> = vec![
        (
            "GetFriendsList",
            Box::new(|g| g.get_friends_list(ctx())),
            r#"<GetFriendsList Cx="0x2f4a10" />"#,
        ),
        (
            "FriendAdd",
            Box::new(|g| g.friend_add("Bob@Shard", ctx())),
            r#"<FriendAdd Cx="0x2f4a10" User="Bob@Shard" />"#,
        ),
        (
            "FriendDelete",
            Box::new(|g| g.friend_delete("Bob", ctx())),
            r#"<FriendDelete Cx="0x2f4a10" User="Bob" />"#,
        ),
        (
            "GetIgnoredList",
            Box::new(|g| g.get_ignored_list(ctx())),
            r#"<GetIgnoredList Cx="0x2f4a10" />"#,
        ),
        (
            "IgnoredAdd",
            Box::new(|g| g.ignored_add("Jerk", ctx())),
            r#"<IgnoredAdd Cx="0x2f4a10" User="Jerk" />"#,
        ),
        (
            "IgnoredDelete",
            Box::new(|g| g.ignored_delete("Jerk", ctx())),
            r#"<IgnoredDelete Cx="0x2f4a10" User="Jerk" />"#,
        ),
        (
            "Flags",
            Box::new(|g| g.flags(FlagOperation::Set, FlagWord::State, 6, ctx())),
            r#"<Flags Cx="0x2f4a10" Op="1" Word="1" Flags="6" />"#,
        ),
        (
            "WhereIs",
            Box::new(|g| g.where_is("Bob", ctx())),
            r#"<WhereIs Cx="0x2f4a10" User="Bob" />"#,
        ),
        (
            "SetLocation",
            Box::new(|g| g.set_location(3, "Holtburg", ctx())),
            r#"<SetLoc Cx="0x2f4a10" Flags="3" Location="Holtburg" />"#,
        ),
        (
            "AcceptAddFriend",
            Box::new(|g| g.accept_add_friend(true, "Bob", ctx())),
            r#"<AcceptAddFriend Cx="0x2f4a10" Reply="1" User="Bob" />"#,
        ),
    ];
    for (name, call, expected) in cases {
        let r = connected();
        let hr = call(&r.gun);
        assert_eq!(sent_one(&r, hr), expected, "{name}");
    }
}

#[test]
fn chat_requests() {
    type Call = Box<dyn Fn(&crate::gundll::CGun) -> Hresult>;
    let cases: Vec<(&str, Call, &str)> = vec![
        (
            "GetListOfRooms",
            Box::new(|g| g.get_list_of_rooms(ctx())),
            r#"<GetListOfRooms Cx="0x2f4a10" />"#,
        ),
        (
            "JoinRoom",
            Box::new(|g| g.join_room("Lobby", "pw", ctx())),
            r#"<JoinRoom Cx="0x2f4a10" Room="Lobby" Password="pw" />"#,
        ),
        (
            "LeaveRoom",
            Box::new(|g| g.leave_room(12, ctx())),
            r#"<LeaveRoom Cx="0x2f4a10" RoomId="12" />"#,
        ),
        (
            "LeaveAllRooms",
            Box::new(|g| g.leave_all_rooms(ctx())),
            r#"<LeaveAllRooms Cx="0x2f4a10" />"#,
        ),
        (
            "GetUsersInRoom",
            Box::new(|g| g.get_users_in_room(12, ctx())),
            r#"<GetUsersInRoom Cx="0x2f4a10" RoomId="12" />"#,
        ),
        (
            "SendToRoom",
            Box::new(|g| g.send_to_room(12, "hi \"all\" & <co>", ctx())),
            r#"<SendToRoom Cx="0x2f4a10" RoomId="12" Text="hi &quot;all&quot; &amp; &lt;co&gt;" />"#,
        ),
        (
            "SendToUser",
            Box::new(|g| g.send_to_user("Bob", "hi", ctx())),
            r#"<SendToUser Cx="0x2f4a10" User="Bob" Text="hi" />"#,
        ),
        (
            "GetSquelchList",
            Box::new(|g| g.get_squelch_list(ctx())),
            r#"<GetSquelchList Cx="0x2f4a10" />"#,
        ),
        (
            "SquelchAdd",
            Box::new(|g| g.squelch_add("Jerk", ctx())),
            r#"<SquelchAdd Cx="0x2f4a10" User="Jerk" />"#,
        ),
        (
            "SquelchDelete",
            Box::new(|g| g.squelch_delete("Jerk", ctx())),
            r#"<SquelchDelete Cx="0x2f4a10" User="Jerk" />"#,
        ),
        (
            "CreateRoom",
            Box::new(|g| g.create_room("R", "T", 2, 50, "pw", ctx())),
            r#"<CreateRoom Cx="0x2f4a10" Room="R" Topic="T" Mode="2" Limit="50" Password="pw" />"#,
        ),
        (
            "GetRoomOwner",
            Box::new(|g| g.get_room_owner(12, ctx())),
            r#"<RoomOwner Cx="0x2f4a10" RoomId="12" />"#,
        ),
        (
            "AssignOwnership",
            Box::new(|g| g.assign_ownership(12, "Bob", ctx())),
            r#"<AssignOwnership Cx="0x2f4a10" RoomId="12" User="Bob" />"#,
        ),
        (
            "SetRoomTopic",
            Box::new(|g| g.set_room_topic(12, "news", ctx())),
            r#"<SetRoomTopic Cx="0x2f4a10" RoomId="12" Text="news" />"#,
        ),
        (
            "SetRoomMode",
            Box::new(|g| g.set_room_mode(12, 1, "pw", ctx())),
            r#"<SetRoomMode Cx="0x2f4a10" RoomId="12" Mode="1" Password="pw" />"#,
        ),
        (
            "InviteUser",
            Box::new(|g| g.invite_user(12, "Bob", "come", ctx())),
            r#"<InviteUser Cx="0x2f4a10" RoomId="12" User="Bob" Text="come" />"#,
        ),
        (
            "KickOutUser",
            Box::new(|g| g.kick_out_user(12, "Bob", "bye", ctx())),
            r#"<KickOutUser Cx="0x2f4a10" RoomId="12" User="Bob" Text="bye" />"#,
        ),
        (
            "RetrieveChatLog",
            Box::new(|g| g.retrieve_chat_log("A", "B", ctx())),
            r#"<RetrieveChatLog Cx="0x2f4a10" Persona1="A" Persona2="B" MaxChatLogLength="30000" />"#,
        ),
        (
            "SendToRoomEx",
            Box::new(|g| g.send_to_room_ex(12, "hi", &[1], ctx())),
            r#"<SendToRoomEx Cx="0x2f4a10" RoomId="12" Text="hi" ><Extra Val="ba"/></SendToRoomEx> "#,
        ),
        (
            "SendToUserEx",
            Box::new(|g| g.send_to_user_ex("Bob@Shard", "hi", &[], ctx())),
            r#"<SendToUserEx Cx="0x2f4a10" User="Bob@Shard" Text="hi" ><Extra Val=""/></SendToUserEx> "#,
        ),
    ];
    for (name, call, expected) in cases {
        let r = connected();
        let hr = call(&r.gun);
        assert_eq!(sent_one(&r, hr), expected, "{name}");
    }
}

#[test]
fn support_persona_and_browser_requests() {
    type Call = Box<dyn Fn(&crate::gundll::CGun) -> Hresult>;
    let cases: Vec<(&str, Call, &str)> = vec![
        (
            "CallCreate",
            Box::new(|g| {
                g.call_create(
                    SupportCallType::BUG,
                    Some("Bob"),
                    Some("it broke"),
                    &[1, 2],
                    1033,
                    ctx(),
                )
            }),
            r#"<CallCreate Cx="0x2f4a10" CallType="32773" TargetPersonaName="Bob" Description="it broke" LangID="1033" ><CallData Val="bia"/></CallCreate> "#,
        ),
        (
            "CallGetStatus",
            Box::new(|g| g.call_get_status(77, ctx())),
            r#"<CallGetStatus Cx="0x2f4a10" QueueID="77" />"#,
        ),
        (
            "EnterPersona",
            Box::new(|g| g.enter_persona("Hero", 2, ctx())),
            r#"<EnterPersonaCmd Cx="0x2f4a10" Name="Hero" Shard="2" />"#,
        ),
        (
            "LeavePersona",
            Box::new(|g| g.leave_persona(ctx())),
            r#"<LeavePersonaCmd Cx="0x2f4a10" />"#,
        ),
        (
            "CreatePersona",
            Box::new(|g| g.create_persona("Hero", 2, ctx())),
            r#"<CreatePersonaCmd Cx="0x2f4a10" Name="Hero" Type="1" Shard="2" />"#,
        ),
        (
            "DeletePersona",
            Box::new(|g| g.delete_persona("Hero", 2, ctx())),
            r#"<DeletePersonaCmd Cx="0x2f4a10" Name="Hero" Shard="2" />"#,
        ),
        (
            "GetPersonaList",
            Box::new(|g| g.get_persona_list(2, ctx())),
            r#"<GetPersonaListCmd Cx="0x2f4a10" Shard="2" />"#,
        ),
        (
            "GetViewList",
            Box::new(|g| g.get_view_list(ctx())),
            r#"<ViewList Cx="0x2f4a10" />"#,
        ),
        (
            "GetHeaderRowOfView",
            Box::new(|g| g.get_header_row_of_view(4, ctx())),
            r#"<HdrRow Cx="0x2f4a10" Vid="4" />"#,
        ),
        (
            "GetPage",
            Box::new(|g| g.get_page(4, 1, 20, "s", ctx())),
            r#"<Page Cx="0x2f4a10" Vid="4" PageNo="1" Num="20" Str="s" />"#,
        ),
        (
            "GetRowPage",
            Box::new(|g| g.get_row_page(4, 5, 20, "s", ctx())),
            r#"<RowPg Cx="0x2f4a10" Vid="4" Rid="5" Num="20" Str="s" />"#,
        ),
        (
            "GetAccountStatus",
            Box::new(|g| g.get_account_status("acct", ctx())),
            r#"<GetAccountStatus Cx="0x2f4a10" User="acct" />"#,
        ),
        (
            "GetAccountName",
            Box::new(|g| g.get_account_name("Hero", ctx())),
            r#"<GetAccountName Cx="0x2f4a10" PersonaName="Hero" />"#,
        ),
        (
            "ReleaseTicket",
            Box::new(|g| g.release_ticket(ctx())),
            r#"<ReleaseTicketCmd Cx="0x2f4a10" />"#,
        ),
        (
            "CheckTicket",
            Box::new(|g| g.check_ticket(&[1], ctx())),
            r#"<CheckTicketCmd Cx="0x2f4a10" AccountName="" ><Ticket Val="ba"/></CheckTicketCmd> "#,
        ),
        (
            "BootUser",
            Box::new(|g| g.boot_user("Bob", "spam", ctx())),
            r#"<Boot Cx="0x2f4a10" User="Bob" Reason="2" ReasonText="spam" />"#,
        ),
    ];
    for (name, call, expected) in cases {
        let r = connected();
        let hr = call(&r.gun);
        assert_eq!(sent_one(&r, hr), expected, "{name}");
    }
}

#[test]
fn the_send_flags_and_framing() {
    let r = connected();
    r.gun.friend_add("Bob", ctx());
    let s = r.dp8.0.borrow();
    let (buf, timeout, flags) = &s.sent[0];
    // DPNSEND_NOCOPY | DPNSEND_GUARANTEED, timeout 0, one buffer: 50 C3 + UTF-8 + NUL
    assert_eq!((*timeout, *flags), (0, 0x9));
    assert_eq!(&buf[..2], &[0x50, 0xC3]);
    assert_eq!(&buf[2..], b"<FriendAdd Cx=\"0x2f4a10\" User=\"Bob\" />\0");
}

#[test]
fn text_is_sent_as_utf8() {
    let r = connected();
    r.gun.send_to_user("B\u{e9}b", "\u{4e16}\u{754c}", ctx());
    let s = r.dp8.0.borrow();
    let b = &s.sent[0].0;
    assert!(b.windows(3).any(|w| w == "\u{4e16}".as_bytes()));
}

#[test]
fn requests_need_a_connection() {
    let r = rig();
    // before Init and before Connect: conn is NULL
    assert_eq!(r.gun.get_friends_list(ctx()), ZT_E_NOT_CONNECTED);
    assert_eq!(r.gun.join_room("a", "", ctx()), ZT_E_NOT_CONNECTED);
    assert_eq!(r.gun.call_get_status(1, ctx()), ZT_E_NOT_CONNECTED);
    assert_eq!(r.gun.leave_persona(ctx()), ZT_E_NOT_CONNECTED);
    assert_eq!(r.gun.get_view_list(ctx()), ZT_E_NOT_CONNECTED);
    // EnterPersona, ReleaseTicket, CheckTicket and PresentTicket test "initialised" first
    assert_eq!(r.gun.enter_persona("a", 1, ctx()), ZT_E_NOT_INITED);
    assert_eq!(r.gun.release_ticket(ctx()), ZT_E_NOT_INITED);
    assert_eq!(r.gun.check_ticket(&[1], ctx()), ZT_E_NOT_INITED);
    assert_eq!(r.gun.present_ticket(&[1], ctx()), ZT_E_NOT_INITED);
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    assert_eq!(r.gun.enter_persona("a", 1, ctx()), ZT_E_NOT_CONNECTED);
    assert_eq!(r.gun.present_ticket(&[1], ctx()), ZT_E_NOT_CONNECTED);
    assert_eq!(r.gun.boot_user("a", "b", ctx()), ZT_E_NOT_CONNECTED);
    assert_eq!(r.dp8.sent_count(), 0);
}

#[test]
fn a_failed_connect_leaves_a_connection_object_that_cannot_send() {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.dp8.0.borrow_mut().client_fails = true;
    assert_eq!(r.gun.connect("h", 1), ZT_E_FAILED_TO_CONNECT);
    let mut st = ConnectStatus::Created;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Failed);
    assert_eq!(r.gun.get_last_error(), ZT_E_FAILED_TO_CONNECT);
    // CGun+0x4c is set, conn+8 is not: the interfaces pass their test and Send fails
    assert_eq!(r.gun.friend_add("Bob", ctx()), ZT_E_NOT_CONNECTED);
}

#[test]
fn a_send_failure_is_not_connected_and_pending_is_success() {
    let r = connected();
    r.dp8.0.borrow_mut().send_result = Some(Hresult(0x8015_8650));
    assert_eq!(r.gun.friend_add("Bob", ctx()), ZT_E_NOT_CONNECTED);
    r.dp8.0.borrow_mut().send_result = Some(Hresult::S_OK);
    assert_eq!(r.gun.friend_add("Bob", ctx()), Hresult::S_OK);
}

#[test]
fn the_argument_checks_of_the_cgun_bodies() {
    let r = connected();
    let bad = Hresult(0x8007_0057);
    // CreateRoom: a zero limit
    assert_eq!(r.gun.create_room("R", "T", 0, 0, "", ctx()), bad);
    // BootUser: empty name, name over 50 characters
    assert_eq!(r.gun.boot_user("", "r", ctx()), bad);
    assert_eq!(
        r.gun.boot_user(&"x".repeat(51), "r", ctx()),
        ZT_E_STR_TOO_LONG
    );
    assert_eq!(r.gun.boot_user(&"x".repeat(50), "r", ctx()), Hresult::S_OK);
    // GetAccountName
    assert_eq!(r.gun.get_account_name("", ctx()), bad);
    assert_eq!(
        r.gun.get_account_name(&"x".repeat(51), ctx()),
        ZT_E_STR_TOO_LONG
    );
    // CallCreate: target <= 50, description <= 250, blob <= 5000, type rules
    let cc = |t: SupportCallType, a: &str, b: &str, blob: &[u8]| {
        r.gun.call_create(t, Some(a), Some(b), blob, 1, ctx())
    };
    assert_eq!(
        cc(SupportCallType::BUG, &"x".repeat(51), "", &[]),
        ZT_E_STR_TOO_LONG
    );
    assert_eq!(
        cc(SupportCallType::BUG, "", &"x".repeat(251), &[]),
        ZT_E_STR_TOO_LONG
    );
    assert_eq!(cc(SupportCallType::LAG, "", "", &vec![0; 5001]), bad);
    assert_eq!(
        cc(SupportCallType::LAG, "", "", &vec![0; 5000]),
        Hresult::S_OK
    );
    // 0x2000 without 0x4000; 0x8000 needs a description; 0x4000 needs a target; (t & 0xff) < 9
    assert_eq!(cc(SupportCallType(0x2000), "t", "d", &[]), bad);
    assert_eq!(cc(SupportCallType(0x8000), "t", "", &[]), bad);
    assert_eq!(cc(SupportCallType(0x4000), "", "d", &[]), bad);
    assert_eq!(cc(SupportCallType(9), "", "", &[]), bad);
    assert_eq!(
        cc(SupportCallType::HARASSMENT, "t", "d", &[]),
        Hresult::S_OK
    );
    // the checks come after the connection test and send nothing
    let before = r.dp8.sent_count();
    assert_eq!(cc(SupportCallType(9), "", "", &[]), bad);
    assert_eq!(r.dp8.sent_count(), before);
}

#[test]
fn unsupported_calls_answer_e_notimpl() {
    let r = connected();
    let notimpl = Hresult(0x8000_4001);
    assert_eq!(r.gun.change_password("a", "b", ctx()), notimpl);
    assert_eq!(
        r.gun.change_password_from_hint("a", "b", "c", ctx()),
        notimpl
    );
    assert_eq!(r.gun.get_password_hint("a", ctx()), notimpl);
    assert_eq!(r.gun.get_ticket_ex(ctx()), notimpl);
    assert_eq!(r.dp8.sent_count(), 0);
}

#[test]
fn host_calls_without_a_hosted_game() {
    let r = rig();
    assert_eq!(r.gun.create_game("g"), ZT_E_NOT_INITED);
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    assert_eq!(r.gun.end_game(), ZT_E_INVALID_OP);
    let mut n = 0;
    assert_eq!(r.gun.get_num_players(&mut n), ZT_E_NOT_INITED);
    assert_eq!(r.gun.add_player(1, "p"), ZT_E_NOT_INITED);
    assert_eq!(r.gun.create_game(""), Hresult(0x8007_0057));
}
