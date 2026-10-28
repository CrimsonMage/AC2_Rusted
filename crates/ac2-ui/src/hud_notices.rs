//! The HUD frameworks' notice pass (`RecvNotice_*`): each frame's notices reach the panels in
//! the order the frameworks get them, and the shortcut bar takes its checks at the end. The
//! client owns the panels and the player; it passes them in as [`NoticePanels`] and turns the
//! friends requests this returns into its commands.

use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::examination_effects::expand_examination_effects;
use crate::{
    apply_shortcut_checks, shortcut_checks_of, AllegianceUi, BackpackUi, CatalogUi, ConsignmentUi,
    DamageTexts, DialogFactory, EffectsUi, ExaminationUi, ExternalContainerUi, FellowshipUi,
    FriendsPanel, FriendsRequest, HudSource, MiniFellowUi, MoneyBagUi, NetStatusUi, PaperDollUi,
    PetUi, PkStatusUi, QuestsUi, SecureTradeUi, ShopUi, ShortcutCheck, ShortcutUi, SkillTreeUi,
    StackUi, StatusBar, ToolbarUi, UiTree, VitaeStatusUi, VitalsUi,
};

/// The panels the notice pass feeds, borrowed from the client's HUD for one frame. A panel that
/// is not made yet is `None` and gets nothing.
pub struct NoticePanels<'a> {
    pub tree: &'a mut UiTree,
    /// The HUD's clock at this frame (a damage text starts at it).
    pub now: f64,
    /// Set when the status bar is composed again (the link heartbeat moved its graph).
    pub dirty: &'a mut bool,
    /// The selection the item slots last saw.
    pub item_selection: &'a mut u64,
    pub vitals: &'a mut Option<VitalsUi>,
    pub effects: &'a mut Option<EffectsUi>,
    pub vitae: &'a mut Option<VitaeStatusUi>,
    pub net: &'a mut Option<NetStatusUi>,
    pub pk: &'a mut Option<PkStatusUi>,
    pub stack: &'a mut Option<StackUi>,
    pub external_container: &'a mut Option<ExternalContainerUi>,
    pub secure_trade: &'a mut Option<SecureTradeUi>,
    pub dialogs: &'a mut Option<DialogFactory>,
    pub shop: &'a mut Option<ShopUi>,
    pub catalog: &'a mut Option<CatalogUi>,
    pub consignment: &'a mut Option<ConsignmentUi>,
    pub fellowship: &'a mut Option<FellowshipUi>,
    pub mini_fellow: &'a mut Option<MiniFellowUi>,
    pub pets: &'a mut Option<PetUi>,
    pub friends: &'a mut Option<FriendsPanel>,
    pub quests: &'a mut Option<QuestsUi>,
    pub allegiance: &'a mut Option<AllegianceUi>,
    pub paper_doll: &'a mut Option<PaperDollUi>,
    pub backpack: &'a mut Option<BackpackUi>,
    pub examination: &'a mut Option<ExaminationUi>,
    pub skills: &'a mut Option<SkillTreeUi>,
    pub shortcuts: &'a mut Option<ShortcutUi>,
    pub money: &'a mut Option<MoneyBagUi>,
    pub toolbar: &'a mut Option<ToolbarUi>,
    pub damage: &'a mut Option<DamageTexts>,
}

/// Each of the frame's notices, in the order the frameworks get them (`RecvNotice_*`), then the
/// shortcut bar's checks (`gmShortcutUI`); `reload_shortcuts` reloads the bar instead of
/// checking it. Returns the friends requests the page raised, in order.
pub fn notice_pass<P: HudSource>(
    panels: NoticePanels<'_>,
    player: &P,
    events: &[SessionEvent],
    server_time: f64,
    mut reload_shortcuts: bool,
) -> Vec<FriendsRequest> {
    let NoticePanels {
        tree,
        now,
        dirty,
        item_selection,
        vitals,
        effects,
        vitae,
        net,
        pk,
        stack,
        external_container,
        secure_trade,
        dialogs,
        shop,
        catalog,
        consignment,
        fellowship,
        mini_fellow,
        pets,
        friends,
        quests,
        allegiance,
        paper_doll,
        backpack,
        examination,
        skills,
        shortcuts,
        money,
        toolbar,
        damage,
    } = panels;
    let mut requests = Vec::new();
    let mut shortcut_checks: Vec<ShortcutCheck> = Vec::new();
    for event in events {
        if let Some(dialog) = stack.as_mut() {
            crate::stack_event(dialog, tree, event);
        }
        if let Some(panel) = external_container.as_mut() {
            crate::external_container_event(
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
            crate::secure_trade_notice(panel, tree, dialogs, player, server_time, notice);
        }
        if let (Some(panel), Some(dialogs)) = (shop.as_mut(), dialogs.as_mut()) {
            crate::shop_notice(panel, tree, dialogs, player, server_time, notice);
        }
        if let Some(panel) = catalog.as_mut() {
            crate::catalog_notice(panel, tree, player, server_time, notice);
        }
        if let (Some(panel), Some(dialogs)) = (consignment.as_mut(), dialogs.as_mut()) {
            crate::consignment_notice(panel, tree, dialogs, player, server_time, notice);
        }
        if let Some(dialogs) = dialogs.as_mut() {
            crate::fellowship_notice(
                fellowship.as_mut(),
                mini_fellow.as_mut(),
                tree,
                dialogs,
                player,
                notice,
            );
        }
        if let Some(panel) = pets.as_mut() {
            crate::pet_notice(panel, tree, player, server_time, notice);
        }
        if let Some(panel) = friends.as_mut() {
            // `gmFriendsUI`: an added friend's request for the list again.
            if let Some(request) = panel.notice(tree, notice) {
                requests.push(request);
            }
        }
        if let Some(panel) = quests.as_mut() {
            panel.recv_gameplay_notice(tree, player, notice);
        }
        if let (Some(page), Some(dialogs)) = (allegiance.as_mut(), dialogs.as_mut()) {
            crate::allegiance_notice(page, tree, dialogs, player, notice);
        }
        if let Some(doll) = paper_doll.as_mut() {
            let old = *item_selection;
            crate::paper_doll_notice(doll, tree, player, old, notice);
        }
        if let Some(pack) = backpack.as_mut() {
            crate::backpack_notice(pack, tree, player, item_selection, notice);
        }
        StatusBar {
            vitals: &mut *vitals,
            effects: &mut *effects,
            vitae: &mut *vitae,
            net: &mut *net,
            pk: &mut *pk,
        }
        .notice(tree, player, server_time, notice);
        shortcut_checks.extend(shortcut_checks_of(notice));
        match notice {
            GameplayNotice::UpdateExaminationProfile(profile) => {
                if let Some(panel) = examination.as_mut() {
                    // `ProcessEffectIDNode`: the effect nodes ask the player's script first.
                    let expanded = expand_examination_effects(player, profile);
                    let items = player.examination_items(&expanded);
                    panel.update_profile(tree, player, server_time, &items);
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
            // `gmSelectionUI::RecvNotice_Select` (the client's `target_select::frame_notices`).
            GameplayNotice::Select(_) => {}
            // `gm3DTextUI::RecvNotice_Do3DDamage`.
            GameplayNotice::Do3DDamage { object, text } => {
                if let Some(damage) = damage.as_mut() {
                    damage.add(tree, *object, text, now);
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
    requests
}
