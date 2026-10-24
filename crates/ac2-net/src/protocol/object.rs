//! `CM_Physics::RecvEvt_CreateObject` (event `0x10058`, dispatched by
//! `gmCEventQueue::DispatchEvent`): the object id, then `VisualDesc`, `PhysicsDesc` and
//! `gmWeenieDesc`, each through its `StreamPack`.

use crate::protocol::physics::PhysicsDesc;
use crate::protocol::reader::Reader;
use crate::protocol::visual::VisualDesc;
use crate::protocol::weenie_desc::WeenieDesc;
use crate::protocol::ProtocolError;

#[derive(Debug, Clone, PartialEq)]
pub struct CreateObject {
    pub id: u64,
    pub visual: VisualDesc,
    pub physics: PhysicsDesc,
    pub weenie: WeenieDesc,
}

impl CreateObject {
    /// Bytes after the weenie description are not read (`RecvEvt_CreateObject` does not check).
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let id = reader.u64()?;
        reader.align()?;
        let visual = VisualDesc::read(reader)?;
        let physics = PhysicsDesc::read(reader)?;
        let weenie = WeenieDesc::read(reader)?;
        reader.rest();
        Ok(Self {
            id,
            visual,
            physics,
            weenie,
        })
    }

    pub fn entity_did(&self) -> Option<u32> {
        self.weenie.entity_did
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
        let position = object.physics.position.unwrap();
        assert_eq!(
            (position.cell, position.origin, position.rotation),
            (0x0299_0109, [59.0, 240.0, -44.5], [1.0, 0.0, 0.0, 0.0])
        );
        assert_eq!(object.physics.timestamps, [1, 2, 3, 4]);
        assert_eq!(object.entity_did(), Some(0x4700_110F));
        for end in 0..bytes.len() {
            assert!(
                CreateObject::read(&mut Reader::new(&bytes[..end])).is_err(),
                "{end}"
            );
        }
    }
}
