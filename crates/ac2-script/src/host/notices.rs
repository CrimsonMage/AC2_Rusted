//! The `gCM_*::CallEFun_SendNotice_*` efuns: each wrapper reads its frame (this, then the
//! arguments; 64-bit ids high word first), calls `CM_*::SendNotice_*` and stores its 1 after
//! the arguments. Notices ac2-weenie types ([`GameplayNotice`]) are raised typed; the rest are
//! raised as a [`ScriptNotice`] with their arguments decoded by the signature.

use ac2_net::protocol as net;
use ac2_weenie::GameplayNotice;

use crate::glue::{word, word64, Call};
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};
use crate::wire;

/// A `CM_*::SendNotice_*` the UI listens to.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptNotice {
    /// The retail function, `CM_Fellowship::SendNotice_UpdateFellowship`.
    pub name: &'static str,
    /// The weenie object the notice is raised on (`obj`); 0 for the `CM_UI` notices.
    pub object: u64,
    pub args: Vec<NoticeArg>,
}

/// One argument of a [`ScriptNotice`], in signature order.
#[derive(Debug, Clone, PartialEq)]
pub enum NoticeArg {
    /// `int` / `ulong` / `bool`.
    Int(u32),
    /// `InstanceID` / `ulong64`.
    Long(u64),
    Float(f32),
    /// `PStringBase<unsigned short>`.
    Text(String),
    /// `StringInfo` and the other natives, in ac2-net's form.
    Native(net::NativeObject),
    /// A null reference.
    Null,
    /// A native the wire has no form for yet (its native type).
    Unconvertible(u32),
}

/// Each notice efun with its arguments after `obj`: `o` the object, `i` one word, `L` a 64-bit
/// id, `f` a float, `w` a wide string, `s` a `StringInfo`, `n` another native.
const NOTICES: [(u32, &str, &str); 100] = [
    (
        0x0084_0001,
        "CM_Allegiance::SendNotice_UpdateAllegianceProfile",
        "on",
    ),
    (
        0x0084_0002,
        "CM_Allegiance::SendNotice_RequestVassal",
        "oLs",
    ),
    (
        0x0084_0003,
        "CM_Allegiance::SendNotice_Patron_SwearDone",
        "oLi",
    ),
    (
        0x0084_0004,
        "CM_Allegiance::SendNotice_Vassal_SwearDone",
        "oLi",
    ),
    (0x0085_0001, "CM_Character::SendNotice_UpdateVital", "oiLL"),
    (0x0085_0002, "CM_Character::SendNotice_UpdateEffect", "o"),
    (
        0x0085_0003,
        "CM_Character::SendNotice_SelectionChanged",
        "oL",
    ),
    (
        0x0085_0004,
        "CM_Character::SendNotice_UpdateSelectability",
        "oi",
    ),
    (
        0x0085_0005,
        "CM_Character::SendNotice_UpdateSkillPanel",
        "oLii",
    ),
    (0x0085_0006, "CM_Character::SendNotice_UpdateSkill", "oi"),
    (0x0085_0007, "CM_Character::SendNotice_UpdateResetTime", "o"),
    (
        0x0085_0008,
        "CM_Character::SendNotice_UpdateDeathState",
        "oi",
    ),
    (0x0085_0009, "CM_Character::SendNotice_ReloadOptions", "o"),
    (0x0085_000A, "CM_Character::SendNotice_ChangeRadarLook", "o"),
    (
        0x0085_000B,
        "CM_Character::SendNotice_UpdateSelectionInfo",
        "on",
    ),
    (
        0x0085_000C,
        "CM_Character::SendNotice_DisplayMessage",
        "osi",
    ),
    (
        0x0085_000D,
        "CM_Character::SendNotice_UpdateFactionStatus",
        "oi",
    ),
    (0x0085_000E, "CM_Character::SendNotice_UpdateVitae", "offf"),
    (
        0x0085_000F,
        "CM_Character::SendNotice_UpdateMountState",
        "oi",
    ),
    (
        0x0085_0012,
        "CM_Character::SendNotice_UpdateHeroStatus",
        "oiLL",
    ),
    (
        0x0085_0013,
        "CM_Character::SendNotice_ResurrectionRequestedByOther",
        "os",
    ),
    (0x0085_0014, "CM_Character::SendNotice_PulseMissed", "oi"),
    (0x0085_0019, "CM_Character::SendNotice_AddMarker", "oiLiis"),
    (0x0085_001A, "CM_Character::SendNotice_ClearMarkers", "oi"),
    (0x0085_001B, "CM_Character::SendNotice_RemoveMarker", "oiL"),
    (0x0085_001C, "CM_Character::SendNotice_RemovePet", "oL"),
    (0x0085_001D, "CM_Character::SendNotice_UpdatePetMode", "oLi"),
    (0x0085_001E, "CM_Character::SendNotice_AddPet", "oLsii"),
    (0x0085_001F, "CM_Character::SendNotice_UpdatePetName", "oLs"),
    (
        0x0085_0020,
        "CM_Character::SendNotice_PetIsUnderAttack",
        "oL",
    ),
    (0x0086_0001, "CM_Combat::SendNotice_UpdateAttackState", "oi"),
    (
        0x0086_0002,
        "CM_Combat::SendNotice_RequestSpecialAttack",
        "oii",
    ),
    (
        0x0086_0003,
        "CM_Combat::SendNotice_StartSpecialAttack",
        "oi",
    ),
    (0x0086_0004, "CM_Combat::SendNotice_EndSpecialAttack", "oi"),
    (
        0x0086_0005,
        "CM_Combat::SendNotice_ToggleSpecialAttack",
        "oii",
    ),
    (
        0x0087_0001,
        "CM_Communication::SendNotice_DisplayStringInfo",
        "ois",
    ),
    (
        0x0087_0002,
        "CM_Communication::SendNotice_DoAssistedTell",
        "ow",
    ),
    (
        0x0087_0004,
        "CM_Communication::SendNotice_ReloadChatSettings",
        "o",
    ),
    (0x0088_0001, "CM_Craft::SendNotice_UpdateCraftPanel", "o"),
    (
        0x0088_0002,
        "CM_Craft::SendNotice_AddIngredient_Done",
        "oiiLii",
    ),
    (0x0088_0003, "CM_Craft::SendNotice_ExecuteCraft_Done", "oii"),
    (0x0088_0004, "CM_Craft::SendNotice_UpdateCost", "oiii"),
    (
        0x0088_0005,
        "CM_Craft::SendNotice_OpenIngredientPanel",
        "oi",
    ),
    (0x0088_0008, "CM_Craft::SendNotice_UpdateCraftSkill", "on"),
    (
        0x0089_0001,
        "CM_Examination::SendNotice_UpdateExaminationProfile",
        "on",
    ),
    (
        0x0089_0002,
        "CM_Examination::SendNotice_ExaminationRequestChanged",
        "on",
    ),
    (
        0x008A_0001,
        "CM_Fellowship::SendNotice_UpdateFellowship",
        "on",
    ),
    (
        0x008A_0002,
        "CM_Fellowship::SendNotice_ClearFellowship",
        "o",
    ),
    (0x008A_0003, "CM_Fellowship::SendNotice_UpdateFellow", "oLn"),
    (0x008A_0004, "CM_Fellowship::SendNotice_ClearFellow", "oL"),
    (
        0x008A_0005,
        "CM_Fellowship::SendNotice_UpdateFellowLevel",
        "oLi",
    ),
    (
        0x008A_0006,
        "CM_Fellowship::SendNotice_RequestRecruitment",
        "oLss",
    ),
    (
        0x008A_0007,
        "CM_Fellowship::SendNotice_ExpireRecruitment",
        "oL",
    ),
    (
        0x008A_0008,
        "CM_Fellowship::SendNotice_UpdateFellowHealth",
        "oLii",
    ),
    (
        0x008A_0009,
        "CM_Fellowship::SendNotice_UpdateFellowMaxHealth",
        "oLi",
    ),
    (
        0x008A_000A,
        "CM_Fellowship::SendNotice_UpdateFellowVigor",
        "oLii",
    ),
    (
        0x008A_000B,
        "CM_Fellowship::SendNotice_UpdateFellowMaxVigor",
        "oLi",
    ),
    (0x008A_000C, "CM_Fellowship::SendNotice_UpdateLeader", "oL"),
    (
        0x008A_000E,
        "CM_Fellowship::SendNotice_UpdateFellowCellID",
        "oLi",
    ),
    (
        0x008B_0001,
        "CM_Inventory::SendNotice_UpdateInventory",
        "oiLi",
    ),
    (
        0x008B_0002,
        "CM_Inventory::SendNotice_UpdateContainer",
        "oLiLi",
    ),
    (
        0x008B_0003,
        "CM_Inventory::SendNotice_ContainerStateUpdate",
        "oLi",
    ),
    (0x008B_0004, "CM_Inventory::SendNotice_ItemUpdate", "o"),
    (0x008C_0001, "CM_Money::SendNotice_GhostMoneyBag", "oi"),
    (0x008C_0002, "CM_Money::SendNotice_UpdateMoney", "oi"),
    (0x008D_0001, "CM_Physics::SendNotice_Do3DDamage", "on"),
    (0x008E_0001, "CM_Quest::SendNotice_UpdateQuests", "oin"),
    (0x008E_0002, "CM_Quest::SendNotice_UpdateStoryQuests", "oii"),
    (0x008E_0003, "CM_Quest::SendNotice_PlaySequence", "on"),
    (0x008F_0001, "CM_Shortcut::SendNotice_ReloadShortcuts", "o"),
    (
        0x0090_0001,
        "CM_Trade::SendNotice_Trade_BeRegistered",
        "oLLs",
    ),
    (0x0090_0002, "CM_Trade::SendNotice_Trade_BeOpened", "o"),
    (0x0090_0003, "CM_Trade::SendNotice_Trade_BeClosed", "oLi"),
    (0x0090_0004, "CM_Trade::SendNotice_Trade_BeAccepted", "oL"),
    (0x0090_0005, "CM_Trade::SendNotice_Trade_BeDeclined", "oL"),
    (0x0090_0006, "CM_Trade::SendNotice_Trade_BeReset", "o"),
    (
        0x0090_0007,
        "CM_Trade::SendNotice_Trade_BeAcceptanceCleared",
        "o",
    ),
    (0x0090_0008, "CM_Trade::SendNotice_Trade_BeOffered", "oLLi"),
    (0x0090_0009, "CM_Trade::SendNotice_Trade_BeRevoked", "oLL"),
    (0x0091_0001, "CM_UI::SendNotice_PortalSpace", "i"),
    (0x0091_0002, "CM_UI::SendNotice_OpenExaminationPanel", "-"),
    (0x0091_0005, "CM_UI::SendNotice_PortalScene", "in"),
    (
        0x0092_0001,
        "CM_Usage::SendNotice_SetTargetingPointer",
        "oi",
    ),
    (0x0092_0002, "CM_Usage::SendNotice_UseBook", "oiiwwi"),
    (
        0x23AC_0004,
        "CM_Friends::SendNotice_GUNResult_GetFriendsList",
        "on",
    ),
    (
        0x23AC_0005,
        "CM_Friends::SendNotice_GUNResult_AddFriend",
        "ow",
    ),
    (
        0x23AC_0006,
        "CM_Friends::SendNotice_GUNResult_RemoveFriend",
        "ow",
    ),
    (
        0x23AC_0007,
        "CM_Friends::SendNotice_GUNResult_FriendStatus",
        "on",
    ),
    (0x23AC_0008, "CM_Friends::SendNotice_LocalStatus", "oi"),
    (0x3839_0001, "CM_Store::SendNotice_UpdateStore", "on"),
    (0x3839_0002, "CM_Store::SendNotice_UpdateSale", "oLi"),
    (0x3839_0003, "CM_Store::SendNotice_LeaveStore", "o"),
    (0x3839_0004, "CM_Store::SendNotice_ConfirmPurchase", "osisi"),
    (0x3839_0006, "CM_Store::SendNotice_ViewStore", "on"),
    (0x3839_0007, "CM_Store::SendNotice_ViewCatalog", "osn"),
    (0x3839_0008, "CM_Store::SendNotice_LeaveCatalog", "o"),
    (0x3839_0009, "CM_Store::SendNotice_ViewConsignment", "on"),
    (0x3839_000A, "CM_Store::SendNotice_UpdateConsignment", "on"),
    (0x3839_000B, "CM_Store::SendNotice_LeaveConsignment", "o"),
    (0x3839_000D, "CM_Store::SendNotice_ShowSaleReminder", "oi"),
];

pub(super) fn register(efuns: &mut EFuns) {
    for (id, _, _) in NOTICES {
        efuns.register(id, generic);
    }
    efuns.register(0x0085_0001, update_vital);
    efuns.register(0x0085_0003, selection_changed);
    efuns.register(0x0085_000B, update_selection_info);
    efuns.register(0x0085_0012, update_hero_status);
    efuns.register(0x0087_0001, display_string_info);
    efuns.register(0x0089_0001, |vm, c| examination(vm, c, false));
    efuns.register(0x0089_0002, |vm, c| examination(vm, c, true));
    efuns.register(0x008B_0001, update_inventory);
    efuns.register(0x008B_0002, update_container);
}

/// The notice efuns, with their retail names (for the catalog and tests).
pub fn notice_names() -> impl Iterator<Item = (u32, &'static str)> {
    NOTICES.iter().map(|&(id, name, _)| (id, name))
}

/// The weenie object a notice is raised on (`obj`, a script instance its process owns).
fn owner(vm: &Vm, reference: u32) -> u64 {
    vm.owners.get(&reference).copied().unwrap_or(0)
}

fn native_arg(vm: &Vm, reference: u32) -> NoticeArg {
    let Ok(object) = vm.refs.heap(reference) else {
        return NoticeArg::Null;
    };
    match object.native.as_ref() {
        None => NoticeArg::Null,
        Some(NativeObject::WString(s)) => NoticeArg::Text(String::from_utf16_lossy(s)),
        Some(native) => match wire::native_to_net(object.native_type, native) {
            Ok(n) => NoticeArg::Native(n),
            Err(_) => NoticeArg::Unconvertible(object.native_type),
        },
    }
}

/// Any notice without an ac2-weenie type: decodes the frame by the signature.
fn generic(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = vm.current_efun();
    let Some(&(_, name, kinds)) = NOTICES.iter().find(|n| n.0 == id) else {
        return InterpStatus::Crash;
    };
    let (mut slot, mut object, mut args) = (2, 0, Vec::new());
    for kind in kinds.bytes() {
        match kind {
            b'o' => object = owner(vm, word(ctx, slot)),
            b'L' => {
                slot += 1;
                args.push(NoticeArg::Long(word64(ctx, slot)));
            }
            b'i' => args.push(NoticeArg::Int(word(ctx, slot))),
            b'f' => args.push(NoticeArg::Float(f32::from_bits(word(ctx, slot)))),
            b'-' => continue,
            _ => args.push(native_arg(vm, word(ctx, slot))),
        }
        slot += 1;
    }
    vm.outbox
        .script_notices
        .push(ScriptNotice { name, object, args });
    Call::new(ctx).ret(slot, 1, tag::BOOL);
    InterpStatus::Success
}

fn wire_native(vm: &Vm, reference: u32) -> Option<net::NativeObject> {
    let object = vm.refs.heap(reference).ok()?;
    wire::native_to_net(object.native_type, object.native.as_ref()?).ok()
}

fn notice(
    vm: &mut Vm,
    ctx: &mut Context,
    ret: usize,
    n: Option<GameplayNotice>,
    what: &str,
) -> InterpStatus {
    match n {
        Some(n) => vm.outbox.notices.push(n),
        None => vm
            .outbox
            .dropped
            .push(format!("{what}: the native has no wire form")),
    }
    Call::new(ctx).ret(ret, 1, tag::BOOL);
    InterpStatus::Success
}
/// `CM_Character::SendNotice_SelectionChanged(obj, newSelection)`.
fn selection_changed(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = word64(ctx, 4);
    notice(
        vm,
        ctx,
        5,
        Some(GameplayNotice::SelectionChanged(id)),
        "SelectionChanged",
    )
}

/// `CM_Character::SendNotice_UpdateSelectionInfo(obj, info)`: `id` is the weenie the notice
/// is raised on.
fn update_selection_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (obj, info) = (word(ctx, 2), word(ctx, 3));
    let n = match wire_native(vm, info) {
        Some(net::NativeObject::SelectionInfo(info)) => Some(GameplayNotice::UpdateSelectionInfo {
            id: owner(vm, obj),
            info,
        }),
        _ => None,
    };
    notice(vm, ctx, 4, n, "UpdateSelectionInfo")
}

/// `CM_Communication::SendNotice_DisplayStringInfo(obj, type, msg)`.
fn display_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (kind, msg) = (word(ctx, 3), word(ctx, 4));
    let n = match wire_native(vm, msg) {
        Some(net::NativeObject::StringInfo(text)) => {
            Some(GameplayNotice::DisplayStringInfo { kind, text })
        }
        _ => None,
    };
    notice(vm, ctx, 5, n, "DisplayStringInfo")
}

/// `CM_Examination::SendNotice_ExaminationRequestChanged` / `UpdateExaminationProfile`.
fn examination(vm: &mut Vm, ctx: &mut Context, request: bool) -> InterpStatus {
    let n = match (wire_native(vm, word(ctx, 3)), request) {
        (Some(net::NativeObject::ExaminationRequest(r)), true) => {
            Some(GameplayNotice::ExaminationRequestChanged(r))
        }
        (Some(net::NativeObject::ExaminationProfile(p)), false) => {
            Some(GameplayNotice::UpdateExaminationProfile(p))
        }
        _ => None,
    };
    notice(vm, ctx, 4, n, "examination notice")
}

/// `CM_Character::SendNotice_UpdateVital(obj, vital, current, max)` (64-bit values high word
/// first).
fn update_vital(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (vital, current, max) = (word(ctx, 3), word64(ctx, 5), word64(ctx, 7));
    let n = GameplayNotice::UpdateVital {
        vital,
        current,
        max,
    };
    notice(vm, ctx, 8, Some(n), "UpdateVital")
}

/// `CM_Character::SendNotice_UpdateHeroStatus(obj, isHero, curFocus, maxFocus)`.
fn update_hero_status(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (hero, current, max) = (word(ctx, 3) != 0, word64(ctx, 5), word64(ctx, 7));
    let n = GameplayNotice::UpdateHeroStatus { hero, current, max };
    notice(vm, ctx, 8, Some(n), "UpdateHeroStatus")
}

/// `CM_Inventory::SendNotice_UpdateInventory(obj, code, itemID, location)`.
fn update_inventory(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (code, item, location) = (word(ctx, 3), word64(ctx, 5), word(ctx, 6));
    let n = GameplayNotice::UpdateInventory {
        code,
        item,
        location,
    };
    notice(vm, ctx, 7, Some(n), "UpdateInventory")
}

/// `CM_Inventory::SendNotice_UpdateContainer(obj, containerID, code, itemID, location)`.
fn update_container(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (container, code) = (word64(ctx, 4), word(ctx, 5));
    let (item, slot) = (word64(ctx, 7), word(ctx, 8) as i32);
    let n = GameplayNotice::UpdateContainer {
        container,
        code,
        item,
        slot,
    };
    notice(vm, ctx, 9, Some(n), "UpdateContainer")
}
