//! The ambient sound's pieces: `SoundDirection` and `AmbSoundObject` (a sound's weight from
//! the cells around, by eight compass directions), `ContinSoundManager` with its
//! `ContinObject`s (continuous sounds) and `IntermitObject` (sounds fired on timers).

use ac2_core::Random;

use crate::ambient_data::{AmbientData, ContinSoundInfo, IntermitSoundInfo};
use crate::manager::SoundManager;
use crate::play_data::{Channel, PlaySoundData, SoundType};

/// `Vector::FromSpherical(radius, theta, phi)`, degrees: x = sin θ cos φ r, y = cos θ cos φ r,
/// z = sin φ r.
pub(crate) fn from_spherical(radius: f32, theta: f32, phi: f32) -> [f32; 3] {
    let rad = 0.017453292f32 as f64;
    let (theta, phi, radius) = (
        f64::from(theta) * rad,
        f64::from(phi) * rad,
        f64::from(radius),
    );
    [
        (theta.sin() * phi.cos() * radius) as f32,
        (theta.cos() * phi.cos() * radius) as f32,
        (phi.sin() * radius) as f32,
    ]
}

/// `SoundDirection`: one 45-degree sector's weight and the distances seen in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SoundDirection {
    weight: f32,
    pub density: f32,
    min_distance: f32,
    max_distance: f32,
}

impl Default for SoundDirection {
    /// `SoundDirection::SoundDirection` / `Reset`.
    fn default() -> Self {
        Self {
            weight: 0.0,
            density: 0.0,
            min_distance: -1.0,
            max_distance: -1.0,
        }
    }
}

impl SoundDirection {
    /// `AddToDistance`.
    fn add_distance(&mut self, distance: f32) {
        if distance < self.min_distance || self.min_distance == -1.0 {
            self.min_distance = distance;
        }
        if self.max_distance < distance || self.max_distance == -1.0 {
            self.max_distance = distance;
        }
    }

    /// `ChooseSoundDistance`.
    fn choose_distance(&self, fraction: f32) -> f32 {
        ((f64::from(self.max_distance) - f64::from(self.min_distance)) * f64::from(fraction)
            + f64::from(self.min_distance)) as f32
    }
}

/// `AmbSoundObject`.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct AmbSoundObject {
    weight: f32,
    pub density: f32,
    pub dirs: [SoundDirection; 8],
}

/// `AmbSoundObject::GetDir`: the heading of `offset` (`Heading` of 5π/2 minus its angle) in
/// 45-degree sectors.
fn direction(offset: [f32; 3]) -> usize {
    let (x, y) = (f64::from(offset[0]), f64::from(offset[1]));
    let length = (x * x + y * y).sqrt();
    let degrees = if length < 0.0002 {
        0.0
    } else {
        let inv = 1.0 / length;
        let angle = (inv * y).atan2(f64::from((inv * x) as f32));
        (7.853981633974483 - angle).to_degrees().rem_euclid(360.0) as f32
    };
    ((f64::from(degrees) * f64::from(0.022222223f32)).floor() as usize).min(7)
}

impl AmbSoundObject {
    /// `AddSoundStart`.
    pub fn start(&mut self) {
        self.weight = 0.0;
        self.density = 0.0;
        self.dirs = Default::default();
    }

    /// `AddSound`: a point within 100 m weighs 1 (or 144 / distance² past 12 m), half when
    /// muffled, in its direction; one on top of the listener weighs 1 spread over all eight.
    pub fn add(&mut self, offset: [f32; 3], muffled: bool, total: &mut f32) -> bool {
        let [x, y, z] = offset.map(f64::from);
        let distance_sq = (z * z + y * y + x * x) as f32;
        if distance_sq > 10000.0 {
            return false;
        }
        let mut weight = if distance_sq > 144.0 {
            (144.0 / f64::from(distance_sq)) as f32
        } else {
            1.0
        };
        if muffled {
            weight = (f64::from(weight) * 0.5) as f32;
        }
        if distance_sq >= 0.0002 {
            let dir = &mut self.dirs[direction(offset)];
            dir.weight = (f64::from(weight) + f64::from(dir.weight)) as f32;
            dir.add_distance(f64::from(distance_sq).sqrt() as f32);
        } else {
            weight = 1.0;
            for dir in &mut self.dirs {
                dir.weight = (0.125 + f64::from(dir.weight)) as f32;
                dir.add_distance(0.0);
            }
        }
        self.weight = (f64::from(weight) + f64::from(self.weight)) as f32;
        if weight > 0.0 {
            *total = (f64::from(weight) + f64::from(*total)) as f32;
            return true;
        }
        false
    }

    /// `AddSoundEnd`.
    pub fn end(&mut self, inv_total: f32) {
        self.density = (f64::from(inv_total) * f64::from(self.weight)) as f32;
        for dir in &mut self.dirs {
            dir.density = (f64::from(inv_total) * f64::from(dir.weight)) as f32;
        }
    }

    /// `GetIntermitLoc`: a direction picked by its density, a random angle inside it and a
    /// distance between the nearest and farthest points seen there.
    fn intermit_offset(&self, random: &mut Random) -> [f32; 3] {
        let mut left = random.roll_dice_f32(0.0, 1.0);
        let mut pick = 7;
        for (index, dir) in self.dirs.iter().take(7).enumerate() {
            left = (f64::from(left) - f64::from(dir.density)) as f32;
            if left < 0.0 {
                pick = index;
                break;
            }
        }
        let angle = random.roll_dice_f32(0.0, 45.0);
        let fraction = random.roll_dice_f32(0.0, 1.0);
        let distance = self.dirs[pick].choose_distance(fraction);
        from_spherical(
            distance,
            (pick as f64 * 45.0 + f64::from(angle)) as f32,
            0.0,
        )
    }
}

/// `ContinObject` (`ContinMusicObject` / `ContinSoundObject`): a continuous sound placed at an
/// offset from the viewer.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinObject {
    pub emitter: u64,
    sound: u32,
    music: bool,
    /// `player_offset`, global axes.
    pub offset: [f32; 3],
    /// `sound_density`.
    density: f32,
    /// `m_soundContext`.
    handle: u32,
}

impl ContinObject {
    /// `ContinObject::Init`.
    fn init(&mut self, offset: [f32; 3], density: f32) {
        self.offset = offset;
        self.density = density;
    }

    /// `ContinMusicObject::PlaySound` / `ContinSoundObject::PlaySound` when `ShouldPlay`:
    /// music on the music channel, a sound on the ambient channel at `pitch`, self-exclusive.
    fn play(&mut self, volume: f32, pitch: f32, manager: &mut SoundManager) {
        if self.density < 0.0002 {
            return;
        }
        let mut data = PlaySoundData::new(self.sound);
        data.emitter = Some(self.emitter);
        data.volume = (f64::from(volume) * f64::from(self.density)) as f32;
        if self.music {
            data.channel = Channel::Music;
        } else {
            data.frequency = pitch;
            data.kind = SoundType::ForceSelfExclusiveBudgeted;
            data.channel = Channel::Ambient;
        }
        if let Some(handle) = manager.play(&data) {
            self.handle = handle;
        }
    }

    /// `StopAllSounds` (music: `StopSoundsForEmitter`) / `ContinSoundObject::StopSound`.
    fn stop(&mut self, manager: &mut SoundManager) {
        if self.music {
            manager.stop_sounds_for_emitter(self.emitter);
        } else {
            manager.stop_context(self.handle, 0.0);
            self.handle = 0;
        }
    }
}

/// `ContinSoundManager`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinSoundManager {
    pub amb: AmbSoundObject,
    info: ContinSoundInfo,
    /// `m_vol`.
    volume: f32,
    pub objects: Vec<ContinObject>,
}

impl ContinSoundManager {
    /// `ContinSoundManager::ContinSoundManager`: four objects with the `0x10` bit, else one;
    /// music for bit 2, a sound for bit 1.
    pub fn new(info: ContinSoundInfo, next_emitter: &mut u64) -> Self {
        let copies = if info.test(0x10) { 4 } else { 1 };
        let music = info.test(2);
        let objects = if music || info.test(1) {
            (0..copies)
                .map(|_| {
                    *next_emitter += 1;
                    ContinObject {
                        emitter: *next_emitter,
                        sound: info.sound,
                        music,
                        offset: [0.0; 3],
                        density: 0.0,
                        handle: 0,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        Self {
            amb: AmbSoundObject::default(),
            info,
            volume: 1.0,
            objects,
        }
    }

    /// `SetVolume`.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    /// `PlaySounds`: each object at the sound's volume times the manager's, a random pitch.
    fn play_sounds(&mut self, manager: &mut SoundManager, random: &mut Random) {
        let volume = (f64::from(self.info.volume) * f64::from(self.volume)) as f32;
        for object in &mut self.objects {
            let pitch = random.roll_dice_f32(self.info.min_pitch, self.info.max_pitch);
            object.play(volume, pitch, manager);
        }
    }

    /// `StopAllSounds`, unless the sound keeps playing (bit 8).
    pub fn stop_all(&mut self, manager: &mut SoundManager) {
        if self.info.test(8) {
            return;
        }
        for object in &mut self.objects {
            object.stop(manager);
        }
    }

    /// `NowDominant`: a dominant-only sound starts.
    pub fn now_dominant(&mut self, manager: &mut SoundManager, random: &mut Random) {
        if self.info.test(4) {
            self.play_sounds(manager, random);
        }
    }

    /// `NoLongerDominant`.
    pub fn no_longer_dominant(&mut self, manager: &mut SoundManager) {
        if self.info.test(4) {
            self.stop_all(manager);
        }
    }

    /// `ReCalcContinSounds`: the object's place and density from the cells' weights (a 2D
    /// sound sits on the viewer; a 3D one at ten times the sum of its eight sector vectors),
    /// then it plays unless it waits to be dominant.
    pub fn recalc(
        &mut self,
        inv_total: f32,
        dominant: bool,
        data: &AmbientData,
        manager: &mut SoundManager,
        random: &mut Random,
    ) {
        self.amb.end(inv_total);
        if self.objects.is_empty() {
            return;
        }
        let density = self.amb.density;
        if !data.is_3d(self.info.sound) {
            self.objects[0].init([0.0; 3], density);
        } else if !self.info.test(0x10) {
            // Retail sums eight vectors of the same length, 45 degrees apart.
            let mut sum = [0.0f32; 3];
            for sector in 0..8 {
                let v = from_spherical(density, ((sector as f64 + 0.05) * 45.0) as f32, 0.0);
                for axis in 0..3 {
                    sum[axis] = (f64::from(sum[axis]) + f64::from(v[axis])) as f32;
                }
            }
            self.objects[0].init(sum.map(|v| (f64::from(v) * 10.0) as f32), density);
        } else {
            // The four-copy spread (bit 0x10) is used by no Legions sound.
            for object in &mut self.objects {
                object.init([0.0; 3], density);
            }
        }
        if dominant || !self.info.test(4) {
            self.play_sounds(manager, random);
        }
    }
}

/// `IntermitObject` (`IntermitMusicObject` / `IntermitSoundObject`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IntermitObject {
    pub amb: AmbSoundObject,
    info: IntermitSoundInfo,
    update_time: f64,
    pub emitter: u64,
    /// Where it last fired, from the viewer.
    pub offset: [f32; 3],
    handle: u32,
}

impl IntermitObject {
    pub fn new(info: IntermitSoundInfo, next_emitter: &mut u64) -> Option<Self> {
        if !info.test(2) && !info.test(1) {
            return None;
        }
        *next_emitter += 1;
        Some(Self {
            amb: AmbSoundObject::default(),
            info,
            update_time: 0.0,
            emitter: *next_emitter,
            offset: [0.0; 3],
            handle: 0,
        })
    }

    /// `PlayCheck`: when its timer runs out it rolls a new wait, and fires (one per environment
    /// each call) by its probability times the environment's density; otherwise the wait is
    /// shortened at random.
    pub fn play_check(
        &mut self,
        now: f64,
        played: &mut u32,
        density: f32,
        dominant: bool,
        random: &mut Random,
    ) -> bool {
        if now <= self.update_time {
            return false;
        }
        let wait = random.roll_dice_f32(self.info.min_time, self.info.max_time);
        self.update_time = f64::from((f64::from(wait) + f64::from(now as f32)) as f32);
        if *played == 0 && (dominant || !self.info.test(4)) {
            let roll = random.roll_dice_f32(0.0, 1.0);
            if f64::from(roll) <= f64::from(self.info.probability) * f64::from(density) {
                *played += 1;
                return true;
            }
        } else {
            let roll = random.roll_dice_f32(0.0, 1.0);
            let left = (self.update_time - now) as f32;
            self.update_time =
                f64::from((f64::from(roll) * f64::from(left) + f64::from(now as f32)) as f32);
        }
        false
    }

    /// `IntermitObject::Play`: unless still playing, a place picked around the viewer
    /// (`GetIntermitLoc`, raised by `CalcHeight`), then `Play_Internal` at the sound's volume.
    pub fn play(&mut self, volume: f32, manager: &mut SoundManager, random: &mut Random) {
        if self.handle != 0 && !manager.has_sound_finished(self.handle) {
            return;
        }
        let mut offset = self.amb.intermit_offset(random);
        // `IntermitObject::CalcHeight` rolls between `m_max_pitch` and `m_min_height`.
        let height = random.roll_dice_f32(self.info.max_pitch, self.info.min_height);
        offset[2] = (f64::from(offset[2]) + f64::from(height)) as f32;
        self.offset = offset;
        random.roll_dice_f32(self.info.min_pitch, self.info.max_pitch);
        let mut data = PlaySoundData::new(self.info.sound);
        data.emitter = Some(self.emitter);
        data.volume = (f64::from(self.info.volume) * f64::from(volume)) as f32;
        if self.info.test(2) {
            data.channel = Channel::Music;
        } else {
            data.kind = SoundType::ForceSelfExclusiveBudgeted;
            data.channel = Channel::Ambient;
        }
        if let Some(handle) = manager.play(&data) {
            self.handle = handle;
        }
    }
}
