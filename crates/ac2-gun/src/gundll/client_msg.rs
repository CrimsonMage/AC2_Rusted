//! From GunDll's message object to what `CGun::DispatchQueue` hands the client: a [`Message`] whose
//! body carries the members the client's `Gun::C*Result` / `Gun::C*Event` getters return (the PDB
//! layouts in [`crate::message`]). A NULL string member (a missing attribute) reads as an empty
//! string here; retail hands the client a NULL `wchar_t*`.

use super::msg::{GunMsg, Value};
use crate::interfaces::{Context, TagBlob};
use crate::message::{ChatRoomAttrs, Message, MessageBody, ResultHeader, UserStatus};
use crate::{Hresult, MessageId};

fn s(m: &GunMsg, name: &str) -> String {
    m.str_field(name).unwrap_or("").to_owned()
}

fn d(m: &GunMsg, name: &str) -> u32 {
    m.dec_field(name) as u32
}

fn tag_blob(b: Option<&[u8]>) -> Option<TagBlob> {
    b.map(|bytes| TagBlob {
        cb_size: bytes.len() as u32,
        data: Some(bytes.to_vec()),
    })
}

fn row_str(v: &Value) -> String {
    match v {
        Value::Str(Some(s)) => s.clone(),
        _ => String::new(),
    }
}

fn row_dec(v: &Value) -> u32 {
    match v {
        Value::Dec(d) => *d as u32,
        _ => 0,
    }
}

fn user_statuses(m: &GunMsg) -> Vec<UserStatus> {
    m.rows_field("FriendStatus")
        .iter()
        .map(|r| UserStatus {
            user_name: row_str(&r[0]),
            mode: row_dec(&r[1]),
            location: row_str(&r[2]),
        })
        .collect()
}

fn str_rows(m: &GunMsg, row: &str) -> Vec<String> {
    m.rows_field(row).iter().map(|r| row_str(&r[0])).collect()
}

/// The client-facing message for a parsed (or locally raised) message object.
pub fn to_client(m: &GunMsg) -> Message {
    use super::ids::*;
    let result = (m.def.header == super::catalog::Header::Response).then_some(ResultHeader {
        context: Context(m.cx),
        hr: Hresult(m.hr),
    });
    let body = match m.def.id {
        // Gun::CGetFriendsListResult: m_Count statuses at m_AllFriendsStatus
        GET_FRIENDS_LIST_RES => MessageBody::GetFriendsList(user_statuses(m)),
        // Gun::CGetIgnoredListResult
        GET_IGNORED_LIST_RES => MessageBody::GetIgnoredList(str_rows(m, "Jerk")),
        // Gun::CFriendStartEvent
        F_START => MessageBody::FriendStart(user_statuses(m)),
        // Gun::CFriendStatusEvent
        FRIEND_STAT => MessageBody::FriendStatus {
            user_name: s(m, "User"),
            mode: d(m, "Mode"),
        },
        // Gun::CFlagsResult
        FLAGS_RES => MessageBody::Flags {
            which_flag: d(m, "WhichFlag"),
            flags: d(m, "Flags"),
        },
        // Gun::CWhereIsResult
        WHERE_IS_RES => MessageBody::WhereIs {
            user_name: s(m, "User"),
            mode: d(m, "Mode"),
            location: s(m, "Location"),
        },
        // Gun::CJoinRoomResult / Gun::CCreateRoomResult: m_RoomId, m_ServerId (attribute SId)
        JOIN_ROOM_RES => MessageBody::JoinRoom {
            room_id: d(m, "RoomId"),
            server_id: d(m, "SId"),
        },
        CREATE_ROOM_RES => MessageBody::CreateRoom {
            room_id: d(m, "RoomId"),
            server_id: d(m, "SId"),
        },
        // Gun::CGetListOfRoomsResult: m_Index, m_Total, the rooms
        GET_LIST_OF_ROOMS_RES => MessageBody::GetListOfRooms {
            index: d(m, "Idx"),
            total: d(m, "Total"),
            rooms: m
                .rows_field("Room")
                .iter()
                .map(|r| ChatRoomAttrs {
                    name: row_str(&r[0]),
                    topic: row_str(&r[1]),
                    mode: row_dec(&r[2]),
                    user_count: row_dec(&r[3]),
                    user_limit: row_dec(&r[4]),
                })
                .collect(),
        },
        // Gun::CCallCreateResult
        CALL_CREATE_RES => MessageBody::CallCreate {
            queue_id: d(m, "QueueID"),
            queue_pos: d(m, "QueuePos"),
        },
        // Gun::CGetTicketResult: m_Ticket
        GET_TICKET_RES => MessageBody::GetTicket(tag_blob(m.blob_field("Ticket"))),
        // Gun::CEnterPersonaResult: m_Type, m_ServerId (attribute SId)
        ENTER_PERSONA_RES => MessageBody::EnterPersona {
            kind: d(m, "Type"),
            server_id: d(m, "SId"),
        },
        // Gun::CGetPersonaListResult
        GET_PERSONA_LIST_RES => MessageBody::GetPersonaList(str_rows(m, "Personas")),
        ENTER_EV => MessageBody::RoomEnter {
            room_id: d(m, "RoomId"),
            user_name: s(m, "User"),
        },
        LEAVE_EV => MessageBody::RoomLeave {
            room_id: d(m, "RoomId"),
            user_name: s(m, "User"),
        },
        R_TELL_EV => MessageBody::RoomSpeak {
            room_id: d(m, "RoomId"),
            user_name: s(m, "User"),
            kind: d(m, "Type"),
            text: s(m, "Text"),
            extra: tag_blob(m.blob_field("Extra")),
        },
        U_TELL_EV => MessageBody::UserSpeak {
            user_name: s(m, "User"),
            kind: d(m, "Type"),
            text: s(m, "Text"),
            extra: tag_blob(m.blob_field("Extra")),
        },
        KICKED_OUT_EV => MessageBody::KickedOut {
            room_id: d(m, "RoomId"),
            reason: s(m, "Reason"),
        },
        RECEIVE_OWNERSHIP_EV => MessageBody::ReceiveOwnership {
            room_id: d(m, "RoomId"),
        },
        INVITE_EV => MessageBody::Invited {
            user_name: s(m, "User"),
            room_name: s(m, "Room"),
            invitation: s(m, "Invitation"),
        },
        _ => MessageBody::None,
    };
    Message {
        inited: true,
        id: MessageId(m.def.id),
        result,
        body,
    }
}
