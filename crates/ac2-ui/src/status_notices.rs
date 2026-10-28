//! The status bar's frameworks that take the player's notices: `gmVitalsUI` (the vital meters),
//! `gmEffectsUI` (the effect strip), `gmVitaeStatusUI` (the vitae indicator), `gmNetStatusUI`
//! (the net indicator, fed by the link heartbeat) and `gmPKStatusUI` (the faction status).
//! The player answers through `StatusSource`; the client delivers the session's events to them.

use ac2_net::LinkAverages;
use ac2_weenie::{GameplayNotice, PlayerVitals};

use crate::effects::{EffectEntry, EffectsUi};
use crate::element::UiTree;
use crate::fellow_info::{vital_text, FellowStrings};
use crate::net_status::{LinkSample, NetStatusUi};
use crate::pk_status::PkStatusUi;
use crate::vitae::VitaeStatusUi;
use crate::vitals::VitalsUi;

/// What the status bar reads from the player: its strings (the tooltips and the vital text) and
/// the effect list (`gmCWInterface::InqEffectList`).
pub trait StatusSource: FellowStrings {
    /// The player's effects in the order the script lists them.
    fn effects(&self) -> Vec<EffectEntry>;
}

/// The frameworks (each present once the HUD is built).
pub struct StatusBar<'a> {
    pub vitals: &'a mut Option<VitalsUi>,
    pub effects: &'a mut Option<EffectsUi>,
    pub vitae: &'a mut Option<VitaeStatusUi>,
    pub net: &'a mut Option<NetStatusUi>,
    pub pk: &'a mut Option<PkStatusUi>,
}

impl StatusBar<'_> {
    /// What the frameworks take when the player first loads: the vitals the script raised so far
    /// and the tooltips, the effects (`gmEffectsUI::gmEffectsUI`'s `Init`) and the vitae.
    pub fn player_loaded<S: StatusSource>(
        &mut self,
        tree: &mut UiTree,
        source: &S,
        vitals: &PlayerVitals,
        now: f64,
    ) {
        if let Some(bar) = self.vitals.as_mut() {
            let text = |cur, max| vital_text(source, cur, max);
            bar.init_tooltips(tree, source);
            bar.apply_snapshot(tree, vitals, &text, source);
        }
        if let Some(strip) = self.effects.as_mut() {
            strip.init(tree, &source.effects(), now);
        }
        if let (Some(indicator), Some(v)) = (self.vitae.as_mut(), vitals.vitae) {
            indicator.update_vitae(tree, source, v.vitae, v.increment, v.cost);
        }
    }

    /// `gmNetStatusUI::OnHeartbeat`: one world heartbeat's link averages (`SessionEvent::LinkHeartbeat`).
    pub fn link_heartbeat<S: StatusSource>(
        &mut self,
        tree: &mut UiTree,
        source: &S,
        links: &LinkAverages,
    ) {
        if let Some(net) = self.net.as_mut() {
            net.on_heartbeat(tree, source, LinkSample::from_link(links));
        }
    }

    /// One notice for the frameworks.
    pub fn notice<S: StatusSource>(
        &mut self,
        tree: &mut UiTree,
        source: &S,
        now: f64,
        notice: &GameplayNotice,
    ) {
        if let Some(strip) = self.effects.as_mut() {
            match notice {
                GameplayNotice::UpdateEffect => {
                    strip.init(tree, &source.effects(), now);
                }
                GameplayNotice::PulseMissed(eid) => strip.pulse_missed(tree, *eid, now),
                GameplayNotice::ExaminationRequestChanged(request) => {
                    let list = source.effects();
                    strip.examination_request_changed(
                        tree,
                        request.kind,
                        request.data_enum,
                        &list,
                        now,
                    );
                }
                _ => {}
            }
        }
        if let Some(bar) = self.vitals.as_mut() {
            let text = |cur, max| vital_text(source, cur, max);
            match *notice {
                GameplayNotice::UpdateVital {
                    vital,
                    current,
                    max,
                } => bar.update_vital(tree, vital, current, max, &text),
                GameplayNotice::UpdateHeroStatus { hero, current, max } => {
                    bar.update_hero_status(tree, hero, current, max, &text, source);
                }
                _ => {}
            }
        }
        pk_notice(self.pk, tree, notice);
        if let Some(indicator) = self.vitae.as_mut() {
            match *notice {
                GameplayNotice::UpdateVitae {
                    vitae,
                    increment,
                    cost,
                } => indicator.update_vitae(tree, source, vitae, increment, cost),
                GameplayNotice::ExaminationRequestChanged(ref request) => {
                    indicator.examination_request_changed(tree, request.kind);
                }
                _ => {}
            }
        }
    }
}

/// `gmPKStatusUI::RecvNotice_UpdateFactionStatus`: the server's faction status (the
/// `CM_Character::SendNotice_UpdateFactionStatus` notice) sets the PK indicator's state.
pub fn pk_notice(pk: &mut Option<PkStatusUi>, tree: &mut UiTree, notice: &GameplayNotice) {
    if let (Some(indicator), GameplayNotice::UpdateFactionStatus(status)) = (pk.as_mut(), notice) {
        indicator.update_faction_status(tree, *status);
    }
}
