//! Asheron's Call 2 `CDataMesh` (`0x06`), decoded in `CBaseMesh::StreamPack` order: mesh type,
//! `MaterialInstance` table, `CVertexArray`, bounding sphere, then `CMeshTriList` degrades whose
//! fragments carry u16 index buffers. Physics polygons and BSP that follow are not retained.
//!
//! `CBaseMesh::StreamPack` (Legions 0x004C6610) runs `PSmartArray<MeshMaterials::MaterialAndDataID>
//! ::StreamPack` (material DID and type per slot), `STREAMPACK_PACKOBJ` of the `CVertexArray` (its
//! bounding box is [`Ac2Mesh::bounds_min`] / `bounds_max`: `CBaseMesh::BuildBoundingBox`) and of
//! each `CMeshTriList`; the mesh is found by DID through the resource store
//! (`CDataMesh::Get`, `CMeshGroup::GetMesh`). It has no morph target or vertex set data.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::{Vec2, Vec3};

use crate::DecodedMeshVertex;

const MAX_VERTICES: usize = 1 << 20;
const MAX_ITEMS: usize = 1 << 16;

/// One indexed triangle list drawn with one of the mesh's materials.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2MeshFragment {
    pub material_index: usize,
    /// Position of the owning `CMeshTriList` in the degrade list: each degrade is a complete level
    /// of detail, highest first.
    pub degrade: usize,
    /// `CMeshTriList::m_DetailLevels`: bitmask of the detail settings (low 1, medium 2, high 4)
    /// that may draw this degrade.
    pub detail_levels: u32,
    pub indices: Arc<[u32]>,
}

/// Decoded AC2 mesh in its own model space.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2Mesh {
    /// `CGfxObj::mesh_type`: the category appearance atoms select meshes by.
    pub mesh_type: u32,
    pub materials: Arc<[DataId]>,
    /// The `CVertexArray` format word (`VertexFormatInfo`).
    pub vertex_format: u32,
    pub vertices: Arc<[DecodedMeshVertex]>,
    /// `D3DFVF_DIFFUSE` colour per vertex (a `D3DCOLOR`, `0xAARRGGBB`) when the format has one.
    pub colors: Option<Arc<[u32]>>,
    /// Texture coordinate sets after the first (which is `DecodedMeshVertex::uv`), in vertex
    /// format order.
    pub extra_uvs: Arc<[Arc<[Vec2]>]>,
    /// Bump-mapping tangents S and T (`0x1000_0000`, `0x2000_0000`) per vertex when the format
    /// has both; the vertex programs read them as `v3` and `v4`.
    pub tangents: Option<Arc<[[Vec3; 2]]>>,
    /// Up to four (bone, weight) influences per vertex when the vertex format carries matrices.
    pub skin: Option<Arc<[[(u8, f32); 4]]>>,
    pub fragments: Arc<[Ac2MeshFragment]>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Ac2Mesh {
    /// The degrade drawn at the high detail setting: the first one admitting it.
    pub fn highest_detail_degrade(&self) -> usize {
        self.fragments
            .iter()
            .find(|fragment| fragment.detail_levels & HIGH_DETAIL_SETTING != 0)
            .or_else(|| self.fragments.first())
            .map_or(0, |fragment| fragment.degrade)
    }
}

/// `m_DetailLevels` bit of the high detail setting.
const HIGH_DETAIL_SETTING: u32 = 4;

impl ResourcePackedDecode for Ac2Mesh {
    const TYPE_NAME: &'static str = "AC2 DataMesh";
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
        let mesh_type = cursor.read_u32()?;
        let material_count = count(&mut cursor, 4).ok_or_else(|| malformed("material count"))?;
        let materials = cursor
            .read_u32_items(material_count)?
            .into_iter()
            .map(DataId::new)
            .collect::<Arc<[_]>>();
        let vertex_format = cursor.read_u32()?;
        let format = VertexFormat::new(vertex_format);
        let vertex_count = cursor.read_u32_count()?;
        if vertex_count > MAX_VERTICES {
            return Err(malformed("vertex count"));
        }
        let raw = cursor.read_sized_slice(vertex_count, format.size, "AC2 vertex array")?;
        cursor.align_to(4)?;
        let mut vertices = Vec::with_capacity(vertex_count);
        let mut skin = format.matrices.then(|| Vec::with_capacity(vertex_count));
        let mut colors = format.diffuse.map(|_| Vec::with_capacity(vertex_count));
        let mut tangents = format.tangents.map(|_| Vec::with_capacity(vertex_count));
        let mut extra_uvs =
            vec![Vec::with_capacity(vertex_count); format.uv_sets.saturating_sub(1)];
        for texel in raw.chunks_exact(format.size) {
            vertices.push(format.vertex(texel));
            if let (Some(colors), Some(offset)) = (colors.as_mut(), format.diffuse) {
                colors.push(read_u32(texel, offset));
            }
            if let (Some(tangents), Some((s, t))) = (tangents.as_mut(), format.tangents) {
                let vector = |o: usize| {
                    Vec3::new(
                        read_f32(texel, o),
                        read_f32(texel, o + 4),
                        read_f32(texel, o + 8),
                    )
                };
                tangents.push([vector(s), vector(t)]);
            }
            for (set, uvs) in extra_uvs.iter_mut().enumerate() {
                uvs.push(format.uv(texel, set + 1));
            }
            if let Some(skin) = skin.as_mut() {
                skin.push(format.influences(texel));
            }
        }
        let bounds = [(); 6].map(|_| cursor.read_f32());
        let [x0, y0, z0, x1, y1, z1] = bounds;
        let _sphere = [(); 4].map(|_| cursor.read_f32());
        let degrades = count(&mut cursor, 12).ok_or_else(|| malformed("degrade count"))?;
        let mut fragments = Vec::new();
        for degrade in 0..degrades {
            let _degrade_type = cursor.read_u32()?;
            let detail_levels = cursor.read_u32()?;
            let lists = count(&mut cursor, 8).ok_or_else(|| malformed("fragment count"))?;
            for _ in 0..lists {
                let material_index = cursor.read_u32_count()?;
                let index_count = cursor.read_u32_count()?;
                let indices = cursor.read_sized_slice(index_count, 2, "AC2 index buffer")?;
                cursor.align_to(4)?;
                let indices = indices
                    .chunks_exact(2)
                    .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
                    .collect::<Arc<[_]>>();
                if material_index >= materials.len().max(1)
                    || indices.iter().any(|index| *index as usize >= vertex_count)
                {
                    return Err(malformed("fragment indexed outside the mesh"));
                }
                fragments.push(Ac2MeshFragment {
                    material_index,
                    degrade,
                    detail_levels,
                    indices,
                });
            }
        }
        Ok(Self {
            mesh_type,
            materials,
            vertex_format,
            vertices: Arc::from(vertices),
            colors: colors.map(Arc::from),
            tangents: tangents.map(Arc::from),
            extra_uvs: extra_uvs.into_iter().map(Arc::from).collect(),
            skin: skin.map(Arc::from),
            fragments: Arc::from(fragments),
            bounds_min: Vec3::new(x0?, y0?, z0?),
            bounds_max: Vec3::new(x1?, y1?, z1?),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.vertices.len() * std::mem::size_of::<DecodedMeshVertex>()
            + self.skin.as_ref().map_or(0, |skin| skin.len() * 32)
            + self
                .fragments
                .iter()
                .map(|fragment| fragment.indices.len() * 4)
                .sum::<usize>()
    }
}

fn count(cursor: &mut ArchiveCursor<'_>, min_item_bytes: usize) -> Option<usize> {
    let count = cursor.read_u32_count().ok()?;
    (count <= MAX_ITEMS && count * min_item_bytes <= cursor.remaining()).then_some(count)
}

/// `VertexFormatInfo::GenerateOffsets`: D3D FVF fields plus AC2's tangent (`0x1000_0000`,
/// `0x2000_0000`), point-to-light (`0x4000_0000`) and matrix-palette (byte 2) extensions.
#[derive(Debug, Clone, Copy)]
struct VertexFormat {
    size: usize,
    normal: Option<usize>,
    diffuse: Option<usize>,
    tangents: Option<(usize, usize)>,
    uv: Option<usize>,
    uv_sets: usize,
    matrices: bool,
    matrix_indices: usize,
    matrix_weights: usize,
    matrix_count: usize,
}

impl VertexFormat {
    fn new(format: u32) -> Self {
        let weights = match format & 0xE {
            6 => 1,
            8 => 2,
            10 => 3,
            12 => 4,
            14 => 5,
            _ => 0,
        };
        let tc_pairs = ((format >> 8) & 0xF).min(8) as usize;
        let matrix_count = ((format >> 16) & 0xFF) as usize;
        let mut offset = if format & 0xE != 0 { 12 } else { 0 } + weights * 4;
        let normal = (format & 0x10 != 0).then_some(offset);
        offset += if format & 0x10 != 0 { 12 } else { 0 };
        offset += if format & 0x20 != 0 { 4 } else { 0 };
        let diffuse = (format & 0x40 != 0).then_some(offset);
        for bit in [0x40, 0x80] {
            offset += if format & bit != 0 { 4 } else { 0 };
        }
        let tangent_s = (format & 0x1000_0000 != 0).then_some(offset);
        offset += if format & 0x1000_0000 != 0 { 12 } else { 0 };
        let tangent_t = (format & 0x2000_0000 != 0).then_some(offset);
        offset += if format & 0x2000_0000 != 0 { 12 } else { 0 };
        offset += if format & 0x4000_0000 != 0 { 12 } else { 0 };
        let uv = (tc_pairs > 0).then_some(offset);
        offset += tc_pairs * 8;
        let matrix_indices = offset;
        offset += matrix_count;
        let matrix_weights = offset;
        offset += matrix_count * 4;
        if format & 0x0100_0000 != 0 {
            offset = offset.next_multiple_of(0x20);
        }
        Self {
            size: offset.max(1),
            normal,
            diffuse,
            tangents: tangent_s.zip(tangent_t),
            uv,
            uv_sets: tc_pairs,
            matrices: matrix_count > 0,
            matrix_indices,
            matrix_weights,
            matrix_count: matrix_count.min(4),
        }
    }

    fn vertex(&self, texel: &[u8]) -> DecodedMeshVertex {
        let f = |offset: usize| {
            texel
                .get(offset..offset + 4)
                .map_or(0.0, |b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        };
        DecodedMeshVertex {
            position: Vec3::new(f(0), f(4), f(8)),
            normal: self
                .normal
                .map_or(Vec3::Z, |o| Vec3::new(f(o), f(o + 4), f(o + 8))),
            uv: self.uv.map_or(Vec2::ZERO, |o| Vec2::new(f(o), f(o + 4))),
        }
    }

    fn uv(&self, texel: &[u8], set: usize) -> Vec2 {
        self.uv.map_or(Vec2::ZERO, |o| {
            let o = o + set * 8;
            Vec2::new(read_f32(texel, o), read_f32(texel, o + 4))
        })
    }

    fn influences(&self, texel: &[u8]) -> [(u8, f32); 4] {
        let mut out = [(0_u8, 0.0_f32); 4];
        for (slot, influence) in out.iter_mut().enumerate().take(self.matrix_count) {
            let weight = texel
                .get(self.matrix_weights + slot * 4..self.matrix_weights + slot * 4 + 4)
                .map_or(0.0, |b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
            *influence = (
                texel.get(self.matrix_indices + slot).copied().unwrap_or(0),
                weight,
            );
        }
        out
    }
}

fn read_u32(texel: &[u8], offset: usize) -> u32 {
    texel
        .get(offset..offset + 4)
        .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn read_f32(texel: &[u8], offset: usize) -> f32 {
    f32::from_bits(read_u32(texel, offset))
}

#[cfg(test)]
mod tests {
    use super::VertexFormat;

    #[test]
    fn legions_vertex_formats_have_their_recovered_strides() {
        assert_eq!(VertexFormat::new(0x0000_02D2).size, 48);
        let lit = VertexFormat::new(0x0000_02D2);
        assert_eq!((lit.diffuse, lit.uv, lit.uv_sets), (Some(24), Some(32), 2));
        let bumped = VertexFormat::new(0x3000_02D2);
        assert_eq!((bumped.tangents, bumped.uv), (Some((32, 44)), Some(56)));
        assert_eq!(VertexFormat::new(0x3000_02D2).size, 72);
        assert_eq!(VertexFormat::new(0x3004_02D2).size, 92);
        assert_eq!(VertexFormat::new(0x3003_02D2).size, 87);
        let skinned = VertexFormat::new(0x3004_02D2);
        assert!(skinned.matrices);
        assert_eq!(skinned.uv, Some(56));
    }
}
