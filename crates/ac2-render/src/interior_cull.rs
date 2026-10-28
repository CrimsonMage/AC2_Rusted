//! Which interior cells draw: the portal walk of `CEnvCell::CheckVisibility` (from outside),
//! `CEnvCell::CullCells` / `CullCellsInternal` (from a cell) and `CullCellGroupInternal` (for
//! the always-drawn groups of `m_bNoCull` cells that `ComputeCellGroups` builds).
//!
//! The walk starts with the camera's frustum. Each portal of a cell whose polygon the frustum
//! sees (clipped against it) passes a narrower frustum, made from the clipped polygon, to the cell
//! behind it; the cells reached are the ones that draw this frame, and a portal to the outside
//! adds a viewport the outdoors shows through.
//!
//! All positions are in the scene's space (the cell's block offset already applied, as retail's
//! `LandDefs::get_block_offset` does against the viewer's block).

use std::collections::{BTreeSet, HashMap, HashSet};

use ac2_math::{BBox, Mat4, Vec3};
use std::sync::Arc;

use ac2_world::{culling_plane_check, Ac2CellStruct};

use crate::frustum::{Frustum, Relation};
use crate::worker_polygon::WorkerPolygon;

/// One `CCellPortal`: the cell behind it (`None` opens to the outside) and its polygon in the
/// cell's own space.
#[derive(Debug, Clone, PartialEq)]
pub struct CullPortal {
    pub other_cell: Option<u32>,
    pub polygon: Vec<Vec3>,
}

/// What the cull reads of a `CEnvCell`.
#[derive(Debug, Clone, PartialEq)]
pub struct CullCell {
    pub id: u32,
    /// `m_boundingBox` in the scene's space.
    pub bbox: BBox,
    /// The cell's frame as a matrix in the scene's space (`Frame::GetMatrix` plus the block
    /// offset).
    pub world: Mat4,
    pub portals: Vec<CullPortal>,
    /// `m_bNoCull`: the cell belongs to a group that draws as a whole.
    pub no_cull: bool,
    /// `m_bDrawSky`.
    pub draw_sky: bool,
    /// `m_sharedCells`.
    pub shared_cells: Vec<u32>,
    /// The cell mesh's structure, to tell whether a point is in the cell (`PointInCell`).
    pub structure: Option<Arc<Ac2CellStruct>>,
    /// `world` inverted: the scene's space into the cell's.
    pub inverse: Mat4,
}

impl CullCell {
    /// `CEnvCell::GetTouchesOutside`.
    pub fn touches_outside(&self) -> bool {
        self.portals.iter().any(|p| p.other_cell.is_none())
    }

    /// `CEnvCell::point_in_cell` for a point of the scene's space.
    pub fn contains(&self, point: Vec3) -> bool {
        self.structure
            .as_ref()
            .is_some_and(|s| s.point_in_cell(self.inverse.transform_point3(point)))
    }

    fn portal_polygon(&self, portal: usize) -> WorkerPolygon {
        let world = self.world;
        WorkerPolygon::from_polygon(&self.portals[portal].polygon, |v| world.transform_point3(v))
    }
}

/// `EnvCellGroup`: the cells that share one set (`m_cellSet`), the portals to the outside any of
/// them has (`m_aOutsidePortals`: cell and portal index) and the cells they open to that do not
/// belong to the group (`m_aExitCells`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvCellGroup {
    pub cells: BTreeSet<u32>,
    pub outside_portals: Vec<(u32, usize)>,
    pub exit_cells: Vec<u32>,
}

/// One landblock's cells for the cull: its env cells by id, their groups, the cells
/// `CLandBlockInfo::m_aOutsideStabList` names and the interior fog's culling plane.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockCull {
    pub cells: HashMap<u32, CullCell>,
    pub groups: HashMap<u32, EnvCellGroup>,
    pub outside_stab: Vec<u32>,
    /// `m_fCullingPlane` of the cell the cull starts from: the block's interior fog far distance
    /// when it has fog, else -1.
    pub culling_plane: f32,
}

impl BlockCull {
    /// The cell holding `point`, if any (`CEnvCell::find_visible_child_cell`'s scan).
    pub fn cell_at(&self, point: Vec3) -> Option<u32> {
        let mut ids: Vec<u32> = self.cells.keys().copied().collect();
        ids.sort_unstable();
        ids.into_iter().find(|id| self.cells[id].contains(point))
    }
}

/// What a cull found.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CullResult {
    /// The cells that draw, in the order the walk reached them.
    pub visible: Vec<u32>,
    /// A visible cell draws the sky (`m_bDrawSky`).
    pub draw_sky: bool,
    /// A cell was dropped by the interior fog's culling plane.
    pub fog_culled: bool,
    /// The viewports the outdoors shows through (`_outsideViewports`).
    pub outside_viewports: Vec<Frustum>,
    /// The share of each reached cell's opening the walk saw (`m_fVisibilityPercentage`).
    pub visibility: HashMap<u32, f32>,
}

/// The cells of the loaded blocks and what the walk needs between calls.
pub struct InteriorCull<'a> {
    cells: &'a HashMap<u32, CullCell>,
    groups: &'a HashMap<u32, EnvCellGroup>,
    camera: Vec3,
    /// `-1` for none (`m_fCullingPlane`).
    culling_plane: f32,
    on_stack: HashSet<u32>,
    result: CullResult,
}

/// `CEnvCell::ComputeCellGroups` over all cells: each `m_bNoCull` cell gets the group of the cells
/// that share with it. Returns the group of every such cell.
pub fn compute_cell_groups(cells: &HashMap<u32, CullCell>) -> HashMap<u32, EnvCellGroup> {
    let mut groups: Vec<EnvCellGroup> = Vec::new();
    let mut of_cell: HashMap<u32, usize> = HashMap::new();
    let mut ids: Vec<u32> = cells.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        let cell = &cells[&id];
        if !cell.no_cull {
            continue;
        }
        // An existing group already holding the cell is reused.
        if let Some(index) = groups.iter().position(|g| g.cells.contains(&id)) {
            of_cell.insert(id, index);
            continue;
        }
        let mut group = EnvCellGroup::default();
        group.cells.insert(id);
        for shared in &cell.shared_cells {
            let Some(other) = cells.get(shared) else {
                continue;
            };
            group.cells.insert(id);
            for (index, portal) in other.portals.iter().enumerate() {
                match portal.other_cell {
                    None => group.outside_portals.push((*shared, index)),
                    Some(behind) => {
                        if cells.get(&behind).is_some_and(|c| !c.no_cull)
                            && !group.exit_cells.contains(&behind)
                        {
                            group.exit_cells.push(behind);
                        }
                    }
                }
            }
        }
        groups.push(group);
        of_cell.insert(id, groups.len() - 1);
    }
    of_cell
        .into_iter()
        .map(|(id, index)| (id, groups[index].clone()))
        .collect()
}

/// `CEnvCell::CheckVisibility`: whether `cell` can be seen from the outside (or from another
/// block's cell) through `frustum`, and if so the frustum to cull it with (the camera's own).
pub fn check_visibility(cell: &CullCell, viewer_inside: bool, frustum: &Frustum) -> bool {
    if frustum.check(cell.bbox.min, cell.bbox.max) == Relation::Outside {
        return false;
    }
    if cell.no_cull {
        return true;
    }
    for (index, portal) in cell.portals.iter().enumerate() {
        let wanted = if portal.other_cell.is_none() {
            !viewer_inside
        } else {
            viewer_inside
        };
        if !wanted {
            continue;
        }
        if frustum
            .check_polygon(&cell.portal_polygon(index), false)
            .is_some()
        {
            return true;
        }
    }
    false
}

impl<'a> InteriorCull<'a> {
    pub fn new(
        cells: &'a HashMap<u32, CullCell>,
        groups: &'a HashMap<u32, EnvCellGroup>,
        camera: Vec3,
    ) -> Self {
        Self {
            cells,
            groups,
            camera,
            culling_plane: -1.0,
            on_stack: HashSet::new(),
            result: CullResult::default(),
        }
    }

    /// `CEnvCell::CullCells`: the walk from `start` with the camera's frustum. `culling_plane` is
    /// the interior fog's far distance, or `-1`.
    pub fn cull_cells(&mut self, start: u32, frustum: &Frustum, culling_plane: f32) {
        if !self.cells.contains_key(&start) {
            return;
        }
        self.culling_plane = culling_plane;
        self.result.visibility.insert(start, 1.0);
        self.cull_cells_internal(start, frustum, true);
    }

    /// The cells reached so far.
    pub fn finish(self) -> CullResult {
        self.result
    }

    fn mark_visible(&mut self, id: u32) {
        if !self.result.visible.contains(&id) {
            self.result.visible.push(id);
        }
    }

    /// `CEnvCell::CullCellsInternal`.
    fn cull_cells_internal(&mut self, id: u32, frustum: &Frustum, first: bool) {
        let Some(cell) = self.cells.get(&id) else {
            return;
        };
        if self.on_stack.contains(&id) {
            return;
        }
        if frustum.check(cell.bbox.min, cell.bbox.max) == Relation::Outside {
            return;
        }
        // Retail checks the cell's own `m_fCullingPlane` here, which only the cell `CullCells`
        // starts from carries (the start skips the check); every other cell holds -1.
        self.on_stack.insert(id);
        self.mark_visible(id);
        if cell.draw_sky {
            self.result.draw_sky = true;
        }
        if cell.no_cull {
            let plane = if first { self.culling_plane } else { -1.0 };
            self.cull_cell_group(id, frustum, plane);
        } else {
            for (index, portal) in cell.portals.iter().enumerate() {
                let behind = portal.other_cell.and_then(|other| self.cells.get(&other));
                let behind_no_cull = behind.is_some_and(|c| c.no_cull);
                let polygon = cell.portal_polygon(index);
                let clipped = if behind_no_cull {
                    None
                } else {
                    match frustum.check_polygon(&polygon, !first) {
                        Some(clipped) => Some(clipped),
                        None => continue,
                    }
                };
                let narrowed = match (&clipped, behind_no_cull) {
                    (Some(clipped), false) => frustum.from_portal(clipped).unwrap_or(*frustum),
                    _ => *frustum,
                };
                match portal.other_cell {
                    None => self.result.outside_viewports.push(narrowed),
                    Some(other) => {
                        if behind.is_none() {
                            continue;
                        }
                        let share = if behind_no_cull {
                            self.result.visibility.get(&id).copied().unwrap_or(0.0)
                        } else {
                            let clipped_area = clipped.as_ref().map_or(0.0, |c| c.calc_area());
                            clipped_area / polygon.calc_area()
                        };
                        *self.result.visibility.entry(other).or_insert(0.0) += share;
                        self.cull_cells_internal(other, &narrowed, false);
                    }
                }
            }
        }
        self.on_stack.remove(&id);
    }

    /// `CEnvCell::CullCellGroupInternal`: every cell of the group is visible; the group's
    /// outside portals open viewports and its exit cells are walked.
    fn cull_cell_group(&mut self, id: u32, frustum: &Frustum, plane: f32) {
        let Some(cell) = self.cells.get(&id) else {
            return;
        };
        for shared in &cell.shared_cells {
            let Some(other) = self.cells.get(shared) else {
                continue;
            };
            let share = self.result.visibility.get(&id).copied().unwrap_or(0.0);
            *self.result.visibility.entry(*shared).or_insert(0.0) += share;
            self.on_stack.insert(*shared);
            if plane != -1.0 && culling_plane_check(self.camera, plane, other.bbox) {
                self.result.fog_culled = true;
                continue;
            }
            self.mark_visible(*shared);
            if other.draw_sky {
                self.result.draw_sky = true;
            }
        }
        let Some(group) = self.groups.get(&id) else {
            self.release_shared(id);
            return;
        };
        for &(owner, index) in &group.outside_portals {
            let Some(shared) = self.cells.get(&owner) else {
                continue;
            };
            // The portal's polygon is placed by the walked cell's matrix, as retail does.
            let world = cell.world;
            let polygon = WorkerPolygon::from_polygon(&shared.portals[index].polygon, |v| {
                world.transform_point3(v)
            });
            let viewport = frustum.from_portal(&polygon).unwrap_or(*frustum);
            self.result.outside_viewports.push(viewport);
        }
        for exit in &group.exit_cells {
            if self.cells.get(exit).is_some_and(|c| !c.no_cull) {
                self.cull_cells_internal(*exit, frustum, false);
            }
        }
        self.release_shared(id);
    }

    /// The shared cells leave the stack when the group walk ends.
    fn release_shared(&mut self, id: u32) {
        if let Some(cell) = self.cells.get(&id) {
            for shared in &cell.shared_cells {
                self.on_stack.remove(shared);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Camera;

    fn camera() -> Frustum {
        Frustum::from_camera(
            &Camera {
                eye: Vec3::ZERO,
                target: Vec3::Y,
                fov_y: 40_f32.to_radians(),
                near: 0.1,
                far: 1000.0,
            },
            1.0,
        )
    }

    /// A portal polygon in the plane `y = at`, spanning x and z by `half`, around (x, z), wound so a
    /// viewer at the origin looking along +y sees it counter-clockwise (the frustum made from it
    /// then points inward).
    fn portal_at(y: f32, x: f32, z: f32, half: f32, other: Option<u32>) -> CullPortal {
        CullPortal {
            other_cell: other,
            polygon: vec![
                Vec3::new(x - half, y, z - half),
                Vec3::new(x - half, y, z + half),
                Vec3::new(x + half, y, z + half),
                Vec3::new(x + half, y, z - half),
            ],
        }
    }

    fn room(id: u32, y0: f32, y1: f32, x0: f32, x1: f32, portals: Vec<CullPortal>) -> CullCell {
        CullCell {
            id,
            bbox: BBox::new(Vec3::new(x0, y0, -5.0), Vec3::new(x1, y1, 5.0)),
            world: Mat4::IDENTITY,
            portals,
            no_cull: false,
            draw_sky: false,
            shared_cells: Vec::new(),
            structure: None,
            inverse: Mat4::IDENTITY,
        }
    }

    fn world(cells: Vec<CullCell>) -> HashMap<u32, CullCell> {
        cells.into_iter().map(|c| (c.id, c)).collect()
    }

    fn walk(cells: &HashMap<u32, CullCell>, start: u32) -> CullResult {
        let groups = compute_cell_groups(cells);
        let mut cull = InteriorCull::new(cells, &groups, Vec3::ZERO);
        cull.cull_cells(start, &camera(), -1.0);
        cull.finish()
    }

    /// A (y 5..10) opens ahead into B (10..20) and, off to the side, into C.
    fn rooms() -> HashMap<u32, CullCell> {
        world(vec![
            room(
                0x100,
                5.0,
                10.0,
                -10.0,
                10.0,
                vec![
                    portal_at(10.0, 0.0, 0.0, 1.0, Some(0x101)),
                    portal_at(10.0, 9.0, 0.0, 0.5, Some(0x102)),
                ],
            ),
            room(
                0x101,
                10.0,
                20.0,
                -2.0,
                2.0,
                vec![portal_at(10.0, 0.0, 0.0, 1.0, Some(0x100))],
            ),
            room(
                0x102,
                10.0,
                20.0,
                8.0,
                10.0,
                vec![portal_at(10.0, 9.0, 0.0, 0.5, Some(0x100))],
            ),
        ])
    }

    #[test]
    fn the_walk_follows_portals_in_view_and_not_those_beside_it() {
        let result = walk(&rooms(), 0x100);
        assert_eq!(result.visible, vec![0x100, 0x101]);
        assert!(!result.draw_sky && !result.fog_culled);
        assert!(result.visibility[&0x101] > 0.0 && result.visibility[&0x101] <= 1.0);
        assert!(!result.visibility.contains_key(&0x102));
    }

    #[test]
    fn a_portal_to_the_outside_adds_a_viewport() {
        let mut cells = rooms();
        cells.get_mut(&0x100).unwrap().portals = vec![portal_at(10.0, 0.0, 0.0, 1.0, None)];
        cells.get_mut(&0x100).unwrap().draw_sky = true;
        let result = walk(&cells, 0x100);
        assert_eq!(result.visible, vec![0x100]);
        assert_eq!(result.outside_viewports.len(), 1);
        assert!(result.draw_sky);
        // The viewport is the portal's narrow cone: a point to its side is outside it.
        let view = result.outside_viewports[0];
        let far = Vec3::new(8.0, 30.0, 0.0);
        assert_eq!(view.check(far, far + Vec3::ONE), Relation::Outside);
        let ahead = Vec3::new(-0.5, 30.0, -0.5);
        assert_ne!(view.check(ahead, ahead + Vec3::ONE), Relation::Outside);
    }

    #[test]
    fn a_cell_out_of_the_frustum_is_not_reached() {
        let mut cells = rooms();
        // Move B behind the camera.
        let b = cells.get_mut(&0x101).unwrap();
        b.bbox = BBox::new(Vec3::new(-2.0, -20.0, -5.0), Vec3::new(2.0, -10.0, 5.0));
        assert_eq!(walk(&cells, 0x100).visible, vec![0x100]);
    }

    #[test]
    fn a_ring_of_cells_ends() {
        let cells = world(vec![
            room(
                0x100,
                5.0,
                10.0,
                -5.0,
                5.0,
                vec![portal_at(8.0, 0.0, 0.0, 1.0, Some(0x101))],
            ),
            room(
                0x101,
                5.0,
                10.0,
                -5.0,
                5.0,
                vec![portal_at(8.0, 0.0, 0.0, 1.0, Some(0x100))],
            ),
        ]);
        assert_eq!(walk(&cells, 0x100).visible, vec![0x100, 0x101]);
    }

    #[test]
    fn check_visibility_needs_a_portal_in_view() {
        let cells = world(vec![room(
            0x100,
            5.0,
            10.0,
            -10.0,
            10.0,
            vec![
                portal_at(10.0, 0.0, 0.0, 1.0, None),
                portal_at(10.0, 9.0, 0.0, 0.5, None),
            ],
        )]);
        let cell = &cells[&0x100];
        assert!(check_visibility(cell, false, &camera()));
        // A viewer inside looks through cell portals, not outside ones.
        assert!(!check_visibility(cell, true, &camera()));
        let behind = Frustum::from_camera(
            &Camera {
                eye: Vec3::ZERO,
                target: -Vec3::Y,
                fov_y: 40_f32.to_radians(),
                near: 0.1,
                far: 1000.0,
            },
            1.0,
        );
        assert!(!check_visibility(cell, false, &behind));
        let mut open = cell.clone();
        open.no_cull = true;
        assert!(check_visibility(&open, true, &camera()));
    }

    #[test]
    fn a_no_cull_group_draws_whole_and_opens_its_outside_portals() {
        let mut a = room(0x100, 5.0, 10.0, -5.0, 5.0, Vec::new());
        a.no_cull = true;
        a.shared_cells = vec![0x100, 0x101];
        let mut b = room(
            0x101,
            10.0,
            20.0,
            -5.0,
            5.0,
            vec![portal_at(20.0, 0.0, 0.0, 1.0, None)],
        );
        b.no_cull = true;
        b.shared_cells = vec![0x100, 0x101];
        let cells = world(vec![a, b]);
        let groups = compute_cell_groups(&cells);
        assert_eq!(groups[&0x100].outside_portals, vec![(0x101, 0)]);
        let mut cull = InteriorCull::new(&cells, &groups, Vec3::ZERO);
        cull.cull_cells(0x100, &camera(), -1.0);
        let result = cull.finish();
        assert_eq!(result.visible, vec![0x100, 0x101]);
        assert_eq!(result.outside_viewports.len(), 1);
    }

    #[test]
    fn the_culling_plane_drops_near_group_cells_but_not_the_start() {
        let mut a = room(0x100, 5.0, 10.0, -5.0, 5.0, Vec::new());
        a.no_cull = true;
        a.shared_cells = vec![0x100, 0x101];
        let mut b = room(0x101, 40.0, 50.0, -5.0, 5.0, Vec::new());
        b.no_cull = true;
        b.shared_cells = vec![0x100, 0x101];
        let cells = world(vec![a, b]);
        let groups = compute_cell_groups(&cells);
        let mut cull = InteriorCull::new(&cells, &groups, Vec3::ZERO);
        // 30 m of fog: every corner of the far cell is past it, so the cell is dropped.
        cull.cull_cells(0x100, &camera(), 30.0);
        let result = cull.finish();
        assert_eq!(result.visible, vec![0x100]);
        assert!(result.fog_culled);
    }
}
