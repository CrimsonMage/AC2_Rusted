//! The player as the HUD's frameworks read it for one frame: every panel source the notice pass
//! and the message and tick passes take, as one supertrait. The client's `ClientPlayer` answers
//! all of them, so the passes take `&dyn HudSource` (or a generic `HudSource`) and upcast to the
//! source each panel names (trait upcasting).

use crate::assistance::AssistanceSource;
use crate::consignment::ConsignmentSource;
use crate::craft::CraftSource;
use crate::examination_effects::ExaminationSource;
use crate::game_dialogs::DialogText;
use crate::inventory_notices::InventoryNoticeSource;
use crate::item_display::InventorySource;
use crate::pets::PetSource;
use crate::quests_panel::QuestSource;
use crate::shop::ShopSource;
use crate::shortcut::ShortcutSource;
use crate::skill_tree::SkillSource;
use crate::social_notices::SocialSource;
use crate::status_notices::StatusSource;
use crate::target_select::SelectionSource;
use crate::ui_string::UiStrings;

/// Every source the HUD's frameworks take from the player for one frame.
pub trait HudSource:
    UiStrings
    + DialogText
    + AssistanceSource
    + PetSource
    + StatusSource
    + SelectionSource
    + SkillSource
    + ShortcutSource
    + InventorySource
    + InventoryNoticeSource
    + ShopSource
    + ConsignmentSource
    + SocialSource
    + QuestSource
    + ExaminationSource
    + CraftSource
{
}

impl<T> HudSource for T where
    T: UiStrings
        + DialogText
        + AssistanceSource
        + PetSource
        + StatusSource
        + SelectionSource
        + SkillSource
        + ShortcutSource
        + InventorySource
        + InventoryNoticeSource
        + ShopSource
        + ConsignmentSource
        + SocialSource
        + QuestSource
        + ExaminationSource
        + CraftSource
{
}
