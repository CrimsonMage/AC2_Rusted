//! The status bar's frameworks that take the player's notices: `gmVitalsUI` (the vital meters),
//! `gmEffectsUI` (the effect strip), `gmVitaeStatusUI` (the vitae indicator) and `gmNetStatusUI`
//! (the net indicator, fed by the link heartbeat).

use ac2_net::LinkAverages;
use ac2_ui::{EffectsUi, LinkSample, NetStatusUi, UiTree, VitaeStatusUi, VitalsUi};
use ac2_weenie::{GameplayNotice, PlayerVitals};

use crate::ui_inventory::effect_entries;
use crate::ui_sources::ClientPlayer;

/// The three frameworks (each present once the HUD is built).
pub(crate) struct StatusBar<'a> {
    pub vitals: &'a mut Option<VitalsUi>,
    pub effects: &'a mut Option<EffectsUi>,
    pub vitae: &'a mut Option<VitaeStatusUi>,
    pub net: &'a mut Option<NetStatusUi>,
}

impl StatusBar<'_> {
    /// What the frameworks take when the player first loads: the vitals the script raised so far
    /// and the tooltips, the effects (`gmEffectsUI::gmEffectsUI`'s `Init`) and the vitae.
    pub fn player_loaded(
        &mut self,
        tree: &mut UiTree,
        player: &ClientPlayer<'_>,
        vitals: &PlayerVitals,
        now: f64,
    ) {
        if let Some(bar) = self.vitals.as_mut() {
            let text = |cur, max| ac2_ui::vital_text(player, cur, max);
            bar.init_tooltips(tree, player);
            bar.apply_snapshot(tree, vitals, &text, player);
        }
        if let Some(strip) = self.effects.as_mut() {
            strip.init(tree, &effect_entries(player), now);
        }
        if let (Some(indicator), Some(v)) = (self.vitae.as_mut(), vitals.vitae) {
            indicator.update_vitae(tree, player, v.vitae, v.increment, v.cost);
        }
    }

    /// `gmNetStatusUI::OnHeartbeat`: one world heartbeat's link averages (`SessionEvent::LinkHeartbeat`).
    pub fn link_heartbeat(
        &mut self,
        tree: &mut UiTree,
        player: &ClientPlayer<'_>,
        links: &LinkAverages,
    ) {
        if let Some(net) = self.net.as_mut() {
            net.on_heartbeat(tree, player, LinkSample::from_link(links));
        }
    }

    /// One notice for the frameworks.
    pub fn notice(
        &mut self,
        tree: &mut UiTree,
        player: &ClientPlayer<'_>,
        now: f64,
        notice: &GameplayNotice,
    ) {
        if let Some(strip) = self.effects.as_mut() {
            match notice {
                GameplayNotice::UpdateEffect => {
                    strip.init(tree, &effect_entries(player), now);
                }
                GameplayNotice::PulseMissed(eid) => strip.pulse_missed(tree, *eid, now),
                GameplayNotice::ExaminationRequestChanged(request) => {
                    let list = effect_entries(player);
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
            let text = |cur, max| ac2_ui::vital_text(player, cur, max);
            match *notice {
                GameplayNotice::UpdateVital {
                    vital,
                    current,
                    max,
                } => bar.update_vital(tree, vital, current, max, &text),
                GameplayNotice::UpdateHeroStatus { hero, current, max } => {
                    bar.update_hero_status(tree, hero, current, max, &text, player);
                }
                _ => {}
            }
        }
        if let Some(indicator) = self.vitae.as_mut() {
            match *notice {
                GameplayNotice::UpdateVitae {
                    vitae,
                    increment,
                    cost,
                } => indicator.update_vitae(tree, player, vitae, increment, cost),
                GameplayNotice::ExaminationRequestChanged(ref request) => {
                    indicator.examination_request_changed(tree, request.kind);
                }
                _ => {}
            }
        }
    }
}
