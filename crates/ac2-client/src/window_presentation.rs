//! The display section's window and presentation state. Retail's `Device::LoadRenderingPreferences`
//! read FullScreen, TripleBuffering, OptimalRefreshRate, SyncToDisplayRefresh and Antialiasing
//! into a `RenderDevicePresentation`, which `Device::ChangePresentation` and
//! `RenderDeviceD3D::ResetDevice` turned into the D3D presentation parameters: `Windowed`,
//! `BackBufferCount`, `PresentationInterval`, `FullScreen_RefreshRateInHz` and `MultiSampleType`.
//!
//! Here the window and the surface take the first three. FullScreen is borderless fullscreen on
//! the window's monitor (`Device::ToggleFullScreenMode`'s display change has no portable form, see
//! the module's skips below). TripleBuffering is the swap chain's frame latency: `BackBufferCount`
//! of 2 is three images (latency 2), 1 is two images (latency 1), as wgpu counts them. A
//! SyncToDisplayRefresh of on is the FIFO present mode (`PresentationInterval` ONE); off asks for
//! immediate presentation (`AutoNoVsync`, which falls back to mailbox, then FIFO).
//!
//! Skipped, with the reason:
//! - OptimalRefreshRate (`FullScreen_RefreshRateInHz`): retail chose the display mode's best
//!   refresh rate for an exclusive full-screen device. Borderless fullscreen keeps the monitor's
//!   current mode, and winit's exclusive fullscreen is not available on Wayland or macOS.
//! - Antialiasing (`MultiSampleType` 2 samples): the scene draws into a single-sampled HDR target
//!   and the tone map and UI read it. Multisampling needs resolve passes in `ac2-gpu` first.

use ac2_prefs::{PreferenceBank, Value};

/// `ID_Rendering_FullScreen` (`Device::InitializeRenderingPreferences`, section `0x4000000B`).
pub const FULL_SCREEN: u32 = ac2_render::prefs::FULL_SCREEN;
/// `ID_Rendering_TripleBuffering`.
pub const TRIPLE_BUFFERING: u32 = 0x4000_0005;
/// `ID_Rendering_SyncToDisplayRefresh`.
pub const SYNC_TO_DISPLAY_REFRESH: u32 = 0x4000_0006;

/// The window's fullscreen state, the surface's vertical sync and its frame latency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPresentation {
    /// Borderless fullscreen on the window's current monitor (`Windowed = !FullScreen`).
    pub fullscreen: bool,
    /// Vertical sync: FIFO when on, immediate presentation when off.
    pub vsync: bool,
    /// Frames queued ahead of the display (`desired_maximum_frame_latency`): 1 for a double
    /// buffer, 2 for triple buffering.
    pub frame_latency: u32,
}

impl WindowPresentation {
    /// The presentation for the three flags (`Device::ChangePresentation`'s mapping).
    pub fn from_flags(fullscreen: bool, triple_buffering: bool, sync_to_refresh: bool) -> Self {
        Self {
            fullscreen,
            vsync: sync_to_refresh,
            frame_latency: if triple_buffering { 2 } else { 1 },
        }
    }

    /// The presentation the bank's values give. A key the bank does not hold takes its retail
    /// default (`Device::InitializeRenderingPreferences`: FullScreen on, TripleBuffering off,
    /// SyncToDisplayRefresh on).
    pub fn from_bank(bank: &PreferenceBank) -> Self {
        let flag = |key: u32, default: bool| {
            bank.find(key)
                .map_or(default, |pref| matches!(pref.value, Value::Bool(true)))
        };
        Self::from_flags(
            flag(FULL_SCREEN, true),
            flag(TRIPLE_BUFFERING, false),
            flag(SYNC_TO_DISPLAY_REFRESH, true),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_prefs::NameTable;

    /// Every key and the device section named, as the DAT names them: a key with no name is
    /// not registered.
    struct Named;

    impl NameTable for Named {
        fn section_name(&self, id: u32) -> Option<String> {
            (id == ac2_render::prefs::DEVICE_SECTION).then(|| "Rendering".to_owned())
        }
        fn key_name(&self, id: u32) -> Option<String> {
            Some(format!("Key{id:08X}"))
        }
        fn enum_name(&self, _map: u32, _label: u32) -> Option<String> {
            None
        }
        fn enum_label(&self, _map: u32, _name: &str) -> Option<u32> {
            None
        }
    }

    #[test]
    fn the_flags_map_to_the_window_and_the_surface() {
        // Retail's defaults: fullscreen, vertical sync, a double buffer.
        let default = WindowPresentation::from_flags(true, false, true);
        assert_eq!(
            default,
            WindowPresentation {
                fullscreen: true,
                vsync: true,
                frame_latency: 1,
            }
        );
        // Triple buffering is a frame of latency more; sync off asks for immediate presentation.
        let windowed = WindowPresentation::from_flags(false, true, false);
        assert_eq!(
            windowed,
            WindowPresentation {
                fullscreen: false,
                vsync: false,
                frame_latency: 2,
            }
        );
    }

    #[test]
    fn the_bank_gives_the_display_flags_and_retail_defaults_for_missing_keys() {
        let mut bank = PreferenceBank::new(&[]);
        // Nothing registered: the retail defaults.
        assert_eq!(
            WindowPresentation::from_bank(&bank),
            WindowPresentation::from_flags(true, false, true)
        );
        ac2_render::prefs::register_device(&mut bank, &Named);
        assert_eq!(
            WindowPresentation::from_bank(&bank),
            WindowPresentation::from_flags(true, false, true)
        );
        // The player turns FullScreen off and TripleBuffering on (the values, not yet applied).
        bank.find_mut(FULL_SCREEN).unwrap().value = Value::Bool(false);
        bank.find_mut(TRIPLE_BUFFERING).unwrap().value = Value::Bool(true);
        bank.apply(ac2_render::prefs::DEVICE_SECTION);
        assert_eq!(
            WindowPresentation::from_bank(&bank),
            WindowPresentation::from_flags(false, true, true)
        );
    }

    #[test]
    fn the_display_keys_are_the_device_section() {
        for key in ac2_ui::DISPLAY_KEYS {
            assert_eq!(key >> 24, 0x40, "{key:08x} is a preference key");
        }
        assert!(ac2_ui::DISPLAY_KEYS.contains(&FULL_SCREEN));
    }
}
