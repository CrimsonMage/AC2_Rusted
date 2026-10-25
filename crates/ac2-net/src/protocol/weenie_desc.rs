//! `gmWeenieDesc::StreamPack` (identical in admin 7148 and Legions 7160: Legions 0x005C46A0): a
//! data mask, then the fields it names, in stream order.

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WeenieDesc {
    pub data_mask: u32,
    pub package_id: Option<u32>,
    pub entity_did: Option<u32>,
    pub bitfield: Option<u32>,
    pub name: Option<StringInfo>,
    pub plural_name: Option<StringInfo>,
    pub icon: Option<u32>,
    pub container: Option<u64>,
    pub wielder: Option<u64>,
    pub monarch: Option<u64>,
    pub originator: Option<u64>,
    pub claimant: Option<u64>,
    pub killer: Option<u64>,
    pub pet_summoner: Option<u64>,
    pub scale: Option<f32>,
    pub quantity: Option<i32>,
    pub value: Option<i32>,
    pub faction_type: Option<i32>,
    pub pk_always_true: Option<i32>,
    pub pk_always_false: Option<i32>,
    pub physics_type: (Option<i32>, Option<i32>),
    pub movement_ethereal: (Option<i32>, Option<i32>),
    pub placement_ethereal: (Option<i32>, Option<i32>),
    pub durability_current: Option<i32>,
    pub durability_max: Option<i32>,
}

impl WeenieDesc {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let mask = reader.u32()?;
        let has = |bit: u32| mask & bit != 0;
        let mut w = Self {
            data_mask: mask,
            ..Self::default()
        };
        macro_rules! field {
            ($bit:expr, $read:expr) => {
                if has($bit) {
                    Some($read)
                } else {
                    None
                }
            };
        }
        w.package_id = field!(0x1, reader.u32()?);
        w.entity_did = field!(0x0200_0000, reader.u32()?);
        w.bitfield = field!(0x2, reader.u32()?);
        w.name = field!(0x4, StringInfo::read(reader)?);
        w.plural_name = field!(0x8, StringInfo::read(reader)?);
        w.icon = field!(0x10, reader.u32()?);
        w.container = field!(0x20, reader.u64()?);
        w.wielder = field!(0x40, reader.u64()?);
        w.monarch = field!(0x100, reader.u64()?);
        w.originator = field!(0x200, reader.u64()?);
        w.claimant = field!(0x2000_0000, reader.u64()?);
        w.killer = field!(0x4000_0000, reader.u64()?);
        w.pet_summoner = field!(0x0400_0000, reader.u64()?);
        w.scale = field!(0x80, reader.f32()?);
        let mut int = |bit: u32| -> Result<Option<i32>, ProtocolError> {
            Ok(if mask & bit != 0 {
                Some(reader.u32()? as i32)
            } else {
                None
            })
        };
        w.quantity = int(0x800)?;
        w.value = int(0x1000)?;
        w.faction_type = int(0x2000)?;
        w.pk_always_true = int(0x20_0000)?;
        w.pk_always_false = int(0x40_0000)?;
        w.physics_type = (int(0x8000)?, int(0x1_0000)?);
        w.movement_ethereal = (int(0x2_0000)?, int(0x4_0000)?);
        w.placement_ethereal = (int(0x8_0000)?, int(0x10_0000)?);
        w.durability_current = int(0x0800_0000)?;
        w.durability_max = int(0x1000_0000)?;
        Ok(w)
    }
}

impl WeenieDesc {
    /// The data mask the present fields need, each field's bit (the physics, movement and
    /// placement pairs one bit per half).
    pub fn mask(&self) -> u32 {
        let bits: [(bool, u32); 24] = [
            (self.package_id.is_some(), 0x1),
            (self.entity_did.is_some(), 0x0200_0000),
            (self.bitfield.is_some(), 0x2),
            (self.name.is_some(), 0x4),
            (self.plural_name.is_some(), 0x8),
            (self.icon.is_some(), 0x10),
            (self.container.is_some(), 0x20),
            (self.wielder.is_some(), 0x40),
            (self.monarch.is_some(), 0x100),
            (self.originator.is_some(), 0x200),
            (self.claimant.is_some(), 0x2000_0000),
            (self.killer.is_some(), 0x4000_0000),
            (self.pet_summoner.is_some(), 0x0400_0000),
            (self.scale.is_some(), 0x80),
            (self.quantity.is_some(), 0x800),
            (self.value.is_some(), 0x1000),
            (self.faction_type.is_some(), 0x2000),
            (self.pk_always_true.is_some(), 0x20_0000),
            (self.pk_always_false.is_some(), 0x40_0000),
            (self.physics_type.0.is_some(), 0x8000),
            (self.physics_type.1.is_some(), 0x1_0000),
            (self.movement_ethereal.0.is_some(), 0x2_0000),
            (self.durability_current.is_some(), 0x0800_0000),
            (self.durability_max.is_some(), 0x1000_0000),
        ];
        let more = [
            (self.movement_ethereal.1.is_some(), 0x4_0000),
            (self.placement_ethereal.0.is_some(), 0x8_0000),
            (self.placement_ethereal.1.is_some(), 0x10_0000),
        ];
        bits.iter()
            .chain(&more)
            .filter(|(present, _)| *present)
            .fold(0, |m, (_, bit)| m | bit)
    }

    /// `gmWeenieDesc::StreamPack` packing: [`WeenieDesc::mask`], then the fields in
    /// [`WeenieDesc::read`]'s order (the server's side; the double's objects).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.mask());
        let words = [self.package_id, self.entity_did, self.bitfield];
        for v in words.into_iter().flatten() {
            w.u32(v);
        }
        for info in [&self.name, &self.plural_name].into_iter().flatten() {
            info.write(&mut w);
        }
        if let Some(icon) = self.icon {
            w.u32(icon);
        }
        let ids = [
            self.container,
            self.wielder,
            self.monarch,
            self.originator,
            self.claimant,
            self.killer,
            self.pet_summoner,
        ];
        for id in ids.into_iter().flatten() {
            w.u64(id);
        }
        if let Some(scale) = self.scale {
            w.f32(scale);
        }
        let ints = [
            self.quantity,
            self.value,
            self.faction_type,
            self.pk_always_true,
            self.pk_always_false,
            self.physics_type.0,
            self.physics_type.1,
            self.movement_ethereal.0,
            self.movement_ethereal.1,
            self.placement_ethereal.0,
            self.placement_ethereal.1,
            self.durability_current,
            self.durability_max,
        ];
        for v in ints.into_iter().flatten() {
            w.u32(v as u32);
        }
        w.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::WeenieDesc;
    use crate::protocol::reader::Reader;
    use crate::protocol::string_info::StringInfo;

    #[test]
    fn the_entity_did_precedes_the_bitfield_and_scale_follows_the_ids() {
        let mut b = Vec::new();
        b.extend((0x0200_0000u32 | 0x2 | 0x20 | 0x80 | 0x800).to_le_bytes());
        b.extend(0x4700_110Fu32.to_le_bytes());
        b.extend(0x10u32.to_le_bytes());
        b.extend(7u64.to_le_bytes());
        b.extend(1.25f32.to_le_bytes());
        b.extend(3u32.to_le_bytes());
        let mut reader = Reader::new(&b);
        let w = WeenieDesc::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            (w.entity_did, w.bitfield, w.container, w.scale, w.quantity),
            (Some(0x4700_110F), Some(0x10), Some(7), Some(1.25), Some(3))
        );
    }

    #[test]
    fn a_desc_packs_back_to_what_it_read() {
        let desc = WeenieDesc {
            package_id: Some(0x1AA9),
            name: Some(StringInfo::literal("Healing Potion")),
            icon: Some(0x0600_1234),
            container: Some(0x2000_0000_0000_0001),
            quantity: Some(5),
            value: Some(12),
            durability_current: Some(3),
            durability_max: Some(9),
            ..WeenieDesc::default()
        };
        let bytes = desc.to_bytes();
        let back = WeenieDesc::read(&mut Reader::new(&bytes)).unwrap();
        assert_eq!(back.data_mask, desc.mask());
        assert_eq!(
            WeenieDesc {
                data_mask: 0,
                ..back
            },
            desc
        );
    }
}
