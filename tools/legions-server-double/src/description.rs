//! The player's qualities as the login describes them (`ECM_Login::RecvEvt_PlayerDesc`,
//! `CBaseQualities`): the weenie type, the race and the vitals.

use crate::character::RACE_HUMAN;
use crate::vitals::{login_floats, login_ints};
use crate::words;

/// The qualities of the double's default player: int 20 (the weenie type, 31: a player for
/// `EngineWeenieTypeSystem` 0xC), int 0x19 and int 290 (the race, Human), the health and vigor
/// qualities of `health` and `vigor` (current, max), and the regeneration rates.
pub(crate) fn player_description(health: (u32, u32), vigor: (u32, u32)) -> Vec<u8> {
    player_description_of(RACE_HUMAN, health, vigor)
}

/// [`player_description`] for a character of `race`: the header names the int and float tables
/// (bits 2 and 8), then the DID, the ints and the floats, each as `PHashTable::StreamPack`.
pub(crate) fn player_description_of(race: u32, health: (u32, u32), vigor: (u32, u32)) -> Vec<u8> {
    let mut ints = vec![(20, 31), (0x19, 7), (290, race as i32)];
    ints.extend(login_ints(health, vigor).map(|(q, v)| (q, v)));
    ints.sort_by_key(|(q, _)| *q);
    let mut b = Vec::new();
    words(
        &mut b,
        &[0x0001_0068, 2 | 8, 0x8100_110F, ints.len() as u32],
    );
    for (quality, value) in ints {
        words(&mut b, &[quality, value as u32]);
    }
    let floats = login_floats();
    words(&mut b, &[floats.len() as u32]);
    for (quality, value) in floats {
        words(&mut b, &[quality, value.to_bits()]);
    }
    b
}
