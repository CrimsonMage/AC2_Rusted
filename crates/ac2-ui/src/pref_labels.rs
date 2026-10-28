//! The labels of the client's preferences on the options page. Each registration in the retail
//! client passes a `Tokenize("ID_<...>")` name beside its `Preference::Bless` (or `Create`), with
//! `_Help` for the tooltip; those names are the table below, keyed by section and key ID. A
//! preference without a name here shows no row on the page.
//!
//! Sources: `CameraManager::SetupPrefs` (camera and input), `Render::InitRenderingPreferences`
//! (graphics and avatar), `Device::InitializeRenderingPreferences` (rendering), and
//! `SoundManager`'s preference setup (sound). The keys are the IDs the port registers.

/// The label of the preference `key` in `section`, when the client names it.
pub fn pref_label(section: u32, key: u32) -> Option<&'static str> {
    PREF_LABELS
        .iter()
        .find(|&&(s, k, _)| s == section && k == key)
        .map(|&(_, _, label)| label)
}

/// (section, key, label), the sections as `PreferenceBank` names them.
const PREF_LABELS: &[(u32, u32, &str)] = &[
    // Sound.
    (0x4000_0007, 0x4000_0040, "ID_Sound_FXVolume"),
    (0x4000_0007, 0x4000_0041, "ID_Sound_MusicVolume"),
    (0x4000_0007, 0x4000_003D, "ID_Sound_Player_Music"),
    (0x4000_0007, 0x4000_0056, "ID_Sound_AmbientVolume"),
    (0x4000_0007, 0x4000_0057, "ID_Sound_SoundFeatures"),
    (0x4000_0007, 0x4000_0058, "ID_Sound_NumPlayingSounds"),
    (0x4000_0007, 0x4000_0033, "ID_Sound_DisableSound"),
    (0x4000_0007, 0x4000_0034, "ID_Sound_DisableMusic"),
    // Rendering (the display section).
    (0x4000_000B, 0x4000_0047, "ID_Rendering_DisplayResolution"),
    (0x4000_000B, 0x4000_0003, "ID_Rendering_FullScreen"),
    (0x4000_000B, 0x4000_0005, "ID_Rendering_TripleBuffering"),
    (0x4000_000B, 0x4000_0007, "ID_Rendering_OptimalRefreshRate"),
    (
        0x4000_000B,
        0x4000_0006,
        "ID_Rendering_SyncToDisplayRefresh",
    ),
    (0x4000_000B, 0x4000_0060, "ID_Rendering_Antialiasing"),
    // Graphics.
    (0x4000_0004, 0x4000_004F, "ID_Graphics_TextureDetail"),
    (0x4000_0004, 0x4000_004D, "ID_Graphics_MaterialDetail"),
    (0x4000_0004, 0x4000_004E, "ID_Graphics_ModelDetail"),
    (0x4000_0004, 0x4000_0021, "ID_Graphics_WaterReflections"),
    (
        0x4000_0004,
        0x4000_005D,
        "ID_Graphics_LandscapeShoreEffects",
    ),
    (0x4000_0004, 0x4000_004C, "ID_Graphics_DynamicObjectShadows"),
    (0x4000_0004, 0x4000_0076, "ID_Graphics_SceneryDrawDistance"),
    (
        0x4000_0004,
        0x4000_0043,
        "ID_Graphics_LandscapeDrawDistance",
    ),
    (
        0x4000_0004,
        0x4000_0044,
        "ID_Graphics_LandscapeFrillDistance",
    ),
    (
        0x4000_0004,
        0x4000_0049,
        "ID_Graphics_LandscapeStaticObjectShadows",
    ),
    (
        0x4000_0004,
        0x4000_0020,
        "ID_Graphics_LandscapeCloudShadows",
    ),
    (0x4000_0004, 0x4000_0042, "ID_Graphics_LightCoronas"),
    (0x4000_0004, 0x4000_005E, "ID_Graphics_TrilinearFiltering"),
    (0x4000_0004, 0x4000_005F, "ID_Graphics_AnisotropicFiltering"),
    (0x4000_0004, 0x4000_0074, "ID_Graphics_VolumetricFogEffects"),
    (0x4000_0004, 0x4000_0061, "ID_Graphics_LightSourceEffects"),
    (0x4000_0004, 0x4000_0062, "ID_Graphics_DecalEffects"),
    (0x4000_0004, 0x4000_0063, "ID_Graphics_StreakEffects"),
    (0x4000_0004, 0x4000_0064, "ID_Graphics_AdaptiveDegrade"),
    (0x4000_0004, 0x4000_0065, "ID_Graphics_AdaptiveDegradeBias"),
    (0x4000_0004, 0x4000_004B, "ID_Graphics_ScreenBrightness"),
    // Avatar.
    (0x4000_000D, 0x4000_006C, "ID_Avatar_VisibleInFirstPerson"),
    (0x4000_000D, 0x4000_0068, "ID_Avatar_Look_Around"),
    (0x4000_000D, 0x4000_0069, "ID_Avatar_Looks_At_Camera"),
    (0x4000_000D, 0x4000_006A, "ID_Avatar_Look_At_Selection"),
    // Input.
    (0x4000_000E, 0x4000_0025, "ID_Input_Mouse_Look_Sensitivity"),
    (0x4000_000E, 0x4000_0026, "ID_Input_Invert_Y"),
    (0x4000_000E, 0x4000_006B, "ID_Input_Steer_Player"),
    // Camera.
    (0x4000_0008, 0x4000_0027, "ID_Camera_Height"),
    (0x4000_0008, 0x4000_002A, "ID_Camera_Distance"),
    (0x4000_0008, 0x4000_0054, "ID_Camera_Tilt"),
    (0x4000_0008, 0x4000_002F, "ID_Camera_Transition_Time"),
    (0x4000_0008, 0x4000_002D, "ID_Camera_Rotate_Speed"),
    (0x4000_0008, 0x4000_002C, "ID_Camera_Zoom_Speed"),
    (0x4000_0008, 0x4000_0051, "ID_Camera_Auto_Distance"),
    (0x4000_0008, 0x4000_005A, "ID_Camera_Align_To_Slope"),
];

#[cfg(test)]
mod tests {
    use super::pref_label;

    #[test]
    fn the_labels_are_the_registration_names() {
        assert_eq!(
            pref_label(0x4000_0007, 0x4000_0040),
            Some("ID_Sound_FXVolume")
        );
        assert_eq!(
            pref_label(0x4000_0004, 0x4000_0043),
            Some("ID_Graphics_LandscapeDrawDistance")
        );
        assert_eq!(
            pref_label(0x4000_000B, 0x4000_0003),
            Some("ID_Rendering_FullScreen")
        );
        // A key the table does not name has no row.
        assert_eq!(pref_label(0x4000_0004, 0x4000_0073), None);
    }

    #[test]
    fn every_label_is_unique_per_key() {
        for (i, &(s, k, _)) in super::PREF_LABELS.iter().enumerate() {
            assert!(
                super::PREF_LABELS[..i]
                    .iter()
                    .all(|&(s2, k2, _)| (s2, k2) != (s, k)),
                "{s:#x}/{k:#x} is named twice"
            );
        }
    }
}
