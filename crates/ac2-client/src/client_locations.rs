//! The player's saved window locations (`GameplayOptionsProfile`'s `UISaveLocations`). They are
//! read on the options' reload and at login (`gmGamePlayUI::RecvNotice_ReloadOptions`, and the
//! refresh `ListenToMessage` 0xffff0007 that `UIElementManager::RefreshEvent` sends), and saved
//! when a window moves (`gmGamePlayUI::RecvNotice_SaveLocationsChanged`: `SetUISaveLocations`,
//! then `UploadUISettings`).

use ac2_net::protocol::{NativeObject, UiLocation, UiSaveLocations};
use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_script::native::{add_ui_location, GameplayOptions};
use ac2_ui::{SavedWindow, WindowLocation};
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::scripts::{
    IFUN_INQ_UI_SAVE_LOCATIONS, IFUN_SET_UI_SAVE_LOCATIONS, IFUN_UPLOAD_UI_SETTINGS,
};

/// The windows of a `UISaveLocations`, in its order (`UISaveLocations::InqUILocationData`'s
/// entries as the stream holds them).
pub fn saved_windows(ui: &UiSaveLocations) -> Vec<SavedWindow> {
    ui.layouts
        .iter()
        .flat_map(|(layout, _, windows)| {
            windows.iter().map(move |(element, l)| SavedWindow {
                layout: *layout,
                element: *element,
                location: WindowLocation {
                    x0: l.x0,
                    y0: l.y0,
                    x1: l.x1,
                    y1: l.y1,
                    shown: l.shown != 0,
                },
            })
        })
        .collect()
}

/// The `UISaveLocations` the UI's windows make (`UIMainFramework::InqLocations`'s `AddLocation`
/// calls in order). `Err` when two windows claim one place: retail's `AddLocation` fails and the
/// save is not made.
pub fn ui_save_locations(saved: &[SavedWindow]) -> Result<UiSaveLocations, String> {
    let mut ui = GameplayOptions::default().ui_locations;
    for window in saved {
        let l = window.location;
        let location = UiLocation {
            x0: l.x0,
            y0: l.y0,
            x1: l.x1,
            y1: l.y1,
            shown: u32::from(l.shown),
        };
        if !add_ui_location(&mut ui, window.layout, window.element, location) {
            return Err(format!(
                "window {:#x} of layout {:#x} is listed twice",
                window.element, window.layout
            ));
        }
    }
    Ok(ui)
}

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
}
