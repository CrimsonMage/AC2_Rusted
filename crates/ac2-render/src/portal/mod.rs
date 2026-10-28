//! Portal space (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`): the
//! teleport's fade and tunnel effect, drawn in place of the world while the player is in transit.

pub mod assets;
pub mod camera;
pub mod clouds;
#[cfg(test)]
mod clouds_tests;
pub mod effect;
#[cfg(test)]
mod effect_tests;
pub mod tunnel;
#[cfg(test)]
mod tunnel_tests;

pub use assets::{load_clouds_desc, load_portal_models, portal_settings, PortalModels};
pub use camera::{
    camera_fov_y, portal_camera, PortalCamera, CAMERA_FAR, CAMERA_FOV_DEGREES, CAMERA_NEAR,
};
pub use clouds::PortalClouds;
pub use effect::{Draw, Phase, PortalSound, PortalSpace, TunnelFrame};
pub use tunnel::{Portal, Tunnel, TunnelVertex};
