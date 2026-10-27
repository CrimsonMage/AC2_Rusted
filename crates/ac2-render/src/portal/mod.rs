//! Portal space (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`): the
//! teleport's fade and tunnel effect, drawn in place of the world while the player is in transit.

pub mod effect;
#[cfg(test)]
mod effect_tests;
pub mod tunnel;
#[cfg(test)]
mod tunnel_tests;

pub use effect::{Draw, Phase, PortalSound, PortalSpace};
pub use tunnel::{Portal, Tunnel, TunnelVertex};
