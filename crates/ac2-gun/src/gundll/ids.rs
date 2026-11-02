//! GunDll message ids (`gun_messages.csv`, `0a_receive_messages.md` Part 2) that the port names.
//! The PDB's own names for the ones the client reads are in [`crate::message_id`].

pub const CONNECT: u32 = 0x01100;
pub const CON_INFO: u32 = 0x01103;
pub const CONNECT_RES: u32 = 0x01200;
pub const CON_INFO_RES: u32 = 0x01203;
pub const INIT_RES: u32 = 0x01204;
pub const CONNECT_EV: u32 = 0x01300;
pub const DISCONNECT_EV: u32 = 0x01301;
pub const CONNECT_FAILED_EV: u32 = 0x01302;
pub const HOST_DATA_RES: u32 = 0x07200;
pub const GET_FRIENDS_LIST_RES: u32 = 0x04200;
pub const GET_IGNORED_LIST_RES: u32 = 0x04203;
pub const FLAGS_RES: u32 = 0x04208;
pub const WHERE_IS_RES: u32 = 0x04209;
pub const F_START: u32 = 0x0430c;
pub const FRIEND_STAT: u32 = 0x0430d;
pub const GET_LIST_OF_ROOMS_RES: u32 = 0x05200;
pub const JOIN_ROOM_RES: u32 = 0x05201;
pub const CREATE_ROOM_RES: u32 = 0x0520a;
pub const ENTER_EV: u32 = 0x05314;
pub const LEAVE_EV: u32 = 0x05315;
pub const R_TELL_EV: u32 = 0x05316;
pub const U_TELL_EV: u32 = 0x05317;
pub const KICKED_OUT_EV: u32 = 0x05318;
pub const RECEIVE_OWNERSHIP_EV: u32 = 0x05319;
pub const INVITE_EV: u32 = 0x0531a;
pub const GET_TICKET_RES: u32 = 0x0f203;
pub const CALL_CREATE_RES: u32 = 0x10200;
pub const ENTER_PERSONA_RES: u32 = 0x08200;
pub const GET_PERSONA_LIST_RES: u32 = 0x08204;
