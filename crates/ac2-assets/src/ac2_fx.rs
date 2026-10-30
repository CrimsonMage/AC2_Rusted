//! Asheron's Call 2 FX records: `FXNode`/`FXData` (embedded in `PartGroupDataDesc` FX overrides)
//! and `DBFXTable` (`0x1E`), decoded in `FXNode::StreamPack` / `DBFXTable::StreamPack` order.
//! The particle-emitter facts of each node are retained (`FXManager::DoParticleFX`), and its
//! sound facts (`FXManager::DoTargettedFX`'s sound branch); the object, camera, appearance,
//! streak, decal and light parts are kept with them, the repulsor parsed to stay aligned.

use std::sync::Arc;

use ac2_anim::Ac2Transform;
use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::Vec3;

use crate::ac2_light_source::{read_ac2_light_source, Ac2LightSource};
use crate::ac2_material::{read_waveform, Ac2Waveform};

pub(crate) type Malformed = &'static str;
const MAX_ITEMS: usize = 4096;

/// The light part of one `FXNode` (`LightSourceFX`): the light, its lifespan and its fade time.
/// `FXManager::DoLightSourceFX` places it at `offset`, the node's bone offset in the object's
/// space (`CPartArray::CreateLightSource`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2FxLight {
    pub offset: Vec3,
    pub source: Ac2LightSource,
    /// `m_fLifespan`: seconds the light lasts; zero keeps it while its object is in the world.
    pub lifespan: f32,
    /// `m_fFadeTime`: seconds of fade in and fade out.
    pub fade_time: f32,
}

/// Where an FX-created object is placed on its emitter (`FXManager::DoCreateObjectFX`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Ac2FxObjectAttach {
    /// Neither a holding point nor a bone: the object stands where its emitter does
    /// (`CPhysicsObj::set_position` at the rep's position). A water-plane node is the same: the
    /// branch writes a water plane it never reads.
    #[default]
    Emitter,
    /// A holding point or bone (`CPhysicsObj::FXSetParent`). The object is re-placed on every
    /// frame from the emitter's posed bone (`CPhysicsObj::UpdateChild`).
    Attached(Ac2FxAttach),
    /// The node names a bone or holding point the emitter's part groups lack, so there is
    /// nothing to attach to and the object is not placed.
    Unplaced,
}

/// The point an attached FX object hangs from, on its emitter's posed skeleton.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ac2FxAttachPoint {
    /// A holding point's bone: the index into the emitter's root Setup bones
    /// (`PHYSICSOBJREP::GetHoldingLocation`).
    Holding(usize),
    /// A bone node's name, found in the part groups' search order (`GetBoneTypeMatrix`).
    Bone(u32),
}

/// An attached FX object's place on its emitter: the point, the local frame from that point
/// (a holding point's transform then the node's offset, or the offset alone for a bone), whether
/// the rotation is dropped (`m_remove_rotation`), and the same frame at the emitter's rest pose
/// (the object's place when it is created).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2FxAttach {
    pub point: Ac2FxAttachPoint,
    pub local: Ac2Transform,
    pub remove_rotation: bool,
    pub rest: Ac2Transform,
}

/// The object facts of one `FXNode`: the object it creates (`FXManager::DoCreateObjectFX`), its
/// colour (`FXManager::DoColorFX`), what it does to the object it targets
/// (`FXManager::DoTargetAndCloningFXs`'s hide and destroy), and whether `StopFX` restores the
/// colour (`FXManager::StopFX`'s `m_restore`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ac2FxObject {
    /// `m_vdesc_id`: the visual of the object the node creates (`CreateTemporaryObject`).
    pub vdesc: u32,
    /// `m_behavior`: the behaviour the created object starts (its failure keeps the object).
    pub behavior: u32,
    /// `m_lifetime` (`FXNode::SetDefaults`: 15 seconds): how long an object-making node's FX
    /// lasts; negative: until stopped.
    pub lifetime: f32,
    /// `m_hide_target`: the FX's target object is hidden until the FX stops.
    pub hide_target: bool,
    /// `m_destroy_target`: the FX's target object fades and is destroyed.
    pub destroy_target: bool,
    /// `m_target_fx_id`: an FX the target object plays.
    pub target_fx_id: u32,
    /// `m_has_color` and `m_color` (r, g, b, a): the emitter's colour, ramped over `ramp_time`.
    pub color: Option<[f32; 4]>,
    /// `m_ramp_time`: seconds a colour change takes.
    pub ramp_time: f32,
    /// `m_color_lifetime`: seconds a colour change holds before the FX restores it.
    pub color_lifetime: f32,
    /// `m_restore`: `StopFX` ramps the colour back to white (over 0.1 s).
    pub restore: bool,
    /// `m_has_streak`'s `StreakFX`: the streak an FX node starts on its emitter
    /// (`FXManager::DoStreakFX`). Each stream word is read in `FXNode::StreamPack` order.
    pub streak: Option<Ac2StreakFx>,
    /// `m_start_streak`: `DoTargettedFX` starts the emitter's streak effects
    /// (`PHYSICSOBJREP::StartStreak`).
    pub start_streak: bool,
    /// `m_stop_streak`: `DoTargettedFX` stops the emitter's streak effects
    /// (`PHYSICSOBJREP::StopStreak`).
    pub stop_streak: bool,
    /// Where the created object is placed on its emitter (see [`Ac2FxObjectAttach`]; its rest
    /// place is in [`Ac2FxAttach::rest`]).
    pub attach: Ac2FxObjectAttach,
    /// `m_holding_location`: the holding point the streak flags act on (`PHYSICSOBJREP::StartStreak`
    /// and `StopStreak` take it; zero reaches every child).
    pub holding_location: u32,
    /// `m_has_decal`'s `DecalFX`: the decal `FXManager::DoDecalFX` projects when the node plays.
    pub decal: Option<Ac2DecalFx>,
    /// The node's gate: who it plays for and when (`FXManager::DoTargettedFX`'s checks).
    pub gate: Ac2FxNodeGate,
    /// `m_apr_id`: the appearance table the node applies to its emitter
    /// (`FXManager::DoAppearanceFX`); zero for none.
    pub appearance: u32,
    /// `m_apr_key` (`AppliedAppearanceKey`, read by `FXNode::GetAppearanceKey`): the key and its scalar modifier.
    pub appearance_key: (u32, f32),
    /// `m_cameraFX_id`: the `CameraFX` the node plays on the camera (`FXManager::DoCameraFX`).
    pub camera_fx: u32,
    /// `m_slider`, `m_intensity` and `m_prop_color`: the slider `FXManager::DoSliderFX` moves
    /// on the emitter (zero for none) to the intensity over `ramp_time`, and whether it goes on
    /// to the emitter's children.
    pub slider: Ac2FxSlider,
    /// `m_has_repulsor`'s `RepulsorFX`: the repulsor `FXManager::DoRepulseFX` makes.
    pub repulsor: Option<Ac2RepulsorFx>,
    /// `m_retarget`: the node retargets (with an object, its `lifetime` still sets the duration).
    pub retarget: bool,
    /// `m_fx_script_id`: the `FxScript` the node plays, whose period is the FX's duration.
    pub fx_script: u32,
    /// `FXManager::ParticleIsLooping` of the node's first system (`FXManager::CalcDuration`).
    pub particle_looping: bool,
    /// `m_parent` (the target hangs from the emitter's `holding_location`), `m_unparent` (the
    /// target leaves its parent) and `m_conditional` (a retargeting node acts on the target
    /// itself, no clone): `FXManager::DoTargetAndCloningFXs`.
    pub parent: bool,
    pub unparent: bool,
    pub conditional: bool,
    /// The node's sound (`FXNode::GetSoundID`; the first of its list, as the pick among several
    /// is random), whose `FXManager::SoundIsLooping` the FX's duration reads
    /// (`FXManager::CalcDuration`). Zero for none.
    pub sound: u32,
    /// `FXManager::SoundIsLooping(sound)`, set by the client from the sound records it read.
    pub sound_looping: bool,
}

/// An FX node's slider effect (`FXManager::DoSliderFX`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ac2FxSlider {
    /// `m_slider`: the slider id, zero for none.
    pub id: u32,
    /// `m_intensity` (default 1.0 when the stream lacks it): the slider's target at full scalar.
    pub intensity: f32,
    /// `m_prop_color`: the slider also moves on the emitter's direct children.
    pub propagate: bool,
}

/// `RepulsorFX` (`RepulsorFX::StreamPack`): a particle repulsor or attractor.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ac2RepulsorFx {
    /// `m_fAcceleration`, `m_fRange`, `m_fLifetime` and `m_fMinimumVelocity`.
    pub acceleration: f32,
    pub range: f32,
    pub lifetime: f32,
    pub minimum_velocity: f32,
    /// `m_bAttractor`: pulls particles in instead of pushing them away.
    pub attractor: bool,
}

/// `DoTargettedFX`'s gate on each node, read before any of the node's parts run: the channel
/// preferences (`FXManager::ChannelMissmatch`), `m_player_only`, and `m_day_time` /
/// `m_night_time` for the time of day. A flag the node does not carry keeps its default: the
/// day and night flags default to on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2FxNodeGate {
    /// `m_channel_keys` is not empty.
    pub has_channels: bool,
    /// The node's values for the four channel keys `ChannelPrefs` holds, in [`CHANNEL_KEYS`] order
    /// (`None`: the node has no such key). A key the preferences do not hold never compares.
    pub channels: [Option<u32>; 4],
    /// `m_player_only`.
    pub player_only: bool,
    /// `m_day_time`: the node plays by day.
    pub day_time: bool,
    /// `m_night_time`: the node plays by night.
    pub night_time: bool,
}

impl Default for Ac2FxNodeGate {
    fn default() -> Self {
        Self {
            has_channels: false,
            channels: [None; 4],
            player_only: false,
            day_time: true,
            night_time: true,
        }
    }
}

/// The channel keys `ChannelPrefs` maps to its preferences, in the order [`Ac2FxChannelPrefs`]
/// and [`Ac2FxNodeGate::channels`] hold them: `gore` (the gore amount, preference `0x40000035`),
/// `BloodColor` (the gore hue, `0x40000036`), `FootStepSounds` (`0x40000037`) and
/// `FootStepParticles` (`0x40000039`). The keys are the channel enum's constants (the FX nodes'
/// `m_channel_keys` use them), not the preference ids: 1, 7, 5 and 8. The enum's other keys (the
/// data also has 9, `PlayerMusic`) are not in the table and never compare.
pub const CHANNEL_KEYS: [u32; 4] = [0x4000_0001, 0x4000_0007, 0x4000_0005, 0x4000_0008];

/// The preference each channel key follows, in [`CHANNEL_KEYS`] order
/// (`ChannelPrefs::HandleApply`'s cases).
pub const CHANNEL_PREFERENCES: [u32; 4] = [0x4000_0035, 0x4000_0036, 0x4000_0037, 0x4000_0039];

/// `ChannelPrefs`' table (`ChannelPrefs::HandleApply` / `HandleChange` set a key to the
/// preference's value): the value each channel key holds now, `None` while it has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ac2FxChannelPrefs {
    pub values: [Option<u32>; 4],
}

impl Ac2FxChannelPrefs {
    /// `HashTable::set(key, value)` for one of the four keys (any other key is not kept).
    pub fn set(&mut self, key: u32, value: u32) {
        if let Some(at) = CHANNEL_KEYS.iter().position(|k| *k == key) {
            self.values[at] = Some(value);
        }
    }
}

/// The preference table `FXManager::ChannelMissmatch` reads (`ChannelPrefs`' one instance, shared
/// by every `FXManager`).
static CHANNEL_PREFS: std::sync::RwLock<Ac2FxChannelPrefs> =
    std::sync::RwLock::new(Ac2FxChannelPrefs { values: [None; 4] });

/// `ChannelPrefs::m_channel_keys` now.
pub fn fx_channel_prefs() -> Ac2FxChannelPrefs {
    CHANNEL_PREFS
        .read()
        .map_or_else(|e| *e.into_inner(), |g| *g)
}

/// Replaces the table (what `ChannelPrefs::HandleApply` leaves after it set each key).
pub fn set_fx_channel_prefs(prefs: Ac2FxChannelPrefs) {
    match CHANNEL_PREFS.write() {
        Ok(mut guard) => *guard = prefs,
        Err(poisoned) => *poisoned.into_inner() = prefs,
    }
}

impl Ac2FxNodeGate {
    /// `FXManager::ChannelMissmatch`: a node with channel keys mismatches when, for a key the
    /// preference table holds, the node holds the key with another value.
    pub fn channel_missmatch(&self, prefs: &Ac2FxChannelPrefs) -> bool {
        self.has_channels
            && self
                .channels
                .iter()
                .zip(prefs.values)
                .any(|(node, pref)| matches!((node, pref), (Some(n), Some(p)) if *n != p))
    }

    /// `ChannelMissmatch` against the shared preference table.
    pub fn missmatches(&self) -> bool {
        self.has_channels && self.channel_missmatch(&fx_channel_prefs())
    }

    /// Whether a running FX's node takes a new scalar or steps (`DoTargettedFX`'s update branch
    /// and `UseTime`): no channel mismatch, and `m_player_only` plays only for the player.
    pub fn updates(&self, is_player: bool) -> bool {
        !self.missmatches() && (!self.player_only || is_player)
    }

    /// `FXManager::DoTargettedFX`'s gates for a node of a new FX: no channel mismatch,
    /// `m_player_only` plays only for the player, and the node's day or night flag must be on
    /// for the time of day.
    pub fn plays(&self, is_player: bool, night: bool) -> bool {
        self.updates(is_player)
            && if night {
                self.night_time
            } else {
                self.day_time
            }
    }
}

impl Ac2FxObject {
    /// Whether the node carries any object fact `DoTargettedFX` acts on.
    pub fn is_active(&self) -> bool {
        self.vdesc != 0
            || self.hide_target
            || self.destroy_target
            || self.parent
            || self.unparent
            || self.retarget
            || self.target_fx_id != 0
            || self.color.is_some()
            || self.restore
            || self.decal.is_some()
            || self.appearance != 0
            || self.camera_fx != 0
            || self.slider.id != 0
            || self.repulsor.is_some()
    }
}

/// `StreakFX`: the streak an FX node draws on its emitter (`StreakFX::StreamPack`). Its
/// colour, the two points it trails from (in the emitter's space), the material instance, and
/// the lifespan, trail duration, minimum segment time and distance and texture scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2StreakFx {
    /// `m_cColor` (r, g, b, a), unpacked from the stream's colour word.
    pub color: [f32; 4],
    /// `m_vStartPoint` and `m_vEndPoint`: the streak's two ends, in the emitter's space.
    pub start: Vec3,
    pub end: Vec3,
    /// `m_MaterialInstanceID`: the material the ribbon is drawn with.
    pub material: u32,
    /// `m_fLifespan`: seconds the streak lasts (zero keeps it until stopped).
    pub lifespan: f32,
    /// `m_fTrailDuration`: seconds a segment takes to fade out.
    pub trail_duration: f32,
    /// `m_fMinSegmentTime` and `m_fMinSegmentDistance`: a new segment is cut only when the
    /// last one is at least this old (zero: at any age) and this far from the previous one.
    pub min_segment_time: f32,
    pub min_segment_distance: f32,
    /// `m_fTextureScale`: the ribbon's texture repeats per metre of its length.
    pub texture_scale: f32,
}

/// `DecalFX`: the decal an FX node projects onto the ground or the objects under its emitter
/// (`FXManager::DoDecalFX`). Its colour (`RGBAColor`), material, origin offset from the emitter,
/// lifespan, and the three waveforms that give the decal's size, its rotation about the vertical
/// axis in degrees, and the radius its position is scattered over.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2DecalFx {
    /// `m_cColor` (r, g, b, a), unpacked from the stream's colour word.
    pub color: [f32; 4],
    /// `m_MaterialDID`: the material the decal is drawn with.
    pub material: u32,
    /// `m_vOrigin`: the offset added to the emitter's position, in its space.
    pub origin: Vec3,
    /// `m_fLifespan`: seconds the decal lasts before it fades out.
    pub lifespan: f32,
    /// `m_wSize`: the decal's edge length in metres.
    pub size: Ac2Waveform,
    /// `m_wRotation`: the decal's rotation about the vertical axis, in degrees.
    pub rotation: Ac2Waveform,
    /// `m_wPositionRadius`: the radius the decal's position is scattered within.
    pub position_radius: Ac2Waveform,
}

/// `DecalFX::StreamPack` (the 40 words after `m_has_decal`): `RGBAColor::StreamPack` writes the
/// colour as one packed word (as for [`read_streak_fx`]), then the material, the origin, the
/// lifespan and the three waveforms (`STREAMPACK_SIZEOF`, 44 bytes each).
fn read_decal_fx(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2DecalFx, Malformed> {
    let packed = word(cursor)?;
    let channel = |shift: u32| ((packed >> shift) & 0xFF) as f32 / 255.0;
    let color = [channel(16), channel(8), channel(0), channel(24)];
    let material = word(cursor)?;
    let origin = Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?);
    let lifespan = float(cursor)?;
    let mut waveform = || read_waveform(cursor).map_err(|_| "record ended early");
    let size = waveform()?;
    let rotation = waveform()?;
    let position_radius = waveform()?;
    Ok(Ac2DecalFx {
        color,
        material,
        origin,
        lifespan,
        size,
        rotation,
        position_radius,
    })
}

/// `StreakFX::StreamPack` (the 13 words after `m_has_streak`): `RGBAColor::StreamPack` writes the
/// colour as one packed word (`RGBAColor::SetColor32`: a in the top byte, then r, g, b), then the
/// two points, the material instance and the five scalars.
fn read_streak_fx(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2StreakFx, Malformed> {
    let packed = word(cursor)?;
    let channel = |shift: u32| ((packed >> shift) & 0xFF) as f32 / 255.0;
    let color = [channel(16), channel(8), channel(0), channel(24)];
    let start = Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?);
    let end = Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?);
    Ok(Ac2StreakFx {
        color,
        start,
        end,
        material: word(cursor)?,
        lifespan: float(cursor)?,
        trail_duration: float(cursor)?,
        min_segment_time: float(cursor)?,
        min_segment_distance: float(cursor)?,
        texture_scale: float(cursor)?,
    })
}

/// The particle and light part of one `FXNode`: the `PSDesc` list (`GetParticleID` picks one at
/// random) and where the system attaches (`PHYSICSOBJREP::CreateParticleSystem`), and the light
/// it places (`FXManager::DoLightSourceFX`). A node with a light and no `PSDesc` keeps only the
/// light.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxParticleNode {
    pub particle_descs: Arc<[DataId]>,
    /// `m_fire_and_forget`: the systems are placed once where their object is, not kept on it
    /// (`FXManager::FireParticleFX`), and are not tracked for `StopFX`.
    pub fire_and_forget: bool,
    /// `m_has_light`'s light.
    pub light: Option<Ac2FxLight>,
    /// `m_holding_location`: a Setup holding point id (takes precedence over the bone).
    pub holding_location: u32,
    /// `m_bone_name`: a Setup bone name (`CPartGroup::FindBoneType`).
    pub bone_name: u32,
    /// `m_bone_offset`, translated in the bone's (or the object's) space.
    pub bone_offset: Vec3,
    /// `m_remove_rotation`: keep only the attachment translation.
    pub remove_rotation: bool,
    /// `m_water_plane`: the system rides the cell's water surface.
    pub water_plane: bool,
    /// The node's object facts (see [`Ac2FxObject`]).
    pub object: Ac2FxObject,
}

/// One FX id's data list (`PSmartArray<FXData>`), reduced to its particle nodes.
pub type Ac2FxEntry = (u32, Arc<[Ac2FxParticleNode]>);

/// What `FXManager::DoTargettedFX` and `UseTime` read of one `FXNode` for sound. Every node is
/// kept (its sound list may be empty), as each node steps the FX's volume ramp.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxSoundNode {
    /// `m_sound_list`: `SoundInfo` / `MusicInfo` DIDs; `FXNode::GetSoundID` picks one.
    pub sounds: Arc<[u32]>,
    /// `m_ramp_time`: seconds for the volume to reach the FX's scalar.
    pub ramp_time: f32,
    /// `m_fire_and_forget`: a looping sound is not kept to be stopped with the FX.
    pub fire_and_forget: bool,
    /// `m_player_only`.
    pub player_only: bool,
    /// `m_day_time` / `m_night_time`: when the node plays.
    pub day_time: bool,
    pub night_time: bool,
    /// `m_channel_keys` is not empty (`FXManager::ChannelMissmatch` consults the channel prefs).
    pub has_channels: bool,
    /// The node's values for [`CHANNEL_KEYS`] (see [`Ac2FxNodeGate::channels`]).
    pub channels: [Option<u32>; 4],
}

impl Ac2FxSoundNode {
    /// The node's gate, as `DoTargettedFX` reads it.
    pub fn gate(&self) -> Ac2FxNodeGate {
        Ac2FxNodeGate {
            has_channels: self.has_channels,
            channels: self.channels,
            player_only: self.player_only,
            day_time: self.day_time,
            night_time: self.night_time,
        }
    }
}

/// The nodes of an FX id's list when its object stands on `terrain` (`FXData::GetData(terrain)`
/// of each `FXData`: the terrain's node where the data has one, else its default node).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxTerrainSet {
    pub terrain: u32,
    pub particles: Arc<[Ac2FxParticleNode]>,
    pub sounds: Arc<[Ac2FxSoundNode]>,
}

/// One FX id's terrain sets.
pub type Ac2FxTerrainEntry = (u32, Arc<[Ac2FxTerrainSet]>);

/// The default node of an `FXData` as the two views keep it.
pub(crate) type FxNodePair = (Option<Ac2FxParticleNode>, Ac2FxSoundNode);

/// One decoded `FXData`: the default node and the terrain-keyed ones (`m_terrainData`).
pub(crate) struct FxDataParts {
    pub(crate) default: FxNodePair,
    pub(crate) terrains: Vec<(u32, FxNodePair)>,
}

/// An FX id's list gathered from `FXData`s, and from segments of other lists
/// (`DBFXTable::GetFXData` appends each table's list): the default nodes, and for every
/// terrain any segment keys, the nodes `FXData::GetData` returns for it.
#[derive(Debug, Clone, Default)]
pub(crate) struct FxSegments {
    segments: Vec<FxSegment>,
}

#[derive(Debug, Clone)]
struct FxSegment {
    particles: Arc<[Ac2FxParticleNode]>,
    sounds: Arc<[Ac2FxSoundNode]>,
    terrains: Arc<[Ac2FxTerrainSet]>,
}

impl FxSegments {
    pub(crate) fn push(
        &mut self,
        particles: Arc<[Ac2FxParticleNode]>,
        sounds: Arc<[Ac2FxSoundNode]>,
        terrains: Arc<[Ac2FxTerrainSet]>,
    ) {
        self.segments.push(FxSegment {
            particles,
            sounds,
            terrains,
        });
    }

    /// The whole list's default particle nodes.
    pub(crate) fn particles(&self) -> Vec<Ac2FxParticleNode> {
        self.segments
            .iter()
            .flat_map(|s| s.particles.iter().cloned())
            .collect()
    }

    /// The whole list's default sound nodes.
    pub(crate) fn sounds(&self) -> Vec<Ac2FxSoundNode> {
        self.segments
            .iter()
            .flat_map(|s| s.sounds.iter().cloned())
            .collect()
    }

    /// The list for each terrain some segment keys, in the order the terrains first appear.
    pub(crate) fn terrain_sets(&self) -> Vec<Ac2FxTerrainSet> {
        let mut ids = Vec::new();
        for terrain in self.segments.iter().flat_map(|s| s.terrains.iter()) {
            if !ids.contains(&terrain.terrain) {
                ids.push(terrain.terrain);
            }
        }
        ids.into_iter()
            .map(|id| {
                let (mut particles, mut sounds) = (Vec::new(), Vec::new());
                for segment in &self.segments {
                    match segment.terrains.iter().find(|t| t.terrain == id) {
                        Some(set) => {
                            particles.extend(set.particles.iter().cloned());
                            sounds.extend(set.sounds.iter().cloned());
                        }
                        None => {
                            particles.extend(segment.particles.iter().cloned());
                            sounds.extend(segment.sounds.iter().cloned());
                        }
                    }
                }
                Ac2FxTerrainSet {
                    terrain: id,
                    particles: particles.into(),
                    sounds: sounds.into(),
                }
            })
            .collect()
    }
}

impl FxDataParts {
    /// The default nodes of a list of `FXData` as one segment: the particle and sound views, and
    /// the terrain sets (each key's nodes for the data that carry it, the default for the rest).
    pub(crate) fn segment(
        list: Vec<FxDataParts>,
    ) -> (
        Arc<[Ac2FxParticleNode]>,
        Arc<[Ac2FxSoundNode]>,
        Arc<[Ac2FxTerrainSet]>,
    ) {
        let mut ids = Vec::new();
        for data in &list {
            for (terrain, _) in &data.terrains {
                if !ids.contains(terrain) {
                    ids.push(*terrain);
                }
            }
        }
        let terrains: Vec<Ac2FxTerrainSet> = ids
            .into_iter()
            .map(|id| {
                let (mut particles, mut sounds) = (Vec::new(), Vec::new());
                for data in &list {
                    let (particle, sound) = data
                        .terrains
                        .iter()
                        .find(|(t, _)| *t == id)
                        .map_or(&data.default, |(_, pair)| pair);
                    particles.extend(particle.iter().cloned());
                    sounds.push(sound.clone());
                }
                Ac2FxTerrainSet {
                    terrain: id,
                    particles: particles.into(),
                    sounds: sounds.into(),
                }
            })
            .collect();
        let mut particles = Vec::new();
        let mut sounds = Vec::new();
        for data in list {
            particles.extend(data.default.0);
            sounds.push(data.default.1);
        }
        (particles.into(), sounds.into(), terrains.into())
    }
}

/// One FX id's sound nodes.
pub type Ac2FxSoundEntry = (u32, Arc<[Ac2FxSoundNode]>);

/// A decoded FX map: the particle view and the sound view of the same entries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2FxMap {
    pub particles: Arc<[Ac2FxEntry]>,
    pub sounds: Arc<[Ac2FxSoundEntry]>,
    /// The ids whose `FXData` key terrain nodes, with the lists per terrain.
    pub terrains: Arc<[Ac2FxTerrainEntry]>,
}

/// `DBFXTable` (`0x1E`): FX id -> `FXData` list, plus sub-tables searched after this one.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxTable {
    pub entries: Arc<[Ac2FxEntry]>,
    pub sounds: Arc<[Ac2FxSoundEntry]>,
    /// The ids whose `FXData` key terrain nodes (`FXData::GetData`), with the lists per terrain.
    pub terrains: Arc<[Ac2FxTerrainEntry]>,
    pub sub_tables: Arc<[DataId]>,
}

impl ResourcePackedDecode for Ac2FxTable {
    const TYPE_NAME: &'static str = "AC2 DBFXTable";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if word(&mut cursor).map_err(malformed)? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let map = read_fx_map(&mut cursor).map_err(malformed)?;
        let count = count(&mut cursor, 4).map_err(malformed)?;
        let sub_tables = (0..count)
            .map(|_| word(&mut cursor).map(DataId::new))
            .collect::<Result<Vec<_>, _>>()
            .map_err(malformed)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after DBFXTable"));
        }
        Ok(Self {
            entries: map.particles,
            sounds: map.sounds,
            terrains: map.terrains,
            sub_tables: Arc::from(sub_tables),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.sub_tables.len() * 4
            + self
                .entries
                .iter()
                .map(|(_, nodes)| 8 + nodes.len() * std::mem::size_of::<Ac2FxParticleNode>())
                .sum::<usize>()
            + self
                .sounds
                .iter()
                .map(|(_, nodes)| 8 + nodes.len() * std::mem::size_of::<Ac2FxSoundNode>())
                .sum::<usize>()
    }
}

impl Ac2FxTable {
    pub fn entry(&self, fx_id: u32) -> Option<&Arc<[Ac2FxParticleNode]>> {
        self.entries
            .iter()
            .find(|(id, _)| *id == fx_id)
            .map(|(_, nodes)| nodes)
    }
}

/// `PHashTable<ulong, PSmartArray<FXData>>`: the FX map of `DBFXTable` and part-group overrides.
/// Entries keep their id even when no node carries particles, so an override still hides the
/// static table (`FXTable::GetFXData`).
pub(crate) fn read_fx_map(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2FxMap, Malformed> {
    let mut particles = Vec::new();
    let mut sounds = Vec::new();
    let mut terrains = Vec::new();
    for _ in 0..hash_count(cursor)? {
        let fx_id = word(cursor)?;
        let list = (0..count(cursor, 8)?)
            .map(|_| read_fx_data(cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let (particle, sound, terrain) = FxDataParts::segment(list);
        particles.push((fx_id, particle));
        sounds.push((fx_id, sound));
        if !terrain.is_empty() {
            terrains.push((fx_id, terrain));
        }
    }
    Ok(Ac2FxMap {
        particles: Arc::from(particles),
        sounds: Arc::from(sounds),
        terrains: Arc::from(terrains),
    })
}

/// `FXData::StreamPack`: the default `FXNode`, then a terrain-keyed `FXNode` table. The default
/// node is what `FXData::GetData` returns off the keyed terrains.
pub(crate) fn read_fx_data(cursor: &mut ArchiveCursor<'_>) -> Result<FxDataParts, Malformed> {
    let default = read_fx_node(cursor)?;
    let mut terrains = Vec::new();
    for _ in 0..hash_count(cursor)? {
        let terrain = word(cursor)?;
        terrains.push((terrain, read_fx_node(cursor)?));
    }
    Ok(FxDataParts { default, terrains })
}

/// `FXNode::StreamPack`: a 64-bit pack bitfield gating each field in stream order (absent
/// fields keep `FXNode::SetDefaults`).
fn read_fx_node(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<(Option<Ac2FxParticleNode>, Ac2FxSoundNode), Malformed> {
    let bits = u64::from(word(cursor)?) | (u64::from(word(cursor)?) << 32);
    let has = |bit: u64| bits & bit != 0;
    let field = |cursor: &mut ArchiveCursor<'_>, bit: u64| -> Result<u32, Malformed> {
        if has(bit) {
            word(cursor)
        } else {
            Ok(0)
        }
    };
    let mut sounds = Vec::new();
    if has(0x1) {
        for _ in 0..count(cursor, 4)? {
            sounds.push(word(cursor)?);
        }
    }
    if has(0x80_0000_0000) {
        // The music list.
        let n = count(cursor, 4)?;
        skip(cursor, n * 4)?;
    }
    let mut particle_descs = Vec::new();
    if has(0x2) {
        for _ in 0..count(cursor, 4)? {
            let did = word(cursor)?;
            if did != 0 {
                particle_descs.push(DataId::new(did));
            }
        }
    }
    // Camera FX and appearance ids; the appearance key is (key, scalar modifier).
    let camera_fx = field(cursor, 0x4)?;
    let appearance = field(cursor, 0x8)?;
    let appearance_key = if has(0x10) {
        (word(cursor)?, float(cursor)?)
    } else {
        (0, 0.0)
    };
    // Ramp time and intensity.
    let ramp_time = if has(0x20) { float(cursor)? } else { 0.0 };
    let intensity = if has(0x40) { float(cursor)? } else { 1.0 };
    let holding_location = field(cursor, 0x80)?;
    let bone_name = field(cursor, 0x100)?;
    let bone_offset = if has(0x200) {
        Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?)
    } else {
        Vec3::ZERO
    };
    let streak = if has(0x400) {
        // `m_has_streak`, then `StreakFX`.
        word(cursor)?;
        Some(read_streak_fx(cursor)?)
    } else {
        None
    };
    let start_streak = field(cursor, 0x800)? != 0;
    let stop_streak = field(cursor, 0x1000)? != 0;
    let slider_id = field(cursor, 0x2000)?;
    let restore = field(cursor, 0x4000)? != 0;
    // `m_has_color`, then `m_color` (r, g, b, a).
    let color = if has(0x8000) {
        let has_color = word(cursor)? != 0;
        let rgba = [
            float(cursor)?,
            float(cursor)?,
            float(cursor)?,
            float(cursor)?,
        ];
        has_color.then_some(rgba)
    } else {
        None
    };
    // Stream order: prop colour, vdesc, lifetime, colour lifetime, behaviour, parent, unparent,
    // retarget, destroy target, hide target, target FX.
    let prop_color = field(cursor, 0x1_0000)? != 0;
    let vdesc = field(cursor, 0x2_0000)?;
    let lifetime = if has(0x4_0000) { float(cursor)? } else { 15.0 };
    let color_lifetime = if has(0x20_0000_0000) {
        float(cursor)?
    } else {
        0.0
    };
    let behavior = field(cursor, 0x8_0000)?;
    let parent = field(cursor, 0x10_0000)? != 0;
    let unparent = field(cursor, 0x20_0000)? != 0;
    let retarget = field(cursor, 0x40_0000)? != 0;
    let destroy_target = field(cursor, 0x80_0000)? != 0;
    let hide_target = field(cursor, 0x8_0000_0000)? != 0;
    let target_fx_id = field(cursor, 0x100_0000)?;
    let object = Ac2FxObject {
        vdesc,
        behavior,
        lifetime,
        hide_target,
        destroy_target,
        target_fx_id,
        color,
        ramp_time,
        color_lifetime,
        restore,
        streak,
        start_streak,
        stop_streak,
        attach: Ac2FxObjectAttach::Emitter,
        holding_location,
        decal: None,
        gate: Ac2FxNodeGate::default(),
        appearance,
        appearance_key,
        camera_fx,
        slider: Ac2FxSlider {
            id: slider_id,
            intensity,
            propagate: prop_color,
        },
        repulsor: None,
        retarget,
        fx_script: 0,
        particle_looping: false,
        parent,
        unparent,
        conditional: false,
        sound: sounds.first().copied().unwrap_or(0),
        sound_looping: false,
    };
    let flag = |cursor: &mut ArchiveCursor<'_>, bit: u64, default: bool| {
        Ok::<_, Malformed>(if has(bit) {
            word(cursor)? != 0
        } else {
            default
        })
    };
    let day_time = flag(cursor, 0x200_0000, true)?;
    let night_time = flag(cursor, 0x400_0000, true)?;
    // `m_channel_keys`: key and value pairs; only the four keys `ChannelPrefs` holds compare.
    let mut has_channels = false;
    let mut channels = [None; 4];
    if has(0x800_0000) {
        let pairs = hash_count(cursor)?;
        has_channels = pairs != 0;
        for _ in 0..pairs {
            let (key, value) = (word(cursor)?, word(cursor)?);
            if let Some(at) = CHANNEL_KEYS.iter().position(|k| *k == key) {
                channels[at] = Some(value);
            }
        }
    }
    let fire_and_forget = flag(cursor, 0x1000_0000, false)?;
    let water_plane = field(cursor, 0x2000_0000)? != 0;
    let conditional = field(cursor, 0x4000_0000)? != 0;
    let mut decal = None;
    if has(0x8000_0000) {
        // `m_has_decal`, then `DecalFX`.
        word(cursor)?;
        decal = Some(read_decal_fx(cursor)?);
    }
    let mut repulsor = None;
    if has(0x2_0000_0000) {
        // `m_has_repulsor`, then `RepulsorFX`: acceleration, range, lifetime, attractor, speed.
        let has_repulsor = word(cursor)? != 0;
        let acceleration = float(cursor)?;
        let range = float(cursor)?;
        let lifetime = float(cursor)?;
        let attractor = word(cursor)? != 0;
        let minimum_velocity = float(cursor)?;
        repulsor = has_repulsor.then_some(Ac2RepulsorFx {
            acceleration,
            range,
            lifetime,
            minimum_velocity,
            attractor,
        });
    }
    let mut light = None;
    if has(0x1_0000_0000) {
        // `m_has_light`, then `LightSourceFX`: an `EntityDesc`, lifespan and fade time.
        word(cursor)?;
        let entity = crate::ac2_entity::read_ac2_entity_desc(cursor)?;
        let lifespan = float(cursor)?;
        let fade_time = float(cursor)?;
        light = Some(Ac2FxLight {
            offset: bone_offset,
            source: entity
                .properties
                .as_deref()
                .map_or(Ac2LightSource::default(), read_ac2_light_source),
            lifespan,
            fade_time,
        });
    }
    let player_only = flag(cursor, 0x4_0000_0000, false)?;
    let fx_script = field(cursor, 0x10_0000_0000)?;
    let remove_rotation = field(cursor, 0x40_0000_0000)? != 0;
    let object = Ac2FxObject {
        gate: Ac2FxNodeGate {
            has_channels,
            channels,
            player_only,
            day_time,
            night_time,
        },
        decal,
        repulsor,
        fx_script,
        conditional,
        ..object
    };
    let particle = (!particle_descs.is_empty()
        || light.is_some()
        || object.is_active()
        || object.streak.is_some()
        || object.start_streak
        || object.stop_streak)
        .then(|| Ac2FxParticleNode {
            particle_descs: Arc::from(particle_descs),
            fire_and_forget,
            light,
            holding_location,
            bone_name,
            bone_offset,
            remove_rotation,
            water_plane,
            object,
        });
    let sound = Ac2FxSoundNode {
        sounds: Arc::from(sounds),
        ramp_time,
        fire_and_forget,
        player_only,
        day_time,
        night_time,
        has_channels,
        channels,
    };
    Ok((particle, sound))
}

/// Packed hash tables lead with a word whose low 16 bits are the entry count.
pub(crate) fn hash_count(cursor: &mut ArchiveCursor<'_>) -> Result<usize, Malformed> {
    let n = (word(cursor)? & 0xFFFF) as usize;
    if n * 4 > cursor.remaining() {
        return Err("hash entry count exceeded the record");
    }
    Ok(n)
}

pub(crate) fn count(cursor: &mut ArchiveCursor<'_>, item: usize) -> Result<usize, Malformed> {
    let n = word(cursor)? as usize;
    if n > MAX_ITEMS || n * item > cursor.remaining() {
        return Err("item count exceeded the record");
    }
    Ok(n)
}

pub(crate) fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, Malformed> {
    cursor.read_u32().map_err(|_| "record ended early")
}

pub(crate) fn float(cursor: &mut ArchiveCursor<'_>) -> Result<f32, Malformed> {
    cursor.read_f32().map_err(|_| "record ended early")
}

pub(crate) fn skip(cursor: &mut ArchiveCursor<'_>, len: usize) -> Result<(), Malformed> {
    cursor
        .read_bytes(len)
        .map(|_| ())
        .map_err(|_| "record ended early")
}

#[cfg(test)]
#[path = "ac2_fx_tests.rs"]
mod tests;
