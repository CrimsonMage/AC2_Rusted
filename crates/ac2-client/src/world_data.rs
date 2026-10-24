//! Where the client's world comes from: the landblocks `CellManager` makes resident, the player
//! body's shape and motion description, and the region's game clock. Everything here reads and
//! decodes DAT files, so the client calls it only from its loader threads (`loader.rs`); the
//! frame thread never does.

use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_physics::{
    load_block_interiors, load_block_objects, load_body_shape, load_terrain_block, BodyShape,
    Terrain,
};
use ac2_world::{load_ac2_motion_interp_desc, Ac2MotionInterpDesc, Ac2WorldSource, GameClock};

/// `m_midesc_did` of the human `VisualDesc`s, used when a visual names none.
pub const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;

/// The player body physics simulates.
pub struct BodyData {
    pub shape: BodyShape,
    pub motion: Option<Arc<Ac2MotionInterpDesc>>,
    /// `CSetup::radius`, unscaled (`CPhysicsObj::GetRadius` scales it).
    pub radius: f32,
}

/// The world data the client reads, shared with its loader threads.
pub trait WorldData: Send + Sync {
    /// Loads landblock `block` (`xxyy0000`) into `terrain`: its terrain, static objects and
    /// interior cells (`CLandBlock` with `CreateStaticObjects` and `InitCellConnections`).
    /// Returns whether the block exists.
    fn load_block(&self, terrain: &mut Terrain, block: u32) -> Result<bool, String>;
    /// A far landblock's `CLandBlockData` without its sub-data (`QualifiedDataIDArray` of
    /// `RequestAsyncLoadBlockRadius` past the near radius): read so the cache holds it. The
    /// physics world never uses far blocks.
    fn prefetch_far_block(&self, _block: u32) -> Result<(), String> {
        Ok(())
    }
    /// The player's collision shape and motion description for its visual DID.
    fn body(&self, visual: Option<u32>) -> Result<BodyData, String>;
    /// The region's `GameTime`, if the world has one.
    fn game_clock(&self) -> Option<GameClock>;
}

/// The AC2 DATs, through `ac2-world` and `ac2-physics`' loaders.
pub struct DatWorld {
    store: Arc<ResourceStore>,
    world: Arc<Ac2WorldSource>,
}

impl DatWorld {
    pub fn new(store: Arc<ResourceStore>) -> Result<Self, String> {
        let world = Ac2WorldSource::open(Arc::clone(&store))
            .map_err(|e| e.to_string())?
            .ok_or("no AC2 landscape in the DATs")?;
        Ok(Self { store, world })
    }
}

impl WorldData for DatWorld {
    fn load_block(&self, terrain: &mut Terrain, block: u32) -> Result<bool, String> {
        let id = CellId::new(block | 0xFFFF);
        if !load_terrain_block(&self.world, terrain, id).map_err(|e| e.to_string())? {
            return Ok(false);
        }
        load_block_objects(&self.world, terrain, id).map_err(|e| e.to_string())?;
        load_block_interiors(&self.world, terrain, id).map_err(|e| e.to_string())?;
        Ok(true)
    }

    fn prefetch_far_block(&self, block: u32) -> Result<(), String> {
        self.world
            .load_landblock(CellId::new(block | 0xFFFF))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn body(&self, visual: Option<u32>) -> Result<BodyData, String> {
        let visual = DataId::new(visual.ok_or("the player has no visual")?);
        let shape = load_body_shape(&self.store, visual)
            .map_err(|e| e.to_string())?
            .ok_or("the player's visual has no Setup")?;
        let radius = match ac2_assets::resolve_ac2_visual_parts(&self.store, visual)
            .map_err(|e| e.to_string())?
            .first()
        {
            Some((setup, _)) => self
                .store
                .load_cached_packed::<ac2_assets::Ac2SetupPhysics>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    *setup,
                    0,
                ))
                .map_err(|e| e.to_string())?
                .map_or(0.0, |physics| physics.radius),
            None => 0.0,
        };
        let midesc = ac2_assets::resolve_ac2_visual_motion_interp(&self.store, visual)
            .map_err(|e| e.to_string())?
            .unwrap_or(DataId::new(HUMAN_MOTION_INTERP_DESC));
        let motion = load_ac2_motion_interp_desc(&self.store, midesc)
            .map_err(|e| e.to_string())?
            .map(|desc| Arc::new((*desc).clone()));
        Ok(BodyData {
            shape,
            motion,
            radius,
        })
    }

    fn game_clock(&self) -> Option<GameClock> {
        Some(self.world.region()?.game_clock())
    }
}
