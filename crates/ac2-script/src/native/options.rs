//! `GameplayOptionsProfile` (native 0x9B) as the Legions client lays it out: the player's
//! options, which the server sends and the player's script reads and edits.
//!
//! Legions differs from admin 7148: a table at +0xA0 (pack bit 0x2000, version 6; chat window
//! -> text type, default 0x10; efuns 0x9B0053/0x9B0054) moved the chat font colours to +0xBC,
//! the popup flags to +0xD8 and the window opacities to +0xF4, and `StreamPack` upgrades to
//! version 6.
//!
//! Every C++ hash here is an `IntrusiveHashTable` (`IntrusiveHashTable(0x20)`: 47 buckets, new
//! elements at their bucket's head), kept in [`ScriptHash`] so iteration order, and with it
//! what packs and what the scripts see, follows retail.

use ac2_net::protocol::{self as net, ShortcutInfo, UiSaveLocations, WordTable};

use super::containers::RefKey;
use super::hash_table::{bucket_size_index, PlainKeys, ScriptHash, BUCKET_SIZES, DEFAULT_BUCKETS};
use crate::glue::compute_hash;

/// `PHashTable<PStringBase<unsigned short>, PStringBase<unsigned short>>`: the aliases, by
/// `PStringBase::compute_hash % buckets`, each bucket head first.
#[derive(Debug, Clone, PartialEq)]
pub struct AliasTable {
    pub buckets: Vec<Vec<(Vec<u16>, Vec<u16>)>>,
}

impl AliasTable {
    pub fn with_buckets(n: u32) -> Self {
        Self {
            buckets: vec![Vec::new(); n.max(1) as usize],
        }
    }

    fn bucket(&self, key: &[u16]) -> usize {
        (compute_hash(key) % self.buckets.len() as u32) as usize
    }

    /// `HashTable::find`.
    pub fn find(&self, key: &[u16]) -> Option<&[u16]> {
        self.buckets[self.bucket(key)]
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_slice())
    }

    /// `HashTable::add`: false when the key is present.
    pub fn add(&mut self, key: Vec<u16>, value: Vec<u16>) -> bool {
        if self.find(&key).is_some() {
            return false;
        }
        let b = self.bucket(&key);
        self.buckets[b].insert(0, (key, value));
        true
    }

    /// `HashTable::set`: replaces the value or adds.
    pub fn set(&mut self, key: Vec<u16>, value: Vec<u16>) {
        let b = self.bucket(&key);
        match self.buckets[b].iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1 = value,
            None => self.buckets[b].insert(0, (key, value)),
        }
    }

    /// `HashTable::erase`.
    pub fn erase(&mut self, key: &[u16]) -> bool {
        let b = self.bucket(key);
        let before = self.buckets[b].len();
        self.buckets[b].retain(|(k, _)| k != key);
        before != self.buckets[b].len()
    }

    /// `delete_contents`.
    pub fn flush(&mut self) {
        self.buckets.iter_mut().for_each(Vec::clear);
    }

    /// Iteration order: buckets ascending, each chain head first.
    pub fn iter(&self) -> impl Iterator<Item = (&[u16], &[u16])> {
        self.buckets
            .iter()
            .flatten()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.iter().all(Vec::is_empty)
    }

    /// `HashTable::operator=`: flushed, then `rhs`'s elements added in its order.
    pub fn assign(&mut self, rhs: &AliasTable) {
        self.flush();
        let pairs: Vec<_> = rhs.iter().map(|(k, v)| (k.to_vec(), v.to_vec())).collect();
        for (k, v) in pairs {
            self.add(k, v);
        }
    }

    /// `PHashTable::UnPack` (`add_for_unpacking`, see [`ScriptHash::from_packed`]).
    fn from_packed(index: u8, pairs: &[(String, String)]) -> Self {
        let size = BUCKET_SIZES[usize::from(index).min(BUCKET_SIZES.len() - 1)];
        let mut table = Self::with_buckets(size);
        let mut prev = None;
        for (k, v) in pairs {
            let (k, v): (Vec<u16>, Vec<u16>) =
                (k.encode_utf16().collect(), v.encode_utf16().collect());
            let b = table.bucket(&k);
            if prev == Some(b) {
                table.buckets[b].push((k, v));
            } else {
                table.buckets[b] = vec![(k, v)];
            }
            prev = Some(b);
        }
        table
    }
}

/// A `HashTable<unsigned long, T>` of words (floats as their bits).
pub fn word_table() -> ScriptHash {
    ScriptHash::with_buckets(DEFAULT_BUCKETS)
}

/// `HashTable::find` of a word table.
pub fn word_get(table: &ScriptHash, key: u32) -> Option<u32> {
    table
        .find(&RefKey::Value(u64::from(key)), &PlainKeys)
        .map(|v| v as u32)
}

/// `HashTable<unsigned long, T>::set`.
pub fn word_set(table: &mut ScriptHash, key: u32, value: u32) {
    table.set(RefKey::Value(u64::from(key)), u64::from(value), &PlainKeys);
}

/// `HashTable::operator=` (and the glue's container copies): `dst` flushed, then `src`'s
/// elements added in `src`'s order at their bucket heads.
pub fn word_assign(dst: &mut ScriptHash, src: &ScriptHash) {
    dst.flush();
    let pairs: Vec<_> = src.nodes().map(|n| (n.key, n.value)).collect();
    for (k, v) in pairs {
        dst.add(k, v, &PlainKeys);
    }
}

fn word_from_net(t: &WordTable) -> ScriptHash {
    let items = t
        .pairs
        .iter()
        .map(|&(k, v)| (RefKey::Value(u64::from(k)), u64::from(v)))
        .collect();
    ScriptHash::from_packed(t.bucket_index, items)
}

fn word_to_net(t: &ScriptHash) -> WordTable {
    WordTable {
        bucket_index: bucket_size_index(t.num_buckets()) as u8,
        pairs: t
            .nodes()
            .map(|n| match n.key {
                RefKey::Value(k) => (k as u32, n.value as u32),
                RefKey::Ref { id, .. } => (id, n.value as u32),
            })
            .collect(),
    }
}

/// Bits of `m_bitField` (`SetBitMask` / `TestBitMask` masks of the glue).
pub mod bits {
    pub const SHOW_SHORTCUT_NUMBERS: u32 = 0x1;
    pub const SHOW_DAMAGE_TEXT: u32 = 0x4;
    pub const SHOW_DAMAGE_TEXT_OTHER: u32 = 0x8;
    pub const SHOW_RADAR_COORDS: u32 = 0x10;
    pub const AUTO_TARGET: u32 = 0x20;
    pub const ACCEPT_ALLEGIANCE_REQUEST: u32 = 0x40;
    pub const ACCEPT_FELLOWSHIP_REQUEST: u32 = 0x80;
    pub const ACCEPT_TRADE_REQUEST: u32 = 0x100;
    pub const SHOW_OBJECT_NAMES: u32 = 0x200;
    pub const BYPASS_GOLD_MELTING_CONFIRMATION: u32 = 0x400;
    pub const SHORTCUT_LOCKED: u32 = 0x800;
    pub const SHOW_PK_DAMAGE: u32 = 0x1000;
    pub const SHORTCUT_INSTA_SCROLL: u32 = 0x2000;
    pub const LATENCY_INDICATOR_GRAPH: u32 = 0x4000;
    /// Set means hidden (`Get/SetShowFellowsOnMap` invert it).
    pub const HIDE_FELLOWS_ON_MAP: u32 = 0x8000;
    /// Set means hidden (`Get/SetShowPetUI` invert it).
    pub const HIDE_PET_UI: u32 = 0x10000;
    /// The profile carries `m_version` and the versioned tables.
    pub const VERSIONED: u32 = 0x8000_0000;
}

/// The text type `StreamPack`'s upgrades give a colour and a popup flag (Legions 0x009E8564).
const BROADCAST_TEXT_TYPE: u32 = 0x100;
/// The version Legions `StreamPack` upgrades to.
pub const VERSION: u32 = 6;

/// `GameplayOptionsProfile`, Legions layout.
#[derive(Debug, Clone, PartialEq)]
pub struct GameplayOptions {
    /// +0x04 `m_aliasTable` (pack bit 0x1).
    pub aliases: AliasTable,
    /// +0x20 `m_shortcutArray` (0x2).
    pub shortcuts: Vec<ShortcutInfo>,
    /// +0x30 `m_whichShortcutSet` (0x4).
    pub which_shortcut_set: u32,
    /// +0x34 `m_iShortcutHeight` (0x1000).
    pub shortcut_height: u32,
    /// +0x38 `m_fDamageTextRangeOther` (0x8).
    pub damage_text_range_other: f32,
    /// +0x3C `m_savedUILocations` (0x10).
    pub ui_locations: UiSaveLocations,
    /// +0x5C `m_radarMask` (0x20).
    pub radar_mask: u32,
    /// +0x60 `m_filterHash`: chat window -> text type mask (0x40).
    pub filters: ScriptHash,
    /// +0x7C `m_bitField` (0x80).
    pub bit_field: u32,
    /// +0x80 `m_version`.
    pub version: u32,
    /// +0x84 chat font sizes by text type (0x200).
    pub font_sizes: ScriptHash,
    /// +0xA0 (Legions): chat window -> text type, default 0x10 (0x2000, version 6).
    pub window_text_types: ScriptHash,
    /// +0xBC chat font colours by text type (0x100).
    pub font_colors: ScriptHash,
    /// +0xD8 chat popup flags by text type (0x400).
    pub popup_flags: ScriptHash,
    /// +0xF4 window opacities by window id, float bits (0x800).
    pub opacities: ScriptHash,
}

impl Default for GameplayOptions {
    /// `GameplayOptionsProfile::GameplayOptionsProfile`: empty tables, version 0, `Clear`.
    fn default() -> Self {
        let mut p = Self {
            aliases: AliasTable::with_buckets(DEFAULT_BUCKETS),
            shortcuts: Vec::new(),
            which_shortcut_set: 0,
            shortcut_height: 1,
            damage_text_range_other: -1.0,
            ui_locations: UiSaveLocations {
                bucket_index: bucket_size_index(DEFAULT_BUCKETS) as u8,
                layouts: Vec::new(),
            },
            radar_mask: 0,
            filters: word_table(),
            bit_field: 0,
            version: 0,
            font_sizes: word_table(),
            window_text_types: word_table(),
            font_colors: word_table(),
            popup_flags: word_table(),
            opacities: word_table(),
        };
        p.clear();
        p
    }
}

/// The text types `FlushChatFontOptions` fills, with each one's colour (Legions 0x005C04F0).
const FONT_COLORS: [(u32, u32); 17] = [
    (0x1, 0),
    (0x2, 1),
    (0x4, 2),
    (0x8, 3),
    (0x10, 4),
    (0x20, 5),
    (0x40, 6),
    (0x80, 4),
    (0x10000, 7),
    (0x20000, 8),
    (0x400000, 4),
    (0x800000, 4),
    (0x40000, 4),
    (0x80000, 4),
    (0x100000, 4),
    (0x200000, 4),
    (0x100, 9),
];

impl GameplayOptions {
    /// `GameplayOptionsProfile::Clear` (Legions 0x005C0A90; `m_version` is kept).
    pub fn clear(&mut self) {
        self.which_shortcut_set = 0;
        self.shortcut_height = 1;
        self.damage_text_range_other = -1.0;
        self.radar_mask = 0;
        self.bit_field = 0;
        self.aliases.flush();
        self.shortcuts.clear();
        self.ui_locations.layouts.clear();
        // Legions 0x005C0310: the chat windows' default filters.
        self.filters.flush();
        let defaults = [
            (1, 0x0060_017F),
            (2, 0x8_0000),
            (3, 0x1_0000),
            (4, 0x2_0000),
        ];
        for (window, mask) in defaults {
            let old = word_get(&self.filters, window).unwrap_or(0);
            word_set(&mut self.filters, window, old | mask);
        }
        self.flush_chat_font_options();
        // Legions 0x005C0800: the default popup flags.
        self.popup_flags.flush();
        for text_type in [0x100, 0x2_0000, 0x1_0000] {
            word_set(&mut self.popup_flags, text_type, 1);
        }
        self.opacities.flush();
    }

    /// `FlushChatFontOptions` (Legions 0x005C04F0): default colours and sizes per text type
    /// and each chat window's text type.
    pub fn flush_chat_font_options(&mut self) {
        self.font_sizes.flush();
        self.font_colors.flush();
        self.window_text_types.flush();
        for (text_type, color) in FONT_COLORS {
            word_set(&mut self.font_colors, text_type, color);
        }
        for (text_type, _) in FONT_COLORS {
            word_set(&mut self.font_sizes, text_type, 0);
        }
        for (window, text_type) in [(1, 0x10), (2, 0x8_0000), (3, 0x1_0000), (4, 0x2_0000)] {
            word_set(&mut self.window_text_types, window, text_type);
        }
    }

    pub fn test_bit_mask(&self, mask: u32) -> bool {
        self.bit_field & mask != 0
    }

    /// `SetBitMask`: clearing the version bit is refused.
    pub fn set_bit_mask(&mut self, on: bool, mask: u32) -> bool {
        if on {
            self.bit_field |= mask;
        } else if mask != bits::VERSIONED {
            self.bit_field &= !mask;
        } else {
            return false;
        }
        true
    }

    /// `GetFilter`: 0 for a window without one.
    pub fn filter(&self, window: u32) -> u32 {
        word_get(&self.filters, window).unwrap_or(0)
    }

    /// `AddFilter` / `RemoveFilter` / `SetFilter`.
    pub fn set_filter(&mut self, window: u32, mask: u32) {
        word_set(&mut self.filters, window, mask);
    }

    /// `GetPackBitfield` (Legions 0x005BF8B0): each field that differs from the defaults.
    pub fn pack_bitfield(&self) -> u64 {
        let d = (0u32, 1u32, -1.0f32, 0u32, 0u32);
        let fields = [
            (0x1, !self.aliases.is_empty()),
            (0x2, !self.shortcuts.is_empty()),
            (0x4, self.which_shortcut_set != d.0),
            (0x1000, self.shortcut_height != d.1),
            (0x8, self.damage_text_range_other != d.2),
            (0x10, !self.ui_locations.layouts.is_empty()),
            (0x20, self.radar_mask != d.3),
            (0x40, self.filters.num_elements() != 0),
            (0x80, self.bit_field != d.4),
            (0x100, self.font_colors.num_elements() != 0),
            (0x200, self.font_sizes.num_elements() != 0),
            (0x400, self.popup_flags.num_elements() != 0),
            (0x800, self.opacities.num_elements() != 0),
            (0x2000, self.window_text_types.num_elements() != 0),
        ];
        fields
            .iter()
            .filter(|(_, on)| *on)
            .fold(0, |bits, (bit, _)| bits | bit)
    }

    /// The upgrades `StreamPack` applies to an unversioned profile, and to versions before 4.
    fn upgrade(&mut self, from: Option<u32>) {
        let filter_or = |p: &mut Self, window: u32, mask: u32| {
            let old = p.filter(window);
            p.set_filter(window, old | mask);
        };
        match from {
            None => {
                // An unversioned profile (Legions 0x005C106F).
                self.version = VERSION;
                self.bit_field |= bits::VERSIONED;
                filter_or(self, 2, 0x8_0000);
                word_set(&mut self.font_colors, BROADCAST_TEXT_TYPE, 9);
                filter_or(self, 1, 0x100);
                word_set(&mut self.popup_flags, BROADCAST_TEXT_TYPE, 1);
            }
            Some(v) => {
                if v < 2 {
                    filter_or(self, 1, 0x8_0000);
                }
                if v < 3 {
                    word_set(&mut self.font_colors, BROADCAST_TEXT_TYPE, 9);
                }
                if v < 4 {
                    filter_or(self, 1, 0x100);
                    word_set(&mut self.popup_flags, BROADCAST_TEXT_TYPE, 1);
                }
                self.version = VERSION;
            }
        }
    }

    /// `StreamPack` unpacking (Legions 0x005C0EB0): `Clear`, the packed fields, then the
    /// version upgrades.
    pub fn from_net(w: &net::GameplayOptionsProfile) -> Self {
        let mut p = Self::default();
        let has = |bit: u64| w.pack_bitfield & bit != 0;
        if has(0x1) {
            p.aliases = AliasTable::from_packed(w.aliases.0, &w.aliases.1);
        }
        if has(0x2) {
            p.shortcuts = w.shortcuts.clone();
        }
        if has(0x4) {
            p.which_shortcut_set = w.which_shortcut_set;
        }
        if has(0x1000) {
            p.shortcut_height = w.shortcut_height;
        }
        if has(0x8) {
            p.damage_text_range_other = w.damage_text_range_other;
        }
        if has(0x10) {
            p.ui_locations = w.saved_ui_locations.clone();
        }
        if has(0x20) {
            p.radar_mask = w.radar_mask;
        }
        if has(0x40) {
            p.filters = word_from_net(&w.filters);
        }
        if has(0x80) {
            p.bit_field = w.bit_field;
        }
        if p.bit_field & bits::VERSIONED == 0 {
            p.upgrade(None);
            return p;
        }
        let version = w.version.unwrap_or(0);
        let tables = [
            (0x100, 1, &w.chat_font_colors, &mut p.font_colors),
            (0x200, 1, &w.chat_font_sizes, &mut p.font_sizes),
            (0x2000, 6, &w.version6_table, &mut p.window_text_types),
            (0x400, 2, &w.chat_popup_flags, &mut p.popup_flags),
            (0x800, 2, &w.window_opacities, &mut p.opacities),
        ];
        for (bit, min, packed, table) in tables {
            if has(bit) && version >= min {
                *table = word_from_net(packed);
            }
        }
        p.upgrade(Some(version));
        p
    }

    /// `StreamPack` packing: the fields `GetPackBitfield` names. An unversioned profile is
    /// upgraded first (retail upgrades it while packing).
    pub fn to_net(&self) -> net::GameplayOptionsProfile {
        let mut p = self.clone();
        if p.bit_field & bits::VERSIONED == 0 {
            p.upgrade(None);
        }
        net::GameplayOptionsProfile {
            pack_bitfield: p.pack_bitfield(),
            aliases: (
                bucket_size_index(p.aliases.buckets.len() as u32) as u8,
                p.aliases
                    .iter()
                    .map(|(k, v)| (String::from_utf16_lossy(k), String::from_utf16_lossy(v)))
                    .collect(),
            ),
            shortcuts: p.shortcuts.clone(),
            which_shortcut_set: p.which_shortcut_set,
            shortcut_height: p.shortcut_height,
            damage_text_range_other: p.damage_text_range_other,
            saved_ui_locations: p.ui_locations.clone(),
            radar_mask: p.radar_mask,
            filters: word_to_net(&p.filters),
            bit_field: p.bit_field,
            version: Some(p.version),
            chat_font_colors: word_to_net(&p.font_colors),
            chat_font_sizes: word_to_net(&p.font_sizes),
            version6_table: word_to_net(&p.window_text_types),
            chat_popup_flags: word_to_net(&p.popup_flags),
            window_opacities: word_to_net(&p.opacities),
        }
    }
}

/// `UISaveLocations::operator=` (`HashTable::operator=` on the layouts): `src`'s layouts
/// re-added in order at their bucket heads in `dst`'s bucket count (each layout's own windows
/// copied as they are).
pub fn assign_ui_locations(dst: &mut UiSaveLocations, src: &UiSaveLocations) {
    let n = BUCKET_SIZES[usize::from(dst.bucket_index).min(BUCKET_SIZES.len() - 1)];
    let mut buckets: Vec<Vec<(u32, u8, Vec<(u32, net::UiLocation)>)>> =
        vec![Vec::new(); n as usize];
    for layout in &src.layouts {
        let b = (layout.0 % n) as usize;
        if buckets[b].iter().any(|l| l.0 == layout.0) {
            continue;
        }
        buckets[b].insert(0, layout.clone());
    }
    dst.layouts = buckets.into_iter().flatten().collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh profile (`GameplayOptionsProfile()` = `Clear`) holds the Legions chat defaults
    /// and packs only what differs from them.
    #[test]
    fn a_fresh_profile_holds_the_legions_defaults() {
        let p = GameplayOptions::default();
        assert_eq!(p.filter(1), 0x0060_017F);
        assert_eq!(p.filter(2), 0x8_0000);
        assert_eq!(word_get(&p.font_colors, 0x100), Some(9));
        assert_eq!(word_get(&p.window_text_types, 1), Some(0x10));
        assert_eq!(word_get(&p.popup_flags, 0x2_0000), Some(1));
        assert_eq!(p.version, 0);
        assert_eq!(p.pack_bitfield(), 0x40 | 0x100 | 0x200 | 0x400 | 0x2000);
    }

    /// Unpacking an unversioned profile upgrades it to version 6 (filter 2 gains 0x80000 and
    /// filter 1 0x100); a versioned one before 4 gets the broadcast colour and popup flag.
    #[test]
    fn unpacking_upgrades_to_version_6() {
        let mut wire = net::GameplayOptionsProfile {
            pack_bitfield: 0x80 | 0x40,
            bit_field: 0x20,
            ..Default::default()
        };
        wire.filters = WordTable {
            bucket_index: 2,
            pairs: vec![(1, 0x1), (2, 0x2)],
        };
        let p = GameplayOptions::from_net(&wire);
        assert_eq!(p.version, VERSION);
        assert_eq!(p.bit_field, 0x8000_0020);
        assert_eq!((p.filter(1), p.filter(2)), (0x101, 0x8_0002));
        wire.bit_field = 0x8000_0000;
        wire.version = Some(3);
        let p = GameplayOptions::from_net(&wire);
        assert_eq!((p.filter(1), p.filter(2)), (0x101, 0x2));
        assert_eq!(word_get(&p.popup_flags, 0x100), Some(1));
        let back = p.to_net();
        assert_eq!(back.version, Some(VERSION));
        assert_eq!(GameplayOptions::from_net(&back).to_net(), back);
    }

    /// Aliases hash by `PStringBase::compute_hash`; copies re-add in order, so a shared bucket
    /// reverses.
    #[test]
    fn alias_copies_reverse_shared_buckets() {
        let w = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        let mut a = AliasTable::with_buckets(1);
        a.set(w("x"), w("1"));
        a.set(w("y"), w("2"));
        let order: Vec<_> = a.iter().map(|(k, _)| String::from_utf16_lossy(k)).collect();
        assert_eq!(order, ["y", "x"]);
        let mut b = AliasTable::with_buckets(1);
        b.assign(&a);
        let order: Vec<_> = b.iter().map(|(k, _)| String::from_utf16_lossy(k)).collect();
        assert_eq!(order, ["x", "y"]);
        assert_eq!(b.find(&w("y")), Some(&w("2")[..]));
        assert!(b.erase(&w("y")) && !b.erase(&w("y")));
    }
}
