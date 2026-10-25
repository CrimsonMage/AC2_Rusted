//! `gmCharGenResult::Pack` / `UnPack` (identical in Legions, 0x005C1E20 / 0x005C1F60): the
//! character the creation screens built, as `Proto_UI::SendCharGenResult` sends it.

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::types::hash_count;
use crate::protocol::ProtocolError;

/// `g_bucketSizes` (Legions 0x009ECCC0): the bucket counts `IntrusiveHashTable` picks from.
const BUCKET_SIZES: [u32; 23] = [
    11, 23, 47, 89, 191, 383, 761, 1531, 3067, 6143, 12281, 24571, 49139, 98299, 196597, 393209,
    786431, 1572853, 3145721, 6291449, 12582893, 25165813, 50331599,
];
/// `gmCharGenResult::gmCharGenResult` asks for 0x20 buckets: `lower_bound` gives 47.
const PT_BUCKET_INDEX: usize = 2;

/// The created character.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CharGenResult {
    pub name: String,
    /// `m_entityDID`: the chosen appearance entity.
    pub entity_did: u32,
    pub start_area: u32,
    pub race: u32,
    pub sex: u32,
    pub class: u32,
    pub implement: u32,
    /// `m_ptValues`: physical-type (body slider) values, in the order the screens set them.
    pub physical: Vec<(u32, f32)>,
}

impl CharGenResult {
    /// `m_ptValues` in `IntrusiveHashTable` order: bucket `key % 47` ascending, each chain from
    /// its head (`HashTable::add` links new nodes there; a key set again keeps its place).
    fn physical_in_table_order(&self) -> Vec<(u32, f32)> {
        let buckets = BUCKET_SIZES[PT_BUCKET_INDEX];
        let mut chains: Vec<Vec<(u32, f32)>> = vec![Vec::new(); buckets as usize];
        for &(key, value) in &self.physical {
            let chain = &mut chains[(key % buckets) as usize];
            match chain.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = value,
                None => chain.insert(0, (key, value)),
            }
        }
        chains.into_iter().flatten().collect()
    }

    /// `gmCharGenResult::Pack`: the name, six words, then `PHashTable<unsigned long, float>`
    /// (`count | bucket index << 24`, then key and value pairs).
    pub(crate) fn write(&self, w: &mut Writer) {
        w.wide_string(&self.name);
        w.u32(self.entity_did)
            .u32(self.start_area)
            .u32(self.race)
            .u32(self.sex)
            .u32(self.class)
            .u32(self.implement);
        let physical = self.physical_in_table_order();
        w.u32(physical.len() as u32 | (PT_BUCKET_INDEX as u32) << 24);
        for (key, value) in physical {
            w.u32(key).f32(value);
        }
    }

    /// `gmCharGenResult::UnPack` (the pairs in packed order).
    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let name = r.wide_string()?;
        let mut result = Self {
            name,
            entity_did: r.u32()?,
            start_area: r.u32()?,
            race: r.u32()?,
            sex: r.u32()?,
            class: r.u32()?,
            implement: r.u32()?,
            physical: Vec::new(),
        };
        for _ in 0..hash_count(r)? {
            result.physical.push((r.u32()?, r.f32()?));
        }
        Ok(result)
    }
}

/// The server's side of `Proto_UI::SendCharGenResult`: the account and the result.
pub fn decode_char_gen_result(bytes: &[u8]) -> Result<(String, CharGenResult), ProtocolError> {
    let mut r = Reader::new(bytes);
    if r.u32()? != crate::protocol::Opcode::CHAR_GEN_RESULT {
        return Err(ProtocolError::Invalid("not a character generation result"));
    }
    let account = String::from_utf8(r.string()?).map_err(|_| ProtocolError::Invalid("account"))?;
    let result = CharGenResult::read(&mut r)?;
    r.finish()?;
    Ok((account, result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_result_packs_its_sliders_in_table_order() {
        let result = CharGenResult {
            name: "Ab".into(),
            entity_did: 0x7000_0001,
            start_area: 2,
            race: 3,
            sex: 4,
            class: 5,
            implement: 6,
            // 48 and 1 share bucket 1: the later-added 48 comes first.
            physical: vec![(5, 0.5), (1, 0.25), (48, 1.0)],
        };
        let mut w = Writer::new();
        result.write(&mut w);
        let mut expected = Writer::new();
        expected.wide_string("Ab");
        for v in [0x7000_0001, 2, 3, 4, 5, 6, 3 | 2 << 24, 48] {
            expected.u32(v);
        }
        expected.f32(1.0).u32(1).f32(0.25).u32(5).f32(0.5);
        assert_eq!(w.bytes, expected.bytes);
        let mut r = Reader::new(&w.bytes);
        let back = CharGenResult::read(&mut r).unwrap();
        r.finish().unwrap();
        assert_eq!(back.physical, vec![(48, 1.0), (1, 0.25), (5, 0.5)]);
        assert_eq!(back.name, "Ab");
    }
}
