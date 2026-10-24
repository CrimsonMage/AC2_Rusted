//! Asheron's Call 2 `CDataMesh` (`0x06`) collision data.
//!
//! `CDataMesh::StreamPack` runs `CStaticMesh::StreamPack`, which first runs `CBaseMesh::StreamPack`: mesh
//! type, materials, `CVertexArray`, bounding box and sphere, degrades, then the render
//! `SmartArray<CPolygon>` and, when it is non-empty, its `BSPTREE` packed as `PHYSICS_BSP`.
//! `CStaticMesh` then appends `physics_polygons`, a has-tree word and the physics `BSPTREE`.
//! Retail collides static meshes against that last pair; this decoder keeps only it and the
//! vertex positions its polygons reference (`CPolygon::pack_verts` is the render vertex array).
//!
//! `CPolygon::UnPack`: u16 id, u16 count, u16 vertex indices, 4-byte align;
//! the plane is computed from the vertices. `BSPNODE::UnPackChild`: u32 tag,
//! `'LEAF'` reads `BSPLEAF::UnPack` (solid, sphere, counted u16 polygon
//! slots, align), every other tag `BSPNODE::UnPack` (plane, children named by
//! the tag, then the node sphere).

use std::sync::Arc;

use ac2_dat::ArchiveCursor;
use ac2_core::QualifiedDataId;
use ac2_math::{Plane, Sphere, Vec3};
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

const MAX_VERTICES: usize = 1 << 20;
const MAX_ITEMS: usize = 1 << 16;
/// Retail recursion is bounded by the authored tree; this bounds a malformed one.
const MAX_BSP_DEPTH: usize = 256;

const TAG_LEAF: u32 = 0x4C45_4146; // 'LEAF'
const TAG_BOTH: u32 = 0x4250_2D4E; // 'BP-N'
const TAG_NEG: u32 = 0x422D_2D4E; // 'B--N'
const TAG_POS: u32 = 0x4250_2D2D; // 'BP--'
const TAG_NONE: u32 = 0x4250_464C; // 'BPFL'

/// One collision `CPolygon`: its authored id and indices into [`Ac2PhysicsMesh::positions`].
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2PhysicsPolygon {
    polygon_id: u16,
    vertex_ids: Box<[u16]>,
}

impl Ac2PhysicsPolygon {
    /// A physics polygon from another AC2 source (a `CCellStruct` cell mesh).
    pub fn new(polygon_id: u16, vertex_ids: Box<[u16]>) -> Self {
        Self {
            polygon_id,
            vertex_ids,
        }
    }

    pub fn polygon_id(&self) -> u16 {
        self.polygon_id
    }

    pub fn vertex_ids(&self) -> &[u16] {
        &self.vertex_ids
    }
}

/// One node of the AC2 physics `BSPTREE`. Leaves carry their `solid` flag, sphere and polygon
/// slots (indices into the physics polygon array); internal nodes their splitting plane and
/// sphere.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2PhysicsBspNode {
    leaf: bool,
    splitting_plane: Plane,
    pos_node: Option<Box<Ac2PhysicsBspNode>>,
    neg_node: Option<Box<Ac2PhysicsBspNode>>,
    solid: i32,
    bounding_sphere: Sphere,
    polygon_slots: Box<[u16]>,
}

impl Ac2PhysicsBspNode {
    /// A physics `BSPNODE` from another AC2 source (a `CCellStruct` physics tree, the same
    /// `BSPTREE` layout `CEnvCell::find_env_collisions` traverses).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        leaf: bool,
        splitting_plane: Plane,
        pos_node: Option<Box<Self>>,
        neg_node: Option<Box<Self>>,
        solid: i32,
        bounding_sphere: Sphere,
        polygon_slots: Box<[u16]>,
    ) -> Self {
        Self {
            leaf,
            splitting_plane,
            pos_node,
            neg_node,
            solid,
            bounding_sphere,
            polygon_slots,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.leaf
    }

    pub fn splitting_plane(&self) -> Plane {
        self.splitting_plane
    }

    pub fn pos_node(&self) -> Option<&Self> {
        self.pos_node.as_deref()
    }

    pub fn neg_node(&self) -> Option<&Self> {
        self.neg_node.as_deref()
    }

    pub fn solid(&self) -> i32 {
        self.solid
    }

    pub fn bounding_sphere(&self) -> Sphere {
        self.bounding_sphere
    }

    pub fn polygon_slots(&self) -> &[u16] {
        &self.polygon_slots
    }

    fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.polygon_slots.len() * 2
            + self.pos_node.as_deref().map_or(0, Self::retained_bytes)
            + self.neg_node.as_deref().map_or(0, Self::retained_bytes)
    }
}

/// The collision part of an AC2 mesh, in mesh space (AC axes).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2PhysicsMesh {
    skinned: bool,
    positions: Arc<[Vec3]>,
    polygons: Arc<[Ac2PhysicsPolygon]>,
    bsp: Option<Arc<Ac2PhysicsBspNode>>,
}

impl Ac2PhysicsMesh {
    /// The collision of an AC2 cell mesh (`CCellStruct`): its positions, physics polygons and
    /// physics tree, in cell space.
    pub fn from_cell_parts(
        positions: Arc<[Vec3]>,
        polygons: Arc<[Ac2PhysicsPolygon]>,
        bsp: Ac2PhysicsBspNode,
    ) -> Self {
        Self {
            skinned: false,
            positions,
            polygons,
            bsp: Some(Arc::new(bsp)),
        }
    }

    /// Whether the vertex format carries a matrix palette: the mesh is authored in Setup space
    /// rather than riding one bone.
    pub fn is_skinned(&self) -> bool {
        self.skinned
    }

    /// Every vertex position of the mesh's vertex array.
    pub fn positions(&self) -> &[Vec3] {
        &self.positions
    }

    pub fn polygons(&self) -> &[Ac2PhysicsPolygon] {
        &self.polygons
    }

    /// Root of the physics tree; `None` when the mesh does not collide.
    pub fn bsp(&self) -> Option<&Ac2PhysicsBspNode> {
        self.bsp.as_deref()
    }
}

impl ResourcePackedDecode for Ac2PhysicsMesh {
    const TYPE_NAME: &'static str = "AC2 DataMesh physics";
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
        let _mesh_type = cursor.read_u32()?;
        let materials = count(&mut cursor, 4).ok_or_else(|| malformed("material count"))?;
        cursor.read_bytes(materials * 4)?;
        let format = cursor.read_u32()?;
        let stride = vertex_stride(format);
        let vertex_count = cursor.read_u32_count()?;
        if vertex_count > MAX_VERTICES {
            return Err(malformed("vertex count"));
        }
        let raw = cursor.read_sized_slice(vertex_count, stride, "AC2 vertex array")?;
        cursor.align_to(4)?;
        let positions = raw
            .chunks_exact(stride)
            .map(|vertex| {
                let f = |at: usize| {
                    f32::from_le_bytes([vertex[at], vertex[at + 1], vertex[at + 2], vertex[at + 3]])
                };
                Vec3::new(f(0), f(4), f(8))
            })
            .collect::<Arc<[_]>>();
        // Bounding box (6 floats) and bounding sphere (4 floats).
        cursor.read_bytes(40)?;
        let degrades = count(&mut cursor, 12).ok_or_else(|| malformed("degrade count"))?;
        for _ in 0..degrades {
            let _degrade_type = cursor.read_u32()?;
            let _detail_levels = cursor.read_u32()?;
            let lists = count(&mut cursor, 8).ok_or_else(|| malformed("fragment count"))?;
            for _ in 0..lists {
                let _material = cursor.read_u32()?;
                let indices = cursor.read_u32_count()?;
                cursor.read_sized_slice(indices, 2, "AC2 index buffer")?;
                cursor.align_to(4)?;
            }
        }
        // `CBaseMesh` render polygons and their tree: parsed to reach the physics data.
        let render_polygons =
            read_polygons(&mut cursor, vertex_count).ok_or_else(|| malformed("render polygons"))?;
        if !render_polygons.is_empty() {
            read_node(&mut cursor, render_polygons.len(), 0)
                .ok_or_else(|| malformed("render BSP"))?;
        }
        let polygons = read_polygons(&mut cursor, vertex_count)
            .ok_or_else(|| malformed("physics polygons"))?;
        let bsp = match cursor.read_u32()? {
            0 => None,
            _ => Some(Arc::new(
                read_node(&mut cursor, polygons.len(), 0)
                    .ok_or_else(|| malformed("physics BSP"))?,
            )),
        };
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after the physics BSP"));
        }
        Ok(Self {
            skinned: (format >> 16) & 0xFF != 0,
            positions,
            polygons: Arc::from(polygons),
            bsp,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.positions.len() * std::mem::size_of::<Vec3>()
            + self
                .polygons
                .iter()
                .map(|polygon| {
                    std::mem::size_of::<Ac2PhysicsPolygon>() + polygon.vertex_ids.len() * 2
                })
                .sum::<usize>()
            + self
                .bsp
                .as_deref()
                .map_or(0, Ac2PhysicsBspNode::retained_bytes)
    }
}

fn count(cursor: &mut ArchiveCursor<'_>, min_item_bytes: usize) -> Option<usize> {
    let count = cursor.read_u32_count().ok()?;
    (count <= MAX_ITEMS && count * min_item_bytes <= cursor.remaining()).then_some(count)
}

/// `PSmartArray<CPolygon>::StreamPack` over `CPolygon::UnPack`.
fn read_polygons(
    cursor: &mut ArchiveCursor<'_>,
    vertex_count: usize,
) -> Option<Vec<Ac2PhysicsPolygon>> {
    let polygons = count(cursor, 4)?;
    let mut out = Vec::with_capacity(polygons);
    for _ in 0..polygons {
        let polygon_id = cursor.read_u16().ok()?;
        let points = usize::from(cursor.read_u16().ok()?);
        let vertex_ids = cursor.read_u16_items(points).ok()?.into_boxed_slice();
        cursor.align_to(4).ok()?;
        if vertex_ids.iter().any(|id| usize::from(*id) >= vertex_count) {
            return None;
        }
        out.push(Ac2PhysicsPolygon {
            polygon_id,
            vertex_ids,
        });
    }
    Some(out)
}

/// `BSPNODE::UnPackChild` for a `PHYSICS_BSP` tree.
fn read_node(
    cursor: &mut ArchiveCursor<'_>,
    polygon_count: usize,
    depth: usize,
) -> Option<Ac2PhysicsBspNode> {
    if depth > MAX_BSP_DEPTH {
        return None;
    }
    let tag = cursor.read_u32().ok()?;
    if tag == TAG_LEAF {
        let solid = cursor.read_i32().ok()?;
        let bounding_sphere = read_sphere(cursor)?;
        let slots = cursor.read_u32_count().ok()?;
        if slots > MAX_ITEMS {
            return None;
        }
        let polygon_slots = cursor.read_u16_items(slots).ok()?.into_boxed_slice();
        cursor.align_to(4).ok()?;
        if polygon_slots
            .iter()
            .any(|slot| usize::from(*slot) >= polygon_count)
        {
            return None;
        }
        return Some(Ac2PhysicsBspNode {
            leaf: true,
            splitting_plane: Plane::default(),
            pos_node: None,
            neg_node: None,
            solid,
            bounding_sphere,
            polygon_slots,
        });
    }
    let normal = read_vec3(cursor)?;
    let splitting_plane = Plane {
        normal,
        d: cursor.read_f32().ok()?,
    };
    let (pos, neg) = match tag {
        TAG_BOTH => (true, true),
        TAG_POS => (true, false),
        TAG_NEG => (false, true),
        TAG_NONE => (false, false),
        _ => return None,
    };
    let pos_node = match pos {
        true => Some(Box::new(read_node(cursor, polygon_count, depth + 1)?)),
        false => None,
    };
    let neg_node = match neg {
        true => Some(Box::new(read_node(cursor, polygon_count, depth + 1)?)),
        false => None,
    };
    Some(Ac2PhysicsBspNode {
        leaf: false,
        splitting_plane,
        pos_node,
        neg_node,
        solid: 0,
        bounding_sphere: read_sphere(cursor)?,
        polygon_slots: Box::default(),
    })
}

fn read_vec3(cursor: &mut ArchiveCursor<'_>) -> Option<Vec3> {
    Some(Vec3::new(
        cursor.read_f32().ok()?,
        cursor.read_f32().ok()?,
        cursor.read_f32().ok()?,
    ))
}

fn read_sphere(cursor: &mut ArchiveCursor<'_>) -> Option<Sphere> {
    let origin = read_vec3(cursor)?;
    Some(Sphere {
        origin,
        radius: cursor.read_f32().ok()?,
    })
}

/// `VertexFormatInfo::GenerateOffsets` stride (the same table `ac2_mesh` decodes vertices with):
/// position first, then D3D FVF weights, normal, colours, AC2 tangent/point-to-light vectors,
/// texture coordinates and the matrix palette, padded to 32 bytes when flagged.
fn vertex_stride(format: u32) -> usize {
    let weights = match format & 0xE {
        6 => 1,
        8 => 2,
        10 => 3,
        12 => 4,
        14 => 5,
        _ => 0,
    };
    let mut size = if format & 0xE != 0 { 12 } else { 0 } + weights * 4;
    size += if format & 0x10 != 0 { 12 } else { 0 };
    size += [0x20, 0x40, 0x80]
        .iter()
        .filter(|bit| format & *bit != 0)
        .count()
        * 4;
    size += [0x1000_0000, 0x2000_0000, 0x4000_0000]
        .iter()
        .filter(|bit| format & *bit != 0)
        .count()
        * 12;
    let matrices = ((format >> 16) & 0xFF) as usize;
    size += ((format >> 8) & 0xF).min(8) as usize * 8 + matrices * 5;
    if format & 0x0100_0000 != 0 {
        size = size.next_multiple_of(0x20);
    }
    size.max(1)
}

mod setup;
pub use setup::Ac2SetupPhysics;

#[cfg(test)]
mod tests;
