//! The gameplay HUD's global key presses (`gmGamePlayUI::ListenToMessage`, message `0xffff0003`)
//! and the child frameworks they toggle (`gmGamePlayUI::ChildKeyPress` over `_childrenInfoList`).
//!
//! The children with a key are the ones `gmGamePlayUI::SetupChildren` registers with one: the
//! radar (`0x4100000B`), the status bar (`0x41000009`) and the toolbar (`0x41000008`). A press
//! runs their `ChildKeyPress`, which shows the child when it is hidden and hides it when shown.
//! The gameplay UI's own key (`0x4100000C`) shows or hides the whole HUD, and only while no death
//! dialog is open and no logout is requested.

/// `gmGamePlayUI::ListenToMessage` case `0x41000008`: the toolbar child (`gmToolbarUI`).
pub const TOOLBAR_CHILD_KEY: u32 = 0x4100_0008;
/// Case `0x41000009`: the status bar child (`gmStatusbarUI`).
pub const STATUSBAR_CHILD_KEY: u32 = 0x4100_0009;
/// Case `0x4100000B`: the radar child (`gmRadarUI`).
pub const RADAR_CHILD_KEY: u32 = 0x4100_000B;
/// Case `0x4100000C`: the gameplay UI itself (`gmGamePlayUI::Show` of its toggle).
pub const GAMEPLAY_SHOW_KEY: u32 = 0x4100_000C;
/// Case `0x4100000D`: `USE_KEY`, which runs `gmCWInterface::UseSelection` on the player's
/// selection (no shown check; it needs only the player object).
pub const USE_KEY: u32 = 0x4100_000D;

/// A gameplay child that a key toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayChild {
    Toolbar,
    Statusbar,
    Radar,
}

/// The child a global key press toggles, if any (`_childrenInfoList`'s `keypress`).
pub fn child_for_key(key: u32) -> Option<GameplayChild> {
    match key {
        TOOLBAR_CHILD_KEY => Some(GameplayChild::Toolbar),
        STATUSBAR_CHILD_KEY => Some(GameplayChild::Statusbar),
        RADAR_CHILD_KEY => Some(GameplayChild::Radar),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_three_registered_children_have_keys() {
        assert_eq!(child_for_key(0x4100_0008), Some(GameplayChild::Toolbar));
        assert_eq!(child_for_key(0x4100_0009), Some(GameplayChild::Statusbar));
        assert_eq!(child_for_key(0x4100_000B), Some(GameplayChild::Radar));
        // The HUD's own key and the toolbar's panel keys are not child keys.
        assert_eq!(child_for_key(GAMEPLAY_SHOW_KEY), None);
        assert_eq!(child_for_key(0x4100_0001), None);
        assert_eq!(child_for_key(0x4100_000D), None);
    }
}
