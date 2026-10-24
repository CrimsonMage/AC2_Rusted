//! Asheron's Call 2 `CCellMesh` (`0x01`) static structure: the collision and portal geometry of
//! an env cell, decoded in `CCellMesh::StreamPack` order.
//!
//! A cell mesh is a `CStaticMesh` (`CBaseMesh` render data, render polygons and tree, then the
//! physics polygons and `PHYSICS_BSP` tree) followed by the cell's
//! `CELL_BSP` tree, its counted `CPortalPoly`s (`CPortalPoly::StreamPack`:
//! u16 side, u16 physics-polygon index, bounding sphere) and the `m_cellMeshInfoDID` word.
//! `BSPNODE::UnPackChild`/`UnPack`: a cell tree reads a 'LEAF' tag as a
//! plain node, every node reads its plane, and only physics trees append a bounding sphere.
//! The render vertices and fragments of the same record are decoded by render-codec's `Ac2Mesh`.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::{Plane, Sphere, Vec3};

const MAX_VERTICES: usize = 1 << 20;
const MAX_ITEMS: usize = 1 << 16;
const MAX_BSP_DEPTH: usize = 256;
/// `BSPNODE::point_inside_cell_bsp` tolerance.
const CELL_BSP_EPSILON: f32 = 0.0002;

const TAG_LEAF: u32 = 0x4C45_4146; // 'LEAF'
const TAG_BOTH: u32 = 0x4250_2D4E; // 'BP-N'
const TAG_NEG: u32 = 0x422D_2D4E; // 'B--N'
const TAG_POS: u32 = 0x4250_2D2D; // 'BP--'
const TAG_NONE: u32 = 0x4250_464C; // 'BPFL'

/// One physics `CPolygon` of a cell, as indices into [`Ac2CellStruct::positions`].
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2CellPolygon {
    pub polygon_id: u16,
    pub vertex_ids: Box<[u16]>,
}

/// One `BSPNODE` of a cell's physics or cell tree. Physics leaves carry `solid`, their sphere and
/// polygon slots; cell-tree nodes carry only their plane.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2CellBspNode {
    pub leaf: bool,
    pub plane: Plane,
    pub pos: Option<Box<Ac2CellBspNode>>,
    pub neg: Option<Box<Ac2CellBspNode>>,
    pub solid: i32,
    pub sphere: Sphere,
    pub polygon_slots: Box<[u16]>,
}

impl Ac2CellBspNode {
    fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.polygon_slots.len() * 2
            + self.pos.as_deref().map_or(0, Self::retained_bytes)
            + self.neg.as_deref().map_or(0, Self::retained_bytes)
    }
}

/// One `CPortalPoly`: the physics polygon spanning a portal opening.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2CellPortalPolygon {
    pub side: u16,
    /// Index into [`Ac2CellStruct::polygons`].
    pub polygon: u16,
    pub sphere: Sphere,
}

/// The static structure of an AC2 cell mesh in cell-local space (AC axes).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2CellStruct {
    pub positions: Arc<[Vec3]>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub polygons: Arc<[Ac2CellPolygon]>,
    pub physics_bsp: Option<Arc<Ac2CellBspNode>>,
    pub cell_bsp: Option<Arc<Ac2CellBspNode>>,
    pub portal_polygons: Arc<[Ac2CellPortalPolygon]>,
    pub mesh_info: Option<DataId>,
}

impl Ac2CellStruct {
    /// `CCellMesh::PointInCell` -> `BSPNODE::point_inside_cell_bsp` for a cell-local point.
    pub fn point_in_cell(&self, point: Vec3) -> bool {
        let Some(mut node) = self.cell_bsp.as_deref() else {
            return false;
        };
        loop {
            while node.plane.normal.dot(point) + node.plane.d < -CELL_BSP_EPSILON {
                match node.neg.as_deref() {
                    Some(neg) => node = neg,
                    None => return false,
                }
            }
            match node.pos.as_deref() {
                Some(pos) => node = pos,
                None => return true,
            }
        }
    }
}

impl ResourcePackedDecode for Ac2CellStruct {
    const TYPE_NAME: &'static str = "AC2 CellMesh structure";
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
        let decoded = decode(&mut cursor, qdid.data_id().raw()).map_err(malformed)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after CellMesh"));
        }
        Ok(decoded)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.positions.len() * std::mem::size_of::<Vec3>()
            + self
                .polygons
                .iter()
                .map(|polygon| std::mem::size_of::<Ac2CellPolygon>() + polygon.vertex_ids.len() * 2)
                .sum::<usize>()
            + self
                .physics_bsp
                .as_deref()
                .map_or(0, Ac2CellBspNode::retained_bytes)
            + self
                .cell_bsp
                .as_deref()
                .map_or(0, Ac2CellBspNode::retained_bytes)
            + self.portal_polygons.len() * std::mem::size_of::<Ac2CellPortalPolygon>()
    }
}

type Read<T> = Result<T, &'static str>;

fn word(cursor: &mut ArchiveCursor<'_>) -> Read<u32> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn float(cursor: &mut ArchiveCursor<'_>) -> Read<f32> {
    cursor.read_f32().map_err(|_| "record ended early")
}

fn vec3(cursor: &mut ArchiveCursor<'_>) -> Read<Vec3> {
    Ok(Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?))
}

fn sphere(cursor: &mut ArchiveCursor<'_>) -> Read<Sphere> {
    Ok(Sphere {
        origin: vec3(cursor)?,
        radius: float(cursor)?,
    })
}

fn skip(cursor: &mut ArchiveCursor<'_>, len: usize) -> Read<()> {
    cursor
        .read_bytes(len)
        .map(|_| ())
        .map_err(|_| "record ended early")
}

fn align(cursor: &mut ArchiveCursor<'_>) -> Read<()> {
    cursor.align_to(4).map_err(|_| "record ended early")
}

fn count(cursor: &mut ArchiveCursor<'_>, min_item_bytes: usize) -> Read<usize> {
    let n = word(cursor)? as usize;
    if n > MAX_ITEMS || n * min_item_bytes > cursor.remaining() {
        return Err("item count exceeded the record");
    }
    Ok(n)
}

fn decode(cursor: &mut ArchiveCursor<'_>, did: u32) -> Read<Ac2CellStruct> {
    if word(cursor)? != did {
        return Err("embedded DID did not match DAT identity");
    }
    let _mesh_type = word(cursor)?;
    let materials = count(cursor, 4)?;
    skip(cursor, materials * 4)?;
    let format = word(cursor)?;
    let stride = vertex_stride(format);
    let vertex_count = word(cursor)? as usize;
    if vertex_count > MAX_VERTICES || vertex_count * stride > cursor.remaining() {
        return Err("vertex count exceeded the record");
    }
    let raw = cursor
        .read_bytes(vertex_count * stride)
        .map_err(|_| "record ended early")?;
    let positions = raw
        .chunks_exact(stride)
        .map(|vertex| {
            let f = |at: usize| {
                f32::from_le_bytes([vertex[at], vertex[at + 1], vertex[at + 2], vertex[at + 3]])
            };
            Vec3::new(f(0), f(4), f(8))
        })
        .collect::<Arc<[_]>>();
    align(cursor)?;
    let bounds_min = vec3(cursor)?;
    let bounds_max = vec3(cursor)?;
    let _bounding_sphere = sphere(cursor)?;
    for _ in 0..count(cursor, 12)? {
        let _degrade_type = word(cursor)?;
        let _detail_levels = word(cursor)?;
        for _ in 0..count(cursor, 8)? {
            let _material = word(cursor)?;
            let indices = word(cursor)? as usize;
            skip(cursor, indices * 2)?;
            align(cursor)?;
        }
    }
    let render_polygons = read_polygons(cursor, vertex_count)?;
    if !render_polygons.is_empty() {
        read_node(cursor, TreeKind::Physics, render_polygons.len(), 0)?;
    }
    let polygons = read_polygons(cursor, vertex_count)?;
    let physics_bsp = match word(cursor)? {
        0 => None,
        _ => Some(Arc::new(read_node(
            cursor,
            TreeKind::Physics,
            polygons.len(),
            0,
        )?)),
    };
    let cell_bsp = Some(Arc::new(read_node(cursor, TreeKind::Cell, 0, 0)?));
    let portal_count = count(cursor, 20)?;
    let mut portal_polygons = Vec::with_capacity(portal_count);
    for _ in 0..portal_count {
        let side = cursor.read_u16().map_err(|_| "record ended early")?;
        let polygon = cursor.read_u16().map_err(|_| "record ended early")?;
        if usize::from(polygon) >= polygons.len() {
            return Err("portal polygon indexed outside the physics polygons");
        }
        portal_polygons.push(Ac2CellPortalPolygon {
            side,
            polygon,
            sphere: sphere(cursor)?,
        });
    }
    let mesh_info = Some(DataId::new(word(cursor)?)).filter(|did| !did.is_null());
    Ok(Ac2CellStruct {
        positions,
        bounds_min,
        bounds_max,
        polygons: Arc::from(polygons),
        physics_bsp,
        cell_bsp,
        portal_polygons: Arc::from(portal_polygons),
        mesh_info,
    })
}

/// `PSmartArray<CPolygon>` over `CPolygon::UnPack`.
fn read_polygons(cursor: &mut ArchiveCursor<'_>, vertex_count: usize) -> Read<Vec<Ac2CellPolygon>> {
    let n = count(cursor, 4)?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let polygon_id = cursor.read_u16().map_err(|_| "record ended early")?;
        let points = usize::from(cursor.read_u16().map_err(|_| "record ended early")?);
        let vertex_ids = cursor
            .read_u16_items(points)
            .map_err(|_| "record ended early")?
            .into_boxed_slice();
        align(cursor)?;
        if vertex_ids.iter().any(|id| usize::from(*id) >= vertex_count) {
            return Err("polygon indexed outside the vertex array");
        }
        out.push(Ac2CellPolygon {
            polygon_id,
            vertex_ids,
        });
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeKind {
    Physics,
    Cell,
}

fn read_node(
    cursor: &mut ArchiveCursor<'_>,
    kind: TreeKind,
    polygons: usize,
    depth: usize,
) -> Read<Ac2CellBspNode> {
    if depth > MAX_BSP_DEPTH {
        return Err("BSP tree exceeded the depth bound");
    }
    let tag = word(cursor)?;
    if tag == TAG_LEAF && kind == TreeKind::Physics {
        let solid = cursor.read_i32().map_err(|_| "record ended early")?;
        let sphere = sphere(cursor)?;
        let slots = word(cursor)? as usize;
        if slots > MAX_ITEMS {
            return Err("BSP leaf slot count exceeded the bound");
        }
        let polygon_slots = cursor
            .read_u16_items(slots)
            .map_err(|_| "record ended early")?
            .into_boxed_slice();
        align(cursor)?;
        if polygon_slots
            .iter()
            .any(|slot| usize::from(*slot) >= polygons)
        {
            return Err("BSP leaf indexed outside the polygons");
        }
        return Ok(Ac2CellBspNode {
            leaf: true,
            plane: Plane::default(),
            pos: None,
            neg: None,
            solid,
            sphere,
            polygon_slots,
        });
    }
    let plane = Plane {
        normal: vec3(cursor)?,
        d: float(cursor)?,
    };
    let (pos, neg) = match tag {
        TAG_BOTH => (true, true),
        TAG_POS => (true, false),
        TAG_NEG => (false, true),
        TAG_NONE | TAG_LEAF => (false, false),
        _ => return Err("unknown BSP node tag"),
    };
    let pos = match pos {
        true => Some(Box::new(read_node(cursor, kind, polygons, depth + 1)?)),
        false => None,
    };
    let neg = match neg {
        true => Some(Box::new(read_node(cursor, kind, polygons, depth + 1)?)),
        false => None,
    };
    let sphere = match kind {
        TreeKind::Physics => sphere(cursor)?,
        TreeKind::Cell => Sphere::default(),
    };
    Ok(Ac2CellBspNode {
        leaf: tag == TAG_LEAF,
        plane,
        pos,
        neg,
        solid: 0,
        sphere,
        polygon_slots: Box::default(),
    })
}

/// `VertexFormatInfo::GenerateOffsets` stride (the table render-codec's `Ac2Mesh` decodes with).
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
