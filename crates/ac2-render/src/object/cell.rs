//! Interior cells. `CEnvCell::DrawCellMesh` draws a cell's `CCellMesh` (a `CBaseMesh`, mesh type
//! 2) whose materials take the cell's dungeon appearance atoms, at the cell's frame. The vertex
//! diffuse is baked once by `CEnvCell::flush_cells`: every vertex starts at the landblock's
//! interior ambient (black for bump-mapped cells, 0.4 grey when the block has no light info), then
//! `CEnvCell::LightIndices` adds each static light over the vertices it lists, quantising to a
//! `D3DCOLOR` after every light. The lights whose range animates are left out of that bake
//! (`LightSource::GetIsRangeAnimating`); `CEnvCell::UpdateDynamicLights` adds them over it once
//! per cell, at the client time of its first light tick, and retail never repeats it (the tick
//! passes no force, and the cell stays marked `m_bDynamicLightingComputed`), so the surfaces keep
//! that one sample while the lights themselves breathe through the point lights each draw binds.

use std::sync::Arc;

use ac2_assets::{
    ac2_appearance_material_atoms, ac2_dungeon_appearance_table, load_ac2_material_with_atoms,
    Ac2Mesh,
};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};
use ac2_math::Frame;
use ac2_world::{Ac2EnvCell, Ac2LightInfo, Ac2LightKind, Ac2StaticLight};

use super::degrade::DegradeLists;
use super::model::{ObjectBatch, ObjectMaterial, ObjectModel, ObjectPart, ObjectVertex};
use super::pass::{layer_drawn, RenderSettings};

/// `CBaseMesh` type of every cell mesh (appearance atoms address mesh types).
const CELL_MESH_TYPE: u32 = 2;
/// Vertex format bit of the point-to-light vector the bump-mapped cells carry.
const FORMAT_POINT_TO_LIGHT: u32 = 0x4000_0000;
/// `flush_cells`' fallback base colour when the block has no `CLightInfo`.
const NO_LIGHT_INFO_BASE: [f32; 4] = [0.4, 0.4, 0.4, 1.0];

/// What a cell's bake needs from its landblock.
#[derive(Debug, Clone, Copy)]
pub struct CellLighting<'a> {
    pub light_info: Option<&'a Ac2LightInfo>,
    /// `CLandBlockInfo::m_InteriorAmbientLight`.
    pub interior_ambient: [f32; 3],
    /// The client time (`Timer::local_time`) the light waveforms are read at: when the cell's
    /// first light tick runs.
    pub time: f64,
}

/// The cell mesh's first degrade on the CPU (`CEnvCell::GetNearbyGeometry`'s source).
pub fn load_cell_geometry(
    store: &ResourceStore,
    cell: &Ac2EnvCell,
) -> ResourceResult<Option<crate::cell_geometry::CellGeometry>> {
    let mesh = store.load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
        DatFileKind::Portal,
        cell.cell_mesh,
        0,
    ))?;
    Ok(mesh.and_then(|mesh| crate::cell_geometry::CellGeometry::from_mesh(&mesh)))
}

/// Loads `cell` placed at `world_frame` (its landblock origin plus its own frame), with its
/// baked vertex colours, the layers `settings` draws and the degrade `CEnvCell::DrawCellMesh`
/// picks. The cell's dungeon appearance atoms are applied to its materials
/// (`CEnvCell::ApplyAtomCollection`).
pub fn load_cell_model(
    store: &ResourceStore,
    cell: &Ac2EnvCell,
    world_frame: Frame,
    lighting: CellLighting<'_>,
    settings: &RenderSettings,
) -> ResourceResult<Option<ObjectModel>> {
    let Some(mesh) = store.load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
        DatFileKind::Portal,
        cell.cell_mesh,
        0,
    ))?
    else {
        return Ok(None);
    };
    let atoms = match (cell.appearance_key, ac2_dungeon_appearance_table(store)?) {
        (Some(key), Some(table)) => {
            ac2_appearance_material_atoms(store, table, key, cell.appearance_scalar)?
        }
        _ => Vec::new(),
    };
    let degrade = cell_degrade(&mesh, settings.detail);
    let mut batches = Vec::new();
    let mut materials = std::collections::HashMap::<DataId, Option<Arc<ObjectMaterial>>>::new();
    for fragment in mesh.fragments.iter().filter(|f| f.degrade == degrade) {
        let Some(did) = mesh.materials.get(fragment.material_index).copied() else {
            continue;
        };
        let material = match materials.get(&did) {
            Some(material) => material.clone(),
            None => {
                let material = load_ac2_material_with_atoms(store, did, CELL_MESH_TYPE, &atoms)?
                    .and_then(|material| {
                        let layers: Vec<_> = material
                            .layers
                            .into_iter()
                            .filter(|layer| layer_drawn(layer, settings))
                            .collect();
                        (!layers.is_empty()).then(|| Arc::new(ObjectMaterial::new(layers)))
                    });
                materials.insert(did, material.clone());
                material
            }
        };
        if let Some(material) = material {
            batches.push(ObjectBatch {
                indices: Arc::clone(&fragment.indices),
                material,
            });
        }
    }
    if batches.is_empty() {
        return Ok(None);
    }
    let colors = bake_cell_colors(&mesh, cell, lighting);
    let vertices = mesh
        .vertices
        .iter()
        .zip(colors)
        .enumerate()
        .map(|(index, (vertex, color))| {
            let mut uvs = [[0.0; 2]; 4];
            uvs[0] = vertex.uv.to_array();
            for (set, extra) in mesh.extra_uvs.iter().take(3).enumerate() {
                uvs[set + 1] = extra[index].to_array();
            }
            ObjectVertex {
                position: vertex.position.to_array(),
                normal: vertex.normal.to_array(),
                color: unpack(color),
                uvs,
                tangents: mesh.tangents.as_ref().map_or([[0.0; 3]; 2], |t| {
                    [t[index][0].to_array(), t[index][1].to_array()]
                }),
            }
        })
        .collect();
    Ok(Some(ObjectModel {
        visual: cell.cell_mesh,
        parts: vec![ObjectPart {
            mesh: cell.cell_mesh,
            transform: world_frame.to_mat4(),
            vertices,
            batches,
            degrades: None,
            slot: (0, 0),
            skin: None,
            mesh_type: 0,
        }],
        degrade: None,
        skeleton: None,
    }))
}

/// The degrade `CEnvCell::DrawCellMesh` hands `CBaseMesh::DrawMesh`: its factor is 1 for the
/// cell's `m_degradeLevel` (0x40000002), capped at 0.5 at the medium detail setting and 0 at the
/// low one, and `DrawMesh` picks that level among the degrades the setting admits.
pub(crate) fn cell_degrade(mesh: &Ac2Mesh, detail: u32) -> usize {
    let factor = match detail {
        0 => 0.0,
        1 => 0.5,
        _ => 1.0,
    };
    let count = mesh
        .fragments
        .iter()
        .map(|f| f.degrade + 1)
        .max()
        .unwrap_or(0);
    let mut levels = vec![0; count];
    for fragment in mesh.fragments.iter() {
        levels[fragment.degrade] = fragment.detail_levels;
    }
    DegradeLists::new(&levels)
        .select(detail, factor)
        .unwrap_or(0)
}

/// `CEnvCell::flush_cells`' diffuse bake followed by `CEnvCell::UpdateDynamicLights`' first run,
/// as packed `D3DCOLOR`s: what the cell's vertices hold from its first light tick on.
pub fn bake_cell_colors(mesh: &Ac2Mesh, cell: &Ac2EnvCell, lighting: CellLighting<'_>) -> Vec<u32> {
    let mut colors = bake_static_colors(mesh, cell, lighting);
    light_animated_lights(mesh, cell, lighting, &mut colors);
    colors
}

/// `CEnvCell::flush_cells`' diffuse bake (`m_aStaticLightDiffuse`): the base colour, then every
/// light of the cell whose range does not animate.
pub fn bake_static_colors(
    mesh: &Ac2Mesh,
    cell: &Ac2EnvCell,
    lighting: CellLighting<'_>,
) -> Vec<u32> {
    let base = match lighting.light_info {
        None => NO_LIGHT_INFO_BASE,
        Some(_) if mesh.vertex_format & FORMAT_POINT_TO_LIGHT != 0 => [0.0, 0.0, 0.0, 1.0],
        Some(_) => {
            let [r, g, b] = lighting.interior_ambient;
            [r, g, b, 1.0]
        }
    };
    let mut colors = vec![pack(base); mesh.vertices.len()];
    if let Some(info) = lighting.light_info {
        for_cell_lights(info, cell, |key, light| {
            if let Some(indices) = info.lit_indices(cell.cell_id, key) {
                if !light.is_range_animating() {
                    light_indices(mesh, cell, light, indices, lighting.time, &mut colors);
                }
            }
        });
    }
    colors
}

/// The lights of `cell` in `CLightInfo::GetNumLights` / `GetLight` order, with their keys.
fn for_cell_lights(
    info: &ac2_world::Ac2LightInfo,
    cell: &Ac2EnvCell,
    mut visit: impl FnMut(u32, &Ac2StaticLight),
) {
    for n in 0..info.num_lights(cell.cell_id) {
        if let Some((key, light)) = info.light(cell.cell_id, n) {
            visit(key, light);
        }
    }
}

/// `CEnvCell::UpdateDynamicLights`' loop over the cell's lights: each one whose range animates
/// adds its lit vertices on top of the static diffuse, at the waveforms' values at
/// `lighting.time`. The cell's lights come in `CLightInfo::GetLight` order.
pub fn light_animated_lights(
    mesh: &Ac2Mesh,
    cell: &Ac2EnvCell,
    lighting: CellLighting<'_>,
    colors: &mut [u32],
) {
    let Some(info) = lighting.light_info else {
        return;
    };
    for_cell_lights(info, cell, |key, light| {
        if !light.is_range_animating() {
            return;
        }
        if let Some(indices) = info.lit_indices(cell.cell_id, key) {
            light_indices(mesh, cell, light, indices, lighting.time, colors);
        }
    });
}

/// `CEnvCell::LightIndices`. Retail reads the vertex position and normal from the device vertex
/// buffer, which holds them y-up (D3D axes), against the light's cell-local AC vector; the
/// pairing of AC y with buffer z there is the same AC-space dot product used here. The light's
/// attenuation reads its waveforms at `time`.
fn light_indices(
    mesh: &Ac2Mesh,
    cell: &Ac2EnvCell,
    light: &Ac2StaticLight,
    indices: &[u16],
    time: f64,
    colors: &mut [u32],
) {
    let local = cell.frame.global_to_local_point(light.origin);
    for &index in indices {
        let index = usize::from(index);
        let Some(vertex) = mesh.vertices.get(index) else {
            return;
        };
        let (p, n) = (vertex.position, vertex.normal);
        let (to_light, attenuation) = match light.kind {
            Ac2LightKind::Point => {
                let d = local - p;
                let distance = d.length().max(0.0002);
                (d / distance, light.attenuation_at(distance, time))
            }
            Ac2LightKind::Directional => (
                cell.frame.global_to_local_vector(-light.direction),
                light.intensity_at(time),
            ),
            // `LightSource::GetDiffuse` of a sunlight-linked light is the sun's ambient, which is
            // black inside.
            Ac2LightKind::Sunlight => continue,
        };
        let n_dot_l = n.dot(to_light);
        if n_dot_l <= 0.0002 {
            continue;
        }
        let scale = n_dot_l.min(1.0) * attenuation;
        let [r, g, b, _] = unpack(colors[index]);
        let lit = [
            r + light.color[0] * scale,
            g + light.color[1] * scale,
            b + light.color[2] * scale,
            1.0,
        ];
        colors[index] = pack(lit.map(|c| c.clamp(0.0, 1.0)));
    }
}

/// `RGBAColor::GetColor32`: `_ftol2` of each channel times 255, ARGB.
fn pack([r, g, b, a]: [f32; 4]) -> u32 {
    let channel = |c: f32| ((c * 255.0) as i32 as u32) & 0xFF;
    (channel(a) << 24) | (channel(r) << 16) | (channel(g) << 8) | channel(b)
}

/// `RGBAColor::SetColor32`.
fn unpack(color: u32) -> [f32; 4] {
    let channel = |shift: u32| ((color >> shift) & 0xFF) as f32 * 0.003_921_569;
    [channel(16), channel(8), channel(0), channel(24)]
}

#[cfg(test)]
mod tests {
    use super::{pack, unpack};

    #[test]
    fn colors_round_trip_through_d3dcolor_truncation() {
        assert_eq!(pack([1.0, 0.5, 0.0, 1.0]), 0xFFFF_7F00);
        let [r, g, b, a] = unpack(0xFF80_4020);
        assert!((r - 128.0 / 255.0).abs() < 1e-6 && (g - 64.0 / 255.0).abs() < 1e-6);
        assert!((b - 32.0 / 255.0).abs() < 1e-6 && a == 1.0);
    }
}
