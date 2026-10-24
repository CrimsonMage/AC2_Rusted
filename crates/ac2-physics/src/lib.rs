#![forbid(unsafe_code)]

//! Asheron's Call 2 physics: the `CPhysicsObj` movement step, `CMotionInterp` locomotion and
//! `CTransition` collision, ported from the retail client (admin 7148 decompile, checked against
//! the Legions 7160 DVD client instruction by instruction).
//!
//! Positions are retail `Position`s: a cell id and a frame in AC axes (z up) relative to the
//! cell's landblock. Arithmetic follows the retail x87 model (see `fpu`).

mod body;
mod bsp;
mod cells;
mod envcell;
#[cfg(test)]
mod envcell_real_data_tests;
#[cfg(test)]
mod envcell_tests;
mod find_cells;
mod fpu;
mod frame;
mod heading;
mod landdefs;
mod loader;
mod motion_interp;
mod objects;
#[cfg(test)]
mod objects_real_data_tests;
#[cfg(test)]
mod objects_tests;
mod physics_obj;
mod physics_obj_contact;
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

pub use body::{MovementInput, MovementState, PhysicsBody, PhysicsClock, PHYSICS_QUANTUM};
pub use bsp::{BspTree, Polygon};
pub use cells::{CellSource, LandCell, TerrainPolygon};
pub use envcell::{CellBsp, CellBspNode, CellPortal, CellStructure, EnvCell};
pub use frame::Frame;
pub use heading::Heading;
pub use loader::{
    load_block_interiors, load_block_objects, load_body_shape, load_object_shape,
    load_terrain_block,
};
pub use motion_interp::MotionInterp;
pub use objects::{BBox, ObjectShape, StaticObject};
pub use physics_obj::BodyShape;
pub use physics_obj_place::{SetPositionError, FORCE_SPF, SLIDE_SPF};
pub use position::Position;
pub use shapes::cylsphere::CylSphere;
pub use terrain::Terrain;
pub use vector::{Plane, Quaternion, Sphere, Vector};
