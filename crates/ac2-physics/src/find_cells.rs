//! `CObjCell::FindCells`: the cells a box in a position's landblock frame touches, through the
//! land-cell grid (`CLandCell::FindOutsideCells`, `AddOutsideCell` and the env cells a land cell
//! touches) and the portals of interior cells (`CEnvCell::FindTransitCells`), with the cell that
//! holds the position.

use std::collections::HashSet;

use crate::cells::CellSource;
use crate::envcell::EnvCell;
use crate::fpu::{ld, st};
use crate::landdefs::{
    adjust_to_outside, get_block_offset, gid_to_lcoord, is_outside, lcoord_to_gid, LAND_WIDTH,
};
use crate::objects::BBox;
use crate::position::Position;
use crate::transition::{CellArray, Transition};
use crate::vector::{Sphere, Vector};

/// `FindCellInfo`.
pub(crate) struct FindCellInfo<'a> {
    pub origin: Position,
    pub bounds: BBox,
    pub touched: &'a mut CellArray,
    pub current: Option<u32>,
    pub found: bool,
    pub touches_inside: bool,
}

impl<'a> FindCellInfo<'a> {
    pub(crate) fn new(origin: Position, bounds: BBox, touched: &'a mut CellArray) -> Self {
        Self {
            origin,
            bounds,
            touched,
            current: None,
            found: false,
            touches_inside: false,
        }
    }
}

/// `CObjCell::FindCells(FindCellInfo*)`: an interior origin starts at its cell's portals, an
/// outdoor one at the land-cell grid.
pub(crate) fn find_cells_info(info: &mut FindCellInfo<'_>, cells: &dyn CellSource) {
    let cell_id = info.origin.cell_id;
    info.current = None;
    info.found = false;
    if !is_outside(cell_id) {
        info.touches_inside = true;
        let cell = cells.env_cell(cell_id);
        info.touched.add_cell(cell_id, cell.is_some());
        if let Some(cell) = cell {
            find_transit_cells(cell, info, cells);
        }
        return;
    }
    find_outside_cells(info, cells);
}

/// `CLandCell::FindOutsideCells`: once per cell array, every land cell of the block-grid
/// rectangle between the box corners, column by column.
fn find_outside_cells(info: &mut FindCellInfo<'_>, cells: &dyn CellSource) {
    if info.touched.added_outside {
        return;
    }
    info.touched.added_outside = true;
    let (mut min, mut max) = (info.bounds.min, info.bounds.max);
    let (mut min_cell, mut max_cell) = (info.origin.cell_id, info.origin.cell_id);
    if !adjust_to_outside(&mut min_cell, &mut min) || !adjust_to_outside(&mut max_cell, &mut max) {
        return;
    }
    let (Some((x0, y0)), Some((x1, y1))) = (gid_to_lcoord(min_cell), gid_to_lcoord(max_cell))
    else {
        return;
    };
    for x in x0..=x1 {
        for y in y0..=y1 {
            add_outside_cell(info, x, y, cells);
        }
    }
}

/// `CLandCell::AddOutsideCell`: the land cell (current when it holds the origin and no interior
/// cell does), then the env cells it touches and their portals.
fn add_outside_cell(info: &mut FindCellInfo<'_>, x: i32, y: i32, cells: &dyn CellSource) {
    if !(0..LAND_WIDTH).contains(&x) || !(0..LAND_WIDTH).contains(&y) {
        return;
    }
    let gid = lcoord_to_gid(x, y);
    let cell = cells.land_cell(gid);
    if let Some(cell) = cell {
        let origin = info.origin.frame.origin;
        let offset = get_block_offset(info.origin.cell_id, gid);
        let local = Vector::new(
            st(ld(origin.x) - ld(offset.x)),
            st(ld(origin.y) - ld(offset.y)),
            st(ld(origin.z) - ld(offset.z)),
        );
        if !info.found && cell.point_in_cell(local) {
            info.current = Some(gid);
        }
    }
    info.touched.add_cell(gid, cell.is_some());
    let Some(cell) = cell else {
        return;
    };
    for &touching in &cell.touching {
        let env = cells.env_cell(touching);
        info.touched.add_cell(touching, env.is_some());
        if let Some(env) = env {
            find_transit_cells(env, info, cells);
        }
    }
}

/// `CEnvCell::FindTransitCells`: a new transit stamp, then the portal walk.
fn find_transit_cells(cell: &EnvCell, info: &mut FindCellInfo<'_>, cells: &dyn CellSource) {
    let mut visited = HashSet::new();
    find_transit_cells_internal(cell, info, &mut visited, cells);
}

/// `CEnvCell::FindTransitCellsInternal`: the cell is current when it holds the box centre; every
/// portal the box reaches adds the cell beyond it (walking on through resident ones), and an
/// outside portal adds the outdoor cells.
fn find_transit_cells_internal(
    cell: &EnvCell,
    info: &mut FindCellInfo<'_>,
    visited: &mut HashSet<u32>,
    cells: &dyn CellSource,
) {
    if !visited.insert(cell.cell_id) {
        return;
    }
    let b = info.bounds;
    let center = Vector::new(
        st((ld(b.max.x) + ld(b.min.x)) * 0.5),
        st((ld(b.max.y) + ld(b.min.y)) * 0.5),
        st((ld(b.max.z) + ld(b.min.z)) * 0.5),
    );
    info.touches_inside = true;
    if !info.found {
        let offset = get_block_offset(info.origin.cell_id, cell.cell_id);
        let local = Vector::new(
            st(ld(center.x) - ld(offset.x)),
            st(ld(center.y) - ld(offset.y)),
            st(ld(center.z) - ld(offset.z)),
        );
        if cell.point_in_cell(local) {
            info.current = Some(cell.cell_id);
            info.found = true;
        }
    }
    let dz = ld(st(ld(b.max.z) - ld(b.min.z)));
    let hx = ld(st((ld(b.max.x) - ld(b.min.x)) * 0.5));
    let hy = (ld(b.max.y) - ld(b.min.y)) * 0.5;
    let hz = dz * 0.5;
    let radius = st(((hz * hz + hy * hy) + hx * hx).sqrt());
    let local_center = cell.position.frame.global_to_local(center);
    let mut outdoors = false;
    for portal in &cell.portals {
        if !cell.portal_reaches(&portal.polygon, &info.bounds, local_center, radius) {
            continue;
        }
        let Some(other) = portal.other_cell else {
            outdoors = true;
            continue;
        };
        let env = cells.env_cell(other);
        info.touched.add_cell(other, env.is_some());
        if let Some(env) = env {
            find_transit_cells_internal(env, info, visited, cells);
        }
    }
    if outdoors {
        find_outside_cells(info, cells);
    }
}

/// The box `BBox::InitFromSpheres` grows by: centre minus and plus the radius, stored.
pub(crate) fn sphere_bounds(spheres: &[Sphere]) -> BBox {
    let mut bounds = BBox::EMPTY;
    for sphere in spheres {
        let (c, r) = (sphere.center, ld(sphere.radius));
        bounds.adjust(Vector::new(
            st(ld(c.x) - r),
            st(ld(c.y) - r),
            st(ld(c.z) - r),
        ));
        bounds.adjust(Vector::new(
            st(r + ld(c.x)),
            st(r + ld(c.y)),
            st(r + ld(c.z)),
        ));
    }
    bounds
}

/// `CObjCell::FindCells(CELLARRAY*, CObjCell**, SPHEREPATH*)`: the cells the path's global
/// spheres touch at the check position, recording an interior hit; returns the cell holding the
/// check position.
pub(crate) fn find_cells(transition: &mut Transition, cells: &dyn CellSource) -> Option<u32> {
    let path = &transition.sphere_path;
    let bounds = sphere_bounds(&path.global_sphere[..path.num_sphere]);
    let mut info = FindCellInfo::new(path.check_pos, bounds, &mut transition.cell_array);
    find_cells_info(&mut info, cells);
    let (current, inside) = (info.current, info.touches_inside);
    if inside {
        transition.sphere_path.hits_interior_cell = true;
    }
    current
}

/// `CObjCell::FindCellsLocal(Position*, BBox*, CELLARRAY*)`: the cells a box of `position`'s
/// frame touches, resident or not (static objects shadow them,
/// `CPhysicsObj::add_shadows_to_cells`).
pub(crate) fn shadow_cells(position: &Position, local: BBox, cells: &dyn CellSource) -> Vec<u32> {
    let mut touched = CellArray::default();
    let bounds = local.local_to_global(position);
    let mut info = FindCellInfo::new(*position, bounds, &mut touched);
    find_cells_info(&mut info, cells);
    touched.cells.iter().map(|&(id, _)| id).collect()
}
