//! The AC1 portal records a creature draws, decoded in the AC1 client's own `UnPack` order
//! (`CSetup`, `CGfxObj`, `GfxObjDegradeInfo`, `CSurface`, `SurfaceTexture`, `RenderSurface`,
//! `Palette`, `PalSet`, `ClothingTable`, `CMotionTable`, `CAnimation`). Only what drawing and the
//! idle cycle need is kept; the rest (physics BSPs, cylinder spheres, lights, hooks) is read past.

use std::collections::HashMap;

use ac2_dat::ArchiveCursor;

/// A record that did not decode: its DID and what was missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac1Error {
    pub did: u32,
    pub context: &'static str,
}

impl std::fmt::Display for Ac1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AC1 record {:08X}: {}", self.did, self.context)
    }
}

impl std::error::Error for Ac1Error {}

pub type Ac1Result<T> = Result<T, Ac1Error>;

/// A cursor over one record that names the record in its errors.
pub(crate) struct Reader<'a> {
    did: u32,
    cursor: ArchiveCursor<'a>,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(did: u32, bytes: &'a [u8]) -> Self {
        Self {
            did,
            cursor: ArchiveCursor::new(bytes),
        }
    }

    fn err(&self, context: &'static str) -> Ac1Error {
        Ac1Error {
            did: self.did,
            context,
        }
    }

    pub(crate) fn f64(&mut self, context: &'static str) -> Ac1Result<f64> {
        self.cursor.read_f64().map_err(|_| self.err(context))
    }

    fn u8(&mut self, context: &'static str) -> Ac1Result<u8> {
        self.cursor.read_u8().map_err(|_| self.err(context))
    }

    fn u16(&mut self, context: &'static str) -> Ac1Result<u16> {
        self.cursor.read_u16().map_err(|_| self.err(context))
    }

    fn i16(&mut self, context: &'static str) -> Ac1Result<i16> {
        self.u16(context).map(|v| v as i16)
    }

    pub(crate) fn u32(&mut self, context: &'static str) -> Ac1Result<u32> {
        self.cursor.read_u32().map_err(|_| self.err(context))
    }

    pub(crate) fn i32(&mut self, context: &'static str) -> Ac1Result<i32> {
        self.cursor.read_i32().map_err(|_| self.err(context))
    }

    pub(crate) fn f32(&mut self, context: &'static str) -> Ac1Result<f32> {
        self.cursor.read_f32().map_err(|_| self.err(context))
    }

    pub(crate) fn vec3(&mut self, context: &'static str) -> Ac1Result<[f32; 3]> {
        Ok([self.f32(context)?, self.f32(context)?, self.f32(context)?])
    }

    /// A count no larger than the bytes left could hold at `item` bytes each.
    fn count(&mut self, raw: u32, item: usize, context: &'static str) -> Ac1Result<usize> {
        let count = raw as usize;
        if count.saturating_mul(item.max(1)) > self.cursor.remaining() {
            return Err(self.err(context));
        }
        Ok(count)
    }

    pub(crate) fn u32_count(&mut self, item: usize, context: &'static str) -> Ac1Result<usize> {
        let raw = self.u32(context)?;
        self.count(raw, item, context)
    }

    /// `PackedDWORD` / `SmartArray` count.
    fn compressed_count(&mut self, item: usize, context: &'static str) -> Ac1Result<usize> {
        let raw = self
            .cursor
            .read_retail_compressed_u32()
            .map_err(|_| self.err(context))?;
        self.count(raw, item, context)
    }

    fn skip(&mut self, len: usize, context: &'static str) -> Ac1Result<()> {
        self.cursor
            .read_bytes(len)
            .map(|_| ())
            .map_err(|_| self.err(context))
    }

    fn bytes(&mut self, len: usize, context: &'static str) -> Ac1Result<&'a [u8]> {
        self.cursor.read_bytes(len).map_err(|_| self.err(context))
    }

    fn align4(&mut self, context: &'static str) -> Ac1Result<()> {
        self.cursor.align_to(4).map_err(|_| self.err(context))
    }

    /// The DBObj header: the record's own DID.
    pub(crate) fn header(&mut self) -> Ac1Result<()> {
        let id = self.u32("missing DBObj id")?;
        if id != self.did {
            return Err(self.err("DBObj id differs from the requested DID"));
        }
        Ok(())
    }

    pub(crate) fn frame(&mut self, context: &'static str) -> Ac1Result<Frame> {
        let origin = self.vec3(context)?;
        let rotation = [
            self.f32(context)?,
            self.f32(context)?,
            self.f32(context)?,
            self.f32(context)?,
        ];
        Ok(Frame { origin, rotation })
    }

    /// `AnimHook::UnPackHook`: read past one hook and its alignment.
    fn skip_hook(&mut self) -> Ac1Result<()> {
        let kind = self.u32("missing AnimHook type")?;
        self.u32("missing AnimHook direction")?;
        self.skip_hook_body(kind)
    }

    /// The rest of a hook of type `kind` after its type and direction, and its alignment.
    pub(crate) fn skip_hook_body(&mut self, kind: u32) -> Ac1Result<()> {
        let len = match kind {
            0 | 4 | 0x11 => 0,
            1 | 2 | 6 | 0x0E | 0x0F | 0x10 | 0x12 | 0x19 => 4,
            3 => 28,
            7 | 9 | 0x0B | 0x15 => 16,
            8 | 0x0A | 0x14 | 0x16 | 0x18 => 12,
            0x0C | 0x13 | 0x17 => 8,
            0x0D | 0x1A => 40,
            5 => {
                // `AnimPartChange`: a part index and a packed GfxObj DID.
                self.u8("missing ReplaceObject part")?;
                if self.u16("missing ReplaceObject DID")? & 0x8000 != 0 {
                    self.u16("missing ReplaceObject DID")?;
                }
                0
            }
            _ => return Err(self.err("unknown AnimHook type")),
        };
        self.skip(len, "truncated AnimHook")?;
        self.align4("missing AnimHook alignment")
    }

    /// `AnimFrame::UnPack`: one frame per part, then the hooks.
    fn anim_frame(&mut self, parts: usize) -> Ac1Result<Vec<Frame>> {
        let frames = (0..parts)
            .map(|_| self.frame("missing AnimFrame part frame"))
            .collect::<Ac1Result<Vec<_>>>()?;
        let hooks = self.u32_count(8, "missing AnimFrame hook count")?;
        for _ in 0..hooks {
            self.skip_hook()?;
        }
        Ok(frames)
    }
}

/// `Frame`: an origin and a rotation `[w, x, y, z]`, as stored.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub origin: [f32; 3],
    pub rotation: [f32; 4],
}

impl Frame {
    pub const IDENTITY: Self = Self {
        origin: [0.0; 3],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
}

/// `CSetup` (`0x02`): its parts and their placements.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    pub parts: Vec<u32>,
    /// `default_scale` per part (one when the Setup has none).
    pub scales: Vec<[f32; 3]>,
    /// `placement_frames` by `Placement` key, one frame per part.
    pub placements: Vec<(u32, Vec<Frame>)>,
    pub default_motion_table: u32,
    /// `default_script`: the `PhysicsScript` the object runs when it is made (its glow, its
    /// particles).
    pub default_script: u32,
}

impl Setup {
    /// `CSetup::UnPack`.
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        r.header()?;
        let flags = r.u32("missing Setup flags")?;
        let count = r.u32_count(4, "missing Setup part count")?;
        let parts = (0..count)
            .map(|_| r.u32("missing Setup part"))
            .collect::<Ac1Result<Vec<_>>>()?;
        if flags & 1 != 0 {
            r.skip(count * 4, "missing Setup parents")?;
        }
        let scales = if flags & 2 != 0 {
            (0..count)
                .map(|_| r.vec3("missing Setup scale"))
                .collect::<Ac1Result<Vec<_>>>()?
        } else {
            vec![[1.0; 3]; count]
        };
        for _ in 0..2 {
            // Holding locations, then connection points: key, part, frame.
            let n = r.u32_count(36, "missing Setup location count")?;
            r.skip(n * 36, "missing Setup locations")?;
        }
        let n = r.u32_count(4, "missing Setup placement count")?;
        let mut placements = Vec::with_capacity(n);
        for _ in 0..n {
            let key = r.u32("missing Setup placement key")?;
            placements.push((key, r.anim_frame(count)?));
        }
        let n = r.u32_count(20, "missing Setup cylinder count")?;
        r.skip(n * 20, "missing Setup cylinders")?;
        let n = r.u32_count(16, "missing Setup sphere count")?;
        r.skip(n * 16, "missing Setup spheres")?;
        // Height, radius, step up, step down, sorting and selection spheres.
        r.skip(16 + 32, "missing Setup extents")?;
        // Lights: key, frame, colour, intensity, falloff, cone angle.
        let n = r.u32_count(48, "missing Setup light count")?;
        r.skip(n * 48, "missing Setup lights")?;
        r.u32("missing Setup default animation")?;
        let default_script = r.u32("missing Setup default script")?;
        let default_motion_table = r.u32("missing Setup default motion table")?;
        Ok(Self {
            parts,
            scales,
            placements,
            default_motion_table,
            default_script,
        })
    }

    /// The placement frame `key`, else the default (0), else the first.
    pub fn placement(&self, key: u32) -> Option<&[Frame]> {
        self.placements
            .iter()
            .find(|(k, _)| *k == key)
            .or_else(|| self.placements.iter().find(|(k, _)| *k == 0))
            .or(self.placements.first())
            .map(|(_, frames)| &frames[..])
    }
}

/// `CSWVertex`: a position, a normal and its texture coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct SwVertex {
    pub origin: [f32; 3],
    pub normal: [f32; 3],
    pub uvs: Vec<[f32; 2]>,
}

/// `CPolygon`: a fan over vertex ids, the surfaces of its two sides and their uv indices.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    /// `sides_type`: 0 single-sided, 1 both sides alike, 2 a distinct negative side.
    pub sides: i32,
    pub pos_surface: i16,
    pub neg_surface: i16,
    pub vertices: Vec<u16>,
    pub pos_uvs: Vec<u8>,
    pub neg_uvs: Vec<u8>,
}

const SIDES_BOTH: i32 = 1;
const SIDES_CLOCKWISE: i32 = 2;
const NO_POS_UVS: u8 = 4;
const NO_NEG_UVS: u8 = 8;

/// `CGfxObj` (`0x01`): what it draws.
#[derive(Debug, Clone, PartialEq)]
pub struct GfxObj {
    pub surfaces: Vec<u32>,
    /// Vertices by their `CVertexArray` id.
    pub vertices: HashMap<u16, SwVertex>,
    pub polygons: Vec<Polygon>,
    /// `did_degrade`: the `GfxObjDegradeInfo` naming the drawn degrades.
    pub degrade: Option<u32>,
}

impl GfxObj {
    /// `CGfxObj::UnPack`.
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        r.header()?;
        let flags = r.u32("missing GfxObj flags")?;
        let n = r.compressed_count(4, "missing GfxObj surface count")?;
        let surfaces = (0..n)
            .map(|_| r.u32("missing GfxObj surface"))
            .collect::<Ac1Result<Vec<_>>>()?;
        r.i32("missing VertexArray type")?;
        let n = r.u32_count(28, "missing VertexArray count")?;
        let mut vertices = HashMap::with_capacity(n);
        for _ in 0..n {
            let id = r.u16("missing SWVertex id")?;
            let uv_count = r.u16("missing SWVertex uv count")? as usize;
            let origin = r.vec3("missing SWVertex origin")?;
            let normal = r.vec3("missing SWVertex normal")?;
            let uvs = (0..uv_count)
                .map(|_| Ok([r.f32("missing SWVertex uv")?, r.f32("missing SWVertex uv")?]))
                .collect::<Ac1Result<Vec<_>>>()?;
            vertices.insert(
                id,
                SwVertex {
                    origin,
                    normal,
                    uvs,
                },
            );
        }
        if flags & 1 != 0 {
            let n = r.compressed_count(12, "missing GfxObj physics polygon count")?;
            for _ in 0..n {
                polygon(&mut r)?;
            }
            skip_physics_bsp(&mut r)?;
        }
        r.vec3("missing GfxObj sort center")?;
        let mut polygons = Vec::new();
        if flags & 2 != 0 {
            let n = r.compressed_count(12, "missing GfxObj polygon count")?;
            polygons.reserve(n);
            for _ in 0..n {
                polygons.push(polygon(&mut r)?);
            }
            skip_drawing_bsp(&mut r)?;
        }
        let degrade = if flags & 8 != 0 {
            Some(r.u32("missing GfxObj degrade DID")?)
        } else {
            None
        };
        Ok(Self {
            surfaces,
            vertices,
            polygons,
            degrade,
        })
    }
}

/// `CPolygon::UnPack`.
fn polygon(r: &mut Reader<'_>) -> Ac1Result<Polygon> {
    r.u16("missing Polygon id")?;
    let count = r.u8("missing Polygon vertex count")? as usize;
    let stippling = r.u8("missing Polygon stippling")?;
    let sides = r.i32("missing Polygon sides type")?;
    let pos_surface = r.i16("missing Polygon positive surface")?;
    let mut neg_surface = r.i16("missing Polygon negative surface")?;
    let vertices = (0..count)
        .map(|_| r.u16("missing Polygon vertex"))
        .collect::<Ac1Result<Vec<_>>>()?;
    let pos_uvs = if stippling & NO_POS_UVS == 0 {
        r.bytes(count, "missing Polygon uv indices")?.to_vec()
    } else {
        Vec::new()
    };
    let mut neg_uvs = if stippling & NO_NEG_UVS == 0 && sides == SIDES_CLOCKWISE {
        r.bytes(count, "missing Polygon negative uv indices")?
            .to_vec()
    } else {
        Vec::new()
    };
    if sides == SIDES_BOTH {
        neg_surface = pos_surface;
        neg_uvs = pos_uvs.clone();
    }
    Ok(Polygon {
        sides,
        pos_surface,
        neg_surface,
        vertices,
        pos_uvs,
        neg_uvs,
    })
}

const BSP_LEAF: u32 = 0x4C45_4146;
const BSP_PORTAL: u32 = 0x504F_5254;

/// The BSP node types with a positive child and with a negative child.
fn bsp_children(kind: u32) -> (bool, bool) {
    const BPNN: u32 = 0x4250_6E6E;
    const BPIN_LOWER: u32 = 0x4250_496E;
    const BPIN_NEG: u32 = 0x4270_494E;
    const BPNN_NEG: u32 = 0x4270_6E4E;
    const BPIN: u32 = 0x4250_494E;
    const BPNN_BOTH: u32 = 0x4250_6E4E;
    (
        matches!(kind, BPNN | BPIN_LOWER | BPIN | BPNN_BOTH),
        matches!(kind, BPIN_NEG | BPNN_NEG | BPIN | BPNN_BOTH),
    )
}

/// `BSPTREE::UnPack` of a physics tree.
fn skip_physics_bsp(r: &mut Reader<'_>) -> Ac1Result<()> {
    let kind = r.u32("missing PhysicsBSP node")?;
    match kind {
        BSP_PORTAL => {
            r.skip(16, "missing PhysicsBSP plane")?;
            skip_physics_bsp(r)?;
            skip_physics_bsp(r)
        }
        BSP_LEAF => {
            r.skip(4 + 4 + 16, "missing PhysicsBSP leaf")?;
            let n = r.u32_count(2, "missing PhysicsBSP leaf polygons")?;
            r.skip(n * 2, "missing PhysicsBSP leaf polygons")
        }
        _ => {
            r.skip(16, "missing PhysicsBSP plane")?;
            let (pos, neg) = bsp_children(kind);
            if pos {
                skip_physics_bsp(r)?;
            }
            if neg {
                skip_physics_bsp(r)?;
            }
            r.skip(16, "missing PhysicsBSP sphere")
        }
    }
}

/// `BSPTREE::UnPack` of a drawing tree.
fn skip_drawing_bsp(r: &mut Reader<'_>) -> Ac1Result<()> {
    let kind = r.u32("missing DrawingBSP node")?;
    match kind {
        BSP_PORTAL => {
            r.skip(16, "missing DrawingBSP plane")?;
            skip_drawing_bsp(r)?;
            skip_drawing_bsp(r)?;
            r.skip(16, "missing DrawingBSP sphere")?;
            let polygons = r.u32_count(2, "missing DrawingBSP polygons")?;
            let portals = r.u32_count(4, "missing DrawingBSP portals")?;
            r.skip(polygons * 2 + portals * 4, "missing DrawingBSP portal refs")
        }
        BSP_LEAF => r.skip(4, "missing DrawingBSP leaf"),
        _ => {
            r.skip(16, "missing DrawingBSP plane")?;
            let (pos, neg) = bsp_children(kind);
            if pos {
                skip_drawing_bsp(r)?;
            }
            if neg {
                skip_drawing_bsp(r)?;
            }
            r.skip(16, "missing DrawingBSP sphere")?;
            let polygons = r.u32_count(2, "missing DrawingBSP polygons")?;
            r.skip(polygons * 2, "missing DrawingBSP polygons")
        }
    }
}

/// `GfxObjDegradeInfo` (`0x11`): the GfxObj of its closest degree.
pub fn decode_degrade_closest(did: u32, bytes: &[u8]) -> Ac1Result<Option<u32>> {
    let mut r = Reader::new(did, bytes);
    r.header()?;
    let n = r.u32_count(20, "missing GfxObjDegradeInfo count")?;
    if n == 0 {
        return Ok(None);
    }
    Ok(Some(r.u32("missing GfxObjDegradeInfo GfxObj")?).filter(|id| *id != 0))
}

/// `SurfaceType` bits a material reads.
pub mod surface_type {
    pub const BASE1_SOLID: u32 = 0x1;
    pub const BASE1_IMAGE: u32 = 0x2;
    pub const BASE1_CLIPMAP: u32 = 0x4;
    pub const TRANSLUCENT: u32 = 0x10;
    pub const LUMINOUS: u32 = 0x40;
    pub const ALPHA: u32 = 0x100;
    pub const INV_ALPHA: u32 = 0x200;
    pub const ADDITIVE: u32 = 0x10000;
}

/// `CSurface` (`0x08`, stored without a DBObj header).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub flags: u32,
    /// `SurfaceTexture` and `Palette` of an image surface.
    pub texture: u32,
    pub palette: u32,
    /// The ARGB colour of a solid surface.
    pub color: u32,
    pub translucency: f32,
    pub luminosity: f32,
    pub diffuse: f32,
}

impl Surface {
    /// `CSurface::UnPack`.
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        let flags = r.u32("missing Surface type")?;
        let (texture, palette, color) =
            if flags & (surface_type::BASE1_IMAGE | surface_type::BASE1_CLIPMAP) != 0 {
                (
                    r.u32("missing Surface texture")?,
                    r.u32("missing Surface palette")?,
                    0,
                )
            } else {
                (0, 0, r.u32("missing Surface colour")?)
            };
        Ok(Self {
            flags,
            texture,
            palette,
            color,
            translucency: r.f32("missing Surface translucency")?,
            luminosity: r.f32("missing Surface luminosity")?,
            diffuse: r.f32("missing Surface diffuse")?,
        })
    }
}

/// `SurfaceTexture` (`0x05`): its `RenderSurface`s, largest first.
pub fn decode_surface_texture(did: u32, bytes: &[u8]) -> Ac1Result<Vec<u32>> {
    let mut r = Reader::new(did, bytes);
    r.header()?;
    r.u32("missing SurfaceTexture category")?;
    r.u8("missing SurfaceTexture type")?;
    let n = r.u32_count(4, "missing SurfaceTexture count")?;
    (0..n)
        .map(|_| r.u32("missing SurfaceTexture level"))
        .collect()
}

/// `PixelFormatID` values a creature texture uses.
pub mod pixel_format {
    pub const R8G8B8: u32 = 20;
    pub const A8R8G8B8: u32 = 21;
    pub const X8R8G8B8: u32 = 22;
    pub const R5G6B5: u32 = 23;
    pub const A4R4G4B4: u32 = 26;
    pub const A8: u32 = 28;
    pub const P8: u32 = 41;
    pub const INDEX16: u32 = 101;
    pub const DXT1: u32 = 0x3154_5844;
    pub const DXT3: u32 = 0x3354_5844;
    pub const DXT5: u32 = 0x3554_5844;
}

/// `RenderSurface` (`0x06`): one image and, when indexed, its default palette.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderSurface {
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub data: Vec<u8>,
    pub palette: u32,
}

impl RenderSurface {
    /// `RenderSurface::UnPack` after the DBObj header and its data category.
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        r.header()?;
        r.u32("missing RenderSurface category")?;
        let width = r.i32("missing RenderSurface width")?;
        let height = r.i32("missing RenderSurface height")?;
        let format = r.u32("missing RenderSurface format")?;
        let len = r.u32_count(1, "missing RenderSurface length")?;
        let data = r.bytes(len, "missing RenderSurface data")?.to_vec();
        let palette = if matches!(format, pixel_format::INDEX16 | pixel_format::P8) {
            r.u32("missing RenderSurface palette")?
        } else {
            0
        };
        if width <= 0 || height <= 0 {
            return Err(r.err("RenderSurface has no size"));
        }
        Ok(Self {
            width: width as u32,
            height: height as u32,
            format,
            data,
            palette,
        })
    }
}

/// `Palette` (`0x04`): ARGB colours.
pub fn decode_palette(did: u32, bytes: &[u8]) -> Ac1Result<Vec<u32>> {
    let mut r = Reader::new(did, bytes);
    r.header()?;
    let n = r.u32_count(4, "missing Palette count")?;
    (0..n).map(|_| r.u32("missing Palette colour")).collect()
}

/// `PalSet` (`0x0F`): palettes by shade.
pub fn decode_palette_set(did: u32, bytes: &[u8]) -> Ac1Result<Vec<u32>> {
    let mut r = Reader::new(did, bytes);
    r.header()?;
    let n = r.u32_count(4, "missing PalSet count")?;
    (0..n).map(|_| r.u32("missing PalSet palette")).collect()
}

/// `PalSet::GetPaletteID`: the palette at `shade` (0..1) of the set.
pub fn palette_for_shade(set: &[u32], shade: f64) -> Option<u32> {
    if set.is_empty() || !(0.0..=1.0).contains(&shade) {
        return None;
    }
    let index = ((set.len() as f64 - 0.000001) * shade) as usize;
    set.get(index.min(set.len() - 1)).copied()
}

/// `CloObjectEffect`: a part's GfxObj and its texture swaps.
#[derive(Debug, Clone, PartialEq)]
pub struct CloObjectEffect {
    pub part: u32,
    pub gfx_obj: u32,
    /// `SurfaceTexture` swaps (old, new).
    pub textures: Vec<(u32, u32)>,
}

/// `CloSubPalette`: palette index ranges (start, count) taken from a palette set.
#[derive(Debug, Clone, PartialEq)]
pub struct CloSubPalette {
    pub ranges: Vec<(u32, u32)>,
    pub palette_set: u32,
}

/// `ClothingTable` (`0x10`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClothingTable {
    /// `CloObjectEffect`s by Setup.
    pub bases: HashMap<u32, Vec<CloObjectEffect>>,
    /// `CloSubPalEffect`s by palette template.
    pub templates: HashMap<u32, Vec<CloSubPalette>>,
    /// Each palette template's icon (`CloSubPalEffect.Icon`; 0 for none).
    pub template_icons: HashMap<u32, u32>,
    /// The templates' keys in the table's stored order (ACE falls back to the first).
    pub template_order: Vec<u32>,
}

impl ClothingTable {
    /// `ClothingTable::UnPack`: two `PackableHashTable`s.
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        r.header()?;
        let mut table = Self::default();
        let n = (r.u32("missing ClothingTable base table")? & 0xFFFF) as usize;
        for _ in 0..n {
            let setup = r.u32("missing ClothingTable setup")?;
            let effects = r.u32_count(12, "missing CloObjectEffect count")?;
            let mut list = Vec::with_capacity(effects);
            for _ in 0..effects {
                let part = r.u32("missing CloObjectEffect part")?;
                let gfx_obj = r.u32("missing CloObjectEffect GfxObj")?;
                let swaps = r.u32_count(8, "missing CloTextureEffect count")?;
                let textures = (0..swaps)
                    .map(|_| {
                        Ok((
                            r.u32("missing CloTextureEffect")?,
                            r.u32("missing CloTextureEffect")?,
                        ))
                    })
                    .collect::<Ac1Result<Vec<_>>>()?;
                list.push(CloObjectEffect {
                    part,
                    gfx_obj,
                    textures,
                });
            }
            table.bases.insert(setup, list);
        }
        let n = (r.u32("missing ClothingTable template table")? & 0xFFFF) as usize;
        for _ in 0..n {
            let key = r.u32("missing ClothingTable template key")?;
            let icon = r.u32("missing CloSubPalEffect icon")?;
            table.template_icons.insert(key, icon);
            let subs = r.u32_count(8, "missing CloSubPalEffect count")?;
            let mut list = Vec::with_capacity(subs);
            for _ in 0..subs {
                let ranges = r.u32_count(8, "missing CloSubPalette range count")?;
                let ranges = (0..ranges)
                    .map(|_| {
                        Ok((
                            r.u32("missing CloSubPaletteRange")?,
                            r.u32("missing CloSubPaletteRange")?,
                        ))
                    })
                    .collect::<Ac1Result<Vec<_>>>()?;
                list.push(CloSubPalette {
                    ranges,
                    palette_set: r.u32("missing CloSubPalette set")?,
                });
            }
            table.templates.insert(key, list);
            table.template_order.push(key);
        }
        Ok(table)
    }
}

/// `AnimData`: an animation's frame range and rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimData {
    pub animation: u32,
    pub low_frame: i32,
    pub high_frame: i32,
    pub framerate: f32,
}

/// `CMotionTable` (`0x09`): its cycles and the links between motions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MotionTable {
    pub default_style: u32,
    pub style_defaults: HashMap<u32, u32>,
    /// `cycles` by `(style << 16) | (substate & 0xFFFFFF)`.
    pub cycles: HashMap<u32, Vec<AnimData>>,
    /// `links` by `(style << 16) | (substate & 0xFFFFFF)`, then by the motion linked to.
    pub links: HashMap<u32, HashMap<u32, Vec<AnimData>>>,
}

impl MotionTable {
    /// `CMotionTable::UnPack` (the modifiers are read past).
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        r.header()?;
        let default_style = r.u32("missing MotionTable default style")?;
        let n = r.u32_count(8, "missing MotionTable style defaults")?;
        let mut style_defaults = HashMap::with_capacity(n);
        for _ in 0..n {
            let style = r.u32("missing MotionTable style")?;
            style_defaults.insert(style, r.u32("missing MotionTable substate")?);
        }
        let n = r.u32_count(8, "missing MotionTable cycle count")?;
        let mut cycles = HashMap::with_capacity(n);
        for _ in 0..n {
            let key = r.u32("missing MotionTable cycle key")?;
            cycles.insert(key, motion_data(&mut r)?);
        }
        let n = r.u32_count(8, "missing MotionTable modifier count")?;
        for _ in 0..n {
            r.u32("missing MotionTable modifier key")?;
            motion_data(&mut r)?;
        }
        let n = r.u32_count(8, "missing MotionTable link count")?;
        let mut links = HashMap::with_capacity(n);
        for _ in 0..n {
            let key = r.u32("missing MotionTable link key")?;
            let m = r.u32_count(8, "missing MotionTable link motion count")?;
            let mut motions = HashMap::with_capacity(m);
            for _ in 0..m {
                let motion = r.u32("missing MotionTable link motion")?;
                motions.insert(motion, motion_data(&mut r)?);
            }
            links.insert(key, motions);
        }
        Ok(Self {
            default_style,
            style_defaults,
            cycles,
            links,
        })
    }

    /// The default style and its default substate (`CMotionTable::SetDefaultState`).
    pub fn default_state(&self) -> Option<(u32, u32)> {
        let substate = self.style_defaults.get(&self.default_style)?;
        Some((self.default_style, *substate))
    }

    /// `CMotionTable::get_link` at positive speed: the animations that take `style`'s
    /// `substate` into `motion`, else the style's own link to it.
    pub fn link(&self, style: u32, substate: u32, motion: u32) -> Option<&[AnimData]> {
        let key = (style << 16) | (substate & 0x00FF_FFFF);
        self.links
            .get(&key)
            .and_then(|motions| motions.get(&motion))
            .or_else(|| {
                self.links
                    .get(&(style << 16))
                    .and_then(|motions| motions.get(&motion))
            })
            .map(|anims| &anims[..])
    }

    /// `CMotionTable::SetDefaultState`: the default style's default substate cycle.
    pub fn idle_cycle(&self) -> Option<&[AnimData]> {
        let substate = self.style_defaults.get(&self.default_style)?;
        let key = (self.default_style << 16) | (substate & 0x00FF_FFFF);
        self.cycles.get(&key).map(|anims| &anims[..])
    }
}

/// `MotionData::UnPack`.
fn motion_data(r: &mut Reader<'_>) -> Ac1Result<Vec<AnimData>> {
    let count = r.u8("missing MotionData anim count")? as usize;
    r.u8("missing MotionData bitfield")?;
    let flags = r.u8("missing MotionData flags")?;
    r.align4("missing MotionData alignment")?;
    let anims = (0..count)
        .map(|_| {
            Ok(AnimData {
                animation: r.u32("missing AnimData animation")?,
                low_frame: r.i32("missing AnimData low frame")?,
                high_frame: r.i32("missing AnimData high frame")?,
                framerate: r.f32("missing AnimData framerate")?,
            })
        })
        .collect::<Ac1Result<Vec<_>>>()?;
    if flags & 1 != 0 {
        r.vec3("missing MotionData velocity")?;
    }
    if flags & 2 != 0 {
        r.vec3("missing MotionData omega")?;
    }
    Ok(anims)
}

/// `CAnimation` (`0x03`): per frame, every part's frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub frames: Vec<Vec<Frame>>,
}

impl Animation {
    /// `CAnimation::UnPack`.
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        r.header()?;
        let flags = r.u32("missing Animation flags")?;
        let parts = r.u32("missing Animation part count")? as usize;
        let count = r.u32_count(4, "missing Animation frame count")?;
        if flags & 1 != 0 {
            r.skip(count * 28, "missing Animation position frames")?;
        }
        let frames = (0..count)
            .map(|_| r.anim_frame(parts))
            .collect::<Ac1Result<Vec<_>>>()?;
        Ok(Self { frames })
    }
}
