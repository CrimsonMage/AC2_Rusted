use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_db::{
    ResourceBodyDecode, ResourceDbObjBody, ResourceDbObjDecodeContext, ResourceError,
    ResourceResult, ResourceStore, ResourceTypeId, TypedResourceRef,
};

pub(crate) const AC2_LAND_DEFS_DID: DataId = DataId::new(0x3600_0000);
const AC2_HEIGHT_COUNT: usize = 256;
const AC2_BODY_BYTES: usize = 48 + AC2_HEIGHT_COUNT * std::mem::size_of::<f32>();

/// AC2's standalone Portal-DAT world lattice record.
///
/// The retail AC2 loader requests this record as `0x40000044`; its on-disk DBObj envelope still
/// consists of the DID followed directly by this body. We route it through the resources crate's
/// supported type-zero identity so resources continues to validate the DID while world retains
/// ownership of the lattice semantics.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ac2LandDefs {
    logical_cell_size: f32,
    block_cells: u32,
    subdivisions_per_cell: u32,
    max_object_height: f32,
    sky_height: f32,
    road_width: f32,
    /// `terrain`, `scene`, `encounter`, `color`, `road`, `replace` bit lengths of a vertex's
    /// cell-info word, packed low to high in that order (`LandDefs::set_bitlengths`).
    cell_info_bits: [u32; 6],
    height_table: Vec<f32>,
}

impl Ac2LandDefs {
    pub(crate) fn logical_cell_size(&self) -> f32 {
        self.logical_cell_size
    }

    pub(crate) fn block_cells(&self) -> u32 {
        self.block_cells
    }

    pub(crate) fn subdivisions_per_cell(&self) -> u32 {
        self.subdivisions_per_cell
    }

    /// (offset, mask) of cell-info field `index` (0 terrain .. 5 replace).
    pub(crate) fn cell_info_field(&self, index: usize) -> (u32, u32) {
        let offset = self.cell_info_bits[..index].iter().sum::<u32>();
        let bits = self.cell_info_bits[index];
        (
            offset,
            if bits >= 32 {
                u32::MAX
            } else {
                (1 << bits) - 1
            },
        )
    }

    pub(crate) fn height_table(&self) -> &[f32] {
        &self.height_table
    }
}

impl ResourceBodyDecode for Ac2LandDefs {
    const TYPE_NAME: &'static str = "AC2 LandDefs";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_body(
        context: ResourceDbObjDecodeContext,
        body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        decode_ac2_land_defs(context.qdid(), body.bytes())
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.height_table.len() * std::mem::size_of::<f32>())
    }
}

pub(crate) fn load_ac2_land_defs(
    store: &ResourceStore,
) -> ResourceResult<Option<std::sync::Arc<Ac2LandDefs>>> {
    store.load_ephemeral_body::<Ac2LandDefs>(TypedResourceRef::new(
        DatFileKind::Portal,
        AC2_LAND_DEFS_DID,
        0,
    ))
}

fn decode_ac2_land_defs(qdid: QualifiedDataId, bytes: &[u8]) -> ResourceResult<Ac2LandDefs> {
    if bytes.len() != AC2_BODY_BYTES {
        return Err(malformed(
            qdid,
            "AC2 LandDefs body had an unexpected byte length",
        ));
    }
    let mut cursor = ArchiveCursor::new(bytes);
    let logical_cell_size = cursor.read_f32()?;
    let block_cells = cursor.read_u32()?;
    let subdivisions_per_cell = cursor.read_u32()?;
    let max_object_height = cursor.read_f32()?;
    let sky_height = cursor.read_f32()?;
    let road_width = cursor.read_f32()?;
    // `LandscapeDefs::UnPack`: terrain, scene, encounter, color, road and replace bit lengths.
    let mut cell_info_bits = [0_u32; 6];
    for bits in &mut cell_info_bits {
        *bits = cursor.read_u32()?;
    }
    if cell_info_bits.iter().sum::<u32>() > 32 {
        return Err(malformed(qdid, "AC2 LandDefs cell-info bits exceeded 32"));
    }
    if logical_cell_size != 20.0 || block_cells != 8 || subdivisions_per_cell != 2 {
        return Err(malformed(
            qdid,
            "AC2 LandDefs lattice was not the retail 20/8/2 layout",
        ));
    }
    if ![max_object_height, sky_height, road_width]
        .into_iter()
        .all(f32::is_finite)
    {
        return Err(malformed(
            qdid,
            "AC2 LandDefs contained a non-finite physical limit",
        ));
    }
    let mut height_table = Vec::with_capacity(AC2_HEIGHT_COUNT);
    for _ in 0..AC2_HEIGHT_COUNT {
        let value = cursor.read_f32()?;
        if !value.is_finite() {
            return Err(malformed(
                qdid,
                "AC2 LandDefs contained a non-finite height",
            ));
        }
        height_table.push(value);
    }
    if cursor.remaining() != 0 {
        return Err(malformed(qdid, "AC2 LandDefs decode left trailing bytes"));
    }
    Ok(Ac2LandDefs {
        logical_cell_size,
        block_cells,
        subdivisions_per_cell,
        max_object_height,
        sky_height,
        road_width,
        cell_info_bits,
        height_table,
    })
}

fn malformed(qdid: QualifiedDataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name: Ac2LandDefs::TYPE_NAME,
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_ac2_land_defs, AC2_BODY_BYTES};
    use ac2_core::{DataId, QualifiedDataId};

    #[test]
    fn decodes_the_ac2_terrain_lattice_and_height_table() {
        let mut bytes = vec![0_u8; AC2_BODY_BYTES];
        bytes[0..4].copy_from_slice(&20.0_f32.to_le_bytes());
        bytes[4..8].copy_from_slice(&8_u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&2_u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&200.0_f32.to_le_bytes());
        bytes[16..20].copy_from_slice(&2000.0_f32.to_le_bytes());
        bytes[20..24].copy_from_slice(&4.0_f32.to_le_bytes());
        for index in 0..256 {
            let offset = 48 + index * 4;
            bytes[offset..offset + 4].copy_from_slice(&(index as f32 * 2.0).to_le_bytes());
        }

        let decoded =
            decode_ac2_land_defs(QualifiedDataId::new(DataId::new(0x3600_0000), 0), &bytes)
                .expect("fixture should decode");

        assert_eq!(decoded.logical_cell_size(), 20.0);
        assert_eq!(decoded.block_cells(), 8);
        assert_eq!(decoded.subdivisions_per_cell(), 2);
        assert_eq!(decoded.height_table()[255], 510.0);
    }
}
