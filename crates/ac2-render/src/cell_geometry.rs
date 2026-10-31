//! The env cell mesh on the CPU, for `CEnvCell::GetNearbyGeometry`: the triangles of a cell's
//! first degrade (every material's index list) that a box reaches, in the scene's space, with the
//! same one-level walk into the cells behind the portals the box touches. A dungeon decal clips
//! against what it returns (`DecalEffect::ConstructDungeonDecal`).
//!
//! The vertices are the decoded mesh's own (`Arc`s shared with the cache while the mesh is
//! resident); retail read them back from the cell's vertex buffer.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_assets::{Ac2Mesh, DecodedMeshVertex};
use ac2_math::{BBox, Vec3};

use crate::interior_cull::CullCell;

/// A cell mesh's first degrade: its vertices and one triangle index list per material
/// (`CMeshDegradeList::m_Degrades[0]`, `CMeshTriList::material_lists`).
#[derive(Debug, Clone, PartialEq)]
pub struct CellGeometry {
    vertices: Arc<[DecodedMeshVertex]>,
    lists: Vec<Arc<[u32]>>,
}

impl CellGeometry {
    /// The geometry of `mesh`'s first degrade; `None` when it has none
    /// (`GetNearbyGeometry` fails without a degrade list).
    pub fn from_mesh(mesh: &Ac2Mesh) -> Option<Self> {
        let lists: Vec<_> = mesh
            .fragments
            .iter()
            .filter(|fragment| fragment.degrade == 0)
            .map(|fragment| Arc::clone(&fragment.indices))
            .collect();
        (!lists.is_empty()).then(|| Self {
            vertices: Arc::clone(&mesh.vertices),
            lists,
        })
    }

    /// Builds the geometry from vertices and index lists directly.
    pub fn new(vertices: Arc<[DecodedMeshVertex]>, lists: Vec<Arc<[u32]>>) -> Self {
        Self { vertices, lists }
    }
}

/// The triangles `CEnvCell::GetNearbyGeometry` collects: three origins and three normals per
/// triangle, in the scene's space.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NearbyGeometry {
    pub origins: Vec<Vec3>,
    pub normals: Vec<Vec3>,
}

/// `CEnvCell::GetNearbyGeometry` for the cell `id` of `cells`: the triangles of its mesh whose
/// bounds overlap the box of half-extent `radius` around `origin` (tested in the cell's own
/// space), then, for each portal to another cell whose polygon bounds overlap the box, the same
/// for that cell, without going on from it. `None` when the cell is unknown or has no geometry.
pub fn nearby_geometry(
    cells: &HashMap<u32, CullCell>,
    id: u32,
    origin: Vec3,
    radius: f32,
) -> Option<NearbyGeometry> {
    let mut out = NearbyGeometry::default();
    collect(cells, id, origin, radius, false, &mut out)?;
    Some(out)
}

fn collect(
    cells: &HashMap<u32, CullCell>,
    id: u32,
    origin: Vec3,
    radius: f32,
    recursing: bool,
    out: &mut NearbyGeometry,
) -> Option<()> {
    let cell = cells.get(&id)?;
    let geometry = cell.geometry.as_ref()?;
    let local = cell.inverse.transform_point3(origin);
    let bounds = BBox::new(local - Vec3::splat(radius), local + Vec3::splat(radius));
    for list in &geometry.lists {
        for tri in list.chunks_exact(3) {
            let corners: Option<Vec<&DecodedMeshVertex>> = tri
                .iter()
                .map(|&i| geometry.vertices.get(i as usize))
                .collect();
            let Some(corners) = corners else { continue };
            let mut test = BBox::for_adjustment();
            for corner in &corners {
                test.expand_to_include_point(corner.position);
            }
            if !test.does_overlap(bounds, 0.0) {
                continue;
            }
            for corner in &corners {
                out.origins
                    .push(cell.world.transform_point3(corner.position));
                out.normals
                    .push(cell.world.transform_vector3(corner.normal));
            }
        }
    }
    if !recursing {
        for portal in &cell.portals {
            let Some(other) = portal.other_cell.filter(|other| *other != id) else {
                continue;
            };
            let mut test = BBox::for_adjustment();
            for point in &portal.polygon {
                test.expand_to_include_point(*point);
            }
            if !portal.polygon.is_empty() && test.does_overlap(bounds, 0.0) {
                // A neighbour that is not loaded adds nothing (`CCellPortal::GetOtherCell`).
                let _ = collect(cells, other, origin, radius, true, out);
            }
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interior_cull::CullPortal;
    use ac2_math::Mat4;

    fn quad(z: f32, x0: f32) -> CellGeometry {
        let v = |x: f32, y: f32| DecodedMeshVertex {
            position: Vec3::new(x, y, z),
            normal: Vec3::Z,
            uv: Default::default(),
        };
        CellGeometry::new(
            Arc::from([v(x0, 0.0), v(x0 + 1.0, 0.0), v(x0 + 1.0, 1.0), v(x0, 1.0)]),
            vec![Arc::from([0, 1, 2, 0, 2, 3])],
        )
    }

    fn cell(id: u32, shift: f32, geometry: CellGeometry, portals: Vec<CullPortal>) -> CullCell {
        let world = Mat4::from_translation(Vec3::new(shift, 0.0, 0.0));
        CullCell {
            id,
            bbox: BBox::new(Vec3::ZERO, Vec3::ONE),
            world,
            portals,
            no_cull: false,
            draw_sky: false,
            shared_cells: Vec::new(),
            structure: None,
            inverse: world.inverse(),
            geometry: Some(geometry),
        }
    }

    /// The box's own cell gives its triangles in the scene's space; a portal the box touches adds
    /// the neighbour's, and the neighbour's own portals are not followed.
    #[test]
    fn a_nearby_walk_takes_the_cell_and_one_ring_of_portal_neighbours() {
        let portal = |other| CullPortal {
            other_cell: Some(other),
            polygon: vec![Vec3::new(0.9, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)],
        };
        let cells: HashMap<u32, CullCell> = [
            cell(1, 0.0, quad(0.0, 0.0), vec![portal(2)]),
            cell(2, 1.0, quad(0.0, 0.0), vec![portal(3)]),
            cell(3, 2.0, quad(0.0, 0.0), vec![]),
        ]
        .into_iter()
        .map(|c| (c.id, c))
        .collect();
        let near = nearby_geometry(&cells, 1, Vec3::new(0.9, 0.5, 0.0), 0.5).unwrap();
        // Cell 1's two triangles and cell 2's two (shifted one metre east), not cell 3's.
        assert_eq!(near.origins.len(), 12);
        assert_eq!(near.normals.len(), 12);
        assert_eq!(near.origins[6], Vec3::new(1.0, 0.0, 0.0));
        // A box that misses the portal polygon stays in the cell.
        let far = nearby_geometry(&cells, 1, Vec3::new(0.2, 0.5, 0.0), 0.1).unwrap();
        assert_eq!(far.origins.len(), 6);
        assert!(nearby_geometry(&cells, 9, Vec3::ZERO, 1.0).is_none());
    }
}
