//! The rain and thunder sounds (`WorldRender::UpdateRainSounds`, Legions
//! `portal/client/src/smartbox/worldrender/worldrender.cpp.c`). The rain loop follows the rain
//! amount (`WorldRender::m_fRainAmount`, `CDayDesc::GetRain`); above 0.4 a thunder sound plays
//! when its timer runs out, and the next one is set 15 to 60 seconds later. The client resolves
//! the DIDs from the DAT's sound enum (`WorldRender::Initialize`) and hands them over once.

use ac2_core::Random;

use crate::manager::SoundManager;
use crate::play_data::PlaySoundData;

/// `UpdateRainSounds`: the thunder starts above this rain amount.
const THUNDER_AMOUNT: f32 = 0.4;
/// `F_EPSILON`: the volume and amount tolerance.
const EPSILON: f32 = 0.0002;

/// The rain loop's and the four thunder sounds' DIDs; 0 where the DAT has none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RainSoundIds {
    pub rain_loop: u32,
    pub thunder: [u32; 4],
}

/// `m_RainLoop0SoundCtx` (the loop's playing sound), `m_fRainSoundVolume`, and
/// `m_dNextRainThunderLocalTime`.
#[derive(Debug, Default)]
pub struct RainSounds {
    ids: RainSoundIds,
    loop_handle: u32,
    loop_volume: f32,
    next_thunder: f64,
}

impl RainSounds {
    pub fn set_ids(&mut self, ids: RainSoundIds) {
        self.ids = ids;
    }

    /// `WorldRender::UpdateRainSounds(amount)`: `amount` is clamped to `[0, 1]` and is 0 when
    /// the camera cannot see outside. Returns the DIDs queued this call, in order (the thunder,
    /// then the loop when it starts).
    pub fn update(
        &mut self,
        amount: f32,
        can_see_outside: bool,
        now: f64,
        random: &mut Random,
        manager: &mut SoundManager,
    ) -> Vec<u32> {
        let mut queued = Vec::new();
        let mut amount = if amount <= 1.0 {
            if amount < 0.0 {
                0.0
            } else {
                amount
            }
        } else {
            1.0
        };
        if !can_see_outside {
            amount = 0.0;
        }
        if amount > THUNDER_AMOUNT && now > self.next_thunder {
            let pick = random.roll_dice(0, 3) as usize;
            let sound = self.ids.thunder.get(pick).copied().unwrap_or(0);
            if sound != 0 {
                let mut data = PlaySoundData::new(sound);
                data.volume = random.roll_dice_f32(0.25, 1.0);
                manager.play(&data);
                queued.push(sound);
            }
            let gap = random.roll_dice_f32(15.0, 60.0);
            self.next_thunder = f64::from(gap + now as f32);
        }
        if (amount - self.loop_volume).abs() >= EPSILON {
            if self.loop_handle == 0 {
                // The loop starts at the amount (`SoundManager::Play` into the loop's context).
                if self.ids.rain_loop != 0 {
                    let mut data = PlaySoundData::new(self.ids.rain_loop);
                    data.volume = amount;
                    self.loop_handle = manager.play(&data).unwrap_or(0);
                    queued.push(self.ids.rain_loop);
                }
            } else if amount.abs() < EPSILON {
                manager.stop_context(self.loop_handle, 0.0);
                self.loop_handle = 0;
            } else {
                manager.set_context_volume(self.loop_handle, amount);
            }
            self.loop_volume = amount;
        }
        queued
    }
}
