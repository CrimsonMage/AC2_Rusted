//! The notices the player's scripts raise for the UI (`CM_*::SendNotice_*`), as plain data,
//! whoever raises them: the native stand-ins, or the script VM's glue.

use ac2_net::protocol::{
    ExaminationProfile, ExaminationRequest, GunChatExtraInfo, SelectionInfo, StringInfo,
};

use crate::social::{AllegianceProfile, FellowInfo, FellowshipInfo};

/// A `CM_*::SendNotice_*` notice, as plain data, in the order the scripts raise them.
#[derive(Debug, Clone, PartialEq)]
pub enum GameplayNotice {
    /// `CM_Character::SendNotice_SelectionChanged`: the new selection (0 for none).
    SelectionChanged(u64),
    /// `CM_Character::SendNotice_Select`: the UI runs a selection action
    /// (`gmShortcutUI::RequestAttackStateChange` raises 0x41000024 when the player should
    /// auto-target before attacking).
    Select(u32),
    /// `CM_Character::SendNotice_UpdateSelectionInfo`: the selected object's health and vigor.
    UpdateSelectionInfo { id: u64, info: SelectionInfo },
    /// `CM_Examination::SendNotice_ExaminationRequestChanged`.
    ExaminationRequestChanged(ExaminationRequest),
    /// `CM_Examination::SendNotice_UpdateExaminationProfile`.
    UpdateExaminationProfile(ExaminationProfile),
    /// `CM_Communication::SendNotice_DisplayStringInfo`: a line for the chat windows, with its
    /// display type (`ChatWindow::RecvNotice_DisplayStringInfo` filters by it).
    DisplayStringInfo { kind: u32, text: StringInfo },
    /// `gmCWInterface::GUNEvent_HearFromRoom`: room chat from the chat server, before the
    /// player's script (`CPlayer` 0x9B) formats it for its channel.
    HearFromRoom {
        room: u32,
        speaker: String,
        text: String,
        /// The sender's `GunChatExtraInfo`, when it decodes.
        extra: Option<GunChatExtraInfo>,
    },
    /// `CCliSendToRoomResult::HandleSendToRoomResult` (`gmCWInterface::Result_SendToRoom`):
    /// the chat server's answer to the room message with this context id.
    SendToRoomResult { context: u32, result: u32 },
    /// `CM_Inventory::SendNotice_UpdateContainer`: `container`'s contents changed (`code`
    /// 0x40000001 refreshes all of them).
    UpdateContainer {
        container: u64,
        code: u32,
        item: u64,
        slot: i32,
    },
    /// `CM_Inventory::SendNotice_ContainerStateUpdate`: a container opened or closed.
    ContainerStateUpdate { container: u64, open: bool },
    /// `CM_Inventory::SendNotice_ItemUpdate`: an item's quantity or durability changed
    /// (`CItem` 0x3 `OnStatChanged`).
    ItemUpdate(u64),
    /// `CM_Inventory::SendNotice_UpdateInventory`: the equipment changed (0x40000001: all).
    UpdateInventory { code: u32, item: u64, location: u32 },
    /// `CUsageSystem` 0x2: a use failed with `error` (the message it looks up by error code
    /// is not mirrored).
    UsageError { error: u32, item: u64, target: u64 },
    /// `CM_Character::SendNotice_UpdateVital`: one of the player's vitals
    /// ([`crate::vital`]), as `gmVitalsUI::RecvNotice_UpdateVital` takes it.
    UpdateVital { vital: u32, current: u64, max: u64 },
    /// `CM_Character::SendNotice_UpdateHeroStatus`: whether the player is a hero, with the
    /// focus the focus meter then shows.
    UpdateHeroStatus { hero: bool, current: u64, max: u64 },
    /// `CM_Character::SendNotice_UpdateSkill`: one skill changed (its enum).
    UpdateSkill(u32),
    /// `CM_Character::SendNotice_UpdateSkillPanel` (`CAgent` 0x3/0x2, `CPlayer` 0x85 on long
    /// 0x12D): the available experience (`Agent` 0x7), skill credits (`Agent` 0x2C) and hero
    /// skill credits (`Agent` 0xBE).
    UpdateSkillPanel {
        available_xp: u64,
        skill_credits: u32,
        hero_skill_credits: u32,
    },
    /// `CM_Shortcut::SendNotice_ReloadShortcuts`.
    ReloadShortcuts,
    /// `CM_Character::SendNotice_ReloadOptions`: the options profile (shortcuts included) was
    /// replaced.
    ReloadOptions,
    /// `CM_Fellowship::SendNotice_UpdateFellowship`: the whole fellowship.
    UpdateFellowship(FellowshipInfo),
    /// `CM_Fellowship::SendNotice_UpdateFellow`.
    UpdateFellow { fellow: u64, info: FellowInfo },
    /// `CM_Fellowship::SendNotice_ClearFellow`: a member left.
    ClearFellow(u64),
    /// `CM_Fellowship::SendNotice_ClearFellowship`: the player is in none.
    ClearFellowship,
    /// `CM_Fellowship::SendNotice_UpdateLeader`.
    UpdateLeader(u64),
    /// `CM_Fellowship::SendNotice_UpdateFellowHealth`: health and PK damage.
    UpdateFellowHealth { fellow: u64, value: u32, pk: u32 },
    /// `CM_Fellowship::SendNotice_UpdateFellowVigor`: vigor and PK vigor loss.
    UpdateFellowVigor { fellow: u64, value: u32, pk: u32 },
    /// `CM_Fellowship::SendNotice_UpdateFellowMaxHealth`.
    UpdateFellowMaxHealth { fellow: u64, value: u32 },
    /// `CM_Fellowship::SendNotice_UpdateFellowMaxVigor`.
    UpdateFellowMaxVigor { fellow: u64, value: u32 },
    /// `CM_Fellowship::SendNotice_UpdateFellowLevel`.
    UpdateFellowLevel { fellow: u64, level: u32 },
    /// `CM_Fellowship::SendNotice_UpdateFellowCellID`.
    UpdateFellowCell { fellow: u64, cell: u32 },
    /// `CM_Fellowship::SendNotice_RequestRecruitment`: an invitation.
    RequestRecruitment {
        leader: u64,
        leader_name: StringInfo,
        fellowship_name: StringInfo,
    },
    /// `CM_Fellowship::SendNotice_ExpireRecruitment`.
    ExpireRecruitment(u64),
    /// `CM_Allegiance::SendNotice_UpdateAllegianceProfile`.
    UpdateAllegianceProfile(Box<AllegianceProfile>),
    /// `CM_Allegiance::SendNotice_RequestVassal`: someone asks to swear to the player.
    RequestVassal { vassal: u64, name: StringInfo },
    /// `CM_Allegiance::SendNotice_Patron_SwearDone`.
    PatronSwearDone { vassal: u64, kind: u32 },
    /// `CM_Allegiance::SendNotice_Vassal_SwearDone`.
    VassalSwearDone { patron: u64, kind: u32 },
}
