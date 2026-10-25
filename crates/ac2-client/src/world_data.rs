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

/// A body physics simulates (the player's, or another object's).
#[derive(Clone)]
pub struct BodyData {
    pub shape: BodyShape,
    pub motion: Option<Arc<Ac2MotionInterpDesc>>,
    /// `CSetup::radius`, unscaled (`CPhysicsObj::GetRadius` scales it).
    pub radius: f32,
    /// What its Setup animates with (`PHYSICSOBJREP::SetupModeTracker`), when the visual has a
    /// behaviour table and an `AnimMap`.
    pub animation: Option<ac2_physics::BodyAnimation>,
    /// Its FX sounds (`FXTable` from the total `VisualDesc`), when it has any.
    pub fx_sounds: Option<Arc<crate::FxSoundData>>,
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
    /// Reads the WStates a weenie's script process may start from (package `pid`, entity
    /// `did`) into the shared cache, so `ScriptHost::create_weenie` on the frame thread finds
    /// them there.
    fn preload_script_state(&self, _pid: u32, _did: u32) {}
    /// Makes a read the script VM stopped for (cache-only script data) resident.
    fn load_script_data(&self, _read: ac2_script::vm::DataRequest) {}
}

/// The AC2 DATs, through `ac2-world` and `ac2-physics`' loaders.
pub struct DatWorld {
    store: Arc<ResourceStore>,
    world: Arc<Ac2WorldSource>,
    /// The script host's data (a clone sharing its state), for the reads the VM misses.
    script_data: Option<ac2_script::data::DatScriptData>,
}

impl DatWorld {
    pub fn new(store: Arc<ResourceStore>) -> Result<Self, String> {
        let world = Ac2WorldSource::open(Arc::clone(&store))
            .map_err(|e| e.to_string())?
            .ok_or("no AC2 landscape in the DATs")?;
        Ok(Self {
            store,
            world,
            script_data: None,
        })
    }

    /// The loader threads serve the script VM's missed reads through `data` (a clone of the
    /// host's `DatScriptData`).
    pub fn with_script_data(mut self, data: ac2_script::data::DatScriptData) -> Self {
        self.script_data = Some(data);
        self
    }

    /// The landscape source the client loads from (what a renderer draws from too).
    pub fn world(&self) -> &Arc<Ac2WorldSource> {
        &self.world
    }
}

impl DatWorld {
    /// The visual's FX sounds and which of them loop (`FXManager::SoundIsLooping` reads them
    /// here, on the loader thread).
    fn fx_sounds(&self, visual: DataId) -> Result<Option<Arc<crate::FxSoundData>>, String> {
        let Some(table) = ac2_assets::resolve_ac2_visual_fx_sounds(&self.store, visual)
            .map_err(|e| e.to_string())?
            .filter(ac2_assets::Ac2VisualFxSounds::has_sounds)
        else {
            return Ok(None);
        };
        let sounds = ac2_audio::DatSounds::new(Arc::clone(&self.store));
        let looping = table
            .sound_ids()
            .into_iter()
            .filter(|&did| sounds.is_looping(did))
            .collect();
        Ok(Some(Arc::new(crate::FxSoundData { table, looping })))
    }
}

impl WorldData for DatWorld {
    fn load_script_data(&self, read: ac2_script::vm::DataRequest) {
        if let Some(data) = &self.script_data {
            data.load(read);
        }
    }

    /// `ScriptHost::create_weenie`'s candidates: the class default `0x70000000 | pid` and, for
    /// an entity DID, `0x71000000 | did`.
    fn preload_script_state(&self, pid: u32, did: u32) {
        let mut dids = vec![0x7000_0000 | (pid & 0xFFFF)];
        if did != 0 {
            dids.push(0x7100_0000 | (did & 0x00FF_FFFF));
        }
        for did in dids {
            let _ = ac2_script::load_wstate(&self.store, ac2_core::DataId::new(did));
        }
    }

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
        let animation =
            ac2_physics::load_body_animation(&self.store, visual).map_err(|e| e.to_string())?;
        Ok(BodyData {
            shape,
            motion,
            radius,
            animation,
            fx_sounds: self.fx_sounds(visual)?,
        })
    }

    fn game_clock(&self) -> Option<GameClock> {
        Some(self.world.region()?.game_clock())
    }
}
