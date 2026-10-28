//! The HUD frameworks' notice pass (`RecvNotice_*`): each frame's notices reach the panels in
//! the order the frameworks get them, and the shortcut bar takes its checks at the end.

use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::frame::ClientFrame;
use crate::hud_player::PassOut;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;
use ac2_ui::StatusBar;
use ac2_ui::{apply_shortcut_checks, shortcut_checks_of, ShortcutCheck};

impl HudView {
    /// The frame's notices, in the order the frameworks get them (`RecvNotice_*`), then the
    /// shortcut bar's checks (`gmShortcutUI`).
    pub(crate) fn notice_pass(
        &mut self,
        player: &ClientPlayer<'_>,
        frame: &ClientFrame,
        events: &[SessionEvent],
        mut reload_shortcuts: bool,
    ) -> PassOut {
        let HudView {
            hud,
            allegiance,
            backpack,
            catalog,
            consignment,
            damage,
            dialogs,
            dirty,
            effects,
            examination,
            external_container,
            fellowship,
            friends,
            item_selection,
            mini_fellow,
            money,
            net,
            now,
            paper_doll,
            pets,
            pk,
            quests,
            secure_trade,
            shop,
            shortcuts,
            skills,
            stack,
            toolbar,
            vitae,
            vitals,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let mut commands = Vec::new();
        let mut shortcut_checks: Vec<ShortcutCheck> = Vec::new();
        for event in events {
            if let Some(dialog) = stack.as_mut() {
                crate::ui_inventory::stack_event(dialog, tree, event);
            }
            if let Some(panel) = external_container.as_mut() {
                crate::ui_inventory::external_container_event(
                    panel,
                    tree,
                    player,
                    stack.as_mut(),
                    *item_selection,
                    event,
                );
            }
            // `gmNetStatusUI::OnHeartbeat`: the link averages the session closed this heartbeat.
            if let SessionEvent::LinkHeartbeat(links) = event {
                StatusBar {
                    vitals: &mut *vitals,
                    effects: &mut *effects,
                    vitae: &mut *vitae,
                    net: &mut *net,
                    pk: &mut *pk,
                }
                .link_heartbeat(tree, player, links);
                // The bandwidth graph moved: the status bar is composed again.
                *dirty = true;
                continue;
            }
            let SessionEvent::Notice(notice) = event else {
                continue;
            };
            if let (Some(panel), Some(dialogs)) = (secure_trade.as_mut(), dialogs.as_mut()) {
                crate::ui_inventory::secure_trade_notice(
                    panel,
                    tree,
                    dialogs,
                    player,
                    frame.server_time,
                    notice,
                );
            }
            if let (Some(panel), Some(dialogs)) = (shop.as_mut(), dialogs.as_mut()) {
                crate::hud_shop::shop_notice(
                    panel,
                    tree,
                    dialogs,
                    player,
                    frame.server_time,
                    notice,
                );
            }
            if let Some(panel) = catalog.as_mut() {
                crate::hud_shop::catalog_notice(panel, tree, player, frame.server_time, notice);
            }
            if let (Some(panel), Some(dialogs)) = (consignment.as_mut(), dialogs.as_mut()) {
                crate::hud_shop::consignment_notice(
                    panel,
                    tree,
                    dialogs,
                    player,
                    frame.server_time,
                    notice,
                );
            }
            if let Some(dialogs) = dialogs.as_mut() {
                ac2_ui::fellowship_notice(
                    fellowship.as_mut(),
                    mini_fellow.as_mut(),
                    tree,
                    dialogs,
                    player,
                    notice,
                );
            }
            if let Some(panel) = pets.as_mut() {
                ac2_ui::pet_notice(panel, tree, player, frame.server_time, notice);
            }
            if let Some(panel) = friends.as_mut() {
                crate::hud_friends::friends_notice(panel, tree, notice, &mut commands);
            }
            if let Some(panel) = quests.as_mut() {
                panel.recv_gameplay_notice(tree, player, notice);
            }
            if let (Some(page), Some(dialogs)) = (allegiance.as_mut(), dialogs.as_mut()) {
                ac2_ui::allegiance_notice(page, tree, dialogs, player, notice);
            }
            if let Some(doll) = paper_doll.as_mut() {
                let old = *item_selection;
                crate::ui_inventory::paper_doll_notice(doll, tree, player, old, notice);
            }
            if let Some(pack) = backpack.as_mut() {
                crate::ui_inventory::backpack_notice(pack, tree, player, item_selection, notice);
            }
            StatusBar {
                vitals: &mut *vitals,
                effects: &mut *effects,
                vitae: &mut *vitae,
                net: &mut *net,
                pk: &mut *pk,
            }
            .notice(tree, player, frame.server_time, notice);
            shortcut_checks.extend(shortcut_checks_of(notice));
            match notice {
                GameplayNotice::UpdateExaminationProfile(profile) => {
                    if let Some(panel) = examination.as_mut() {
                        // `ProcessEffectIDNode`: the effect nodes ask the player's script first.
                        let expanded = player
                            .client
                            .borrow_mut()
                            .expand_examination_effects(profile);
                        let items = player.examination_items(&expanded);
                        panel.update_profile(tree, player, frame.server_time, &items);
                    }
                }
                GameplayNotice::ExaminationRequestChanged(request) => {
                    if let Some(panel) = examination.as_mut() {
                        panel.request_changed(tree, request.kind);
                    }
                    if let Some(skills) = skills.as_mut() {
                        skills.examination_request_changed(
                            tree,
                            player,
                            request.kind,
                            request.data_enum,
                        );
                    }
                }
                GameplayNotice::UpdateSkillPanel { .. } => {
                    if let Some(skills) = skills.as_mut() {
                        skills.update_skill_panel(tree, player);
                    }
                }
                GameplayNotice::UpdateSkill(skill) => {
                    if let Some(skills) = skills.as_mut() {
                        skills.update_skill(tree, player, *skill);
                    }
                }
                GameplayNotice::ReloadShortcuts | GameplayNotice::ReloadOptions => {
                    reload_shortcuts = true;
                }
                // `gmShortcutUI::RecvNotice_ToggleSpecialAttack`, `RequestSpecialAttack`,
                // `StartSpecialAttack` and `EndSpecialAttack`.
                GameplayNotice::ToggleSpecialAttack { maneuver, toggled } => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.toggle_special_attack(tree, *maneuver, *toggled);
                    }
                }
                GameplayNotice::RequestSpecialAttack { maneuver, attack } => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.request_special_attack(tree, *attack, *maneuver);
                    }
                }
                GameplayNotice::StartSpecialAttack(attack) => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.start_special_attack(tree, *attack);
                    }
                }
                GameplayNotice::EndSpecialAttack(attack) => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.end_special_attack(tree, *attack);
                    }
                }
                GameplayNotice::UpdateMoney(amount) => {
                    if let Some(bag) = money.as_mut() {
                        bag.set_money(tree, player, *amount);
                    }
                }
                // `gmToolbarUI::RecvNotice_ShowSaleReminder` and `RecvNotice_UpdateMountState`.
                GameplayNotice::ShowSaleReminder(show) => {
                    if let Some(bar) = toolbar.as_ref() {
                        bar.sale_reminder(tree, *show);
                    }
                }
                GameplayNotice::UpdateMountState(on_mount) => {
                    if let Some(bar) = toolbar.as_ref() {
                        bar.mount_state(tree, *on_mount);
                    }
                }
                // `gmSelectionUI::RecvNotice_Select` (`target_select::frame_notices`).
                GameplayNotice::Select(_) => {}
                // `gm3DTextUI::RecvNotice_Do3DDamage`.
                GameplayNotice::Do3DDamage { object, text } => {
                    if let Some(damage) = damage.as_mut() {
                        damage.add(tree, *object, text, *now);
                    }
                }
                _ => {}
            }
        }
        if let Some(bar) = shortcuts.as_mut() {
            if reload_shortcuts {
                bar.reload(tree, player);
            } else {
                apply_shortcut_checks(bar, tree, player, &shortcut_checks);
            }
        }
        PassOut {
            commands,
            ..PassOut::default()
        }
    }
}
