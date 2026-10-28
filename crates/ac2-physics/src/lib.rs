#![forbid(unsafe_code)]

//! Asheron's Call 2 physics: the `CPhysicsObj` movement step, `CMotionInterp` locomotion and
//! `CTransition` collision, ported from the retail client (admin 7148 decompile, checked against
//! the Legions 7160 DVD client instruction by instruction).
//!
//! Positions are retail `Position`s: a cell id and a frame in AC axes (z up) relative to the
//! cell's landblock. Arithmetic follows the retail x87 model (see `fpu`).

mod anim_pose;
#[cfg(test)]
mod anim_real_data_tests;
mod anim_rep;
mod appearance;
mod appearance_delta;
mod body;
#[cfg(test)]
mod body_anim_tests;
mod bsp;
mod cells;
mod collide_property;
mod collision_impact;
#[cfg(test)]
mod collision_impact_tests;
mod collision_report;
#[cfg(test)]
mod collision_report_tests;
mod collision_types;
#[cfg(test)]
mod collision_types_tests;
mod constraint;
mod envcell;
#[cfg(test)]
mod envcell_real_data_tests;
#[cfg(test)]
mod envcell_tests;
mod find_cells;
mod fpu;
mod frame;
mod fx_follow;
mod heading;
mod held;
mod interp;
pub mod landdefs;
mod loader;
mod loader_anim;
mod look_at;
mod los;
mod missile;
mod motion;
mod motion_interp;
mod motion_interp_behaviors;
#[cfg(test)]
mod motion_tests;
mod moveto;
mod objects;
#[cfg(test)]
mod objects_dynamic_tests;
#[cfg(test)]
mod objects_real_data_tests;
#[cfg(test)]
mod objects_tests;
mod part_pose;
mod physics_material;
mod physics_obj;
mod physics_obj_contact;
mod physics_obj_look;
mod physics_obj_place;
#[cfg(test)]
mod physics_obj_tests;
mod position;
mod pushers;
#[cfg(test)]
mod real_data_tests;
mod shapes;
mod target;
mod terrain;
#[cfg(test)]
mod terrain_material_tests;
#[cfg(test)]
mod test_support;
mod transition;
mod vector;

pub use anim_rep::BodyAnimation;
pub use appearance::{Appearance, PartGroupAppearance, GLOBAL_PART_GROUP};
pub use appearance_delta::local_visual_desc;
pub use body::{
    AnimationClock, MovementInput, MovementState, PhysicsBody, PhysicsClock, Tuning, ANIM_QUANTUM,
    PHYSICS_QUANTUM,
};
pub use bsp::{BspTree, Polygon};
pub use cells::{CellSource, LandCell, TerrainPolygon};
pub use collide_property::{CollideProperty, CollideType, PropertyDescs, TERRAIN_TYPE_PROPERTY};
pub use collision_impact::ImpactReport;
pub use collision_report::{
    CollisionReport, ContactMaterials, ContactRecord, ObjectRecord, RecordState,
};
pub use collision_types::{CollisionParameters, InstanceId, PhysicsTypeMask};
pub use envcell::{CellBsp, CellBspNode, CellPortal, CellStructure, EnvCell};
pub use frame::Frame;
pub use fx_follow::{follow_frame, placed_at, FxFollow};
pub use heading::Heading;
pub use held::{held_place, placement_order, HoldPoint};
pub use loader::{
    load_block_interiors, load_block_objects, load_body_shape, load_object_scale,
    load_object_shape, load_terrain_block,
};
pub use loader_anim::load_body_animation;
pub use look_at::LookTarget;
pub use los::{LosParameters, Ray, RayIntersection};
pub use missile::{
    chase_velocity, get_gravitational_velocity, get_max_time, get_min_time, MissileHost,
    MissileManager, MissileParameters, TargetPosition, MISSILE_FLIGHT_FX,
};
pub use motion::{apply_motion, apply_motion_from, apply_motion_owned, MotionEvent};
pub use motion_interp::MotionInterp;
pub use moveto::{
    MoveToResult, MovementParameters, PackedMovementParameters, MOVE_TO_AWAY, MOVE_TO_MOVE,
    MOVE_TO_OBJECT, MOVE_TO_PERSISTENT, MOVE_TO_TURN, MOVE_TO_USE_CYLINDERS,
};
pub use objects::{BBox, ObjectShape, StaticObject};
pub use part_pose::{drawn_place, part_poses, PartPose};
pub use physics_material::{load_physics_material, PhysicsMaterial};
pub use physics_obj::{BodyShape, DescSlider};
pub use physics_obj_place::{SetPositionError, FORCE_SPF, SLIDE_SPF};
pub use position::Position;
pub use pushers::{PusherSum, PusherValues, Pushers};
pub use shapes::cylsphere::CylSphere;
pub use target::{
    TargetEffect, TargetInfo, TargetManager, TargetOwner, TargetStatus, TargetVoyeur,
};
pub use terrain::Terrain;
pub use vector::{Plane, Quaternion, Sphere, Vector};
