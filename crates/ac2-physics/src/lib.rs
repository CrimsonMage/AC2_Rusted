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
mod body;
#[cfg(test)]
mod body_anim_tests;
mod bsp;
mod cells;
mod collision_impact;
#[cfg(test)]
mod collision_impact_tests;
mod constraint;
mod envcell;
#[cfg(test)]
mod envcell_real_data_tests;
#[cfg(test)]
mod envcell_tests;
mod find_cells;
mod fpu;
mod frame;
mod heading;
mod interp;
pub mod landdefs;
mod loader;
mod loader_anim;
mod look_at;
mod los;
mod motion_interp;
mod motion_interp_behaviors;
mod moveto;
mod objects;
#[cfg(test)]
mod objects_real_data_tests;
#[cfg(test)]
mod objects_tests;
mod physics_obj;
mod physics_obj_contact;
mod physics_obj_look;
mod physics_obj_place;
#[cfg(test)]
mod physics_obj_tests;
mod position;
#[cfg(test)]
mod real_data_tests;
mod shapes;
mod terrain;
#[cfg(test)]
mod test_support;
mod transition;
mod vector;

pub use anim_rep::BodyAnimation;
pub use body::{
    AnimationClock, MovementInput, MovementState, PhysicsBody, PhysicsClock, Tuning, ANIM_QUANTUM,
    PHYSICS_QUANTUM,
};
pub use bsp::{BspTree, Polygon};
pub use cells::{CellSource, LandCell, TerrainPolygon};
pub use collision_impact::ImpactReport;
pub use envcell::{CellBsp, CellBspNode, CellPortal, CellStructure, EnvCell};
pub use frame::Frame;
pub use heading::Heading;
pub use loader::{
    load_block_interiors, load_block_objects, load_body_shape, load_object_shape,
    load_terrain_block,
};
pub use loader_anim::load_body_animation;
pub use look_at::LookTarget;
pub use los::{LosParameters, Ray, RayIntersection};
pub use motion_interp::MotionInterp;
pub use moveto::{
    MoveToResult, MovementParameters, PackedMovementParameters, MOVE_TO_AWAY, MOVE_TO_MOVE,
    MOVE_TO_OBJECT, MOVE_TO_PERSISTENT, MOVE_TO_TURN, MOVE_TO_USE_CYLINDERS,
};
pub use objects::{BBox, ObjectShape, StaticObject};
pub use physics_obj::{BodyShape, DescSlider};
pub use physics_obj_place::{SetPositionError, FORCE_SPF, SLIDE_SPF};
pub use position::Position;
pub use shapes::cylsphere::CylSphere;
pub use terrain::Terrain;
pub use vector::{Plane, Quaternion, Sphere, Vector};
