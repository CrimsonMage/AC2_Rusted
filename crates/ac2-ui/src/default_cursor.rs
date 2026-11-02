//! The UI's default pointer cursor. `gmUIFlow::gmUIFlow` (admin 0x00406B10) ends its setup with
//! `IUIManager::iuim_->SetCursor(FileArray::GetDIDFromEnum(0x41000002, 0x40000003), 0, 0, 1)`:
//! the render surface the layout group's enum `0x41000002` names (`cursor_main.tga`, 0x41000870)
//! with its hot spot at the corner, as the *default* cursor. `UIElementManager::UseDefaultCursor`
//! and `UpdateMouse` (the pointer over nothing, or over an element with no cursor of its own) go
//! back to it, so an element's cursor (a resize grip's arrows, a drag bar's move cursor, the
//! targeting pointer) never outlasts the pointer's stay on it. Without this default
//! [`UiTree::use_default_pointer_cursor`] had nothing to go back to and the last cursor an element
//! set stayed on the pointer.

use ac2_db::{ResourceResult, ResourceStore};

use crate::element::UiTree;
use crate::hud::UILAYOUT_GROUP;

/// The enum `gmUIFlow` reads the default cursor's DID from (in the layout group).
pub const DEFAULT_CURSOR_ENUM: u32 = 0x4100_0002;

/// The default cursor's image DID (`FileArray::GetDIDFromEnum(0x41000002, 0x40000003)`).
pub fn default_cursor_did(store: &ResourceStore) -> ResourceResult<Option<u32>> {
    Ok(ac2_assets::ac2_enum_did(store, UILAYOUT_GROUP, DEFAULT_CURSOR_ENUM)?.map(|d| d.raw()))
}

/// `gmUIFlow::gmUIFlow`'s `SetCursor(did, 0, 0, 1)` on `tree`.
pub fn install_default_cursor(tree: &mut UiTree, store: &ResourceStore) -> ResourceResult<()> {
    if let Some(did) = default_cursor_did(store)? {
        tree.set_pointer_cursor(did, 0, 0, true);
    }
    Ok(())
}
