//! `CBaseQualities::UnPack` (admin 0x005192F0, Legions 0x005074A0, instruction-identical): the
//! player description `ECM_Login::RecvEvt_PlayerDesc` hands to `CWInterface::RecvEvt_PlayerDesc`.

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::string_info::StringInfo;
use crate::protocol::types::{hash_count, Position};
use crate::protocol::weenie_desc::WeenieDesc;
use crate::protocol::ProtocolError;

/// A weenie's qualities: a pack header naming the tables present, the `DBObj` DID
/// (`DBObj::UnPack_Type`; type `0x40000019` is not categorized), then each table
/// (`PHashTable<unsigned long, T>::StreamPack`: count, then key and value pairs).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Qualities {
    pub header: u32,
    pub did: u32,
    /// Header bit 1: `gmWeenieDesc` (`gmCWAllocator::AllocateWeenieDesc_Internal`).
    pub weenie: Option<WeenieDesc>,
    /// Bit 2: `m_int_table`.
    pub ints: Vec<(u32, i32)>,
    /// Bit 4: `m_pbool_table`.
    pub bools: Vec<(u32, i32)>,
    /// Bit 8: `m_float_table`.
    pub floats: Vec<(u32, f32)>,
    /// Bit `0x10`: `m_ts_table` (timestamps).
    pub timestamps: Vec<(u32, f64)>,
    /// Bit `0x80`: `m_str_table`.
    pub strings: Vec<(u32, Vec<u8>)>,
    /// Bit `0x20`: `m_did_table`.
    pub dids: Vec<(u32, u32)>,
    /// Bit `0x40`: `m_iid_table`.
    pub iids: Vec<(u32, u64)>,
    /// Bit `0x100`: `m_pos_table`.
    pub positions: Vec<(u32, Position)>,
    /// Bit `0x200`: `m_si_table`.
    pub string_infos: Vec<(u32, StringInfo)>,
    /// Bit `0x400`: `m_pid_table` (package ids).
    pub package_ids: Vec<(u32, u32)>,
    /// Bit `0x800`: `m_lint_table`.
    pub long_ints: Vec<(u32, i64)>,
}

/// `PHashTable::StreamPack` on the write side: the count with a bucket-size index of 0 in its
/// high byte, then each key and value. The retail index and table order come from its
/// `IntrusiveHashTable`, which is not kept here, so re-packed tables are in wire order.
fn write_table<T>(w: &mut Writer, pairs: &[(u32, T)], mut value: impl FnMut(&T, &mut Writer)) {
    // `CBaseQualities::Pack` writes a table only when its header bit is set (it has entries).
    if pairs.is_empty() {
        return;
    }
    w.u32(pairs.len() as u32);
    for (key, item) in pairs {
        w.u32(*key);
        value(item, w);
    }
}

fn table<T>(
    r: &mut Reader<'_>,
    present: bool,
    mut value: impl FnMut(&mut Reader<'_>) -> Result<T, ProtocolError>,
) -> Result<Vec<(u32, T)>, ProtocolError> {
    if !present {
        return Ok(Vec::new());
    }
    (0..hash_count(r)?)
        .map(|_| Ok((r.u32()?, value(r)?)))
        .collect()
}

impl Qualities {
    /// `CBaseQualities::GetPackHeader`: bit 1 when a weenie description is present and one bit
    /// per table that has entries, as [`Qualities::read`] tests them.
    pub fn pack_header(&self) -> u32 {
        let bits = [
            (self.weenie.is_some(), 0x1),
            (!self.ints.is_empty(), 0x2),
            (!self.bools.is_empty(), 0x4),
            (!self.floats.is_empty(), 0x8),
            (!self.timestamps.is_empty(), 0x10),
            (!self.dids.is_empty(), 0x20),
            (!self.iids.is_empty(), 0x40),
            (!self.strings.is_empty(), 0x80),
            (!self.positions.is_empty(), 0x100),
            (!self.string_infos.is_empty(), 0x200),
            (!self.package_ids.is_empty(), 0x400),
            (!self.long_ints.is_empty(), 0x800),
        ];
        bits.iter()
            .filter(|(present, _)| *present)
            .fold(0, |header, (_, bit)| header | bit)
    }

    /// `CBaseQualities::Pack`: the pack header, the DID, the weenie description and then each
    /// table in `UnPack`'s order. Only the bucket index of each table is not reproduced (see
    /// `write_table`), so the bytes match retail's when those indices are 0.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.pack_header()).u32(self.did);
        if let Some(weenie) = &self.weenie {
            w.bytes.extend(weenie.to_bytes());
        }
        write_table(&mut w, &self.ints, |v, w| {
            w.u32(*v as u32);
        });
        write_table(&mut w, &self.bools, |v, w| {
            w.u32(*v as u32);
        });
        write_table(&mut w, &self.floats, |v, w| {
            w.f32(*v);
        });
        write_table(&mut w, &self.timestamps, |v, w| {
            w.f64(*v);
        });
        write_table(&mut w, &self.strings, |v, w| {
            w.string(v);
        });
        write_table(&mut w, &self.dids, |v, w| {
            w.u32(*v);
        });
        write_table(&mut w, &self.iids, |v, w| {
            w.u64(*v);
        });
        write_table(&mut w, &self.positions, |v, w| v.write(w));
        write_table(&mut w, &self.string_infos, |v, w| v.write(w));
        write_table(&mut w, &self.package_ids, |v, w| {
            w.u32(*v);
        });
        write_table(&mut w, &self.long_ints, |v, w| {
            w.u64(*v as u64);
        });
        w.bytes
    }

    /// A whole qualities record: a DAT `DB_TYPE_QUALITIES` record's packed bytes read as
    /// `CBaseQualities::UnPack` reads them (the class defaults, `CBaseQualities::InitDefaults`).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut r = Reader::new(bytes);
        Self::read(&mut r)
    }

    /// Tables are read in `UnPack`'s order (strings before DIDs). Bytes after the last table
    /// are not read (`PackObj::VERIFY_ADDR` only checks the end was not passed).
    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let header = r.u32()?;
        let has = |bit: u32| header & bit != 0;
        let did = r.u32()?;
        let weenie = has(1).then(|| WeenieDesc::read(r)).transpose()?;
        let ints = table(r, has(2), |r| r.u32().map(|v| v as i32))?;
        let bools = table(r, has(4), |r| r.u32().map(|v| v as i32))?;
        let floats = table(r, has(8), |r| r.f32())?;
        let timestamps = table(r, has(0x10), |r| r.f64())?;
        let strings = table(r, has(0x80), |r| r.string())?;
        let dids = table(r, has(0x20), |r| r.u32())?;
        let iids = table(r, has(0x40), |r| r.u64())?;
        let positions = table(r, has(0x100), Position::read)?;
        let string_infos = table(r, has(0x200), StringInfo::read)?;
        let package_ids = table(r, has(0x400), |r| r.u32())?;
        let long_ints = table(r, has(0x800), |r| r.u64().map(|v| v as i64))?;
        r.rest();
        Ok(Self {
            header,
            did,
            weenie,
            ints,
            bools,
            floats,
            timestamps,
            strings,
            dids,
            iids,
            positions,
            string_infos,
            package_ids,
            long_ints,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Qualities;
    use crate::pack_string;
    use crate::protocol::reader::Reader;

    fn put(out: &mut Vec<u8>, words: &[u32]) {
        out.extend(words.iter().flat_map(|w| w.to_le_bytes()));
    }

    #[test]
    fn pack_writes_the_header_did_and_tables_in_unpack_order_with_exact_bytes() {
        let q = Qualities {
            did: 0x4000_0001,
            ints: vec![(0x19, -3)],
            dids: vec![(0x1A, 0x0600_1234)],
            ..Qualities::default()
        };
        assert_eq!(q.pack_header(), 0x22);
        let expected = [
            0x22, 0, 0, 0, // header: ints and dids
            1, 0, 0, 0x40, // did 0x40000001
            1, 0, 0, 0, 0x19, 0, 0, 0, 0xFD, 0xFF, 0xFF, 0xFF, // ints: count, key 0x19, -3
            1, 0, 0, 0, 0x1A, 0, 0, 0, 0x34, 0x12, 0, 0x06, // dids: count, key 0x1A, value
        ];
        assert_eq!(q.to_bytes(), expected);
        assert_eq!(
            Qualities::from_bytes(&expected).unwrap().to_bytes(),
            expected
        );
    }

    #[test]
    fn an_empty_table_is_not_in_the_header() {
        let q = Qualities {
            ints: Vec::new(),
            strings: vec![(1, b"x".to_vec())],
            ..Qualities::default()
        };
        assert_eq!(q.pack_header(), 0x80);
        let bytes = q.to_bytes();
        let back = Qualities::from_bytes(&bytes).unwrap();
        assert_eq!(back.strings, q.strings);
        assert!(back.ints.is_empty());
        assert_eq!(back.to_bytes(), bytes);
    }

    #[test]
    fn tables_follow_the_header_in_unpack_order() {
        let mut b = Vec::new();
        put(&mut b, &[0x2 | 0x10 | 0x20 | 0x80 | 0x800, 0x4000_0001]);
        put(&mut b, &[1, 0x19, (-3i32) as u32]); // ints
        put(&mut b, &[1, 0x05]); // timestamps
        b.extend(1234.5f64.to_le_bytes());
        put(&mut b, &[1, 0x01]); // strings come before DIDs
        pack_string(&mut b, 0, b"Explorer");
        put(&mut b, &[1 | (3 << 24), 0x1A, 0x0600_1234]); // DIDs
        put(&mut b, &[1, 0x02]); // long ints
        b.extend((-9i64).to_le_bytes());
        b.extend([0xEE; 3]); // ignored trailing bytes
        let mut r = Reader::new(&b);
        let q = Qualities::read(&mut r).unwrap();
        r.finish().unwrap();
        assert_eq!(q.did, 0x4000_0001);
        assert_eq!(q.ints, [(0x19, -3)]);
        assert_eq!(q.timestamps, [(0x05, 1234.5)]);
        assert_eq!(q.strings, [(0x01, b"Explorer".to_vec())]);
        assert_eq!(q.dids, [(0x1A, 0x0600_1234)]);
        assert_eq!(q.long_ints, [(0x02, -9)]);
        assert!(q.weenie.is_none() && q.floats.is_empty());
        for end in 0..b.len() - 3 {
            assert!(Qualities::read(&mut Reader::new(&b[..end])).is_err());
        }
    }
}
