//! `ObjectClearance`: the space a placed object keeps scenery out of. Groups and scene objects
//! clear a circle (or sphere) around their origin; `CLandBlockInfo` entities clear the boxes and
//! cylinders their `EntityDesc` properties describe (`CLandBlock::AddEntityToClearList`).

use ac2_db::{
    read_ac2_property_groups, update_ac2_property_collection, Ac2PropertyGroup, Ac2PropertyValue,
};

use super::geometry::{qrot, Q, V3};

/// `PropertyCollection` group holding the clearance properties (`CLandBlock::InqClearanceInfo`).
const CLEARANCE_GROUP: u32 = 0x4000_0009;
const EPSILON: f64 = 0.0002;

/// `Cylsphere`: a vertical cylinder from `low` up `height`, radius `radius`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Cylinder {
    pub(super) low: V3,
    pub(super) radius: f64,
    pub(super) height: f64,
}

/// An `ObjectClearance` in block space.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Clearance {
    pub(super) origin: V3,
    /// Rotation of the clearance's frame, used to bring tested points into box/cylinder space.
    pub(super) rotation: Q,
    pub(super) radius: f64,
    pub(super) is_3d: bool,
    /// Groups clear only once their children are placed (`ObjectClearance::InSuperGroup`).
    pub(super) is_group: bool,
    pub(super) group_done: bool,
    pub(super) boxes: Vec<(V3, V3)>,
    pub(super) cylinders: Vec<Cylinder>,
}

impl Clearance {
    /// `ObjectClearance::InitAsGroup` + `SetClearance`.
    pub(super) fn group(origin: V3, rotation: Q, radius: f64, is_3d: bool) -> Self {
        Self {
            origin,
            rotation,
            radius,
            is_3d,
            is_group: true,
            group_done: false,
            boxes: Vec::new(),
            cylinders: Vec::new(),
        }
    }

    /// `ObjectClearance::InitAsEntity` for a scene object or an `CLandBlockInfo` entity.
    pub(super) fn entity(origin: V3, rotation: Q, shapes: (Vec<(V3, V3)>, Vec<Cylinder>)) -> Self {
        Self {
            origin,
            rotation,
            radius: 0.0,
            is_3d: false,
            is_group: false,
            group_done: false,
            boxes: shapes.0,
            cylinders: shapes.1,
        }
    }

    /// `CLandBlock::ObjectCollides`' per-entry test: open groups are skipped.
    pub(super) fn blocks(&self, loc: V3, radius: f64, is_3d: bool) -> bool {
        (!self.is_group || self.group_done) && self.check_intersection(loc, radius, is_3d)
    }

    /// `ObjectClearance::CheckIntersection`.
    fn check_intersection(&self, loc: V3, radius: f64, is_3d: bool) -> bool {
        if self.boxes.is_empty() && self.cylinders.is_empty() {
            let dz = if is_3d && self.is_3d {
                loc[2] - self.origin[2]
            } else {
                0.0
            };
            let (dx, dy) = (loc[0] - self.origin[0], loc[1] - self.origin[1]);
            let reach = radius + self.radius;
            return dz * dz + dy * dy + dx * dx <= reach * reach;
        }
        let local = self.global_to_local(loc);
        self.boxes
            .iter()
            .any(|b| intersects_box(b, local, radius, is_3d))
            || self
                .cylinders
                .iter()
                .any(|c| intersects_cylinder(c, local, radius, is_3d))
    }

    /// `Frame::globaltolocal`.
    fn global_to_local(&self, p: V3) -> V3 {
        let [w, x, y, z] = self.rotation;
        let d = [
            p[0] - self.origin[0],
            p[1] - self.origin[1],
            p[2] - self.origin[2],
        ];
        qrot([w, -x, -y, -z], d)
    }
}

/// `ObjectClearance::IntersectsBox`: distance from the box to the point within the radius; a
/// degenerate box (all extents under 0.0002) clears nothing.
fn intersects_box(&(min, max): &(V3, V3), loc: V3, radius: f64, is_3d: bool) -> bool {
    if (0..3).all(|i| (min[i] - max[i]).abs() < EPSILON) {
        return false;
    }
    let gap = |i: usize| {
        if loc[i] < min[i] {
            min[i] - loc[i]
        } else if max[i] < loc[i] {
            loc[i] - max[i]
        } else {
            0.0
        }
    };
    let dz = if is_3d { gap(2) } else { 0.0 };
    gap(0) * gap(0) + gap(1) * gap(1) + dz * dz <= radius * radius
}

/// `ObjectClearance::IntersectsCyl`: a zero or negative radius clears nothing; 3D tests use
/// `Cylsphere::Intersects(Sphere)`, 2D the circle distance to the base point.
fn intersects_cylinder(c: &Cylinder, loc: V3, radius: f64, is_3d: bool) -> bool {
    if c.radius <= 0.0 {
        return false;
    }
    if is_3d {
        return sphere_intersects_cylinder(loc, radius, c);
    }
    let reach = radius + c.radius;
    let (dx, dy) = (loc[0] - c.low[0], loc[1] - c.low[1]);
    dx * dx + dy * dy <= reach * reach
}

/// `Sphere::Intersects(Cylsphere, zmin = 0)`: the sphere centre's height clamped into the cylinder,
/// then strictly within the summed radii.
fn sphere_intersects_cylinder(center: V3, radius: f64, c: &Cylinder) -> bool {
    let top = c.low[2] + c.height;
    let z = if center[2] > top {
        top
    } else if center[2] < c.low[2] {
        c.low[2]
    } else {
        center[2]
    };
    let (dx, dy, dz) = (c.low[0] - center[0], c.low[1] - center[1], center[2] - z);
    let reach = c.radius + radius;
    dx * dx + dy * dy + dz * dz < reach * reach
}

/// `EntityDesc::GetTotalProperties`: the template chain's properties first, then each level's
/// own (`PropertyCollection::UpdateCollection`). `template` resolves a template `EntityDesc`
/// DID to its packed properties and its own template.
pub(super) fn total_properties(
    properties: Option<&[u8]>,
    template: Option<u32>,
    resolve: &dyn Fn(u32) -> Option<(Option<std::sync::Arc<[u8]>>, Option<u32>)>,
) -> Vec<Ac2PropertyGroup> {
    let mut total = Vec::new();
    // `MasterDBMap::DivineType` == `DB_TYPE_ENTITYDESC` (0x47xxxxxx).
    if let Some(did) = template.filter(|did| did >> 24 == 0x47) {
        if let Some((bytes, next)) = resolve(did) {
            total = total_properties(bytes.as_deref(), next, resolve);
        }
    }
    if let Some(bytes) = properties {
        if let Ok(groups) = read_ac2_property_groups(&mut ac2_dat::ArchiveCursor::new(bytes)) {
            update_ac2_property_collection(&mut total, &groups);
        }
    }
    total
}

/// `CLandBlock::InqClearanceInfo` + `ObjectClearance::ApplyClearanceProperty`: boxes from
/// properties `0x40000028`-`2F` (min/max of boxes 0-3), cylinders from `0x40000030`-`35`
/// (radius, centre, height of cylinders 0-1). Setting slot `n` first grows the list with zeroed
/// entries; a value of the wrong type leaves the zeroed slot.
pub(super) fn clearance_shapes(groups: &[Ac2PropertyGroup]) -> (Vec<(V3, V3)>, Vec<Cylinder>) {
    let (mut boxes, mut cylinders) = (Vec::<(V3, V3)>::new(), Vec::<Cylinder>::new());
    let Some(group) = groups.iter().find(|g| g.name == CLEARANCE_GROUP) else {
        return (boxes, cylinders);
    };
    let vector = |p: &Ac2PropertyValue| p.vector().map(|v| v.map(f64::from));
    let float = |p: &Ac2PropertyValue| p.float().map(f64::from);
    for p in &group.properties {
        let id = p.name.wrapping_sub(0x4000_0028);
        match id {
            0..=7 => {
                let index = (id % 4) as usize;
                if boxes.len() <= index {
                    boxes.resize(index + 1, ([0.0; 3], [0.0; 3]));
                }
                if let Some(v) = vector(p) {
                    if id < 4 {
                        boxes[index].0 = v;
                    } else {
                        boxes[index].1 = v;
                    }
                }
            }
            8..=13 => {
                let index = (id % 2) as usize;
                if cylinders.len() <= index {
                    cylinders.resize(index + 1, Cylinder::default());
                }
                let c = &mut cylinders[index];
                match id {
                    8 | 9 => c.radius = float(p).unwrap_or(c.radius),
                    10 | 11 => c.low = vector(p).unwrap_or(c.low),
                    _ => c.height = float(p).unwrap_or(c.height),
                }
            }
            _ => {}
        }
    }
    (boxes, cylinders)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: Q = [1.0, 0.0, 0.0, 0.0];

    #[test]
    fn point_clearance_uses_summed_radii_in_2d_unless_both_are_3d() {
        let c = Clearance::group([10.0, 10.0, 0.0], IDENTITY, 2.0, true);
        let mut done = c.clone();
        assert!(
            !c.blocks([11.0, 10.0, 0.0], 1.0, true),
            "open groups do not clear"
        );
        done.group_done = true;
        assert!(done.blocks([13.0, 10.0, 50.0], 1.0, false));
        assert!(!done.blocks([13.0, 10.0, 50.0], 1.0, true));
    }

    #[test]
    fn boxes_clear_in_the_entity_frame() {
        // A 4 x 2 box turned 90 degrees about z: it spans 2 in x and 4 in y in block space.
        let half_turn = std::f64::consts::FRAC_PI_4;
        let rotation = [half_turn.cos(), 0.0, 0.0, half_turn.sin()];
        let c = Clearance::entity(
            [50.0, 50.0, 0.0],
            rotation,
            (vec![([-2.0, -1.0, 0.0], [2.0, 1.0, 5.0])], Vec::new()),
        );
        assert!(c.blocks([50.5, 51.9, 0.0], 0.0, false));
        assert!(!c.blocks([51.9, 50.5, 0.0], 0.5, false));
    }

    #[test]
    fn cylinders_need_a_positive_radius_and_clamp_height_in_3d() {
        let cyl = Cylinder {
            low: [0.0, 0.0, 0.0],
            radius: 1.0,
            height: 4.0,
        };
        assert!(intersects_cylinder(&cyl, [1.5, 0.0, 2.0], 0.6, true));
        assert!(!intersects_cylinder(&cyl, [1.5, 0.0, 6.0], 0.6, true));
        assert!(intersects_cylinder(&cyl, [1.5, 0.0, 6.0], 0.6, false));
        let flat = Cylinder { radius: 0.0, ..cyl };
        assert!(!intersects_cylinder(&flat, [0.0; 3], 5.0, false));
    }

    #[test]
    fn degenerate_boxes_clear_nothing() {
        assert!(!intersects_box(&([1.0; 3], [1.0; 3]), [1.0; 3], 10.0, true));
    }
}
