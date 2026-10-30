//! Static world objects for collision: the `CPhysicsObj`s a landblock creates for its entities
//! (`CLandBlock::CreateStaticObjects`) with their Setup shape (`CSetup` physics: part BSPs,
//! `Cylsphere`s, `Sphere`s), and `CPhysicsObj::FindObjCollisions`, which a transition runs for
//! every object in the shadow lists of the cells it touches.

use std::sync::Arc;

use crate::bsp::{BspTree, ContactKind};
use crate::cells::CellSource;
use crate::collision_types::{CollisionParameters, PhysicsTypeMask};
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

    /// The box grown by `margin` on every side.
    pub(crate) fn grown(self, margin: f32) -> Self {
        Self {
            min: Vector::new(
                self.min.x - margin,
                self.min.y - margin,
                self.min.z - margin,
            ),
            max: Vector::new(
                self.max.x + margin,
                self.max.y + margin,
                self.max.z + margin,
            ),
        }
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

/// One object in the world's collision lists: a static object (`CLandBlock::CreateStaticObjects`)
/// or a server object's body (`CObjCell::AddObject` for a `CPhysicsObj` that is not static). Its
/// position, total scale (`CPhysicsObj::GetTotalScale`), `PhysicsState`, shape, and whether it is
/// ethereal to every mover (an FX object, `FXManager::DoCreateObjectFX`).
#[derive(Debug, Clone, PartialEq)]
pub struct StaticObject {
    pub(crate) position: Position,
    pub(crate) scale: Vector,
    pub(crate) state: u32,
    /// `CPhysicsObj::m_fPhysicsContainmentRadius` (`SetPhysicsContainmentRadius`).
    pub(crate) containment_radius: f32,
    pub(crate) shape: Arc<ObjectShape>,
    /// The object's blocking physics-type mask (`CPhysicsObj::m_blocking_mask`), set from its
    /// ethereal properties (`CPhysicsObj::ApplyEtherealProperty`). Zero unless set.
    pub(crate) blocking: PhysicsTypeMask,
    /// `ETHEREAL` as `FXManager::DoCreateObjectFX` sets it: no mover is blocked by it, whatever
    /// its blocking mask says.
    pub(crate) ethereal: bool,
    /// `CPhysicsObj::ignore_collisions` (`m_state` bit 8).
    pub(crate) ignore_collisions: bool,
    /// The position its cell list was built for, while the list may be reused (`None` for a
    /// static object, which never moves). See `Terrain::move_object`.
    pub(crate) covered: Option<Position>,
    /// The object's `InstanceID` when it is a server object a collision report names
    /// (`CCollisionManager::AddObjectCollision` lists only objects with an id). `None` for
    /// scenery.
    pub(crate) instance_id: Option<u64>,
    /// The object's own collision parameters (`GetCollisionStatePtr`), when its body has a
    /// collision manager: the second direction of `CPhysicsObj::Interpermiable`.
    pub(crate) collision: Option<CollisionParameters>,
    /// The property groups of the object's body (`GetObjectProperties`), which a mover touching
    /// it lists in its collision report (`CCollisionManager::SetProperties`).
    pub(crate) properties: Vec<ac2_db::Ac2PropertyGroup>,
}

impl StaticObject {
    /// A static object (`STATIC_PS`, with `HAS_PHYSICS_BSP_PS` when its shape has BSPs).
    pub fn new(position: Position, scale: Vector, shape: Arc<ObjectShape>) -> Self {
        Self::with_state(position, scale, shape, STATIC_PS)
    }

    /// A server object's body in the world: not `STATIC_PS`, so it moves under physics; mobile
    /// (`MOBILE_PS`) bodies can be stood on only when the mover is ethereal to them, as
    /// `CPhysicsObj::FindObjCollisions` decides.
    pub fn dynamic(
        position: Position,
        scale: Vector,
        shape: Arc<ObjectShape>,
        mobile: bool,
    ) -> Self {
        let state = if mobile { MOBILE_PS } else { 0 };
        Self::with_state(position, scale, shape, state)
    }

    fn with_state(position: Position, scale: Vector, shape: Arc<ObjectShape>, state: u32) -> Self {
        let (x, y, z) = (ld(scale.x), ld(scale.y), ld(scale.z));
        let length = ld(st((x * x + y * y) + z * z)).sqrt();
        let mut state = state;
        if !shape.bsps.is_empty() {
            state |= HAS_PHYSICS_BSP_PS;
        }
        Self {
            position,
            scale,
            state,
            containment_radius: st(length * ld(shape.containment_radius)),
            shape,
            blocking: PhysicsTypeMask::default(),
            ethereal: false,
            ignore_collisions: false,
            covered: None,
            instance_id: None,
            collision: None,
            properties: Vec::new(),
        }
    }

    /// Names the object by its server `InstanceID`, so collision reports list it.
    pub fn with_instance_id(mut self, id: u64) -> Self {
        self.instance_id = Some(id);
        self
    }

    /// The object's `InstanceID` and whether it is ethereal, when it has an id.
    pub(crate) fn identity(&self) -> Option<(u64, bool)> {
        self.instance_id.map(|id| (id, self.ethereal))
    }

    /// Sets the blocking physics-type bits (`CPhysicsObj::SetPhysicsCollideMask`).
    pub fn with_blocking_mask(mut self, bits: u64) -> Self {
        self.blocking = PhysicsTypeMask::from_bits(bits);
        self
    }

    /// `CPhysicsObj::ignore_collisions` of the body this entry stands for.
    pub fn set_ignore_collisions(&mut self, ignore: bool) {
        self.ignore_collisions = ignore;
    }

    pub(crate) fn ignores_collisions(&self) -> bool {
        self.ignore_collisions
    }

    /// `FXManager::DoCreateObjectFX`'s ETHEREAL: the object is passed through by every mover
    /// (outside a placement, where the overlap test still runs).
    pub fn set_ethereal(&mut self, ethereal: bool) {
        self.ethereal = ethereal;
    }

    /// Whether the object is static (`STATIC_PS`): it never moves, and its cells are fixed.
    pub fn is_static(&self) -> bool {
        self.state & STATIC_PS != 0
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
        !self.is_static()
    }

    fn is_mobile(&self) -> bool {
        self.state & MOBILE_PS != 0
    }

    /// `CPhysicsObj::Interpermiable(mover, 0)` as the transition asks it: the object is ethereal
    /// to the mover when its ETHEREAL state is set (an FX object, `FXManager::DoCreateObjectFX`),
    /// when the mover's set does not make the object solid (`CheckIfSolid`: `blocks` by the
    /// object's `InstanceID` for the solid and ethereal lists, and the mover's temporary ethereal
    /// list, `AddToTempEtherealList`), or when the object has a collision manager of its own
    /// and its set does not make the mover (`mover_id`, `mover_blocking`) solid.
    fn is_ethereal_to(
        &self,
        collision: &CollisionParameters,
        mover: (u64, PhysicsTypeMask),
        placement: bool,
    ) -> bool {
        if self.ethereal {
            return true;
        }
        let id = self.instance_id.unwrap_or(0);
        if !collision.check_if_solid(id, self.blocking, placement) {
            return true;
        }
        self.collision
            .as_ref()
            .is_some_and(|own| !own.check_if_solid(mover.0, mover.1, false))
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
            // `pIVar1 != transition->object_info.object`: the mover is not its own obstruction.
            if self.identity == Some(id) {
                continue;
            }
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
        // is ethereal to it (`StaticObject::is_ethereal_to`, which also reads the ETHEREAL state).
        // Outside a placement the transition passes through it; a placement keeps the overlap test
        // with the obstruction marked ethereal.
        let placement = self.sphere_path.placement_ethereal;
        let ethereal = object.is_ethereal_to(&self.collision, self.mover, placement);
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
            if object.instance_id.is_none() {
                // Scenery: `AddObjectCollision` adds its blocking mask to the mover's current
                // collide mask when the two pass through one another.
                self.collision_info.scenery.push(object.blocking);
            }
            self.collision_info
                .add_object(id, self.sphere_path.obstruction_ethereal);
        }
        self.sphere_path.obstruction_ethereal = false;
        ts
    }
}
