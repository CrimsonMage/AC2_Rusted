//! Asheron's Call 2 material resolution: a `RenderMaterial` (`0x2B`) directly, or a
//! `MaterialInstance` (`0x31`) whose `MaterialModifier`s (`0x30`) are applied to a copy of its
//! template material, as `MaterialInstance::RefreshMaterial` does.

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};

use crate::ac2_material::{
    waveform_from_bytes, Ac2LayerModifier, Ac2MaterialInstance, Ac2MaterialModifier,
    Ac2MaterialProperty, Ac2RenderMaterial,
};

const RENDER_MATERIAL_PREFIX: u32 = 0x2B00_0000;
const MATERIAL_INSTANCE_PREFIX: u32 = 0x3100_0000;

// `RMFieldType` values handled by `MaterialProperty::Apply`.
const FIELD_AMBIENT: u32 = 0;
const FIELD_DIFFUSE: u32 = 1;
const FIELD_SPECULAR: u32 = 2;
const FIELD_SPECULAR_POWER: u32 = 3;
const FIELD_CULL_MODE: u32 = 5;
const FIELD_DEPTH_TEST: u32 = 6;
const FIELD_DEPTH_WRITE: u32 = 7;
const FIELD_ALPHA_TEST: u32 = 8;
const FIELD_ALPHA_TEST_REF: u32 = 9;
const FIELD_STAGE_TEXTURE: u32 = 10;
const FIELD_STAGE_ADDRESS_U: u32 = 11;
const FIELD_STAGE_ADDRESS_V: u32 = 12;
const FIELD_UV_TRANSLATE_U: u32 = 33;
const FIELD_UV_TRANSLATE_V: u32 = 34;
const FIELD_UV_ROTATE: u32 = 35;
const FIELD_UV_SCALE_U: u32 = 36;
const FIELD_UV_SCALE_V: u32 = 37;

// `CullMode`, `DepthTestMode`, `TexAddress` values the boolean fields map to.
const CULLMODE_NONE: u32 = 1;
const CULLMODE_CW: u32 = 2;
const DEPTHTEST_LESSEQUAL: u32 = 4;
const DEPTHTEST_ALWAYS: u32 = 8;
const TEXADDRESS_WRAP: u32 = 1;
const TEXADDRESS_CLAMP: u32 = 3;

/// Loads an AC2 material by `RenderMaterial` or `MaterialInstance` DID. Instances resolve to their
/// template with every modifier applied; a missing modifier is skipped like a null slot.
pub fn load_ac2_material(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<Ac2RenderMaterial>> {
    match did.raw() & 0xFF00_0000 {
        RENDER_MATERIAL_PREFIX => {
            Ok(load_portal::<Ac2RenderMaterial>(store, did)?.map(|material| (*material).clone()))
        }
        MATERIAL_INSTANCE_PREFIX => {
            let Some(instance) = load_portal::<Ac2MaterialInstance>(store, did)? else {
                return Ok(None);
            };
            if instance.material.raw() & 0xFF00_0000 != RENDER_MATERIAL_PREFIX {
                return Ok(None);
            }
            let Some(template) = load_portal::<Ac2RenderMaterial>(store, instance.material)? else {
                return Ok(None);
            };
            let mut material = (*template).clone();
            for modifier_did in &instance.modifiers {
                if let Some(modifier) = load_portal::<Ac2MaterialModifier>(store, *modifier_did)? {
                    apply_modifier(&mut material, &template, &modifier);
                }
            }
            Ok(Some(material))
        }
        _ => Ok(None),
    }
}

fn load_portal<T: ac2_db::ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<ac2_db::CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}

/// `MaterialInstance::RefreshMaterial`: each modifier property overrides the template property
/// with the same name, written through that template property's fields.
pub(crate) fn apply_modifier(
    material: &mut Ac2RenderMaterial,
    template: &Ac2RenderMaterial,
    modifier: &Ac2MaterialModifier,
) {
    for property in &modifier.properties {
        if let Some(reference) = template
            .properties
            .iter()
            .find(|candidate| candidate.name_id == property.name_id)
        {
            apply_property(material, property, reference);
        }
    }
}

/// `MaterialProperty::Apply`.
fn apply_property(
    material: &mut Ac2RenderMaterial,
    property: &Ac2MaterialProperty,
    reference: &Ac2MaterialProperty,
) {
    let word = property
        .data
        .get(..4)
        .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
    let color = color_from_bytes(&property.data);
    let waveform = waveform_from_bytes(&property.data);
    for field in &reference.fields {
        let Some(layer) = material.layers.get_mut(field.layer as usize) else {
            continue;
        };
        let index = field.index as usize;
        match field.field_type {
            FIELD_AMBIENT => set(&mut layer.ambient, color),
            FIELD_DIFFUSE => set(&mut layer.diffuse, color),
            FIELD_SPECULAR => set(&mut layer.specular, color),
            FIELD_SPECULAR_POWER => set(&mut layer.specular_power, waveform),
            FIELD_CULL_MODE => set(
                &mut layer.cull_mode,
                word.map(|w| if w == 0 { CULLMODE_CW } else { CULLMODE_NONE }),
            ),
            FIELD_DEPTH_TEST => set(
                &mut layer.depth_test,
                word.map(|w| {
                    if w == 0 {
                        DEPTHTEST_ALWAYS
                    } else {
                        DEPTHTEST_LESSEQUAL
                    }
                }),
            ),
            FIELD_DEPTH_WRITE => set(&mut layer.depth_write, word.map(|w| u32::from(w != 0))),
            FIELD_ALPHA_TEST => set(&mut layer.alpha_test_mode, word.map(|w| u32::from(w != 0))),
            FIELD_ALPHA_TEST_REF => set(&mut layer.alpha_test_ref, waveform),
            FIELD_STAGE_TEXTURE | FIELD_STAGE_ADDRESS_U | FIELD_STAGE_ADDRESS_V => {
                let (Some(stage), Some(word)) = (layer.stages.get_mut(index), word) else {
                    continue;
                };
                let address = if word == 0 {
                    TEXADDRESS_CLAMP
                } else {
                    TEXADDRESS_WRAP
                };
                match field.field_type {
                    FIELD_STAGE_TEXTURE => stage.texture = (word != 0).then(|| DataId::new(word)),
                    FIELD_STAGE_ADDRESS_U => stage.address_u = address,
                    _ => stage.address_v = address,
                }
            }
            FIELD_UV_TRANSLATE_U..=FIELD_UV_SCALE_V => {
                if let (Some(modifier), Some(waveform)) = (layer.modifiers.get_mut(index), waveform)
                {
                    apply_uv_waveform(modifier, field.field_type, waveform);
                }
            }
            // Origin, normal and diffuse layer modifiers are not streamed by `MaterialLayer`.
            _ => {}
        }
    }
}

/// The client writes these through `LM_UVTranslate` offsets regardless of the modifier's class:
/// rotate and the U fields land in the first waveform, the V fields in the second.
fn apply_uv_waveform(modifier: &mut Ac2LayerModifier, field: u32, waveform: crate::Ac2Waveform) {
    let second = match field {
        FIELD_UV_TRANSLATE_U | FIELD_UV_ROTATE | FIELD_UV_SCALE_U => false,
        FIELD_UV_TRANSLATE_V | FIELD_UV_SCALE_V => true,
        _ => return,
    };
    match modifier {
        Ac2LayerModifier::UvTranslate { u, v, .. } | Ac2LayerModifier::UvScale { u, v, .. } => {
            *(if second { v } else { u }) = waveform;
        }
        Ac2LayerModifier::UvRotate { rotate, .. } if !second => *rotate = waveform,
        _ => {}
    }
}

fn color_from_bytes(bytes: &[u8]) -> Option<[f32; 4]> {
    let values = bytes.get(..16)?;
    let mut color = [0.0_f32; 4];
    for (slot, chunk) in color.iter_mut().zip(values.chunks_exact(4)) {
        *slot = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    Some(color)
}

fn set<T>(slot: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *slot = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ac2_material::{
        Ac2LayerLighting, Ac2LayerStage, Ac2MaterialField, Ac2MaterialLayer,
    };
    use crate::Ac2Waveform;

    fn layer(texture: u32) -> Ac2MaterialLayer {
        Ac2MaterialLayer {
            true_flags: 0,
            false_flags: 0,
            stages: vec![Ac2LayerStage {
                texture: Some(DataId::new(texture)),
                address_u: TEXADDRESS_WRAP,
                ..Ac2LayerStage::default()
            }],
            modifiers: Vec::new(),
            source_blend: 2,
            dest_blend: 1,
            depth_test: DEPTHTEST_LESSEQUAL,
            depth_write: 1,
            cull_mode: CULLMODE_CW,
            alpha_test_mode: 0,
            alpha_test_ref: Ac2Waveform::default(),
            lighting: Ac2LayerLighting::default(),
            ambient: [1.0; 4],
            diffuse: [1.0; 4],
            specular: [0.0; 4],
            specular_power: Ac2Waveform::default(),
            options: 0,
            vertex_format: 0,
            vertex_shader: Default::default(),
            pixel_shader: Default::default(),
        }
    }

    #[test]
    fn modifier_texture_property_replaces_the_template_stage_texture() {
        let field = |field_type| Ac2MaterialField {
            field_type,
            layer: 0,
            index: 0,
        };
        let template = Ac2RenderMaterial {
            properties: vec![
                Ac2MaterialProperty {
                    name_id: 7,
                    data: 0x4000_0001_u32.to_le_bytes().to_vec(),
                    fields: vec![field(FIELD_STAGE_TEXTURE)],
                },
                Ac2MaterialProperty {
                    name_id: 9,
                    data: 1_u32.to_le_bytes().to_vec(),
                    fields: vec![field(FIELD_STAGE_ADDRESS_U), field(FIELD_CULL_MODE)],
                },
            ],
            layers: vec![layer(0x4000_0001)],
            sort_mode: 1,
        };
        let modifier = Ac2MaterialModifier {
            properties: vec![
                Ac2MaterialProperty {
                    name_id: 7,
                    data: 0x4000_0BEE_u32.to_le_bytes().to_vec(),
                    fields: Vec::new(),
                },
                Ac2MaterialProperty {
                    name_id: 9,
                    data: 0_u32.to_le_bytes().to_vec(),
                    fields: Vec::new(),
                },
            ],
        };
        let mut material = template.clone();
        apply_modifier(&mut material, &template, &modifier);
        assert_eq!(material.base_texture(), Some(DataId::new(0x4000_0BEE)));
        assert_eq!(material.layers[0].stages[0].address_u, TEXADDRESS_CLAMP);
        assert_eq!(material.layers[0].cull_mode, CULLMODE_CW);
    }
}
