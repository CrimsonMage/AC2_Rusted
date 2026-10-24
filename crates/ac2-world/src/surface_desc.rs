//! AC2 `CSurfaceDesc` (`0x13000000`): the landscape surface vocabulary, decoded in
//! `CSurfaceDesc::StreamPack` order. Each `LScapeSurfaceType` (terrain surfaces and roads) holds
//! slope-banded `MaterialGroup`s whose weighted lists pick a vertex's base, detail and bump
//! materials, its four alpha masks and its vertex tint (`MaterialGroup::GetVertexInfo`).

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

const AC2_SURFACE_DESC_DID: DataId = DataId::new(0x1300_0000);
const MAX_ITEMS: u32 = 4096;

/// `LScapeMaterialDesc` / `LScapeAlphaDesc`: a DID with a pick weight and a tile width in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WeightedDid {
    pub(crate) did: DataId,
    pub(crate) frequency: f32,
    pub(crate) tile_width: f32,
}

/// `LScapeVertexColorDesc`: a base tint with random hue, saturation and brightness ranges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VertexColorDesc {
    pub(crate) far_vertex_color: [f32; 4],
    pub(crate) vertex_color: [f32; 4],
    pub(crate) hue: (f32, f32, u32),
    pub(crate) saturate: (f32, f32, u32),
    pub(crate) bright: (f32, f32, u32),
    pub(crate) frequency: f32,
}

/// `MaterialGroup`: one slope band (`min_pitch`..`max_pitch` of the normal's horizontal length).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MaterialGroup {
    pub(crate) min_pitch: f32,
    pub(crate) max_pitch: f32,
    pub(crate) physics_material: DataId,
    pub(crate) vertex_colors: Vec<VertexColorDesc>,
    pub(crate) vcolor_freq_count: f32,
    pub(crate) base: (Vec<WeightedDid>, f32),
    pub(crate) corner_alpha: (Vec<WeightedDid>, f32),
    pub(crate) three_corner_alpha: (Vec<WeightedDid>, f32),
    pub(crate) diagonal_alpha: (Vec<WeightedDid>, f32),
    pub(crate) side_alpha: (Vec<WeightedDid>, f32),
    pub(crate) detail: (Vec<WeightedDid>, f32),
    pub(crate) bump: (Vec<WeightedDid>, f32),
}

/// `LScapeSurfaceType`: the slope bands of one surface (`surf_index` `0x800000NN`) or road.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SurfaceType {
    pub(crate) surf_index: u32,
    pub(crate) groups: Vec<MaterialGroup>,
}

impl SurfaceType {
    /// `LScapeSurfaceType::GetVertexInfo`'s band test: `min^2 <= pitchSq <= max^2`, first match.
    pub(crate) fn group_for_pitch(&self, pitch_squared: f32) -> Option<&MaterialGroup> {
        self.groups.iter().find(|group| {
            group.min_pitch * group.min_pitch <= pitch_squared
                && pitch_squared <= group.max_pitch * group.max_pitch
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ac2SurfaceDesc {
    pub(crate) far_texture: DataId,
    pub(crate) far_texture_scale: f32,
    pub(crate) far_water_material: DataId,
    pub(crate) cloud_texture: DataId,
    pub(crate) cloud_texture_scale: f32,
    pub(crate) cloud_scroll: [f32; 2],
    surfaces: Vec<SurfaceType>,
    roads: Vec<SurfaceType>,
}

impl Ac2SurfaceDesc {
    /// `CSurfaceDesc::GetVertexInfo`'s lookup: the surface whose `surf_index` matches.
    pub(crate) fn surface(&self, surf_index: u32) -> Option<&SurfaceType> {
        self.surfaces.iter().find(|s| s.surf_index == surf_index)
    }

    /// `CSurfaceDesc::GetRoadInfo`'s lookup by the vertex's road bits.
    pub(crate) fn road(&self, road_index: u32) -> Option<&SurfaceType> {
        self.roads.iter().find(|s| s.surf_index == road_index)
    }
}

impl ResourcePackedDecode for Ac2SurfaceDesc {
    const TYPE_NAME: &'static str = "AC2 SurfaceDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode(&mut ArchiveCursor::new(bytes), qdid).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        let groups: usize = self
            .surfaces
            .iter()
            .chain(&self.roads)
            .map(|s| s.groups.len())
            .sum();
        std::mem::size_of::<Self>() + groups * 512
    }
}

pub(crate) fn load_ac2_surface_desc(
    store: &ResourceStore,
) -> ResourceResult<Option<CachedResource<Ac2SurfaceDesc>>> {
    store.load_cached_packed::<Ac2SurfaceDesc>(TypedResourceRef::new(
        DatFileKind::Portal,
        AC2_SURFACE_DESC_DID,
        0,
    ))
}

type Decode<T> = Result<T, &'static str>;

fn word(cursor: &mut ArchiveCursor<'_>) -> Decode<u32> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn float(cursor: &mut ArchiveCursor<'_>) -> Decode<f32> {
    cursor.read_f32().map_err(|_| "record ended early")
}

fn count(cursor: &mut ArchiveCursor<'_>) -> Decode<usize> {
    let n = word(cursor)?;
    if n > MAX_ITEMS {
        return Err("list count exceeded the record");
    }
    Ok(n as usize)
}

/// `RGBAColor::StreamPack`: one ARGB word (`RGBAColor::SetColor32`).
fn color(cursor: &mut ArchiveCursor<'_>) -> Decode<[f32; 4]> {
    Ok(crate::color::set_color32(word(cursor)?))
}

/// `CSurfaceDesc::StreamPack`: version, `DBObj` type header (the DID), far-terrain and cloud
/// parameters, then the surface and road `LScapeSurfaceType` lists.
fn decode(cursor: &mut ArchiveCursor<'_>, qdid: QualifiedDataId) -> Decode<Ac2SurfaceDesc> {
    let _version = word(cursor)?;
    if word(cursor)? != qdid.data_id().raw() {
        return Err("embedded DID did not match DAT identity");
    }
    let desc = Ac2SurfaceDesc {
        far_texture: DataId::new(word(cursor)?),
        far_texture_scale: float(cursor)?,
        far_water_material: DataId::new(word(cursor)?),
        cloud_texture: DataId::new(word(cursor)?),
        cloud_texture_scale: float(cursor)?,
        cloud_scroll: [float(cursor)?, float(cursor)?],
        surfaces: surface_types(cursor)?,
        roads: surface_types(cursor)?,
    };
    if cursor.remaining() != 0 {
        return Err("trailing bytes after SurfaceDesc");
    }
    Ok(desc)
}

/// `LScapeSurfaceType::StreamPack`: `surf_index`, aligned `MaterialGroup` list, `empty_road`.
fn surface_types(cursor: &mut ArchiveCursor<'_>) -> Decode<Vec<SurfaceType>> {
    (0..count(cursor)?)
        .map(|_| {
            let surf_index = word(cursor)?;
            let groups = (0..count(cursor)?)
                .map(|_| material_group(cursor))
                .collect::<Decode<Vec<_>>>()?;
            let _empty_road = word(cursor)?;
            Ok(SurfaceType { surf_index, groups })
        })
        .collect()
}

/// `MaterialGroup::StreamPack` field order: pitch band, colour distance and interpolation,
/// auto-edge, physics material, vertex colours, then the base, corner, three-corner, diagonal,
/// side, detail and bump lists, each followed by its frequency total.
fn material_group(cursor: &mut ArchiveCursor<'_>) -> Decode<MaterialGroup> {
    let min_pitch = float(cursor)?;
    let max_pitch = float(cursor)?;
    let _color_distance = float(cursor)?;
    let _color_interp = word(cursor)?;
    let _auto_edge = word(cursor)?;
    let physics_material = DataId::new(word(cursor)?);
    let vertex_colors = (0..count(cursor)?)
        .map(|_| vertex_color(cursor))
        .collect::<Decode<Vec<_>>>()?;
    let vcolor_freq_count = float(cursor)?;
    Ok(MaterialGroup {
        min_pitch,
        max_pitch,
        physics_material,
        vertex_colors,
        vcolor_freq_count,
        base: weighted(cursor)?,
        corner_alpha: weighted(cursor)?,
        three_corner_alpha: weighted(cursor)?,
        diagonal_alpha: weighted(cursor)?,
        side_alpha: weighted(cursor)?,
        detail: weighted(cursor)?,
        bump: weighted(cursor)?,
    })
}

/// A counted `LScapeMaterialDesc`/`LScapeAlphaDesc` list (id, frequency, tile width) and the
/// frequency total that follows it.
fn weighted(cursor: &mut ArchiveCursor<'_>) -> Decode<(Vec<WeightedDid>, f32)> {
    let list = (0..count(cursor)?)
        .map(|_| {
            Ok(WeightedDid {
                did: DataId::new(word(cursor)?),
                frequency: float(cursor)?,
                tile_width: float(cursor)?,
            })
        })
        .collect::<Decode<Vec<_>>>()?;
    Ok((list, float(cursor)?))
}

/// `LScapeVertexColorDesc::StreamPack`.
fn vertex_color(cursor: &mut ArchiveCursor<'_>) -> Decode<VertexColorDesc> {
    let far_vertex_color = color(cursor)?;
    let vertex_color = color(cursor)?;
    let mut range =
        || -> Decode<(f32, f32, u32)> { Ok((float(cursor)?, float(cursor)?, word(cursor)?)) };
    let hue = range()?;
    let saturate = range()?;
    let bright = range()?;
    Ok(VertexColorDesc {
        far_vertex_color,
        vertex_color,
        hue,
        saturate,
        bright,
        frequency: float(cursor)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(bytes: &mut Vec<u8>, words: &[u32]) {
        for w in words {
            bytes.extend_from_slice(&w.to_le_bytes());
        }
    }

    #[test]
    fn decodes_streampack_order_and_selects_the_pitch_band() {
        let f = f32::to_bits;
        let mut bytes = Vec::new();
        push(
            &mut bytes,
            &[
                0,
                0x1300_0000,
                0x4000_0001,
                f(0.5),
                0x3100_0001,
                0x4000_0002,
            ],
        );
        push(&mut bytes, &[f(0.1), f(0.2), f(0.3)]);
        // One surface with two bands, no roads.
        push(&mut bytes, &[1, 0x8000_0001, 2]);
        for (min, max, base) in [(0.0, 0.5, 0x2B00_0010_u32), (0.5, 1.0, 0x2B00_0011)] {
            push(
                &mut bytes,
                &[f(min), f(max), f(20.0), 1, 1, 0x3700_0000, 0, 0],
            );
            push(&mut bytes, &[1, base, f(1.0), f(20.0), f(1.0)]);
            for _ in 0..6 {
                push(&mut bytes, &[0, 0]);
            }
        }
        push(&mut bytes, &[0, 0]);
        let desc =
            Ac2SurfaceDesc::decode_packed(QualifiedDataId::new(AC2_SURFACE_DESC_DID, 0), 0, &bytes)
                .unwrap();
        assert_eq!(desc.cloud_scroll, [0.2, 0.3]);
        let surface = desc.surface(0x8000_0001).unwrap();
        assert_eq!(
            surface.group_for_pitch(0.04).unwrap().base.0[0].did.raw(),
            0x2B00_0010
        );
        assert_eq!(
            surface.group_for_pitch(0.64).unwrap().base.0[0].did.raw(),
            0x2B00_0011
        );
        assert!(surface.group_for_pitch(1.01).is_none());
        assert!(desc.road(1).is_none());
    }
}
