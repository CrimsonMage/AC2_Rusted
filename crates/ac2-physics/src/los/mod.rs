//! Line of sight (`CObjCell::LOSCheck` over `LOSParameters`): a ray traced through the resident
//! cells -- terrain polygons, interior physics BSPs, portals and the cells' static objects --
//! from its start cell to the first thing that blocks it. The camera's pull-in
//! (`SmartBox::UpdateCamera`) and the height-above-walkable probes cast these.
//!
//! Every function here is identical in admin 7148 and Legions 7160 (checked instruction by
//! instruction; only the vtable slots differ).

mod env;
mod land;
mod objects;
#[cfg(test)]
mod tests;

use crate::cells::{CellSource, ObjCell};
use crate::fpu::{ld, st, F_EPSILON};
use crate::landdefs::{get_block_offset, is_outside};
use crate::position::{offset_between, Position};
use crate::vector::Vector;

/// `RayIntersectionType`: what a line of sight collides with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RayIntersection {
    /// Terrain, interior physics BSPs and the physics shapes of objects.
    #[default]
    PhysicsRep,
    /// The drawn meshes (interior render meshes and object parts are not ported: nothing hits).
    ExactDrawingRep,
    /// Terrain and interior physics BSPs only.
    EnvironmentOnly,
    /// Interior portals only.
    PortalsOnly,
    /// Interior portals; connected ones are walked through.
    UnconnectedPortalsOnly,
}

/// `Ray`: a start point, a unit direction and a length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub pt: Vector,
    pub dir: Vector,
    pub length: f32,
}

impl Default for Ray {
    /// `Ray::Ray`: zero point and direction, length 100000.
    fn default() -> Self {
        Self {
            pt: Vector::ZERO,
            dir: Vector::ZERO,
            length: 100_000.0,
        }
    }
}

impl Ray {
    /// `Ray::InitFromOffset`: from `pt` along `offset`. An offset within `F_EPSILON` of zero on
    /// every axis gives length 0 pointing along +y; otherwise the length is
    /// `sqrt((z*z + y*y) + x*x)` and the direction `offset * (1 / length)`.
    pub fn from_offset(pt: Vector, offset: Vector) -> Self {
        let small = |v: f32| ld(v).abs() < ld(F_EPSILON);
        if small(offset.x) && small(offset.y) && small(offset.z) {
            return Self {
                pt,
                dir: Vector::new(0.0, 1.0, 0.0),
                length: 0.0,
            };
        }
        let (x, y, z) = (ld(offset.x), ld(offset.y), ld(offset.z));
        let length = st(((z * z + y * y) + x * x).sqrt());
        let inverse = 1.0 / ld(length);
        Self {
            pt,
            dir: Vector::new(st(inverse * x), st(inverse * y), st(inverse * z)),
            length,
        }
    }

    /// `Ray::InitLineSegment`: from `a` to `b` (the offset stored first).
    pub fn line_segment(a: Vector, b: Vector) -> Self {
        Self::from_offset(a, sub(b, a))
    }

    /// `Ray::GetEndPoint`: `length * dir + pt`.
    pub fn end_point(&self) -> Vector {
        let length = ld(self.length);
        Vector::new(
            st(length * ld(self.dir.x) + ld(self.pt.x)),
            st(length * ld(self.dir.y) + ld(self.pt.y)),
            st(length * ld(self.dir.z) + ld(self.pt.z)),
        )
    }

    /// The end point as most trace functions form it: the x product stays on the x87 stack, the
    /// y and z products are stored first.
    pub(crate) fn end_stored_yz(&self) -> Vector {
        let length = ld(self.length);
        Vector::new(
            st(length * ld(self.dir.x) + ld(self.pt.x)),
            st(ld(st(length * ld(self.dir.y))) + ld(self.pt.y)),
            st(ld(st(length * ld(self.dir.z))) + ld(self.pt.z)),
        )
    }
}

/// `a - b` with each component stored.
pub(crate) fn sub(a: Vector, b: Vector) -> Vector {
    Vector::new(
        st(ld(a.x) - ld(b.x)),
        st(ld(a.y) - ld(b.y)),
        st(ld(a.z) - ld(b.z)),
    )
}

/// `LOSParameters`: the ray, what it may hit, and the result of the trace.
#[derive(Debug, Clone, PartialEq)]
pub struct LosParameters {
    /// `m_intersect_type` (`SetCurrentCGR` writes it).
    pub intersect: RayIntersection,
    /// `m_bTraversePortals`.
    pub traverse_portals: bool,
    checked_outside: bool,
    initial_cell_id: u32,
    initial_ray: Ray,
    current_ray: Ray,
    current_cell_id: u32,
    delta: f32,
    trace_result: u32,
    final_cell_id: u32,
    final_ray: Ray,
    impact_normal: Vector,
    portal_index: Option<usize>,
    direction: u32,
    next_block_id: u32,
}

impl Default for LosParameters {
    fn default() -> Self {
        Self::new()
    }
}

impl LosParameters {
    /// `LOSParameters::LOSParameters`: physics representation, portals traversed, no ray.
    pub fn new() -> Self {
        Self {
            intersect: RayIntersection::PhysicsRep,
            traverse_portals: true,
            checked_outside: false,
            initial_cell_id: 0,
            initial_ray: Ray::default(),
            current_ray: Ray::default(),
            current_cell_id: 0,
            delta: 0.0,
            trace_result: 0,
            final_cell_id: 0,
            final_ray: Ray::default(),
            impact_normal: Vector::ZERO,
            portal_index: None,
            direction: 0,
            next_block_id: 0,
        }
    }

    /// `LOSParameters::SetRay`: the ray, starting in `cell_id` (its block's frame).
    pub fn set_ray(&mut self, ray: Ray, cell_id: u32) {
        self.initial_ray = ray;
        self.initial_cell_id = cell_id;
        self.current_cell_id = cell_id;
    }

    /// `LOSParameters::SetFromPositionAndOffset`.
    pub fn set_from_position_and_offset(&mut self, position: &Position, offset: Vector) {
        self.set_ray(
            Ray::from_offset(position.frame.origin, offset),
            position.cell_id,
        );
    }

    /// `LOSParameters::SetFromBeginAndEndPosition`: the ray from `begin` to `end` across
    /// landblocks (the offset's z sum is stored, its x and y sums are not).
    pub fn set_from_begin_and_end_position(&mut self, begin: &Position, end: &Position) {
        let block = get_block_offset(begin.cell_id, end.cell_id);
        let (b, e) = (begin.frame.origin, end.frame.origin);
        let z = st(ld(block.z) + ld(e.z));
        let offset = Vector::new(
            st((ld(block.x) + ld(e.x)) - ld(b.x)),
            st((ld(block.y) + ld(e.y)) - ld(b.y)),
            st(ld(z) - ld(b.z)),
        );
        self.set_ray(Ray::from_offset(b, offset), begin.cell_id);
    }

    /// `LOSParameters::GetEndPoint`: where the final ray ends, in the final cell's landblock
    /// (`Position(final_cell_id, end_point)` is where the trace stopped).
    pub fn end_point(&self) -> Vector {
        let ray = &self.final_ray;
        let length = ld(ray.length);
        let x = length * ld(ray.dir.x) + ld(ray.pt.x);
        let y = ld(st(length * ld(ray.dir.y))) + ld(ray.pt.y);
        let z = ld(st(length * ld(ray.dir.z))) + ld(ray.pt.z);
        let offset = get_block_offset(self.initial_cell_id, self.final_cell_id);
        Vector::new(
            st(x - ld(offset.x)),
            st(y - ld(offset.y)),
            st(z - ld(offset.z)),
        )
    }

    /// The start cell (`m_initial_cell_id`; the outdoor trace may move it to the land cell under
    /// the start).
    pub fn initial_cell_id(&self) -> u32 {
        self.initial_cell_id
    }

    /// `m_final_cell_id`: the cell the trace ended in (0 off the map).
    pub fn final_cell_id(&self) -> u32 {
        self.final_cell_id
    }

    /// `m_final_ray`: from the start to where the trace ended.
    pub fn final_ray(&self) -> Ray {
        self.final_ray
    }

    /// `m_impact_normal`: the normal of what stopped the ray (zero when nothing did).
    pub fn impact_normal(&self) -> Vector {
        self.impact_normal
    }

    /// `m_trace_result`: 0 clear, 1 blocked, 2 left the resident world.
    pub fn trace_result(&self) -> u32 {
        self.trace_result
    }

    /// `m_delta`: the fraction of the current ray travelled before the hit.
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// `m_portal_index`: the portal a portal-only trace stopped at.
    pub fn portal_index(&self) -> Option<usize> {
        self.portal_index
    }

    /// `m_next_block_id` / `m_dir`: the non-resident landblock (and `DirectionEnum`) the ray
    /// ran into when the trace result is 2 outdoors.
    pub fn next_block(&self) -> (u32, u32) {
        (self.next_block_id, self.direction)
    }

    /// `LOSParameters::ResetState`.
    fn reset_state(&mut self) {
        self.current_ray = self.initial_ray;
        self.current_cell_id = self.initial_cell_id;
        self.delta = 1.0;
        self.checked_outside = false;
        self.next_block_id = 0;
    }

    /// `LOSParameters::CalcFinalRay`: the final ray from the start to `delta` along the current
    /// ray, when the start and final cells are both set or both unset.
    pub(crate) fn calc_final_ray(&mut self) -> bool {
        let ray = &self.current_ray;
        let (length, delta) = (ld(ray.length), ld(self.delta));
        let y = st(delta * ld(ray.dir.y) * length);
        let z = st(delta * ld(ray.dir.z) * length);
        let end = Vector::new(
            st(delta * ld(ray.dir.x) * length + ld(ray.pt.x)),
            st(ld(y) + ld(ray.pt.y)),
            st(ld(z) + ld(ray.pt.z)),
        );
        if (self.initial_cell_id == 0) != (self.final_cell_id == 0) {
            return false;
        }
        let offset = offset_between(
            self.initial_cell_id,
            self.initial_ray.pt,
            self.final_cell_id,
            end,
        );
        self.final_ray = Ray::from_offset(self.initial_ray.pt, offset);
        true
    }
}

/// `CObjCell::LOSCheck`: traces the parameters' ray from its start cell (`CLandCell::Trace`
/// outdoors, `CEnvCell::TraceRay` indoors). False when the start cell is not resident; the
/// result is in the parameters.
pub(crate) fn los_check(cells: &dyn CellSource, params: &mut LosParameters) -> bool {
    let cell_id = params.initial_cell_id;
    if cell_id == 0 {
        return false;
    }
    params.reset_state();
    let cell = if is_outside(cell_id) {
        cells.land_cell(cell_id).map(ObjCell::Land)
    } else {
        cells.env_cell(cell_id).map(ObjCell::Env)
    };
    let Some(cell) = cell else {
        return false;
    };
    LosTrace::new(cells).trace(params, cell);
    true
}

/// One trace through the resident cells, with the interior cells it is inside of
/// (`CEnvCell::m_bOnStack`).
pub(crate) struct LosTrace<'a> {
    cells: &'a dyn CellSource,
    on_stack: Vec<u32>,
}

impl<'a> LosTrace<'a> {
    fn new(cells: &'a dyn CellSource) -> Self {
        Self {
            cells,
            on_stack: Vec::new(),
        }
    }

    /// The cell's `Trace` slot: `CLandCell::Trace` or `CEnvCell::Trace` (`TraceRay` with no
    /// portal to skip).
    fn trace(&mut self, params: &mut LosParameters, cell: ObjCell<'a>) -> u32 {
        match cell {
            ObjCell::Land(land) => self.land_trace(params, land),
            ObjCell::Env(env) => self.env_trace_ray(params, env, None),
        }
    }
}
