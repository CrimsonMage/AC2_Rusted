//! Interior cells (`CEnvCell`): a cell mesh placed at a position, with its physics BSP for
//! collision (`CEnvCell::find_env_collisions`), its cell BSP for containment
//! (`CCellMesh::PointInCell`) and the portals the cell search walks through
//! (`CEnvCell::FindTransitCells`, with `WorkerPolygon::Clip`).

use std::sync::Arc;

use crate::bsp::{BspTree, ContactKind, Polygon};
use crate::cells::CellSource;
use crate::fpu::{ld, st, F_EPSILON};
use crate::objects::BBox;
use crate::position::Position;
use crate::transition::collision::CONTACT_OI;
use crate::transition::sphere_path::InsertType;
use crate::transition::{Transition, TransitionState};
use crate::vector::{Plane, Vector};

/// One node of a cell's `CELL_BSP`: planes only, `pos`/`neg` children by index.
#[derive(Debug, Clone, PartialEq)]
pub struct CellBspNode {
    pub plane: Plane,
    pub pos: Option<usize>,
    pub neg: Option<usize>,
}

/// A cell mesh's containment tree (root first).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CellBsp {
    pub nodes: Vec<CellBspNode>,
}

impl CellBsp {
    /// `BSPNODE::point_inside_cell_bsp`: down the negative side while the point is more than
    /// `F_EPSILON` behind a plane, else the positive side; inside when the positive side ends.
    pub(crate) fn point_inside(&self, p: Vector) -> bool {
        let Some(mut node) = self.nodes.first() else {
            return false;
        };
        let limit = ld(st(-ld(F_EPSILON)));
        loop {
            let n = node.plane.n;
            let dist =
                ((ld(n.y) * ld(p.y) + ld(n.z) * ld(p.z)) + ld(n.x) * ld(p.x)) + ld(node.plane.d);
            let next = if dist < limit { node.neg } else { node.pos };
            match next {
                Some(index) => node = &self.nodes[index],
                None => return !(dist < limit),
            }
        }
    }
}

/// A `CCellPortal`: the portal polygon (cell space), the cell beyond it (`None` outside) and
/// the index of this portal in that cell's list (`other_portal_id`; `None` when negative).
#[derive(Debug, Clone, PartialEq)]
pub struct CellPortal {
    pub other_cell: Option<u32>,
    pub other_portal: Option<usize>,
    pub polygon: Polygon,
}

/// The collision structure of a cell mesh, shared by every cell that uses it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CellStructure {
    pub physics: Option<Arc<BspTree>>,
    pub cell_bsp: CellBsp,
    /// Portal polygons by the index `CCellPortal` names.
    pub portal_polygons: Vec<Polygon>,
}

/// A resident interior cell.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvCell {
    pub cell_id: u32,
    /// The cell's frame in its landblock (`CEnvCell` is a `Position`).
    pub position: Position,
    pub structure: Arc<CellStructure>,
    pub portals: Vec<CellPortal>,
    /// `CEnvCell::m_seenOutside`.
    pub seen_outside: bool,
    /// `CObjCell::m_stabList`: the cells visible from this one.
    pub stab_list: Vec<u32>,
    /// `CObjCell::m_staticShadowList`.
    pub objects: Vec<u32>,
    /// `CEnvCell::GetPhysicsMaterialDID` (the cell's physics material id, `None` when it has none).
    pub physics_material: Option<u64>,
}

impl EnvCell {
    /// `CEnvCell::GetTouchesOutside`: a portal opens to the outside.
    pub(crate) fn touches_outside(&self) -> bool {
        self.portals
            .iter()
            .any(|portal| portal.other_cell.is_none())
    }

    /// `CEnvCell::point_in_cell`: a point of the cell's landblock inside its cell BSP.
    pub(crate) fn point_in_cell(&self, point: Vector) -> bool {
        let local = self.position.frame.global_to_local(point);
        self.structure.cell_bsp.point_inside(local)
    }

    /// `CEnvCell::find_env_collisions`: the body against the cell's physics BSP (unit scale, as
    /// environment); a non-contact body that is stopped records an environment collision.
    pub(crate) fn find_env_collisions(
        &self,
        transition: &mut Transition,
        cells: &dyn CellSource,
    ) -> TransitionState {
        transition.sphere_path.obstruction_ethereal = false;
        let Some(tree) = self.structure.physics.as_deref() else {
            return TransitionState::Ok;
        };
        let one = Vector::new(1.0, 1.0, 1.0);
        transition
            .sphere_path
            .cache_localspace_sphere(&self.position, one);
        let ts = if transition.sphere_path.insert_type == InsertType::InitialPlacement {
            tree.placement_insert(transition)
        } else {
            let kind = ContactKind {
                environment: true,
                ..ContactKind::default()
            };
            tree.find_collisions(transition, one, one, kind, cells)
        };
        if ts != TransitionState::Ok && transition.object_info.state & CONTACT_OI == 0 {
            transition.collision_info.collided_with_environment = true;
            if !transition.collision_info.collision_normal_valid {
                transition
                    .collision_info
                    .set_collision_normal(Vector::new(0.0, 0.0, 1.0));
            }
        }
        ts
    }

    /// Whether `portal`, moved into the landblock by the cell's frame, reaches into `bounds`:
    /// near enough to the box centre (`center` in cell space, `radius` its half diagonal) and
    /// not clipped away by any of the box's planes (`WorkerPolygon::Clip`, 0.01 tolerance).
    pub(crate) fn portal_reaches(
        &self,
        portal: &Polygon,
        bounds: &BBox,
        center: Vector,
        radius: f32,
    ) -> bool {
        let n = portal.plane.n;
        let dist = (((ld(center.y) * ld(n.y) + ld(center.z) * ld(n.z)) + ld(center.x) * ld(n.x))
            + ld(portal.plane.d))
        .abs();
        if !(dist < ld(F_EPSILON) + ld(radius)) {
            return false;
        }
        let m = self.position.frame.m.map(ld);
        let o = self.position.frame.origin;
        let mut clipper: Vec<Vector> = portal
            .vertices
            .iter()
            .map(|v| {
                let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
                Vector::new(
                    st(((z * m[6] + y * m[3]) + x * m[0]) + ld(o.x)),
                    st(((z * m[7] + x * m[1]) + y * m[4]) + ld(o.y)),
                    st(((y * m[5] + z * m[8]) + x * m[2]) + ld(o.z)),
                )
            })
            .collect();
        (0..6).all(|i| {
            let plane = bbox_plane(bounds, i);
            let flipped = Plane::new(
                Vector::new(st(-ld(plane.n.x)), st(-ld(plane.n.y)), st(-ld(plane.n.z))),
                st(-ld(plane.d)),
            );
            clip(&mut clipper, &flipped, 0.01) != Side::Negative
        })
    }
}

/// `BBox::GetPlane`: the outward planes of the box's faces.
fn bbox_plane(b: &BBox, index: usize) -> Plane {
    let v = Vector::new;
    match index {
        0 => Plane::new(v(-1.0, 0.0, 0.0), b.min.x),
        1 => Plane::new(v(1.0, 0.0, 0.0), -b.max.x),
        2 => Plane::new(v(0.0, -1.0, 0.0), b.min.y),
        3 => Plane::new(v(0.0, 1.0, 0.0), -b.max.y),
        4 => Plane::new(v(0.0, 0.0, 1.0), -b.max.z),
        _ => Plane::new(v(0.0, 0.0, -1.0), b.min.z),
    }
}

/// `Sidedness`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Positive,
    Negative,
    InPlane,
    Crossing,
}

/// `WorkerPolygon::Clip` with `i = 1`: keep the part of the polygon on the plane's positive side
/// (vertices within `alpha` of it count as on it).
fn clip(vertices: &mut Vec<Vector>, plane: &Plane, alpha: f32) -> Side {
    let n = plane.n;
    let mut dists = Vec::with_capacity(vertices.len() + 1);
    let mut sides = Vec::with_capacity(vertices.len() + 1);
    let mut counts = [0_usize; 3];
    for v in vertices.iter() {
        let dist = ((ld(v.z) * ld(n.z) + ld(v.y) * ld(n.y)) + ld(n.x) * ld(v.x)) + ld(plane.d);
        dists.push(st(dist));
        let side = if !(dist <= ld(alpha)) {
            0
        } else if dist < -ld(alpha) {
            1
        } else {
            2
        };
        counts[side] += 1;
        sides.push(side);
    }
    if counts[0] == 0 {
        return if counts[1] == 0 {
            Side::InPlane
        } else {
            Side::Negative
        };
    }
    if counts[1] == 0 {
        return Side::Positive;
    }
    let count = vertices.len();
    let mut kept = Vec::with_capacity(count + 2);
    for i in 0..count {
        let next = (i + 1) % count;
        let v = vertices[i];
        if sides[i] == 2 {
            kept.push(v);
            continue;
        }
        if sides[i] == 0 {
            kept.push(v);
        }
        if sides[i] != sides[next] && sides[next] != 2 {
            let w = vertices[next];
            let t = ld(st(ld(dists[i]) / (ld(dists[i]) - ld(dists[next]))));
            let dx = ld(st(ld(w.x) - ld(v.x)));
            let dy = ld(w.y) - ld(v.y);
            let dz = ld(w.z) - ld(v.z);
            kept.push(Vector::new(
                st(ld(st(dx * t)) + ld(v.x)),
                st(ld(st(dy * t)) + ld(v.y)),
                st(ld(st(dz * t)) + ld(v.z)),
            ));
        }
    }
    *vertices = kept;
    Side::Crossing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping_keeps_the_positive_part() {
        let mut square = vec![
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(2.0, 0.0, 0.0),
            Vector::new(2.0, 2.0, 0.0),
            Vector::new(0.0, 2.0, 0.0),
        ];
        let x_above_one = Plane::new(Vector::new(1.0, 0.0, 0.0), -1.0);
        assert_eq!(clip(&mut square, &x_above_one, 0.01), Side::Crossing);
        assert!(square.iter().all(|v| v.x >= 1.0 - 1e-6), "{square:?}");
        assert_eq!(square.len(), 4);
        let x_above_five = Plane::new(Vector::new(1.0, 0.0, 0.0), -5.0);
        assert_eq!(clip(&mut square, &x_above_five, 0.01), Side::Negative);
    }

    #[test]
    fn the_cell_bsp_holds_points_in_front_of_every_plane() {
        // x >= 0 and y >= 0.
        let tree = CellBsp {
            nodes: vec![
                CellBspNode {
                    plane: Plane::new(Vector::new(1.0, 0.0, 0.0), 0.0),
                    pos: Some(1),
                    neg: None,
                },
                CellBspNode {
                    plane: Plane::new(Vector::new(0.0, 1.0, 0.0), 0.0),
                    pos: None,
                    neg: None,
                },
            ],
        };
        assert!(tree.point_inside(Vector::new(1.0, 1.0, 0.0)));
        assert!(tree.point_inside(Vector::new(-0.0001, 1.0, 0.0)));
        assert!(!tree.point_inside(Vector::new(-0.01, 1.0, 0.0)));
        assert!(!tree.point_inside(Vector::new(1.0, -0.01, 0.0)));
    }
}
