//! `StringState` (`DB_TYPE_STRING_STATE` 0x40000055, DIDs 0x58000000..=0x58FFFFFE in the local
//! DAT): the language's number formatting, plural rules and meta letters, read through
//! `StringStateInfo`.
//!
//! `StringStateInfo::Startup` loads it with `DBObj::GetByEnum(0x40000001, 0x40000004,
//! 0x40000055)` (the Legions DAT resolves that to 0x58000000). `StringState::StreamPack`:
//! version (u32), base (u16), decimal digits (u16), leading zero (i32), grouping size (u16),
//! alignment, numerals, decimal separator, grouping separator and negative number format
//! (`PStringBase<unsigned short>`), the five "is singular" flags (u32), the treasure prefix,
//! middle and suffix letters and the male and female player letters (strings), twenty u32
//! settings (gore, blood and the IME appearance), the additional settings string and flags.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};

use crate::string_meta_text::wide;
use crate::string_table::read_wide_pstring;
use crate::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

/// `DB_TYPE_STRING_STATE`'s DID range.
pub const STRING_STATE_FIRST: u32 = 0x5800_0000;
pub const STRING_STATE_LAST: u32 = 0x58FF_FFFE;
/// `StringStateInfo::Startup`'s enum and enum group (`DBObj::GetByEnum`).
pub const STRING_STATE_ENUM: u32 = 0x4000_0001;
pub const STRING_STATE_ENUM_GROUP: u32 = 0x4000_0004;

/// A `StringState`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringState {
    pub version: u32,
    pub base: u16,
    pub num_decimal_digits: u16,
    pub leading_zero: i32,
    pub grouping_size: u16,
    /// The digit characters by value (`GetNumerals`).
    pub numerals: Vec<u16>,
    pub decimal_separator: Vec<u16>,
    pub grouping_separator: Vec<u16>,
    /// A `%s` format for negative numbers (`GetNegNumFormat`).
    pub negative_number_format: Vec<u16>,
    /// `isZeroSingular`, `isOneSingular`, `isNegativeOneSingular`, `isTwoOrMoreSingular`,
    /// `isNegativeTwoOrLessSingular`.
    pub singular: [u32; 5],
    pub treasure_prefix_letters: Vec<u16>,
    pub treasure_middle_letters: Vec<u16>,
    pub treasure_suffix_letters: Vec<u16>,
    pub male_player_letters: Vec<u16>,
    pub female_player_letters: Vec<u16>,
    /// `m_topGoreLevel`, `m_topBloodColor`, `m_ImeEnabledSetting`, the IME symbol, candidate
    /// and composition colours, `m_otherIME`, `m_wordWrapOnSpace`, in pack order.
    pub settings: [u32; 20],
    pub additional_settings: Vec<u16>,
    pub additional_flags: u32,
}

impl StringState {
    /// `StringStateInfo::IsNumberSingular`.
    pub fn is_number_singular(&self, num: i32) -> bool {
        let [zero, one, neg_one, two_or_more, neg_two_or_less] = self.singular;
        match num {
            0 => zero != 0,
            1 => one != 0,
            -1 => neg_one != 0,
            n if n > 1 => two_or_more != 0,
            _ => neg_two_or_less != 0,
        }
    }
}

/// `StringTableMetaLanguage::IsNumberSingular`: the state's rule, or `num == 1` without one.
pub fn is_number_singular(state: Option<&StringState>, num: i32) -> bool {
    match state {
        Some(state) => state.is_number_singular(num),
        None => num == 1,
    }
}

/// `StringTableMetaLanguage::GetMaleMetaLetters` (`"mn"` without a state).
pub fn male_meta_letters(state: Option<&StringState>) -> Vec<u16> {
    state.map_or_else(|| wide("mn"), |s| s.male_player_letters.clone())
}

/// `StringTableMetaLanguage::GetFemaleMetaLetters` (`"fn"` without a state).
pub fn female_meta_letters(state: Option<&StringState>) -> Vec<u16> {
    state.map_or_else(|| wide("fn"), |s| s.female_player_letters.clone())
}

/// `StringTableMetaLanguage::GetTreasureMetaLetters`: prefix, middle and suffix letters
/// (`"v"`, `"mfnp"`, `"s"` without a state).
pub fn treasure_meta_letters(state: Option<&StringState>) -> [Vec<u16>; 3] {
    match state {
        Some(s) => [
            s.treasure_prefix_letters.clone(),
            s.treasure_middle_letters.clone(),
            s.treasure_suffix_letters.clone(),
        ],
        None => [wide("v"), wide("mfnp"), wide("s")],
    }
}

impl ResourcePackedDecode for StringState {
    const TYPE_NAME: &'static str = "AC2 StringState";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let mut c = ArchiveCursor::new(bytes);
        let mut state = StringState {
            version: c.read_u32()?,
            base: c.read_u16()?,
            num_decimal_digits: c.read_u16()?,
            leading_zero: c.read_u32()? as i32,
            grouping_size: c.read_u16()?,
            ..Default::default()
        };
        c.align_to(4)?;
        state.numerals = read_wide_pstring(&mut c)?;
        state.decimal_separator = read_wide_pstring(&mut c)?;
        state.grouping_separator = read_wide_pstring(&mut c)?;
        state.negative_number_format = read_wide_pstring(&mut c)?;
        for flag in &mut state.singular {
            *flag = c.read_u32()?;
        }
        state.treasure_prefix_letters = read_wide_pstring(&mut c)?;
        state.treasure_middle_letters = read_wide_pstring(&mut c)?;
        state.treasure_suffix_letters = read_wide_pstring(&mut c)?;
        state.male_player_letters = read_wide_pstring(&mut c)?;
        state.female_player_letters = read_wide_pstring(&mut c)?;
        for setting in &mut state.settings {
            *setting = c.read_u32()?;
        }
        state.additional_settings = read_wide_pstring(&mut c)?;
        state.additional_flags = c.read_u32()?;
        if c.remaining() != 0 {
            return Err(ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context: "trailing bytes after StringState",
            });
        }
        Ok(state)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + 256
    }
}

/// The `StringState` `did` from the local DAT.
pub fn load_string_state(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<StringState>>> {
    store.load_cached_packed::<StringState>(TypedResourceRef::new(DatFileKind::Local, did, 0))
}

/// The command line's overrides of the top gore and blood levels (`StringStateInfo`'s two
/// statics, Legions `DAT_00af6eac` and `DAT_00af6eb0`; 0 is none).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TopLevelOverrides {
    /// `StringStateInfo::SetTopGoreLevelOverride`.
    pub gore: u32,
    /// `StringStateInfo::SetTopBloodColorOverride`.
    pub blood: u32,
}

impl TopLevelOverrides {
    /// `Turbine::Client::EvaluateCommandLineArg` `maxgore <level>` (Legions client.cpp,
    /// "Forces the maximum gore level (fun,none,low,normal)"): the value in lower case, `fun` 1,
    /// `none` 2, `low` 3, `normal` 4; anything else is refused with retail's message.
    pub fn set_max_gore(&mut self, value: &str) -> Result<(), &'static str> {
        self.gore = match value.to_ascii_lowercase().as_str() {
            "fun" => 1,
            "none" => 2,
            "low" => 3,
            "normal" => 4,
            _ => return Err("gore level must be one of these: (fun,none,low,normal)"),
        };
        Ok(())
    }

    /// `greenblood` ("Forces the game to use green blood"): `SetTopBloodColorOverride(1)`.
    pub fn set_green_blood(&mut self) {
        self.blood = 1;
    }

    /// `StringStateInfo::GetTopGoreLevel` and `GetTopBloodLevel` (Legions 0x00602330,
    /// 0x00602370): a set override, else the `StringState`'s own word (`settings` 0 and 1).
    pub fn top_levels(self, state: &StringState) -> (u32, u32) {
        let pick = |over: u32, own: u32| if over != 0 { over } else { own };
        (
            pick(self.gore, state.settings[0]),
            pick(self.blood, state.settings[1]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_line_overrides_the_top_gore_and_blood_levels() {
        let state = StringState {
            settings: {
                let mut s = [0; 20];
                s[0] = 4;
                s[1] = 2;
                s
            },
            ..Default::default()
        };
        let mut over = TopLevelOverrides::default();
        assert_eq!(over.top_levels(&state), (4, 2), "the state's own");
        over.set_max_gore("LOW").expect("low");
        over.set_green_blood();
        assert_eq!(over.top_levels(&state), (3, 1));
        for (text, level) in [("fun", 1), ("none", 2), ("normal", 4)] {
            over.set_max_gore(text).expect(text);
            assert_eq!(over.gore, level);
        }
        assert!(over.set_max_gore("gory").is_err());
    }

    #[test]
    fn singular_rules_and_fallbacks() {
        let state = StringState {
            singular: [0, 1, 0, 0, 0],
            male_player_letters: wide("mn"),
            ..Default::default()
        };
        assert!(state.is_number_singular(1));
        assert!(!state.is_number_singular(0));
        assert!(!state.is_number_singular(7));
        assert!(!state.is_number_singular(-3));
        assert!(is_number_singular(None, 1) && !is_number_singular(None, 2));
        assert_eq!(male_meta_letters(None), wide("mn"));
        assert_eq!(female_meta_letters(None), wide("fn"));
        assert_eq!(treasure_meta_letters(None)[1], wide("mfnp"));
    }
}
