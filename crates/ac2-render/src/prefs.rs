//! The render preferences: `GraphicsPreferences` (section `0x40000004`, registered by
//! `Render::InitRenderingPreferences`), the avatar's two in section `0x4000000D` (the same
//! function), the display's five in section `0x4000000B` (`Device::InitializeRenderingPreferences`),
//! and the quality presets of `Render::SetOverallGraphicsQuality`.
//!
//! The keys are the IDs the retail code passes to `PreferenceBank::Create` and `Bless`, in the
//! order it registers them. Each enum's labels are the IDs of its choices in the render mapper
//! `0x4000004A`. `DisplayResolution` (`0x40000047`) is registered apart
//! ([`register_display_resolution`]), once the window's monitor has listed its modes.
//!
//! `HighQualityUI` (a bool whose default the function sets to 1) has no `Create` in
//! `Render::InitRenderingPreferences`, so it is not in the bank and is not registered here.
//!
//! The presets are the values and defaults `Render::SetOverallGraphicsQuality` sets for levels
//! 1 to 5 (the keys with the same value and default), then `PreferenceBank::Apply(0x40000004)`.
//! Levels 4 and 5 jump into the function's shared tail; the table lists the values that tail
//! sets for them.

use std::io;
use std::thread::JoinHandle;

use ac2_prefs::{Choice, NameTable, Pref, PreferenceBank, PrefsFile, Value};

use crate::frill::{grass_copies_for_draw_distance, FrillSettings};
use crate::object::DegradeOptions;
use crate::terrain::far_radius_for_draw_distance;

/// The graphics section (`Preferences` section `0x40000004`, named `Graphics`).
pub const GRAPHICS_SECTION: u32 = 0x4000_0004;
/// The avatar's section (`0x4000000D`, the two avatar keys).
pub const AVATAR_SECTION: u32 = 0x4000_000D;
/// The display section (`0x4000000B`, `Device::InitializeRenderingPreferences`).
pub const DEVICE_SECTION: u32 = 0x4000_000B;

/// `ID_Graphics_LandscapeDrawDistance`: the landscape's far radius (`far_radius_for_draw_distance`).
pub const LANDSCAPE_DRAW_DISTANCE: u32 = 0x4000_0043;
/// `LightSourceEffects` (`Render::m_Prefs.LightSourceEffects`): `FXManager::DoLightSourceFX`
/// places no light while it is off.
pub const LIGHT_SOURCE_EFFECTS: u32 = 0x4000_0061;
/// `StreakEffects` (`Render::m_Prefs.StreakEffects`): `FXManager::DoStreakFX` makes no streak and
/// `StreakEffectManager::RenderStreakEffects` draws none while it is off.
pub const STREAK_EFFECTS: u32 = 0x4000_0063;
/// `VolumetricFogEffects` (`Render::m_Prefs.VolumetricFogEffects`).
pub const VOLUMETRIC_FOG_EFFECTS: u32 = 0x4000_0074;
/// `ID_Avatar_VisibleInFirstPerson`.
pub const AVATAR_VISIBLE_IN_FIRST_PERSON: u32 = 0x4000_006C;
/// `ID_Rendering_FullScreen`.
pub const FULL_SCREEN: u32 = 0x4000_0003;
/// `ID_Rendering_DisplayResolution`: a display mode, valued (and labelled) by its
/// `PrefDisplayModeType::nModeDesc`, the width over the height ([`mode_desc`]).
pub const DISPLAY_RESOLUTION: u32 = 0x4000_0047;
/// The mode `Device::InitializeRenderingPreferences` always lists and takes as the default.
pub const DEFAULT_MODE: (u32, u32) = (1024, 768);

/// The render mapper: the labels of the graphics enums are its IDs.
const RENDER_MAP: u32 = 0x4000_004A;
/// The avatar's mapper: its labels are ids in the render mapper as well.
const AVATAR_MAP: u32 = 0x4000_004A;

/// One preset line: a preference key, its value, and the default it is set to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset {
    pub key: u32,
    pub value: f32,
    pub default: f32,
}

/// The quality presets, levels 1 to 5 (`Render::SetOverallGraphicsQuality`). The keys are
/// `TextureDetail` 0x4000_004F, `MaterialDetail` 0x4000_004D, `ModelDetail` 0x4000_004E, `WaterReflections` 0x4000_0021,
/// `LightCoronas` 0x4000_0042, `DynamicObjectShadows` 0x4000_004C, `LandscapeDrawDistance` 0x4000_0043,
/// `LandscapeFrillDistance` 0x4000_0044, `LandscapeStaticObjectShadows` 0x4000_0049, `LandscapeCloudShadows`
/// 0x4000_0020, `LandscapeShoreEffects` 0x4000_005D, `TrilinearFiltering` 0x4000_005E, `AnisotropicFiltering` 0x4000_005F,
/// `VolumetricFogEffects` 0x4000_0074, `LightSourceEffects` 0x4000_0061, `DecalEffects` 0x4000_0062, `StreakEffects`
/// 0x4000_0063, `SceneryDrawDistance` 0x4000_0076, `AdaptiveDegrade` 0x4000_0064, `AdaptiveDegradeBias` 0x4000_0065.
pub const QUALITY_PRESETS: [&[Preset]; 5] = [
    // Level 1.
    &[
        preset(0x4000_004F, 2.0, 2.0),
        preset(0x4000_004D, 0.0, 0.0),
        preset(0x4000_004E, 0.0, 0.0),
        preset(0x4000_0021, 0.0, 0.0),
        preset(0x4000_0042, 0.0, 0.0),
        preset(0x4000_004C, 0.0, 0.0),
        preset(0x4000_0043, 0.0, 0.0),
        preset(0x4000_0044, 0.0, 0.0),
        preset(0x4000_0049, 0.0, 0.0),
        preset(0x4000_0020, 0.0, 0.0),
        preset(0x4000_005D, 0.0, 0.0),
        preset(0x4000_005E, 0.0, 0.0),
        preset(0x4000_005F, 0.0, 0.0),
        preset(0x4000_0074, 0.0, 0.0),
        preset(0x4000_0061, 0.0, 0.0),
        preset(0x4000_0062, 0.0, 0.0),
        preset(0x4000_0063, 0.0, 0.0),
        preset(0x4000_0076, 0.0, 0.0),
        preset(0x4000_0064, 1.0, 1.0),
        preset(0x4000_0065, 0.5, 0.5),
    ],
    // Level 2.
    &[
        preset(0x4000_004F, 2.0, 2.0),
        preset(0x4000_004D, 0.0, 0.0),
        preset(0x4000_004E, 0.0, 0.0),
        preset(0x4000_0021, 0.0, 0.0),
        preset(0x4000_0042, 0.0, 0.0),
        preset(0x4000_004C, 0.0, 0.0),
        preset(0x4000_0043, 1.0, 1.0),
        preset(0x4000_0044, 0.0, 0.0),
        preset(0x4000_0049, 0.0, 0.0),
        preset(0x4000_0020, 0.0, 0.0),
        preset(0x4000_005D, 0.0, 0.0),
        preset(0x4000_005E, 0.0, 0.0),
        preset(0x4000_005F, 0.0, 0.0),
        preset(0x4000_0074, 0.0, 0.0),
        preset(0x4000_0061, 1.0, 1.0),
        preset(0x4000_0062, 1.0, 1.0),
        preset(0x4000_0063, 1.0, 1.0),
        preset(0x4000_0076, 1.0, 1.0),
        preset(0x4000_0064, 1.0, 1.0),
        preset(0x4000_0065, 0.0, 0.0),
    ],
    // Level 3.
    &[
        preset(0x4000_004F, 3.0, 3.0),
        preset(0x4000_004D, 1.0, 1.0),
        preset(0x4000_004E, 1.0, 1.0),
        preset(0x4000_0021, 1.0, 1.0),
        preset(0x4000_0042, 0.0, 0.0),
        preset(0x4000_004C, 0.0, 0.0),
        preset(0x4000_0043, 2.0, 2.0),
        preset(0x4000_0044, 0.0, 0.0),
        preset(0x4000_0049, 0.0, 0.0),
        preset(0x4000_0020, 0.0, 0.0),
        preset(0x4000_005D, 1.0, 1.0),
        preset(0x4000_005E, 1.0, 1.0),
        preset(0x4000_005F, 0.0, 0.0),
        preset(0x4000_0074, 0.0, 0.0),
        preset(0x4000_0061, 1.0, 1.0),
        preset(0x4000_0062, 1.0, 1.0),
        preset(0x4000_0063, 1.0, 1.0),
        preset(0x4000_0076, 1.0, 1.0),
        preset(0x4000_0064, 1.0, 1.0),
        preset(0x4000_0065, -0.5, -0.5),
    ],
    // Level 4: the `LandscapeCloudShadows` default and the `AdaptiveDegrade` default are the
    // level's 0 (they are set after the jump to the shared tail).
    &[
        preset(0x4000_004F, 3.0, 3.0),
        preset(0x4000_004D, 2.0, 2.0),
        preset(0x4000_004E, 2.0, 2.0),
        preset(0x4000_0021, 2.0, 2.0),
        preset(0x4000_0042, 1.0, 1.0),
        preset(0x4000_004C, 2.0, 2.0),
        preset(0x4000_0043, 3.0, 3.0),
        preset(0x4000_0044, 1.0, 1.0),
        preset(0x4000_0049, 1.0, 1.0),
        preset(0x4000_0020, 0.0, 0.0),
        preset(0x4000_005D, 2.0, 2.0),
        preset(0x4000_005E, 1.0, 1.0),
        preset(0x4000_005F, 0.0, 0.0),
        preset(0x4000_0074, 1.0, 1.0),
        preset(0x4000_0061, 1.0, 1.0),
        preset(0x4000_0062, 1.0, 1.0),
        preset(0x4000_0063, 1.0, 1.0),
        preset(0x4000_0076, 2.0, 2.0),
        preset(0x4000_0064, 0.0, 0.0),
        preset(0x4000_0065, 0.0, 0.0),
    ],
    // Level 5.
    &[
        preset(0x4000_004F, 4.0, 4.0),
        preset(0x4000_004D, 2.0, 2.0),
        preset(0x4000_004E, 2.0, 2.0),
        preset(0x4000_0021, 2.0, 2.0),
        preset(0x4000_0042, 1.0, 1.0),
        preset(0x4000_004C, 2.0, 2.0),
        preset(0x4000_0043, 4.0, 4.0),
        preset(0x4000_0044, 2.0, 2.0),
        preset(0x4000_0049, 1.0, 1.0),
        preset(0x4000_0020, 1.0, 1.0),
        preset(0x4000_005D, 2.0, 2.0),
        preset(0x4000_005E, 1.0, 1.0),
        preset(0x4000_005F, 0.0, 0.0),
        preset(0x4000_0074, 1.0, 1.0),
        preset(0x4000_0061, 1.0, 1.0),
        preset(0x4000_0062, 1.0, 1.0),
        preset(0x4000_0063, 1.0, 1.0),
        preset(0x4000_0076, 2.0, 2.0),
        preset(0x4000_0064, 0.0, 0.0),
        preset(0x4000_0065, 0.0, 0.0),
    ],
];

const fn preset(key: u32, value: f32, default: f32) -> Preset {
    Preset {
        key,
        value,
        default,
    }
}

/// The enum choices of the graphics enums: `(value, label)` pairs, in `AssignChoice` order.
const TEXTURE_DETAIL: &[(i32, u32)] = &[
    (0, 0x4000_0002),
    (1, 0x4000_0003),
    (2, 0x4000_0004),
    (3, 0x4000_0005),
    (4, 0x4000_0006),
];
const LEVELS_3: &[(i32, u32)] = &[(0, 0x4000_0003), (1, 0x4000_0004), (2, 0x4000_0005)];
const WATER: &[(i32, u32)] = &[(0, 0x4000_0001), (1, 0x4000_0003), (2, 0x4000_0005)];
const SHORE: &[(i32, u32)] = &[(0, 0x4000_0001), (1, 0x4000_0004), (2, 0x4000_0005)];
const OBJECT_SHADOWS: &[(i32, u32)] = &[
    (0, 0x4000_0001),
    (1, 0x4000_0007),
    (2, 0x4000_0008),
    (3, 0x4000_000A),
];
const LANDSCAPE_DRAW: &[(i32, u32)] = &[
    (0, 0x4000_0002),
    (1, 0x4000_0003),
    (2, 0x4000_0004),
    (3, 0x4000_0005),
    (4, 0x4000_0006),
];
const FRILL: &[(i32, u32)] = &[
    (0, 0x4000_0001),
    (1, 0x4000_0003),
    (2, 0x4000_0004),
    (3, 0x4000_0005),
];
const AVATAR_FIRST_PERSON: &[(i32, u32)] = &[(0, 0x4000_0012), (1, 0x4000_0014), (2, 0x4000_0013)];

/// Registers the graphics and avatar preferences (`Render::InitRenderingPreferences`) and links
/// them to the renderer. Returns how many were registered.
pub fn register_graphics(bank: &mut PreferenceBank, names: &dyn NameTable) -> usize {
    let g = GRAPHICS_SECTION;
    let enumeration = |key, choices: &[(i32, u32)], default| {
        Pref::enumeration(
            g,
            key,
            Some(RENDER_MAP),
            choices
                .iter()
                .map(|&(value, label)| Choice { value, label })
                .collect(),
            default,
        )
    };
    let flag = |key, default| Some(Pref::boolean(g, key, default));
    let defs = [
        enumeration(0x4000_004F, TEXTURE_DETAIL, 2),
        enumeration(0x4000_004D, LEVELS_3, 1),
        enumeration(0x4000_004E, LEVELS_3, 1),
        enumeration(WATER_REFLECTIONS, WATER, 0),
        enumeration(LANDSCAPE_SHORE_EFFECTS, SHORE, 1),
        enumeration(DYNAMIC_OBJECT_SHADOWS, OBJECT_SHADOWS, 0),
        enumeration(0x4000_0076, LEVELS_3, 1),
        enumeration(LANDSCAPE_DRAW_DISTANCE, LANDSCAPE_DRAW, 2),
        enumeration(0x4000_0044, FRILL, 0),
        flag(LANDSCAPE_STATIC_OBJECT_SHADOWS, false),
        flag(LANDSCAPE_CLOUD_SHADOWS, false),
        flag(LIGHT_CORONAS, false),
        flag(0x4000_005E, true),
        flag(0x4000_005F, false),
        flag(0x4000_0074, false),
        flag(0x4000_0061, true),
        flag(0x4000_0062, true),
        flag(0x4000_0063, true),
        flag(0x4000_0064, false),
        Pref::float(g, 0x4000_0065, 0.0, -1.0, 1.0),
        Pref::float(g, SCREEN_BRIGHTNESS, 0.0, -0.2, 1.0),
    ];
    let mut registered = register_all(bank, names, defs.into_iter().flatten(), "Render");
    let avatar = [
        Pref::enumeration(
            AVATAR_SECTION,
            AVATAR_VISIBLE_IN_FIRST_PERSON,
            Some(AVATAR_MAP),
            AVATAR_FIRST_PERSON
                .iter()
                .map(|&(value, label)| Choice { value, label })
                .collect(),
            2,
        ),
        Some(Pref::boolean(AVATAR_SECTION, 0x4000_006D, true)),
    ];
    registered += register_all(bank, names, avatar.into_iter().flatten(), "Render");
    registered
}

/// Registers the display preferences (`Device::InitializeRenderingPreferences`, section
/// `0x4000000B`) and links them to the device. `DisplayResolution` waits for the monitor's modes
/// ([`register_display_resolution`]). Returns how many were registered.
pub fn register_device(bank: &mut PreferenceBank, names: &dyn NameTable) -> usize {
    let flag = |key, default| Pref::boolean(DEVICE_SECTION, key, default);
    let defs = [
        flag(FULL_SCREEN, true),
        flag(0x4000_0005, false),
        flag(0x4000_0007, false),
        flag(0x4000_0006, true),
        flag(0x4000_0060, false),
    ];
    register_all(bank, names, defs, "Device")
}

/// `PrefDisplayModeType::nModeDesc`: the width in the high 16 bits, the height in the low.
pub fn mode_desc((width, height): (u32, u32)) -> i32 {
    ((width << 16) | (height & 0xFFFF)) as i32
}

/// The width and height of a `nModeDesc`.
pub fn mode_of(desc: i32) -> (u32, u32) {
    let desc = desc as u32;
    (desc >> 16, desc & 0xFFFF)
}

/// `Device::InitializeRenderingPreferences`'s list: the modes over 799 x 599 (retail also wanted
/// 32 bits a pixel, which every mode here is), each size once, with 1024 x 768 always in it,
/// smallest first (by pixels, then width).
pub fn display_modes(modes: impl IntoIterator<Item = (u32, u32)>) -> Vec<(u32, u32)> {
    let mut list: Vec<(u32, u32)> = modes
        .into_iter()
        .filter(|&(w, h)| w > 799 && h > 599 && w < 0x8000 && h < 0x1_0000)
        .chain(std::iter::once(DEFAULT_MODE))
        .collect();
    list.sort_by_key(|&(w, h)| (u64::from(w) * u64::from(h), w, h));
    list.dedup();
    list
}

/// Registers `DisplayResolution` (section `0x4000000B`) over `modes` (a [`display_modes`] list):
/// each mode a choice valued and labelled by its `nModeDesc`, 1024 x 768 the default. A value
/// saved for a mode this monitor does not list falls back to the default. Returns whether it was
/// registered.
pub fn register_display_resolution(
    bank: &mut PreferenceBank,
    names: &dyn NameTable,
    modes: &[(u32, u32)],
) -> bool {
    let choices = modes
        .iter()
        .map(|&mode| Choice {
            value: mode_desc(mode),
            label: mode_desc(mode) as u32,
        })
        .collect();
    let Some(def) = Pref::enumeration(
        DEVICE_SECTION,
        DISPLAY_RESOLUTION,
        None,
        choices,
        mode_desc(DEFAULT_MODE),
    ) else {
        return false;
    };
    register_all(bank, names, [def], "Device") == 1
}

/// The display mode the bank's `DisplayResolution` holds, once it is registered.
pub fn display_resolution(bank: &PreferenceBank) -> Option<(u32, u32)> {
    match bank.find(DISPLAY_RESOLUTION)?.value {
        Value::Int(desc) => Some(mode_of(desc)),
        _ => None,
    }
}

fn register_all(
    bank: &mut PreferenceBank,
    names: &dyn NameTable,
    defs: impl IntoIterator<Item = Pref>,
    owner: &'static str,
) -> usize {
    let mut registered = 0;
    for def in defs {
        let key = def.key;
        if bank.register(def, names).is_some() {
            bank.link(key, owner);
            registered += 1;
        }
    }
    registered
}

/// `Render::SetOverallGraphicsQuality(level)`: the preset's values and defaults for `level` (1 to
/// 5), then the graphics section applied. Returns whether the level has a preset. The section is
/// applied either way, as the function does.
pub fn apply_quality(bank: &mut PreferenceBank, level: u32) -> bool {
    let preset = usize::try_from(level)
        .ok()
        .and_then(|l| l.checked_sub(1))
        .and_then(|i| QUALITY_PRESETS.get(i));
    if let Some(lines) = preset {
        for line in lines.iter() {
            if let Some(pref) = bank.find_mut(line.key) {
                match pref.kind() {
                    ac2_prefs::Kind::Bool => {
                        pref.set_default(Value::Bool(line.default != 0.0));
                        pref.set_bool(line.value != 0.0);
                    }
                    ac2_prefs::Kind::Float => {
                        pref.set_default(Value::Float(line.default));
                        pref.set_float(line.value);
                    }
                    ac2_prefs::Kind::Enum => {
                        let (value, default) = (line.value as i32, line.default as i32);
                        pref.set_default(Value::Int(default));
                        pref.set_choice(value);
                    }
                    ac2_prefs::Kind::Text => {}
                }
            }
        }
    }
    bank.apply(GRAPHICS_SECTION);
    preset.is_some()
}

/// `ID_Graphics_LandscapeFrillDistance`: choice 1 is the short distance (`FrillRender`'s 0.65).
pub const LANDSCAPE_FRILL_DISTANCE: u32 = 0x4000_0044;

/// The graphics preferences the scene reads (`WorldScene::set_options`): the far ring's radius
/// from `LandscapeDrawDistance`, the frill settings from `LandscapeFrillDistance`, and the object
/// degrade settings (`ModelDetail`, `MaterialDetail`, `SceneryDrawDistance`, `AdaptiveDegrade`,
/// `AdaptiveDegradeBias`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneOptions {
    /// The far radius in blocks (`far_radius_for_draw_distance`).
    pub far_radius: i32,
    pub frill: FrillSettings,
    pub degrade: DegradeOptions,
    /// The sun shadows of bodies and objects (`DynamicObjectShadows`).
    pub object_shadows: ObjectShadows,
    /// The sky lights' lens flares (`LightCoronas`; `LScapeSky::Draw(1)` skips
    /// `LensFlareEffect::Draw` while it is off).
    pub light_coronas: bool,
    /// The near landscape's static shadow map (`LandscapeStaticObjectShadows`): without it the
    /// terrain takes no static object shadows and no cloud shadows.
    pub static_shadows: bool,
    /// The cloud shadows over the landscape (`LandscapeCloudShadows`, with the map only).
    pub cloud_shadows: bool,
    /// FX lights and the light pools on the ground (`LightSourceEffects`).
    pub light_source_effects: bool,
    /// `VolumetricFogEffects`: off holds back the particle systems that respond to repulsors
    /// (`ParticleSystem::Think` and `Render`).
    pub volumetric_fog: bool,
    /// `WaterReflections`: 0 none, 1 the reflection, 2 rippled as well.
    pub water_reflections: u8,
    /// `LandscapeShoreEffects`: 2 draws the breaking water on the shores.
    pub shore_effects: u8,
    /// `ScreenBrightness` (-0.2..1, 0 neutral; `SceneTool::SetGammaBrightness`).
    pub brightness: f32,
    /// `AvatarVisibleInFirstPerson`: 0 never, 1 always, 2 (the default) except while the player is
    /// in mode `0x40000001` (`CPhysicsObj`'s draw and `CameraManager::UpdateCamera`).
    pub avatar_first_person: u8,
}

impl Default for SceneOptions {
    /// The scene's own defaults: draw distance 2 (radius 4), the long frill distance, medium
    /// object shadows, lens flares and the static shadow map without cloud shadows (quality
    /// level 4's landscape).
    fn default() -> Self {
        Self {
            far_radius: far_radius_for_draw_distance(2),
            frill: FrillSettings {
                copies: grass_copies_for_draw_distance(2),
                ..FrillSettings::default()
            },
            degrade: DegradeOptions::default(),
            object_shadows: ObjectShadows::Medium,
            light_coronas: true,
            static_shadows: true,
            cloud_shadows: false,
            light_source_effects: true,
            volumetric_fog: true,
            water_reflections: 2,
            shore_effects: 2,
            brightness: 0.0,
            avatar_first_person: 2,
        }
    }
}

/// `ID_Graphics_DynamicObjectShadows` (`Render::m_Prefs.DynamicObjectShadows`).
pub const DYNAMIC_OBJECT_SHADOWS: u32 = 0x4000_004C;

/// The `DynamicObjectShadows` choices. Retail drew 1 as a dark disc decal under each moving
/// object (`WorldRender::CreateDiscShadows`), 2 as stencil shadow volumes of the moving objects
/// (`SceneTool::GenerateGlobalShadowVolume`, `RenderGlobalVolumetricShadows`) and 3 as the same
/// volumes self-shadowing. The client draws every level above 0 as sun shadow maps of bodies and
/// objects (`ac2_gpu::SunShadows`), sized by the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectShadows {
    Off,
    Low,
    Medium,
    High,
}

impl ObjectShadows {
    /// The preference's value (0 to 3; larger values are high).
    pub fn from_preference(value: i32) -> Self {
        match value {
            i32::MIN..=0 => Self::Off,
            1 => Self::Low,
            2 => Self::Medium,
            _ => Self::High,
        }
    }
}

/// The scene's options as the bank holds them. A preference that is not registered keeps its
/// default.
pub fn scene_options(bank: &PreferenceBank) -> SceneOptions {
    let defaults = SceneOptions::default();
    let draw_distance = landscape_draw_distance(bank);
    let far_radius = draw_distance.map_or(defaults.far_radius, far_radius_for_draw_distance);
    let short = matches!(
        bank.find(LANDSCAPE_FRILL_DISTANCE).map(|pref| &pref.value),
        Some(Value::Int(1))
    );
    SceneOptions {
        far_radius,
        frill: FrillSettings {
            short_distance: short,
            copies: draw_distance.map_or(defaults.frill.copies, grass_copies_for_draw_distance),
            ..defaults.frill
        },
        degrade: degrade_options(bank),
        object_shadows: match bank.find(DYNAMIC_OBJECT_SHADOWS).map(|pref| &pref.value) {
            Some(Value::Int(value)) => ObjectShadows::from_preference(*value),
            _ => defaults.object_shadows,
        },
        light_coronas: flag(bank, LIGHT_CORONAS, defaults.light_coronas),
        static_shadows: flag(
            bank,
            LANDSCAPE_STATIC_OBJECT_SHADOWS,
            defaults.static_shadows,
        ),
        cloud_shadows: flag(bank, LANDSCAPE_CLOUD_SHADOWS, defaults.cloud_shadows),
        light_source_effects: light_source_effects(bank),
        volumetric_fog: flag(bank, VOLUMETRIC_FOG_EFFECTS, defaults.volumetric_fog),
        water_reflections: match bank.find(WATER_REFLECTIONS).map(|pref| &pref.value) {
            Some(Value::Int(level)) => (*level).clamp(0, 2) as u8,
            _ => defaults.water_reflections,
        },
        shore_effects: match bank.find(LANDSCAPE_SHORE_EFFECTS).map(|pref| &pref.value) {
            Some(Value::Int(level)) => (*level).clamp(0, 2) as u8,
            _ => defaults.shore_effects,
        },
        brightness: match bank.find(SCREEN_BRIGHTNESS).map(|pref| &pref.value) {
            Some(Value::Float(value)) => *value,
            _ => defaults.brightness,
        },
        avatar_first_person: match bank
            .find(AVATAR_VISIBLE_IN_FIRST_PERSON)
            .map(|pref| &pref.value)
        {
            Some(Value::Int(level)) => (*level).clamp(0, 2) as u8,
            _ => defaults.avatar_first_person,
        },
    }
}

/// A boolean preference's value, `default` when it is not registered.
fn flag(bank: &PreferenceBank, key: u32, default: bool) -> bool {
    match bank.find(key).map(|pref| &pref.value) {
        Some(Value::Bool(on)) => *on,
        _ => default,
    }
}

/// `ID_Graphics_ScreenBrightness`.
pub const SCREEN_BRIGHTNESS: u32 = 0x4000_004B;
/// `ID_Graphics_LandscapeShoreEffects`.
pub const LANDSCAPE_SHORE_EFFECTS: u32 = 0x4000_005D;
/// `ID_Graphics_WaterReflections`.
pub const WATER_REFLECTIONS: u32 = 0x4000_0021;
/// `ID_Graphics_LandscapeStaticObjectShadows`.
pub const LANDSCAPE_STATIC_OBJECT_SHADOWS: u32 = 0x4000_0049;
/// `ID_Graphics_LandscapeCloudShadows`.
pub const LANDSCAPE_CLOUD_SHADOWS: u32 = 0x4000_0020;

/// `ID_Graphics_LightCoronas` (`Render::m_Prefs.LightCoronas`).
pub const LIGHT_CORONAS: u32 = 0x4000_0042;

/// `ModelDetail` (`Render::m_Prefs.ModelDetail`).
pub const MODEL_DETAIL: u32 = 0x4000_004E;
/// `MaterialDetail`.
pub const MATERIAL_DETAIL: u32 = 0x4000_004D;
/// `SceneryDrawDistance`.
pub const SCENERY_DRAW_DISTANCE: u32 = 0x4000_0076;
/// `AdaptiveDegrade`.
pub const ADAPTIVE_DEGRADE: u32 = 0x4000_0064;
/// `AdaptiveDegradeBias`.
pub const ADAPTIVE_DEGRADE_BIAS: u32 = 0x4000_0065;

/// The degrade settings `CPartArray::Draw` reads, as the bank holds them; a preference that is not
/// registered keeps its default.
pub fn degrade_options(bank: &PreferenceBank) -> DegradeOptions {
    let defaults = DegradeOptions::default();
    let level = |key, default| match bank.find(key).map(|pref| &pref.value) {
        Some(Value::Int(v)) => u32::try_from(*v).unwrap_or(default),
        _ => default,
    };
    DegradeOptions {
        model_detail: level(MODEL_DETAIL, defaults.model_detail),
        material_detail: level(MATERIAL_DETAIL, defaults.material_detail),
        scenery_draw_distance: level(SCENERY_DRAW_DISTANCE, defaults.scenery_draw_distance),
        adaptive_degrade: match bank.find(ADAPTIVE_DEGRADE).map(|pref| &pref.value) {
            Some(Value::Bool(on)) => *on,
            _ => defaults.adaptive_degrade,
        },
        adaptive_degrade_bias: match bank.find(ADAPTIVE_DEGRADE_BIAS).map(|pref| &pref.value) {
            Some(Value::Float(bias)) => *bias,
            _ => defaults.adaptive_degrade_bias,
        },
    }
}

/// Whether FX light sources are placed (`LightSourceEffects`). A preference that is not
/// registered keeps its default, on.
pub fn light_source_effects(bank: &PreferenceBank) -> bool {
    match bank.find(LIGHT_SOURCE_EFFECTS).map(|pref| &pref.value) {
        Some(Value::Bool(on)) => *on,
        _ => true,
    }
}

/// `DecalEffects` (`Render::m_Prefs.DecalEffects`): `FXManager::DoDecalFX` makes no decal while it
/// is off.
pub const DECAL_EFFECTS: u32 = 0x4000_0062;

/// Whether FX decals are made (`DecalEffects`). A preference that is not registered keeps its
/// default, on.
pub fn decal_effects(bank: &PreferenceBank) -> bool {
    match bank.find(DECAL_EFFECTS).map(|pref| &pref.value) {
        Some(Value::Bool(on)) => *on,
        _ => true,
    }
}

/// Whether FX streaks are made and drawn (`StreakEffects`). A preference that is not registered
/// keeps its default, on.
pub fn streak_effects(bank: &PreferenceBank) -> bool {
    match bank.find(STREAK_EFFECTS).map(|pref| &pref.value) {
        Some(Value::Bool(on)) => *on,
        _ => true,
    }
}

/// The renderer's settings as the bank holds them: the landscape's draw distance (the choice
/// `LandscapeDrawDistance`), or `None` when it is not registered.
pub fn landscape_draw_distance(bank: &PreferenceBank) -> Option<u32> {
    match bank.find(LANDSCAPE_DRAW_DISTANCE)?.value {
        Value::Int(v) => u32::try_from(v).ok(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "prefs_tests.rs"]
mod prefs_tests;

/// `OverallGraphicsQuality` (key `0x40000073` in the graphics section): the saved quality level.
pub const OVERALL_GRAPHICS_QUALITY: u32 = 0x4000_0073;
/// `[Graphics] OverallGraphicsQuality` in the file (literal in the client).
const OVERALL_GRAPHICS_QUALITY_FILE: (&str, &str) = ("Graphics", "OverallGraphicsQuality");

/// `Preferences::InqBool(0x40000004, 0x40000073)`: the saved graphics quality (the render applies
/// it when it is 1 to 5).
pub fn overall_graphics_quality<N: NameTable>(file: &PrefsFile<N>) -> Option<u32> {
    file.inq_bool(GRAPHICS_SECTION, OVERALL_GRAPHICS_QUALITY)
}

/// `Render::SaveOverallGraphicsQuality(level)`: the level is set for the session and written to
/// the file at once, on a worker thread.
pub fn save_overall_graphics_quality<N: NameTable>(
    file: &mut PrefsFile<N>,
    level: u32,
) -> JoinHandle<io::Result<()>> {
    file.save_uint_now(
        (GRAPHICS_SECTION, OVERALL_GRAPHICS_QUALITY),
        OVERALL_GRAPHICS_QUALITY_FILE,
        level,
    )
}
