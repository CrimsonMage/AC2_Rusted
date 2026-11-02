//! What the GUN classes raise. Retail's handlers call `CM_UI::SendNotice_Gun*` and
//! `gmCWInterface::Result_*` / `GUNEvent_*` (each runs the player's script IFun) from inside
//! `DispatchQueue`; the crate boundary puts the script VM after this one, so the calls are
//! recorded here in order and the program carries them out right after `CheckForMessages`
//! returns (deviation, listed in AGENTS.md).

use crate::friends::FriendInfo;

/// The GUN `CM_UI` notices (`CM_UI::SendNotice_Gun*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GunNotice {
    /// 0x469 `SendNotice_GunConnected` (admin 0x00723650, Legions 0x006A3790): no handler.
    Connected,
    /// 0x46A `SendNotice_GunConnectionLost` (admin 0x007236E0, Legions 0x006A37E0):
    /// `gmUIFlow::RecvNotice_GunConnectionLost`.
    ConnectionLost,
    /// 0x46B `SendNotice_GunReconnected` (admin 0x00723770, Legions 0x006A3830):
    /// `gmClient::RecvNotice_GunReconnected`.
    Reconnected,
    /// 0x46C `SendNotice_GunAuthenticated(bool)` (admin 0x007235C0, Legions 0x006A3740):
    /// `gmClient::RecvNotice_GunAuthenticated`.
    Authenticated(bool),
}

/// A call into the player's script or the UI that a GUN class made. The `gmCWInterface` entry
/// points run their IFun only when the interface has a process (`vtbl[+0x10]()` non-NULL,
/// except `GUNHandleReauthentication`, which has no check); the program that carries these
/// out skips them when the player's script is not running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GunOutput {
    /// A `CM_UI` notice that reached no gmClient handler: the program shows `ConnectionLost`.
    Notice(GunNotice),
    /// `gmCWInterface::Result_SendToUser` (admin 0x00450D10, Legions 0x00451E50) ->
    /// `CallIFun_Result_SendToUser` (IFun 0xB4003D).
    ResultSendToUser {
        user: String,
        message: String,
        result: u32,
    },
    /// `Result_SendToRoom` (admin 0x00450D40, Legions 0x00451E80), IFun 0xB4003E.
    ResultSendToRoom {
        message: String,
        room: u32,
        result: u32,
    },
    /// `Result_FriendAdd` (admin 0x00450D70, Legions 0x00451EB0), IFun 0xB40040.
    ResultFriendAdd { result: u32, name: String },
    /// `Result_FriendDelete` (admin 0x00450D90, Legions 0x00451ED0), IFun 0xB40041.
    ResultFriendDelete { result: u32, name: String },
    /// `Result_IgnoredAdd` (admin 0x00450DE0, Legions 0x00451F20), IFun 0xB40045.
    ResultIgnoredAdd { result: u32, name: String },
    /// `Result_IgnoredDelete` (admin 0x00450E00, Legions 0x00451F40), IFun 0xB40046.
    ResultIgnoredDelete { result: u32, name: String },
    /// `Result_CallCreate` (admin 0x00450E20, Legions 0x00451F60), IFun 0xB40048.
    ResultCallCreate {
        result: u32,
        call_type: u32,
        target: String,
        description: String,
        queue_id: u32,
        queue_pos: u32,
    },
    /// `Result_GetFriendsList` (admin 0x00453B90, Legions 0x00454C10), IFun 0xB40042.
    ResultGetFriendsList {
        result: u32,
        user_requested: u32,
        list: Vec<FriendInfo>,
    },
    /// `Result_GetIgnoredList` (admin 0x00453C10, Legions 0x00454C90), IFun 0xB40047.
    ResultGetIgnoredList {
        result: u32,
        user_requested: u32,
        list: Vec<String>,
    },
    /// `GUNEvent_FriendStatus` (admin 0x00450DB0, Legions 0x00451EF0), IFun 0xB40044.
    EventFriendStatus(FriendInfo),
    /// `GUNEvent_FriendStart` (admin 0x00453BD0, Legions 0x00454C50), IFun 0xB40043.
    EventFriendStart(Vec<FriendInfo>),
    /// `GUNEvent_HearFromRoom` (admin 0x00453320, Legions 0x00454430), IFun 0xB4003F: the speaker
    /// with `[metaLetters]` appended (a `StringInfo` literal, escape 0), the message as it came
    /// (a raw `PStringBase`), the room and the unpacked weenie chat flags (0 if the blob did not
    /// unpack).
    EventHearFromRoom {
        speaker: String,
        message: String,
        room: u32,
        flags: u32,
    },
    /// `HearTell` (admin 0x004530F0, Legions 0x00454200), IFun 0xB4003C: the teller with
    /// `[metaLetters]` appended (`StringInfo` literal, escape 0), the message (literal, escape 1)
    /// and the flags; `INVALID_IID` is the context.
    HearTell {
        teller: String,
        message: String,
        flags: u32,
    },
    /// `GUNHandleReauthentication` (admin 0x00450D00, Legions 0x00451E40), IFun 0xB400D8: no
    /// process check.
    HandleReauthentication,
    /// A `StringInfo` the gmClient shows through the UI sink (`this+0x64` virtual `+0x38`):
    /// the string `name` of the table enum `table` (`ID_GunReAuthenticationFailed` /
    /// `ID_GunReAuthenticationSuccess` in 0x4100000E).
    UiString { name: &'static str, table: u32 },
    /// `NetError::GetLogString` of a failed `AuthenticateWithGUN(true)` shown through the same
    /// sink (`gmClient::RecvNotice_GunReconnected`).
    UiNetError(super::client::GunError),
}
