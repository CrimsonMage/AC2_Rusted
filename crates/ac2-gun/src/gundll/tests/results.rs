//! Parses of every result and event the client uses (inputs built from the `InitFromPacket` reads
//! and the templates of `0a_receive_messages.md` Part 2), delivered as the client's message types,
//! and the missing-attribute tolerance versus the strict readers.

use crate::gundll::client_msg::to_client;
use crate::gundll::msg::parse_message;
use crate::interfaces::{Context, TagBlob};
use crate::message::*;
use crate::zt::*;
use crate::{Hresult, MessageId};

fn parse(xml: &str) -> Result<Message, Hresult> {
    let mut b = xml.as_bytes().to_vec();
    b.push(0);
    parse_message(&b).map(|m| to_client(&m))
}

fn ok(xml: &str) -> Message {
    parse(xml).unwrap_or_else(|e| panic!("{xml}: {e:?}"))
}

fn header(ctx: u32, hr: u32) -> Option<ResultHeader> {
    Some(ResultHeader {
        context: Context(ctx),
        hr: Hresult(hr),
    })
}

#[test]
fn friends_list_result_with_rows() {
    let m = ok(
        r#"<GetFriendsListRes Cx="0x2f4a10" HR="0x0" Cnt="2" ><FriendStatus User="Bob@Shard" Mode="1" Location="Holtburg" /><FriendStatus User="Al" Mode="0" Location="" /></GetFriendsListRes> "#,
    );
    assert_eq!(m.id, MessageId(0x4200));
    assert!(m.inited);
    assert_eq!(m.result, header(0x2f4a10, 0));
    assert_eq!(
        m.body,
        MessageBody::GetFriendsList(vec![
            UserStatus {
                user_name: "Bob@Shard".into(),
                mode: 1,
                location: "Holtburg".into()
            },
            UserStatus {
                user_name: "Al".into(),
                mode: 0,
                location: "".into()
            },
        ])
    );
}

#[test]
fn a_row_missing_an_attribute_empties_the_list_and_cnt_is_never_read() {
    // The row reader answers 0xc1120007 and every caller tolerates exactly that code
    // (GetFriendsListRes Init 0x1001a2c6): the message is accepted. Retail then holds a NULL array and
    // a stale count; the port holds an empty list (a crash-path deviation).
    let m = ok(
        r#"<GetFriendsListRes Cx="0x1" HR="0x0" Cnt="1" ><FriendStatus User="a" Mode="1" /></GetFriendsListRes> "#,
    );
    assert_eq!(m.body, MessageBody::GetFriendsList(vec![]));
    // one bad row empties the whole list
    let m = ok(
        r#"<GetFriendsListRes Cx="0x1" HR="0x0" Cnt="2" ><FriendStatus User="a" Mode="1" Location="x" /><FriendStatus User="b" /></GetFriendsListRes> "#,
    );
    assert_eq!(m.body, MessageBody::GetFriendsList(vec![]));
    let m = ok(r#"<GetIgnoredListRes Cx="0x1" HR="0x0" Cnt="1" ><Jerk /></GetIgnoredListRes> "#);
    assert_eq!(m.body, MessageBody::GetIgnoredList(vec![]));
    let m = ok(r#"<FStart Cnt="1" ><FriendStatus Mode="1" /></FStart> "#);
    assert_eq!(m.body, MessageBody::FriendStart(vec![]));
    let m = ok(
        r#"<GetListOfRoomsRes Cx="0x1" HR="0x0" Idx="0" Total="1" Cnt="1" ><Room Room="R" /></GetListOfRoomsRes> "#,
    );
    assert!(
        matches!(m.body, MessageBody::GetListOfRooms { total: 1, ref rooms, .. } if rooms.is_empty())
    );
    for xml in [
        r#"<GetUsersInRoomRes Cx="0x1" HR="0x0" Cnt="1" ><User /></GetUsersInRoomRes> "#,
        r#"<GetSquelchListRes Cx="0x1" HR="0x0" Cnt="1" ><Jerk /></GetSquelchListRes> "#,
        r#"<FRListEv Uid="1" SubCount="0" Cnt="1" ><List /></FRListEv> "#,
        r#"<FRStatusEv Uid="1" Name="n" Status="0" Cnt="1" ><Targs /></FRStatusEv> "#,
        r#"<CtEnterEv Cnt="1" User="u" RoomId="1" ><Targs /></CtEnterEv> "#,
        r#"<CtLeaveEv Cnt="1" User="u" RoomId="1" ><Targs /></CtLeaveEv> "#,
    ] {
        ok(xml);
    }
    // only the missing-attribute code is tolerated: an invalid decimal in a row still fails the message
    assert_eq!(
        parse(r#"<GetFriendsListRes Cx="0x1" HR="0x0" Cnt="1" ><FriendStatus User="a" Mode="q" Location="" /></GetFriendsListRes> "#).unwrap_err(),
        ZT_E_INVALID_CHARACTER
    );
    // other child names are ignored; Cnt may lie
    let m =
        ok(r#"<GetFriendsListRes Cx="0x1" HR="0x0" Cnt="9" ><Other X="1" /></GetFriendsListRes> "#);
    assert_eq!(m.body, MessageBody::GetFriendsList(vec![]));
}

#[test]
fn header_attributes_are_required_and_lower_case() {
    assert_eq!(
        parse(r#"<FriendAddRes HR="0x0" />"#).unwrap_err(),
        ZT_E_ATTR_NOT_FOUND
    );
    assert_eq!(
        parse(r#"<FriendAddRes Cx="0x1" />"#).unwrap_err(),
        ZT_E_ATTR_NOT_FOUND
    );
    let m = ok(r#"<FriendAddRes Cx="0x1f" HR="0xc1121400" />"#);
    assert_eq!(m.result, header(0x1f, 0xc1121400));
    // 1.3 s1c: upper case digits give wrong values rather than errors ('A' - 0x57 = -22)
    let m = ok(r#"<FriendAddRes Cx="0x1" HR="0xc112140A" />"#);
    assert_ne!(m.result.unwrap().hr, Hresult(0xc112140a));
    assert_eq!(
        parse(r#"<FriendAddRes Cx="1g" HR="0x0" />"#).unwrap_err(),
        ZT_E_INVALID_CHARACTER
    );
}

#[test]
fn missing_string_and_decimal_attributes_are_tolerated() {
    let m = ok(r#"<WhereIsRes Cx="0x1" HR="0x0" />"#);
    assert_eq!(
        m.body,
        MessageBody::WhereIs {
            user_name: "".into(),
            mode: 0,
            location: "".into()
        }
    );
    // a bad decimal still fails
    assert_eq!(
        parse(r#"<WhereIsRes Cx="0x1" HR="0x0" Mode="x" />"#).unwrap_err(),
        ZT_E_INVALID_CHARACTER
    );
}

#[test]
fn flags_whereis_and_the_friend_events() {
    let m = ok(r#"<FlagsRes Cx="0x7" HR="0x0" WhichFlag="1" Flags="12" />"#);
    assert_eq!(
        m.body,
        MessageBody::Flags {
            which_flag: 1,
            flags: 12
        }
    );
    let m = ok(r#"<WhereIsRes Cx="0x7" HR="0x0" User="Bob" Mode="2" Location="Here" />"#);
    assert_eq!(
        m.body,
        MessageBody::WhereIs {
            user_name: "Bob".into(),
            mode: 2,
            location: "Here".into()
        }
    );
    let m = ok(r#"<FriendStat User="Bob@Shard" Mode="1" />"#);
    assert_eq!(m.id, MessageId(0x430d));
    assert_eq!(m.result, None);
    assert_eq!(
        m.body,
        MessageBody::FriendStatus {
            user_name: "Bob@Shard".into(),
            mode: 1
        }
    );
    let m = ok(r#"<FStart Cnt="1" ><FriendStatus User="a" Mode="3" Location="b" /></FStart> "#);
    assert_eq!(
        m.body,
        MessageBody::FriendStart(vec![UserStatus {
            user_name: "a".into(),
            mode: 3,
            location: "b".into()
        }])
    );
    // FriendRequest and FriendAddComplete carry no member the client reads
    assert_eq!(ok(r#"<FriendRequest User="x" />"#).body, MessageBody::None);
    assert_eq!(
        ok(r#"<FriendAddComplete Status="0" User="x" />"#).body,
        MessageBody::None
    );
}

#[test]
fn ignored_list_rows_are_jerk_elements() {
    let m = ok(
        r#"<GetIgnoredListRes Cx="0x1" HR="0x0" Cnt="2" ><Jerk Str="a" /><Jerk Str="b&amp;c" /></GetIgnoredListRes> "#,
    );
    assert_eq!(
        m.body,
        MessageBody::GetIgnoredList(vec!["a".into(), "b&c".into()])
    );
}

#[test]
fn chat_results() {
    let m = ok(r#"<JoinRoomRes Cx="0x9" HR="0x0" RoomId="12" SId="3" />"#);
    assert_eq!(
        m.body,
        MessageBody::JoinRoom {
            room_id: 12,
            server_id: 3
        }
    );
    let m = ok(r#"<CreateRoomRes Cx="0x9" HR="0x0" RoomId="13" SId="4" />"#);
    assert_eq!(
        m.body,
        MessageBody::CreateRoom {
            room_id: 13,
            server_id: 4
        }
    );
    let m = ok(
        r#"<GetListOfRoomsRes Cx="0x9" HR="0x0" Idx="0" Total="1" Cnt="1" ><Room Room="R" Topic="T" Mode="2" Cnt="5" Limit="50" /></GetListOfRoomsRes> "#,
    );
    assert_eq!(
        m.body,
        MessageBody::GetListOfRooms {
            index: 0,
            total: 1,
            rooms: vec![ChatRoomAttrs {
                name: "R".into(),
                topic: "T".into(),
                mode: 2,
                user_count: 5,
                user_limit: 50
            }],
        }
    );
    // results with only Cx and HR
    for xml in [
        r#"<LeaveRoomRes Cx="0x1" HR="0x0" />"#,
        r#"<SendToRoomRes Cx="0x1" HR="0x0" />"#,
        r#"<SendToUserExRes Cx="0x1" HR="0x0" />"#,
        r#"<SendToRoomExRes Cx="0x1" HR="0x0" />"#,
        r#"<CreatePChannelRes Cx="0x1" HR="0x0" />"#,
        r#"<DeletePChannelR Cx="0x1" HR="0x0" />"#,
        r#"<SayRes Cx="0x1" HR="0x0" />"#,
    ] {
        assert_eq!(ok(xml).body, MessageBody::None, "{xml}");
    }
}

#[test]
fn chat_events_need_their_extra_child() {
    let m = ok(r#"<UTellEv User="Bob@Shard" Type="0" Text="hi" ><Extra Val="ba"/></UTellEv> "#);
    assert_eq!(m.id, MessageId(0x5317));
    assert_eq!(
        m.body,
        MessageBody::UserSpeak {
            user_name: "Bob@Shard".into(),
            kind: 0,
            text: "hi".into(),
            extra: Some(TagBlob {
                cb_size: 1,
                data: Some(vec![1])
            }),
        }
    );
    // an empty Val is a zero-length blob; a missing <Extra/> fails the message (0xc1121c04)
    let m = ok(r#"<UTellEv User="u" Type="0" Text="t" ><Extra Val=""/></UTellEv> "#);
    assert!(matches!(
        m.body,
        MessageBody::UserSpeak {
            extra: Some(TagBlob { cb_size: 0, .. }),
            ..
        }
    ));
    assert_eq!(
        parse(r#"<UTellEv User="u" Type="0" Text="t" />"#).unwrap_err(),
        ZT_E_NOT_FOUND
    );
    let m = ok(r#"<RTellEv RoomId="4" User="u" Type="1" Text="t" ><Extra Val=""/></RTellEv> "#);
    assert!(matches!(
        m.body,
        MessageBody::RoomSpeak {
            room_id: 4,
            kind: 1,
            ..
        }
    ));
    assert_eq!(
        ok(r#"<EnterEv RoomId="4" User="u" />"#).body,
        MessageBody::RoomEnter {
            room_id: 4,
            user_name: "u".into()
        }
    );
    assert_eq!(
        ok(r#"<LeaveEv RoomId="4" User="u" />"#).body,
        MessageBody::RoomLeave {
            room_id: 4,
            user_name: "u".into()
        }
    );
    assert_eq!(
        ok(r#"<KickedOutEv RoomId="4" Reason="bad" />"#).body,
        MessageBody::KickedOut {
            room_id: 4,
            reason: "bad".into()
        }
    );
    assert_eq!(
        ok(r#"<ReceiveOwnershipEv RoomId="4" />"#).body,
        MessageBody::ReceiveOwnership { room_id: 4 }
    );
    assert_eq!(
        ok(r#"<InviteEv User="u" Room="r" Invitation="come" />"#).body,
        MessageBody::Invited {
            user_name: "u".into(),
            room_name: "r".into(),
            invitation: "come".into()
        }
    );
    // SayEv's attribute is the single letter R
    assert_eq!(
        ok(r#"<SayEv User="u" Text="t" R="1" />"#).body,
        MessageBody::None
    );
}

/// Parses a class the DLL raises only locally (no factory case): the body reads are still the
/// class's `InitFromPacket`.
fn local(xml: &str) -> Message {
    use crate::gundll::catalog::by_element;
    use crate::gundll::msg::GunMsg;
    use crate::gundll::xml_parse::XmlPacket;
    let w: Vec<u16> = xml.encode_utf16().collect();
    let pkt = XmlPacket::parse(&w).unwrap();
    let def = by_element(pkt.name(None)).unwrap();
    to_client(&GunMsg::init_from_packet(def, &pkt).unwrap())
}

#[test]
fn the_five_local_only_classes_are_rejected_from_the_wire() {
    // the DLL factory FUN_10004d80 has no case for these ids: 0x8007000E (final messages audit N3)
    for xml in [
        r#"<ConnectEv />"#,
        r#"<DisconnectEv />"#,
        r#"<InitRes Cx="0x1" HR="0x0" />"#,
        r#"<GetTicketRes Cx="0x5" HR="0x0" ><Ticket Val="ba"/></GetTicketRes> "#,
        r#"<GetNewsRes Cx="0x1" HR="0x0" News="x" />"#,
    ] {
        assert_eq!(parse(xml).unwrap_err(), Hresult(0x8007_000E), "{xml}");
    }
    // the classes stay: their bodies still parse locally
    assert_eq!(
        local(r#"<InitRes Cx="0x1" HR="0x0" />"#).id,
        MessageId(0x1204)
    );
}

#[test]
fn ticket_support_and_persona_results() {
    let m = local(r#"<GetTicketRes Cx="0x5" HR="0x0" ><Ticket Val="ba"/></GetTicketRes> "#);
    assert_eq!(m.id, MessageId(0xf203));
    assert_eq!(
        m.body,
        MessageBody::GetTicket(Some(TagBlob {
            cb_size: 1,
            data: Some(vec![1])
        }))
    );
    assert_eq!(
        ok(r#"<PresentTicketRes Cx="0x1" HR="0x0" Status="0" />"#).id,
        MessageId(0xf201)
    );
    assert_eq!(
        ok(r#"<CheckTicketRes Cx="0x1" HR="0x0" />"#).id,
        MessageId(0xf200)
    );
    assert_eq!(
        ok(r#"<ReleaseTicketRes Cx="0x1" HR="0xc112080b" />"#).result,
        header(1, 0xc112080b)
    );
    assert_eq!(
        ok(r#"<SetGameServerRes Cx="0x1" HR="0x0" />"#).id,
        MessageId(0xf205)
    );
    let m = ok(r#"<CallCreateRes Cx="0x1" HR="0x0" QueueID="77" QueuePos="3" />"#);
    assert_eq!(
        m.body,
        MessageBody::CallCreate {
            queue_id: 77,
            queue_pos: 3
        }
    );
    assert_eq!(
        ok(r#"<CallGetStatusRes Cx="0x1" HR="0x0" QueuePos="2" />"#).id,
        MessageId(0x10201)
    );
    // personas (extractor-table rows, not independently verified)
    let m = ok(r#"<EnterPersonaRes Cx="0x1" HR="0x0" Type="1" SId="9" />"#);
    assert_eq!(
        m.body,
        MessageBody::EnterPersona {
            kind: 1,
            server_id: 9
        }
    );
    let m = ok(
        r#"<GetPersonaListRes Cx="0x1" HR="0x0" Cnt="2" ><Personas Str="A" /><Personas Str="B" /></GetPersonaListRes> "#,
    );
    assert_eq!(
        m.body,
        MessageBody::GetPersonaList(vec!["A".into(), "B".into()])
    );
    assert_eq!(
        ok(r#"<LeavePersonaRes Cx="0x1" HR="0x0" />"#).id,
        MessageId(0x8201)
    );
}

#[test]
fn unknown_elements_and_open_classes_are_dropped() {
    assert_eq!(
        parse(r#"<NoSuchThing Cx="0x1" />"#).unwrap_err(),
        ZT_E_MISSING_OBJECT
    );
    // the connect-failed event and LoginRes are not in the 216-entry table: they cannot be parsed
    assert_eq!(
        parse(r#"<ConnectFailedEv />"#).unwrap_err(),
        ZT_E_MISSING_OBJECT
    );
    assert_eq!(
        parse(r#"<LoginRes Cx="0x1" HR="0x0" Status="1" />"#).unwrap_err(),
        ZT_E_MISSING_OBJECT
    );
    // table ids that have no class in GunDll
    for xml in [
        r#"<GetNews Cx="0x1" />"#,
        r#"<NewsEv />"#,
        r#"<GetIdFromName Cx="0x1" />"#,
    ] {
        assert_eq!(parse(xml).unwrap_err(), Hresult(0x8007_000E), "{xml}");
    }
}

#[test]
fn text_must_be_nul_terminated_utf8() {
    assert_eq!(
        parse_message(b"<FriendAddRes Cx=\"0x1\" HR=\"0x0\" />").unwrap_err(),
        Hresult(0x8007_0057)
    );
    assert_eq!(parse_message(b"").unwrap_err(), Hresult(0x8007_0057));
    // UTF-8 attribute values
    let m = ok("<FriendStat User=\"B\u{e9}b\" Mode=\"1\" />");
    assert_eq!(
        m.body,
        MessageBody::FriendStatus {
            user_name: "B\u{e9}b".into(),
            mode: 1
        }
    );
}

/// The Init read order (disassembly): which error comes first can differ from the render order.
#[test]
fn the_init_read_order_is_retails() {
    // CallCreate reads CallType, TargetPersonaName, Description, CallData (child), LangID: with both
    // a missing CallData child and a bad LangID, the child's 0xc1121c04 comes first
    assert_eq!(
        parse(r#"<CallCreate Cx="0x1" CallType="1" LangID="x" />"#).unwrap_err(),
        ZT_E_NOT_FOUND
    );
    // CallType is read first
    assert_eq!(
        parse(r#"<CallCreate Cx="0x1" CallType="z" LangID="x" />"#).unwrap_err(),
        ZT_E_INVALID_CHARACTER
    );
    // ConInfo: Locale first, then CdKey
    assert_eq!(
        parse(r#"<ConInfo Cx="0x1" Locale="q" ><Random Val=""/></ConInfo> "#).unwrap_err(),
        ZT_E_INVALID_CHARACTER
    );
    assert_eq!(
        parse(r#"<ConInfo Cx="0x1" Locale="1" ><Random Val=""/></ConInfo> "#).unwrap_err(),
        ZT_E_NOT_FOUND
    );
    // CtEnterEv: the Targs list (strict decimals do not exist there) before User and RoomId
    assert_eq!(
        parse(r#"<CtEnterEv Cnt="0" User="u" RoomId="x" ></CtEnterEv> "#).unwrap_err(),
        ZT_E_INVALID_CHARACTER
    );
}

#[test]
fn friends_fstart_and_room_rows_sit_at_their_child_index_and_count_only_matches() {
    // FUN_10036df0 (observed in the DLL): `Other, FriendStatus u1, FriendStatus u2` is count 2 with
    // the rows [blank, u1]; the last row is lost. GetListOfRoomsRes behaves the same.
    use crate::gundll::msg::Value;
    let msg = |xml: &str| {
        let mut b = xml.as_bytes().to_vec();
        b.push(0);
        parse_message(&b).unwrap()
    };
    let fs = |n: &str, m: i32, l: &str| {
        vec![
            Value::Str(Some(n.into())),
            Value::Dec(m),
            Value::Str(Some(l.into())),
        ]
    };
    let blank = vec![Value::Str(None), Value::Dec(0), Value::Str(None)];
    let rows = |x: &str| msg(x).rows_field("FriendStatus").to_vec();
    let a = r#"<FriendStatus User="u1" Mode="1" Location="l1" />"#;
    let b = r#"<FriendStatus User="u2" Mode="2" Location="l2" />"#;
    let list = |kids: &str| {
        format!(r#"<GetFriendsListRes Cx="0x1" HR="0x0" Cnt="2" >{kids}</GetFriendsListRes> "#)
    };
    assert_eq!(
        rows(&list(&format!("<Other/>{a}{b}"))),
        vec![blank.clone(), fs("u1", 1, "l1")]
    );
    // rows first, a stray child last: the rows stay in front, the count is 2 of 3 slots
    assert_eq!(
        rows(&list(&format!("{a}{b}<Other/>"))),
        vec![fs("u1", 1, "l1"), fs("u2", 2, "l2")]
    );
    assert_eq!(
        rows(&list(&format!("{a}<Other/>{b}"))),
        vec![fs("u1", 1, "l1"), blank.clone()]
    );
    let start = format!(r#"<FStart Cnt="2" ><Other/>{a}{b}</FStart> "#);
    assert_eq!(rows(&start), vec![blank, fs("u1", 1, "l1")]);
    // GetListOfRoomsRes: 20-byte rows
    let room = |n: &str| format!(r#"<Room Room="{n}" Topic="t" Mode="1" Cnt="2" Limit="3" />"#);
    let m = msg(&format!(
        r#"<GetListOfRoomsRes Cx="0x1" HR="0x0" Idx="0" Total="2" Cnt="2" ><Other/>{}{}</GetListOfRoomsRes> "#,
        room("r1"),
        room("r2")
    ));
    let rooms = m.rows_field("Room");
    assert_eq!(rooms.len(), 2);
    assert_eq!(rooms[0][0], Value::Str(None));
    assert_eq!(rooms[1][0], Value::Str(Some("r1".into())));
    // the other row classes compact
    let m = msg(
        r#"<GetIgnoredListRes Cx="0x1" HR="0x0" Cnt="2" ><Other/><Jerk Str="a" /><Jerk Str="b" /></GetIgnoredListRes> "#,
    );
    assert_eq!(m.rows_field("Jerk").len(), 2);
    assert_eq!(m.rows_field("Jerk")[0][0], Value::Str(Some("a".into())));
}
