//! The shortcut bar's checks that the gameplay notices ask for (`gmShortcutUI::RecvNotice_*`):
//! `CheckAllItems`, `CheckAllSkills`, `CheckSkill(skill)`, `CheckItem(item, setPending)` and
//! `RecvNotice_CreateObject`'s set-up of the slots waiting on that object.

use ac2_weenie::GameplayNotice;

use crate::element::UiTree;
use crate::shortcut::{ShortcutSource, ShortcutUi};

/// One shortcut bar check a notice asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCheck {
    /// `CheckAllItems`.
    Items,
    /// `CheckAllSkills`.
    Skills,
    /// `CheckSkill(skill)`.
    Skill(u32),
    /// `CheckItem(item, setPending)`.
    Item(u64, bool),
    /// `RecvNotice_CreateObject(object)`.
    Created(u64),
}

/// The bar's checks, in the order the notices arrived.
pub fn apply_shortcut_checks(
    bar: &mut ShortcutUi,
    tree: &mut UiTree,
    source: &dyn ShortcutSource,
    checks: &[ShortcutCheck],
) {
    for &check in checks {
        match check {
            ShortcutCheck::Items => bar.check_all_items(tree, source),
            ShortcutCheck::Skills => bar.check_all_skills(tree, source),
            ShortcutCheck::Skill(skill) => bar.check_skill(tree, source, skill),
            ShortcutCheck::Item(item, pending) => bar.check_item(tree, source, item, pending),
            ShortcutCheck::Created(object) => bar.create_object(tree, source, object),
        }
    }
}

/// `gmShortcutUI::RecvNotice_BeingDeleted` (Legions 0x4377c0): an object other than the player
/// going checks the slots holding it (`CheckItem(id, false)`), which drops a shortcut to an item
/// the player no longer has. Objects go one at a time from the world's destruction queue, never
/// in a shutdown, so retail's `_shutdown` skip has no case here.
pub fn removal_check(object: u64, player: u64) -> Option<ShortcutCheck> {
    (object != player).then_some(ShortcutCheck::Item(object, false))
}

/// The checks a notice asks of the bar: `RecvNotice_UpdateSkill` (CheckSkill),
/// `RecvNotice_UpdateInventory` (all items for 0x40000001, else the one item, pending for
/// 0x40000005; then all skills), `RecvNotice_UpdateContainer` (the same items part) and
/// `RecvNotice_UpdateResetTime` (CheckAllSkills) and `RecvNotice_CreateObject`.
pub fn shortcut_checks_of(notice: &GameplayNotice) -> Vec<ShortcutCheck> {
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
        GameplayNotice::CreateObject(object) => vec![ShortcutCheck::Created(*object)],
        _ => Vec::new(),
    }
}
