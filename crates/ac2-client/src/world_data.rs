//! Where the client's world comes from: the landblocks `CellManager` makes resident, the player
//! body's shape and motion description, and the region's game clock. Everything here reads and
//! decodes DAT files, so the client calls it only from its loader threads (`loader.rs`); the
//! frame thread never does.

use std::sync::Arc;

use ac2_anim::{Ac2Location, Ac2Setup};
use ac2_core::{CellId, DataId};
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_physics::Terrain;
use ac2_world::{Ac2MotionInterpDesc, Ac2WorldSource, GameClock};

pub use ac2_smartbox::HUMAN_MOTION_INTERP_DESC;

/// A body physics simulates (the player's, or another object's): the residency body
/// (`ac2_smartbox::BodyData`) with the client's FX and holding data.
#[derive(Clone)]
pub struct BodyData {
    /// Shape, motion description, radius, animation and collision object.
    pub body: ac2_smartbox::BodyData,
    /// Its FX sounds (`FXTable` from the total `VisualDesc`), when it has any.
    pub fx_sounds: Option<Arc<crate::FxSoundData>>,
    /// Its FX particle systems per FX id (the same table), when it has any.
    pub fx_particles: Option<Arc<ac2_assets::Ac2VisualFxParticles>>,
    /// The holding locations of every part group's Setup, in `CPartGroup::GetHoldingLocation`'s
    /// search order (the first entry for an id is the one retail finds); the objects held by this
    /// one hang from them. Empty without a Setup.
    pub holding: Arc<[GroupHolding]>,
}

/// A Setup holding location (`CSetup`'s `LocationType` list) and the part group whose Setup lists
/// it: the group's index in the visual's drawn groups.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupHolding {
    pub group: usize,
    pub location: Ac2Location,
}

/// The world data the client reads, shared with its loader threads: the residency reads
/// (`ac2_smartbox::ResidentWorld`), the script and audio reads ([`WorldServices`]) and the
/// client's own body (its FX and holding data).
pub trait WorldData: ac2_smartbox::ResidentWorld + WorldServices {
    /// The player's collision shape and motion description for its visual DID, with its FX
    /// sounds, particles and holding locations.
    fn body(&self, visual: Option<u32>) -> Result<BodyData, String>;
}

/// The client's script and ambient-sound reads, which need `ac2-script` and `ac2-audio`. A world
/// that answers none of them keeps the defaults.
pub trait WorldServices: Send + Sync {
    /// Reads the WStates a weenie's script process may start from (package `pid`, entity
    /// `did`) into the shared cache, so `ScriptHost::create_weenie` on the frame thread finds
    /// them there.
    fn preload_script_state(&self, _pid: u32, _did: u32) {}
    /// Makes a read the script VM stopped for (cache-only script data) resident, or notes that
    /// the DATs lack it (`ScriptData::load`). `true` when the read was answered that way; a
    /// world that drops reads returns `false`, and the contexts waiting on them stay waiting.
    fn load_script_data(&self, _read: ac2_script::vm::DataRequest) -> bool {
        false
    }
    /// The region's ambient sound data, if it has any.
    fn ambient_data(&self) -> Option<Arc<ac2_audio::AmbientData>> {
        None
    }
    /// Landblock `block`'s ambient sound points.
    fn ambient_points(&self, _block: u32) -> Vec<ac2_world::Ac2AmbientSoundPoint> {
        Vec::new()
    }
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
        // `MasterProperty` names for the collision sets (`MasterProperty::GetPropertyDesc`),
        // read here on the preload path, never on the frame thread.
        ac2_db::install_ac2_property_types(&store).map_err(|e| e.to_string())?;
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

impl ac2_smartbox::BlockSource for DatWorld {
    fn load_block(&self, terrain: &mut Terrain, block: u32) -> Result<bool, String> {
        ac2_smartbox::load_dat_block(&self.world, terrain, block)
    }

    fn prefetch_far_block(&self, block: u32) -> Result<(), String> {
        ac2_smartbox::prefetch_dat_far_block(&self.world, block)
    }
}

impl ac2_smartbox::ResidentWorld for DatWorld {
    fn body_shape(&self, visual: Option<u32>) -> Result<ac2_smartbox::BodyData, String> {
        let visual = DataId::new(visual.ok_or("the player has no visual")?);
        // `ICPhysicsObj::makeObject`: an `EntityDesc` (an FX node's object) draws its template's visual.
        let visual = ac2_assets::resolve_ac2_entity_visual(&self.store, visual)
            .map_err(|e| e.to_string())?;
        ac2_smartbox::load_body_data(&self.store, visual)
    }

    fn game_clock(&self) -> Option<GameClock> {
        Some(self.world.region()?.game_clock())
    }

    fn region(&self) -> Option<Arc<ac2_world::RegionDesc>> {
        self.world.region()
    }

    fn motion_desc(&self, did: u32) -> Result<Option<Arc<Ac2MotionInterpDesc>>, String> {
        let desc = ac2_world::load_ac2_motion_interp_desc(&self.store, DataId::new(did))
            .map_err(|e| e.to_string())?;
        Ok(desc.map(|desc| Arc::new((*desc).clone())))
    }

    fn entity_ethereal_mask(&self, did: u32) -> Result<u64, String> {
        ac2_world::load_entity_ethereal_mask(&self.store, DataId::new(did))
            .map_err(|e| e.to_string())
    }
}

impl WorldServices for DatWorld {
    fn ambient_data(&self) -> Option<Arc<ac2_audio::AmbientData>> {
        let did = ac2_world::ac2_region_sound_desc(&self.store).ok()??;
        ac2_audio::AmbientData::load(&self.store, did.raw()).ok()?
    }

    fn ambient_points(&self, block: u32) -> Vec<ac2_world::Ac2AmbientSoundPoint> {
        ac2_world::ac2_ambient_sound_points(&self.store, CellId::new(block | 0xFFFF))
            .unwrap_or_default()
    }

    fn load_script_data(&self, read: ac2_script::vm::DataRequest) -> bool {
        let Some(data) = &self.script_data else {
            return false;
        };
        data.load(read);
        true
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
}

impl WorldData for DatWorld {
    fn body(&self, visual: Option<u32>) -> Result<BodyData, String> {
        let visual = DataId::new(visual.ok_or("the player has no visual")?);
        load_body_data(&self.store, visual)
    }
}

/// A body's collision shape, motion description, radius and animation for `visual` (its
/// `VisualDesc` or `Setup`), and its FX sounds, read on a loader thread.
pub(crate) fn load_body_data(
    store: &Arc<ResourceStore>,
    visual: DataId,
) -> Result<BodyData, String> {
    // `ICPhysicsObj::makeObject`: an `EntityDesc` (an FX node's object) draws its template's visual.
    let visual = ac2_assets::resolve_ac2_entity_visual(store, visual).map_err(|e| e.to_string())?;
    let body = ac2_smartbox::load_body_data(store, visual)?;
    Ok(BodyData {
        body,
        fx_sounds: load_fx_sounds(store, visual)?,
        fx_particles: ac2_assets::resolve_ac2_visual_fx_particles(store, visual)
            .map_err(|e| e.to_string())?
            .filter(|table| !table.is_empty())
            .map(Arc::new),
        holding: load_holding_locations(store, visual)?,
    })
}

/// The holding locations of every part group's Setup for `visual`, read on the loader thread in
/// `CPartGroup::GetHoldingLocation`'s search order. A visual with no part groups has none.
fn load_holding_locations(
    store: &Arc<ResourceStore>,
    visual: DataId,
) -> Result<Arc<[GroupHolding]>, String> {
    let groups = ac2_render::object::resolve_drawn_groups(store, visual, None)
        .ok()
        .flatten();
    let Some(groups) = groups else {
        return Ok(Arc::from([]));
    };
    let mut holding = Vec::new();
    for group in ac2_assets::holding_search_order(&groups) {
        let setup = store
            .load_cached_packed::<Ac2Setup>(TypedResourceRef::new(
                DatFileKind::Portal,
                groups[group].setup,
                0,
            ))
            .map_err(|e| e.to_string())?;
        for location in setup.iter().flat_map(|setup| setup.holding_points.iter()) {
            holding.push(GroupHolding {
                group,
                location: *location,
            });
        }
    }
    Ok(holding.into())
}

/// The visual's FX sounds and which of them loop (`FXManager::SoundIsLooping` reads them
/// here, on the loader thread).
fn load_fx_sounds(
    store: &Arc<ResourceStore>,
    visual: DataId,
) -> Result<Option<Arc<crate::FxSoundData>>, String> {
    let Some(table) = ac2_assets::resolve_ac2_visual_fx_sounds(store, visual)
        .map_err(|e| e.to_string())?
        .filter(ac2_assets::Ac2VisualFxSounds::has_sounds)
    else {
        return Ok(None);
    };
    let sounds = ac2_audio::DatSounds::new(Arc::clone(store));
    let looping = table
        .sound_ids()
        .into_iter()
        .filter(|&did| sounds.is_looping(did))
        .collect();
    Ok(Some(Arc::new(crate::FxSoundData { table, looping })))
}
