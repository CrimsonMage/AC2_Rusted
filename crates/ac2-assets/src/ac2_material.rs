//! Asheron's Call 2 material records, decoded in the order of the client's own `StreamPack`
//! methods (`RenderMaterial`, `MaterialLayer`, `LayerStage`, `LM_*`, `MaterialModifier`,
//! `MaterialProperty`). Every Legions `0x2B` material decodes to its exact record length.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, ArchiveError};
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

/// `RenderTexture` (`0x40`) DIDs: what a stage texture or texture property names.
pub(crate) const AC2_TEXTURE_CHAIN_PREFIX: u32 = 0x4000_0000;
/// `MaterialLayer::StreamPack` rejects more than four stages.
const MAX_LAYER_STAGES: usize = 4;
const WAVEFORM_BYTES: usize = 44;

/// AC2 `Waveform`, evaluated by `Waveform::GetValue` in `ac2-math`.
pub use ac2_math::Waveform as Ac2Waveform;

/// AC2 `LayerStage`: one fixed-function texture stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ac2LayerStage {
    /// `RenderTexture` chain, or `None` for an untextured stage.
    pub texture: Option<DataId>,
    pub color_op: u32,
    pub color_arg1: u32,
    pub color_arg2: u32,
    pub alpha_op: u32,
    pub alpha_arg1: u32,
    pub alpha_arg2: u32,
    pub address_u: u32,
    pub address_v: u32,
    pub min_filter: u32,
    pub mag_filter: u32,
    pub tex_coord: u32,
    pub use_projection: bool,
    pub special_texture: u32,
}

/// The four layer modifiers `MaterialLayer::StreamPack` accepts (types 4-7).
#[derive(Debug, Clone, PartialEq)]
pub enum Ac2LayerModifier {
    UvTranslate {
        tex_coord: u32,
        u: Ac2Waveform,
        v: Ac2Waveform,
    },
    UvRotate {
        tex_coord: u32,
        rotate: Ac2Waveform,
    },
    UvScale {
        tex_coord: u32,
        u: Ac2Waveform,
        v: Ac2Waveform,
    },
    UvTransform {
        stage: u32,
        matrix: [f32; 16],
    },
}

/// Lighting and fog switches of a `MaterialLayer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ac2LayerLighting {
    pub particle: bool,
    pub dynamic: bool,
    pub per_pixel: bool,
    pub ignore_directional: bool,
    pub ignore_point: bool,
    pub specular_highlights: bool,
    pub pixel_specular: bool,
    pub material_diffuse: bool,
    pub vertex_diffuse: bool,
    pub modulate_vertex_diffuse: bool,
    pub pixel_multiple_lights: bool,
    pub depth_fog: bool,
    pub depth_alpha_fog: bool,
    pub diffuse_fog_amount: bool,
}

/// AC2 `MaterialLayer`: one render pass of a material.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2MaterialLayer {
    pub true_flags: u32,
    pub false_flags: u32,
    pub stages: Vec<Ac2LayerStage>,
    pub modifiers: Vec<Ac2LayerModifier>,
    pub source_blend: u32,
    pub dest_blend: u32,
    pub depth_test: u32,
    pub depth_write: u32,
    pub cull_mode: u32,
    pub alpha_test_mode: u32,
    pub alpha_test_ref: Ac2Waveform,
    pub lighting: Ac2LayerLighting,
    pub ambient: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub specular_power: Ac2Waveform,
    pub options: u64,
    pub vertex_format: u32,
    /// Authored vertex/pixel shader source sizes (the source itself is not retained).
    pub vertex_shader_bytes: u32,
    pub pixel_shader_bytes: u32,
}

/// `MaterialField`: where a material property writes (`RMFieldType`, layer, stage/modifier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ac2MaterialField {
    pub(crate) field_type: u32,
    pub(crate) layer: u32,
    pub(crate) index: u32,
}

/// `MaterialProperty`: a named value (`RMDataType` waveform/color/texture/bool) and, on a base
/// material, the fields it drives.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ac2MaterialProperty {
    pub(crate) name_id: u32,
    pub(crate) data: Vec<u8>,
    pub(crate) fields: Vec<Ac2MaterialField>,
}

/// AC2 `RenderMaterial` (`0x2B`): property table, layers and sort mode.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2RenderMaterial {
    pub(crate) properties: Vec<Ac2MaterialProperty>,
    pub layers: Vec<Ac2MaterialLayer>,
    pub sort_mode: u32,
}

impl Ac2RenderMaterial {
    /// Base texture: the first textured stage of the first layer that has one.
    pub fn base_texture(&self) -> Option<DataId> {
        self.layers
            .iter()
            .flat_map(|layer| layer.stages.iter())
            .find_map(|stage| stage.texture)
    }
}

impl ResourcePackedDecode for Ac2RenderMaterial {
    const TYPE_NAME: &'static str = "AC2 RenderMaterial";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        // The property table is an embedded `MaterialModifier` with its own (zero) type header.
        let _embedded_did = cursor.read_u32()?;
        let properties = read_properties(&mut cursor)?;
        let layer_count = read_count(&mut cursor, 8)?;
        let layers = (0..layer_count)
            .map(|_| read_layer(&mut cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let sort_mode = cursor.read_u32()?;
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            properties,
            layers,
            sort_mode,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.layers.len() * std::mem::size_of::<Ac2MaterialLayer>()
            + self
                .properties
                .iter()
                .map(|p| p.data.len() + 32)
                .sum::<usize>()
    }
}

/// AC2 `MaterialModifier` (`0x30`): property overrides matched to a material by name.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ac2MaterialModifier {
    pub(crate) properties: Vec<Ac2MaterialProperty>,
}

impl ResourcePackedDecode for Ac2MaterialModifier {
    const TYPE_NAME: &'static str = "AC2 MaterialModifier";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let properties = read_properties(&mut cursor)?;
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self { properties })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .properties
                .iter()
                .map(|p| p.data.len() + 32)
                .sum::<usize>()
    }
}

/// AC2 `MaterialInstance` (`0x31`): a template material plus the modifiers applied to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Ac2MaterialInstance {
    pub(crate) material: DataId,
    /// `m_materialType`: the slot an `AppearanceTable` `MaterialModifierAtom` addresses.
    pub(crate) material_type: u32,
    pub(crate) modifiers: Vec<DataId>,
}

impl ResourcePackedDecode for Ac2MaterialInstance {
    const TYPE_NAME: &'static str = "AC2 MaterialInstance";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let material = DataId::new(cursor.read_u32()?);
        let material_type = cursor.read_u32()?;
        let count = read_count(&mut cursor, 4)?;
        let modifiers = cursor
            .read_u32_items(count)?
            .into_iter()
            .filter(|did| *did != 0)
            .map(DataId::new)
            .collect();
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            material,
            material_type,
            modifiers,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.modifiers.len() * 4
    }
}

fn read_properties(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<Vec<Ac2MaterialProperty>, ArchiveError> {
    let count = read_count(cursor, 16)?;
    (0..count)
        .map(|_| {
            let name_id = cursor.read_u32()?;
            let _data_type = cursor.read_u32()?;
            let len = read_count(cursor, 1)?;
            let data = cursor.read_bytes(len)?.to_vec();
            let field_count = read_count(cursor, 16)?;
            let fields = (0..field_count)
                .map(|_| {
                    let field_type = cursor.read_u32()?;
                    let _data_type = cursor.read_u32()?;
                    Ok(Ac2MaterialField {
                        field_type,
                        layer: cursor.read_u32()?,
                        index: cursor.read_u32()?,
                    })
                })
                .collect::<Result<Vec<_>, ArchiveError>>()?;
            Ok(Ac2MaterialProperty {
                name_id,
                data,
                fields,
            })
        })
        .collect()
}

fn read_layer(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2MaterialLayer, ArchiveError> {
    let true_flags = cursor.read_u32()?;
    let false_flags = cursor.read_u32()?;
    let stage_count = read_count(cursor, 56)?;
    let stages = (0..stage_count)
        .map(|_| read_stage(cursor))
        .collect::<Result<Vec<_>, _>>()?;
    if stages.len() > MAX_LAYER_STAGES {
        return Err(out_of_range("AC2 material layer stage count", stages.len()));
    }
    let modifier_count = read_count(cursor, 8)?;
    let modifiers = (0..modifier_count)
        .map(|_| read_modifier(cursor))
        .collect::<Result<Vec<_>, _>>()?;
    let mut words = [0_u32; 6];
    for word in &mut words {
        *word = cursor.read_u32()?;
    }
    let alpha_test_ref = read_waveform(cursor)?;
    let mut flags = [false; 14];
    for flag in &mut flags {
        *flag = cursor.read_u32()? != 0;
    }
    let ambient = read_color(cursor)?;
    let diffuse = read_color(cursor)?;
    let specular = read_color(cursor)?;
    let specular_power = read_waveform(cursor)?;
    let options = cursor.read_u64()?;
    let _reserved_future_use = cursor.read_u32()?;
    let _reserved_do_not_use = cursor.read_u32()?;
    let vertex_format = cursor.read_u32()?;
    let vertex_shader_bytes = cursor.read_u32()?;
    cursor.read_bytes(vertex_shader_bytes as usize)?;
    let pixel_shader_bytes = cursor.read_u32()?;
    cursor.read_bytes(pixel_shader_bytes as usize)?;
    Ok(Ac2MaterialLayer {
        true_flags,
        false_flags,
        stages,
        modifiers,
        source_blend: words[0],
        dest_blend: words[1],
        depth_test: words[2],
        depth_write: words[3],
        cull_mode: words[4],
        alpha_test_mode: words[5],
        alpha_test_ref,
        // `MaterialLayer::StreamPack` order.
        lighting: Ac2LayerLighting {
            particle: flags[0],
            dynamic: flags[1],
            per_pixel: flags[2],
            ignore_directional: flags[3],
            ignore_point: flags[4],
            specular_highlights: flags[5],
            pixel_specular: flags[6],
            material_diffuse: flags[7],
            vertex_diffuse: flags[8],
            modulate_vertex_diffuse: flags[9],
            pixel_multiple_lights: flags[10],
            depth_fog: flags[11],
            depth_alpha_fog: flags[12],
            diffuse_fog_amount: flags[13],
        },
        ambient,
        diffuse,
        specular,
        specular_power,
        options,
        vertex_format,
        vertex_shader_bytes,
        pixel_shader_bytes,
    })
}

fn read_stage(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2LayerStage, ArchiveError> {
    let texture = cursor.read_u32()?;
    let mut words = [0_u32; 13];
    for word in &mut words {
        *word = cursor.read_u32()?;
    }
    Ok(Ac2LayerStage {
        texture: (texture != 0).then(|| DataId::new(texture)),
        color_op: words[0],
        color_arg1: words[1],
        color_arg2: words[2],
        alpha_op: words[3],
        alpha_arg1: words[4],
        alpha_arg2: words[5],
        address_u: words[6],
        address_v: words[7],
        min_filter: words[8],
        mag_filter: words[9],
        tex_coord: words[10],
        use_projection: words[11] != 0,
        special_texture: words[12],
    })
}

fn read_modifier(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2LayerModifier, ArchiveError> {
    let kind = cursor.read_u32()?;
    let index = cursor.read_u32()?;
    Ok(match kind {
        4 => Ac2LayerModifier::UvTranslate {
            tex_coord: index,
            u: read_waveform(cursor)?,
            v: read_waveform(cursor)?,
        },
        5 => Ac2LayerModifier::UvRotate {
            tex_coord: index,
            rotate: read_waveform(cursor)?,
        },
        6 => Ac2LayerModifier::UvScale {
            tex_coord: index,
            u: read_waveform(cursor)?,
            v: read_waveform(cursor)?,
        },
        7 => {
            let mut matrix = [0.0_f32; 16];
            for value in &mut matrix {
                *value = cursor.read_f32()?;
            }
            Ac2LayerModifier::UvTransform {
                stage: index,
                matrix,
            }
        }
        other => {
            return Err(out_of_range(
                "AC2 material layer modifier type",
                other as usize,
            ))
        }
    })
}

pub(crate) fn read_waveform(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2Waveform, ArchiveError> {
    let kind = cursor.read_u32()?;
    let mut v = [0.0_f32; 10];
    for value in &mut v {
        *value = cursor.read_f32()?;
    }
    Ok(Ac2Waveform {
        kind,
        base: v[0],
        base_velocity: v[1],
        amplitude: v[2],
        amplitude_velocity: v[3],
        phase: v[4],
        phase_velocity: v[5],
        frequency: v[6],
        frequency_velocity: v[7],
        roughness: v[8],
        roughness_velocity: v[9],
    })
}

/// Parses a waveform from a property's raw data (`RMDATA_WAVEFORM`).
pub(crate) fn waveform_from_bytes(bytes: &[u8]) -> Option<Ac2Waveform> {
    (bytes.len() >= WAVEFORM_BYTES).then(|| read_waveform(&mut ArchiveCursor::new(bytes)).ok())?
}

fn read_color(cursor: &mut ArchiveCursor<'_>) -> Result<[f32; 4], ArchiveError> {
    Ok([
        cursor.read_f32()?,
        cursor.read_f32()?,
        cursor.read_f32()?,
        cursor.read_f32()?,
    ])
}

/// Reads a u32 count whose items need at least `min_item_bytes` each, so a corrupt count
/// fails before allocating.
fn read_count(
    cursor: &mut ArchiveCursor<'_>,
    min_item_bytes: usize,
) -> Result<usize, ArchiveError> {
    let count = cursor.read_u32_count()?;
    if count.saturating_mul(min_item_bytes) > cursor.remaining_bytes().len() {
        return Err(out_of_range("AC2 material item count", count));
    }
    Ok(count)
}

fn out_of_range(context: &'static str, value: usize) -> ArchiveError {
    ArchiveError::ValueOutOfRange {
        context,
        value: value as u64,
        max: u64::from(u32::MAX),
    }
}

fn expect_did(qdid: QualifiedDataId, type_name: &'static str, actual: u32) -> ResourceResult<()> {
    if actual == qdid.data_id().raw() {
        Ok(())
    } else {
        Err(ResourceError::MalformedTypedObject {
            qdid,
            type_name,
            context: "embedded DID did not match DAT identity",
        })
    }
}

fn expect_end(
    qdid: QualifiedDataId,
    type_name: &'static str,
    cursor: &ArchiveCursor<'_>,
) -> ResourceResult<()> {
    if cursor.remaining_bytes().is_empty() {
        Ok(())
    } else {
        Err(ResourceError::MalformedTypedObject {
            qdid,
            type_name,
            context: "trailing bytes after record",
        })
    }
}
