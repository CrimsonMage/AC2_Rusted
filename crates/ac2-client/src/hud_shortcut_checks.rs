//! The shortcut bar's checks that the gameplay notices ask for (`gmShortcutUI::RecvNotice_*`):
//! `CheckAllItems`, `CheckAllSkills`, `CheckSkill(skill)` and `CheckItem(item, setPending)`.

use ac2_ui::ShortcutUi;
use ac2_weenie::GameplayNotice;

use crate::ui_sources::ClientPlayer;

/// One shortcut bar check a notice asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShortcutCheck {
    /// `CheckAllItems`.
    Items,
    /// `CheckAllSkills`.
    Skills,
    /// `CheckSkill(skill)`.
    Skill(u32),
    /// `CheckItem(item, setPending)`.
    Item(u64, bool),
}

/// The bar's checks, in the order the notices arrived.
pub(crate) fn apply_shortcut_checks(
    bar: &mut ShortcutUi,
    tree: &mut ac2_ui::UiTree,
    player: &ClientPlayer,
    checks: &[ShortcutCheck],
) {
    for &check in checks {
        match check {
            ShortcutCheck::Items => bar.check_all_items(tree, player),
            ShortcutCheck::Skills => bar.check_all_skills(tree, player),
            ShortcutCheck::Skill(skill) => bar.check_skill(tree, player, skill),
            ShortcutCheck::Item(item, pending) => bar.check_item(tree, player, item, pending),
        }
    }
}

/// The checks a notice asks of the bar: `RecvNotice_UpdateSkill` (CheckSkill),
/// `RecvNotice_UpdateInventory` (all items for 0x40000001, else the one item, pending for
/// 0x40000005; then all skills), `RecvNotice_UpdateContainer` (the same items part) and
/// `RecvNotice_UpdateResetTime` (CheckAllSkills).
pub(crate) fn shortcut_checks_of(notice: &GameplayNotice) -> Vec<ShortcutCheck> {
    const ALL: u32 = 0x4000_0001;
    const PENDING: u32 = 0x4000_0005;
    let items = |code: u32, item: u64| {
        if code == ALL {
            ShortcutCheck::Items
        } else {
            ShortcutCheck::Item(item, code == PENDING)
        }
    };
    match notice {
        GameplayNotice::UpdateSkill(skill) => vec![ShortcutCheck::Skill(*skill)],
        GameplayNotice::UpdateInventory { code, item, .. } => {
            vec![items(*code, *item), ShortcutCheck::Skills]
        }
        GameplayNotice::UpdateContainer { code, item, .. } => vec![items(*code, *item)],
        GameplayNotice::UpdateResetTime => vec![ShortcutCheck::Skills],
        _ => Vec::new(),
    }
}
