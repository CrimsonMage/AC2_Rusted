//! The player's saved window locations (`GameplayOptionsProfile`'s `UISaveLocations`), as the
//! client's wiring. The windows' conversions and the `UISaveLocations` they make are
//! `ac2_ui::window_locations`. They are read on the options' reload and at login
//! (`gmGamePlayUI::RecvNotice_ReloadOptions`, and the refresh `ListenToMessage` 0xffff0007 that
//! `UIElementManager::RefreshEvent` sends), and saved when a window moves
//! (`gmGamePlayUI::RecvNotice_SaveLocationsChanged`: `SetUISaveLocations`, then
//! `UploadUISettings`).

use ac2_net::protocol::{NativeObject, UiSaveLocations};
use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_ui::{saved_windows, ui_save_locations, SavedWindow};
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::scripts::{
    IFUN_INQ_UI_SAVE_LOCATIONS, IFUN_SET_UI_SAVE_LOCATIONS, IFUN_SET_WINDOW_OPACITY,
    IFUN_UPLOAD_UI_SETTINGS,
};

impl Client {
    /// `gmCWInterface::InqUISaveLocations`: the player's saved window locations (the script's
    /// answer, through its `UISaveLocations` out parameter).
    pub fn saved_window_locations(&mut self) -> Result<Vec<SavedWindow>, String> {
        let args = [IFunArg::Out(OutValue::Native(
            NativeObject::UiSaveLocations(UiSaveLocations::default()),
        ))];
        let r = self.query(IFUN_INQ_UI_SAVE_LOCATIONS, &args, IFunReturn::Word)?;
        match r.out_native(0) {
            Some(NativeObject::UiSaveLocations(ui)) => Ok(saved_windows(ui)),
            _ => Err("no saved window locations".into()),
        }
    }

    /// `gmGamePlayUI::RecvNotice_SaveLocationsChanged`: `gmCWInterface::SetUISaveLocations` with
    /// the windows' locations, then `UploadUISettings` (the profile to the server). Nothing is
    /// sent when the locations cannot be made or the player's script does not run.
    pub(crate) fn save_window_locations(&mut self, saved: &[SavedWindow]) -> Vec<SessionEvent> {
        if !self.scripts_own(IFUN_SET_UI_SAVE_LOCATIONS) {
            return Vec::new();
        }
        let Ok(ui) = ui_save_locations(saved) else {
            return Vec::new();
        };
        let mut events = self.script_ifun_args(
            IFUN_SET_UI_SAVE_LOCATIONS,
            &[IFunArg::Native(NativeObject::UiSaveLocations(ui))],
        );
        events.extend(self.script_ifun(IFUN_UPLOAD_UI_SETTINGS, &[]));
        events
    }

    /// `ChatWindow::ListenToMessage`'s release of the opacity slider: `gmCWInterface::
    /// SetWindowOpacity(window, opacity)`, then `UploadUISettings`.
    pub(crate) fn set_window_opacity(&mut self, window: u32, opacity: f32) -> Vec<SessionEvent> {
        if !self.scripts_own(IFUN_SET_WINDOW_OPACITY) {
            return Vec::new();
        }
        let mut events = self.script_ifun_args(
            IFUN_SET_WINDOW_OPACITY,
            &[IFunArg::Int(window), IFunArg::Int(opacity.to_bits())],
        );
        events.extend(self.script_ifun(IFUN_UPLOAD_UI_SETTINGS, &[]));
        events
    }
}
