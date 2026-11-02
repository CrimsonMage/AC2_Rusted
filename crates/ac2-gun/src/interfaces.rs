//! The `IGun*` interfaces as the client PDB lays them out (`decompiled-pdb/gundll/gun_interfaces.csv`:
//! interface, IID, slot, method, PDB prototype, GunDll address). Methods are declared in slot
//! order; slots 0-2 are `IUnknown`'s and have no counterpart here. GunDll's one COM object
//! (`CGun`) answers all ten interfaces the client queries (`TurbineGun::Init` fails unless every
//! one is there): [`GunObject`] is that object.
//!
//! Types: `const ushort*` (UTF-16) and `const char*` are `&str` (`TurbineGun` passes
//! `PStringBase::m_data`); a `void*` request context is [`Context`]; an input `tagBLOB*` is a byte
//! slice (`cbSize` bytes at `pBlobData`), the in/out one of `GetTicket` a [`TagBlob`]; out pointers
//! are `&mut`. Parameter names are `NSTurbineGun::TurbineGun`'s PDB names for the arguments it
//! passes through, where it calls the method; elsewhere they are neutral (`a`, `b`, ...), as the
//! PDB's interface prototypes carry no names.

use crate::message::GunQueueMessage;
use crate::Hresult;

/// A request's `void*` context: the caller's object that the result's `m_Context` carries back.
/// GunDll writes it into the request as `Cx="0x%x"` and keeps no pending-request table, so the
/// server's result must carry it back (`0a_receive_messages.md`, verified); 0 is NULL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Context(pub u32);

/// `tagBLOB`: `cbSize`, then `pBlobData` (NULL as `None`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TagBlob {
    pub cb_size: u32,
    pub data: Option<Vec<u8>>,
}

impl TagBlob {
    /// The `cbSize` bytes at `pBlobData` (empty when it is NULL). A blob whose buffer is shorter
    /// than `cbSize` (retail would read past it) gives the bytes there are.
    pub fn bytes(&self) -> &[u8] {
        match &self.data {
            Some(data) => &data[..data.len().min(self.cb_size as usize)],
            None => &[],
        }
    }
}

/// A COM `GUID` (`_GUID`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

/// `IGunConnection::ConnectStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ConnectStatus {
    Created = 0,
    Connecting = 1,
    Connected = 2,
    Disconnected = 3,
    Failed = 4,
}

/// `IGunFriends::FLAG_OPERATION`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FlagOperation {
    Get = 0,
    Set = 1,
    Clear = 2,
}

/// `IGunFriends::FLAG_WORD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FlagWord {
    Control = 0,
    State = 1,
}

/// `SUPPORT_CALLTYPE`: the PDB's values, except `NUM_CALLTYPES` = 9, which is left out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SupportCallType(pub u32);

impl SupportCallType {
    pub const LAG: Self = Self(4);
    pub const MACRO: Self = Self(16390);
    pub const SPAM: Self = Self(24578);
    pub const STUCK: Self = Self(32771);
    pub const BUG: Self = Self(32773);
    pub const OTHER: Self = Self(32776);
    pub const ABUSE: Self = Self(57344);
    pub const HARASSMENT: Self = Self(57345);
    pub const OTHER_COC: Self = Self(57351);
}

/// `ACCOUNTVALUE_TYPE`: the PDB has only a forward reference, no values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountValueType(pub u32);

/// `Gun::IGUNQueueRead`: the sink `IGunConnection::DispatchQueue` hands each queued message to
/// (`TurbineGun::Read`). A negative result stops the dispatch.
///
/// COM methods are calls on a shared `this`: every `IGun*` method takes `&self` and an
/// implementation uses interior mutability. The sink runs inside `DispatchQueue` and calls back
/// into the same GunDll object (see [`IGunConnection::dispatch_queue`]).
pub trait IGunQueueRead {
    fn read(&self, message: &GunQueueMessage) -> Hresult;
}

/// `IGunConnection` (IID eeeeee21-85ea-40f7-8157-cd07a138b49f).
pub trait IGunConnection {
    /// Slot 3, GunDll 0x1000fab0: `HRESULT Init(const _GUID&, const ushort*, ulong, const ushort*, ushort)`.
    /// (`TurbineGun::RequestConnect` passes its `gameVersion`, `locale`, `cdKey`, `portLocal`.)
    fn init(
        &self,
        a: &Guid,
        game_version: &str,
        locale: u32,
        cd_key: &str,
        port_local: u16,
    ) -> Hresult;
    /// Slot 4, GunDll 0x1000fbc0: `HRESULT Connect(const char*, ushort)`.
    fn connect(&self, gun_server_name: &str, port: u16) -> Hresult;
    /// Slot 5, GunDll 0x1000fce0: `HRESULT Disconnect()`.
    fn disconnect(&self) -> Hresult;
    /// Slot 6, GunDll 0x1000fd30: `HRESULT GetStatus(IGunConnection::ConnectStatus*)`.
    fn get_status(&self, a: &mut ConnectStatus) -> Hresult;
    /// Slot 7, GunDll 0x1000fd60: `HRESULT GetLastError()`.
    fn get_last_error(&self) -> Hresult;
    /// Slot 8, GunDll 0x1000fd90: `HRESULT DispatchQueue(Gun::IGUNQueueRead*)`.
    ///
    /// The sink re-enters this object, exactly as retail's `CGun` was re-entered: from `Read`
    /// the client calls `Connect` (`RcvConnectEvent`'s reconnect on 0x1301/0x1302) and, on
    /// RECONNECTED, `GetTicket` / `PresentTicket` (`RecvNotice_GunReconnected` ->
    /// `AuthenticateWithGUN(true)`). An implementation must therefore NOT hold its internal
    /// borrow or lock while it calls the sink: pop one message, release, call the sink, repeat.
    /// An implementation may also be dropped-from or called while a sink call is on the stack.
    fn dispatch_queue(&self, sink: &dyn IGunQueueRead) -> Hresult;
}

/// `IGunNews` (IID eeeeee22-85ea-40f7-8157-cd07a138b49f).
pub trait IGunNews {
    /// Slot 3, GunDll 0x1000f8c0: `HRESULT GetNews(const char*, const char*, ushort, void*)`.
    fn get_news(&self, a: &str, b: &str, c: u16, context: Context) -> Hresult;
}

/// `IGunAccount` (IID eeeeee23-85ea-40f7-8157-cd07a138b49f).
pub trait IGunAccount {
    /// Slot 3, GunDll 0x1000f170: `HRESULT CreateAccount(const ushort*, const ushort*, const ushort*, const ushort*, void*)`.
    fn create_account(&self, a: &str, b: &str, c: &str, d: &str, context: Context) -> Hresult;
    /// Slot 4, GunDll 0x1000f250: `HRESULT ChangePassword(const ushort*, const ushort*, void*)`.
    fn change_password(&self, a: &str, b: &str, context: Context) -> Hresult;
    /// Slot 5, GunDll 0x1000f260: `HRESULT ChangePasswordFromHint(const ushort*, const ushort*, const ushort*, void*)`.
    fn change_password_from_hint(&self, a: &str, b: &str, c: &str, context: Context) -> Hresult;
    /// Slot 6, GunDll 0x1000f270: `HRESULT GetAccountStatus(const ushort*, void*)`.
    fn get_account_status(&self, a: &str, context: Context) -> Hresult;
    /// Slot 7, GunDll 0x1000f310: `HRESULT GetPasswordHint(const ushort*, void*)`.
    fn get_password_hint(&self, a: &str, context: Context) -> Hresult;
    /// Slot 8, GunDll 0x1000f320: `HRESULT GetAccountName(const ushort*, void*)`.
    fn get_account_name(&self, a: &str, context: Context) -> Hresult;
    /// Slot 9, GunDll 0x1000f3e0: `HRESULT SetAccountValue(ACCOUNTVALUE_TYPE, uchar*)`. Retail
    /// passes a bare `uchar*`; how many bytes GunDll reads is not determined.
    fn set_account_value(&self, a: AccountValueType, b: &[u8]) -> Hresult;
}

/// `IGunFriends` (IID eeeeee24-85ea-40f7-8157-cd07a138b49f).
pub trait IGunFriends {
    /// Slot 3, GunDll 0x1000dc40: `HRESULT GetFriendsList(void*)`.
    fn get_friends_list(&self, context: Context) -> Hresult;
    /// Slot 4, GunDll 0x1000dcd0: `HRESULT FriendAdd(const ushort*, void*)`.
    fn friend_add(&self, user: &str, context: Context) -> Hresult;
    /// Slot 5, GunDll 0x1000dd70: `HRESULT FriendDelete(const ushort*, void*)`.
    fn friend_delete(&self, user: &str, context: Context) -> Hresult;
    /// Slot 6, GunDll 0x1000de10: `HRESULT GetIgnoredList(void*)`.
    fn get_ignored_list(&self, context: Context) -> Hresult;
    /// Slot 7, GunDll 0x1000dea0: `HRESULT IgnoredAdd(const ushort*, void*)`.
    fn ignored_add(&self, user: &str, context: Context) -> Hresult;
    /// Slot 8, GunDll 0x1000df40: `HRESULT IgnoredDelete(const ushort*, void*)`.
    fn ignored_delete(&self, user: &str, context: Context) -> Hresult;
    /// Slot 9, GunDll 0x1000dfe0: `HRESULT Flags(IGunFriends::FLAG_OPERATION, IGunFriends::FLAG_WORD, ulong, void*)`.
    fn flags(
        &self,
        op: FlagOperation,
        which_flag: FlagWord,
        bits_to_set_or_clear: u32,
        context: Context,
    ) -> Hresult;
    /// Slot 10, GunDll 0x1000e080: `HRESULT WhereIs(const ushort*, void*)`.
    fn where_is(&self, a: &str, context: Context) -> Hresult;
    /// Slot 11, GunDll 0x1000e120: `HRESULT SetLocation(ulong, const ushort*, void*)`.
    fn set_location(&self, mode: u32, location: &str, context: Context) -> Hresult;
    /// Slot 12, GunDll 0x1000e1c0: `HRESULT AcceptAddFriend(bool, const ushort*, void*)`.
    fn accept_add_friend(&self, a: bool, b: &str, context: Context) -> Hresult;
}

/// `IGunChat` (IID eeeeee25-85ea-40f7-8157-cd07a138b49f).
pub trait IGunChat {
    /// Slot 3, GunDll 0x1000e260: `HRESULT GetListOfRooms(void*)`.
    fn get_list_of_rooms(&self, context: Context) -> Hresult;
    /// Slot 4, GunDll 0x1000e2f0: `HRESULT JoinRoom(const ushort*, const ushort*, void*)`.
    fn join_room(&self, a: &str, b: &str, context: Context) -> Hresult;
    /// Slot 5, GunDll 0x1000e3a0: `HRESULT LeaveRoom(ulong, void*)`.
    fn leave_room(&self, a: u32, context: Context) -> Hresult;
    /// Slot 6, GunDll 0x1000e430: `HRESULT LeaveAllRooms(void*)`.
    fn leave_all_rooms(&self, context: Context) -> Hresult;
    /// Slot 7, GunDll 0x1000e4c0: `HRESULT GetUsersInRoom(ulong, void*)`.
    fn get_users_in_room(&self, a: u32, context: Context) -> Hresult;
    /// Slot 8, GunDll 0x1000e550: `HRESULT SendToRoom(ulong, const ushort*, void*)`.
    fn send_to_room(&self, a: u32, b: &str, context: Context) -> Hresult;
    /// Slot 9, GunDll 0x1000e6b0: `HRESULT SendToUser(const ushort*, const ushort*, void*)`.
    fn send_to_user(&self, a: &str, b: &str, context: Context) -> Hresult;
    /// Slot 10, GunDll 0x1000e820: `HRESULT GetSquelchList(void*)`.
    fn get_squelch_list(&self, context: Context) -> Hresult;
    /// Slot 11, GunDll 0x1000e8b0: `HRESULT SquelchAdd(const ushort*, void*)`.
    fn squelch_add(&self, a: &str, context: Context) -> Hresult;
    /// Slot 12, GunDll 0x1000e950: `HRESULT SquelchDelete(const ushort*, void*)`.
    fn squelch_delete(&self, a: &str, context: Context) -> Hresult;
    /// Slot 13, GunDll 0x1000e9e0: `HRESULT CreateRoom(const ushort*, const ushort*, ulong, ulong, const ushort*, void*)`.
    fn create_room(&self, a: &str, b: &str, c: u32, d: u32, e: &str, context: Context) -> Hresult;
    /// Slot 14, GunDll 0x1000eaa0: `HRESULT GetRoomOwner(ulong, void*)`.
    fn get_room_owner(&self, a: u32, context: Context) -> Hresult;
    /// Slot 15, GunDll 0x1000eb30: `HRESULT AssignOwnership(ulong, const ushort*, void*)`.
    fn assign_ownership(&self, a: u32, b: &str, context: Context) -> Hresult;
    /// Slot 16, GunDll 0x1000ebd0: `HRESULT SetRoomTopic(ulong, const ushort*, void*)`.
    fn set_room_topic(&self, a: u32, b: &str, context: Context) -> Hresult;
    /// Slot 17, GunDll 0x1000ec70: `HRESULT SetRoomMode(ulong, ulong, const ushort*, void*)`.
    fn set_room_mode(&self, a: u32, b: u32, c: &str, context: Context) -> Hresult;
    /// Slot 18, GunDll 0x1000ed10: `HRESULT InviteUser(ulong, const ushort*, const ushort*, void*)`.
    fn invite_user(&self, a: u32, b: &str, c: &str, context: Context) -> Hresult;
    /// Slot 19, GunDll 0x1000edc0: `HRESULT KickOutUser(ulong, const ushort*, const ushort*, void*)`.
    fn kick_out_user(&self, a: u32, b: &str, c: &str, context: Context) -> Hresult;
    /// Slot 20, GunDll 0x1000ee70: `HRESULT RetrieveChatLog(const ushort*, const ushort*, void*)`.
    fn retrieve_chat_log(&self, a: &str, b: &str, context: Context) -> Hresult;
    /// Slot 21, GunDll 0x1000e5f0: `HRESULT SendToRoomEx(ulong, const ushort*, tagBLOB*, void*)`.
    fn send_to_room_ex(
        &self,
        room_id: u32,
        message: &str,
        blob: &[u8],
        context: Context,
    ) -> Hresult;
    /// Slot 22, GunDll 0x1000e760: `HRESULT SendToUserEx(const ushort*, const ushort*, tagBLOB*, void*)`.
    fn send_to_user_ex(&self, user: &str, message: &str, blob: &[u8], context: Context) -> Hresult;
}

/// `IGunHostEnum`: the callback `IGunHost::EnumPlayers` / `EnumItemValues` call (no `IUnknown`
/// slots in the PDB's layout).
pub trait IGunHostEnum {
    /// Slot 0: `HRESULT OnEnumPlayer(ulong, const ushort*)`.
    fn on_enum_player(&mut self, a: u32, b: &str) -> Hresult;
    /// Slot 1: `HRESULT OnEnumItemValue(const char*, const ushort*)`.
    fn on_enum_item_value(&mut self, a: &str, b: &str) -> Hresult;
}

/// `IGunHost` (IID eeeeee26-85ea-40f7-8157-cd07a138b49f).
pub trait IGunHost {
    /// Slot 3, GunDll 0x1000d920: `HRESULT CreateGame(const ushort*)`.
    fn create_game(&self, a: &str) -> Hresult;
    /// Slot 4, GunDll 0x1000d760: `HRESULT EndGame()`.
    fn end_game(&self) -> Hresult;
    /// Slot 5, GunDll 0x1000d330: `HRESULT AddPlayer(ulong, const ushort*)`.
    fn add_player(&self, a: u32, b: &str) -> Hresult;
    /// Slot 6, GunDll 0x1000d440: `HRESULT DeletePlayer(ulong)`.
    fn delete_player(&self, a: u32) -> Hresult;
    /// Slot 7, GunDll 0x1000cc20: `HRESULT GetNumPlayers(ulong*)`.
    fn get_num_players(&self, a: &mut u32) -> Hresult;
    /// Slot 8, GunDll 0x1000cc60: `HRESULT EnumPlayers(IGunHostEnum*)`.
    fn enum_players(&self, sink: &mut dyn IGunHostEnum) -> Hresult;
    /// Slot 9, GunDll 0x1000ccb0: `HRESULT SetItemValue(ulong, const char*, const ushort*)`.
    fn set_item_value(&self, a: u32, b: &str, c: &str) -> Hresult;
    /// Slot 10, GunDll 0x1000cda0: `HRESULT GetItemValue(ulong, const char*, ushort*, ulong)`:
    /// `c` is the `ushort*` buffer and its `ulong` length together.
    fn get_item_value(&self, a: u32, b: &str, c: &mut [u16]) -> Hresult;
    /// Slot 11, GunDll 0x1000ce50: `HRESULT DeleteItemValue(ulong, const char*)`.
    fn delete_item_value(&self, a: u32, b: &str) -> Hresult;
    /// Slot 12, GunDll 0x1000cf20: `HRESULT GetNumItemValues(ulong, ulong*)`.
    fn get_num_item_values(&self, a: u32, b: &mut u32) -> Hresult;
    /// Slot 13, GunDll 0x1000cfa0: `HRESULT EnumItemValues(ulong, IGunHostEnum*)`.
    fn enum_item_values(&self, a: u32, sink: &mut dyn IGunHostEnum) -> Hresult;
}

/// `IGunBrowser` (IID eeeeee27-85ea-40f7-8157-cd07a138b49f).
pub trait IGunBrowser {
    /// Slot 3, GunDll 0x1000bff0: `HRESULT GetViewList(void*)`.
    fn get_view_list(&self, context: Context) -> Hresult;
    /// Slot 4, GunDll 0x1000c080: `HRESULT GetHeaderRowOfView(ulong, void*)`.
    fn get_header_row_of_view(&self, a: u32, context: Context) -> Hresult;
    /// Slot 5, GunDll 0x1000c110: `HRESULT GetPage(ulong, ulong, ulong, const ushort*, void*)`.
    fn get_page(&self, a: u32, b: u32, c: u32, d: &str, context: Context) -> Hresult;
    /// Slot 6, GunDll 0x1000c1c0: `HRESULT GetRowPage(ulong, ulong, ulong, const ushort*, void*)`.
    fn get_row_page(&self, a: u32, b: u32, c: u32, d: &str, context: Context) -> Hresult;
}

/// `IGunSupport` (IID eeeeee29-85ea-40f7-8157-cd07a138b49f).
pub trait IGunSupport {
    /// Slot 3, GunDll 0x1000ef20: `HRESULT CallCreate(SUPPORT_CALLTYPE, const ushort*, const ushort*, tagBLOB*, ulong, void*)`.
    /// `a` (the target user) and `b` (the description) are NULL (`None`) for an empty string in
    /// `TurbineGun::RequestCallCreate`.
    fn call_create(
        &self,
        call_type: SupportCallType,
        a: Option<&str>,
        b: Option<&str>,
        c: &[u8],
        d: u32,
        context: Context,
    ) -> Hresult;
    /// Slot 4, GunDll 0x1000f0e0: `HRESULT CallGetStatus(ulong, void*)`.
    fn call_get_status(&self, a: u32, context: Context) -> Hresult;
}

/// `IGunPersona` (IID eb60ba8a-54a6-4a6d-9432-8337dfff7612).
pub trait IGunPersona {
    /// Slot 3, GunDll 0x1000b2e0: `HRESULT EnterPersona(const ushort*, ulong, void*)`.
    fn enter_persona(&self, a: &str, b: u32, context: Context) -> Hresult;
    /// Slot 4, GunDll 0x1000b390: `HRESULT LeavePersona(void*)`.
    fn leave_persona(&self, context: Context) -> Hresult;
    /// Slot 5, GunDll 0x1000b420: `HRESULT CreatePersona(const ushort*, ulong, void*)`.
    fn create_persona(&self, a: &str, b: u32, context: Context) -> Hresult;
    /// Slot 6, GunDll 0x1000b4c0: `HRESULT DeletePersona(const ushort*, ulong, void*)`.
    fn delete_persona(&self, a: &str, b: u32, context: Context) -> Hresult;
    /// Slot 7, GunDll 0x1000b560: `HRESULT GetPersonaList(ulong, void*)`.
    fn get_persona_list(&self, a: u32, context: Context) -> Hresult;
}

/// `IGunTicket` (IID c4ef31f6-6d0a-4e84-a67b-cf97a22c57a2).
pub trait IGunTicket {
    /// Slot 3, GunDll 0x1000b5f0: `HRESULT SimulateAuthentication(const ushort*, const ushort*,
    /// tagBLOB*)`. The blob is in/out: `cbSize` is the buffer size on entry and 0xCC on return, and
    /// the 0xCC bytes written are the simulated ticket (two 51-character fields; their NUL
    /// terminators are the zero markers `PresentTicket` tests at offsets 100 and 0xCA).
    fn simulate_authentication(&self, a: &str, b: &str, c: &mut TagBlob) -> Hresult;
    /// Slot 4, GunDll 0x1000ba80: `HRESULT GetTicket(tagBLOB*)`. `gmClient::AuthenticateWithGUN`
    /// (through `TurbineGun::GetAuthenticationTicket`) calls it with an empty blob; on
    /// `ZT_E_BUFFER_TOO_SMALL` with `cbSize` set it allocates that many bytes and calls again.
    fn get_ticket(&self, blob: &mut TagBlob) -> Hresult;
    /// Slot 5, GunDll 0x1000b6d0: `HRESULT GetTicketEx(void*)`.
    fn get_ticket_ex(&self, context: Context) -> Hresult;
    /// Slot 6, GunDll 0x1000bc00: `HRESULT SetTicket(tagBLOB*)`. GunDll rewrites the caller's blob
    /// in place and sets its `cbSize` (`0a_send_session.md` §6.1).
    fn set_ticket(&self, ticket: &mut TagBlob) -> Hresult;
    /// Slot 7, GunDll 0x1000be40: `HRESULT PresentTicket(tagBLOB*, void*)`.
    fn present_ticket(&self, ticket: &[u8], context: Context) -> Hresult;
    /// Slot 8, GunDll 0x1000b6e0: `HRESULT ReleaseTicket(void*)`.
    fn release_ticket(&self, context: Context) -> Hresult;
    /// Slot 9, GunDll 0x1000b780: `HRESULT CheckTicket(tagBLOB*, void*)`.
    fn check_ticket(&self, a: &[u8], context: Context) -> Hresult;
    /// Slot 10, GunDll 0x1000b9b0: `HRESULT BootUser(const ushort*, const ushort*, void*)`.
    fn boot_user(&self, a: &str, b: &str, context: Context) -> Hresult;
}

/// GunDll's one COM object (`CGun`): every interface the client queries.
pub trait GunObject:
    IGunConnection
    + IGunNews
    + IGunAccount
    + IGunFriends
    + IGunChat
    + IGunHost
    + IGunBrowser
    + IGunSupport
    + IGunPersona
    + IGunTicket
{
}

impl<T> GunObject for T where
    T: IGunConnection
        + IGunNews
        + IGunAccount
        + IGunFriends
        + IGunChat
        + IGunHost
        + IGunBrowser
        + IGunSupport
        + IGunPersona
        + IGunTicket
{
}
