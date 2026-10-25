//! `AmbientSound`: the environment's sounds. When the player has moved far enough
//! (`DoSoundUpdate`), every land point and visible cell within reach adds its sound type's weight
//! by direction and distance (`ChangeCellStart`, `AddCellSound`); `ChangeCellEnd` turns weights
//! into densities, starts and stops each environment's continuous sounds, and makes the densest
//! environment dominant (its dominant-only sounds play, its `EnvInfo` reverb applies).
//! `UseTime` fires the intermittent sounds and drops environments left behind.

use std::sync::Arc;

use ac2_core::Random;

use crate::ambient_data::{AmbientData, AmbientSoundInfo};
use crate::ambient_objects::{AmbSoundObject, ContinSoundManager, IntermitObject};
use crate::manager::SoundManager;

/// The ambient sounds' emitter ids (`IIDUtils` type 0x13, temporary objects).
pub const AMBIENT_EMITTERS: u64 = 0x1300_0000_0000_0000;

/// `AmbientSoundDesc`: one environment (sound type) and its sounds.
#[derive(Debug, Clone)]
struct AmbientSoundDesc {
    sound_type: u32,
    env: u32,
    /// `min_sound_play`: the density below which it falls silent.
    min_sound_play: f32,
    intermit: Vec<IntermitObject>,
    contin: Vec<ContinSoundManager>,
    timeout_time: f64,
    active: bool,
    weight: f32,
    density: f32,
    dominant: bool,
}

impl AmbientSoundDesc {
    /// `AmbientSoundDesc::AmbientSoundDesc`.
    fn new(info: &AmbientSoundInfo, next_emitter: &mut u64) -> Self {
        Self {
            sound_type: info.sound_type,
            env: info.env,
            min_sound_play: 0.01,
            intermit: info
                .intermit
                .iter()
                .filter_map(|i| IntermitObject::new(*i, next_emitter))
                .collect(),
            contin: info
                .contin
                .iter()
                .map(|c| ContinSoundManager::new(*c, next_emitter))
                .collect(),
            timeout_time: 0.0,
            active: false,
            weight: 0.0,
            density: 0.0,
            dominant: false,
        }
    }

    /// `Reset`.
    fn reset(&mut self) {
        self.weight = 0.0;
        for amb in self.objects_mut() {
            amb.start();
        }
    }

    fn objects_mut(&mut self) -> impl Iterator<Item = &mut AmbSoundObject> {
        self.intermit
            .iter_mut()
            .map(|i| &mut i.amb)
            .chain(self.contin.iter_mut().map(|c| &mut c.amb))
    }

    /// `AddSound`: the point's weight in each of its sounds (flattened to the ground plane),
    /// averaged over those that took it; returns what it adds to the total.
    fn add_sound(&mut self, offset: [f32; 3], muffled: bool) -> f32 {
        let flat = [offset[0], offset[1], 0.0];
        let mut total = 0.0f32;
        let mut took = 0u32;
        for amb in self.objects_mut() {
            if amb.add(flat, muffled, &mut total) {
                took += 1;
            }
        }
        if took > 1 {
            total = (f64::from(total) / f64::from(took)) as f32;
        }
        self.weight = (f64::from(total) + f64::from(self.weight)) as f32;
        if total == 0.0 {
            let distance_sq = (f64::from(flat[1]) * f64::from(flat[1])
                + f64::from(flat[0]) * f64::from(flat[0])) as f32;
            return if distance_sq <= 144.0 {
                1.0
            } else {
                (144.0 / f64::from(distance_sq)) as f32
            };
        }
        total
    }

    /// `CalcWeights`: below its minimum density it falls silent and starts timing out;
    /// otherwise its sounds take their densities and play.
    fn calc_weights(
        &mut self,
        now: f64,
        inv_total: f32,
        data: &AmbientData,
        manager: &mut SoundManager,
        random: &mut Random,
    ) -> f32 {
        self.density = (f64::from(inv_total) * f64::from(self.weight)) as f32;
        if self.density < self.min_sound_play {
            for contin in &mut self.contin {
                contin.stop_all(manager);
            }
            if self.active {
                self.timeout_time = now;
                self.active = false;
            }
            return 0.0;
        }
        self.active = true;
        for intermit in &mut self.intermit {
            intermit.amb.end(inv_total);
        }
        for contin in &mut self.contin {
            contin.recalc(inv_total, self.dominant, data, manager, random);
        }
        self.density
    }

    /// `SetAsDominant`.
    fn set_dominant(&mut self, manager: &mut SoundManager, random: &mut Random) {
        self.dominant = true;
        for contin in &mut self.contin {
            contin.now_dominant(manager, random);
        }
    }

    /// `NoLongerDominant`.
    fn no_longer_dominant(&mut self, manager: &mut SoundManager) {
        self.dominant = false;
        for contin in &mut self.contin {
            contin.no_longer_dominant(manager);
        }
    }

    /// `AmbientSoundDesc::UseTime`.
    fn use_time(&mut self, now: f64, volume: f32, manager: &mut SoundManager, random: &mut Random) {
        if !self.active {
            return;
        }
        let mut played = 0;
        for intermit in &mut self.intermit {
            if intermit.play_check(now, &mut played, self.density, self.dominant, random) {
                intermit.play(volume, manager, random);
            }
        }
        for contin in &mut self.contin {
            contin.set_volume(volume);
        }
    }

    /// The environment goes (`~AmbientSoundDesc`): its sounds stop.
    fn destroy(&mut self, manager: &mut SoundManager) {
        for contin in &mut self.contin {
            contin.stop_all(manager);
        }
    }
}

/// `AmbientSound`.
#[derive(Debug)]
pub struct AmbientSound {
    data: Option<Arc<AmbientData>>,
    /// `m_bAmbientSoundsEnabled`.
    enabled: bool,
    /// `m_fAmbientSoundVolume`.
    volume: f32,
    /// `m_fTotalSoundCount`.
    total: f32,
    timeout_tick: f64,
    timeout_check: f64,
    /// `m_fMaxSoundDistance`: cells past this add nothing.
    pub max_distance: f32,
    /// `m_fSoundUpdateDistance`: how far the player moves before the cells are weighed again.
    pub update_distance: f32,
    /// `m_fNearby`: closer than this a point counts as on top of the player.
    nearby: f32,
    /// `m_currentEnvironmentIndex`.
    current: Option<usize>,
    types: Vec<AmbientSoundDesc>,
    environment: Option<u32>,
    next_emitter: u64,
    random: Random,
}

impl Default for AmbientSound {
    /// `AmbientSound::AmbientSound`.
    fn default() -> Self {
        Self {
            data: None,
            enabled: true,
            volume: 1.0,
            total: 0.0,
            timeout_tick: 0.5,
            timeout_check: 0.0,
            max_distance: 100.0,
            update_distance: 4.0,
            nearby: 0.0002,
            current: None,
            types: Vec::new(),
            environment: None,
            next_emitter: AMBIENT_EMITTERS,
            random: Random::default(),
        }
    }
}

impl AmbientSound {
    /// The region's ambient data (`CRegionDesc::GetCurrentSoundInfo`'s `CSoundDesc`), read on
    /// the loader thread; without it `AddCellSound` disables the ambient sound, as retail does.
    pub fn set_data(&mut self, data: Option<Arc<AmbientData>>) {
        self.data = data;
    }

    /// `AmbientSound::Init`.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// `m_fAmbientSoundVolume`.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    /// `ChangeCellStart`.
    pub fn change_cell_start(&mut self) {
        self.total = 0.0;
        for desc in &mut self.types {
            desc.reset();
        }
    }

    /// `AddCellSound`: a land point (`outdoor_point`) or a visible cell's sound type at
    /// `offset` from the player (global axes); muffled when it is on the other side of the
    /// inside / outside line from the player. Whether it was taken.
    pub fn add_cell_sound(
        &mut self,
        sound_type: u32,
        offset: [f32; 3],
        outdoor_point: bool,
        player_outside: bool,
    ) -> bool {
        let [x, y, z] = offset.map(f64::from);
        let distance_sq = (x * x + y * y + z * z) as f32;
        let muffled = outdoor_point != player_outside;
        let max = f64::from(self.max_distance);
        if f64::from(distance_sq) > max * max {
            return false;
        }
        let nearby = f64::from(self.nearby);
        let offset = if f64::from(distance_sq) < nearby * nearby {
            [0.0; 3]
        } else {
            offset
        };
        let index = match self.types.iter().position(|d| d.sound_type == sound_type) {
            Some(index) => index,
            None => {
                let Some(data) = &self.data else {
                    // "AmbientSound: No ambient sound information, disabling..."
                    self.enabled = false;
                    return false;
                };
                let Some(info) = data.desc.sound_info(sound_type) else {
                    return false;
                };
                let desc = AmbientSoundDesc::new(info, &mut self.next_emitter);
                self.types.push(desc);
                self.types.len() - 1
            }
        };
        let added = self.types[index].add_sound(offset, muffled);
        self.total = (f64::from(added) + f64::from(self.total)) as f32;
        true
    }

    /// `ChangeCellEnd`: densities from the weights over the total; the densest environment
    /// becomes dominant and its reverb the environment's.
    pub fn change_cell_end(&mut self, now: f64, manager: &mut SoundManager) {
        let inv_total = if f64::from(self.total).abs() >= 0.0002 {
            (1.0 / f64::from(self.total)) as f32
        } else {
            0.0
        };
        let Some(data) = self.data.clone() else {
            return;
        };
        let mut best = None;
        let mut best_density = 0.0f32;
        for (index, desc) in self.types.iter_mut().enumerate() {
            let density = desc.calc_weights(now, inv_total, &data, manager, &mut self.random);
            if best_density < density {
                best = Some(index);
                best_density = density;
            }
        }
        if best != self.current {
            if let Some(desc) = best.and_then(|index| self.types.get_mut(index)) {
                desc.set_dominant(manager, &mut self.random);
                self.environment = Some(desc.env);
            }
            if let Some(desc) = self.current.and_then(|index| self.types.get_mut(index)) {
                desc.no_longer_dominant(manager);
            }
            self.current = best;
        }
    }

    /// `AmbientSound::UseTime`: every half second the environments silent long enough go;
    /// the active ones fire their intermittent sounds.
    pub fn use_time(&mut self, now: f64, manager: &mut SoundManager) {
        if !self.enabled {
            return;
        }
        if self.timeout_check < now {
            self.timeout_check = now + self.timeout_tick;
            let mut index = 0;
            while index < self.types.len() {
                let desc = &self.types[index];
                if !desc.active && desc.timeout_time <= now {
                    let mut desc = self.types.swap_remove(index);
                    desc.destroy(manager);
                } else {
                    index += 1;
                }
            }
        }
        for desc in &mut self.types {
            desc.use_time(now, self.volume, manager, &mut self.random);
        }
    }

    /// The dominant environment's `EnvInfo` (the reverb `SetAsDominant` applied).
    pub fn environment(&self) -> Option<u32> {
        self.environment
    }

    /// The sound types being weighed, the dominant one first.
    pub fn sound_types(&self) -> Vec<u32> {
        let mut types: Vec<_> = self.types.iter().map(|d| d.sound_type).collect();
        if let Some(index) = self.current.filter(|&i| i < types.len()) {
            types.swap(0, index);
        }
        types
    }

    /// Every ambient emitter and where it is from the viewer (global axes).
    pub fn emitters(&self) -> Vec<(u64, [f32; 3])> {
        self.types
            .iter()
            .flat_map(|desc| {
                desc.contin
                    .iter()
                    .flat_map(|c| c.objects.iter().map(|o| (o.emitter, o.offset)))
                    .chain(desc.intermit.iter().map(|i| (i.emitter, i.offset)))
            })
            .collect()
    }

    /// Where an ambient emitter is, from the viewer (global axes): a continuous sound's offset
    /// or an intermittent sound's last place.
    pub fn emitter_offset(&self, emitter: u64) -> Option<[f32; 3]> {
        self.types.iter().find_map(|desc| {
            desc.contin
                .iter()
                .flat_map(|c| c.objects.iter())
                .find(|o| o.emitter == emitter)
                .map(|o| o.offset)
                .or_else(|| {
                    desc.intermit
                        .iter()
                        .find(|i| i.emitter == emitter)
                        .map(|i| i.offset)
                })
        })
    }

    /// `AmbientSound::Clear`: every environment goes, its sounds stopped.
    pub fn clear(&mut self, manager: &mut SoundManager) {
        for mut desc in self.types.drain(..) {
            desc.destroy(manager);
        }
        self.current = None;
        self.environment = None;
    }
}
