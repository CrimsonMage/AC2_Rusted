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
use std::sync::Arc;

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
    /// `CBaseQualities::m_defaults`: the class defaults, the DAT qualities at [`Self::did`]
    /// (`CBaseQualities::InitDefaults`), which every `Inq*` asks after the tables. Set once
    /// the client has read them ([`WeenieQualities::set_defaults`]).
    pub defaults: Option<Arc<WeenieQualities>>,
}

/// The bool quality `CBaseQualities::IsMobile` asks for.
const MOBILE_QUALITY: u32 = 8;

/// The int quality `CWInterface` reads the weenie type from.
const WEENIE_TYPE_QUALITY: u32 = 0x14;

/// `CWInterface::IsPlayer`'s weenie type bits.
const PLAYER_TYPE: i32 = 0x1F;

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

    /// `CBaseQualities::SetWeenieDesc`: the description is replaced by a copy of `weenie`.
    pub fn set_weenie_desc(&mut self, weenie: WeenieDesc) {
        self.weenie = Some(weenie);
    }

    /// `CBaseQualities::IsMobile`: the bool quality 8, mobile unless it reads 0 (an absent
    /// quality leaves the default 1).
    pub fn is_mobile(&self) -> bool {
        self.inq_bool(MOBILE_QUALITY).unwrap_or(1) != 0
    }

    /// `CWRepository::GatherPhysicsState` (Legions 0x00515E70): the physics state bits from the
    /// engine bools: 0x4 unless ReportCollisions (2) reads false, 0x8 unless IgnoreCollisions (1)
    /// reads false, 0x400 when LightsStatus (5) reads true, 0x200 unless GravityStatus (4) reads
    /// false, 0x2000 when Inelastic (6) reads true, 0x8000 when AllowEdgeSlide (7) reads true.
    pub fn gather_physics_state(&self) -> u32 {
        let read = |stype: u32| self.inq_bool(stype);
        let mut state = 0;
        if read(2).is_none_or(|v| v != 0) {
            state |= 0x4;
        }
        if read(1).is_none_or(|v| v != 0) {
            state |= 0x8;
        }
        if read(5).is_some_and(|v| v != 0) {
            state |= 0x400;
        }
        if read(4).is_none_or(|v| v != 0) {
            state |= 0x200;
        }
        if read(6).is_some_and(|v| v != 0) {
            state |= 0x2000;
        }
        if read(7).is_some_and(|v| v != 0) {
            state |= 0x8000;
        }
        state
    }

    /// `CWRepository::BuildPhysObjProfile` (Legions 0x00516230): the state, the placement (int
    /// quality 1, 0 without it) and the scale (float 0x12A, 1 without it). (Its friction,
    /// elasticity and mass are not read by `CPhysicsObj::ApplyPhysObjProfile`.)
    pub fn phys_obj_profile(&self) -> (u32, u32, f32) {
        (
            self.gather_physics_state(),
            self.inq_int(1).map_or(0, |v| v as u32),
            self.inq_float(0x12A).unwrap_or(1.0),
        )
    }

    /// `CWInterface::IsPlayer`: the weenie type (int quality 0x14) has all of 0x1F.
    pub fn is_player(&self) -> bool {
        self.inq_int(WEENIE_TYPE_QUALITY)
            .is_some_and(|t| t & PLAYER_TYPE == PLAYER_TYPE)
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
            defaults: None,
        }
    }

    /// `CWRepository::Construct`'s defaults DID for a created object: the qualities record of its
    /// package (`GetBaseDID(0x40000019) | pid & 0xFFFF`) when it has no entity DID, else of its
    /// entity description (`+ 0x1000000 | did & 0xFFFFFF`, only for a `0x40000049` entity DID).
    /// 0 for none.
    pub fn defaults_did(package: Option<u32>, entity: Option<u32>) -> u32 {
        const QUALITIES_BASE: u32 = 0x8000_0000;
        match entity.unwrap_or(0) {
            0 => package.map_or(0, |pid| QUALITIES_BASE | (pid & 0xFFFF)),
            did if (0x4700_0000..=0x47FF_FFFE).contains(&did) => {
                (QUALITIES_BASE + 0x100_0000) | (did & 0xFF_FFFF)
            }
            _ => 0,
        }
    }

    /// `CBaseQualities::InitDefaults`: the class defaults the `Inq*` fall back on.
    pub fn set_defaults(&mut self, defaults: Arc<WeenieQualities>) {
        self.defaults = Some(defaults);
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
/// (`gmWeenieDesc::Query*Stat` names its fields; `gmWeenieDesc::InqInt`, `InqBool`, `InqFloat`,
/// `InqDataID`, `InqInstanceID` and `InqStringInfo` answer only a field that was set), then the
/// quality's table, then `m_defaults` (the class defaults, the DAT qualities of the weenie's DID,
/// [`WeenieQualities::defaults`]; the script glue also asks its data source's `class_defaults`
/// for an object whose defaults are not set here yet).
/// `CBaseQualities::TruncateInt` (Legions 0x00507AE0): a change that wrapped (a decrease
/// by a positive `modifier` or an increase by a negative one, or no change at all) is pulled
/// to the bound it wrapped toward, then the result is clamped to `[minimum, maximum]`.
pub fn truncate_int(modifier: i32, old: i32, minimum: i32, maximum: i32, new: i32) -> i32 {
    let mut value = new;
    if modifier < 0 {
        if old < value {
            value = minimum;
        }
        if value > maximum {
            value = maximum;
        }
    } else if old <= value {
        if value > maximum {
            value = maximum;
        }
    } else {
        value = maximum;
    }
    value.max(minimum)
}

/// `CBaseQualities::TruncateLongInt` (Legions 0x00507B20): [`truncate_int`] over 64 bits.
pub fn truncate_long_int(modifier: i64, old: i64, minimum: i64, maximum: i64, new: i64) -> i64 {
    let mut value = new;
    if modifier < 0 {
        if old < value {
            value = minimum;
        }
        if value > maximum {
            value = maximum;
        }
    } else if old <= value {
        if value > maximum {
            value = maximum;
        }
    } else {
        value = maximum;
    }
    value.max(minimum)
}

/// `CBaseQualities::TruncateFloat` (Legions 0x00507BB0). Unlike the integer forms, a positive
/// modifier that lowers the value sets the maximum, and no equal-value branch exists.
pub fn truncate_float(modifier: f32, old: f32, minimum: f32, maximum: f32, new: f32) -> f32 {
    let mut value = new;
    if modifier < 0.0 {
        if old < value {
            value = minimum;
        }
    } else if value < old {
        value = maximum;
    }
    if value > maximum {
        value = maximum;
    }
    if value < minimum {
        value = minimum;
    }
    value
}

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
            .or_else(|| self.defaults.as_ref().and_then(|d| d.inq_int(stype)))
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
            .or_else(|| self.defaults.as_ref().and_then(|d| d.inq_bool(stype)))
    }

    /// `gmCWInterface::HandleSetWeenieDesc` (Legions 0x004531B0): what the description makes of
    /// the physics object's no-draw: `NoDraw` (bool 10) set turns it on; otherwise it turns off,
    /// unless the object is `Dead` (bool 0x117), when it is left as it is (`None`).
    pub fn set_weenie_desc_no_draw(&self) -> Option<bool> {
        if self.inq_bool(10).unwrap_or(0) != 0 {
            Some(true)
        } else if self.inq_bool(0x117).unwrap_or(0) == 0 {
            Some(false)
        } else {
            None
        }
    }

    pub fn inq_float(&self, stype: u32) -> Option<f32> {
        self.desc(stype)
            .filter(|_| stype == 0x12A)
            .and_then(|w| w.scale)
            .or_else(|| self.floats.get(&stype).copied())
            .or_else(|| self.defaults.as_ref().and_then(|d| d.inq_float(stype)))
    }

    pub fn inq_data_id(&self, stype: u32) -> Option<u32> {
        self.desc(stype)
            .filter(|_| stype == 0x101)
            .and_then(|w| w.icon)
            .or_else(|| self.dids.get(&stype).copied())
            .or_else(|| self.defaults.as_ref().and_then(|d| d.inq_data_id(stype)))
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
            .or_else(|| {
                self.defaults
                    .as_ref()
                    .and_then(|d| d.inq_instance_id(stype))
            })
    }

    pub fn inq_string_info(&self, stype: u32) -> Option<StringInfo> {
        let field = match (self.desc(stype), stype) {
            (Some(w), 0x100) => w.name.as_ref(),
            (Some(w), 0x101) => w.plural_name.as_ref(),
            _ => None,
        };
        field
            .or_else(|| self.string_infos.get(&stype))
            .cloned()
            .or_else(|| {
                self.defaults
                    .as_ref()
                    .and_then(|d| d.inq_string_info(stype))
            })
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

/// `gmWeenieDesc::Remove*` for one field (the two-argument `RemoveBool`, `RemoveFloat`,
/// `RemoveDataID`, `RemoveInstanceID` and `RemoveStringInfo`, like `RemoveInt`): its mask bit
/// clears with it. A bool keeps its
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

    /// `gmCWInterface::HandleSetWeenieDesc`: NoDraw (bool 10, bitfield 4) hides the physics
    /// object; otherwise it shows, unless the object is Dead (bool 0x117, bitfield 8), which
    /// leaves it as it is.
    #[test]
    fn the_weenie_description_sets_the_no_draw() {
        let desc = |bitfield: u32| {
            WeenieQualities::from_weenie_desc(ac2_net::protocol::WeenieDesc {
                data_mask: 0x0100_0000 | 0x4000,
                bitfield: Some(bitfield),
                ..Default::default()
            })
        };
        assert_eq!(desc(4).set_weenie_desc_no_draw(), Some(true));
        assert_eq!(desc(4 | 8).set_weenie_desc_no_draw(), Some(true));
        assert_eq!(desc(0).set_weenie_desc_no_draw(), Some(false));
        assert_eq!(desc(8).set_weenie_desc_no_draw(), None);
        assert_eq!(
            WeenieQualities::default().set_weenie_desc_no_draw(),
            Some(false)
        );
    }

    /// `CWRepository::GatherPhysicsState` and `BuildPhysObjProfile`: absent bools leave report,
    /// ignore-collisions and gravity on and the rest off; the placement and scale default to 0 and 1.
    #[test]
    fn the_physics_profile_follows_the_engine_bools() {
        let mut q = WeenieQualities::default();
        assert_eq!(q.phys_obj_profile(), (0x4 | 0x8 | 0x200, 0, 1.0));
        for (stype, value) in [(1, 0), (2, 0), (4, 0), (5, 1), (6, 1), (7, 1)] {
            q.set(stype, &QualityValue::Bool(value));
        }
        q.set(1, &QualityValue::Int(3));
        q.set(0x12A, &QualityValue::Float(2.0));
        assert_eq!(q.phys_obj_profile(), (0x400 | 0x2000 | 0x8000, 3, 2.0));
    }

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

    #[test]
    fn the_defaults_did_follows_the_package_and_the_entity() {
        // `CWRepository::Construct`: no entity DID reads the package's record, an entity
        // description its own, anything else none.
        assert_eq!(
            WeenieQualities::defaults_did(Some(0x1169), None),
            0x8000_1169
        );
        assert_eq!(
            WeenieQualities::defaults_did(Some(0x1169), Some(0)),
            0x8000_1169
        );
        assert_eq!(
            WeenieQualities::defaults_did(None, Some(0x4700_07D1)),
            0x8100_07D1
        );
        assert_eq!(
            WeenieQualities::defaults_did(Some(7), Some(0x4700_07D1)),
            0x8100_07D1
        );
        assert_eq!(WeenieQualities::defaults_did(None, Some(0x0400_0001)), 0);
        assert_eq!(WeenieQualities::defaults_did(None, None), 0);
    }

    #[test]
    fn inq_falls_back_on_the_class_defaults_after_the_tables() {
        let mut defaults = WeenieQualities::default();
        defaults.ints.insert(0x14, 0x1004F);
        defaults.ints.insert(0x12E, 24);
        defaults.bools.insert(0x270, 1);
        let mut q = WeenieQualities::default();
        q.ints.insert(0x12E, 5);
        assert_eq!(q.inq_int(0x14), None);
        q.set_defaults(std::sync::Arc::new(defaults));
        // The object's own table answers first; the defaults fill the rest.
        assert_eq!(q.inq_int(0x12E), Some(5));
        assert_eq!(q.inq_int(0x14), Some(0x1004F));
        assert_eq!(q.inq_bool(0x270), Some(1));
        assert_eq!(q.inq_bool(0x271), None);
    }
}

#[cfg(test)]
mod truncate_tests {
    use super::*;

    #[test]
    fn an_int_change_is_clamped_to_its_bounds() {
        assert_eq!(truncate_int(5, 10, 0, 100, 12), 12);
        assert_eq!(truncate_int(5, 10, 0, 100, 150), 100);
        assert_eq!(truncate_int(-5, 10, 0, 100, 3), 3);
        assert_eq!(truncate_int(-5, 10, 0, 100, -4), 0);
    }

    #[test]
    fn a_wrapped_int_change_goes_to_the_bound_it_wrapped_toward() {
        // A positive change that went down wrapped past the maximum.
        assert_eq!(truncate_int(5, 10, 0, 100, 8), 100);
        // A negative change that went up wrapped past the minimum.
        assert_eq!(truncate_int(-5, 10, 0, 100, 12), 0);
        assert_eq!(truncate_long_int(5, 10, 0, 100, 8), 100);
        assert_eq!(truncate_long_int(-5, 10, 0, 100, 12), 0);
    }

    #[test]
    fn a_long_int_uses_the_full_64_bit_range() {
        let big = 1_i64 << 40;
        assert_eq!(truncate_long_int(1, big, 0, i64::MAX, big + 1), big + 1);
        assert_eq!(truncate_long_int(1, big, 0, big, big + 1), big);
    }

    #[test]
    fn a_float_wrapped_upward_by_a_positive_modifier_keeps_its_value_within_bounds() {
        assert_eq!(truncate_float(0.5, 1.0, 0.0, 10.0, 1.5), 1.5);
        // A positive change that went down (overflow) takes the maximum.
        assert_eq!(truncate_float(0.5, 1.0, 0.0, 10.0, 0.5), 10.0);
        // A negative change that went up takes the minimum.
        assert_eq!(truncate_float(-0.5, 1.0, 0.0, 10.0, 1.5), 0.0);
        assert_eq!(truncate_float(0.0, 3.0, 0.0, 2.0, 3.0), 2.0);
    }
}
