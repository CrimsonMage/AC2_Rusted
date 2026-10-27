//! A weenie's qualities as `CWRepository` keeps them (`m_qualities`, a `CBaseQualities`):
//! replaced whole by the player description (`CWRepository::SetQualities`) and changed one
//! quality at a time by the update events (`CWRepository::RecvEvt_Update*_Private` ->
//! `CBaseQualities::Set*`).
//!
//! `CBaseQualities::Set*` first asks `IsInWeenieDesc` whether the quality is a
//! `gmWeenieDesc` field (`gmWeenieDesc::Query*Stat`, type ids below `0x10000` only); those go
//! into the description (`gmWeenieDesc::Set*`, which also sets the field's data-mask bit),
//! everything else into the quality's hash table.

use std::collections::BTreeMap;

use ac2_net::protocol::{Position, Qualities, QualityValue, StringInfo, WeenieDesc};

/// One weenie's qualities.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WeenieQualities {
    /// The qualities DID (`DBObj::m_DID`).
    pub did: u32,
    pub weenie: Option<WeenieDesc>,
    pub ints: BTreeMap<u32, i32>,
    pub bools: BTreeMap<u32, i32>,
    pub floats: BTreeMap<u32, f32>,
    pub timestamps: BTreeMap<u32, f64>,
    pub strings: BTreeMap<u32, Vec<u8>>,
    pub dids: BTreeMap<u32, u32>,
    pub iids: BTreeMap<u32, u64>,
    pub positions: BTreeMap<u32, Position>,
    pub string_infos: BTreeMap<u32, StringInfo>,
    pub package_ids: BTreeMap<u32, u32>,
    pub long_ints: BTreeMap<u32, i64>,
}

fn table<T: Clone>(pairs: &[(u32, T)]) -> BTreeMap<u32, T> {
    pairs.iter().cloned().collect()
}

impl WeenieQualities {
    /// A created object's qualities: its weenie description (`CWRepository::SetWeenieDesc`).
    pub fn from_weenie_desc(weenie: WeenieDesc) -> Self {
        Self {
            weenie: Some(weenie),
            ..Self::default()
        }
    }

    /// `CBaseQualities::operator=` from an unpacked `CBaseQualities` (`CopyWeenieDesc`, then
    /// `CopyTables`).
    pub fn from_unpacked(q: &Qualities) -> Self {
        Self {
            did: q.did,
            weenie: q.weenie.clone(),
            ints: table(&q.ints),
            bools: table(&q.bools),
            floats: table(&q.floats),
            timestamps: table(&q.timestamps),
            strings: table(&q.strings),
            dids: table(&q.dids),
            iids: table(&q.iids),
            positions: table(&q.positions),
            string_infos: table(&q.string_infos),
            package_ids: table(&q.package_ids),
            long_ints: table(&q.long_ints),
        }
    }

    /// One quality changed in `from` (a `CWRepository::RecvEvt_Update*` for `quality` of `table`,
    /// the codes of [`QualityValue::table`]): the value `from` holds for it, written here through
    /// [`WeenieQualities::set`] so a description field lands in the description and a table
    /// quality in its table, as the update did there. Everything else (what the scripts wrote
    /// since) is left as it is; the description is not copied whole.
    pub fn copy_quality(&mut self, from: &WeenieQualities, table: u32, quality: u32) {
        let value = match table {
            1 => from.inq_bool(quality).map(QualityValue::Bool),
            2 => from.inq_data_id(quality).map(QualityValue::DataId),
            3 => from.inq_float(quality).map(QualityValue::Float),
            4 => from.inq_int(quality).map(QualityValue::Int),
            5 => from.inq_instance_id(quality).map(QualityValue::InstanceId),
            6 => from
                .positions
                .get(&quality)
                .copied()
                .map(QualityValue::Position),
            7 => from
                .strings
                .get(&quality)
                .cloned()
                .map(QualityValue::String),
            8 => from
                .timestamps
                .get(&quality)
                .copied()
                .map(QualityValue::Timestamp),
            10 => from
                .inq_string_info(quality)
                .map(|s| QualityValue::StringInfo(Box::new(s))),
            12 => from
                .long_ints
                .get(&quality)
                .copied()
                .map(QualityValue::LongInt),
            _ => None,
        };
        if let Some(value) = value {
            self.set(quality, &value);
        }
    }

    /// `CBaseQualities::SetInt`, `SetBool`, ... for one update's value.
    pub fn set(&mut self, quality: u32, value: &QualityValue) {
        if quality < 0x1_0000 {
            let desc = self.weenie.get_or_insert_with(WeenieDesc::default);
            if set_in_weenie_desc(desc, quality, value) {
                return;
            }
        }
        match value.clone() {
            QualityValue::Int(v) => _ = self.ints.insert(quality, v),
            QualityValue::Bool(v) => _ = self.bools.insert(quality, v),
            QualityValue::Float(v) => _ = self.floats.insert(quality, v),
            QualityValue::String(v) => _ = self.strings.insert(quality, v),
            QualityValue::DataId(v) => _ = self.dids.insert(quality, v),
            QualityValue::InstanceId(v) => _ = self.iids.insert(quality, v),
            QualityValue::Position(v) => _ = self.positions.insert(quality, v),
            QualityValue::Timestamp(v) => _ = self.timestamps.insert(quality, v),
            QualityValue::StringInfo(v) => _ = self.string_infos.insert(quality, *v),
            QualityValue::LongInt(v) => _ = self.long_ints.insert(quality, v),
        }
    }
}

/// `CBaseQualities::Inq*` (`InqInt`, `InqBool`, ...): the description answers first
/// (`gmWeenieDesc::Query*Stat` names its fields; `gmWeenieDesc::Inq*` answers only a field that
/// was set), then the quality's table. Retail then asks `m_defaults` (the class defaults),
/// which the client does not have.
impl WeenieQualities {
    fn desc(&self, stype: u32) -> Option<&WeenieDesc> {
        self.weenie.as_ref().filter(|_| stype < 0x1_0000)
    }

    /// `gmWeenieDesc::QueryIntStat`'s fields, as set.
    fn desc_int(w: &WeenieDesc, stype: u32) -> Option<i32> {
        match stype {
            3 => w.physics_type.0,
            4 => w.physics_type.1,
            5 => w.placement_ethereal.0,
            6 => w.placement_ethereal.1,
            7 => w.movement_ethereal.0,
            8 => w.movement_ethereal.1,
            0x106 => w.quantity,
            0x226 => w.pk_always_true,
            0x227 => w.pk_always_false,
            0x264 => w.value,
            0x26A => w.durability_max,
            0x26B => w.durability_current,
            2000 => w.faction_type,
            _ => None,
        }
    }

    pub fn inq_int(&self, stype: u32) -> Option<i32> {
        self.desc(stype)
            .and_then(|w| Self::desc_int(w, stype))
            .or_else(|| self.ints.get(&stype).copied())
    }

    /// `gmWeenieDesc::QueryBoolStat`: each flag is a `_bitfield` bit, set when its mask bit is.
    pub fn inq_bool(&self, stype: u32) -> Option<i32> {
        let (mask, bit) = match stype {
            10 => (0x0100_0000, 4),
            0x117 => (0x4000, 8),
            0x270 => (0x0080_0000, 2),
            0x2BE => (0x400, 1),
            _ => (0, 0),
        };
        self.desc(stype)
            .filter(|w| w.data_mask & mask != 0)
            .map(|w| i32::from(w.bitfield.unwrap_or(0) & bit != 0))
            .or_else(|| self.bools.get(&stype).copied())
    }

    pub fn inq_float(&self, stype: u32) -> Option<f32> {
        self.desc(stype)
            .filter(|_| stype == 0x12A)
            .and_then(|w| w.scale)
            .or_else(|| self.floats.get(&stype).copied())
    }

    pub fn inq_data_id(&self, stype: u32) -> Option<u32> {
        self.desc(stype)
            .filter(|_| stype == 0x101)
            .and_then(|w| w.icon)
            .or_else(|| self.dids.get(&stype).copied())
    }

    pub fn inq_instance_id(&self, stype: u32) -> Option<u64> {
        let field = |w: &WeenieDesc| match stype {
            0x102 => w.container,
            0x105 => w.wielder,
            0x140 => w.killer,
            0x141 => w.originator,
            0x142 => w.claimant,
            0x1C4 => w.monarch,
            0x1F9 => w.pet_summoner,
            _ => None,
        };
        self.desc(stype)
            .and_then(field)
            .or_else(|| self.iids.get(&stype).copied())
    }

    pub fn inq_string_info(&self, stype: u32) -> Option<StringInfo> {
        let field = match (self.desc(stype), stype) {
            (Some(w), 0x100) => w.name.as_ref(),
            (Some(w), 0x101) => w.plural_name.as_ref(),
            _ => None,
        };
        field.or_else(|| self.string_infos.get(&stype)).cloned()
    }
}

/// The value kinds a `CBaseQualities::Remove*` can name a description field of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescKind {
    Int,
    Bool,
    Float,
    DataId,
    InstanceId,
    StringInfo,
}

impl WeenieQualities {
    /// The first step of `CBaseQualities::Remove*`: while the weenie description holds `quality`
    /// (its data-mask bit set), `gmWeenieDesc::Remove*` clears the field and the bit and answers
    /// 1, and the quality's table is not touched. `false` leaves the table to answer.
    pub fn remove_in_weenie_desc(&mut self, quality: u32, kind: DescKind) -> bool {
        self.weenie
            .as_mut()
            .is_some_and(|w| remove_from_weenie_desc(w, quality, kind))
    }
}

/// `gmWeenieDesc::Remove*` for one field: its mask bit clears with it. A bool keeps its
/// `_bitfield` flag (the mask bit alone says it is not set).
fn remove_from_weenie_desc(w: &mut WeenieDesc, quality: u32, kind: DescKind) -> bool {
    fn clear<T>(mask: &mut u32, bit: u32, field: &mut Option<T>) -> bool {
        if *mask & bit == 0 {
            return false;
        }
        *field = None;
        *mask &= !bit;
        true
    }
    match kind {
        DescKind::Int => {
            let (bit, field) = match quality {
                3 => (0x8000, &mut w.physics_type.0),
                4 => (0x1_0000, &mut w.physics_type.1),
                5 => (0x8_0000, &mut w.placement_ethereal.0),
                6 => (0x10_0000, &mut w.placement_ethereal.1),
                7 => (0x2_0000, &mut w.movement_ethereal.0),
                8 => (0x4_0000, &mut w.movement_ethereal.1),
                0x106 => (0x800, &mut w.quantity),
                0x226 => (0x20_0000, &mut w.pk_always_true),
                0x227 => (0x40_0000, &mut w.pk_always_false),
                0x264 => (0x1000, &mut w.value),
                0x26A => (0x1000_0000, &mut w.durability_max),
                0x26B => (0x0800_0000, &mut w.durability_current),
                2000 => (0x2000, &mut w.faction_type),
                _ => return false,
            };
            clear(&mut w.data_mask, bit, field)
        }
        DescKind::Bool => {
            let mask = match quality {
                10 => 0x0100_0000,
                0x117 => 0x4000,
                0x270 => 0x0080_0000,
                0x2BE => 0x400,
                _ => return false,
            };
            if w.data_mask & mask == 0 {
                return false;
            }
            w.data_mask &= !mask;
            true
        }
        DescKind::Float => quality == 0x12A && clear(&mut w.data_mask, 0x80, &mut w.scale),
        DescKind::DataId => quality == 0x101 && clear(&mut w.data_mask, 0x10, &mut w.icon),
        DescKind::InstanceId => {
            let (bit, field) = match quality {
                0x102 => (0x20, &mut w.container),
                0x105 => (0x40, &mut w.wielder),
                0x140 => (0x4000_0000, &mut w.killer),
                0x141 => (0x200, &mut w.originator),
                0x142 => (0x2000_0000, &mut w.claimant),
                0x1C4 => (0x100, &mut w.monarch),
                0x1F9 => (0x0400_0000, &mut w.pet_summoner),
                _ => return false,
            };
            clear(&mut w.data_mask, bit, field)
        }
        DescKind::StringInfo => match quality {
            0x100 => clear(&mut w.data_mask, 4, &mut w.name),
            0x101 => clear(&mut w.data_mask, 8, &mut w.plural_name),
            _ => false,
        },
    }
}

/// `gmWeenieDesc::Query*Stat` and `gmWeenieDesc::Set*` together: whether the quality is a
/// description field, and if so the field set with its data-mask bit.
fn set_in_weenie_desc(w: &mut WeenieDesc, quality: u32, value: &QualityValue) -> bool {
    match *value {
        QualityValue::Int(v) => {
            let (bit, field) = match quality {
                3 => (0x8000, &mut w.physics_type.0),
                4 => (0x1_0000, &mut w.physics_type.1),
                5 => (0x8_0000, &mut w.placement_ethereal.0),
                6 => (0x10_0000, &mut w.placement_ethereal.1),
                7 => (0x2_0000, &mut w.movement_ethereal.0),
                8 => (0x4_0000, &mut w.movement_ethereal.1),
                0x106 => (0x800, &mut w.quantity),
                0x226 => (0x20_0000, &mut w.pk_always_true),
                0x227 => (0x40_0000, &mut w.pk_always_false),
                0x264 => (0x1000, &mut w.value),
                0x26A => (0x1000_0000, &mut w.durability_max),
                0x26B => (0x0800_0000, &mut w.durability_current),
                2000 => (0x2000, &mut w.faction_type),
                _ => return false,
            };
            *field = Some(v);
            w.data_mask |= bit;
        }
        QualityValue::Bool(v) => {
            // Each flag lives in `_bitfield`; its own mask bit says it was set, and a true
            // value also marks the bitfield present (mask bit 2).
            let (mask, flag) = match quality {
                10 => (0x0100_0000, 4),
                0x117 => (0x4000, 8),
                0x270 => (0x0080_0000, 2),
                0x2BE => (0x400, 1),
                _ => return false,
            };
            w.data_mask |= mask;
            if v != 0 {
                w.data_mask |= 2;
                w.bitfield = Some(w.bitfield.unwrap_or(0) | flag);
            } else if let Some(bits) = &mut w.bitfield {
                *bits &= !flag;
            }
        }
        QualityValue::Float(v) if quality == 0x12A => {
            w.scale = Some(v);
            w.data_mask |= 0x80;
        }
        QualityValue::DataId(v) if quality == 0x101 => {
            w.icon = Some(v);
            w.data_mask |= 0x10;
        }
        QualityValue::InstanceId(v) => {
            let (bit, field) = match quality {
                0x102 => (0x20, &mut w.container),
                0x105 => (0x40, &mut w.wielder),
                0x140 => (0x4000_0000, &mut w.killer),
                0x141 => (0x200, &mut w.originator),
                0x142 => (0x2000_0000, &mut w.claimant),
                0x1C4 => (0x100, &mut w.monarch),
                0x1F9 => (0x0400_0000, &mut w.pet_summoner),
                _ => return false,
            };
            *field = Some(v);
            w.data_mask |= bit;
        }
        QualityValue::StringInfo(ref v) if quality == 0x100 || quality == 0x101 => {
            if quality == 0x100 {
                w.name = Some((**v).clone());
                w.data_mask |= 4;
            } else {
                w.plural_name = Some((**v).clone());
                w.data_mask |= 8;
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::WeenieQualities;
    use ac2_net::protocol::QualityValue;

    #[test]
    fn inq_reads_the_description_before_the_tables() {
        let mut q = WeenieQualities::default();
        q.set(0x106, &QualityValue::Int(5));
        q.set(0x2BE, &QualityValue::Bool(1));
        q.set(0x101, &QualityValue::DataId(0x0600_1001));
        q.set(261, &QualityValue::Int(10));
        assert_eq!(q.inq_int(0x106), Some(5));
        assert_eq!(q.inq_int(261), Some(10));
        assert_eq!(q.inq_int(0x264), None);
        // A description field it does not carry falls through to the table.
        q.ints.insert(0x264, 7);
        assert_eq!(q.inq_int(0x264), Some(7));
        assert_eq!(q.inq_bool(0x2BE), Some(1));
        assert_eq!(q.inq_bool(10), None);
        assert_eq!(q.inq_data_id(0x101), Some(0x0600_1001));
        assert!(!q.ints.contains_key(&0x106));
    }

    #[test]
    fn description_fields_go_into_the_weenie_desc_with_their_mask_bits() {
        let mut q = WeenieQualities::default();
        q.set(0x106, &QualityValue::Int(5));
        q.set(0x12A, &QualityValue::Float(1.5));
        q.set(0x117, &QualityValue::Bool(1));
        let w = q.weenie.as_ref().unwrap();
        assert_eq!(
            (w.quantity, w.scale, w.bitfield),
            (Some(5), Some(1.5), Some(8))
        );
        assert_eq!(w.data_mask, 0x800 | 0x80 | 0x4000 | 2);
        q.set(0x117, &QualityValue::Bool(0));
        assert_eq!(q.weenie.as_ref().unwrap().bitfield, Some(0));
        assert!(q.ints.is_empty() && q.floats.is_empty() && q.bools.is_empty());
    }

    #[test]
    fn remove_clears_a_description_field_and_its_mask_bit() {
        use super::DescKind;
        let mut q = WeenieQualities::default();
        q.set(0x106, &QualityValue::Int(5));
        q.set(0x100, &QualityValue::StringInfo(Box::default()));
        q.set(0x2BE, &QualityValue::Bool(1));
        // `gmWeenieDesc::RemoveInt` answers 1 and clears the field and its bit.
        assert!(q.remove_in_weenie_desc(0x106, DescKind::Int));
        let w = q.weenie.as_ref().unwrap();
        assert_eq!((w.quantity, w.data_mask & 0x800), (None, 0));
        assert_eq!(q.inq_int(0x106), None);
        // A second remove finds nothing in the description.
        assert!(!q.remove_in_weenie_desc(0x106, DescKind::Int));
        // A bool keeps its flag; its mask bit alone says it is not set.
        assert!(q.remove_in_weenie_desc(0x2BE, DescKind::Bool));
        assert_eq!(q.inq_bool(0x2BE), None);
        assert_eq!(q.weenie.as_ref().unwrap().bitfield.unwrap() & 1, 1);
        assert!(q.remove_in_weenie_desc(0x100, DescKind::StringInfo));
        assert!(q.weenie.as_ref().unwrap().name.is_none());
        // Other kinds and ids never match the description.
        assert!(!q.remove_in_weenie_desc(0x106, DescKind::Float));
        assert!(!q.remove_in_weenie_desc(0x19, DescKind::Int));
    }

    #[test]
    fn other_qualities_go_into_their_tables() {
        let mut q = WeenieQualities::default();
        q.set(0x19, &QualityValue::Int(12));
        q.set(0x19, &QualityValue::Int(13));
        q.set(0x101, &QualityValue::Int(1)); // a DID-field id, but an int: the table
        q.set(0x1_0106, &QualityValue::Int(2)); // past 0x10000: never the description
        q.set(4, &QualityValue::LongInt(-1));
        assert_eq!(q.ints.get(&0x19), Some(&13));
        assert_eq!(q.ints.get(&0x101), Some(&1));
        assert_eq!(q.ints.get(&0x1_0106), Some(&2));
        assert_eq!(q.long_ints.get(&4), Some(&-1));
        assert_eq!(q.weenie.as_ref().unwrap().data_mask, 0);
    }

    /// `CWRepository::RecvEvt_UpdateInt_Private` on the scripts' copy: the server's update for
    /// one quality lands there, and what the scripts wrote since (a description field the
    /// VM adjusted) stays.
    #[test]
    fn copy_quality_takes_one_quality_and_keeps_what_the_scripts_wrote() {
        let mut scripts = WeenieQualities::default();
        scripts.set(0x106, &QualityValue::Int(3)); // a script's quantity
        let mut server = WeenieQualities::default();
        server.set(0x102, &QualityValue::Int(7)); // health, from the server
        server.set(0x26B, &QualityValue::Int(9)); // durability, from the server
        scripts.copy_quality(&server, 4, 0x102);
        assert_eq!(scripts.inq_int(0x102), Some(7));
        assert_eq!(scripts.inq_int(0x106), Some(3));
        assert_eq!(scripts.inq_int(0x26B), None);
        // A description field the server set goes into the description on this side too.
        scripts.copy_quality(&server, 4, 0x26B);
        assert_eq!(scripts.inq_int(0x26B), Some(9));
        assert_eq!(scripts.weenie.as_ref().unwrap().quantity, Some(3));
        assert!(!scripts.ints.contains_key(&0x26B));
    }
}
