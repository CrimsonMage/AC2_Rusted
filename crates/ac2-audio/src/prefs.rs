//! The sound preferences (`SoundManager::InitPrefs`, section `0x40000007`): the eight keys the
//! sound manager registers in the bank, with their defaults and ranges as the function sets them,
//! and the [`SoundPreference`]s the manager takes from the bank's values. Its handler is the
//! bank's `SoundManager` owner (`PreferenceBank::Link`).
//!
//! The keys (retail's `ID_Sound_*` names come from the DAT's enum tables; the IDs are code):
//! `0x40000040` FX volume (float, default 1, range 0 to 1), `0x40000041` music volume (the same),
//! `0x4000003D` player music (the same), `0x40000056` ambient volume (the same), `0x40000057`
//! sound features (an enum: 0 software, 1 hardware, 2 EAX, default 1; its names are the mapper
//! `0x4000004A`), `0x40000058` playing sounds (float, default 32, range 8 to 64), and `0x40000033`
//! disable sound and `0x40000034` disable music (bools, default false).

use ac2_prefs::{Choice, NameTable, Pref, PreferenceBank};

use crate::manager::{SoundManager, SoundPreference};

/// The section the sound preferences are in.
pub const SECTION: u32 = 0x4000_0007;
/// `ID_Sound_FXVolume`.
pub const FX_VOLUME: u32 = 0x4000_0040;
/// `ID_Sound_MusicVolume`.
pub const MUSIC_VOLUME: u32 = 0x4000_0041;
/// `ID_Sound_Player_Music`.
pub const PLAYER_MUSIC: u32 = 0x4000_003D;
/// `ID_Sound_AmbientVolume`.
pub const AMBIENT_VOLUME: u32 = 0x4000_0056;
/// `ID_Sound_SoundFeatures`, and the mapper of its names.
pub const SOUND_FEATURES: u32 = 0x4000_0057;
pub const SOUND_FEATURES_MAP: u32 = 0x4000_004A;
/// `ID_Sound_NumPlayingSounds`.
pub const NUM_PLAYING_SOUNDS: u32 = 0x4000_0058;
/// `ID_Sound_DisableSound`.
pub const DISABLE_SOUND: u32 = 0x4000_0033;
/// `ID_Sound_DisableMusic`.
pub const DISABLE_MUSIC: u32 = 0x4000_0034;

/// Every key of the sound section the manager takes ([`settings`]'s keys).
pub const SOUND_KEYS: [u32; 8] = [
    FX_VOLUME,
    MUSIC_VOLUME,
    PLAYER_MUSIC,
    AMBIENT_VOLUME,
    SOUND_FEATURES,
    NUM_PLAYING_SOUNDS,
    DISABLE_SOUND,
    DISABLE_MUSIC,
];

/// The sound features' choices and their labels (`PreferenceEnum::AssignChoice`).
const FEATURES: [(i32, u32); 3] = [(0, 0x4000_000F), (1, 0x4000_0010), (2, 0x4000_0011)];
/// The sound features' default (`SetDefault(1)`).
const FEATURES_DEFAULT: i32 = 1;

/// Registers the sound manager's preferences in `bank` and links them to the sound manager.
/// Returns how many were registered (the ones whose names the tables give).
pub fn register(bank: &mut PreferenceBank, names: &dyn NameTable) -> usize {
    let volume = |key| Pref::float(SECTION, key, 1.0, 0.0, 1.0);
    let defs = [
        volume(FX_VOLUME),
        volume(MUSIC_VOLUME),
        volume(PLAYER_MUSIC),
        volume(AMBIENT_VOLUME),
        Pref::enumeration(
            SECTION,
            SOUND_FEATURES,
            Some(SOUND_FEATURES_MAP),
            FEATURES
                .iter()
                .map(|&(value, label)| Choice { value, label })
                .collect(),
            FEATURES_DEFAULT,
        ),
        Pref::float(SECTION, NUM_PLAYING_SOUNDS, 32.0, 8.0, 64.0),
        Some(Pref::boolean(SECTION, DISABLE_SOUND, false)),
        Some(Pref::boolean(SECTION, DISABLE_MUSIC, false)),
    ];
    let mut registered = 0;
    for def in defs.into_iter().flatten() {
        let key = def.key;
        if bank.register(def, names).is_some() {
            bank.link(key, "SoundManager");
            registered += 1;
        }
    }
    registered
}

/// The sound manager's settings as the bank holds them now (the keys the bank does not have are
/// left out, so the manager keeps its own default for them).
pub fn settings(bank: &PreferenceBank) -> Vec<SoundPreference> {
    let float = |key| {
        bank.find(key).and_then(|p| match p.value {
            ac2_prefs::Value::Float(f) => Some(f),
            _ => None,
        })
    };
    let flag = |key| {
        bank.find(key).and_then(|p| match p.value {
            ac2_prefs::Value::Bool(b) => Some(b),
            _ => None,
        })
    };
    let choice = |key| {
        bank.find(key).and_then(|p| match p.value {
            ac2_prefs::Value::Int(v) => u32::try_from(v).ok(),
            _ => None,
        })
    };
    let mut out = Vec::new();
    if let Some(v) = choice(SOUND_FEATURES) {
        out.push(SoundPreference::SoundFeatures(v));
    }
    if let Some(v) = float(FX_VOLUME) {
        out.push(SoundPreference::SoundVolume(v));
    }
    if let Some(v) = float(MUSIC_VOLUME) {
        out.push(SoundPreference::MusicVolume(v));
    }
    if let Some(v) = float(PLAYER_MUSIC) {
        out.push(SoundPreference::PlayerMusic(v));
    }
    if let Some(v) = float(AMBIENT_VOLUME) {
        out.push(SoundPreference::AmbientVolume(v));
    }
    if let Some(v) = float(NUM_PLAYING_SOUNDS) {
        out.push(SoundPreference::NumPlayingSounds(v));
    }
    if let Some(v) = flag(DISABLE_SOUND) {
        out.push(SoundPreference::DisableSound(v));
    }
    if let Some(v) = flag(DISABLE_MUSIC) {
        out.push(SoundPreference::DisableMusic(v));
    }
    out
}

/// Pushes the bank's sound settings to `manager` (the sound section's `HandleApply`): every
/// setting the bank holds, in [`settings`]'s order.
pub fn apply(manager: &mut SoundManager, bank: &PreferenceBank) {
    for setting in settings(bank) {
        manager.set_preference(setting);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_prefs::Value;

    /// The sound keys' names, as the DAT's enum tables give them (fixed here for the test).
    struct Names;

    impl NameTable for Names {
        fn section_name(&self, id: u32) -> Option<String> {
            (id == SECTION).then(|| "Sound".to_owned())
        }
        fn key_name(&self, id: u32) -> Option<String> {
            Some(
                match id {
                    FX_VOLUME => "SoundVolume",
                    MUSIC_VOLUME => "MusicVolume",
                    PLAYER_MUSIC => "PlayerMusic",
                    AMBIENT_VOLUME => "AmbientVolume",
                    SOUND_FEATURES => "SoundFeatures",
                    NUM_PLAYING_SOUNDS => "NumPlayingSounds",
                    DISABLE_SOUND => "DisableSound",
                    DISABLE_MUSIC => "DisableMusic",
                    _ => return None,
                }
                .to_owned(),
            )
        }
        fn enum_name(&self, map: u32, label: u32) -> Option<String> {
            (map == SOUND_FEATURES_MAP)
                .then(|| {
                    FEATURES
                        .iter()
                        .find(|(_, l)| *l == label)
                        .map(|(v, _)| ["Software", "Hardware", "EAX"][*v as usize].to_owned())
                })
                .flatten()
        }
        fn enum_label(&self, map: u32, name: &str) -> Option<u32> {
            (map == SOUND_FEATURES_MAP)
                .then(|| {
                    ["Software", "Hardware", "EAX"]
                        .iter()
                        .position(|n| n.eq_ignore_ascii_case(name))
                        .map(|i| FEATURES[i].1)
                })
                .flatten()
        }
    }

    #[test]
    fn the_registration_has_the_defaults_and_ranges_the_manager_sets() {
        let mut bank = PreferenceBank::new(b"");
        assert_eq!(register(&mut bank, &Names), 8);
        let fx = bank.find(FX_VOLUME).unwrap();
        assert_eq!(
            (fx.value.clone(), fx.min, fx.max),
            (Value::Float(1.0), 0.0, 1.0)
        );
        let num = bank.find(NUM_PLAYING_SOUNDS).unwrap();
        assert_eq!(
            (num.value.clone(), num.min, num.max),
            (Value::Float(32.0), 8.0, 64.0)
        );
        assert_eq!(bank.find(DISABLE_SOUND).unwrap().value, Value::Bool(false));
        assert_eq!(bank.find(SOUND_FEATURES).unwrap().value, Value::Int(1));
        assert_eq!(bank.owners(FX_VOLUME), &["SoundManager"]);
    }

    #[test]
    fn the_file_overrides_the_defaults_and_the_manager_takes_the_values() {
        let file = b"[Sound]\r\nSoundVolume=0.500000\r\nDisableMusic=1\r\nSoundFeatures=EAX\r\nNumPlayingSounds=99\r\n";
        let mut bank = PreferenceBank::new(file);
        register(&mut bank, &Names);
        assert_eq!(bank.find(SOUND_FEATURES).unwrap().value, Value::Int(2));
        let settings = settings(&bank);
        assert!(settings.contains(&SoundPreference::SoundVolume(0.5)));
        assert!(settings.contains(&SoundPreference::DisableMusic(true)));
        // Retail's setters do not clamp: 99 is kept as read, even outside the 8 to 64 range.
        assert!(settings.contains(&SoundPreference::NumPlayingSounds(99.0)));
    }

    #[test]
    fn the_sound_features_choice_reaches_the_manager() {
        let file = b"[Sound]\r\nSoundFeatures=EAX\r\n";
        let mut bank = PreferenceBank::new(file);
        register(&mut bank, &Names);
        assert!(settings(&bank).contains(&SoundPreference::SoundFeatures(2)));
        let mut manager =
            crate::SoundManager::new(Box::new(crate::NullSink), std::sync::Arc::new(NoSounds));
        assert_eq!(
            manager.sound_features(),
            0,
            "software until the setting applies"
        );
        apply(&mut manager, &bank);
        assert_eq!(manager.sound_features(), 2);
    }

    /// A source with no sounds: the test never loads one.
    struct NoSounds;

    impl crate::SoundSource for NoSounds {
        fn load(&self, _did: u32) -> Option<crate::LoadedSound> {
            None
        }
    }
}
