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
        _ => return None,
    })
}
