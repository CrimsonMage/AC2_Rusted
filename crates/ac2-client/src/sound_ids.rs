//! The sound DIDs the client resolves from the DAT's sound enum once the DAT is open: the rain
//! sounds (`WorldRender::Initialize`) and the portal space's (`PortalSpace::Effect::Initialize`).
//! The sound code that plays them lives in `ac2-audio`.

use ac2_audio::{PortalSoundIds, RainSoundIds};

/// `FileArray::GetDIDFromEnum`'s group for the sound enums (`0x40000002`).
const SOUND_GROUP: u32 = 0x4000_0002;
/// The rain loop's enum (`m_RainLoop0SoundID`).
const RAIN_LOOP_ENUM: u32 = 0x4000_0025;
/// The four thunder enums (`m_RainThunder0..3SoundID`).
const THUNDER_ENUMS: [u32; 4] = [0x4000_0026, 0x4000_0027, 0x4000_0028, 0x4000_0029];
/// The enum values `PortalSpace::Effect::Initialize` looks up: the enter sound, the two tunnel
/// loops and the exit sound (the third loop is never set: `m_TunnelLoop2SoundID = 0`).
const PORTAL_ENTER_ENUM: u32 = 0x4000_0018;
const PORTAL_LOOP0_ENUM: u32 = 0x4000_001D;
const PORTAL_LOOP1_ENUM: u32 = 0x4000_001E;
const PORTAL_EXIT_ENUM: u32 = 0x4000_001C;

/// The DID of a sound enum value; 0 where the DAT has none.
fn sound_did(store: &ac2_db::ResourceStore, value: u32) -> u32 {
    ac2_assets::ac2_enum_did(store, SOUND_GROUP, value)
        .ok()
        .flatten()
        .map_or(0, |id| id.raw())
}

/// Resolves the rain sounds through the DAT's enum mapper (`WorldRender::Initialize`).
pub fn rain_sound_ids(store: &ac2_db::ResourceStore) -> RainSoundIds {
    RainSoundIds {
        rain_loop: sound_did(store, RAIN_LOOP_ENUM),
        thunder: THUNDER_ENUMS.map(|value| sound_did(store, value)),
    }
}

/// Resolves the portal space's sounds through the DAT's enum mapper (`Initialize`'s lookups).
pub fn portal_sound_ids(store: &ac2_db::ResourceStore) -> PortalSoundIds {
    PortalSoundIds {
        enter: sound_did(store, PORTAL_ENTER_ENUM),
        loop0: sound_did(store, PORTAL_LOOP0_ENUM),
        loop1: sound_did(store, PORTAL_LOOP1_ENUM),
        exit: sound_did(store, PORTAL_EXIT_ENUM),
    }
}

#[cfg(test)]
#[path = "sound_ids_tests.rs"]
mod tests;
