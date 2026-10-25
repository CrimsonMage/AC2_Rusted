//! `ggmCWInterface::CallEFun_RequestGUN_*`: the chat-service requests scripts make
//! (`gmCWInterface::RequestGUN_*`). Each wrapper crashes on an undefined `this` and stores the
//! request's result after its arguments. The host sends the queued [`ChatRequest`]s.

use crate::glue::{word, Call};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

/// A chat-service request from a script, as `gmCWInterface::RequestGUN_*` makes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRequest {
    /// `RequestGUN_SendToRoom(roomID, message, flags)`: Turbine chat
    /// (`CWBookKeeper::CSendToTurbineRoomByID`, its extra info a `GunChatExtraInfo` with these
    /// weenie chat flags) when the client uses it, else GUN `RequestSendToRoomEx`.
    SendToRoom { room: u32, text: String, flags: u32 },
    /// `RequestGUN_SendToUser(user, message, flags)`.
    SendToUser {
        user: String,
        text: String,
        flags: u32,
    },
    /// `RequestGUN_SetStateFlags(flags)` (`TurbineGun::RequestFlags(STATE, SET, …)`).
    SetStateFlags(u32),
    /// `RequestGUN_SetLocation(flags, location)`.
    SetLocation { flags: u32, location: String },
    /// `RequestGUN_FriendAdd(name)`.
    FriendAdd(String),
    /// `RequestGUN_FriendDelete(name)`.
    FriendDelete(String),
    /// `RequestGUN_GetFriendsList(userRequested)`.
    GetFriendsList { user_requested: bool },
    /// `RequestGUN_IgnoredAdd(name)`.
    IgnoredAdd(String),
    /// `RequestGUN_IgnoredDelete(name)`.
    IgnoredDelete(String),
    /// `RequestGUN_GetIgnoredList(userRequested)`.
    GetIgnoredList { user_requested: bool },
    /// `RequestGUN_CallCreate(callType, worldName, targetUserName, description)`.
    CallCreate {
        call_type: u32,
        world: String,
        target: String,
        description: String,
    },
}

/// Which chat services the client has (`AC2ClientChatManager::GetIChatClient` for Turbine
/// chat, `TurbineGun::S_pTurbineGun` for GUN). Requests to a missing service return 0.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChatServices {
    pub turbine_chat: bool,
    pub gun: bool,
}

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x00B4_0032, send_to_user);
    efuns.register(0x00B4_0033, send_to_room);
    efuns.register(0x00B4_0034, |vm, c| {
        let flags = word(c, 2);
        gun(vm, c, 3, ChatRequest::SetStateFlags(flags))
    });
    efuns.register(0x00B4_0035, |vm, c| {
        let r = ChatRequest::FriendAdd(text(vm, c, 2));
        gun(vm, c, 3, r)
    });
    efuns.register(0x00B4_0036, |vm, c| {
        let r = ChatRequest::FriendDelete(text(vm, c, 2));
        gun(vm, c, 3, r)
    });
    efuns.register(0x00B4_0037, |vm, c| {
        let user_requested = word(c, 2) != 0;
        gun(vm, c, 3, ChatRequest::GetFriendsList { user_requested })
    });
    efuns.register(0x00B4_0038, |vm, c| {
        let r = ChatRequest::IgnoredAdd(text(vm, c, 2));
        gun(vm, c, 3, r)
    });
    efuns.register(0x00B4_0039, |vm, c| {
        let r = ChatRequest::IgnoredDelete(text(vm, c, 2));
        gun(vm, c, 3, r)
    });
    efuns.register(0x00B4_003A, |vm, c| {
        let user_requested = word(c, 2) != 0;
        gun(vm, c, 3, ChatRequest::GetIgnoredList { user_requested })
    });
    efuns.register(0x00B4_003B, call_create);
    efuns.register(0x00B4_00EB, |vm, c| {
        let r = ChatRequest::SetLocation {
            flags: word(c, 2),
            location: text(vm, c, 3),
        };
        gun(vm, c, 4, r)
    });
}

fn text(vm: &Vm, ctx: &Context, n: usize) -> String {
    String::from_utf16_lossy(vm.wstring(word(ctx, n)).unwrap_or(&[]))
}

/// Stores `result` in slot `ret`, crashing first on an undefined `this` ("index !=
/// Undef_ReferenceID").
fn finish(
    vm: &mut Vm,
    ctx: &mut Context,
    ret: usize,
    request: Option<ChatRequest>,
    result: u32,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    if this == u32::MAX {
        return vm.bad_this(this);
    }
    if let Some(request) = request {
        vm.outbox.chat_requests.push(request);
    }
    Call::new(ctx).ret(ret, result, tag::BOOL);
    InterpStatus::Success
}

/// A GUN-only request: 0 without GUN, else queued (the request's result is taken as 1).
fn gun(vm: &mut Vm, ctx: &mut Context, ret: usize, request: ChatRequest) -> InterpStatus {
    if vm.chat.gun {
        finish(vm, ctx, ret, Some(request), 1)
    } else {
        finish(vm, ctx, ret, None, 0)
    }
}

/// `RequestGUN_SendToRoom`: 0 for an empty message or room 0; Turbine chat first, then GUN;
/// with neither, retail returns the room id.
fn send_to_room(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (room, flags) = (word(ctx, 2), word(ctx, 4));
    let text = text(vm, ctx, 3);
    if text.is_empty() || room == 0 {
        return finish(vm, ctx, 5, None, 0);
    }
    let request = ChatRequest::SendToRoom { room, text, flags };
    match vm.chat {
        ChatServices {
            turbine_chat: true, ..
        }
        | ChatServices { gun: true, .. } => finish(vm, ctx, 5, Some(request), 1),
        _ => finish(vm, ctx, 5, None, room),
    }
}

/// `RequestGUN_SendToUser(user, message, flags)`: 0 for an empty user or message.
fn send_to_user(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (user, text) = (text(vm, ctx, 2), text(vm, ctx, 3));
    if user.is_empty() || text.is_empty() {
        return finish(vm, ctx, 5, None, 0);
    }
    let flags = word(ctx, 4);
    gun(vm, ctx, 5, ChatRequest::SendToUser { user, text, flags })
}

/// `RequestGUN_CallCreate(callType, worldName, targetUserName, description)`.
fn call_create(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let r = ChatRequest::CallCreate {
        call_type: word(ctx, 2),
        world: text(vm, ctx, 3),
        target: text(vm, ctx, 4),
        description: text(vm, ctx, 5),
    };
    gun(vm, ctx, 6, r)
}
