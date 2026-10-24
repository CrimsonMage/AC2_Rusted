//! Data-download events on queue 5 (`clidatevents.cpp`, `CLCache::UseTime`).

use crate::protocol::opcode::Opcode;
use crate::protocol::reader::{Reader, Writer};
use crate::protocol::ProtocolError;

/// Ids accepted in one iteration set (`CMostlyConsecutiveIntSet::StreamPack`: `0x186a0`).
const MAX_SET: usize = 0x186A0;

/// `CMostlyConsecutiveIntSet::StreamPack`: the id count, then entries; a run of three or more
/// consecutive ids is its negated length and first id, any other id is itself (bit 30 set
/// means negative).
pub(crate) fn read_int_set_inner(reader: &mut Reader<'_>) -> Result<Vec<i32>, ProtocolError> {
    let total = reader.u32()? as usize;
    if total > MAX_SET {
        return Err(ProtocolError::Invalid("iteration set"));
    }
    let mut out = Vec::with_capacity(total);
    while out.len() < total {
        let entry = reader.u32()? as i32;
        if entry < 0 {
            let run = entry.unsigned_abs() as usize;
            let mut base = reader.u32()? as i32;
            for _ in 0..run {
                if out.len() >= total {
                    return Err(ProtocolError::Invalid("iteration run"));
                }
                out.push(base);
                base = base.wrapping_add(1);
            }
        } else {
            let mut value = entry as u32;
            if value & 0x4000_0000 != 0 {
                value |= 0x8000_0000;
            }
            out.push(value as i32);
        }
    }
    Ok(out)
}

/// Decodes a packed iteration set from `bytes`.
pub fn read_int_set(bytes: &[u8]) -> Result<Vec<i32>, ProtocolError> {
    read_int_set_inner(&mut Reader::new(bytes))
}

/// The packing direction of [`read_int_set`] (ids sorted first, as `Sort` does).
pub fn write_int_set(writer: &mut Writer, ids: &[i32]) {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    writer.u32(ids.len() as u32);
    let mut i = 0;
    while i < ids.len() {
        let mut end = i;
        let mut next = ids[i];
        while end < ids.len() && ids[end] == next {
            end += 1;
            next = next.wrapping_add(1);
        }
        let run = end - i;
        if run > 2 {
            writer.u32((-(run as i32)) as u32).u32(ids[i] as u32);
            i = end;
        } else {
            writer.u32(ids[i] as u32 & 0x7FFF_FFFF);
            i += 1;
        }
    }
}

/// `CMissingIteration::StreamPack`: the DAT file, its iteration, ids to fetch and ids to purge.
#[derive(Debug, Clone, PartialEq)]
pub struct MissingIteration {
    pub dat_file: u64,
    pub iteration: u32,
    pub download: Vec<u32>,
    pub purge: Vec<u32>,
}

fn ids(reader: &mut Reader<'_>) -> Result<Vec<u32>, ProtocolError> {
    (0..reader.count()?).map(|_| reader.u32()).collect()
}

/// A queue-5 event from the server.
#[derive(Debug, Clone, PartialEq)]
pub enum DatEvent {
    /// `0x10035`: a data object (`CCliDatDataEvent`), kept as its body.
    Data(Vec<u8>),
    /// `0x10036`: the request for this data type and id failed.
    RequestFailed { kind: u32, id: u32 },
    /// `0x10037`: the server asks for the client's iterations (`OnServerInterrogation`).
    Interrogation(Vec<u8>),
    /// `0x10039`: bytes still to patch.
    PatchtimePending(u32),
    /// `0x1003A`: total bytes, then the missing iterations (`OnBeginDDD`).
    BeginDdd {
        bytes: u32,
        missing: Vec<MissingIteration>,
    },
    /// `0x1003B`: patching is over (`OnEndDDD`).
    EndDdd,
}

impl DatEvent {
    pub(crate) fn read(opcode: u32, r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(match opcode {
            Opcode::DAT_DATA => Self::Data(r.rest().to_vec()),
            Opcode::DAT_REQUEST_FAILED => Self::RequestFailed {
                kind: r.u32()?,
                id: r.u32()?,
            },
            Opcode::DAT_INTERROGATION => Self::Interrogation(r.rest().to_vec()),
            Opcode::DAT_PATCHTIME_PENDING => Self::PatchtimePending(r.u32()?),
            Opcode::BEGIN_DDD => {
                let bytes = r.u32()?;
                let missing = (0..r.count()?)
                    .map(|_| {
                        Ok(MissingIteration {
                            dat_file: r.u64()?,
                            iteration: r.u32()?,
                            download: ids(r)?,
                            purge: ids(r)?,
                        })
                    })
                    .collect::<Result<_, ProtocolError>>()?;
                Self::BeginDdd { bytes, missing }
            }
            _ => Self::EndDdd,
        })
    }
}

/// `CCliDatInterrogationResponseEvent::CreateData`: language, then `CAllIterationList` (a count
/// of DAT files, each its id and iteration set).
pub fn interrogation_response(language: u32, lists: &[(u64, Vec<i32>)]) -> Vec<u8> {
    let mut w = Writer::event(Opcode::DAT_INTERROGATION_RESPONSE);
    w.u32(language).u32(lists.len() as u32);
    for (dat_file, iterations) in lists {
        w.u64(*dat_file);
        write_int_set(&mut w, iterations);
    }
    w.bytes
}

/// `CCliDatRequestEvent::CreateForSend`: data type and id.
pub fn dat_request(kind: u32, id: u32) -> Vec<u8> {
    let mut w = Writer::event(Opcode::DAT_REQUEST);
    w.u32(kind).u32(id);
    w.bytes
}

/// `CCliDatEndDDDEvent::CreateForSend`.
pub fn end_ddd() -> Vec<u8> {
    Opcode::END_DDD.to_le_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::{dat_request, end_ddd, read_int_set_inner, write_int_set};
    use crate::protocol::reader::{Reader, Writer};

    #[test]
    fn runs_of_three_or_more_pack_as_negative_lengths() {
        let mut w = Writer { bytes: Vec::new() };
        write_int_set(&mut w, &[7, 1, 2, 3, 4, 9, 10]);
        let words: Vec<i32> = w
            .bytes
            .chunks(4)
            .map(|c| i32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(words, [7, -4, 1, 7, 9, 10]);
        let mut r = Reader::new(&w.bytes);
        assert_eq!(read_int_set_inner(&mut r).unwrap(), [1, 2, 3, 4, 7, 9, 10]);
    }

    #[test]
    fn client_events_have_their_retail_bytes() {
        assert_eq!(end_ddd(), [0x3B, 0, 1, 0]);
        assert_eq!(
            dat_request(0x4000_0021, 0x1F00_0001),
            [0x34, 0, 1, 0, 0x21, 0, 0, 0x40, 1, 0, 0, 0x1F]
        );
    }
}
