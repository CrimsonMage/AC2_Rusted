//! `BSPNODE`/`BSPLEAF` traversals: a sphere goes down the side(s) of each splitting plane it
//! reaches (both when it straddles, positive first) and is tested against the leaf polygons.

use super::{BspTree, Node};
use crate::fpu::{ld, F_EPSILON};
use crate::transition::sphere_path::SpherePath;
use crate::vector::{Plane, Sphere, Vector};

/// Which children of a node a sphere reaches.
enum Side {
    Pos,
    Neg,
    Both,
}

/// `Sphere::Intersects`: a node's bounding sphere reaches the sphere at `center` of `radius`
/// (`|d|^2 - (r1 + r2)^2 < F_EPSILON`).
pub(crate) fn sphere_intersects(bounds: &Sphere, center: Vector, radius: f32) -> bool {
    let dx = ld(bounds.center.x) - ld(center.x);
    let dy = ld(bounds.center.y) - ld(center.y);
    let dz = ld(bounds.center.z) - ld(center.z);
    let reach = ld(radius) + ld(bounds.radius);
    ((dx * dx + dz * dz) + dy * dy) - reach * reach < ld(F_EPSILON)
}

/// The split every node traversal shares: `((Nz*z + Ny*y) + x*Nx) + d` against the radius less
/// `F_EPSILON`.
fn side(plane: &Plane, sphere: &Sphere) -> (Side, f64) {
    let c = sphere.center;
    let n = plane.n;
    let dist = ((ld(n.z) * ld(c.z) + ld(n.y) * ld(c.y)) + ld(c.x) * ld(n.x)) + ld(plane.d);
    let reach = ld(sphere.radius) - ld(F_EPSILON);
    let side = if !(dist < reach) {
        Side::Pos
    } else if !(dist > -reach) {
        Side::Neg
    } else {
        Side::Both
    };
    (side, dist)
}

/// What `sphere_intersects_poly` reports beyond its result: the last polygon touched (moving
/// away or not) and the contact point.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PolyHit {
    pub polygon: Option<usize>,
    pub contact: Vector,
}

impl BspTree {
    /// `BSPNODE::sphere_intersects_poly` / `BSPLEAF::sphere_intersects_poly`: a polygon the
    /// sphere hits while moving into its front.
    pub(crate) fn sphere_intersects_poly(
        &self,
        node: usize,
        sphere: &Sphere,
        movement: Vector,
        hit: &mut PolyHit,
    ) -> bool {
        match &self.nodes[node] {
            Node::Leaf {
                sphere: bounds,
                polygons,
                ..
            } => {
                if polygons.is_empty() || !sphere_intersects(bounds, sphere.center, sphere.radius) {
                    return false;
                }
                polygons.iter().any(|&index| {
                    let mut struck = false;
                    let found = self.polygons[index].pos_hits_sphere(
                        sphere,
                        movement,
                        &mut hit.contact,
                        &mut struck,
                    );
                    if struck {
                        hit.polygon = Some(index);
                    }
                    found
                })
            }
            Node::Internal {
                sphere: bounds,
                plane,
                pos,
                neg,
            } => {
                if !sphere_intersects(bounds, sphere.center, sphere.radius) {
                    return false;
                }
                let go = |child: &Option<usize>, hit: &mut PolyHit| {
                    child.is_some_and(|c| self.sphere_intersects_poly(c, sphere, movement, hit))
                };
                match side(plane, sphere).0 {
                    Side::Pos => go(pos, hit),
                    Side::Neg => go(neg, hit),
                    Side::Both => go(pos, hit) || go(neg, hit),
                }
            }
        }
    }

    /// `BSPNODE::sphere_intersects_solid` / `BSPLEAF::sphere_intersects_solid`: the sphere
    /// overlaps a polygon, or (with `center_check`) its centre lies in a solid leaf. A
    /// straddling sphere checks the centre only on the side it is on.
    pub(crate) fn sphere_intersects_solid(
        &self,
        node: usize,
        sphere: &Sphere,
        center_check: bool,
    ) -> bool {
        match &self.nodes[node] {
            Node::Leaf {
                sphere: bounds,
                solid,
                polygons,
            } => {
                if polygons.is_empty() {
                    return false;
                }
                if center_check && *solid {
                    return true;
                }
                sphere_intersects(bounds, sphere.center, sphere.radius)
                    && polygons
                        .iter()
                        .any(|&index| self.polygons[index].hits_sphere(sphere))
            }
            Node::Internal {
                sphere: bounds,
                plane,
                pos,
                neg,
            } => {
                if !sphere_intersects(bounds, sphere.center, sphere.radius) {
                    return false;
                }
                let go = |child: &Option<usize>, check: bool| {
                    child.is_some_and(|c| self.sphere_intersects_solid(c, sphere, check))
                };
                match side(plane, sphere) {
                    (Side::Pos, _) => go(pos, center_check),
                    (Side::Neg, _) => go(neg, center_check),
                    (Side::Both, dist) if dist < 0.0 => go(pos, false) || go(neg, center_check),
                    (Side::Both, _) => go(pos, center_check) || go(neg, false),
                }
            }
        }
    }

    /// `BSPNODE::find_walkable` / `BSPLEAF::find_walkable`: walkable polygons under the sphere
    /// pull it back along `movement` onto them (`adjust_sphere_to_plane`); the last one that
    /// moved it is reported.
    pub(crate) fn find_walkable(
        &self,
        node: usize,
        path: &mut SpherePath,
        sphere: &mut Sphere,
        polygon: &mut Option<usize>,
        movement: Vector,
        up: Vector,
        changed: &mut bool,
    ) {
        match &self.nodes[node] {
            Node::Leaf {
                sphere: bounds,
                polygons,
                ..
            } => {
                if polygons.is_empty() || !sphere_intersects(bounds, sphere.center, sphere.radius) {
                    return;
                }
                for &index in polygons {
                    let p = &self.polygons[index];
                    if p.walkable_hits_sphere(path, sphere, up)
                        && p.adjust_sphere_to_plane(path, sphere, movement)
                    {
                        *changed = true;
                        *polygon = Some(index);
                    }
                }
            }
            Node::Internal {
                sphere: bounds,
                plane,
                pos,
                neg,
            } => {
                if !sphere_intersects(bounds, sphere.center, sphere.radius) {
                    return;
                }
                let side = side(plane, sphere).0;
                let mut go = |child: Option<usize>, sphere: &mut Sphere| {
                    if let Some(c) = child {
                        self.find_walkable(c, path, sphere, polygon, movement, up, changed);
                    }
                };
                match side {
                    Side::Pos => go(*pos, sphere),
                    Side::Neg => go(*neg, sphere),
                    Side::Both => {
                        go(*pos, sphere);
                        go(*neg, sphere);
                    }
                }
            }
        }
    }

    /// `BSPNODE::hits_walkable` / `BSPLEAF::hits_walkable`: a walkable polygon is under the
    /// sphere (`check_small_walkable`).
    pub(crate) fn hits_walkable(
        &self,
        node: usize,
        path: &SpherePath,
        sphere: &Sphere,
        up: Vector,
    ) -> bool {
        match &self.nodes[node] {
            Node::Leaf {
                sphere: bounds,
                polygons,
                ..
            } => {
                !polygons.is_empty()
                    && sphere_intersects(bounds, sphere.center, sphere.radius)
                    && polygons.iter().any(|&index| {
                        let p = &self.polygons[index];
                        p.walkable_hits_sphere(path, sphere, up)
                            && p.check_walkable(sphere, up, true)
                    })
            }
            Node::Internal {
                sphere: bounds,
                plane,
                pos,
                neg,
            } => {
                if !sphere_intersects(bounds, sphere.center, sphere.radius) {
                    return false;
                }
                let go = |child: &Option<usize>| {
                    child.is_some_and(|c| self.hits_walkable(c, path, sphere, up))
                };
                match side(plane, sphere).0 {
                    Side::Pos => go(pos),
                    Side::Neg => go(neg),
                    Side::Both => go(pos) || go(neg),
                }
            }
        }
    }
}
