//! The social panel's fellowship page (`gmFellowshipUI`) driven from the client: the player
//! answers its questions (`ClientPlayer`), the fellowship notices go to it, its requests become
//! gameplay commands (`gmCWInterface::DoFellowship*`, `SetSelection`), and the invitation is a
//! dialog whose answer comes back to it.

use ac2_ui::{AllegianceRequest, AllegianceSource, AllegianceUi};
use ac2_ui::{
    DialogFactory, DialogOutcome, Fellow, FellowshipData, FellowshipRequest, FellowshipUi,
};
use ac2_ui::{FellowStrings, FellowshipSource, SkillSource, SkillVar, UiTree};
use ac2_weenie::{FellowInfo, FellowshipInfo, GameplayNotice, SocialRequest};

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

impl FellowStrings for ClientPlayer<'_> {
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        SkillSource::render(self, string_id, vars)
    }
}

impl FellowshipSource for ClientPlayer<'_> {
    fn player(&self) -> u64 {
        self.frame.player.as_ref().map_or(0, |p| p.id)
    }

    /// `gmCWInterface::GetFellowXPHash`: the shares the player's script fills in.
    fn fellow_xp(&self) -> Option<Vec<(u64, f32)>> {
        self.client.borrow_mut().fellow_xp_hash().ok()
    }
}

impl AllegianceSource for ClientPlayer<'_> {
    fn player(&self) -> u64 {
        FellowshipSource::player(self)
    }

    fn render_wire(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.render_info(info)
    }
}

/// One notice for the allegiance page (`gmAllegianceUI`'s `RecvNotice_*`).
pub(crate) fn allegiance_notice(
    page: &mut AllegianceUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    player: &ClientPlayer<'_>,
    notice: &GameplayNotice,
) {
    use GameplayNotice as N;
    match notice {
        N::UpdateAllegianceProfile(profile) => page.update_profile(tree, player, profile),
        N::RequestVassal { name, .. } => page.request_vassal(tree, dialogs, player, name),
        N::PatronSwearDone { .. } => page.patron_swear_done(tree, dialogs),
        N::SelectionChanged(selection) => page.selection_changed(tree, *selection),
        _ => {}
    }
}

/// The allegiance page's requests as commands.
pub(crate) fn allegiance_commands(requests: Vec<AllegianceRequest>, commands: &mut Vec<Command>) {
    for request in requests {
        commands.push(Command::Social(match request {
            AllegianceRequest::Query(target) => SocialRequest::QueryAllegianceProfile(target),
            AllegianceRequest::Swear => SocialRequest::SwearAllegiance,
            AllegianceRequest::Break(target) => SocialRequest::BreakAllegiance(target),
            AllegianceRequest::AcceptVassal => SocialRequest::AcceptVassal,
            AllegianceRequest::DeclineVassal => SocialRequest::DeclineVassal(0x1585),
        }));
    }
}

impl ClientPlayer<'_> {
    /// A wire member as the panel's plain data, its name rendered.
    fn fellow(&self, id: u64, info: &FellowInfo) -> Fellow {
        Fellow {
            id,
            name: self.render_info(&info.name),
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

    fn fellowship(&self, info: &FellowshipInfo) -> FellowshipData {
        FellowshipData {
            name: info.name.clone(),
            leader: info.leader,
            social: info.social,
            looting: info.looting_method,
            fellows: info
                .fellows
                .iter()
                .map(|(id, f)| self.fellow(*id, f))
                .collect(),
        }
    }
}

/// One notice for the fellowship page (`gmFellowshipUI` / `FellowshipHelper` / `FellowInfo`'s
/// `RecvNotice_*`).
pub(crate) fn fellowship_notice(
    panel: &mut FellowshipUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    player: &ClientPlayer<'_>,
    notice: &GameplayNotice,
) {
    use GameplayNotice as N;
    match notice {
        N::UpdateFellowship(info) => {
            let data = player.fellowship(info);
            panel.update_fellowship(tree, player, &data);
        }
        N::ClearFellowship => panel.clear_fellowship(tree),
        N::UpdateFellow { fellow, info } => {
            let fellow = player.fellow(*fellow, info);
            panel.update_fellow(tree, player, &fellow);
        }
        N::ClearFellow(fellow) => panel.clear_fellow(tree, player, *fellow),
        N::UpdateLeader(leader) => panel.update_leader(tree, player, *leader),
        N::UpdateFellowHealth { fellow, value, pk } => {
            panel.update_health(tree, player, *fellow, *value, *pk);
        }
        N::UpdateFellowMaxHealth { fellow, value } => {
            panel.update_max_health(tree, player, *fellow, *value);
        }
        N::UpdateFellowVigor { fellow, value, pk } => {
            panel.update_vigor(tree, player, *fellow, *value, *pk);
        }
        N::UpdateFellowMaxVigor { fellow, value } => {
            panel.update_max_vigor(tree, player, *fellow, *value);
        }
        N::UpdateFellowLevel { fellow, level } => panel.update_level(tree, player, *fellow, *level),
        N::UpdateFellowCell { fellow, cell } => panel.update_cell(*fellow, *cell),
        N::SelectionChanged(selection) => {
            panel.selection_changed(tree, (*selection != 0).then_some(*selection));
        }
        N::RequestRecruitment {
            leader_name,
            fellowship_name,
            ..
        } => {
            let leader = player.render_info(leader_name);
            let name = player.render_info(fellowship_name);
            panel.request_recruitment(tree, dialogs, player, &leader, &name);
        }
        N::ExpireRecruitment(_) => panel.expire_recruitment(tree, dialogs),
        _ => {}
    }
}

/// The dialogs' answers the social pages wait for.
pub(crate) fn dialog_outcomes(
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

/// The page's requests as commands; an assisted tell's name is returned for the chat.
pub(crate) fn fellowship_commands(
    requests: Vec<FellowshipRequest>,
    selection: Option<u64>,
    commands: &mut Vec<Command>,
) -> Vec<String> {
    let mut tells = Vec::new();
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
                commands.push(Command::Select(Some(id)));
                continue;
            }
            FellowshipRequest::AssistedTell(name) => {
                tells.push(name);
                continue;
            }
        };
        commands.push(Command::Social(social));
    }
    tells
}
