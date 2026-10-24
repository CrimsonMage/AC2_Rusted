//! The cells a transition collides with: land cells and their terrain polygons
//! (`CLandCell::m_polygons` as `CLandBlockMesh::ConstructPolygons` builds them), the outdoor
//! cell search (`CObjCell::FindCells` -> `CLandCell::FindOutsideCells`) and the terrain contact
//! (`CLandCell::find_env_collisions`).

use crate::envcell::EnvCell;
use crate::fpu::{ld, st, F_EPSILON};
use crate::landdefs::{get_block_offset, is_outside, POLYS_PER_LANDCELL, SQUARE_LENGTH};
use crate::objects::StaticObject;
use crate::transition::collision::PREDETERMINED_TRANSITION_OI;
use crate::transition::{Transition, TransitionState};
use crate::vector::{Plane, Sphere, Vector};

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

/// A resident land cell (`CLandCell`): its id, the terrain polygons over it, in slot order
/// (`(vertex_per_cell * ix + iy) * 2` and `+1`; a slot over a landscape hole is empty), the
/// static objects shadowing it (`CObjCell::m_staticShadowList`) and the interior cells whose
/// outside portals it touches (`CObjCell::m_touchingCells`).
#[derive(Debug, Clone, PartialEq)]
pub struct LandCell {
    pub cell_id: u32,
    pub polygons: [Option<TerrainPolygon>; POLYS_PER_LANDCELL],
    pub objects: Vec<u32>,
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

    pub(crate) fn objects(&self) -> &'a [u32] {
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

/// `CLandCell::find_env_collisions`: the terrain polygon under the path's low point, moved into
/// the cell's landblock, and `OBJECTINFO::validate_walkable` of the first global sphere against
/// its plane as landscape.
fn find_env_collisions(transition: &mut Transition, cell: &LandCell) -> TransitionState {
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
            polygons: [None; POLYS_PER_LANDCELL],
            objects: Vec::new(),
            touching: Vec::new(),
        };
        // Cell 0x0A is (x 1, y 1): 20..40 on both axes.
        assert!(cell.point_in_cell(Vector::new(25.0, 39.9, 0.0)));
        assert!(!cell.point_in_cell(Vector::new(25.0, 40.0, 0.0)));
    }
}
