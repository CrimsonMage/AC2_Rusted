//! `VisualDesc::StreamPack` and the part-group, icon and appearance structures it carries
//! (identical in admin 7148 and Legions 7160: Legions 0x005F1DC0).

use crate::protocol::fx::{fx_overrides, FxData};
use crate::protocol::reader::Reader;
use crate::protocol::types::{float_table, hash_count};
use crate::protocol::ProtocolError;

/// `PartGroupDataDesc::StreamPack`: a pack bitfield, then the fields it names.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PartGroupDesc {
    pub bitfield: u32,
    pub key: u32,
    pub parent_key: u32,
    pub connection_point: u32,
    pub setup: u32,
    pub anim_map: u32,
    /// `m_app_hash`: appearance key -> `AppearanceInfo` (appearance key -> scalar).
    pub appearance: Vec<(u32, Vec<(u32, f32)>)>,
    pub fx_table: u32,
    /// `m_startup_fx`: FX id -> scalar.
    pub startup_fx: Vec<(u32, f32)>,
    /// `m_fx_overrides`: FX id -> its `FXData` variants.
    pub fx_overrides: Vec<(u32, Vec<FxData>)>,
}

impl PartGroupDesc {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bitfield = reader.u32()?;
        let mut word = |bit: u32| -> Result<u32, ProtocolError> {
            if bitfield & bit != 0 {
                reader.u32()
            } else {
                Ok(0)
            }
        };
        let mut desc = Self {
            bitfield,
            key: word(0x1)?,
            parent_key: word(0x2)?,
            connection_point: word(0x4)?,
            setup: word(0x8)?,
            anim_map: word(0x10)?,
            ..Self::default()
        };
        if bitfield & 0x20 != 0 {
            for _ in 0..hash_count(reader)? {
                let key = reader.u32()?;
                desc.appearance.push((key, float_table(reader)?));
            }
        }
        if bitfield & 0x40 != 0 {
            desc.fx_table = reader.u32()?;
        }
        if bitfield & 0x80 != 0 {
            desc.startup_fx = float_table(reader)?;
        }
        if bitfield & 0x100 != 0 {
            desc.fx_overrides = fx_overrides(reader)?;
        }
        Ok(desc)
    }
}

/// `IconLayerDesc::StreamPack`: layer id, image DID, shift colour (`RGBAColor`, packed 32-bit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconLayer {
    pub layer: u32,
    pub image: u32,
    pub color: u32,
}

/// `VisualDesc::StreamPack`: a pack bitfield, then the fields it names.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VisualDesc {
    pub bitfield: u32,
    /// Bit 0: the `DBObj` DID (`DBObj::UnPack_Type`; `VisualDesc` is not categorized).
    pub did: Option<u32>,
    pub parent: Option<u32>,
    pub motion_interp: Option<u32>,
    pub behavior_table: Option<u32>,
    pub modes: Option<u32>,
    pub scale: Option<[f32; 3]>,
    pub child_scale: Option<f32>,
    pub particle_scale: Option<f32>,
    pub icon: Vec<IconLayer>,
    pub global_mods: Option<PartGroupDesc>,
    /// `m_pgd_desc_table`: part-group key -> description.
    pub part_groups: Vec<(u32, PartGroupDesc)>,
}

impl VisualDesc {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bitfield = reader.u32()?;
        let mut word = |bit: u32| -> Result<Option<u32>, ProtocolError> {
            if bitfield & bit != 0 {
                reader.u32().map(Some)
            } else {
                Ok(None)
            }
        };
        let did = word(0x1)?;
        let parent = word(0x2)?;
        let motion_interp = word(0x4)?;
        let behavior_table = word(0x8)?;
        let modes = word(0x10)?;
        let scale = (bitfield & 0x20 != 0).then(|| reader.vec3()).transpose()?;
        let child_scale = (bitfield & 0x40 != 0).then(|| reader.f32()).transpose()?;
        let particle_scale = (bitfield & 0x400 != 0).then(|| reader.f32()).transpose()?;
        let mut icon = Vec::new();
        if bitfield & 0x80 != 0 {
            // `PList<IconLayerDesc>::UnPack`: a u32 count, then the layers.
            for _ in 0..reader.count()? {
                icon.push(IconLayer {
                    layer: reader.u32()?,
                    image: reader.u32()?,
                    color: reader.u32()?,
                });
            }
        }
        let global_mods = (bitfield & 0x100 != 0)
            .then(|| PartGroupDesc::read(reader))
            .transpose()?;
        let mut part_groups = Vec::new();
        if bitfield & 0x200 != 0 {
            for _ in 0..hash_count(reader)? {
                let key = reader.u32()?;
                part_groups.push((key, PartGroupDesc::read(reader)?));
            }
        }
        Ok(Self {
            bitfield,
            did,
            parent,
            motion_interp,
            behavior_table,
            modes,
            scale,
            child_scale,
            particle_scale,
            icon,
            global_mods,
            part_groups,
        })
    }

    /// The root part group's Setup (no parent key), else the first part group's.
    pub fn root_setup(&self) -> Option<u32> {
        let groups = || self.part_groups.iter().map(|(_, g)| g);
        groups()
            .find(|g| g.parent_key == 0 && g.setup != 0)
            .or_else(|| groups().find(|g| g.setup != 0))
            .map(|g| g.setup)
    }
}

#[cfg(test)]
mod tests {
    use super::VisualDesc;
    use crate::protocol::reader::Reader;

    #[test]
    fn fields_follow_the_pack_bitfield_in_stream_order() {
        let mut b = Vec::new();
        b.extend(0x0000_02A2u32.to_le_bytes()); // parent, scale, icon, part groups
        b.extend(0x1F00_0023u32.to_le_bytes());
        for v in [1.0f32, 2.0, 3.0] {
            b.extend(v.to_le_bytes());
        }
        b.extend(1u32.to_le_bytes());
        b.extend(
            [7u32, 0x0600_0001, 0xFF00_00FF]
                .iter()
                .flat_map(|v| v.to_le_bytes()),
        );
        b.extend((1u32 | (3 << 24)).to_le_bytes()); // one part group, bucket index 3
        b.extend(0x10u32.to_le_bytes());
        b.extend(0x09u32.to_le_bytes()); // key, setup
        b.extend(0x10u32.to_le_bytes());
        b.extend(0x0200_0001u32.to_le_bytes());
        let mut reader = Reader::new(&b);
        let desc = VisualDesc::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!(desc.parent, Some(0x1F00_0023));
        assert_eq!(desc.scale, Some([1.0, 2.0, 3.0]));
        assert_eq!(desc.icon[0].image, 0x0600_0001);
        assert_eq!(desc.part_groups[0].0, 0x10);
        assert_eq!(desc.root_setup(), Some(0x0200_0001));
        for end in 0..b.len() {
            assert!(VisualDesc::read(&mut Reader::new(&b[..end])).is_err());
        }
    }

    #[test]
    fn fx_overrides_follow_the_startup_fx() {
        let mut b = Vec::new();
        for v in [0x180u32, 1, 0x22, 0, 1, 0x22, 1] {
            b.extend(v.to_le_bytes()); // startup fx 0x22 -> 0.0, then one override, one FXData
        }
        b.extend(0x20_0000u64.to_le_bytes());
        b.extend([1u32, 0].iter().flat_map(|v| v.to_le_bytes())); // unparent, no terrain
        let mut reader = Reader::new(&b);
        let group = super::PartGroupDesc::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!(group.startup_fx, vec![(0x22, 0.0)]);
        assert_eq!(group.fx_overrides[0].1[0].default.unparent, Some(1));
    }
}
