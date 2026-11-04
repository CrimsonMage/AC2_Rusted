//! The interior sun pass of `CEnvCell::UpdateDynamicLights` (Legions 0x0047C5B0) as
//! `WorldRender::Render` runs it through `CEnvCell::DynamicLightVisibleCells(force, bSunOnly 1,
//! L0, L1, alpha)` while the viewer is inside: the cell's vertices go back to the static diffuse
//! (`m_aStaticLightDiffuse`), then two copies of the sun (`SceneTool::GetSunlight`, the directional
//! `LightSource::s_spSunlight`, type `0x40000002`) light the vertices `CLightInfo::GetSunLitIndices`
//! lists for the two nearest shadow-generation directions: the first at the sun's intensity times
//! `alpha`, the second at it times `1 - alpha`, through `CEnvCell::LightIndices`. The result is
//! the cell's `m_aStaticLightDiffusePlusSun` and its vertex colours. The lights whose range
//! animates are not added again (`bSunOnly`), so a sun pass leaves them out of the vertices.

use ac2_assets::Ac2Mesh;
use ac2_math::{Frame, Vec3};
use ac2_world::Ac2EnvCell;

use super::cell::{add_light, bake_static_colors, unpack, CellLighting};
use crate::terrain::sun_light_direction_bits;
use crate::SceneLighting;

/// `D3DFVF_DIFFUSE`: `UpdateDynamicLights` returns before lighting a mesh without it.
const FORMAT_DIFFUSE: u32 = 0x40;
/// `CLightInfo::m_aSunlights`' nine shadow-generation directions.
const SUN_DIRECTIONS: usize = 9;

/// The sun as one frame's `WorldRender::Render` hands it to the sun pass: `GetSunLightDirectionBits`'
/// two nearest directions and the first one's dot product, and `SceneTool::SetSunlight`'s
/// direction (unit), colour and diffuse intensity from `m_finalLightDir`, `m_finalDirColor` and
/// `m_finalDirBright`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunPass {
    pub l0: Option<usize>,
    pub l1: Option<usize>,
    pub alpha: f32,
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
}

impl SunPass {
    /// The pass for `lighting`. `SetSunlight` normalises the direction, and takes straight down
    /// for the zero vector.
    pub fn from_lighting(lighting: &SceneLighting) -> Self {
        let (l0, l1, alpha) = sun_light_direction_bits(lighting.light_dir);
        let direction = if lighting.light_dir.abs().max_element() < 0.0002 {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            lighting.light_dir.normalize()
        };
        Self {
            l0,
            l1,
            alpha,
            direction,
            color: lighting.dir_color,
            intensity: lighting.dir_brightness,
        }
    }
}

/// What a cell keeps for its sun passes: the static diffuse, the vertex normals, the sun-lit
/// vertex lists of the nine directions and the cell's frame.
#[derive(Debug, Clone, PartialEq)]
pub struct CellSun {
    static_colors: Box<[u32]>,
    normals: Box<[Vec3]>,
    lists: [Box<[u16]>; SUN_DIRECTIONS],
    frame: Frame,
}

impl CellSun {
    /// The sun-pass data of `cell`, or `None` where a pass cannot change its vertices: a mesh
    /// without vertex colours, or a cell with no sun-lit vertex and no light whose range
    /// animates (its vertices already hold the static diffuse).
    pub fn new(mesh: &Ac2Mesh, cell: &Ac2EnvCell, lighting: CellLighting<'_>) -> Option<Self> {
        if mesh.vertex_format & FORMAT_DIFFUSE == 0 {
            return None;
        }
        let info = lighting.light_info?;
        let lists: [Box<[u16]>; SUN_DIRECTIONS] = std::array::from_fn(|direction| {
            info.sun_lit_indices(cell.cell_id, direction)
                .map_or_else(|| Box::from([]), Box::from)
        });
        let animated = (0..info.num_lights(cell.cell_id)).any(|n| {
            info.light(cell.cell_id, n)
                .is_some_and(|(_, light)| light.is_range_animating())
        });
        if lists.iter().all(|list| list.is_empty()) && !animated {
            return None;
        }
        Some(Self {
            static_colors: bake_static_colors(mesh, cell, lighting).into(),
            normals: mesh.vertices.iter().map(|v| v.normal).collect(),
            lists,
            frame: cell.frame,
        })
    }

    /// The vertices listed for direction `index`.
    pub fn sun_lit(&self, index: usize) -> &[u16] {
        self.lists.get(index).map_or(&[], |list| list)
    }

    /// The cell's `m_aStaticLightDiffuse`.
    pub fn static_colors(&self) -> &[u32] {
        &self.static_colors
    }

    /// One sun pass: the vertex colours as packed `D3DCOLOR`s, or `None` where `L0` is not a
    /// direction (`UpdateDynamicLights` then leaves the vertices as they are).
    pub fn bake(&self, pass: &SunPass) -> Option<Vec<u32>> {
        let l0 = pass.l0?;
        let mut colors = self.static_colors.to_vec();
        let to_light = self.frame.global_to_local_vector(-pass.direction);
        let copies = [
            (Some(l0), pass.intensity * pass.alpha),
            (pass.l1, (1.0 - pass.alpha) * pass.intensity),
        ];
        for (direction, intensity) in copies {
            let Some(list) = direction.and_then(|d| self.lists.get(d)) else {
                continue;
            };
            for &index in list.iter() {
                let index = usize::from(index);
                let (Some(normal), Some(color)) = (self.normals.get(index), colors.get_mut(index))
                else {
                    // `LightIndices` stops at an index past the vertices.
                    break;
                };
                add_light(color, normal.dot(to_light), intensity, pass.color);
            }
        }
        Some(colors)
    }

    /// [`Self::bake`] as the model's vertex colours.
    pub fn bake_vertex_colors(&self, pass: &SunPass) -> Option<Vec<[f32; 4]>> {
        self.bake(pass)
            .map(|colors| colors.into_iter().map(unpack).collect())
    }
}
