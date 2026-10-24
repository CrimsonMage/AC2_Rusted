//! The outdoor line of sight: `CLandCell::Trace`, `CLandBlock::TraceRayCrossBlock` with
//! `LandDefs::BlockRayClip` from block to block, `CLandBlock::TraceRay` over the land cells under
//! the ray, and `CLandCell::TraceRay` (terrain polygons, shadow objects, then the outside portals
//! of the interior cells touching the cell).

use super::env::trace_polygon;
use super::{sub, LosParameters, LosTrace, Ray, RayIntersection};
use crate::cells::{LandCell, ObjCell};
use crate::fpu::{ld, st, F_EPSILON};
use crate::landdefs::{
    adjust_to_outside, blockid_to_lcoord, is_outside, lcoord_to_gid, BLOCK_LENGTH, LBLOCK_SIDE,
    SQUARE_LENGTH,
};
use crate::position::offset_between;
use crate::vector::Vector;

/// `DirectionEnum` values `BlockRayClip` produces.
const NORTH: u32 = 1;
const SOUTH: u32 = 2;
const EAST: u32 = 3;
const WEST: u32 = 4;
const NORTHWEST: u32 = 5;
const SOUTHWEST: u32 = 6;
const NORTHEAST: u32 = 7;
const SOUTHEAST: u32 = 8;

/// `LandDefs::GetBlockOffset(DirectionEnum, long*, long*)`: the neighbour's step in blocks.
fn direction_step(direction: u32) -> (i32, i32) {
    match direction {
        NORTH => (0, 1),
        SOUTH => (0, -1),
        EAST => (1, 0),
        WEST => (-1, 0),
        NORTHWEST => (-1, 1),
        SOUTHWEST => (-1, -1),
        NORTHEAST => (1, 1),
        SOUTHEAST => (1, -1),
        _ => (0, 0),
    }
}

/// `LandDefs::GetBlockID(ulong, DirectionEnum)`: the neighbouring block's id (`0xXXYYFFFF`), 0
/// off the map.
fn neighbour_block(block_id: u32, direction: u32) -> u32 {
    let Some((x, y)) = blockid_to_lcoord(block_id) else {
        return 0;
    };
    let (dx, dy) = direction_step(direction);
    let gid = lcoord_to_gid(x + LBLOCK_SIDE * dx, y + LBLOCK_SIDE * dy);
    if gid == 0 {
        0
    } else {
        (gid & 0xFFFF_0000) | 0xFFFF
    }
}

/// `LandDefs::BlockRayClip`: when the ray leaves its block, the part inside it, the rest moved
/// into the neighbour's frame (start kept inside that block), and the direction of the
/// neighbour. `None` when the ray stays in the block, starts outside it, or turns back in at a
/// corner.
fn block_ray_clip(ray: &Ray) -> Option<(Ray, Ray, u32)> {
    let p = ray.pt;
    let end = ray.end_stored_yz();
    let eps = ld(F_EPSILON);
    let low = ld(st(-eps));
    let high = ld(st(eps + ld(BLOCK_LENGTH)));
    if ld(p.x) < low || ld(p.x) > high || ld(p.y) < low || ld(p.y) > high {
        return None;
    }
    let mut tx = 1.0_f64;
    if ld(end.x) < low {
        tx = ld(p.x) / (ld(p.x) - ld(end.x));
    } else if ld(end.x) > high {
        tx = (ld(BLOCK_LENGTH) - ld(p.x)) / (ld(end.x) - ld(p.x));
    }
    let mut ty = 1.0_f64;
    if ld(end.y) < low {
        ty = ld(p.y) / (ld(p.y) - ld(end.y));
    } else if ld(end.y) > high {
        ty = (ld(BLOCK_LENGTH) - ld(p.y)) / (ld(end.y) - ld(p.y));
    }
    let limit = 1.0 - eps;
    if limit <= tx && limit <= ty {
        return None;
    }
    // The nearer exit: x's fraction is stored as `float`, y's stays on the x87 stack.
    let clip = if tx <= ty {
        let t = ld(st(tx));
        let dz = st(ld(end.z) - ld(p.z));
        let cx = st((ld(end.x) - ld(p.x)) * t);
        let cy = st((ld(end.y) - ld(p.y)) * t);
        Vector::new(
            st(ld(cx) + ld(p.x)),
            st(ld(cy) + ld(p.y)),
            st(ld(dz) * t + ld(p.z)),
        )
    } else {
        let d = sub(end, p);
        let cx = st(ld(d.x) * ty);
        let cy = st(ld(d.y) * ty);
        Vector::new(
            st(ld(cx) + ld(p.x)),
            st(ld(cy) + ld(p.y)),
            st(ty * ld(d.z) + ld(p.z)),
        )
    };
    let inside = Ray::line_segment(p, clip);
    let direction = exit_direction(clip, ray.dir)?;
    let (dx, dy) = direction_step(direction);
    let offset = Vector::new(
        st(f64::from(dx) * ld(BLOCK_LENGTH)),
        st(f64::from(dy) * ld(BLOCK_LENGTH)),
        0.0,
    );
    let mut start = sub(clip, offset);
    move_vector_to_be_in_block(&mut start);
    let rest = Ray::line_segment(start, sub(end, offset));
    Some((inside, rest, direction))
}

/// The neighbour `BlockRayClip` steps into from the exit point `clip`: by the edge (or corner)
/// the point lies within `F_EPSILON` of and the ray's heading there.
fn exit_direction(clip: Vector, dir: Vector) -> Option<u32> {
    let eps = ld(F_EPSILON);
    let (x, y) = (ld(clip.x), ld(clip.y));
    let (west, south) = (dir.x < 0.0, dir.y < 0.0);
    if x < eps {
        if y < eps {
            return match (west, south) {
                (false, false) => None,
                (false, true) => Some(SOUTH),
                (true, false) => Some(WEST),
                (true, true) => Some(SOUTHWEST),
            };
        }
        // `block_length - F_EPSILON` stays on the x87 stack here.
        if ld(BLOCK_LENGTH) - eps < y {
            return match (west, south) {
                (false, false) => Some(NORTH),
                (false, true) => None,
                (true, true) => Some(WEST),
                (true, false) => Some(NORTHWEST),
            };
        }
        return Some(WEST);
    }
    let high = ld(st(ld(BLOCK_LENGTH) - eps));
    if x > high {
        if y < eps {
            return match (west, south) {
                (true, false) => None,
                (true, true) => Some(SOUTH),
                (false, false) => Some(EAST),
                (false, true) => Some(SOUTHEAST),
            };
        }
        if y <= high {
            return Some(EAST);
        }
        return match (west, south) {
            (true, false) => Some(NORTH),
            (true, true) => None,
            (false, true) => Some(EAST),
            (false, false) => Some(NORTHEAST),
        };
    }
    if y < eps {
        return Some(SOUTH);
    }
    (y > high).then_some(NORTH)
}

/// `LandDefs::MoveVectorToBeInBlock`: clamps x and y into `[0, block_length - F_EPSILON]`.
fn move_vector_to_be_in_block(v: &mut Vector) {
    let high = ld(BLOCK_LENGTH) - ld(F_EPSILON);
    if high < ld(v.x) {
        v.x = st(high);
    }
    if high < ld(v.y) {
        v.y = st(high);
    }
    if v.x < 0.0 {
        v.x = 0.0;
    }
    if v.y < 0.0 {
        v.y = 0.0;
    }
}

/// Whether landblock `block_id` is resident (`LScapeManager::GetLandBlock`).
fn block_resident(trace: &LosTrace<'_>, block_id: u32) -> bool {
    block_id != 0
        && trace
            .cells
            .land_cell((block_id & 0xFFFF_0000) | 1)
            .is_some()
}

impl<'a> LosTrace<'a> {
    /// `CLandCell::Trace`: once per check, restart from the land cell under the start (the
    /// initial ray is rebased when that is another cell), then trace straight down in that cell
    /// or across the landblocks.
    pub(super) fn land_trace(&mut self, params: &mut LosParameters, cell: &'a LandCell) -> u32 {
        if params.checked_outside {
            return 0;
        }
        params.checked_outside = true;
        let mut cell_id = cell.cell_id;
        let mut start = params.initial_ray.pt;
        adjust_to_outside(&mut cell_id, &mut start);
        let mut current = cell.cell_id;
        if cell_id != cell.cell_id {
            let ray = params.initial_ray;
            let length = ld(ray.length);
            let offset = Vector::new(
                st(length * ld(ray.dir.x)),
                st(length * ld(ray.dir.y)),
                st(length * ld(ray.dir.z)),
            );
            params.set_ray(Ray::from_offset(start, offset), cell_id);
            params.current_ray = params.initial_ray;
            current = cell_id;
        }
        if params.current_ray.dir.x == 0.0 && params.current_ray.dir.y == 0.0 {
            let below = is_outside(params.initial_cell_id)
                .then(|| self.cells.land_cell(params.initial_cell_id))
                .flatten();
            if let Some(below) = below {
                params.trace_result = self.land_cell_trace_ray(params, below);
                params.final_cell_id = below.cell_id;
                params.calc_final_ray();
                return params.trace_result;
            }
            params.final_cell_id = params.initial_cell_id;
            return 2;
        }
        let block = (current & 0xFFFF_0000) | 0xFFFF;
        if !block_resident(self, block) {
            params.final_cell_id = params.initial_cell_id;
            return 2;
        }
        let result = self.trace_ray_cross_block(params, block);
        params.trace_result = result;
        result
    }

    /// `CLandBlock::TraceRayCrossBlock`: the part of the ray in this block, then on into the
    /// neighbour when nothing blocked it. A neighbour that is not resident ends the trace at the
    /// block edge with result 2.
    fn trace_ray_cross_block(&mut self, params: &mut LosParameters, block_id: u32) -> u32 {
        let whole = params.current_ray;
        let Some((inside, rest, direction)) = block_ray_clip(&whole) else {
            params.current_ray = whole;
            params.delta = 1.0;
            return self.land_block_trace_ray(params, block_id);
        };
        let next = neighbour_block(block_id, direction);
        params.current_ray = inside;
        params.delta = 1.0;
        let result = self.land_block_trace_ray(params, block_id);
        if result != 0 {
            return result;
        }
        if !block_resident(self, next) {
            let mut cell_id = block_id;
            let mut end = inside.end_point();
            adjust_to_outside(&mut cell_id, &mut end);
            params.final_cell_id = cell_id;
            let start = params.initial_ray.pt;
            let offset = offset_between(params.initial_cell_id, start, cell_id, end);
            params.final_ray = Ray::from_offset(start, offset);
            params.next_block_id = next;
            params.direction = direction;
            return 2;
        }
        params.current_ray = rest;
        params.delta = 1.0;
        self.trace_ray_cross_block(params, next)
    }

    /// `CLandBlock::TraceRay`: every land cell of the block in the rectangle the current ray
    /// spans, x outer, y inner. Without a hit the final cell is the land cell under the ray's
    /// end.
    fn land_block_trace_ray(&mut self, params: &mut LosParameters, block_id: u32) -> u32 {
        let ray = params.current_ray;
        let length = ld(ray.length);
        let end_x = st(length * ld(ray.dir.x) + ld(ray.pt.x));
        let end_y = st(ld(st(length * ld(ray.dir.y))) + ld(ray.pt.y));
        let inverse = ld(st(1.0 / ld(SQUARE_LENGTH)));
        let cell = |v: f32| (inverse * ld(v)).floor() as i32;
        let (mut x0, mut x1) = (cell(ray.pt.x), cell(end_x));
        let (mut y0, mut y1) = (cell(ray.pt.y), cell(end_y));
        if x1 < x0 {
            std::mem::swap(&mut x0, &mut x1);
        }
        if y1 < y0 {
            std::mem::swap(&mut y0, &mut y1);
        }
        let side = LBLOCK_SIDE - 1;
        let [x0, y0, x1, y1] = [x0, y0, x1, y1].map(|v| v.max(0).min(side));
        params.final_cell_id = 0;
        let mut hit = false;
        let block = block_id & 0xFFFF_0000;
        for x in x0..=x1 {
            for y in y0..=y1 {
                let id = block | (x * LBLOCK_SIDE + y + 1) as u32;
                if let Some(cell) = self.cells.land_cell(id) {
                    if self.land_cell_trace_ray(params, cell) == 1 {
                        hit = true;
                    }
                }
            }
        }
        if !hit {
            if params.final_cell_id == 0 {
                let ray = params.current_ray;
                let length = ld(ray.length);
                let mut end = Vector::new(
                    st(length * ld(ray.dir.x) + ld(ray.pt.x)),
                    st(length * ld(ray.dir.y) + ld(ray.pt.y)),
                    st(length * ld(ray.dir.z) + ld(ray.pt.z)),
                );
                let mut cell_id = block_id;
                params.final_cell_id = if adjust_to_outside(&mut cell_id, &mut end) {
                    cell_id
                } else {
                    0
                };
            }
            params.trace_result = 0;
        }
        params.calc_final_ray();
        u32::from(hit)
    }

    /// `CLandCell::TraceRay`: the nearest terrain polygon nearer than `delta`, the shadow objects
    /// (unless environment only), then into each touching interior cell through its outside
    /// portals the ray (or, after a hit, the final ray) crosses.
    pub(super) fn land_cell_trace_ray(
        &mut self,
        params: &mut LosParameters,
        cell: &'a LandCell,
    ) -> u32 {
        let mut best = params.delta;
        let mut result = 0;
        params.current_cell_id = cell.cell_id;
        for polygon in cell.polygons.iter().flatten() {
            let hit = trace_polygon(&polygon.vertices, &polygon.plane, &params.current_ray);
            if let Some(t) = hit.filter(|&t| t < best) {
                params.delta = t;
                params.trace_result = 1;
                best = t;
                params.impact_normal = polygon.plane.n;
                result = 1;
            }
        }
        if params.intersect != RayIntersection::EnvironmentOnly {
            let objects = self.trace_ray_objects(params, ObjCell::Land(cell));
            if objects != 0 {
                params.trace_result = objects;
                result = objects;
            }
        }
        let ray = if params.trace_result == 0 {
            params.current_ray
        } else {
            params.final_cell_id = cell.cell_id;
            params.calc_final_ray();
            params.final_ray
        };
        let length = ld(ray.length);
        let start = ray.pt;
        let end = Vector::new(
            st(ld(st(ld(ray.dir.x) * length)) + ld(start.x)),
            st(ld(start.y) + ld(ray.dir.y) * length),
            st(ld(start.z) + ld(ray.dir.z) * length),
        );
        for &env_id in &cell.touching {
            if is_outside(env_id) {
                continue;
            }
            let Some(env) = self.cells.env_cell(env_id) else {
                continue;
            };
            let frame = &env.position.frame;
            let local_end = frame.global_to_local(end);
            let local_start = frame.global_to_local(start);
            let local = Ray::from_offset(local_start, sub(local_end, local_start));
            for (index, portal) in env.portals.iter().enumerate() {
                if portal.other_cell.is_some() {
                    continue;
                }
                let hit = trace_polygon(&portal.polygon.vertices, &portal.polygon.plane, &local);
                let Some(t) = hit.filter(|&t| 0.0 < t) else {
                    continue;
                };
                let t = ld(t);
                let cx = st((ld(end.x) - ld(start.x)) * t);
                let dz = st(ld(end.z) - ld(start.z));
                let entry = Vector::new(
                    st(ld(start.x) + ld(cx)),
                    st(ld(start.y) + (ld(end.y) - ld(start.y)) * t),
                    st(ld(start.z) + ld(dz) * t),
                );
                params.current_ray = Ray::from_offset(entry, sub(end, entry));
                params.delta = 1.0;
                let inside = self.env_trace_ray(params, env, Some(index));
                if inside != 0 {
                    params.trace_result = inside;
                    result = inside;
                }
            }
        }
        result
    }
}
