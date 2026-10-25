//! `gmCharGenResult` (native 0x9E): the character the creation screens build.

use super::{PlainKeys, Position, RefKey, ScriptHash};

/// `gmCharGenResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CharGenResult {
    pub name: Vec<u16>,
    /// `m_slot`.
    pub slot: i32,
    pub entity_did: u32,
    /// `startPos`.
    pub start_pos: Position,
    pub race: u32,
    pub sex: u32,
    pub class: u32,
    pub implement: u32,
    /// `m_ptValues` (`PHashTable<unsigned long,float>`, values as float bits).
    pub pt_values: ScriptHash,
    pub start_area: u32,
}

impl Default for CharGenResult {
    /// `gmCharGenResult::gmCharGenResult`: an identity start position, a 47-bucket value table,
    /// then `Clear`.
    fn default() -> Self {
        Self {
            name: Vec::new(),
            slot: 0,
            entity_did: 0,
            start_pos: Position {
                cell: 0,
                origin: [0.0; 3],
                rotation: [1.0, 0.0, 0.0, 0.0],
            },
            race: 0,
            sex: 0,
            class: 0,
            implement: 0,
            pt_values: ScriptHash::default(),
            start_area: 0,
        }
    }
}

impl CharGenResult {
    /// `gmCharGenResult::Clear` (the start position and slot stay).
    pub fn clear(&mut self) {
        self.start_area = 0;
        self.entity_did = 0;
        self.name.clear();
        self.race = 0;
        self.sex = 0;
        self.class = 0;
        self.implement = 0;
        self.pt_values.flush();
    }

    /// `gmCharGenResult::operator=` (and the copy constructor): everything but the start
    /// position and slot.
    pub fn assign(&mut self, rhs: &CharGenResult) {
        self.start_area = rhs.start_area;
        self.entity_did = rhs.entity_did;
        self.name = rhs.name.clone();
        self.race = rhs.race;
        self.sex = rhs.sex;
        self.class = rhs.class;
        self.implement = rhs.implement;
        self.pt_values = rhs.pt_values.clone();
    }

    /// `gmCharGenResult::GetPhysTypeValue`: -1 when unset.
    pub fn phys_type_value(&self, phys_type: u32) -> f32 {
        self.pt_values
            .find(&RefKey::Value(u64::from(phys_type)), &PlainKeys)
            .map_or(-1.0, |bits| f32::from_bits(bits as u32))
    }

    /// `gmCharGenResult::SetPhysTypeValue` (`HashTable::set`).
    pub fn set_phys_type_value(&mut self, phys_type: u32, value: f32) {
        self.pt_values.set(
            RefKey::Value(u64::from(phys_type)),
            u64::from(value.to_bits()),
            &PlainKeys,
        );
    }

    /// The values in table order (`IntrusiveHashTable` iteration).
    pub fn phys_type_values(&self) -> Vec<(u32, f32)> {
        self.pt_values
            .nodes()
            .filter_map(|n| match n.key {
                RefKey::Value(k) => Some((k as u32, f32::from_bits(n.value as u32))),
                RefKey::Ref { .. } => None,
            })
            .collect()
    }
}
