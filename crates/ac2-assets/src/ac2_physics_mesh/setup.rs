//! Asheron's Call 2 `CSetup` (`0x04`) collision primitives.
//!
//! `CharacterSetup::UnPack` order (see `ac2_setup`): flags, meshes, bones, placement frames,
//! holding and connection locations, then the `CCylSphere` list (low point, radius, height) and
//! the `CSphere` list (center, radius), followed by fixed physics words, planes, `DegradeInfo`
//! and the render box. `CPhysicsObj::FindObjCollisions` collides an
//! object without a physics tree through its cylspheres, or else its spheres.
//!
//! The four physics words are `CSetup::height`, `radius`, `step_up_height` and
//! `step_down_height` (`CharacterSetup::UnPack`; read back by
//! `CPartArray::GetHeight`/`GetRadius`/`GetStepUpHeight`/`GetStepDownHeight` at Setup offsets
//! 0x74, 0x78, 0x68 and 0x64). Flag bit 2 is `allow_free_heading` (offset 0x70).

use std::sync::Arc;

use ac2_core::QualifiedDataId;
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::{Sphere, Vec3};

const MAX_ITEMS: usize = 4096;
const TRANSFORM_BYTES: usize = 40;
/// Two spheres, 18 `DegradeInfo` distances, a scalar, a word, the render box after the four
/// physics words.
const TAIL_BYTES: usize = 16 + 16 + 18 * 4 + 4 + 4 + 24;
/// `CharacterSetup::UnPack` flag bit stored as `allow_free_heading`.
const FLAG_ALLOW_FREE_HEADING: u32 = 0x4;

/// An upright collision cylinder in Setup space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2CylSphere {
    pub low_point: Vec3,
    pub radius: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ac2SetupPhysics {
    pub cylspheres: Arc<[Ac2CylSphere]>,
    pub spheres: Arc<[Sphere]>,
    /// `CSetup::height` (the human male body is 2.06 m).
    pub height: f32,
    /// `CSetup::radius`.
    pub radius: f32,
    /// `CSetup::step_up_height`, unscaled (`CPartArray::GetStepUpHeight`).
    pub step_up_height: f32,
    /// `CSetup::step_down_height`, unscaled (`CPartArray::GetStepDownHeight`).
    pub step_down_height: f32,
    pub allow_free_heading: bool,
}

impl ResourcePackedDecode for Ac2SetupPhysics {
    const TYPE_NAME: &'static str = "AC2 Setup physics";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let flags = cursor.read_u32()?;
        let skip_counted = |cursor: &mut ArchiveCursor<'_>, item: usize| -> Option<()> {
            let count = cursor.read_u32_count().ok()?;
            (count <= MAX_ITEMS).then_some(())?;
            cursor.read_bytes(count.checked_mul(item)?).ok().map(drop)
        };
        skip_counted(&mut cursor, 4).ok_or_else(|| malformed("meshes"))?;
        skip_counted(&mut cursor, 8).ok_or_else(|| malformed("bones"))?;
        let frames = cursor.read_u32_count()?;
        if frames > MAX_ITEMS {
            return Err(malformed("placement count"));
        }
        for _ in 0..frames {
            let _kind = cursor.read_u32()?;
            skip_counted(&mut cursor, TRANSFORM_BYTES).ok_or_else(|| malformed("frame bones"))?;
        }
        for _ in 0..2 {
            skip_counted(&mut cursor, 8 + TRANSFORM_BYTES).ok_or_else(|| malformed("locations"))?;
        }
        let floats =
            |cursor: &mut ArchiveCursor<'_>, width: usize| -> ResourceResult<Vec<Vec<f32>>> {
                let count = cursor.read_u32_count()?;
                if count > MAX_ITEMS || count * width * 4 > cursor.remaining() {
                    return Err(malformed("primitive count"));
                }
                (0..count)
                    .map(|_| Ok(cursor.read_f32_items(width)?))
                    .collect()
            };
        let cylspheres = floats(&mut cursor, 5)?
            .into_iter()
            .map(|v| Ac2CylSphere {
                low_point: Vec3::new(v[0], v[1], v[2]),
                radius: v[3],
                height: v[4],
            })
            .collect();
        let spheres = floats(&mut cursor, 4)?
            .into_iter()
            .map(|v| Sphere {
                origin: Vec3::new(v[0], v[1], v[2]),
                radius: v[3],
            })
            .collect();
        let [height, radius, step_up_height, step_down_height] = [(); 4].map(|_| cursor.read_f32());
        if cursor.remaining() != TAIL_BYTES {
            return Err(malformed("Setup tail length"));
        }
        Ok(Self {
            cylspheres,
            spheres,
            height: height?,
            radius: radius?,
            step_up_height: step_up_height?,
            step_down_height: step_down_height?,
            allow_free_heading: flags & FLAG_ALLOW_FREE_HEADING != 0,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.cylspheres.len() * std::mem::size_of::<Ac2CylSphere>()
            + self.spheres.len() * std::mem::size_of::<Sphere>()
    }
}
