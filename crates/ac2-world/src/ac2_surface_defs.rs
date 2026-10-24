use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_dat::DatFileKind;
use ac2_core::{DataId, QualifiedDataId};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

const AC2_SURFACE_DEFS_DID: DataId = DataId::new(0x1300_0000);
const AC2_MATERIAL_PREFIX: u32 = 0x2B00_0000;
const AC2_FALLBACK_MATERIAL_DID: DataId = DataId::new(0x2B00_0007);
const ENTRY_COUNT_OFFSET: usize = 0x24;
const FIRST_ENTRY_OFFSET: usize = 0x28;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Ac2SlopeBand {
    min_slope: f32,
    max_slope: f32,
    material: DataId,
}

/// Static mapping from AC2 land terrain indices to authored slope-dependent surfaces.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ac2SurfaceDefs {
    bands_by_terrain: BTreeMap<u8, Arc<[Ac2SlopeBand]>>,
}

impl Ac2SurfaceDefs {
    pub(crate) fn material_for_slope(
        &self,
        terrain_type: u8,
        slope_squared: f32,
    ) -> Option<DataId> {
        let bands = self.bands_by_terrain.get(&terrain_type)?;
        bands
            .iter()
            .find(|band| {
                band.min_slope * band.min_slope <= slope_squared
                    && slope_squared < band.max_slope * band.max_slope
            })
            .or_else(|| bands.last())
            .map(|band| band.material)
    }
}

impl ResourcePackedDecode for Ac2SurfaceDefs {
    const TYPE_NAME: &'static str = "AC2 SurfaceDefs";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode_ac2_surface_defs(qdid, bytes)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(
            self.bands_by_terrain
                .values()
                .map(|bands| bands.len() * std::mem::size_of::<Ac2SlopeBand>())
                .sum::<usize>(),
        )
    }
}

pub(crate) fn load_ac2_surface_defs(
    store: &ResourceStore,
) -> ResourceResult<Option<CachedResource<Ac2SurfaceDefs>>> {
    store.load_cached_packed::<Ac2SurfaceDefs>(TypedResourceRef::new(
        DatFileKind::Portal,
        AC2_SURFACE_DEFS_DID,
        0,
    ))
}

fn decode_ac2_surface_defs(qdid: QualifiedDataId, bytes: &[u8]) -> ResourceResult<Ac2SurfaceDefs> {
    if qdid.data_id() != AC2_SURFACE_DEFS_DID || bytes.len() < FIRST_ENTRY_OFFSET + 8 {
        return Err(malformed(qdid, "invalid AC2 SurfaceDefs envelope"));
    }
    let declared_count = read_u32(bytes, ENTRY_COUNT_OFFSET)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| malformed(qdid, "missing AC2 SurfaceDefs entry count"))?;
    let mut entries = Vec::<(usize, u8, usize)>::new();
    for offset in (FIRST_ENTRY_OFFSET..bytes.len().saturating_sub(4)).step_by(4) {
        let Some(key) = read_u32(bytes, offset) else {
            break;
        };
        let Some(choice_count) = read_u32(bytes, offset + 4) else {
            break;
        };
        if key & 0xFFFF_FF00 == 0x8000_0000 && (1..=8).contains(&choice_count) {
            entries.push((offset, key as u8, choice_count as usize));
        }
    }
    if entries.len() != declared_count {
        return Err(malformed(
            qdid,
            "AC2 SurfaceDefs entry count did not match payload",
        ));
    }
    let mut bands_by_terrain = BTreeMap::new();
    for (index, &(start, terrain_type, choice_count)) in entries.iter().enumerate() {
        let end = entries.get(index + 1).map_or(bytes.len(), |entry| entry.0);
        let candidates = slope_band_offsets(bytes, start + 8, end);
        if candidates.len() < choice_count {
            return Err(malformed(
                qdid,
                "AC2 SurfaceDefs slope-band count was incomplete",
            ));
        }
        let mut bands = Vec::with_capacity(choice_count);
        for band_index in 0..choice_count {
            let band_start = candidates[band_index];
            let band_end = candidates.get(band_index + 1).copied().unwrap_or(end);
            let min_slope = read_f32(bytes, band_start)
                .ok_or_else(|| malformed(qdid, "missing AC2 minimum terrain slope"))?;
            let max_slope = read_f32(bytes, band_start + 4)
                .ok_or_else(|| malformed(qdid, "missing AC2 maximum terrain slope"))?;
            let material = bytes[band_start..band_end]
                .chunks_exact(4)
                .map(|word| DataId::new(u32::from_le_bytes(word.try_into().expect("four bytes"))))
                .find(|did| {
                    did.raw() & 0xFF00_0000 == AC2_MATERIAL_PREFIX
                        && *did != AC2_FALLBACK_MATERIAL_DID
                })
                .ok_or_else(|| malformed(qdid, "AC2 slope band had no terrain material"))?;
            bands.push(Ac2SlopeBand {
                min_slope,
                max_slope,
                material,
            });
        }
        if !bands.is_empty() {
            bands_by_terrain.insert(terrain_type, Arc::from(bands));
        }
    }
    if bands_by_terrain.is_empty() {
        return Err(malformed(
            qdid,
            "AC2 SurfaceDefs contained no terrain slope bands",
        ));
    }
    Ok(Ac2SurfaceDefs { bands_by_terrain })
}

fn slope_band_offsets(bytes: &[u8], start: usize, end: usize) -> Vec<usize> {
    let mut offsets = Vec::new();
    for offset in (start..end.saturating_sub(24)).step_by(4) {
        let Some(min_slope) = read_f32(bytes, offset) else {
            continue;
        };
        let Some(max_slope) = read_f32(bytes, offset + 4) else {
            continue;
        };
        let Some(texture_scale) = read_f32(bytes, offset + 8) else {
            continue;
        };
        let fixed_fields_match = read_u32(bytes, offset + 12) == Some(1)
            && read_u32(bytes, offset + 16) == Some(1)
            && read_u32(bytes, offset + 20).is_some_and(|flags| flags >> 24 == 0x37);
        if min_slope >= 0.0
            && min_slope < max_slope
            && max_slope <= 1.000_01
            && (1.0..=64.0).contains(&texture_scale)
            && fixed_fields_match
        {
            offsets.push(offset);
        }
    }
    offsets
}

fn read_f32(bytes: &[u8], offset: usize) -> Option<f32> {
    Some(f32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn malformed(qdid: QualifiedDataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name: Ac2SurfaceDefs::TYPE_NAME,
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_ac2_surface_defs, AC2_SURFACE_DEFS_DID};
    use ac2_core::QualifiedDataId;

    #[test]
    fn selects_material_from_the_retail_slope_band() {
        let mut bytes = vec![0_u8; 0x28];
        bytes[0x24..0x28].copy_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&0x8000_000F_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        for (min, max, material) in [
            (0.0_f32, 0.5_f32, 0x2B00_004A_u32),
            (0.5, 1.0, 0x2B00_003B_u32),
        ] {
            bytes.extend_from_slice(&min.to_le_bytes());
            bytes.extend_from_slice(&max.to_le_bytes());
            bytes.extend_from_slice(&20.0_f32.to_le_bytes());
            bytes.extend_from_slice(&1_u32.to_le_bytes());
            bytes.extend_from_slice(&1_u32.to_le_bytes());
            bytes.extend_from_slice(&0x3700_0000_u32.to_le_bytes());
            bytes.extend_from_slice(&material.to_le_bytes());
        }
        let defs =
            decode_ac2_surface_defs(QualifiedDataId::new(AC2_SURFACE_DEFS_DID, 0), &bytes).unwrap();
        assert_eq!(
            defs.material_for_slope(0x0F, 0.04).unwrap().raw(),
            0x2B00_004A
        );
        assert_eq!(
            defs.material_for_slope(0x0F, 0.64).unwrap().raw(),
            0x2B00_003B
        );
    }
}
