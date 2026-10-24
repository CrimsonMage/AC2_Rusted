#![forbid(unsafe_code)]

//! Asheron's Call 2 physics: the `CPhysicsObj` movement step, `CMotionInterp` locomotion and
//! `CTransition` collision, ported from the retail client (admin 7148 decompile, checked against
//! the Legions 7160 DVD client instruction by instruction).
//!
//! Positions are retail `Position`s: a cell id and a frame in AC axes (z up) relative to the
//! cell's landblock. Arithmetic follows the retail x87 model (see `fpu`).

mod body;
mod cells;
mod fpu;
mod frame;
mod heading;
mod landdefs;
mod loader;
mod motion_interp;
mod physics_obj;
mod physics_obj_contact;
#[cfg(test)]
mod physics_obj_tests;
mod position;
#[cfg(test)]
mod real_data_tests;
mod terrain;
#[cfg(test)]
mod test_support;
mod transition;
mod vector;

pub use body::{MovementInput, MovementState, PhysicsBody, PhysicsClock, PHYSICS_QUANTUM};
pub use cells::{CellSource, LandCell, TerrainPolygon};
pub use frame::Frame;
pub use heading::Heading;
pub use loader::{load_body_shape, load_terrain_block};
pub use motion_interp::MotionInterp;
pub use physics_obj::BodyShape;
pub use position::Position;
pub use terrain::Terrain;
pub use vector::{Plane, Quaternion, Sphere, Vector};
