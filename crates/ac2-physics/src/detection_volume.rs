//! `DetectionVolume` (`portal/shared/src/physobj/detect/detectionvolume.cpp`): a region a
//! `CDetectionManager` watches objects enter, placed by a `Position` and shaped as a sphere, a box
//! or an upright cylinder. The tests it makes of a sphere, a cylinder or a box (`IsWithin`) put
//! the object's shape in the volume's frame first (`Position::localtolocal`) and run the shape's
//! own intersection. The bound-volume, sphere-cone and sphere-plane-cone shapes are not ported:
//! a volume can not be made with them.

use crate::fpu::{ld, st};
use crate::objects::BBox;
use crate::position::Position;
use crate::shapes::cylsphere::CylSphere;
use crate::vector::{Sphere, Vector};

/// `DetectionVolumeType::Type` for the shapes ported.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectionShape {
    /// `DetSphere`.
    Sphere(Sphere),
    /// `DetBoundBox`, in the volume's frame.
    BoundBox(BBox),
    /// `DetCylinder` (`Cylsphere`).
    Cylinder(CylSphere),
}

/// `DetectionVolume`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DetectionVolume {
    /// `m_volume_pos`: where the volume stands (cell 0 until it is placed).
    pub position: Position,
    shape: Option<DetectionShape>,
}

/// `BBox::Within`: the point is inside the box, its faces included.
pub fn box_within(b: &BBox, p: Vector) -> bool {
    b.min.x <= p.x
        && p.x <= b.max.x
        && b.min.y <= p.y
        && p.y <= b.max.y
        && b.min.z <= p.z
        && p.z <= b.max.z
}

/// `BBox::Intersects(Sphere*)`: the nearest point of the box to the sphere's centre is within its
/// radius.
pub fn box_intersects_sphere(b: &BBox, s: &Sphere) -> bool {
    let clamp = |low: f32, high: f32, c: f32| {
        if c <= high {
            if low <= c {
                c
            } else {
                low
            }
        } else {
            high
        }
    };
    let near_x = clamp(b.min.x, b.max.x, s.center.x);
    let near_y = clamp(b.min.y, b.max.y, s.center.y);
    let near_z = clamp(b.min.z, b.max.z, s.center.z);
    let dx = ld(st(ld(near_x) - ld(s.center.x)));
    let dy = ld(st(ld(near_y) - ld(s.center.y)));
    let dz = ld(st(ld(near_z) - ld(s.center.z)));
    (dy * dy + dz * dz) + dx * dx <= ld(s.radius) * ld(s.radius)
}

/// `BBox::Intersects(Cylsphere*)`: the vertical extents overlap and the box's nearest point to
/// the axis (in x and y) is within the radius.
pub fn box_intersects_cylinder(b: &BBox, c: &CylSphere) -> bool {
    let box_half = ld(st((ld(b.max.z) - ld(b.min.z)) * 0.5));
    let cyl_half = ld(st(ld(c.height) * 0.5));
    if ((box_half + ld(b.min.z)) - (cyl_half + ld(c.low.z))).abs() > box_half.abs() + cyl_half.abs()
    {
        return false;
    }
    let (x, y) = (c.low.x, c.low.y);
    let near_x = if x <= b.max.x {
        if x < b.min.x {
            b.min.x
        } else {
            x
        }
    } else {
        b.max.x
    };
    let near_y = if y <= b.max.y {
        if y < b.min.y {
            b.min.y
        } else {
            y
        }
    } else {
        b.max.y
    };
    let dy = ld(near_y) - ld(y);
    let dx = ld(near_x) - ld(x);
    dy * dy + dx * dx <= ld(c.radius) * ld(c.radius)
}

/// `BBox::DoesOverlap(box, epsilon)`: the boxes overlap on every axis by more than `epsilon`
/// (half extents compared with the distance between centres).
pub fn box_does_overlap(a: &BBox, b: &BBox, epsilon: f32) -> bool {
    let axis = |a_min: f32, a_max: f32, b_min: f32, b_max: f32| {
        let ha = ld(st((ld(a_max) - ld(a_min)) * 0.5));
        let hb = ld(st((ld(b_max) - ld(b_min)) * 0.5));
        ((hb + ld(b_min)) - (ha + ld(a_min))).abs() + ld(epsilon) <= ha.abs() + hb.abs()
    };
    axis(a.min.x, a.max.x, b.min.x, b.max.x)
        && axis(a.min.y, a.max.y, b.min.y, b.max.y)
        && axis(a.min.z, a.max.z, b.min.z, b.max.z)
}

/// `Sphere::Intersects(Cylsphere*, 0)`: the sphere's centre against the cylinder grown by the
/// sphere's radius.
pub fn sphere_intersects_cylinder(s: &Sphere, c: &CylSphere) -> bool {
    let top = st(ld(c.low.z) + ld(c.height));
    let z = if s.center.z <= top {
        if c.low.z <= s.center.z {
            s.center.z
        } else {
            c.low.z
        }
    } else {
        top
    };
    let dx = ld(st(ld(c.low.x) - ld(s.center.x)));
    let dy = ld(st(ld(c.low.y) - ld(s.center.y)));
    let dz = ld(st(ld(s.center.z) - ld(z)));
    let reach = ld(st(ld(c.radius) + ld(s.radius)));
    let horizontal = ld(st(dy * dy + dx * dx));
    let vertical = ld(st(dz * dz));
    !(reach * reach <= vertical + horizontal)
}

/// `Cylsphere::Intersects(Cylsphere*)`: the vertical extents overlap and the axes are within the
/// radii.
pub fn cylinder_intersects_cylinder(a: &CylSphere, b: &CylSphere) -> bool {
    let ha = ld(st(ld(a.height) * 0.5));
    let hb = ld(st(ld(b.height) * 0.5));
    if ((ha + ld(a.low.z)) - (hb + ld(b.low.z))).abs() > ha.abs() + hb.abs() {
        return false;
    }
    let dx = ld(st(ld(a.low.x) - ld(b.low.x)));
    let dy = ld(st(ld(a.low.y) - ld(b.low.y)));
    let reach = ld(st(ld(b.radius) + ld(a.radius)));
    dy * dy + dx * dx <= reach * reach
}

impl DetectionVolume {
    /// `DetectionVolume::DetectionVolume`: no shape, no cell.
    pub fn new() -> Self {
        Self::default()
    }

    /// The shape, when one has been made.
    pub fn shape(&self) -> Option<&DetectionShape> {
        self.shape.as_ref()
    }

    /// `DetectionVolume::CreateBBox`: a box around the origin reaching `east`, `north` and `up`
    /// one way and `west`, `south` and `down` the other.
    pub fn create_bbox(
        &mut self,
        north: f32,
        south: f32,
        east: f32,
        west: f32,
        up: f32,
        down: f32,
    ) {
        let mut b = BBox {
            min: Vector::new(east, north, up),
            max: Vector::new(east, north, up),
        };
        b.adjust(Vector::new(-west, -south, -down));
        self.shape = Some(DetectionShape::BoundBox(b));
    }

    /// `DetectionVolume::CreateCylinder`.
    pub fn create_cylinder(&mut self, radius: f32, low: Vector, height: f32) {
        self.shape = Some(DetectionShape::Cylinder(CylSphere {
            low,
            height,
            radius,
        }));
    }

    /// `DetectionVolume::CreateSphere`.
    pub fn create_sphere(&mut self, radius: f32, center: Vector) {
        self.shape = Some(DetectionShape::Sphere(Sphere { center, radius }));
    }

    /// `DetectionVolume::GetGlobalBox`: the box around the volume in its landblock's axes, or
    /// `None` for a volume not yet placed (cell 0) or without a shape.
    pub fn global_box(&self) -> Option<BBox> {
        if self.position.cell_id == 0 {
            return None;
        }
        let pos = &self.position;
        Some(match self.shape? {
            DetectionShape::Sphere(s) => {
                let vc = pos.local_to_global(pos, s.center);
                let r = s.radius;
                let high = Vector::new(
                    st(ld(vc.x) + ld(r)),
                    st(ld(vc.y) + ld(r)),
                    st(ld(vc.z) + ld(r)),
                );
                let mut b = BBox {
                    min: high,
                    max: high,
                };
                b.adjust(Vector::new(
                    st(ld(vc.x) - ld(r)),
                    st(ld(vc.y) - ld(r)),
                    st(ld(vc.z) - ld(r)),
                ));
                b
            }
            DetectionShape::BoundBox(b) => b.local_to_global(pos),
            DetectionShape::Cylinder(c) => cylinder_global_box(&c, pos),
        })
    }

    /// `DetectionVolume::IsWithin(Sphere*, Position*, Vector* scale)`: the sphere, of an object
    /// at `at` scaled by `scale` (uniformly, else not at all), meets the volume.
    pub fn is_within_sphere(&self, sphere: &Sphere, at: &Position, scale: Vector) -> bool {
        let s = if scale.x != scale.y || scale.x != scale.z {
            1.0
        } else {
            scale.z
        };
        let local = Vector::new(
            st(ld(s) * ld(sphere.center.x)),
            st(ld(s) * ld(sphere.center.y)),
            st(ld(s) * ld(sphere.center.z)),
        );
        let global = Sphere {
            center: self.position.local_to_local(at, local),
            radius: st(ld(s) * ld(sphere.radius)),
        };
        match self.shape {
            Some(DetectionShape::Sphere(v)) => {
                let dx = ld(st(ld(v.center.x) - ld(global.center.x)));
                let dy = ld(st(ld(v.center.y) - ld(global.center.y)));
                let dz = ld(st(ld(v.center.z) - ld(global.center.z)));
                let reach = ld(st(ld(global.radius) + ld(v.radius)));
                (dx * dx + dy * dy + dz * dz) - reach * reach < 0.0002
            }
            Some(DetectionShape::BoundBox(b)) => box_intersects_sphere(&b, &global),
            Some(DetectionShape::Cylinder(c)) => sphere_intersects_cylinder(&global, &c),
            None => false,
        }
    }

    /// `DetectionVolume::IsWithin(Cylsphere*, Position*)`.
    pub fn is_within_cylinder(&self, cylinder: &CylSphere, at: &Position) -> bool {
        let global = CylSphere {
            low: self.position.local_to_local(at, cylinder.low),
            height: cylinder.height,
            radius: cylinder.radius,
        };
        match self.shape {
            Some(DetectionShape::Sphere(v)) => sphere_intersects_cylinder(&v, &global),
            Some(DetectionShape::BoundBox(b)) => box_intersects_cylinder(&b, &global),
            Some(DetectionShape::Cylinder(c)) => cylinder_intersects_cylinder(&c, &global),
            None => false,
        }
    }

    /// `DetectionVolume::IsWithin(BBox*, Position*)`: the box's two corners are moved into the
    /// volume's frame as they are, not re-sorted.
    pub fn is_within_box(&self, b: &BBox, at: &Position) -> bool {
        let global = BBox {
            min: self.position.local_to_local(at, b.min),
            max: self.position.local_to_local(at, b.max),
        };
        match self.shape {
            Some(DetectionShape::Sphere(v)) => box_intersects_sphere(&global, &v),
            Some(DetectionShape::BoundBox(v)) => box_does_overlap(&v, &global, 0.0),
            Some(DetectionShape::Cylinder(c)) => box_intersects_cylinder(&global, &c),
            None => false,
        }
    }
}

/// `Cylsphere::BuildBoundingBox`: an axis-aligned box around the cylinder at `pos`. When the
/// frame keeps the cylinder's axis out of the x direction (retail tests only the x component of
/// the turned axis, twice, and not y) the box is the two end points grown by the radius; else the
/// local box of the cylinder is turned into the global frame.
fn cylinder_global_box(c: &CylSphere, pos: &Position) -> BBox {
    let m = pos.frame.m.map(ld);
    let h = ld(c.height);
    let axis_x = (h * m[6] + (m[3] + m[0]) * 0.0).abs();
    let r = c.radius;
    if axis_x < 0.0002 {
        let low = pos.local_to_global(pos, c.low);
        let high_local = Vector::new(c.low.x, c.low.y, st(h + ld(c.low.z)));
        let high = pos.local_to_global(pos, high_local);
        let start = Vector::new(st(ld(low.x) + ld(r)), st(ld(low.y) + ld(r)), low.z);
        let mut b = BBox {
            min: start,
            max: start,
        };
        b.adjust(Vector::new(
            st(ld(high.x) - ld(r)),
            st(ld(high.y) - ld(r)),
            high.z,
        ));
        b
    } else {
        let start = Vector::new(st(ld(r) + ld(c.low.x)), st(ld(r) + ld(c.low.y)), c.low.z);
        let mut local = BBox {
            min: start,
            max: start,
        };
        local.adjust(Vector::new(
            st(ld(c.low.x) - ld(r)),
            st(ld(c.low.y) - ld(r)),
            st(h + ld(c.low.z)),
        ));
        local.local_to_global(pos)
    }
}

#[cfg(test)]
#[path = "detection_volume_tests.rs"]
mod tests;
