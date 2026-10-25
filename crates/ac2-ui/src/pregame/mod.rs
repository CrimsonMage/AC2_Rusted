//! The pregame UI: the main frameworks `gmUIFlow` switches between before the world (character
//! selection, character generation, disconnected), over the layouts the DATs ship.
//!
//! [`PregameUi`] owns the element tree, the mode flow (`UIFlow`), the shared data
//! (`UIPersistantData`), the dialogs (`DialogFactory`) and the current framework. The client
//! feeds it the session's login events and the frame's UI input, and carries out its
//! [`PregameRequest`]s (enter the world, create or delete a character, quit).

mod arms;
mod assets;
mod avatar_modification;
mod avatar_selection;
mod character_selection;
mod chargen;
mod confirmation;
mod credits;
mod data_patch;
mod disconnected;
mod flow;
mod intro;
mod keyboard;
mod keyboard_list;
#[cfg(test)]
mod keyboard_list_tests;
mod persistent;
mod strings;
#[cfg(test)]
mod tests;
mod ui;

pub use crate::scene_viewport::SceneViewport;
pub use arms::{CharacterArms, SceneEntity};
pub use assets::{load_pregame, PregameAssets, PREGAME_LAYOUTS};
pub use avatar_modification::{AvatarModificationUi, ModificationCamera};
pub use avatar_selection::{Avatar, AvatarSelectionUi};
pub use character_selection::{ArmOccupant, CharacterSelectionUi};
pub use chargen::{
    new_char_gen, phys_type, phys_type_value, result_key, set_phys_type_value, CharGenSystem,
    NoCharGen, AVATAR_LOCATIONS, RACE_NAMES, SEX_NAMES,
};
pub use confirmation::{is_name_character, ConfirmationUi};
pub use flow::{
    char_error, character_error_mode, UiFlow, MODE_AVATAR_MODIFICATION, MODE_AVATAR_SELECTION,
    MODE_CHARACTER_SELECTION, MODE_CREDITS, MODE_DATA_PATCH, MODE_DISCONNECTED, MODE_GAMEPLAY,
    MODE_INTRO, MODE_KEYBOARD,
};
pub use keyboard_list::{
    BindingRow, KeyBinding, KeyboardList, SetOutcome, SlotRef, META_CONTROL, META_NONE, META_SHIFT,
};
pub use persistent::{greyed_out_for, identity, PersistentData};
pub use strings::{UiStrings, UI_STRING_TABLE};
pub use ui::{PregameRequest, PregameUi};

use std::sync::Arc;

use crate::dialog::DialogFactory;
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::token::tokenize;

/// `UIElement::GetAttribute_DataID(element, token_music)`: the music a screen's element names,
/// which the screen's constructor plays on `Channel_Main` (`gmIntroUI`, `gmCharacterSelectionUI`
/// and `gmAvatarModificationUI`).
pub(crate) fn screen_music(tree: &UiTree, element: Option<usize>) -> Option<u32> {
    match tree.get(element?)?.attribute(tokenize("music")) {
        Some(AttributeValue::DataId(did)) => Some(*did),
        _ => None,
    }
}

/// `gmDisconnectedUI`'s layout.
pub const LAYOUT_DISCONNECTED: u32 = 0x4100_0005;
/// `gmCharacterSelectionUI`'s layout.
pub const LAYOUT_CHARACTER_SELECTION: u32 = 0x4100_0006;
/// `gmIntroUI`'s layout.
pub const LAYOUT_INTRO: u32 = 0x4100_0008;
/// `gmDataPatchUI`'s layout.
pub const LAYOUT_DATA_PATCH: u32 = 0x4100_000D;
/// `gmCreditsUI`'s layout.
pub const LAYOUT_CREDITS: u32 = 0x4100_000E;
/// `gmKeyboardUI`'s layout.
pub const LAYOUT_KEYBOARD: u32 = 0x4100_0001;
/// `gmAvatarSelectionUI`'s layout.
pub const LAYOUT_AVATAR_SELECTION: u32 = 0x4100_000A;
/// `gmAvatarModificationUI`'s layout.
pub const LAYOUT_AVATAR_MODIFICATION: u32 = 0x4100_000B;
/// `gmGenerationConfirmationUI`'s layout.
pub const LAYOUT_GENERATION_CONFIRMATION: u32 = 0x4100_000C;

/// What a framework works on: the tree, the dialogs, the flow, the shared data, the strings
/// and the requests it makes of the client.
pub(crate) struct Ctx<'a> {
    pub tree: &'a mut UiTree,
    pub dialogs: &'a mut DialogFactory,
    pub flow: &'a mut UiFlow,
    pub data: &'a mut PersistentData,
    pub strings: &'a UiStrings,
    pub assets: &'a PregameAssets,
    pub requests: &'a mut Vec<PregameRequest>,
    pub system: &'a mut dyn CharGenSystem,
}

impl UiTree {
    /// `UIFramework::CreateElement(layoutEnum, elementID)`: the layout's top-level element made
    /// as a free element (no root, no parent).
    pub(crate) fn create_from(
        &mut self,
        layout: &Arc<LayoutDesc>,
        element_id: u32,
    ) -> Option<usize> {
        let reference = Arc::clone(layout.element(element_id)?);
        let index = self.create_element(Arc::clone(layout), reference, false)?;
        self.initialize_panel_pages(index);
        Some(index)
    }
}
