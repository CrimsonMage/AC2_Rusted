//! Asheron's Call 2 `CSetup` (`0x04`), decoded in `CharacterSetup::UnPack` order: flags, mesh DIDs,
//! bones (parent index, name id), placement frames of per-bone `TransformBase`s, holding and
//! connection `LocationType`s, cylspheres, spheres, physics words, height/radius planes,
//! `DegradeInfo`, then a scalar, a word and the render `BBox`.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::{Quat, Vec3};

const MAX_ITEMS: usize = 4096;
const TRANSFORM_BYTES: usize = 40;

/// `TransformBase`: offset, per-axis scale and rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2Transform {
    pub offset: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl Ac2Transform {
    pub const IDENTITY: Self = Self {
        offset: Vec3::ZERO,
        scale: Vec3::ONE,
        rotation: Quat::IDENTITY,
    };

    pub fn apply(&self, point: Vec3) -> Vec3 {
        self.offset + self.rotation * (point * self.scale)
    }

    pub fn to_mat4(&self) -> ac2_math::Mat4 {
        ac2_math::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.offset)
    }

    /// `self` applied after `inner` (`inner` is the child's local transform).
    pub fn compose(&self, inner: &Self) -> Self {
        Self {
            offset: self.apply(inner.offset),
            scale: self.scale * inner.scale,
            rotation: (self.rotation * inner.rotation).normalize(),
        }
    }
}

/// A Setup bone: parent index (`None` for a root) and its name enum id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2Bone {
    pub parent: Option<usize>,
    pub name: u32,
}

/// A named placement frame (`PlacementFrame`): one local transform per bone.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2PlacementFrame {
    pub kind: u32,
    pub bones: Arc<[Ac2Transform]>,
}

/// A `LocationType`: an id, the bone it rides and its transform in that bone's space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2Location {
    pub id: u32,
    pub bone: usize,
    pub transform: Ac2Transform,
}

/// One detail setting's distances in `DegradeInfo::m_DLDegradeInfo` (`CPartArray::Draw` scales
/// each by the object's degrade scale before using it).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ac2DegradeDistances {
    /// `m_fVisibleDistance`: past it the object is not drawn; it fades over the last 7.5%.
    pub visible: f32,
    /// `m_fNearDegradeDistance`: the meshes start to degrade.
    pub near: f32,
    /// `m_fIdealDegradeDistance`: the degrade factor reaches one half.
    pub ideal: f32,
    /// `m_fFarDegradeDistance`: the degrade factor reaches zero (the lowest degrade).
    pub far: f32,
    /// `m_fHighDetailDistance`: three times it, high material detail drops to medium.
    pub high_detail: f32,
    /// `m_fMediumDetailDistance`: three times it, material detail drops to low.
    pub medium_detail: f32,
}

/// `DegradeInfo`: the distances of each `ModelDetail` setting (low, medium, high).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ac2DegradeInfo {
    pub levels: [Ac2DegradeDistances; 3],
}

impl Ac2DegradeInfo {
    /// The distances for `ModelDetail` setting `detail` (clamped to high).
    pub fn level(&self, detail: u32) -> &Ac2DegradeDistances {
        &self.levels[(detail as usize).min(2)]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ac2Setup {
    pub meshes: Arc<[DataId]>,
    pub bones: Arc<[Ac2Bone]>,
    pub placements: Arc<[Ac2PlacementFrame]>,
    /// Holding locations (weapons, held items).
    pub holding_points: Arc<[Ac2Location]>,
    /// Connection points where child part groups (heads, hair, attachments) attach.
    pub connection_points: Arc<[Ac2Location]>,
    /// `CSetup::degrade_info` (`DegradeInfo::UnPack`).
    pub degrade: Ac2DegradeInfo,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Ac2Setup {
    pub fn connection_point(&self, id: u32) -> Option<Ac2Location> {
        self.connection_points
            .iter()
            .copied()
            .find(|point| point.id == id)
    }

    /// Local (parent-relative) bone transforms of the default (first) placement frame.
    pub fn rest_locals(&self) -> Vec<Ac2Transform> {
        let local = self
            .placements
            .first()
            .map(|frame| &frame.bones[..])
            .unwrap_or(&[]);
        (0..self.bones.len())
            .map(|index| local.get(index).copied().unwrap_or(Ac2Transform::IDENTITY))
            .collect()
    }

    /// Model-space transform of every bone in the default (first) placement frame.
    pub fn rest_pose(&self) -> Vec<Ac2Transform> {
        self.world_pose(&self.rest_locals())
    }

    /// Model-space bone transforms for parent-relative `locals`.
    pub fn world_pose(&self, locals: &[Ac2Transform]) -> Vec<Ac2Transform> {
        let mut world: Vec<Ac2Transform> = Vec::with_capacity(self.bones.len());
        for (index, bone) in self.bones.iter().enumerate() {
            let own = locals.get(index).copied().unwrap_or(Ac2Transform::IDENTITY);
            let pose = match bone.parent.filter(|parent| *parent < index) {
                Some(parent) => world[parent].compose(&own),
                None => own,
            };
            world.push(pose);
        }
        world
    }
}

impl ResourcePackedDecode for Ac2Setup {
    const TYPE_NAME: &'static str = "AC2 Setup";
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
        let _flags = cursor.read_u32()?;
        let count = |cursor: &mut ArchiveCursor<'_>, item: usize| {
            let count = cursor.read_u32_count().ok()?;
            (count <= MAX_ITEMS && count * item <= cursor.remaining()).then_some(count)
        };
        let n = count(&mut cursor, 4).ok_or_else(|| malformed("mesh count"))?;
        let meshes = cursor
            .read_u32_items(n)?
            .into_iter()
            .map(DataId::new)
            .collect();
        let n = count(&mut cursor, 8).ok_or_else(|| malformed("bone count"))?;
        let bones = (0..n)
            .map(|_| {
                let parent = cursor.read_u32()?;
                Ok(Ac2Bone {
                    parent: (parent != u32::MAX).then_some(parent as usize),
                    name: cursor.read_u32()?,
                })
            })
            .collect::<ResourceResult<Arc<[_]>>>()?;
        // `PlacementFrame::UnPack`: the frame's kind (its `Placement` enum), the bone count, and a
        // `TransformBase` per bone.
        let n = count(&mut cursor, 8).ok_or_else(|| malformed("placement count"))?;
        let mut placements = Vec::with_capacity(n);
        for _ in 0..n {
            let kind = cursor.read_u32()?;
            let bone_count =
                count(&mut cursor, TRANSFORM_BYTES).ok_or_else(|| malformed("frame bones"))?;
            let frame = (0..bone_count)
                .map(|_| read_transform(&mut cursor))
                .collect::<ResourceResult<Arc<[_]>>>()?;
            placements.push(Ac2PlacementFrame { kind, bones: frame });
        }
        // Holding locations and connection points: id, part index, `TransformBase`.
        let mut locations = [Vec::new(), Vec::new()];
        for list in &mut locations {
            let n =
                count(&mut cursor, 8 + TRANSFORM_BYTES).ok_or_else(|| malformed("locations"))?;
            for _ in 0..n {
                let id = cursor.read_u32()?;
                let bone = cursor.read_u32()? as usize;
                list.push(Ac2Location {
                    id,
                    bone,
                    transform: read_transform(&mut cursor)?,
                });
            }
        }
        let [holding_points, connection_points] = locations.map(Arc::from);
        let n = count(&mut cursor, 20).ok_or_else(|| malformed("cylspheres"))?;
        cursor.read_bytes(n * 20)?;
        let n = count(&mut cursor, 16).ok_or_else(|| malformed("spheres"))?;
        cursor.read_bytes(n * 16)?;
        // Four physics words and two planes, then `DegradeInfo::UnPack`'s 18 distances.
        cursor.read_bytes(4 * 4 + 16 + 16)?;
        let degrade = read_degrade_info(&mut cursor)?;
        // A scalar and a word.
        cursor.read_bytes(4 + 4)?;
        let [x0, y0, z0, x1, y1, z1] = [(); 6].map(|_| cursor.read_f32());
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after Setup"));
        }
        Ok(Self {
            meshes,
            bones,
            placements: Arc::from(placements),
            holding_points,
            connection_points,
            degrade,
            bounds_min: Vec3::new(x0?, y0?, z0?),
            bounds_max: Vec3::new(x1?, y1?, z1?),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.meshes.len() * 4
            + self.bones.len() * 16
            + self
                .placements
                .iter()
                .map(|frame| frame.bones.len() * 40)
                .sum::<usize>()
    }
}

/// `DegradeInfo::UnPack`: per detail setting, visible, near, ideal, far, high and medium detail
/// distances.
fn read_degrade_info(cursor: &mut ArchiveCursor<'_>) -> ResourceResult<Ac2DegradeInfo> {
    let mut info = Ac2DegradeInfo::default();
    for level in &mut info.levels {
        let [visible, near, ideal, far, high_detail, medium_detail] =
            [(); 6].map(|_| cursor.read_f32());
        *level = Ac2DegradeDistances {
            visible: visible?,
            near: near?,
            ideal: ideal?,
            far: far?,
            high_detail: high_detail?,
            medium_detail: medium_detail?,
        };
    }
    Ok(info)
}

/// `TransformBase::UnPack`: offset, scale, then a `Quaternion` stored w, x, y, z.
fn read_transform(cursor: &mut ArchiveCursor<'_>) -> ResourceResult<Ac2Transform> {
    let offset = Vec3::new(cursor.read_f32()?, cursor.read_f32()?, cursor.read_f32()?);
    let scale = Vec3::new(cursor.read_f32()?, cursor.read_f32()?, cursor.read_f32()?);
    let w = cursor.read_f32()?;
    let rotation = Quat::from_xyzw(
        cursor.read_f32()?,
        cursor.read_f32()?,
        cursor.read_f32()?,
        w,
    );
    Ok(Ac2Transform {
        offset,
        scale,
        rotation: if rotation.length_squared() > 0.0 {
            rotation.normalize()
        } else {
            Quat::IDENTITY
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_pose_accumulates_parent_transforms() {
        let step = Ac2Transform {
            offset: Vec3::new(0.0, 0.0, 1.0),
            ..Ac2Transform::IDENTITY
        };
        let setup = Ac2Setup {
            meshes: Arc::from([]),
            bones: Arc::from([
                Ac2Bone {
                    parent: None,
                    name: 0,
                },
                Ac2Bone {
                    parent: Some(0),
                    name: 0,
                },
            ]),
            placements: Arc::from([Ac2PlacementFrame {
                kind: 0,
                bones: Arc::from([step, step]),
            }]),
            holding_points: Arc::from([]),
            connection_points: Arc::from([]),
            degrade: Ac2DegradeInfo::default(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        };
        let pose = setup.rest_pose();
        assert_eq!(pose[1].apply(Vec3::ZERO), Vec3::new(0.0, 0.0, 2.0));
    }
}

/// Every retail Setup's `DegradeInfo` decodes to finite distances with near <= ideal <= far,
/// and the visible distance does not shrink from low to high detail (only with `AC2_DAT_DIR`).
#[cfg(test)]
#[test]
fn retail_setups_decode_ordered_degrade_distances() {
    use ac2_dat::DatFileKind;
    use ac2_db::{ResourceStore, TypedResourceRef};

    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, 0x0400_0000, 0x04FF_FFFF)
        .collect();
    assert!(!ids.is_empty());
    for did in ids {
        let setup = store
            .load_cached_packed::<Ac2Setup>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .expect("Setup decodes")
            .expect("listed Setup exists");
        let levels = &setup.degrade.levels;
        for level in levels {
            let d = [level.visible, level.near, level.ideal, level.far];
            assert!(d.iter().all(|v| v.is_finite() && *v >= 0.0), "{did:08X}");
            assert!(level.visible > 0.0, "{did:08X}: {level:?}");
            assert!(
                level.near <= level.ideal && level.ideal <= level.far,
                "{did:08X}: {level:?}"
            );
        }
        assert!(
            levels[0].visible <= levels[1].visible && levels[1].visible <= levels[2].visible,
            "{did:08X}: {levels:?}"
        );
    }
}
