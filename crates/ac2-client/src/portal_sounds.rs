//! The portal space's sounds (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`).
//! `PortalSpace::Effect::StartNow` plays the enter sound on `Channel_Main`, then the tunnel's
//! two loops (each kept in its playing context); `StopNow` plays the exit sound and stops the
//! loops at once. The DIDs come from the DAT's sound enum (`PortalSpace::Effect::Initialize`),
//! resolved here once; the effect itself only records which of the two happened.

use ac2_audio::{PlaySoundData, SoundManager};
use ac2_render::portal::PortalSound;

/// `FileArray::GetDIDFromEnum`'s group for the sound enums (`0x40000002`).
const SOUND_GROUP: u32 = 0x4000_0002;
/// The enum values `PortalSpace::Effect::Initialize` looks up: the enter sound, the two tunnel
/// loops and the exit sound (the third loop is never set: `m_TunnelLoop2SoundID = 0`).
const ENTER_ENUM: u32 = 0x4000_0018;
const LOOP0_ENUM: u32 = 0x4000_001D;
const LOOP1_ENUM: u32 = 0x4000_001E;
const EXIT_ENUM: u32 = 0x4000_001C;

/// The portal space's sound DIDs; 0 where the DAT has none (that sound is not played).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PortalSoundIds {
    pub enter: u32,
    pub loop0: u32,
    pub loop1: u32,
    pub exit: u32,
}

/// Resolves the portal space's sounds through the DAT's enum mapper (`Initialize`'s lookups).
pub fn portal_sound_ids(store: &ac2_db::ResourceStore) -> PortalSoundIds {
    let did = |value| {
        ac2_assets::ac2_enum_did(store, SOUND_GROUP, value)
            .ok()
            .flatten()
            .map_or(0, |id| id.raw())
    };
    PortalSoundIds {
        enter: did(ENTER_ENUM),
        loop0: did(LOOP0_ENUM),
        loop1: did(LOOP1_ENUM),
        exit: did(EXIT_ENUM),
    }
}

/// `m_TunnelLoop0SoundCtx` and `m_TunnelLoop1SoundCtx`: the loops' playing sounds, kept until
/// the effect stops.
#[derive(Debug, Default)]
pub(crate) struct PortalSounds {
    ids: PortalSoundIds,
    loops: [Option<u32>; 2],
}

impl PortalSounds {
    pub(crate) fn set_ids(&mut self, ids: PortalSoundIds) {
        self.ids = ids;
    }

    /// Plays what the effect asked for. Returns the DIDs queued, in order.
    pub(crate) fn play(&mut self, manager: &mut SoundManager, events: &[PortalSound]) -> Vec<u32> {
        let mut queued = Vec::new();
        for event in events {
            match event {
                PortalSound::Start => {
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
                PortalSound::Stop => {
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

#[cfg(test)]
#[path = "portal_sounds_tests.rs"]
mod tests;
