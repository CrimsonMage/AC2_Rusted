//! What the residency side reads from the world's data (`CellManager`'s landblocks and the
//! body and object facts the client loads on its loader threads). The data source implements
//! it; `ac2-client`'s `DatWorld` is the DAT-backed one. Read on loader threads only.

use std::sync::Arc;

use ac2_world::{Ac2MotionInterpDesc, GameClock, RegionDesc};

use crate::{BlockSource, BodyData};

/// A world's residency data: landblocks ([`BlockSource`]), a visual's body shape, and the region
/// and motion facts the object loader reads.
pub trait ResidentWorld: BlockSource {
    /// A body's collision shape, motion description, radius and animation for `visual` (its
    /// visual DID, `None` when the body has none), read on a loader thread.
    fn body_shape(&self, visual: Option<u32>) -> Result<BodyData, String>;
    /// The region's `GameTime`, if the world has one.
    fn game_clock(&self) -> Option<GameClock> {
        None
    }
    /// The region (its weather and sky).
    fn region(&self) -> Option<Arc<RegionDesc>> {
        None
    }
    /// The `MotionInterpDesc` `did` (`CMotionInterp::SetMIDESC`'s `DBObj::Get`). `None` when the
    /// DATs have none.
    fn motion_desc(&self, _did: u32) -> Result<Option<Arc<Ac2MotionInterpDesc>>, String> {
        Ok(None)
    }
    /// The ethereal blocking bits of the `EntityDesc` `did`'s own properties
    /// (`CPhysicsObj::InitVisualDesc` applies them to the body it makes).
    fn entity_ethereal_mask(&self, _did: u32) -> Result<u64, String> {
        Ok(0)
    }
}
