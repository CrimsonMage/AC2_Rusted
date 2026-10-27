//! The render preferences: the registered keys and their defaults, the presets' values and
//! defaults, and the landscape draw distance the renderer reads.

use ac2_prefs::{NameTable, PreferenceBank, Value};

use super::{
    apply_quality, landscape_draw_distance, register_device, register_graphics, AVATAR_SECTION,
    DEVICE_SECTION, GRAPHICS_SECTION, QUALITY_PRESETS,
};

/// Every section and key the render registers, named as the DAT names them (fixed here).
struct Names;

impl NameTable for Names {
    fn section_name(&self, id: u32) -> Option<String> {
        match id {
            GRAPHICS_SECTION => Some("Graphics".to_owned()),
            AVATAR_SECTION => Some("Avatar".to_owned()),
            DEVICE_SECTION => Some("Rendering".to_owned()),
            _ => None,
        }
    }

    fn key_name(&self, id: u32) -> Option<String> {
        Some(format!("Key{id:08X}"))
    }

    fn enum_name(&self, _map: u32, label: u32) -> Option<String> {
        Some(format!("Label{label:X}"))
    }

    fn enum_label(&self, _map: u32, name: &str) -> Option<u32> {
        name.strip_prefix("Label")
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
    }
}

fn bank() -> PreferenceBank {
    PreferenceBank::new(b"")
}

#[test]
fn the_graphics_section_registers_twenty_one_keys_in_retail_order() {
    let mut bank = bank();
    assert_eq!(register_graphics(&mut bank, &Names), 23);
    let keys = bank.list(GRAPHICS_SECTION);
    assert_eq!(keys.len(), 21);
    assert_eq!(keys[0], 0x4000_004F, "TextureDetail is registered first");
    assert_eq!(keys[20], 0x4000_004B, "ScreenBrightness is registered last");
    assert_eq!(bank.list(AVATAR_SECTION), vec![0x4000_006C, 0x4000_006D]);
}

#[test]
fn defaults_and_ranges_are_the_constructor_values() {
    let mut bank = bank();
    register_graphics(&mut bank, &Names);
    let value = |key| bank.find(key).expect("registered").value.clone();
    assert_eq!(
        value(0x4000_004F),
        Value::Int(2),
        "TextureDetail defaults to 2"
    );
    assert_eq!(
        value(0x4000_0043),
        Value::Int(2),
        "LandscapeDrawDistance defaults to 2"
    );
    assert_eq!(
        value(0x4000_005E),
        Value::Bool(true),
        "TrilinearFiltering defaults on"
    );
    assert_eq!(
        value(0x4000_0064),
        Value::Bool(false),
        "AdaptiveDegrade defaults off"
    );
    assert_eq!(value(0x4000_004B), Value::Float(0.0));
    let screen = bank.find(0x4000_004B).expect("registered");
    assert_eq!((screen.min, screen.max), (-0.2, 1.0));
    let bias = bank.find(0x4000_0065).expect("registered");
    assert_eq!((bias.min, bias.max), (-1.0, 1.0));
    assert_eq!(
        value(0x4000_006D),
        Value::Bool(true),
        "MouseHoverTransparency defaults on"
    );
    assert_eq!(
        value(0x4000_006C),
        Value::Int(2),
        "VisibleInFirstPerson defaults to 2"
    );
}

#[test]
fn the_display_section_registers_five_flags_and_leaves_the_resolution_out() {
    let mut bank = bank();
    assert_eq!(register_device(&mut bank, &Names), 5);
    assert_eq!(
        bank.list(DEVICE_SECTION),
        vec![
            0x4000_0003,
            0x4000_0005,
            0x4000_0007,
            0x4000_0006,
            0x4000_0060
        ]
    );
    assert!(
        bank.find(0x4000_0047).is_none(),
        "DisplayResolution waits for the device's modes"
    );
    assert_eq!(
        bank.find(0x4000_0003).expect("registered").value,
        Value::Bool(true)
    );
    assert_eq!(
        bank.find(0x4000_0060).expect("registered").value,
        Value::Bool(false)
    );
}

#[test]
fn the_presets_hold_their_values_and_defaults() {
    assert_eq!(QUALITY_PRESETS.len(), 5);
    for (index, lines) in QUALITY_PRESETS.iter().enumerate() {
        assert_eq!(lines.len(), 20, "level {} sets twenty keys", index + 1);
        for line in lines.iter() {
            assert_eq!(
                line.value,
                line.default,
                "level {} sets value and default",
                index + 1
            );
        }
    }
    let level5 = QUALITY_PRESETS[4];
    let draw = level5
        .iter()
        .find(|l| l.key == 0x4000_0043)
        .expect("draw distance");
    assert_eq!(draw.value, 4.0);
}

#[test]
fn a_quality_level_sets_the_registered_values_and_the_draw_distance() {
    let mut bank = bank();
    register_graphics(&mut bank, &Names);
    assert_eq!(landscape_draw_distance(&bank), Some(2));
    assert!(apply_quality(&mut bank, 5));
    assert_eq!(landscape_draw_distance(&bank), Some(4));
    let pref = bank.find(0x4000_0043).expect("registered");
    assert_eq!(
        pref.default,
        Value::Int(4),
        "the level sets the default too"
    );
    assert!(
        bank.submit(GRAPHICS_SECTION).is_empty(),
        "the section is applied"
    );
}

#[test]
fn an_out_of_range_level_applies_the_section_without_a_preset() {
    let mut bank = bank();
    register_graphics(&mut bank, &Names);
    assert!(!apply_quality(&mut bank, 0));
    assert!(!apply_quality(&mut bank, 6));
    assert_eq!(landscape_draw_distance(&bank), Some(2));
}

#[test]
fn the_scene_options_read_the_draw_distance_and_the_frill_distance() {
    use crate::frill::FrillSettings;
    use crate::prefs::{scene_options, SceneOptions, LANDSCAPE_FRILL_DISTANCE};
    use crate::terrain::far_radius_for_draw_distance;

    let mut bank = bank();
    register_graphics(&mut bank, &Names);
    // The defaults are the scene's own: draw distance 2 (radius 4), the long frill distance.
    assert_eq!(scene_options(&bank), SceneOptions::default());
    assert_eq!(SceneOptions::default().far_radius, 4);
    assert!(!SceneOptions::default().frill.short_distance);

    bank.find_mut(0x4000_0043)
        .expect("LandscapeDrawDistance")
        .set_choice(0);
    assert_eq!(
        scene_options(&bank).far_radius,
        0,
        "the nearest draw distance"
    );
    bank.find_mut(0x4000_0043)
        .expect("LandscapeDrawDistance")
        .set_choice(4);
    assert_eq!(
        scene_options(&bank).far_radius,
        far_radius_for_draw_distance(4)
    );

    bank.find_mut(LANDSCAPE_FRILL_DISTANCE)
        .expect("LandscapeFrillDistance")
        .set_choice(1);
    assert_eq!(
        scene_options(&bank).frill,
        FrillSettings {
            short_distance: true,
            ..FrillSettings::default()
        }
    );
}
