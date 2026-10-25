//! Asheron's Call 2 `CLightInfo` (`xxyyFFFC` in the cell DAT): a landblock's static lights and,
//! per light, the cells and vertex indices it reaches, decoded in `CLightInfo::StreamPack` order.
//!
//! Each light: owning cell id, `PropertyCollection` (`LightSource::ApplyProperty`), origin, then counted cells (`StreamPackCell`: cell id, index
//! count, RLE byte count, RLE words, 4-byte align; `RLE::Decode`). The record ends with the
//! cell -> lights lookup hash and the optional nine sunlight cell sets.

use std::sync::Arc;

use ac2_core::{CellId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::read_ac2_property_collection;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::Vec3;

const MAX_ITEMS: usize = 1 << 16;
const SUNLIGHT_SETS: usize = 9;

const PROPERTY_TYPE: u32 = 0x4000_0002;
const PROPERTY_DIRECTION: u32 = 0x4000_0003;
const PROPERTY_RANGE: u32 = 0x4000_0004;
const PROPERTY_DIFFUSE: u32 = 0x4000_0008;
const PROPERTY_INTENSITY: u32 = 0x4000_0009;

const LIGHT_POINT: u32 = 0x4000_0001;
const LIGHT_DIRECTIONAL: u32 = 0x4000_0002;

/// `LightSource::m_Type`. Sunlight-linked lights take the sun's colour at run time
/// and add nothing to the static bake (`CEnvCell::LightIndices`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ac2LightKind {
    Point,
    Directional,
    Sunlight,
}

/// One static light of a landblock, in landblock space (AC axes).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2StaticLight {
    pub cell: CellId,
    pub kind: Ac2LightKind,
    pub origin: Vec3,
    pub direction: Vec3,
    /// `m_wRange` and `m_wDiffuseIntensity` base values (`LightSource` defaults 5 and 0.5).
    pub range: f32,
    pub intensity: f32,
    /// `m_cDiffuse` as linear RGB in 0..1.
    pub color: [f32; 3],
    /// Per lit cell, the vertex indices of its cell mesh this light reaches.
    pub lit: Box<[(CellId, Box<[u16]>)]>,
}

impl Ac2StaticLight {
    /// `LightSource::CalculateAttenuation`: `intensity / (10 d^2 / range^2 + 1)` for point lights.
    pub fn attenuation(&self, distance: f32) -> f32 {
        match self.kind {
            Ac2LightKind::Directional => self.intensity,
            _ => {
                let range = self.range.max(0.0002);
                self.intensity / ((distance / (range * range)) * distance * 10.0 + 1.0)
            }
        }
    }

    /// The vertex indices this light reaches in `cell`, if any.
    pub fn lit_indices(&self, cell: CellId) -> Option<&[u16]> {
        self.lit
            .iter()
            .find(|(lit_cell, _)| lit_cell.raw() & 0xFFFF == cell.raw() & 0xFFFF)
            .map(|(_, indices)| &indices[..])
    }
}

/// The decoded static lights of one AC2 landblock.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2LightInfo {
    pub lights: Arc<[Ac2StaticLight]>,
}

impl ResourcePackedDecode for Ac2LightInfo {
    const TYPE_NAME: &'static str = "AC2 LightInfo";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        let info = decode(&mut cursor, qdid.data_id().raw()).map_err(malformed)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after LightInfo"));
        }
        Ok(info)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .lights
                .iter()
                .map(|light| {
                    std::mem::size_of::<Ac2StaticLight>()
                        + light
                            .lit
                            .iter()
                            .map(|(_, indices)| 8 + indices.len() * 2)
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}

type Read<T> = Result<T, &'static str>;

fn word(cursor: &mut ArchiveCursor<'_>) -> Read<u32> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn count(cursor: &mut ArchiveCursor<'_>, min_item_bytes: usize) -> Read<usize> {
    let n = word(cursor)? as usize;
    if n > MAX_ITEMS || n * min_item_bytes > cursor.remaining() {
        return Err("item count exceeded the record");
    }
    Ok(n)
}

fn decode(cursor: &mut ArchiveCursor<'_>, did: u32) -> Read<Ac2LightInfo> {
    if word(cursor)? != did {
        return Err("embedded DID did not match DAT identity");
    }
    let block = did & 0xFFFF_0000;
    let light_count = count(cursor, 20)?;
    let mut lights = Vec::with_capacity(light_count);
    for _ in 0..light_count {
        let cell = CellId::new(word(cursor)?);
        let mut light = Ac2StaticLight {
            cell,
            kind: Ac2LightKind::Point,
            origin: Vec3::ZERO,
            direction: Vec3::ZERO,
            range: 5.0,
            intensity: 0.5,
            color: [1.0; 3],
            lit: Box::default(),
        };
        for property in read_ac2_property_collection(cursor)? {
            match property.name {
                PROPERTY_TYPE => {
                    light.kind = match property.word() {
                        Some(LIGHT_POINT) => Ac2LightKind::Point,
                        Some(LIGHT_DIRECTIONAL) => Ac2LightKind::Directional,
                        _ => Ac2LightKind::Sunlight,
                    }
                }
                PROPERTY_DIRECTION => {
                    if let (Some(x), Some(y), Some(z)) = (
                        property.float_at(0),
                        property.float_at(1),
                        property.float_at(2),
                    ) {
                        light.direction = Vec3::new(x, y, z);
                    }
                }
                // `Waveform`: kind, then base value first.
                PROPERTY_RANGE => light.range = property.float_at(1).unwrap_or(light.range),
                PROPERTY_INTENSITY => {
                    light.intensity = property.float_at(1).unwrap_or(light.intensity)
                }
                PROPERTY_DIFFUSE => {
                    if let Some(argb) = property.word() {
                        let channel = |shift: u32| ((argb >> shift) & 0xFF) as f32 / 255.0;
                        light.color = [channel(16), channel(8), channel(0)];
                    }
                }
                _ => {}
            }
        }
        let f =
            |cursor: &mut ArchiveCursor<'_>| cursor.read_f32().map_err(|_| "record ended early");
        light.origin = Vec3::new(f(cursor)?, f(cursor)?, f(cursor)?);
        light.lit = read_cells(cursor, block)?.into_boxed_slice();
        lights.push(light);
    }
    // Cell -> light lookup hash: entry count in the low half, then (cell, counted light keys).
    let entries = (word(cursor)? & 0xFFFF) as usize;
    for _ in 0..entries {
        let _cell = word(cursor)?;
        let keys = count(cursor, 4)?;
        cursor
            .read_bytes(keys * 4)
            .map_err(|_| "record ended early")?;
    }
    if word(cursor)? != 0 {
        for _ in 0..SUNLIGHT_SETS {
            read_cells(cursor, block)?;
        }
    }
    Ok(Ac2LightInfo {
        lights: Arc::from(lights),
    })
}

/// Counted `LightCellInfo`s: cell id and its RLE-packed vertex indices.
fn read_cells(cursor: &mut ArchiveCursor<'_>, block: u32) -> Read<Vec<(CellId, Box<[u16]>)>> {
    let cells = count(cursor, 8)?;
    let mut out = Vec::with_capacity(cells);
    for _ in 0..cells {
        let cell = CellId::new(block | (word(cursor)? & 0xFFFF));
        let indices = word(cursor)? as usize;
        if indices == 0 {
            out.push((cell, Box::default()));
            continue;
        }
        let bytes = word(cursor)? as usize;
        if !bytes.is_multiple_of(2) || bytes > cursor.remaining() {
            return Err("RLE byte count exceeded the record");
        }
        let words = cursor
            .read_u16_items(bytes / 2)
            .map_err(|_| "record ended early")?;
        cursor.align_to(4).map_err(|_| "record ended early")?;
        let decoded = rle_decode(&words, indices)?;
        out.push((cell, decoded.into_boxed_slice()));
    }
    Ok(out)
}

/// `RLE::Decode`: a word with the high bit set starts a run of the following word's length.
fn rle_decode(words: &[u16], expected: usize) -> Read<Vec<u16>> {
    let mut out = Vec::with_capacity(expected);
    let mut index = 0;
    while index < words.len() {
        let value = words[index];
        if value & 0x8000 != 0 {
            let run = words
                .get(index + 1)
                .copied()
                .ok_or("RLE run length missing")? as i16;
            let start = value & 0x7FFF;
            for step in 0..run.max(0) as u16 {
                out.push(start.wrapping_add(step));
            }
            index += 2;
        } else {
            out.push(value);
            index += 1;
        }
        if out.len() > expected {
            return Err("RLE decoded more indices than declared");
        }
    }
    if out.len() != expected {
        return Err("RLE decoded fewer indices than declared");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::rle_decode;

    #[test]
    fn rle_runs_and_literals_decode_in_order() {
        assert_eq!(
            rle_decode(&[4, 0x8000 | 10, 3, 20], 5).unwrap(),
            vec![4, 10, 11, 12, 20]
        );
        assert!(rle_decode(&[1, 2], 3).is_err());
    }
}
