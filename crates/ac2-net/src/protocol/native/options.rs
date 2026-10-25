//! The player's options as natives: `ShortcutInfo` (0xAC), `GameplayOptionsProfile` (0x9B, the
//! Legions layout) and `UISaveLocations` (0x6F).

use crate::protocol::reader::{Reader, Writer, MAX_ITEMS};
use crate::protocol::ProtocolError;

/// `ShortcutInfo::StreamPack` (Legions 0x005C8FF0): the type, then the datum it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutInfo {
    /// 0x40000001, 0x40000004: `_data_enum`.
    Enum { kind: u32, value: u32 },
    /// 0x40000002: `_data_iid`.
    Instance(u64),
    /// 0x40000003: `_data_str` (a `wpstring`).
    Text(String),
    /// 0x40000006: `_data_dataid`.
    DataId(u32),
    /// Any other type: nothing more is packed.
    Empty(u32),
}

impl ShortcutInfo {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let kind = r.u32()?;
        Ok(match kind {
            0x4000_0001 | 0x4000_0004 => Self::Enum {
                kind,
                value: r.u32()?,
            },
            0x4000_0002 => Self::Instance(r.u64()?),
            0x4000_0003 => Self::Text(r.wide_string()?),
            0x4000_0006 => Self::DataId(r.u32()?),
            _ => Self::Empty(kind),
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        match self {
            Self::Enum { kind, value } => {
                w.u32(*kind).u32(*value);
            }
            Self::Instance(iid) => {
                w.u32(0x4000_0002).u64(*iid);
            }
            Self::Text(text) => {
                w.u32(0x4000_0003).wide_string(text);
            }
            Self::DataId(did) => {
                w.u32(0x4000_0006).u32(*did);
            }
            Self::Empty(kind) => {
                w.u32(*kind);
            }
        }
    }
}

/// A `PHashTable::StreamPack` of word pairs: the header keeps the bucket-size index.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WordTable {
    pub bucket_index: u8,
    pub pairs: Vec<(u32, u32)>,
}

impl WordTable {
    fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let header = r.u32()?;
        let n = count(header & 0x00FF_FFFF, 8, r)?;
        Ok(Self {
            bucket_index: (header >> 24) as u8,
            pairs: (0..n)
                .map(|_| Ok((r.u32()?, r.u32()?)))
                .collect::<Result<_, ProtocolError>>()?,
        })
    }

    fn write(&self, w: &mut Writer) {
        w.u32(header(self.pairs.len(), self.bucket_index));
        for (k, v) in &self.pairs {
            w.u32(*k).u32(*v);
        }
    }
}

fn count(n: u32, item: usize, r: &Reader<'_>) -> Result<usize, ProtocolError> {
    let n = n as usize;
    if n > MAX_ITEMS || n.saturating_mul(item) > r.remaining() {
        return Err(ProtocolError::Invalid("table count"));
    }
    Ok(n)
}

fn header(len: usize, bucket_index: u8) -> u32 {
    len as u32 & 0x00FF_FFFF | u32::from(bucket_index) << 24
}

/// `UILocationData::StreamPack`: the window's corners and whether it shows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiLocation {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub shown: u32,
}

/// `UISaveLocations::StreamPack`: `PHashTable<unsigned long, PHashTable<unsigned long,
/// UILocationData>>` (each table's header keeps its bucket-size index).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiSaveLocations {
    pub bucket_index: u8,
    /// Per layout: its bucket-size index and the windows' locations.
    pub layouts: Vec<(u32, u8, Vec<(u32, UiLocation)>)>,
}

impl UiSaveLocations {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let header = r.u32()?;
        let n = count(header & 0x00FF_FFFF, 8, r)?;
        let mut layouts = Vec::with_capacity(n);
        for _ in 0..n {
            let key = r.u32()?;
            let inner = r.u32()?;
            let m = count(inner & 0x00FF_FFFF, 24, r)?;
            let windows = (0..m)
                .map(|_| {
                    Ok((
                        r.u32()?,
                        UiLocation {
                            x0: r.f32()?,
                            y0: r.f32()?,
                            x1: r.f32()?,
                            y1: r.f32()?,
                            shown: r.u32()?,
                        },
                    ))
                })
                .collect::<Result<_, ProtocolError>>()?;
            layouts.push((key, (inner >> 24) as u8, windows));
        }
        Ok(Self {
            bucket_index: (header >> 24) as u8,
            layouts,
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(header(self.layouts.len(), self.bucket_index));
        for (key, bucket_index, windows) in &self.layouts {
            w.u32(*key).u32(header(windows.len(), *bucket_index));
            for (id, l) in windows {
                w.u32(*id)
                    .f32(l.x0)
                    .f32(l.y0)
                    .f32(l.x1)
                    .f32(l.y1)
                    .u32(l.shown);
            }
        }
    }
}

/// `GameplayOptionsProfile::StreamPack` as Legions packs it: the 64-bit pack bitfield, the
/// fields it names, then, when `m_bitField` has its top bit, `m_version` and the version's
/// tables (Legions adds the table under bit 0x2000 for version 6 and up). The fields keep the
/// packed values; the version upgrades `StreamPack` applies after unpacking belong to the
/// profile's users.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GameplayOptionsProfile {
    pub pack_bitfield: u64,
    /// 0x1: `m_aliasTable` (`wpstring` pairs; header bucket index, then the pairs).
    pub aliases: (u8, Vec<(String, String)>),
    /// 0x2: `m_shortcutArray`.
    pub shortcuts: Vec<ShortcutInfo>,
    /// 0x4: `m_whichShortcutSet`.
    pub which_shortcut_set: u32,
    /// 0x1000: `m_iShortcutHeight`.
    pub shortcut_height: u32,
    /// 0x8: `m_fDamageTextRangeOther`.
    pub damage_text_range_other: f32,
    /// 0x10: `m_savedUILocations`.
    pub saved_ui_locations: UiSaveLocations,
    /// 0x20: `m_radarMask`.
    pub radar_mask: u32,
    /// 0x40: `m_filterHash`.
    pub filters: WordTable,
    /// 0x80: `m_bitField`.
    pub bit_field: u32,
    /// `m_version`, packed when `m_bitField` has 0x80000000.
    pub version: Option<u32>,
    /// 0x100 (version 1 and up): `m_chatFontColors`.
    pub chat_font_colors: WordTable,
    /// 0x200 (version 1 and up): `m_chatFontSizes`.
    pub chat_font_sizes: WordTable,
    /// 0x2000 (version 6 and up, Legions only).
    pub version6_table: WordTable,
    /// 0x400 (version 2 and up): `m_chatPopupFlags`.
    pub chat_popup_flags: WordTable,
    /// 0x800 (version 2 and up): `m_windowOpacities`.
    pub window_opacities: WordTable,
}

impl GameplayOptionsProfile {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bits = r.u64()?;
        let has = |bit: u64| bits & bit != 0;
        let mut p = Self {
            pack_bitfield: bits,
            ..Self::default()
        };
        if has(0x1) {
            let header = r.u32()?;
            let n = count(header & 0x00FF_FFFF, 4, r)?;
            let pairs = (0..n)
                .map(|_| Ok((r.wide_string()?, r.wide_string()?)))
                .collect::<Result<_, ProtocolError>>()?;
            p.aliases = ((header >> 24) as u8, pairs);
        }
        if has(0x2) {
            let n = count(r.u32()?, 4, r)?;
            p.shortcuts = (0..n)
                .map(|_| ShortcutInfo::read(r))
                .collect::<Result<_, _>>()?;
        }
        if has(0x4) {
            p.which_shortcut_set = r.u32()?;
        }
        if has(0x1000) {
            p.shortcut_height = r.u32()?;
        }
        if has(0x8) {
            p.damage_text_range_other = r.f32()?;
        }
        if has(0x10) {
            p.saved_ui_locations = UiSaveLocations::read(r)?;
        }
        if has(0x20) {
            p.radar_mask = r.u32()?;
        }
        if has(0x40) {
            p.filters = WordTable::read(r)?;
        }
        if has(0x80) {
            p.bit_field = r.u32()?;
        }
        if p.bit_field & 0x8000_0000 != 0 {
            let version = r.u32()?;
            p.version = Some(version);
            for (bit, min, table) in p.versioned_mut() {
                if bits & bit != 0 && version >= min {
                    *table = WordTable::read(r)?;
                }
            }
        }
        Ok(p)
    }

    /// The versioned tables in stream order: bit, minimum version, table.
    fn versioned_mut(&mut self) -> [(u64, u32, &mut WordTable); 5] {
        [
            (0x100, 1, &mut self.chat_font_colors),
            (0x200, 1, &mut self.chat_font_sizes),
            (0x2000, 6, &mut self.version6_table),
            (0x400, 2, &mut self.chat_popup_flags),
            (0x800, 2, &mut self.window_opacities),
        ]
    }

    pub(super) fn write(&self, w: &mut Writer) {
        let bits = self.pack_bitfield;
        let has = |bit: u64| bits & bit != 0;
        w.u64(bits);
        if has(0x1) {
            w.u32(header(self.aliases.1.len(), self.aliases.0));
            for (from, to) in &self.aliases.1 {
                w.wide_string(from).wide_string(to);
            }
        }
        if has(0x2) {
            w.u32(self.shortcuts.len() as u32);
            for shortcut in &self.shortcuts {
                shortcut.write(w);
            }
        }
        if has(0x4) {
            w.u32(self.which_shortcut_set);
        }
        if has(0x1000) {
            w.u32(self.shortcut_height);
        }
        if has(0x8) {
            w.f32(self.damage_text_range_other);
        }
        if has(0x10) {
            self.saved_ui_locations.write(w);
        }
        if has(0x20) {
            w.u32(self.radar_mask);
        }
        if has(0x40) {
            self.filters.write(w);
        }
        if has(0x80) {
            w.u32(self.bit_field);
        }
        if let (true, Some(version)) = (self.bit_field & 0x8000_0000 != 0, self.version) {
            w.u32(version);
            let tables = [
                (0x100, 1, &self.chat_font_colors),
                (0x200, 1, &self.chat_font_sizes),
                (0x2000, 6, &self.version6_table),
                (0x400, 2, &self.chat_popup_flags),
                (0x800, 2, &self.window_opacities),
            ];
            for (bit, min, table) in tables {
                if has(bit) && version >= min {
                    table.write(w);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::NativeObject;
    use super::*;

    fn round_trip(object: NativeObject) -> Vec<u8> {
        let mut w = Writer::new();
        object.write(&mut w);
        let mut r = Reader::new(&w.bytes);
        assert_eq!(
            NativeObject::read(object.native_type(), &mut r).unwrap(),
            object
        );
        r.finish().unwrap();
        w.bytes
    }

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn shortcuts_pack_their_type_then_their_datum() {
        let item = ShortcutInfo::Instance(0x8000_0001_0000_0002);
        let mut expected = words(&[0x4000_0002]);
        expected.extend(0x8000_0001_0000_0002u64.to_le_bytes());
        assert_eq!(round_trip(NativeObject::ShortcutInfo(item)), expected);
        let skill = ShortcutInfo::Enum {
            kind: 0x4000_0001,
            value: 0x41,
        };
        assert_eq!(
            round_trip(NativeObject::ShortcutInfo(skill)),
            words(&[0x4000_0001, 0x41])
        );
        round_trip(NativeObject::ShortcutInfo(ShortcutInfo::Text("/hi".into())));
        assert_eq!(
            round_trip(NativeObject::ShortcutInfo(ShortcutInfo::Empty(0))),
            words(&[0])
        );
    }

    #[test]
    fn the_profile_packs_what_its_bitfield_names() {
        let profile = GameplayOptionsProfile {
            pack_bitfield: 0x2 | 0x4 | 0x10 | 0x80 | 0x100 | 0x2000 | 0x800,
            shortcuts: vec![ShortcutInfo::DataId(0x7000_0001)],
            which_shortcut_set: 1,
            saved_ui_locations: UiSaveLocations {
                bucket_index: 0,
                layouts: vec![(
                    1,
                    0,
                    vec![(
                        5,
                        UiLocation {
                            x0: 0.5,
                            y0: 0.25,
                            x1: 1.0,
                            y1: 0.75,
                            shown: 1,
                        },
                    )],
                )],
            },
            bit_field: 0x8000_0001,
            version: Some(6),
            chat_font_colors: WordTable {
                bucket_index: 0,
                pairs: vec![(1, 0x0A)],
            },
            version6_table: WordTable {
                bucket_index: 0,
                pairs: vec![(2, 3)],
            },
            ..GameplayOptionsProfile::default()
        };
        let bytes = round_trip(NativeObject::GameplayOptionsProfile(Box::new(profile)));
        let mut expected = words(&[0x2996, 0, 1, 0x4000_0006, 0x7000_0001, 1, 1, 1, 1, 5]);
        for v in [0.5f32, 0.25, 1.0, 0.75] {
            expected.extend(v.to_le_bytes());
        }
        expected.extend(words(&[1, 0x8000_0001, 6, 1, 1, 0x0A, 1, 2, 3, 0]));
        assert_eq!(bytes, expected);
    }
}
