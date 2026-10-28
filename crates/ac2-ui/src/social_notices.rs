//! The fellowship and allegiance pages' gameplay notices (`gmFellowshipUI`, `gmMiniFellowUI`,
//! `FellowshipHelper` / `FellowInfo`'s `RecvNotice_*`, `gmAllegianceUI`), the dialogs' answers
//! they wait for, and the requests the pages queue as gameplay calls
//! (`gmCWInterface::DoFellowship*`, `DoAssistedTell`, `SetSelection`). The player answers the
//! questions through `SocialSource`; the client turns `SocialCall`s into its commands.

use ac2_weenie::{FellowInfo, FellowshipInfo, GameplayNotice, SocialRequest};

use crate::allegiance::{AllegianceRequest, AllegianceSource, AllegianceUi};
use crate::dialog::{DialogFactory, DialogOutcome};
use crate::element::UiTree;
use crate::fellow_info::Fellow;
use crate::fellow_rows::FellowNotice;
use crate::fellowship::{FellowshipData, FellowshipRequest, FellowshipSource, FellowshipUi};
use crate::mini_fellow::MiniFellowUi;

/// What the social pages read from the player: the fellowship's and the allegiance's answers.
pub trait SocialSource: FellowshipSource + AllegianceSource {}

impl<T: FellowshipSource + AllegianceSource> SocialSource for T {}

/// One gameplay call a social page asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocialCall {
    /// A `CPlayer` call (`SocialRequest`).
    Social(SocialRequest),
    /// `SetSelection`.
    Select(u64),
    /// `DoAssistedTell(name)`: the chat's tell, not a gameplay call.
    AssistedTell(String),
}

/// A wire member as the fellowship panel's plain data, its name rendered.
fn fellow<S: AllegianceSource>(source: &S, id: u64, info: &FellowInfo) -> Fellow {
    Fellow {
        id,
        name: source.render_wire(&info.name),
        level: info.level,
        health: info.health,
        max_health: info.max_health,
        vigor: info.vigor,
        max_vigor: info.max_vigor,
        pk_damage: info.pk_damage,
        pk_vigor_loss: info.pk_vigor_loss,
        cell: info.cell,
    }
}

fn fellowship<S: AllegianceSource>(source: &S, info: &FellowshipInfo) -> FellowshipData {
    FellowshipData {
        name: info.name.clone(),
        leader: info.leader,
        social: info.social,
        looting: info.looting_method,
        fellows: info
            .fellows
            .iter()
            .map(|(id, f)| fellow(source, *id, f))
            .collect(),
    }
}

/// A member-level notice, both lists' (`FellowshipHelper::RecvNotice_*`).
fn member_notice<S: AllegianceSource>(source: &S, notice: &GameplayNotice) -> Option<FellowNotice> {
    use GameplayNotice as N;
    Some(match notice {
        N::UpdateFellow { fellow: id, info } => FellowNotice::Update(fellow(source, *id, info)),
        N::ClearFellow(id) => FellowNotice::Clear(*id),
        N::UpdateLeader(leader) => FellowNotice::Leader(*leader),
        N::UpdateFellowHealth { fellow, value, pk } => FellowNotice::Health {
            fellow: *fellow,
            value: *value,
            pk: *pk,
        },
        N::UpdateFellowMaxHealth { fellow, value } => FellowNotice::MaxHealth {
            fellow: *fellow,
            value: *value,
        },
        N::UpdateFellowVigor { fellow, value, pk } => FellowNotice::Vigor {
            fellow: *fellow,
            value: *value,
            pk: *pk,
        },
        N::UpdateFellowMaxVigor { fellow, value } => FellowNotice::MaxVigor {
            fellow: *fellow,
            value: *value,
        },
        N::UpdateFellowLevel { fellow, level } => FellowNotice::Level {
            fellow: *fellow,
            level: *level,
        },
        N::UpdateFellowCell { fellow, cell } => FellowNotice::Cell {
            fellow: *fellow,
            cell: *cell,
        },
        N::SelectionChanged(selection) => {
            FellowNotice::Selection((*selection != 0).then_some(*selection))
        }
        _ => return None,
    })
}

/// One notice for the fellowship page and the mini panel (`gmFellowshipUI` / `gmMiniFellowUI` /
/// `FellowshipHelper` / `FellowInfo`'s `RecvNotice_*`).
pub fn fellowship_notice<S: SocialSource>(
    panel: Option<&mut FellowshipUi>,
    mini: Option<&mut MiniFellowUi>,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    source: &S,
    notice: &GameplayNotice,
) {
    use GameplayNotice as N;
    if let Some(member) = member_notice(source, notice) {
        if let Some(panel) = panel {
            panel.fellow_notice(tree, source, &member);
        }
        if let Some(mini) = mini {
            mini.fellow_notice(tree, source, &member);
        }
        return;
    }
    match notice {
        N::UpdateFellowship(info) => {
            let data = fellowship(source, info);
            if let Some(panel) = panel {
                panel.update_fellowship(tree, source, &data);
            }
            if let Some(mini) = mini {
                mini.update_fellowship(tree, source, &data);
            }
        }
        N::ClearFellowship => {
            if let Some(panel) = panel {
                panel.clear_fellowship(tree);
            }
            if let Some(mini) = mini {
                mini.clear_fellowship(tree);
            }
        }
        N::RequestRecruitment {
            leader_name,
            fellowship_name,
            ..
        } => {
            let leader = source.render_wire(leader_name);
            let name = source.render_wire(fellowship_name);
            if let Some(panel) = panel {
                panel.request_recruitment(tree, dialogs, source, &leader, &name);
            }
        }
        N::ExpireRecruitment(_) => {
            if let Some(panel) = panel {
                panel.expire_recruitment(tree, dialogs);
            }
        }
        _ => {}
    }
}

/// One notice for the allegiance page (`gmAllegianceUI`'s `RecvNotice_*`).
pub fn allegiance_notice<S: SocialSource>(
    page: &mut AllegianceUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    source: &S,
    notice: &GameplayNotice,
) {
    use GameplayNotice as N;
    match notice {
        N::UpdateAllegianceProfile(profile) => page.update_profile(tree, source, profile),
        N::RequestVassal { name, .. } => page.request_vassal(tree, dialogs, source, name),
        N::PatronSwearDone { .. } => page.patron_swear_done(tree, dialogs),
        N::SelectionChanged(selection) => page.selection_changed(tree, *selection),
        _ => {}
    }
}

/// The dialogs' answers the social pages wait for.
pub fn social_dialog_outcomes(
    tree: &mut UiTree,
    fellowship: Option<&mut FellowshipUi>,
    allegiance: Option<&mut AllegianceUi>,
    outcomes: &[DialogOutcome],
) {
    let (mut fellowship, mut allegiance) = (fellowship, allegiance);
    for outcome in outcomes {
        if let DialogOutcome::Confirmation { context, yes } = outcome {
            if let Some(panel) = fellowship.as_deref_mut() {
                panel.close_confirmation(*context, *yes);
            }
            if let Some(page) = allegiance.as_deref_mut() {
                page.close_confirmation(tree, *context, *yes);
            }
        }
    }
}

/// The allegiance page's requests as calls.
pub fn allegiance_calls(requests: Vec<AllegianceRequest>) -> Vec<SocialCall> {
    requests
        .into_iter()
        .map(|request| {
            SocialCall::Social(match request {
                AllegianceRequest::Query(target) => SocialRequest::QueryAllegianceProfile(target),
                AllegianceRequest::Swear => SocialRequest::SwearAllegiance,
                AllegianceRequest::Break(target) => SocialRequest::BreakAllegiance(target),
                AllegianceRequest::AcceptVassal => SocialRequest::AcceptVassal,
                AllegianceRequest::DeclineVassal => SocialRequest::DeclineVassal(0x1585),
            })
        })
        .collect()
}

/// The fellowship page's requests as calls. Recruit and dismiss need a selection.
pub fn fellowship_calls(
    requests: Vec<FellowshipRequest>,
    selection: Option<u64>,
) -> Vec<SocialCall> {
    let mut calls = Vec::new();
    for request in requests {
        let social = match request {
            FellowshipRequest::Create {
                name,
                social,
                looting,
            } => SocialRequest::FellowshipCreate {
                name,
                social: u32::from(social),
                looting: looting as u32,
            },
            FellowshipRequest::Leave => SocialRequest::FellowshipLeave,
            FellowshipRequest::Disband => SocialRequest::FellowshipDisband,
            // `HandleRecruitButton` / `HandleDismissButton`: only with a selection.
            FellowshipRequest::Recruit | FellowshipRequest::Dismiss => {
                let Some(target) = selection else {
                    continue;
                };
                if matches!(request, FellowshipRequest::Recruit) {
                    SocialRequest::FellowshipRecruit(target)
                } else {
                    SocialRequest::FellowshipDismiss(target)
                }
            }
            FellowshipRequest::RecruitAccept => SocialRequest::FellowshipRecruitAccept,
            FellowshipRequest::RecruitDecline => SocialRequest::FellowshipRecruitDecline(0x138D),
            FellowshipRequest::Select(id) => {
                calls.push(SocialCall::Select(id));
                continue;
            }
            FellowshipRequest::AssistedTell(name) => {
                calls.push(SocialCall::AssistedTell(name));
                continue;
            }
        };
        calls.push(SocialCall::Social(social));
    }
    calls
}
