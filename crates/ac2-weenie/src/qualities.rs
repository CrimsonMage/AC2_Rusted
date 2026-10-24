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
}
