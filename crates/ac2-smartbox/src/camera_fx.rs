//! Camera effects (Legions `portal/client/src/smartbox/camera/camerafx.cpp` and
//! `camerafxmanager.cpp`): the `CameraFX` record an FX node names (`FXNode::m_cameraFX_id`, DB type
//! `0x4000004E`, DIDs `0x55xxxxxx`), and `CameraFxManager`, which plays each effect as a track
//! that ramps in, holds and ramps out over its period. The tracks' sums are the camera's field of
//! view shift, the screen flash colour, the motion blur amount and the shake
//! (`CameraManager::UseTime`). Editor-only parsing (`FromFileNode`, `ToFileNode`) is not ported.

use ac2_core::QualifiedDataId;
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};
use ac2_physics::Vector;

use crate::camera::{ld, st};

/// `CameraFX::GetDBOType` (`0x4000004E`); the DIDs of the type are `0x55000000..=0x55FFFFFE`.
pub const CAMERA_FX_PREFIX: u32 = 0x5500_0000;

/// A decoded `CameraFX` (`CameraFX::SetDefaults` for the fields the record does not carry).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraFx {
    /// `mShake`.
    pub shake: bool,
    /// `mLookAt` (read, never acted on by `CameraManager::DoCameraFX`).
    pub look_at: bool,
    /// `mFreeCamera`, `mParentCamera`, `mTargetCamera` and `mRestoreCamera`: what the effect
    /// does to the camera rig.
    pub free_camera: bool,
    pub parent_camera: bool,
    pub target_camera: bool,
    pub restore_camera: bool,
    /// `mIntensity`, `mPeriod`, `mRampUpTime` and `mRampDownTime`.
    pub intensity: f32,
    pub period: f32,
    pub ramp_up_time: f32,
    pub ramp_down_time: f32,
    /// `mBlur`, `mHasFOV`, `mHasScreenFlash` (with `mScreenFlash`, r g b a), `mPlayerOnly` and
    /// `mCameraDistance`.
    pub blur: bool,
    pub has_fov: bool,
    pub has_screen_flash: bool,
    pub screen_flash: [f32; 4],
    pub player_only: bool,
    pub camera_distance: bool,
}

impl Default for CameraFx {
    /// `CameraFX::SetDefaults`.
    fn default() -> Self {
        Self {
            shake: false,
            look_at: false,
            free_camera: false,
            parent_camera: false,
            target_camera: false,
            restore_camera: false,
            intensity: 1.0,
            period: 1.0,
            ramp_up_time: 0.01,
            ramp_down_time: 0.01,
            blur: false,
            has_fov: false,
            has_screen_flash: false,
            screen_flash: [0.0; 4],
            player_only: false,
            camera_distance: false,
        }
    }
}

impl ResourcePackedDecode for CameraFx {
    const TYPE_NAME: &'static str = "AC2 CameraFX";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// `CameraFX::StreamPack`: the embedded DID (`DBObj::UnPack_Type`), the 64-bit pack bitfield
    /// (`get_pack_bitfield`), then the fields it names in stream order; flags travel as 32-bit
    /// words and the flash colour as one ARGB word (`RGBAColor::StreamPack`). The ramp-down time
    /// (bit `0x2000`) streams before the blur flag (`0x200`).
    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let bits = u64::from(cursor.read_u32()?) | (u64::from(cursor.read_u32()?) << 32);
        let mut fx = Self::default();
        let flag = |cursor: &mut ArchiveCursor<'_>, bit: u64, slot: &mut bool| {
            if bits & bit != 0 {
                *slot = cursor.read_u32()? != 0;
            }
            ResourceResult::Ok(())
        };
        flag(&mut cursor, 0x1, &mut fx.shake)?;
        flag(&mut cursor, 0x2, &mut fx.look_at)?;
        flag(&mut cursor, 0x4, &mut fx.free_camera)?;
        flag(&mut cursor, 0x8, &mut fx.parent_camera)?;
        flag(&mut cursor, 0x10, &mut fx.target_camera)?;
        flag(&mut cursor, 0x20, &mut fx.restore_camera)?;
        let scalar = |cursor: &mut ArchiveCursor<'_>, bit: u64, slot: &mut f32| {
            if bits & bit != 0 {
                *slot = cursor.read_f32()?;
            }
            ResourceResult::Ok(())
        };
        scalar(&mut cursor, 0x40, &mut fx.intensity)?;
        scalar(&mut cursor, 0x80, &mut fx.period)?;
        scalar(&mut cursor, 0x100, &mut fx.ramp_up_time)?;
        scalar(&mut cursor, 0x2000, &mut fx.ramp_down_time)?;
        flag(&mut cursor, 0x200, &mut fx.blur)?;
        flag(&mut cursor, 0x1000, &mut fx.has_fov)?;
        if bits & 0x400 != 0 {
            fx.has_screen_flash = cursor.read_u32()? != 0;
            // `RGBAColor::SetColor32`: a in the top byte, then r, g, b.
            let packed = cursor.read_u32()?;
            let channel = |shift: u32| ((packed >> shift) & 0xFF) as f32 / 255.0;
            fx.screen_flash = [channel(16), channel(8), channel(0), channel(24)];
        }
        flag(&mut cursor, 0x800, &mut fx.player_only)?;
        flag(&mut cursor, 0x4000, &mut fx.camera_distance)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after CameraFX"));
        }
        Ok(fx)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Loads the `CameraFX` `did` from the portal DAT (`CameraManager::DoCameraFX`'s `DBObj::Get`
/// with the `0x4000004E` type). `None` when the DAT has no such record
/// (`IError::ReportMissingDataFrom`).
pub fn load_camera_fx(
    store: &ResourceStore,
    did: u32,
) -> ResourceResult<Option<CachedResource<CameraFx>>> {
    store.load_cached_packed::<CameraFx>(TypedResourceRef::new(
        DatFileKind::Portal,
        ac2_core::DataId::new(did),
        0,
    ))
}

/// What a track adds to (`CameraFxTrack`'s `mFOV`, `mScreenFlash`, `mBlur`, `mShake` and
/// `mCameraDistance`, one set per track).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackKind {
    Fov,
    ScreenFlash([f32; 4]),
    Blur,
    Shake,
    CameraDistance,
}

/// `CameraFxTrack`: one playing effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraFxTrack {
    pub kind: TrackKind,
    /// `mWeight`: 0 to 1 over the ramps.
    pub weight: f32,
    /// `mIntensity`: the record's intensity times the FX's scalar.
    pub intensity: f32,
    pub time: f32,
    pub period: f32,
    pub ramp_up_time: f32,
    pub ramp_down_time: f32,
    /// `mSourceID`: the object the effect came from; the shake falls off with its distance.
    pub source: u64,
}

impl CameraFxTrack {
    /// `CameraFxManager::Add*`: a track of `kind` for `fx` at `scalar`, from `source`.
    fn new(kind: TrackKind, fx: &CameraFx, source: u64, scalar: f32) -> Self {
        Self {
            kind,
            weight: 0.0,
            intensity: st(ld(scalar) * ld(fx.intensity)),
            time: 0.0,
            period: fx.period,
            ramp_up_time: fx.ramp_up_time,
            ramp_down_time: fx.ramp_down_time,
            source,
        }
    }

    /// `CameraFxTrack::Update`: the clock advances; the weight rises over the ramp up, is 1
    /// while it holds and falls over the ramp down. At a ramp's exact boundary it is left as it
    /// was.
    pub fn update(&mut self, quantum: f32) {
        let time = ld(quantum) + ld(self.time);
        self.time = st(time);
        let (up, down, period) = (
            ld(self.ramp_up_time),
            ld(self.ramp_down_time),
            ld(self.period),
        );
        let time = ld(self.time);
        if time < up {
            self.weight = st(time / up);
        } else if up < time && time < period - down {
            self.weight = 1.0;
        } else if period - down < time {
            self.weight = st((period - time) / down);
        }
    }
}

/// `CameraFxManager`: the playing tracks (newest first, `List::insert` at the head) and their
/// sums after the last `use_time`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CameraFxManager {
    pub tracks: Vec<CameraFxTrack>,
    /// `mFOVSum`: degrees added to the base field of view, clamped to 10..=150 (so 10 with no
    /// track: retail's camera sees 75 degrees while it runs).
    pub fov_sum: f32,
    /// `mFlashColorSum` (r, g, b, a), each at most 1.
    pub flash_sum: [f32; 4],
    /// `mBlurSum`, at most 1.
    pub blur_sum: f32,
    /// `mShakeSum`: whether any shake plays (`GetDistortion` returns zero without one).
    pub shake_sum: f32,
    /// `mCameraDistanceSum` (summed, read by nothing).
    pub camera_distance_sum: f32,
    /// `GetDistortion`'s statics `seedx`, `seedy`, `seedz`: rolled once, on the first shake.
    seeds: Option<[f32; 3]>,
}

impl CameraFxManager {
    /// `CameraFxManager::Add{Shake,ScreenFlash,FovShift,MotionBlur,CameraDistance}`.
    pub fn add(&mut self, kind: TrackKind, fx: &CameraFx, source: u64, scalar: f32) {
        self.tracks
            .insert(0, CameraFxTrack::new(kind, fx, source, scalar));
    }

    /// `CameraFxManager::StopAllCameraFX`.
    pub fn stop_all(&mut self) {
        self.tracks.clear();
    }

    /// `CameraFxManager::UseTime`: each track advances; one past its period is removed, the
    /// others add `intensity * weight` (the flash its colour times both) to their sum. Then the
    /// clamps.
    pub fn use_time(&mut self, quantum: f32) {
        self.fov_sum = 0.0;
        self.flash_sum = [0.0; 4];
        self.blur_sum = 0.0;
        self.shake_sum = 0.0;
        self.camera_distance_sum = 0.0;
        let mut kept = Vec::with_capacity(self.tracks.len());
        for mut track in std::mem::take(&mut self.tracks) {
            track.update(quantum);
            if track.time > track.period {
                continue;
            }
            let add = |sum: f32| st(ld(track.intensity) * ld(track.weight) + ld(sum));
            match track.kind {
                TrackKind::Fov => self.fov_sum = add(self.fov_sum),
                TrackKind::ScreenFlash(colour) => {
                    for (sum, c) in self.flash_sum.iter_mut().zip(colour) {
                        *sum = st(ld(c) * ld(track.weight) * ld(track.intensity) + ld(*sum));
                    }
                }
                TrackKind::Blur => self.blur_sum = add(self.blur_sum),
                TrackKind::Shake => self.shake_sum = add(self.shake_sum),
                TrackKind::CameraDistance => {
                    self.camera_distance_sum = add(self.camera_distance_sum)
                }
            }
            kept.push(track);
        }
        self.tracks = kept;
        self.fov_sum = self.fov_sum.clamp(10.0, 150.0);
        if self.blur_sum > 1.0 {
            self.blur_sum = 1.0;
        }
        for sum in &mut self.flash_sum {
            if *sum > 1.0 {
                *sum = 1.0;
            }
        }
    }

    /// `CameraFxManager::GetDistortion`: with a shake playing, each shake track whose source and
    /// the player both exist moves the camera by fractal noise (`PerlinNoise::fBm1` at 60 steps a
    /// second from three once-rolled seeds, `H` 0.5, lacunarity 2, 3 octaves), scaled by its
    /// intensity over the source's distance (at least 1 m) and its weight; z moves three times as
    /// far. `distance(source)` is the source's distance from the player, `None` when either is
    /// gone; `roll(lo, hi)` is `Random::RollDice`.
    pub fn distortion(
        &mut self,
        distance: &mut dyn FnMut(u64) -> Option<f64>,
        roll: &mut dyn FnMut(f32, f32) -> f32,
    ) -> Vector {
        if self.shake_sum == 0.0 {
            return Vector::new(0.0, 0.0, 0.0);
        }
        let [sx, sy, sz] = *self
            .seeds
            .get_or_insert_with(|| [roll(0.0, 10.0), roll(0.0, 10.0), roll(0.0, 10.0)]);
        let noise = ac2_math::perlin::shared();
        let mut out = Vector::new(0.0, 0.0, 0.0);
        for track in &self.tracks {
            if track.kind != TrackKind::Shake {
                continue;
            }
            let steps = st(ld(track.time) * 60.0);
            let fbm = |seed: f32| ld(noise.fbm1(ld(st(ld(seed) + ld(steps))), 0.5, 2.0, 3.0));
            let (nx, ny, nz) = (fbm(sx), fbm(sy), fbm(sz));
            let Some(apart) = distance(track.source) else {
                continue;
            };
            let apart = st(apart).max(1.0);
            let scale = st(ld(track.intensity) / ld(apart) * ld(track.weight));
            out.x = st(nx * ld(scale) + ld(out.x));
            out.y = st(ld(scale) * ny + ld(out.y));
            out.z = st(nz * 3.0 * ld(scale) + ld(out.z));
        }
        out
    }
}

#[cfg(test)]
#[path = "camera_fx_tests.rs"]
mod tests;
