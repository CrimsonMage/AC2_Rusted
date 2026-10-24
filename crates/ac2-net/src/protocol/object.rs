//! `CreateObject` and the descriptions it carries: `VisualDesc`, `PhysicsDesc`, then the
//! `gmWeenieDesc` this decoder keeps as raw bytes.

use crate::protocol::position::CellPosition;
use crate::protocol::reader::Reader;
use crate::protocol::ProtocolError;

/// `VisualDesc::StreamPack`: a pack bitfield, then the fields it names. The DBObj header (bit
/// 0), icon description (bit 7), global appearance modifiers (bit 8) and part-group table (bit
/// 9) are not unpacked here.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VisualDesc {
    pub bitfield: u32,
    pub parent: Option<u32>,
    pub motion_interp: Option<u32>,
    pub behavior_table: Option<u32>,
    pub modes: Option<u32>,
    pub scale: Option<[f32; 3]>,
    pub child_scale: Option<f32>,
    pub particle_scale: Option<f32>,
}

impl VisualDesc {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bitfield = reader.u32()?;
        if bitfield & 0x381 != 0 {
            return Err(ProtocolError::Unsupported(
                "VisualDesc DBObj/icon/appearance/part groups",
            ));
        }
        let mut field = |bit: u32| -> Result<Option<u32>, ProtocolError> {
            if bitfield & bit != 0 {
                reader.u32().map(Some)
            } else {
                Ok(None)
            }
        };
        let parent = field(0x2)?;
        let motion_interp = field(0x4)?;
        let behavior_table = field(0x8)?;
        let modes = field(0x10)?;
        let scale = if bitfield & 0x20 != 0 {
            Some(reader.vec3()?)
        } else {
            None
        };
        let child_scale = (bitfield & 0x40 != 0).then(|| reader.f32()).transpose()?;
        let particle_scale = (bitfield & 0x400 != 0).then(|| reader.f32()).transpose()?;
        Ok(Self {
            bitfield,
            parent,
            motion_interp,
            behavior_table,
            modes,
            scale,
            child_scale,
            particle_scale,
        })
    }
}

/// `PhysicsDesc::StreamPack`, in its wire order. Behaviours (bit 0) and FX (bit 9) are not
/// unpacked here.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhysicsDesc {
    pub bitfield: u32,
    pub instance_stamp: u16,
    pub visual_ordering_stamp: u16,
    pub mode: Option<u32>,
    /// Animation sliders: key and the two slider floats.
    pub sliders: Vec<(u32, f32, f32)>,
    pub anim_frame: Option<u32>,
    /// Cell position and orientation quaternion (w, x, y, z).
    pub position: Option<(CellPosition, [f32; 4])>,
    /// Parent object, location and the parent's instance stamp.
    pub parent: Option<(u64, u32, u16)>,
    pub orientation_id: Option<u32>,
    pub velocity: Option<[f32; 3]>,
    pub external_acceleration: Option<[f32; 3]>,
    pub velocity_scale: Option<f32>,
    pub jump_scale: Option<f32>,
    pub acceleration: Option<[f32; 3]>,
    pub omega: Option<[f32; 3]>,
    pub look_at: Option<u64>,
    pub head_x: Option<f32>,
    pub head_z: Option<f32>,
    pub target: Option<u64>,
    pub target_position: Option<(CellPosition, [f32; 4])>,
    pub target_offset: Option<[f32; 3]>,
    pub timestamps: [u16; 4],
}

fn framed_position(reader: &mut Reader<'_>) -> Result<(CellPosition, [f32; 4]), ProtocolError> {
    let cell = reader.u32()?;
    let origin = reader.vec3()?;
    let orientation = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
    if origin
        .iter()
        .chain(orientation.iter())
        .any(|v| !v.is_finite())
    {
        return Err(ProtocolError::Invalid("physics position"));
    }
    Ok((CellPosition { cell, origin }, orientation))
}

impl PhysicsDesc {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bitfield = reader.u32()?;
        if bitfield & 0x201 != 0 {
            return Err(ProtocolError::Unsupported("PhysicsDesc behaviours or FX"));
        }
        let mut desc = Self {
            bitfield,
            instance_stamp: reader.u16()?,
            visual_ordering_stamp: reader.u16()?,
            ..Self::default()
        };
        let has = |bit: u32| bitfield & bit != 0;
        if has(0x80) {
            desc.mode = Some(reader.u32()?);
        }
        if has(0x2) {
            let count = reader.u32()?;
            if count > 64 {
                return Err(ProtocolError::Invalid("slider count"));
            }
            for _ in 0..count {
                desc.sliders
                    .push((reader.u32()?, reader.f32()?, reader.f32()?));
            }
        }
        if has(0x2_0000) {
            desc.anim_frame = Some(reader.u32()?);
        }
        if has(0x8000) {
            desc.position = Some(framed_position(reader)?);
        }
        if has(0x20) {
            let parent = reader.u64()?;
            let location = reader.u32()?;
            let stamp = reader.u16()?;
            reader.align()?;
            desc.parent = Some((parent, location, stamp));
        }
        if has(0x100) {
            desc.orientation_id = Some(reader.u32()?);
        }
        if has(0x4) {
            desc.velocity = Some(reader.vec3()?);
        }
        if has(0x40_0000) {
            desc.external_acceleration = Some(reader.vec3()?);
        }
        if has(0x80_0000) {
            desc.velocity_scale = Some(reader.f32()?);
        }
        if has(0x100_0000) {
            desc.jump_scale = Some(reader.f32()?);
        }
        if has(0x8) {
            desc.acceleration = Some(reader.vec3()?);
        }
        if has(0x10) {
            desc.omega = Some(reader.vec3()?);
        }
        if has(0x200_0000) {
            desc.look_at = Some(reader.u64()?);
        }
        if has(0x400_0000) {
            desc.head_x = Some(reader.f32()?);
        }
        if has(0x800_0000) {
            desc.head_z = Some(reader.f32()?);
        }
        if has(0x400) {
            desc.target = Some(reader.u64()?);
        }
        if has(0x800) {
            desc.target_position = Some(framed_position(reader)?);
        }
        if has(0x1000) {
            desc.target_offset = Some(reader.vec3()?);
        }
        for stamp in &mut desc.timestamps {
            *stamp = reader.u16()?;
        }
        reader.align()?;
        Ok(desc)
    }
}

/// `CM_Physics::RecvEvt_CreateObject`: the object id, its visual and physics descriptions, and
/// the trailing `gmWeenieDesc` kept as raw bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateObject {
    pub id: u64,
    pub visual: VisualDesc,
    pub physics: PhysicsDesc,
    pub weenie: Vec<u8>,
}

impl CreateObject {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let id = reader.u64()?;
        let visual = VisualDesc::read(reader)?;
        let physics = PhysicsDesc::read(reader)?;
        Ok(Self {
            id,
            visual,
            physics,
            weenie: reader.rest().to_vec(),
        })
    }

    /// The entity DID when the weenie description leads with one (bit `0x02000000`).
    pub fn entity_did(&self) -> Option<u32> {
        let mask = u32::from_le_bytes(self.weenie.get(..4)?.try_into().ok()?);
        if mask & 0x0200_0000 == 0 {
            return None;
        }
        Some(u32::from_le_bytes(self.weenie.get(4..8)?.try_into().ok()?))
    }
}

#[cfg(test)]
mod tests {
    use super::CreateObject;
    use crate::protocol::reader::Reader;

    /// A placed object with a mode, one slider and a position, as servers send a standing body.
    fn placed_object() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(0x8140_0000_0003_0003u64.to_le_bytes());
        b.extend(2u32.to_le_bytes());
        b.extend(0x1F00_0023u32.to_le_bytes());
        b.extend(0x8082u32.to_le_bytes());
        b.extend(5u16.to_le_bytes());
        b.extend(6u16.to_le_bytes());
        b.extend(0x4000_0001u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(0x4000_000Au32.to_le_bytes());
        b.extend(1.0f32.to_le_bytes());
        b.extend(0.0f32.to_le_bytes());
        b.extend(0x0299_0109u32.to_le_bytes());
        for v in [59.0f32, 240.0, -44.5, 1.0, 0.0, 0.0, 0.0] {
            b.extend(v.to_le_bytes());
        }
        for stamp in [1u16, 2, 3, 4] {
            b.extend(stamp.to_le_bytes());
        }
        b.extend(0x0200_0000u32.to_le_bytes());
        b.extend(0x4700_110Fu32.to_le_bytes());
        b
    }

    #[test]
    fn decodes_visual_physics_and_the_weenie_entity() {
        let bytes = placed_object();
        let object = CreateObject::read(&mut Reader::new(&bytes)).unwrap();
        assert_eq!(object.visual.parent, Some(0x1F00_0023));
        assert_eq!(object.physics.instance_stamp, 5);
        assert_eq!(object.physics.mode, Some(0x4000_0001));
        assert_eq!(object.physics.sliders, vec![(0x4000_000A, 1.0, 0.0)]);
        let (position, orientation) = object.physics.position.unwrap();
        assert_eq!(
            (position.cell, position.origin),
            (0x0299_0109, [59.0, 240.0, -44.5])
        );
        assert_eq!(orientation, [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(object.physics.timestamps, [1, 2, 3, 4]);
        assert_eq!(object.entity_did(), Some(0x4700_110F));
        for end in 0..bytes.len() - 8 {
            assert!(
                CreateObject::read(&mut Reader::new(&bytes[..end])).is_err(),
                "{end}"
            );
        }
    }

    #[test]
    fn a_contained_item_has_a_parent_and_no_position() {
        let mut b = Vec::new();
        b.extend(9u64.to_le_bytes());
        b.extend(2u32.to_le_bytes());
        b.extend(0x1F00_0056u32.to_le_bytes());
        b.extend(0x20u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(0u16.to_le_bytes());
        b.extend(7u64.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(4u16.to_le_bytes());
        b.extend(0u16.to_le_bytes());
        b.extend([0; 8]);
        let object = CreateObject::read(&mut Reader::new(&b)).unwrap();
        assert_eq!(object.physics.parent, Some((7, 1, 4)));
        assert!(object.physics.position.is_none());
        assert!(object.weenie.is_empty());
    }
}
