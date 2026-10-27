//! Static world objects for collision: the `CPhysicsObj`s a landblock creates for its entities
//! (`CLandBlock::CreateStaticObjects`) with their Setup shape (`CSetup` physics: part BSPs,
//! `Cylsphere`s, `Sphere`s), and `CPhysicsObj::FindObjCollisions`, which a transition runs for
//! every object in the shadow lists of the cells it touches.

use std::sync::Arc;

use crate::bsp::{BspTree, ContactKind};
use crate::cells::CellSource;
use crate::collision_types::PhysicsTypeMask;
use crate::fpu::{ld, st};
use crate::position::Position;
use crate::shapes::cylsphere::CylSphere;
use crate::transition::collision::PREDETERMINED_TRANSITION_OI;
use crate::transition::{Transition, TransitionState};
use crate::vector::{Sphere, Vector};

/// `PhysicsState` bits of static objects.
pub(crate) const STATIC_PS: u32 = 0x1;
pub(crate) const CANNOT_STAND_ON_PS: u32 = 0x2;
pub(crate) const HAS_PHYSICS_BSP_PS: u32 = 0x1000;
pub(crate) const MOBILE_PS: u32 = 0x20_0000;
pub(crate) const PHYSICSDISABLED_PS: u32 = 0x2000_0000;

/// An axis-aligned box (`BBox`), `min` and `max` corners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub min: Vector,
    pub max: Vector,
}

impl BBox {
    /// `BBox::InitForAdjustment`: empty, ready for `adjust`.
    pub(crate) const EMPTY: Self = Self {
        min: Vector::new(f32::MAX, f32::MAX, f32::MAX),
        max: Vector::new(-f32::MAX, -f32::MAX, -f32::MAX),
    };

    /// `BBox::AdjustBBox`: grow to hold `v`.
    pub(crate) fn adjust(&mut self, v: Vector) {
        let grow = |low: &mut f32, high: &mut f32, value: f32| {
            if value < *low {
                *low = value;
            }
            if value > *high {
                *high = value;
            }
        };
        grow(&mut self.min.x, &mut self.max.x, v.x);
        grow(&mut self.min.y, &mut self.max.y, v.y);
        grow(&mut self.min.z, &mut self.max.z, v.z);
    }

    /// `BBox::localtoglobal` with one position for both frames: the box around the eight
    /// corners moved by `pos`'s frame.
    pub(crate) fn local_to_global(&self, pos: &Position) -> Self {
        let (a, b) = (self.min, self.max);
        let mut out = Self::EMPTY;
        for corner in [
            a,
            b,
            Vector::new(b.x, b.y, a.z),
            Vector::new(b.x, a.y, b.z),
            Vector::new(a.x, b.y, b.z),
            Vector::new(a.x, b.y, a.z),
            Vector::new(b.x, a.y, a.z),
            Vector::new(a.x, a.y, b.z),
        ] {
            out.adjust(pos.local_to_global(pos, corner));
        }
        out
    }
}

/// The collision shape a Setup gives the objects that use it: the physics BSPs of its meshes,
/// its cylinders and spheres, and the containment radius and box `CharacterSetup::InitFileLoad`
/// and `CSetup::CachePhysicsInformation` derive from them (unscaled).
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectShape {
    pub(crate) bsps: Vec<Arc<BspTree>>,
    pub(crate) cylspheres: Vec<CylSphere>,
    pub(crate) spheres: Vec<Sphere>,
    pub(crate) containment_radius: f32,
    pub(crate) bounds: BBox,
}

impl ObjectShape {
    /// A shape from its parts. The containment radius covers the spheres and cylinders, or, when
    /// the Setup has neither, the BSP vertices; the box always holds the origin.
    pub fn new(bsps: Vec<Arc<BspTree>>, cylspheres: Vec<CylSphere>, spheres: Vec<Sphere>) -> Self {
        let mut radius = 0.0_f32;
        let mut keep = |value: f64| {
            if !(value <= ld(radius)) {
                radius = st(value);
            }
        };
        let mut bounds = BBox::EMPTY;
        if spheres.is_empty() && cylspheres.is_empty() {
            for tree in &bsps {
                for polygon in &tree.polygons {
                    for &v in &polygon.vertices {
                        let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
                        keep(((x * x + y * y) + z * z).sqrt());
                        bounds.adjust(v);
                    }
                }
            }
        } else {
            let mut min = Vector::new(f32::MAX, f32::MAX, f32::MAX);
            let mut max = Vector::new(-f32::MAX, -f32::MAX, -f32::MAX);
            let low = |slot: &mut f32, v: f32| *slot = slot.min(v);
            for s in &spheres {
                let (c, r) = (s.center, ld(s.radius));
                low(&mut min.x, st(ld(c.x) - r));
                max.x = max.x.max(st(r + ld(c.x)));
                low(&mut min.y, st(ld(c.y) - r));
                max.y = max.y.max(st(r + ld(c.y)));
                low(&mut min.z, st(ld(c.z) - r));
                max.z = max.z.max(st(r + ld(c.z)));
                let (x, y, z) = (ld(c.x), ld(c.y), ld(c.z));
                keep(((x * x + z * z) + y * y).sqrt() + r);
            }
            for c in &cylspheres {
                let (l, r, h) = (c.low, ld(c.radius), ld(c.height));
                low(&mut min.x, st(ld(l.x) - r));
                max.x = max.x.max(st(r + ld(l.x)));
                low(&mut min.y, st(ld(l.y) - r));
                max.y = max.y.max(st(ld(l.y) + r));
                low(&mut min.z, l.z);
                max.z = max.z.max(st(h + ld(l.z)));
                let (x, y, z) = (ld(l.x), ld(l.y), ld(l.z));
                keep(((z * z + x * x) + y * y).sqrt() + r);
                let top = h + z;
                keep(((x * x + y * y) + top * top).sqrt() + r);
            }
            bounds.adjust(min);
            bounds.adjust(max);
        }
        bounds.adjust(Vector::ZERO);
        Self {
            bsps,
            cylspheres,
            spheres,
            containment_radius: radius,
            bounds,
        }
    }
}

/// One static object: its position, total scale (`CPhysicsObj::GetTotalScale`), `PhysicsState`
/// and shape.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticObject {
    pub(crate) position: Position,
    pub(crate) scale: Vector,
    pub(crate) state: u32,
    /// `CPhysicsObj::m_fPhysicsContainmentRadius` (`SetPhysicsContainmentRadius`).
    pub(crate) containment_radius: f32,
    pub(crate) shape: Arc<ObjectShape>,
}

impl StaticObject {
    /// A static object (`STATIC_PS`, with `HAS_PHYSICS_BSP_PS` when its shape has BSPs).
    pub fn new(position: Position, scale: Vector, shape: Arc<ObjectShape>) -> Self {
        let (x, y, z) = (ld(scale.x), ld(scale.y), ld(scale.z));
        let length = ld(st((x * x + y * y) + z * z)).sqrt();
        let mut state = STATIC_PS;
        if !shape.bsps.is_empty() {
            state |= HAS_PHYSICS_BSP_PS;
        }
        Self {
            position,
            scale,
            state,
            containment_radius: st(length * ld(shape.containment_radius)),
            shape,
        }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    /// The box, in the object's frame, whose cells hold the object in their shadow lists
    /// (`CPhysicsObj::calc_cross_cells`): the scaled physics box for BSP objects, the unscaled
    /// cylinders or spheres otherwise.
    pub fn shadow_box(&self) -> BBox {
        let shape = &self.shape;

        if self.state & HAS_PHYSICS_BSP_PS != 0 {
            let (b, s) = (shape.bounds, self.scale);
            BBox {
                min: Vector::new(
                    st(ld(b.min.x) * ld(s.x)),
                    st(ld(b.min.y) * ld(s.y)),
                    st(ld(s.z) * ld(b.min.z)),
                ),
                max: Vector::new(
                    st(ld(b.max.x) * ld(s.x)),
                    st(ld(b.max.y) * ld(s.y)),
                    st(ld(s.z) * ld(b.max.z)),
                ),
            }
        } else if !shape.cylspheres.is_empty() || !shape.spheres.is_empty() {
            let mut b = BBox::EMPTY;
            for c in &shape.cylspheres {
                let (l, r) = (c.low, ld(c.radius));
                b.adjust(Vector::new(st(ld(l.x) - r), st(ld(l.y) - r), l.z));
                b.adjust(Vector::new(
                    st(ld(l.x) + r),
                    st(ld(l.y) + r),
                    st(ld(c.height) + ld(l.z)),
                ));
            }
            for s in &shape.spheres {
                let (c, r) = (s.center, ld(s.radius));
                b.adjust(Vector::new(
                    st(ld(c.x) - r),
                    st(ld(c.y) - r),
                    st(ld(c.z) - r),
                ));
                b.adjust(Vector::new(
                    st(r + ld(c.x)),
                    st(r + ld(c.y)),
                    st(r + ld(c.z)),
                ));
            }
            b
        } else {
            BBox {
                min: Vector::ZERO,
                max: Vector::ZERO,
            }
        }
    }

    fn is_dynamic(&self) -> bool {
        self.state & STATIC_PS == 0
    }

    fn is_mobile(&self) -> bool {
        self.state & MOBILE_PS != 0
    }

    /// `CPhysicsObj::CanCollide`: the mover at `check` (containment `mover`) is within the
    /// sum of both containment radii and the step height of the object.
    fn can_collide(&self, step: f32, mover: f32, check: &Position) -> bool {
        let diff = check.offset_to(&self.position);
        let reach = (ld(self.containment_radius) + ld(step)) + ld(mover);
        let (x, y, z) = (ld(diff.x), ld(diff.y), ld(diff.z));
        !(reach * reach < (z * z + y * y) + x * x)
    }
}

impl Transition {
    /// `CObjCell::find_obj_collisions`: the cell's shadow objects in order, until one does not
    /// leave the transition `Ok`. Nothing collides on an initial placement.
    pub(crate) fn find_obj_collisions(
        &mut self,
        objects: &[u32],
        cells: &dyn CellSource,
    ) -> TransitionState {
        if self.sphere_path.insert_type
            == crate::transition::sphere_path::InsertType::InitialPlacement
        {
            return TransitionState::Ok;
        }
        for &id in objects {
            let Some(object) = cells.object(id) else {
                continue;
            };
            let ts = self.find_object_collisions(id, object, cells);
            if ts != TransitionState::Ok {
                return ts;
            }
        }
        TransitionState::Ok
    }

    /// `CPhysicsObj::FindObjCollisions` for a static object: once per insertion pass, within
    /// reach, through its BSPs (`CPartArray::FindObjCollisions`), else its cylinders, else its
    /// spheres; the object is recorded when it stops the transition.
    fn find_object_collisions(
        &mut self,
        id: u32,
        object: &StaticObject,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let stamp = self.physics_timestamp;
        if self.object_stamps.insert(id, stamp) == Some(stamp)
            || object.state & PHYSICSDISABLED_PS != 0
        {
            return TransitionState::Ok;
        }
        let dynamic_mobile = object.is_dynamic() && object.is_mobile();
        if self.object_info.state & PREDETERMINED_TRANSITION_OI != 0 && !dynamic_mobile {
            return TransitionState::Ok;
        }
        let cannot_stand_on = object.state & CANNOT_STAND_ON_PS != 0
            || (dynamic_mobile && !self.sphere_path.placement_ethereal);
        let step = self.object_info.max_step;
        let mover = self.object_info.containment_radius;
        if !object.can_collide(step, mover, &self.sphere_path.check_pos) {
            return TransitionState::Ok;
        }
        // `CPhysicsObj::IsBlocking` against the mover's sets: an object the mover is not blocked by
        // is ethereal to it. Outside a placement the transition passes through it; a placement
        // keeps the overlap test with the obstruction marked ethereal. A static object's own
        // blocking mask (`CreateStaticObjects`'s ethereal physics type) is not set, so it is zero.
        let placement = self.sphere_path.placement_ethereal;
        let ethereal = !self
            .collision
            .blocks(u64::from(id), PhysicsTypeMask::default(), placement);
        if ethereal && !placement {
            return TransitionState::Ok;
        }
        self.sphere_path.obstruction_ethereal = ethereal;
        let (p, scale, shape) = (object.position, object.scale, &object.shape);
        let mut ts = TransitionState::Ok;
        if object.state & HAS_PHYSICS_BSP_PS != 0 {
            let inverse = Vector::new(
                st(1.0 / ld(scale.x)),
                st(1.0 / ld(scale.y)),
                st(1.0 / ld(scale.z)),
            );
            for tree in &shape.bsps {
                self.sphere_path.cache_localspace_sphere(&p, inverse);
                ts = tree.find_collisions(self, scale, inverse, ContactKind::default(), cells);
                if ts != TransitionState::Ok {
                    break;
                }
            }
        } else if !shape.cylspheres.is_empty() {
            for cylinder in &shape.cylspheres {
                ts = cylinder.intersects_object(self, &p, scale, cells);
                if ts != TransitionState::Ok {
                    break;
                }
            }
        } else {
            for &sphere in &shape.spheres {
                ts = self.intersects_object_sphere(sphere, &p, scale, cannot_stand_on, cells);
                if ts != TransitionState::Ok {
                    break;
                }
            }
        }
        if ts != TransitionState::Ok && !self.sphere_path.step_down {
            self.collision_info.add_object(id);
        }
        ts
    }
}
