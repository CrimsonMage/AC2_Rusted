//! The cells a transition collides with: land cells and their terrain polygons
//! (`CLandCell::m_polygons` as `CLandBlockMesh::ConstructPolygons` builds them), the outdoor
//! cell search (`CObjCell::FindCells` -> `CLandCell::FindOutsideCells`) and the terrain contact
//! (`CLandCell::find_env_collisions`).

use crate::envcell::EnvCell;
use crate::fpu::{ld, st, F_EPSILON};
use crate::landdefs::{get_block_offset, is_outside, POLYS_PER_LANDCELL, SQUARE_LENGTH};
use crate::objects::StaticObject;
use crate::position::offset_between;
use crate::transition::collision::PREDETERMINED_TRANSITION_OI;
use crate::transition::{Transition, TransitionState};
use crate::vector::{Plane, Sphere, Vector};

/// `LandDefs::half_square_length`: half of `SQUARE_LENGTH`.
const HALF_SQUARE_LENGTH: f32 = SQUARE_LENGTH * 0.5;

/// One terrain triangle in its landblock's frame (`CPolygon` with three vertices).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainPolygon {
    pub vertices: [Vector; 3],
    pub plane: Plane,
}

impl TerrainPolygon {
    /// A triangle with its plane from `CPolygon::MakePlane`: the cross product of the fan edges
    /// (`(v1 - v0) x (v2 - v0)`, first two terms stored as `float`), `D3DXVec3Normalize`d, and
    /// `d` from the mean of `N . v`.
    pub fn new(vertices: [Vector; 3]) -> Self {
        let [v0, v1, v2] = vertices;
        let ax = ld(v2.x) - ld(v0.x);
        let ay = ld(v2.y) - ld(v0.y);
        let az = ld(v2.z) - ld(v0.z);
        let bx = ld(st(ld(v1.x) - ld(v0.x)));
        let by = ld(st(ld(v1.y) - ld(v0.y)));
        let bz = ld(v1.z) - ld(v0.z);
        let nx = st(ld(st(by * az - bz * ay)) + 0.0);
        let ny = st(ld(st(bz * ax - az * bx)) + 0.0);
        let nz = st((ay * bx - by * ax) + 0.0);
        let n = Vector::new(nx, ny, nz).d3dx_normalize();
        let mut sum = 0.0;
        for v in vertices {
            sum += (ld(n.z) * ld(v.z) + ld(n.y) * ld(v.y)) + ld(n.x) * ld(v.x);
        }
        let d = st(-(sum / 3.0));
        Self {
            vertices,
            plane: Plane::new(n, d),
        }
    }

    /// `CPolygon::point_in_poly2D` with `POSITIVE` sidedness: inside when no edge has the point
    /// more than `F_EPSILON` on its outer side. Edges run from the last vertex back to the first.
    pub(crate) fn point_in_poly_2d(&self, point: Vector) -> bool {
        let (px, py) = (ld(point.x), ld(point.y));
        let mut prev = self.vertices[0];
        for &cur in self.vertices.iter().rev() {
            let a = ld(prev.y) - ld(cur.y);
            let b = ld(cur.x) - ld(prev.x);
            let c = -(a * ld(cur.x)) - b * ld(cur.y);
            let side = (c + a * px) + b * py;
            if side > ld(F_EPSILON) {
                return false;
            }
            prev = cur;
        }
        true
    }
}

/// A cell's two shadow lists (`CObjCell::m_staticShadowList`, `m_dynamicShadowList`): object ids,
/// kept apart by the object's static test (`CPhysicsObj::is_static`, vtable +0x234), so every walk
/// visits the static list first and the dynamic list second.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShadowLists {
    /// `CObjCell::m_staticShadowList`.
    pub static_list: Vec<u32>,
    /// `CObjCell::m_dynamicShadowList`.
    pub dynamic_list: Vec<u32>,
}

impl ShadowLists {
    /// `CObjCell::AddShadowObject`: the object goes on the static list when it is static, else on
    /// the dynamic list.
    pub fn add_shadow_object(&mut self, id: u32, is_static: bool) {
        if is_static {
            self.static_list.push(id);
        } else {
            self.dynamic_list.push(id);
        }
    }

    /// `CObjCell::RemoveShadowObject`: the first entry of the object's list takes the last
    /// entry's place (order is not kept); nothing happens when it is not listed.
    pub fn remove_shadow_object(&mut self, id: u32, is_static: bool) {
        let list = if is_static {
            &mut self.static_list
        } else {
            &mut self.dynamic_list
        };
        if let Some(index) = list.iter().position(|&other| other == id) {
            list.swap_remove(index);
        }
    }

    /// Every listed object: the static list, then the dynamic list.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.static_list
            .iter()
            .chain(self.dynamic_list.iter())
            .copied()
    }

    /// Whether `id` is on either list.
    pub fn contains(&self, id: &u32) -> bool {
        self.static_list.contains(id) || self.dynamic_list.contains(id)
    }

    /// Whether both lists are empty.
    pub fn is_empty(&self) -> bool {
        self.static_list.is_empty() && self.dynamic_list.is_empty()
    }
}

/// A resident land cell (`CLandCell`): its id, the terrain polygons over it, in slot order
/// (`(vertex_per_cell * ix + iy) * 2` and `+1`; a slot over a landscape hole is empty), the
/// objects shadowing it (`CObjCell::m_staticShadowList`, `m_dynamicShadowList`) and the interior cells whose
/// outside portals it touches (`CObjCell::m_touchingCells`), and whether its landblock can be
/// entered (`CLandBlockData::m_bPassable`).
#[derive(Debug, Clone, PartialEq)]
pub struct LandCell {
    pub cell_id: u32,
    /// `CLandBlockData::m_bPassable` of the cell's landblock (1 until `CacheNearValues` says
    /// otherwise).
    pub passable: bool,
    pub polygons: [Option<TerrainPolygon>; POLYS_PER_LANDCELL],
    pub objects: ShadowLists,
    pub touching: Vec<u32>,
}

impl LandCell {
    /// `CLandCell::find_terrain_poly`: the first polygon whose 2D outline holds `origin`.
    pub(crate) fn find_terrain_poly(&self, origin: Vector) -> Option<&TerrainPolygon> {
        self.polygons
            .iter()
            .flatten()
            .find(|polygon| polygon.point_in_poly_2d(origin))
    }

    /// The cell's position origin (`CObjCell::m_position`) in its landblock:
    /// `CLandBlockMesh::ConstructPolygons` puts it at the cell's centre,
    /// `(2 * cell + 1) * LandDefs::half_square_length` on each axis.
    pub(crate) fn origin(&self) -> Vector {
        let local = (self.cell_id & 0xFFFF).wrapping_sub(1);
        let (cell_x, cell_y) = ((local >> 3) & 7, local & 7);
        let centre = |cell: u32| st(ld(((2 * cell + 1) as i32) as f32) * ld(HALF_SQUARE_LENGTH));
        Vector::new(centre(cell_x), centre(cell_y), 0.0)
    }

    /// `CLandCell::handle_move_restriction`: turns the mover back off an impassable block. The
    /// offset from the cell's position to the path's `curr_pos` picks the side (`y` against
    /// plus and minus `half_square_length` first, then `x`), the path's offset from `curr_pos`
    /// to `check_pos` loses its component along that side's axis, `check_pos` moves by it, and
    /// the collision normal points back (0, -1, 0), (0, 1, 0), (-1, 0, 0) or (1, 0, 0).
    /// Always true.
    pub(crate) fn handle_move_restriction(&self, transition: &mut Transition) -> bool {
        let path = &mut transition.sphere_path;
        let cell = self.origin();
        let offset = offset_between(
            self.cell_id,
            cell,
            path.curr_pos.cell_id,
            path.curr_pos.frame.origin,
        );
        let mut path_offset = offset_between(
            path.curr_pos.cell_id,
            path.curr_pos.frame.origin,
            path.check_pos.cell_id,
            path.check_pos.frame.origin,
        );
        let half = HALF_SQUARE_LENGTH;
        let normal = if offset.y < -half {
            path_offset.y = -path_offset.y;
            Vector::new(0.0, -1.0, 0.0)
        } else if offset.y > half {
            path_offset.y = -path_offset.y;
            Vector::new(0.0, 1.0, 0.0)
        } else {
            path_offset.x = -path_offset.x;
            Vector::new(if offset.x < -half { -1.0 } else { 1.0 }, 0.0, 0.0)
        };
        path.add_offset_to_check_pos(path_offset);
        transition.collision_info.set_collision_normal(normal);
        true
    }

    /// `CLandCell::point_in_cell`: whether a point of this cell's landblock lies in this cell.
    pub(crate) fn point_in_cell(&self, point: Vector) -> bool {
        let x = (ld(point.x) / ld(SQUARE_LENGTH)).floor() as i32 as u32;
        let y = (ld(point.y) / ld(SQUARE_LENGTH)).floor() as i32 as u32;
        let local = ((y & 7) + 1 + ((x & 7) << 3)) & 0xFFFF;
        (local | (self.cell_id & 0xFFFF_0000)) == self.cell_id
    }
}

/// The resident world a transition collides against (`CObjCell::GetVisible`).
pub trait CellSource {
    /// The land cell `cell_id` (`0xXXYY00NN`, `NN` in `1..=64`) when its landblock is resident.
    fn land_cell(&self, cell_id: u32) -> Option<&LandCell>;

    /// The interior cell `cell_id` (`0xXXYYNNNN`, `NNNN` from `0x0100`) when resident.
    fn env_cell(&self, _cell_id: u32) -> Option<&EnvCell> {
        None
    }

    /// A static object a cell's shadow list names.
    fn object(&self, _id: u32) -> Option<&StaticObject> {
        None
    }
}

/// A resident `CObjCell`: a land cell or an interior cell.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ObjCell<'a> {
    Land(&'a LandCell),
    Env(&'a EnvCell),
}

impl<'a> ObjCell<'a> {
    /// `CObjCell::GetVisible`: the resident cell `cell_id`.
    pub(crate) fn get(cells: &'a dyn CellSource, cell_id: u32) -> Option<Self> {
        if is_outside(cell_id) {
            cells.land_cell(cell_id).map(ObjCell::Land)
        } else {
            cells.env_cell(cell_id).map(ObjCell::Env)
        }
    }

    pub(crate) fn objects(&self) -> &'a ShadowLists {
        match self {
            ObjCell::Land(cell) => &cell.objects,
            ObjCell::Env(cell) => &cell.objects,
        }
    }
}

/// `CEnvCell::find_collisions` (land cells run it too): the cell's environment -- terrain
/// (`CLandCell::find_env_collisions`) or cell BSP (`CEnvCell::find_env_collisions`) -- unless
/// the transition is predetermined, then its shadow objects (`CObjCell::find_obj_collisions`).
pub(crate) fn find_collisions(
    transition: &mut Transition,
    cell: ObjCell<'_>,
    cells: &dyn CellSource,
) -> TransitionState {
    if transition.object_info.state & PREDETERMINED_TRANSITION_OI == 0 {
        let ts = match cell {
            ObjCell::Land(land) => find_env_collisions(transition, land),
            ObjCell::Env(env) => env.find_env_collisions(transition, cells),
        };
        if ts != TransitionState::Ok {
            return ts;
        }
    }
    transition.find_obj_collisions(cell.objects(), cells)
}

/// `CLandCell::find_env_collisions`: on an impassable landblock (`CLandBlockData::m_bPassable`
/// clear) `handle_move_restriction` runs and the transition is `Adjusted` at once. Otherwise the
/// terrain polygon under the path's low point, moved into the cell's landblock, and
/// `OBJECTINFO::validate_walkable` of the first global sphere against its plane as landscape.
fn find_env_collisions(transition: &mut Transition, cell: &LandCell) -> TransitionState {
    if !cell.passable {
        cell.handle_move_restriction(transition);
        return TransitionState::Adjusted;
    }
    let path = &transition.sphere_path;
    let offset = get_block_offset(path.check_pos.cell_id, cell.cell_id);
    let low = path.global_low_point;
    let local = Vector::new(
        st(ld(low.x) - ld(offset.x)),
        st(ld(low.y) - ld(offset.y)),
        st(ld(low.z) - ld(offset.z)),
    );
    let Some(polygon) = cell.find_terrain_poly(local) else {
        return TransitionState::Ok;
    };
    let sphere = path.global_sphere[0];
    let check = Sphere::new(
        Vector::new(
            st(ld(sphere.center.x) - ld(offset.x)),
            st(ld(sphere.center.y) - ld(offset.y)),
            st(ld(sphere.center.z) - ld(offset.z)),
        ),
        sphere.radius,
    );
    let object_info = transition.object_info;
    object_info.validate_walkable(
        check,
        polygon.plane,
        false,
        0.0,
        &mut transition.sphere_path,
        &mut transition.collision_info,
        cell.cell_id,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_lists_split_by_the_static_test_and_walk_static_first() {
        let mut lists = ShadowLists::default();
        lists.add_shadow_object(7, false);
        lists.add_shadow_object(3, true);
        lists.add_shadow_object(9, true);
        assert_eq!(lists.static_list, vec![3, 9]);
        assert_eq!(lists.dynamic_list, vec![7]);
        assert_eq!(lists.iter().collect::<Vec<_>>(), vec![3, 9, 7]);
        // Removal looks only on the object's own list, and the last entry takes the slot.
        lists.remove_shadow_object(7, true);
        assert_eq!(lists.dynamic_list, vec![7]);
        lists.remove_shadow_object(3, true);
        assert_eq!(lists.static_list, vec![9]);
        lists.remove_shadow_object(7, false);
        lists.remove_shadow_object(9, true);
        assert!(lists.is_empty());
    }

    fn flat(z: f32) -> TerrainPolygon {
        TerrainPolygon::new([
            Vector::new(0.0, 0.0, z),
            Vector::new(10.0, 0.0, z),
            Vector::new(0.0, 10.0, z),
        ])
    }

    #[test]
    fn make_plane_faces_up_for_counter_clockwise_triangles() {
        let polygon = flat(7.0);
        assert_eq!(polygon.plane, Plane::new(Vector::new(0.0, 0.0, 1.0), -7.0));
        let slope = TerrainPolygon::new([
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(10.0, 0.0, 10.0),
            Vector::new(0.0, 10.0, 0.0),
        ]);
        let n = slope.plane.n;
        assert!((n.x + std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6 && n.y == 0.0);
    }

    #[test]
    fn point_in_poly_uses_the_epsilon_band() {
        let polygon = flat(0.0);
        assert!(polygon.point_in_poly_2d(Vector::new(2.0, 2.0, 0.0)));
        assert!(polygon.point_in_poly_2d(Vector::new(5.0, 5.0, 0.0)));
        assert!(!polygon.point_in_poly_2d(Vector::new(5.1, 5.1, 0.0)));
        assert!(polygon.point_in_poly_2d(Vector::new(-0.00001, 3.0, 0.0)));
        assert!(!polygon.point_in_poly_2d(Vector::new(-0.01, 3.0, 0.0)));
    }

    #[test]
    fn point_in_cell_matches_the_cell_grid() {
        let cell = LandCell {
            cell_id: 0x7F7F_000A,
            passable: true,
            polygons: [None; POLYS_PER_LANDCELL],
            objects: ShadowLists::default(),
            touching: Vec::new(),
        };
        // Cell 0x0A is (x 1, y 1): 20..40 on both axes.
        assert!(cell.point_in_cell(Vector::new(25.0, 39.9, 0.0)));
        assert!(!cell.point_in_cell(Vector::new(25.0, 40.0, 0.0)));
    }

    /// A cell-0x0A land cell (centre (30, 30)) with a mover going from `curr` to `check`.
    fn restricted(curr: Vector, check: Vector) -> (LandCell, Transition) {
        let cell = LandCell {
            cell_id: 0x7F7F_000A,
            passable: false,
            polygons: [None; POLYS_PER_LANDCELL],
            objects: ShadowLists::default(),
            touching: Vec::new(),
        };
        let mut transition = Transition::default();
        let path = &mut transition.sphere_path;
        path.num_sphere = 1;
        path.curr_pos.cell_id = 0x7F7F_000A;
        path.curr_pos.frame.origin = curr;
        path.check_pos.cell_id = 0x7F7F_000A;
        path.check_pos.frame.origin = check;
        path.global_sphere[0] = Sphere::new(check, 0.5);
        path.global_low_point = check;
        path.cell_array_valid = true;
        (cell, transition)
    }

    #[test]
    fn land_cell_origin_is_the_cell_centre() {
        let (cell, _) = restricted(Vector::ZERO, Vector::ZERO);
        assert_eq!(cell.origin(), Vector::new(30.0, 30.0, 0.0));
        let corner = LandCell {
            cell_id: 0x7F7F_0040,
            ..cell
        };
        assert_eq!(corner.origin(), Vector::new(150.0, 150.0, 0.0));
    }

    #[test]
    fn move_restriction_south_side_flips_y_and_faces_minus_y() {
        // Offset from the cell centre is (0, -15, 0): below -half_square_length.
        let (cell, mut t) = restricted(Vector::new(30.0, 15.0, 1.0), Vector::new(32.0, 14.0, 2.0));
        assert!(cell.handle_move_restriction(&mut t));
        // Path offset (2, -1, 1) loses its y sign: (2, 1, 1).
        assert_eq!(
            t.sphere_path.check_pos.frame.origin,
            Vector::new(34.0, 15.0, 3.0)
        );
        assert_eq!(
            t.sphere_path.global_sphere[0].center,
            Vector::new(34.0, 15.0, 3.0)
        );
        assert_eq!(t.sphere_path.global_low_point, Vector::new(34.0, 15.0, 3.0));
        assert!(!t.sphere_path.cell_array_valid);
        assert!(t.collision_info.collision_normal_valid);
        assert_eq!(
            t.collision_info.collision_normal,
            Vector::new(0.0, -1.0, 0.0)
        );
    }

    #[test]
    fn move_restriction_north_side_flips_y_and_faces_plus_y() {
        let (cell, mut t) = restricted(Vector::new(30.0, 45.0, 0.0), Vector::new(31.0, 46.0, 0.0));
        assert!(cell.handle_move_restriction(&mut t));
        assert_eq!(
            t.sphere_path.check_pos.frame.origin,
            Vector::new(32.0, 45.0, 0.0)
        );
        assert!(!t.sphere_path.cell_array_valid);
        assert_eq!(
            t.collision_info.collision_normal,
            Vector::new(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn move_restriction_west_and_east_sides_flip_x() {
        // Inside the y band, x below -half: west, normal (-1, 0, 0).
        let (cell, mut t) = restricted(Vector::new(15.0, 30.0, 0.0), Vector::new(14.0, 33.0, 0.0));
        assert!(cell.handle_move_restriction(&mut t));
        assert_eq!(
            t.sphere_path.check_pos.frame.origin,
            Vector::new(15.0, 36.0, 0.0)
        );
        assert!(!t.sphere_path.cell_array_valid);
        assert_eq!(
            t.collision_info.collision_normal,
            Vector::new(-1.0, 0.0, 0.0)
        );
        // Otherwise east, normal (1, 0, 0); exactly -half counts as east.
        let (cell, mut t) = restricted(Vector::new(20.0, 30.0, 0.0), Vector::new(21.0, 30.0, 0.0));
        assert!(cell.handle_move_restriction(&mut t));
        assert_eq!(
            t.sphere_path.check_pos.frame.origin,
            Vector::new(20.0, 30.0, 0.0)
        );
        assert_eq!(
            t.collision_info.collision_normal,
            Vector::new(1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn impassable_block_restricts_the_move_and_a_passable_one_does_not() {
        let (mut cell, mut t) =
            restricted(Vector::new(30.0, 30.0, 0.0), Vector::new(31.0, 30.0, 0.0));
        assert_eq!(
            find_env_collisions(&mut t, &cell),
            TransitionState::Adjusted
        );
        assert!(!t.sphere_path.cell_array_valid);
        assert_eq!(
            t.sphere_path.check_pos.frame.origin,
            Vector::new(30.0, 30.0, 0.0)
        );
        // A passable block with no terrain polygon leaves the transition alone.
        cell.passable = true;
        let (_, mut t) = restricted(Vector::new(30.0, 30.0, 0.0), Vector::new(31.0, 30.0, 0.0));
        assert_eq!(find_env_collisions(&mut t, &cell), TransitionState::Ok);
        assert!(t.sphere_path.cell_array_valid);
        assert!(!t.collision_info.collision_normal_valid);
        assert_eq!(
            t.sphere_path.check_pos.frame.origin,
            Vector::new(31.0, 30.0, 0.0)
        );
    }

    #[test]
    fn terrain_carries_block_passability_to_its_land_cells() {
        let mut terrain = crate::terrain::Terrain::new();
        let heights = vec![0.0; 17 * 17];
        terrain.insert_heights(0x7F7F_0001, &heights, |_, _| false);
        terrain.insert_heights(0x7F80_0001, &heights, |_, _| false);
        assert!(terrain.land_cell(0x7F7F_0001).unwrap().passable);
        terrain.set_block_passable(0x7F7F_FFFF, false);
        assert!(!terrain.land_cell(0x7F7F_0001).unwrap().passable);
        assert!(!terrain.land_cell(0x7F7F_0040).unwrap().passable);
        assert!(terrain.land_cell(0x7F80_0001).unwrap().passable);
    }
}
