//! The player's body (`SmartBox::InitPlayer`): its collision shape, motion description,
//! animation and the facts the camera reads from it, loaded from a visual's DAT chain on a
//! loader thread.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_physics::{
    load_body_animation, load_body_shape, load_object_scale, load_object_shape, BodyAnimation,
    BodyShape, ObjectShape, PartGroupTree, PhysicsBody, Position, Vector,
};
use ac2_world::{load_ac2_motion_interp_desc, Ac2MotionInterpDesc};

use crate::camera::CameraSubject;

/// `m_midesc_did` of the human `VisualDesc`s, used when a visual names none.
pub const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;

/// What a body simulates with (the player's, or another object's).
#[derive(Clone)]
pub struct BodyData {
    pub shape: BodyShape,
    pub motion: Option<Arc<Ac2MotionInterpDesc>>,
    /// `CSetup::radius`, unscaled (`CPhysicsObj::GetRadius` scales it).
    pub radius: f32,
    /// What its Setup animates with (`PHYSICSOBJREP::SetupModeTracker`), when the visual has a
    /// behaviour table and an `AnimMap`.
    pub animation: Option<BodyAnimation>,
    /// Its collision shape as a world object (`CPartArray`'s physics BSPs, cylinders and spheres)
    /// and its root scale, when it has any: the object other movers collide with.
    pub object: Option<(Arc<ObjectShape>, Vector)>,
    /// The body's part-group tree (`PHYSICSOBJREP::m_partGroup`): its Setups' bones, spheres,
    /// cylspheres and holding locations. `None` for a visual that draws no groups.
    pub parts: Option<Arc<PartGroupTree>>,
}

/// A body's collision shape, motion description, radius and animation for `visual` (its
/// `VisualDesc` or `Setup`), read on a loader thread.
pub fn load_body_data(store: &Arc<ResourceStore>, visual: DataId) -> Result<BodyData, String> {
    let shape = load_body_shape(store, visual)
        .map_err(|e| e.to_string())?
        .ok_or("the visual has no Setup")?;
    let radius = match ac2_assets::resolve_ac2_visual_parts(store, visual)
        .map_err(|e| e.to_string())?
        .first()
    {
        Some((setup, _)) => store
            .load_cached_packed::<ac2_assets::Ac2SetupPhysics>(TypedResourceRef::new(
                DatFileKind::Portal,
                *setup,
                0,
            ))
            .map_err(|e| e.to_string())?
            .map_or(0.0, |physics| physics.radius),
        None => 0.0,
    };
    let midesc = ac2_assets::resolve_ac2_visual_motion_interp(store, visual)
        .map_err(|e| e.to_string())?
        .unwrap_or(DataId::new(HUMAN_MOTION_INTERP_DESC));
    let motion = load_ac2_motion_interp_desc(store, midesc)
        .map_err(|e| e.to_string())?
        .map(|desc| Arc::new((*desc).clone()));
    let animation = load_body_animation(store, visual).map_err(|e| e.to_string())?;
    let object = match load_object_shape(store, visual).map_err(|e| e.to_string())? {
        Some(shape) => Some((
            Arc::new(shape),
            load_object_scale(store, visual).map_err(|e| e.to_string())?,
        )),
        None => None,
    };
    let parts = ac2_physics::load_part_group_tree(store, visual)
        .map_err(|e| e.to_string())?
        .map(Arc::new);
    Ok(BodyData {
        shape,
        motion,
        radius,
        animation,
        object,
        parts,
    })
}

/// The player's physics object with the scaled size the camera needs.
pub struct PlayerBody {
    pub physics: PhysicsBody,
    /// `GetHeight` and `GetRadius`, scaled.
    pub height: f64,
    pub radius: f64,
    pub scale: f32,
}

impl PlayerBody {
    /// The body at `position` (`CPhysicsObj::IsThePlayer`, radius and animation set).
    pub fn new(position: Position, data: BodyData) -> Self {
        let scale = data.shape.scale;
        let height = f64::from(data.shape.height) * f64::from(scale);
        let radius = f64::from(data.radius) * f64::from(scale);
        let mut physics = PhysicsBody::new(position, data.shape, data.motion);
        physics.set_player(true);
        physics.set_radius(data.radius);
        if let Some(parts) = data.parts {
            physics.set_part_group(parts);
            // `CPhysicsObj::InitObjectEnd`: `PHYSICSOBJREP::CacheHasPhysicsBSP` sets the state bit.
            physics.cache_has_physics_bsp();
        }
        if let Some(animation) = data.animation {
            physics.set_animation(animation);
        }
        Self {
            physics,
            height,
            radius,
            scale,
        }
    }

    /// What the camera reads from the player's physics object, `id` its instance id.
    pub fn camera_subject(&self, id: u64) -> CameraSubject {
        self.subject_at(id, self.physics.position())
    }

    /// The camera's subject where the body is drawn, `t` of the way through the current quantum
    /// (`PhysicsBody::led_drawing_position`). The camera updates every frame against this
    /// (`AGENTS.md`): retail's `SmartBox::UseTime` read the current quantum, so the camera stepped
    /// a whole quantum (0.1 s of running) at a time while the body was drawn between quanta, and
    /// the two jittered against each other.
    pub fn drawn_camera_subject(&self, id: u64, t: f32) -> CameraSubject {
        self.subject_at(id, self.physics.led_drawing_position(t))
    }

    fn subject_at(&self, id: u64, position: ac2_physics::Position) -> CameraSubject {
        let state = self.physics.movement_state();
        let [vx, vy, vz] = state.velocity;
        CameraSubject {
            id,
            position,
            height: self.height,
            radius: self.radius,
            scale_z: self.scale,
            local_velocity: position.frame.global_to_local_vec(Vector::new(vx, vy, vz)),
            element: state.element,
            no_draw: false,
        }
    }
}
