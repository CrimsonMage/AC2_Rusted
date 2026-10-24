//! Terrain polygon layers: `LScapeMaterialInfo::Create` picks, from the `SurfaceInfo` of a
//! polygon's four corners and their roads, up to three material layers. Layer 0 is opaque; each
//! later layer is blended over it through an alpha mask (corner, side, three-corner or diagonal)
//! rotated so the mask's authored corner lands on the right polygon corner.

use ac2_core::DataId;
use ac2_world::Ac2SurfaceInfo;

/// AC2 `rot_types` for an alpha layer's UVs (`LScapeMaterialInfo::CalcRotUVs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AlphaRotation {
    Rot0 = 1,
    Rot90 = 2,
    Rot180 = 3,
    Rot270 = 4,
}

impl AlphaRotation {
    /// `CalcRotUVs`: the mask UV at polygon corner `index` (0 SW, 1 SE, 2 NW, 3 NE).
    pub fn corner_uv(self, index: usize) -> [f32; 2] {
        // ROT_0 puts the mask's top-left on the NW corner; each step turns it a quarter.
        const TABLE: [[[f32; 2]; 4]; 4] = [
            [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
            [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
            [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
        ];
        TABLE[self as usize - 1][index & 3]
    }
}

/// One drawn layer: the base `RenderMaterial`, its tile width in metres and, above layer 0, the
/// alpha mask texture blending it over the layers beneath.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TerrainLayer {
    pub material: DataId,
    /// `SurfaceInfo::base_tiling` bits (metres per texture repeat).
    tiling_bits: u32,
    pub alpha: Option<(DataId, AlphaRotation)>,
}

impl TerrainLayer {
    pub fn tiling(&self) -> f32 {
        f32::from_bits(self.tiling_bits)
    }
}

/// The layers of one terrain polygon, bottom first; polygons with equal layers share a material
/// (`LScapeMaterialInfo::Match`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TerrainLayers(pub Vec<TerrainLayer>);

/// Corner order of every polygon: SW, SE, NW, NE.
pub type Corners<'a> = [&'a Ac2SurfaceInfo; 4];

const SW: usize = 0;
const SE: usize = 1;
const NW: usize = 2;
const NE: usize = 3;

/// `LScapeMaterialInfo::Create` with landscape blending on (`SceneTool::m_bLandscapeTerrainBlending`).
/// Returns `None` when layer 0 has no base material, where retail builds no material.
pub fn terrain_layers(corners: Corners<'_>, roads: Corners<'_>) -> Option<TerrainLayers> {
    let road = road_pick(roads);
    let mut mat: Vec<(&Ac2SurfaceInfo, Option<(Option<DataId>, AlphaRotation)>)> = Vec::new();
    if let RoadPick::Full(sample) = road {
        mat.push((sample, None));
    } else {
        corner_layers(corners, &mut mat);
        if let RoadPick::Masked {
            sample,
            alpha,
            rotation,
        } = road
        {
            if mat.len() < 3 {
                mat.push((sample, Some((Some(alpha), rotation))));
            }
        }
    }
    let mut layers = Vec::with_capacity(mat.len());
    for (index, (surface, alpha)) in mat.into_iter().enumerate() {
        // `GetCombineMaterials` and `GetAlphaTexture` fail the whole material on a missing slot.
        let material = surface.base_material?;
        let alpha = match (index, alpha) {
            (0, _) => None,
            (_, Some((Some(mask), rotation))) => Some((mask, rotation)),
            _ => return None,
        };
        layers.push(TerrainLayer {
            material,
            tiling_bits: surface.base_tiling.to_bits(),
            alpha,
        });
    }
    Some(TerrainLayers(layers))
}

/// The terrain half of `Create`: which corners differ, the base layer, and up to two overlays
/// each masked by its own corner (or side) alpha.
fn corner_layers<'a>(
    c: Corners<'a>,
    mat: &mut Vec<(&'a Ac2SurfaceInfo, Option<(Option<DataId>, AlphaRotation)>)>,
) {
    use AlphaRotation::*;
    let same = |a: usize, b: usize| c[a].same_layer(c[b]);
    let (s, w, n, e) = (same(SW, SE), same(SW, NW), same(NE, NW), same(SE, NE));
    let (sw_ne, se_nw) = (same(SW, NE), same(SE, NW));
    let corner = |i: usize, rot| (c[i], Some((c[i].corner_alpha, rot)));
    let side = |i: usize, mask: usize, rot| (c[i], Some((c[mask].side_alpha, rot)));
    let base = |i: usize| (c[i], None);
    let layers: Vec<_> = if s {
        if w {
            if e {
                vec![base(SW)]
            } else {
                vec![base(SW), corner(NE, Rot90)]
            }
        } else if e {
            vec![base(SW), corner(NW, Rot0)]
        } else if n {
            vec![base(NW), side(SE, NE, Rot180)]
        } else {
            vec![base(SW), corner(NW, Rot0), corner(NE, Rot90)]
        }
    } else if n {
        if w {
            vec![base(NE), corner(SE, Rot180)]
        } else if e {
            vec![base(NE), corner(SW, Rot270)]
        } else {
            vec![base(NE), corner(SE, Rot180), corner(SW, Rot270)]
        }
    } else if e {
        if w {
            vec![base(SW), side(NE, NE, Rot90)]
        } else {
            vec![base(NE), corner(NW, Rot0), corner(SW, Rot270)]
        }
    } else if w {
        vec![base(SW), corner(NE, Rot90), corner(SE, Rot180)]
    } else if sw_ne {
        vec![base(SW), corner(NW, Rot0), corner(SE, Rot180)]
    } else if se_nw {
        vec![base(SE), corner(SW, Rot270), corner(NE, Rot90)]
    } else {
        // Four different corners: only three layers fit, SE is dropped.
        vec![base(SW), corner(NW, Rot0), corner(NE, Rot90)]
    };
    mat.extend(layers);
}

#[derive(Debug, Clone, Copy)]
enum RoadPick<'a> {
    None,
    /// All four corners are the same road: the polygon draws the road alone.
    Full(&'a Ac2SurfaceInfo),
    Masked {
        sample: &'a Ac2SurfaceInfo,
        alpha: DataId,
        rotation: AlphaRotation,
    },
}

/// The road half of `Create`: which road corners match and which road alpha shapes the strip.
fn road_pick(r: Corners<'_>) -> RoadPick<'_> {
    use AlphaRotation::*;
    let has = |i: usize| r[i].base_material.is_some();
    // Road matches also require the second corner to carry a road.
    let m = |a: usize, b: usize, need: usize| r[a].same_layer(r[b]) && has(need);
    let (s, w, n, e) = (m(SW, SE, SW), m(SW, NW, NW), m(NE, NW, NW), m(SE, NE, NE));
    let (sw_ne, se_nw) = (m(SW, NE, NE), m(SE, NW, NW));
    if !(0..4).any(has) {
        return RoadPick::None;
    }
    let masked = |i: usize, alpha: Option<DataId>, rotation| match alpha {
        Some(alpha) => RoadPick::Masked {
            sample: r[i],
            alpha,
            rotation,
        },
        None => RoadPick::None,
    };
    #[derive(Clone, Copy)]
    enum Step {
        L72,
        L9b,
        Lc5,
        L0b,
    }
    let mut step = if s {
        if w {
            if n && e {
                return RoadPick::Full(r[NE]);
            }
            Step::L72
        } else {
            Step::L9b
        }
    } else {
        Step::L72
    };
    loop {
        step = match step {
            Step::L72 if !w => Step::L9b,
            Step::L72 if !n => Step::Lc5,
            Step::L72 if !e => Step::L9b,
            Step::L72 => return masked(SW, r[NE].corner_alpha, Rot0),
            Step::L9b if n && e => return masked(NE, r[NE].three_corner_alpha, Rot90),
            Step::L9b => Step::Lc5,
            Step::Lc5 if s && e => return masked(SE, r[SE].three_corner_alpha, Rot180),
            Step::Lc5 if s && w => return masked(SW, r[SW].three_corner_alpha, Rot270),
            Step::Lc5 => Step::L0b,
            Step::L0b => {
                return if n {
                    let alpha = if w {
                        r[NW].three_corner_alpha
                    } else {
                        r[NW].side_alpha
                    };
                    masked(NW, alpha, Rot0)
                } else if e {
                    masked(NE, r[NE].side_alpha, Rot90)
                } else if s {
                    masked(SE, r[SE].side_alpha, Rot180)
                } else if w {
                    masked(NW, r[NW].side_alpha, Rot270)
                } else if sw_ne {
                    masked(SW, r[SW].diagonal_alpha, Rot0)
                } else if se_nw {
                    masked(SE, r[SE].diagonal_alpha, Rot90)
                } else if has(NW) {
                    masked(NW, r[NW].corner_alpha, Rot0)
                } else if has(NE) {
                    masked(NE, r[NE].corner_alpha, Rot90)
                } else if has(SE) {
                    masked(SE, r[SE].corner_alpha, Rot180)
                } else {
                    masked(SW, r[SW].corner_alpha, Rot270)
                };
            }
        }
    }
}

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;
