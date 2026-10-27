//! The `CM_*` notices ac2-weenie has a [`GameplayNotice`] for, built from the generic
//! decoding (`notices::generic`): the skill panel's, the inventory's, the fellowship's and
//! the allegiance's. The adaptors (`FellowshipAdaptor`, `FellowAdaptor`,
//! `AllegianceProfileAdaptor`) are read from the VM, their hashes in retail order.

use ac2_net::protocol::{NativeObject as Wire, StringInfo};
use ac2_weenie::GameplayNotice as N;

use super::notices::NoticeArg;
use crate::native::{social, NativeObject};
use crate::vm::Vm;

fn long(args: &[NoticeArg], n: usize) -> Option<u64> {
    match args.get(n)? {
        NoticeArg::Long(v) => Some(*v),
        _ => None,
    }
}

fn int(args: &[NoticeArg], n: usize) -> Option<u32> {
    match args.get(n)? {
        NoticeArg::Int(v) => Some(*v),
        _ => None,
    }
}

fn flag(args: &[NoticeArg], n: usize) -> Option<bool> {
    int(args, n).map(|v| v != 0)
}

fn float(args: &[NoticeArg], n: usize) -> Option<f32> {
    match args.get(n)? {
        NoticeArg::Float(v) => Some(*v),
        _ => None,
    }
}

fn text(args: &[NoticeArg], n: usize) -> String {
    match args.get(n) {
        Some(NoticeArg::Text(s)) => s.clone(),
        _ => String::new(),
    }
}

fn info(args: &[NoticeArg], n: usize) -> StringInfo {
    match args.get(n) {
        Some(NoticeArg::Native(Wire::StringInfo(s))) => s.clone(),
        _ => StringInfo::default(),
    }
}

/// `name`'s typed notice (`object` the weenie it is raised on, `args` the decoded arguments,
/// `raw` each argument's first frame word), or `None` to raise it as a `ScriptNotice`.
pub(super) fn typed(
    vm: &Vm,
    name: &str,
    object: u64,
    args: &[NoticeArg],
    raw: &[u32],
) -> Option<N> {
    let short = name.split_once("::SendNotice_").map_or(name, |(_, n)| n);
    Some(match short {
        "UpdateSkill" => N::UpdateSkill(int(args, 0)?),
        "UpdateSkillPanel" => N::UpdateSkillPanel {
            available_xp: long(args, 0)?,
            skill_credits: int(args, 1)?,
            hero_skill_credits: int(args, 2)?,
        },
        "ReloadOptions" => N::ReloadOptions,
        "ReloadShortcuts" => N::ReloadShortcuts,
        "ContainerStateUpdate" => N::ContainerStateUpdate {
            container: long(args, 0)?,
            open: int(args, 1)? != 0,
        },
        "ItemUpdate" => N::ItemUpdate(object),
        "UpdateFellowship" => match vm.native(*raw.first()?)? {
            NativeObject::Fellowship(f) => N::UpdateFellowship(social::fellowship_ordered(f)),
            _ => return None,
        },
        "ClearFellowship" => N::ClearFellowship,
        "UpdateFellow" => match vm.native(*raw.get(1)?)? {
            NativeObject::Fellow(f) => N::UpdateFellow {
                fellow: long(args, 0)?,
                info: (**f).clone(),
            },
            _ => return None,
        },
        "ClearFellow" => N::ClearFellow(long(args, 0)?),
        "UpdateLeader" => N::UpdateLeader(long(args, 0)?),
        "ExpireRecruitment" => N::ExpireRecruitment(long(args, 0)?),
        "UpdateFellowLevel" => N::UpdateFellowLevel {
            fellow: long(args, 0)?,
            level: int(args, 1)?,
        },
        "UpdateFellowCellID" => N::UpdateFellowCell {
            fellow: long(args, 0)?,
            cell: int(args, 1)?,
        },
        "UpdateFellowHealth" => N::UpdateFellowHealth {
            fellow: long(args, 0)?,
            value: int(args, 1)?,
            pk: int(args, 2)?,
        },
        "UpdateFellowVigor" => N::UpdateFellowVigor {
            fellow: long(args, 0)?,
            value: int(args, 1)?,
            pk: int(args, 2)?,
        },
        "UpdateFellowMaxHealth" => N::UpdateFellowMaxHealth {
            fellow: long(args, 0)?,
            value: int(args, 1)?,
        },
        "UpdateFellowMaxVigor" => N::UpdateFellowMaxVigor {
            fellow: long(args, 0)?,
            value: int(args, 1)?,
        },
        "RequestRecruitment" => N::RequestRecruitment {
            leader: long(args, 0)?,
            leader_name: info(args, 1),
            fellowship_name: info(args, 2),
        },
        "UpdateAllegianceProfile" => match vm.native(*raw.first()?)? {
            NativeObject::AllegianceProfile(p) => {
                N::UpdateAllegianceProfile(Box::new(social::allegiance_ordered(p)))
            }
            _ => return None,
        },
        "RequestVassal" => N::RequestVassal {
            vassal: long(args, 0)?,
            name: info(args, 1),
        },
        "Patron_SwearDone" => N::PatronSwearDone {
            vassal: long(args, 0)?,
            kind: int(args, 1)?,
        },
        "Vassal_SwearDone" => N::VassalSwearDone {
            patron: long(args, 0)?,
            kind: int(args, 1)?,
        },
        "UpdateQuests" => {
            let code = int(args, 0)?;
            let quest = match args.get(1)? {
                NoticeArg::Native(Wire::GmQuestInfo(q)) => q.clone(),
                // The list notice (code 2, `LIST`) names no quest: the journal asks the script
                // for the list (`gmCWInterface::InqQuestList`) and reads no quest from it.
                NoticeArg::Null if code == 0x4000_0002 => Box::default(),
                _ => return None,
            };
            N::UpdateQuests { code, quest }
        }
        "UpdateStoryQuests" => N::UpdateStoryQuests {
            code: int(args, 0)?,
            act: int(args, 1)?,
        },
        "UpdateEffect" => N::UpdateEffect,
        "UpdateMoney" => N::UpdateMoney(int(args, 0)?),
        "GhostMoneyBag" => N::GhostMoneyBag(flag(args, 0)?),
        "UpdateVitae" => N::UpdateVitae {
            vitae: float(args, 0)?,
            increment: float(args, 1)?,
            cost: float(args, 2)?,
        },
        "UpdateFactionStatus" => N::UpdateFactionStatus(int(args, 0)?),
        "UpdateSelectability" => N::UpdateSelectability {
            object,
            selectable: flag(args, 0)?,
        },
        "UpdateResetTime" => N::UpdateResetTime,
        "UpdateDeathState" => N::UpdateDeathState(flag(args, 0)?),
        "ChangeRadarLook" => N::ChangeRadarLook(object),
        "DisplayMessage" => N::DisplayMessage {
            text: info(args, 0),
            topmost: flag(args, 1)?,
        },
        "UpdateMountState" => N::UpdateMountState(flag(args, 0)?),
        "ResurrectionRequestedByOther" => N::ResurrectionRequestedByOther(info(args, 0)),
        "PulseMissed" => N::PulseMissed(int(args, 0)?),
        "AddMarker" => N::AddMarker {
            kind: int(args, 0)?,
            id: long(args, 1)?,
            cell: int(args, 2)?,
            image: int(args, 3)?,
            message: info(args, 4),
        },
        "ClearMarkers" => N::ClearMarkers(int(args, 0)?),
        "RemoveMarker" => N::RemoveMarker {
            kind: int(args, 0)?,
            id: long(args, 1)?,
        },
        "AddPet" => N::AddPet {
            pet: long(args, 0)?,
            name: info(args, 1),
            icon: int(args, 2)?,
            mode: int(args, 3)?,
        },
        "RemovePet" => N::RemovePet(long(args, 0)?),
        "UpdatePetMode" => N::UpdatePetMode {
            pet: long(args, 0)?,
            mode: int(args, 1)?,
        },
        "UpdatePetName" => N::UpdatePetName {
            pet: long(args, 0)?,
            name: info(args, 1),
        },
        "PetIsUnderAttack" => N::PetIsUnderAttack(long(args, 0)?),
        "UpdateAttackState" => N::UpdateAttackState(flag(args, 0)?),
        "RequestSpecialAttack" => N::RequestSpecialAttack {
            maneuver: int(args, 0)?,
            attack: int(args, 1)?,
        },
        "StartSpecialAttack" => N::StartSpecialAttack(int(args, 0)?),
        "EndSpecialAttack" => N::EndSpecialAttack(int(args, 0)?),
        // `bool _toggled`: the argument's low byte.
        "ToggleSpecialAttack" => N::ToggleSpecialAttack {
            maneuver: int(args, 0)?,
            toggled: int(args, 1)? & 0xFF != 0,
        },
        "DoAssistedTell" => N::DoAssistedTell(text(args, 0)),
        "ReloadChatSettings" => N::ReloadChatSettings,
        "UpdateCraftPanel" => N::UpdateCraftPanel,
        "AddIngredient_Done" => N::AddIngredientDone {
            recipe: int(args, 0)?,
            ordinal: int(args, 1)?,
            ingredient: long(args, 2)?,
            spinner: int(args, 3)?,
            error: int(args, 4)?,
        },
        "ExecuteCraft_Done" => N::ExecuteCraftDone {
            recipe: int(args, 0)?,
            error: int(args, 1)?,
        },
        "UpdateCost" => N::UpdateCraftCost {
            recipe: int(args, 0)?,
            cost: int(args, 1)? as i32,
            request: int(args, 2)?,
        },
        "OpenIngredientPanel" => N::OpenIngredientPanel(int(args, 0)?),
        "Trade_BeRegistered" => N::TradeRegistered {
            initiator: long(args, 0)?,
            partner: long(args, 1)?,
            partner_name: info(args, 2),
        },
        "Trade_BeOpened" => N::TradeOpened,
        "Trade_BeClosed" => N::TradeClosed {
            source: long(args, 0)?,
            kind: int(args, 1)?,
        },
        "Trade_BeAccepted" => N::TradeAccepted(long(args, 0)?),
        "Trade_BeDeclined" => N::TradeDeclined(long(args, 0)?),
        "Trade_BeReset" => N::TradeReset,
        "Trade_BeAcceptanceCleared" => N::TradeAcceptanceCleared,
        "Trade_BeOffered" => N::TradeOffered {
            source: long(args, 0)?,
            item: long(args, 1)?,
            amount: int(args, 2)?,
        },
        "Trade_BeRevoked" => N::TradeRevoked {
            source: long(args, 0)?,
            item: long(args, 1)?,
        },
        "PortalSpace" => N::PortalSpace(flag(args, 0)?),
        "OpenExaminationPanel" => N::OpenExaminationPanel,
        "SetTargetingPointer" => N::SetTargetingPointer(flag(args, 0)?),
        "UseBook" => N::UseBook {
            show_controls: flag(args, 0)?,
            mode: int(args, 1)?,
            title: text(args, 2),
            text: text(args, 3),
            image: int(args, 4)?,
        },
        "GUNResult_AddFriend" => N::FriendAdded(text(args, 0)),
        "GUNResult_RemoveFriend" => N::FriendRemoved(text(args, 0)),
        // The friends list: the script's `RList` of `TurbineUserStatus` natives, in order.
        "GUNResult_GetFriendsList" => N::FriendsList(match vm.native(*raw.first()?)? {
            NativeObject::List(l) => l
                .items
                .iter()
                .filter_map(|r| friend(vm, *r as u32))
                .collect(),
            _ => return None,
        }),
        "GUNResult_FriendStatus" => N::FriendStatus(friend(vm, *raw.first()?)?),
        "LocalStatus" => N::LocalStatus(int(args, 0)?),
        "UpdateSale" => N::UpdateSale {
            item: long(args, 0)?,
            stock: int(args, 1)? as i32,
        },
        "LeaveStore" => N::LeaveStore,
        "ConfirmPurchase" => N::ConfirmPurchase {
            product: info(args, 0),
            quantity: int(args, 1)?,
            trade_name: info(args, 2),
            cost: int(args, 3)?,
        },
        "LeaveCatalog" => N::LeaveCatalog,
        "LeaveConsignment" => N::LeaveConsignment,
        "ShowSaleReminder" => N::ShowSaleReminder(int(args, 0)? & 0xFF != 0),
        "UpdateStore" => N::UpdateStore(Box::new(shop(vm, *raw.first()?)?)),
        "ViewStore" => N::ViewStore(Box::new(shop(vm, *raw.first()?)?)),
        "ViewConsignment" => N::ViewConsignment(Box::new(shop(vm, *raw.first()?)?)),
        "UpdateConsignment" => N::UpdateConsignment(Box::new(shop(vm, *raw.first()?)?)),
        // `RListToUIShopList`: the script's list of `UIShop`s, in order.
        "ViewCatalog" => N::ViewCatalog {
            header: info(args, 0),
            shops: match vm.native(*raw.get(1)?)? {
                NativeObject::List(l) => {
                    l.items.iter().filter_map(|r| shop(vm, *r as u32)).collect()
                }
                _ => return None,
            },
        },
        "UpdateCraftSkill" => match vm.native(*raw.first()?)? {
            NativeObject::CraftSkill(s) => N::UpdateCraftSkill(s.clone()),
            _ => return None,
        },
        "Do3DDamage" => match vm.native(*raw.first()?)? {
            NativeObject::DamageText(d) => N::Do3DDamage {
                object,
                text: (**d).clone(),
            },
            _ => return None,
        },
        _ => return None,
    })
}

fn friend(vm: &Vm, reference: u32) -> Option<ac2_weenie::FriendInfo> {
    match vm.native(reference)? {
        NativeObject::TurbineUserStatus(f) => Some((**f).clone()),
        _ => None,
    }
}

fn shop(vm: &Vm, reference: u32) -> Option<ac2_weenie::Shop> {
    match vm.native(reference)? {
        NativeObject::Shop(s) => Some((**s).clone()),
        _ => None,
    }
}
