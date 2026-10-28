//! The portal space's sounds (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`).
//! `PortalSpace::Effect::StartNow` plays the enter sound on `Channel_Main`, then the tunnel's
//! two loops (each kept in its playing context); `StopNow` plays the exit sound and stops the
//! loops at once. The client resolves the DIDs from the DAT's sound enum
//! (`PortalSpace::Effect::Initialize`) and reports the effect's start and stop as [`PortalCue`].

use crate::manager::SoundManager;
use crate::play_data::PlaySoundData;

/// The portal space's sound DIDs; 0 where the DAT has none (that sound is not played).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PortalSoundIds {
    pub enter: u32,
    pub loop0: u32,
    pub loop1: u32,
    pub exit: u32,
}

/// What the portal effect did since the last frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalCue {
    /// `StartNow`: the enter sound, then the tunnel's two loops.
    Start,
    /// `StopNow`: the exit sound, then the loops stop.
    Stop,
}

/// `m_TunnelLoop0SoundCtx` and `m_TunnelLoop1SoundCtx`: the loops' playing sounds, kept until
/// the effect stops.
#[derive(Debug, Default)]
pub struct PortalSounds {
    ids: PortalSoundIds,
    loops: [Option<u32>; 2],
}

impl PortalSounds {
    pub fn set_ids(&mut self, ids: PortalSoundIds) {
        self.ids = ids;
    }

    /// Plays what the effect asked for. Returns the DIDs queued, in order.
    pub fn play(&mut self, manager: &mut SoundManager, cues: &[PortalCue]) -> Vec<u32> {
        let mut queued = Vec::new();
        for cue in cues {
            match cue {
                PortalCue::Start => {
                    let ids = self.ids;
                    let starts = std::iter::once((ids.enter, None))
                        .chain([(ids.loop0, Some(0)), (ids.loop1, Some(1))]);
                    for (did, slot) in starts {
                        if did == 0 {
                            continue;
                        }
                        let handle = manager.play(&PlaySoundData::new(did));
                        if let (Some(slot), Some(handle)) = (slot, handle.filter(|h| *h != 0)) {
                            self.loops[slot] = Some(handle);
                        }
                        queued.push(did);
                    }
                }
                PortalCue::Stop => {
                    if self.ids.exit != 0 {
                        manager.play(&PlaySoundData::new(self.ids.exit));
                        queued.push(self.ids.exit);
                    }
                    for handle in self.loops.iter_mut().filter_map(Option::take) {
                        manager.stop_context(handle, 0.0);
                    }
                }
            }
        }
        queued
    }
}
