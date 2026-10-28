//! The HUD frameworks' notice pass (`RecvNotice_*`): each frame's notices reach the panels in
//! the order the frameworks get them, and the shortcut bar takes its checks at the end. The
//! panels are the HUD's ([`Hud`]); the client passes its tree and the player, and turns the
//! friends requests this returns into its commands.

use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::examination_effects::expand_examination_effects;
use crate::{
    apply_shortcut_checks, shortcut_checks_of, FriendsRequest, Hud, HudSource, ShortcutCheck,
    StatusBar, UiTree,
};

/// Each of the frame's notices, in the order the frameworks get them (`RecvNotice_*`), then the
/// shortcut bar's checks (`gmShortcutUI`); `reload_shortcuts` reloads the bar instead of
/// checking it. Returns the friends requests the page raised, in order.
pub fn notice_pass<P: HudSource>(
    hud: &mut Hud,
    tree: &mut UiTree,
    dirty: &mut bool,
    player: &P,
    events: &[SessionEvent],
    server_time: f64,
    mut reload_shortcuts: bool,
) -> Vec<FriendsRequest> {
    let now = hud.now();
    let mut requests = Vec::new();
    let mut shortcut_checks: Vec<ShortcutCheck> = Vec::new();
    for event in events {
        if let Some(dialog) = hud.stack.as_mut() {
            crate::stack_event(dialog, tree, event);
        }
        if let Some(panel) = hud.external_container.as_mut() {
            crate::external_container_event(
                panel,
                tree,
                player,
                hud.stack.as_mut(),
                hud.item_selection,
                event,
            );
        }
        // `gmNetStatusUI::OnHeartbeat`: the link averages the session closed this heartbeat.
        if let SessionEvent::LinkHeartbeat(links) = event {
            StatusBar {
                vitals: &mut hud.vitals,
                effects: &mut hud.effects,
                vitae: &mut hud.vitae,
                net: &mut hud.net,
                pk: &mut hud.pk,
            }
            .link_heartbeat(tree, player, links);
            // The bandwidth graph moved: the status bar is composed again.
            *dirty = true;
            continue;
        }
        let SessionEvent::Notice(notice) = event else {
            continue;
        };
        if let (Some(panel), Some(dialogs)) = (hud.secure_trade.as_mut(), hud.dialogs.as_mut()) {
            crate::secure_trade_notice(panel, tree, dialogs, player, server_time, notice);
        }
        if let (Some(panel), Some(dialogs)) = (hud.shop.as_mut(), hud.dialogs.as_mut()) {
            crate::shop_notice(panel, tree, dialogs, player, server_time, notice);
        }
        if let Some(panel) = hud.catalog.as_mut() {
            crate::catalog_notice(panel, tree, player, server_time, notice);
        }
        if let (Some(panel), Some(dialogs)) = (hud.consignment.as_mut(), hud.dialogs.as_mut()) {
            crate::consignment_notice(panel, tree, dialogs, player, server_time, notice);
        }
        if let Some(dialogs) = hud.dialogs.as_mut() {
            crate::fellowship_notice(
                hud.fellowship.as_mut(),
                hud.mini_fellow.as_mut(),
                tree,
                dialogs,
                player,
                notice,
            );
        }
        if let Some(panel) = hud.pets.as_mut() {
            crate::pet_notice(panel, tree, player, server_time, notice);
        }
        if let Some(panel) = hud.friends.as_mut() {
            // `gmFriendsUI`: an added friend's request for the list again.
            if let Some(request) = panel.notice(tree, notice) {
                requests.push(request);
            }
        }
        if let Some(panel) = hud.quests.as_mut() {
            panel.recv_gameplay_notice(tree, player, notice);
        }
        if let (Some(page), Some(dialogs)) = (hud.allegiance.as_mut(), hud.dialogs.as_mut()) {
            crate::allegiance_notice(page, tree, dialogs, player, notice);
        }
        if let Some(doll) = hud.paper_doll.as_mut() {
            let old = hud.item_selection;
            crate::paper_doll_notice(doll, tree, player, old, notice);
        }
        if let Some(pack) = hud.backpack.as_mut() {
            crate::backpack_notice(pack, tree, player, &mut hud.item_selection, notice);
        }
        StatusBar {
            vitals: &mut hud.vitals,
            effects: &mut hud.effects,
            vitae: &mut hud.vitae,
            net: &mut hud.net,
            pk: &mut hud.pk,
        }
        .notice(tree, player, server_time, notice);
        shortcut_checks.extend(shortcut_checks_of(notice));
        match notice {
            GameplayNotice::UpdateExaminationProfile(profile) => {
                if let Some(panel) = hud.examination.as_mut() {
                    // `ProcessEffectIDNode`: the effect nodes ask the player's script first.
                    let expanded = expand_examination_effects(player, profile);
                    let items = player.examination_items(&expanded);
                    panel.update_profile(
                        tree,
                        player,
                        server_time,
                        &items,
                        (profile.request.extra[2], profile.available),
                    );
                }
            }
            GameplayNotice::ExaminationRequestChanged(request) => {
                if let Some(panel) = hud.examination.as_mut() {
                    panel.request_changed(tree, request.kind, request.extra[2]);
                }
                if let Some(skills) = hud.skills.as_mut() {
                    skills.examination_request_changed(
                        tree,
                        player,
                        request.kind,
                        request.data_enum,
                    );
                }
            }
            GameplayNotice::UpdateSkillPanel { .. } => {
                if let Some(skills) = hud.skills.as_mut() {
                    skills.update_skill_panel(tree, player);
                }
            }
            GameplayNotice::UpdateSkill(skill) => {
                if let Some(skills) = hud.skills.as_mut() {
                    skills.update_skill(tree, player, *skill);
                }
            }
            GameplayNotice::ReloadShortcuts | GameplayNotice::ReloadOptions => {
                reload_shortcuts = true;
            }
            // `gmShortcutUI::RecvNotice_ToggleSpecialAttack`, `RequestSpecialAttack`,
            // `StartSpecialAttack` and `EndSpecialAttack`.
            GameplayNotice::ToggleSpecialAttack { maneuver, toggled } => {
                if let Some(bar) = hud.shortcuts.as_mut() {
                    bar.toggle_special_attack(tree, *maneuver, *toggled);
                }
            }
            GameplayNotice::RequestSpecialAttack { maneuver, attack } => {
                if let Some(bar) = hud.shortcuts.as_mut() {
                    bar.request_special_attack(tree, *attack, *maneuver);
                }
            }
            GameplayNotice::StartSpecialAttack(attack) => {
                if let Some(bar) = hud.shortcuts.as_mut() {
                    bar.start_special_attack(tree, *attack);
                }
            }
            GameplayNotice::EndSpecialAttack(attack) => {
                if let Some(bar) = hud.shortcuts.as_mut() {
                    bar.end_special_attack(tree, *attack);
                }
            }
            GameplayNotice::UpdateMoney(amount) => {
                if let Some(bag) = hud.money.as_mut() {
                    bag.set_money(tree, player, *amount);
                }
            }
            // `gmToolbarUI::RecvNotice_ShowSaleReminder` and `RecvNotice_UpdateMountState`.
            GameplayNotice::ShowSaleReminder(show) => {
                if let Some(bar) = hud.toolbar.as_ref() {
                    bar.sale_reminder(tree, *show);
                }
            }
            GameplayNotice::UpdateMountState(on_mount) => {
                if let Some(bar) = hud.toolbar.as_ref() {
                    bar.mount_state(tree, *on_mount);
                }
            }
            // `gmSelectionUI::RecvNotice_Select` (the client's `target_select::frame_notices`).
            GameplayNotice::Select(_) => {}
            // `gm3DTextUI::RecvNotice_Do3DDamage`.
            GameplayNotice::Do3DDamage { object, text } => {
                if let Some(damage) = hud.damage_mut() {
                    damage.add(tree, *object, text, now);
                }
            }
            _ => {}
        }
    }
    if let Some(bar) = hud.shortcuts.as_mut() {
        if reload_shortcuts {
            bar.reload(tree, player);
        } else {
            apply_shortcut_checks(bar, tree, player, &shortcut_checks);
        }
    }
    requests
}
