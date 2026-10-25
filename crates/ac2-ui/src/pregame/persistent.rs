//! `UIPersistantData`: what the pregame frameworks share across mode switches. The character
//! set the server sent (`SetCharSet`, kept up to date by the client: a created character is
//! added on its verification, `CharacterSet::AddIdentity`), the slot to select when character
//! selection opens (`_genSlot`), and character generation's results by race and sex
//! (`_generationResultTable`, `_currentCGR`, `_inCharGen`).

use std::collections::HashMap;

use ac2_net::protocol::{CharGenResult, CharacterIdentity, CharacterSet};

/// `UIPersistantData`'s fields.
#[derive(Debug, Clone, Default)]
pub struct PersistentData {
    /// `_charSet`, once received (`_receivedSet`).
    characters: Option<CharacterSet>,
    /// `_genSlot`.
    gen_slot: usize,
    /// `_generationResultTable`: one result per race and sex, keyed `race + sex`.
    generation: HashMap<u32, CharGenResult>,
    /// `_currentCGR`: the key of the result being edited.
    current: u32,
    /// `_inCharGen`.
    in_char_gen: bool,
}

impl PersistentData {
    /// `UIPersistantData::SetCharSet`.
    pub fn set_characters(&mut self, set: CharacterSet) {
        self.characters = Some(set);
    }

    /// `_charSet` (`None` until received).
    pub fn characters(&self) -> Option<&CharacterSet> {
        self.characters.as_ref()
    }

    /// `_charSet` for an update.
    pub fn characters_mut(&mut self) -> Option<&mut CharacterSet> {
        self.characters.as_mut()
    }

    /// `CharacterSet::AddIdentity(identity, -1)`: added at the end unless already listed; its
    /// slot.
    pub fn add_identity(&mut self, identity: CharacterIdentity) -> Option<usize> {
        let set = self.characters.as_mut()?;
        if let Some(slot) = set.characters.iter().position(|c| c.id == identity.id) {
            return Some(slot);
        }
        set.characters.push(identity);
        Some(set.characters.len() - 1)
    }

    /// `GetGenSlot`.
    pub fn gen_slot(&self) -> usize {
        self.gen_slot
    }

    /// `SetGenSlot`.
    pub fn set_gen_slot(&mut self, slot: usize) {
        self.gen_slot = slot;
    }

    /// `UIPersistantData::InCharGen` (getter).
    pub fn in_char_gen(&self) -> bool {
        self.in_char_gen
    }

    /// `UIPersistantData::InCharGen`.
    pub fn set_in_char_gen(&mut self, in_char_gen: bool) {
        self.in_char_gen = in_char_gen;
    }

    /// `UIPersistantData::SetCharGenResult`.
    pub fn set_char_gen_result(&mut self, key: u32, result: CharGenResult) {
        self.generation.insert(key, result);
    }

    /// `UIPersistantData::InqCharGenResult`.
    pub fn char_gen_result(&self, key: u32) -> Option<&CharGenResult> {
        self.generation.get(&key)
    }

    /// `UIPersistantData::SetCurrentCGR`.
    pub fn set_current(&mut self, key: u32) {
        self.current = key;
    }

    /// `UIPersistantData::InqCurrentCGR`.
    pub fn current(&self) -> Option<&CharGenResult> {
        self.generation.get(&self.current)
    }

    /// `InqCurrentCGR` for editing.
    pub fn current_mut(&mut self) -> Option<&mut CharGenResult> {
        self.generation.get_mut(&self.current)
    }

    /// `UIPersistantData::ClearGenerationResultTable`.
    pub fn clear_generation(&mut self) {
        self.generation.clear();
        self.in_char_gen = false;
        self.current = 0;
    }
}

/// `CharacterSet::GetIdentity`: the identity in `slot`, unless its id is `INVALID_IID` (0).
pub fn identity(set: &CharacterSet, slot: usize) -> Option<&CharacterIdentity> {
    set.characters.get(slot).filter(|c| c.id != 0)
}

/// `CharacterSet::GetGreyedOutFor`: the seconds until a character pending deletion is gone
/// (0 for an active one); only slots within the allowed count (any when it is below 1).
pub fn greyed_out_for(set: &CharacterSet, slot: usize) -> u32 {
    let allowed = set.allowed as i32;
    if slot as i64 >= i64::from(allowed) && allowed >= 1 {
        return 0;
    }
    identity(set, slot).map_or(0, |c| c.seconds_greyed_out)
}

/// `CharacterSet::SetGreyedOutFor`.
pub fn set_greyed_out_for(set: &mut CharacterSet, slot: usize, seconds: u32) {
    if let Some(c) = set.characters.get_mut(slot).filter(|c| c.id != 0) {
        c.seconds_greyed_out = seconds;
    }
}
