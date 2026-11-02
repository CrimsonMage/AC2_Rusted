#![forbid(unsafe_code)]

//! Shared low-level vocabulary: ids, scalars, colours and the enums and flags used across
//! the client.
//!
//! This crate deliberately stays small. It owns shared ids, passive scalar wrappers,
//! common sentinels, deterministic low-level helpers, and portability-safe PODs. It does
//! not own legacy container/string/runtime scaffolding or client orchestration policy.

mod color;
mod constants;
mod data_release;
mod enums;
mod fast_hash;
mod ids;
mod native_type;
pub mod pstring;
mod random;
mod scalar;
mod timer;
mod turbine_guid;

pub use color::{ColorArgb, ColorArgbByteReader};
pub use data_release::{data_release, set_data_release, DataRelease};
pub use enums::*;
pub use fast_hash::{fast_hash_set_with_capacity, FastHashMap, FastHashSet, FastHasher};
pub use ids::{CellId, DataId, InstanceId, ObjectId, QualifiedDataId, VersionHandle};
pub use native_type::{native_type_from_release, native_type_to_release};
pub use random::Random;
pub use scalar::VersionToken;
pub use timer::Timer;
pub use turbine_guid::TurbineGuid;
