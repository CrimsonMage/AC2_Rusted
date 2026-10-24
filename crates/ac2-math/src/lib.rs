//! Pure math vocabulary and narrow geometry helpers for AC2_Rusted.

mod box2;
mod clip;
mod frame;
mod geometry;

pub use frame::Frame;
pub use geometry::{BBox, Plane, Sphere};
pub use glam::{EulerRot, Mat4, Quat, Vec2, Vec3, Vec4};
