//! Asheron's Call 1 creatures drawn in the AC2 world (the `ac1-monsters` test branch).
//!
//! The AC1 DATs mount beside the AC2 ones through `ac2-dat`'s retail `DiskController` reader, the
//! same B-tree and positioned-read path Vitaeum mounts them with (the two games share the Turbine
//! DAT container). [`Ac1Dats`] reads the records a creature draws ([`formats`]), applies an ACE
//! weenie's appearance ([`weenie`]: Setup, ClothingBase, PaletteTemplate and shade, as the server
//! sends it in an `ObjDesc`), and builds an `ac2-render` `ObjectModel` with its images
//! ([`creature`]). [`Ac1Models`] hands those models to the scene loader under visual DIDs of their
//! own, and [`IdleCycle`] poses them with the motion table's idle cycle.

pub mod creature;
mod dats;
pub mod formats;
mod image;
pub mod weenie;

pub use creature::{Ac1Creature, Ac1Models, IdleCycle};
pub use dats::Ac1Dats;
pub use formats::{Ac1Error, Ac1Result};
pub use weenie::AceWeenie;

/// The environment variable naming the AC1 DAT directory (`client_portal.dat` and its siblings).
pub const AC1_DAT_DIR_ENV: &str = "AC1_DAT_DIR";
