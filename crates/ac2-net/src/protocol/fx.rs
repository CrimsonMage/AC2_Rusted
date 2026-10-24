//! `FXData` and `FXNode` as `PartGroupDataDesc` FX overrides carry them (`FXData::StreamPack`,
//! `FXNode::StreamPack`: admin 0x006A6180, Legions 0x00642410, instruction-identical).

use crate::protocol::property::{read_property_collection, PropertyGroup};
use crate::protocol::reader::Reader;
use crate::protocol::types::hash_count;
use crate::protocol::ProtocolError;

/// `Waveform`, packed by `STREAMPACK_SIZEOF` as its raw 0x2C bytes: the type, then base,
/// amplitude, phase, frequency and roughness, each with its velocity.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Waveform {
    pub kind: u32,
    pub values: [f32; 10],
}

impl Waveform {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let kind = reader.u32()?;
        let mut values = [0.0; 10];
        for value in &mut values {
            *value = reader.f32()?;
        }
        Ok(Self { kind, values })
    }
}

/// `StreakFX::StreamPack`: colour (`RGBAColor::StreamPack`, packed 32-bit), start and end
/// points, material instance, then the timing scalars.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StreakFx {
    pub color: u32,
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub material_instance: u32,
    pub lifespan: f32,
    pub trail_duration: f32,
    pub min_segment_time: f32,
    pub min_segment_distance: f32,
    pub texture_scale: f32,
}

/// `DecalFX::StreamPack`: colour, material, origin, lifespan, then the size, rotation and
/// position-radius waveforms.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DecalFx {
    pub color: u32,
    pub material: u32,
    pub origin: [f32; 3],
    pub lifespan: f32,
    pub size: Waveform,
    pub rotation: Waveform,
    pub position_radius: Waveform,
}

/// `RepulsorFX::StreamPack`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RepulsorFx {
    pub acceleration: f32,
    pub range: f32,
    pub lifetime: f32,
    pub attractor: u32,
    pub minimum_velocity: f32,
}

/// `EntityDesc::StreamPack` (the light of a `LightSourceFX`): a pack bitfield, then the fields
/// it names.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntityDesc {
    pub bitfield: u32,
    /// Bit 0: `StreamDBObj::StreamPack`, the DID.
    pub did: Option<u32>,
    pub kind: Option<u32>,
    pub runtime_id: Option<u64>,
    pub data_id: Option<u32>,
    /// Bit `0x10`: the `Frame`, copied raw (quaternion, cached matrix, origin: 16 floats).
    pub offset: Option<[f32; 16]>,
    pub scale: Option<[f32; 3]>,
    pub version: Option<u32>,
    /// Bit `0x80`: the property collection (decoded only with `MasterProperty` types).
    pub properties: Vec<PropertyGroup>,
    /// Bit `0x100`: the workbench name.
    pub workbench_name: Option<Vec<u8>>,
}

impl EntityDesc {
    fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bits = reader.u32()?;
        let mut d = Self {
            bitfield: bits,
            ..Self::default()
        };
        if bits & 0x1 != 0 {
            d.did = Some(reader.u32()?);
        }
        if bits & 0x2 != 0 {
            d.kind = Some(reader.u32()?);
        }
        if bits & 0x4 != 0 {
            d.runtime_id = Some(reader.u64()?);
        }
        if bits & 0x8 != 0 {
            d.data_id = Some(reader.u32()?);
        }
        if bits & 0x10 != 0 {
            let mut frame = [0.0; 16];
            for value in &mut frame {
                *value = reader.f32()?;
            }
            d.offset = Some(frame);
        }
        if bits & 0x20 != 0 {
            d.scale = Some(reader.vec3()?);
        }
        if bits & 0x80 != 0 {
            // `BaseProperty::StreamPack` types each value by `MasterProperty`'s table for its
            // name, which only the DAT holds.
            let Some(types) = reader.property_types else {
                return Err(ProtocolError::Unsupported(
                    "EntityDesc property collection (MasterProperty types)",
                ));
            };
            d.properties = read_property_collection(reader, types)?;
        }
        if bits & 0x40 != 0 {
            d.version = Some(reader.u32()?);
        }
        if bits & 0x100 != 0 {
            d.workbench_name = Some(reader.string()?);
        }
        Ok(d)
    }
}

/// `LightSourceFX::StreamPack`: the light's entity description, lifespan and fade time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LightSourceFx {
    pub entity: EntityDesc,
    pub lifespan: f32,
    pub fade_time: f32,
}

/// `FXNode::StreamPack`: a 64-bit pack bitfield (`FXNode::get_pack_bitfield`), then the fields
/// it names in stream order. Integer flags travel as 32-bit words.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FxNode {
    pub bitfield: u64,
    pub sounds: Vec<u32>,
    pub music: Vec<u32>,
    pub particles: Vec<u32>,
    pub camera_fx: Option<u32>,
    pub appearance: Option<u32>,
    /// `AppliedAppearanceKey`: key and scalar.
    pub appearance_key: Option<(u32, f32)>,
    pub ramp_time: Option<f32>,
    pub intensity: Option<f32>,
    pub holding_location: Option<u32>,
    pub bone_name: Option<u32>,
    pub bone_offset: Option<[f32; 3]>,
    /// `m_has_streak` and the streak.
    pub streak: Option<(u32, StreakFx)>,
    pub start_streak: Option<u32>,
    pub stop_streak: Option<u32>,
    pub slider: Option<u32>,
    pub restore: Option<u32>,
    /// `m_has_color` and the colour's r, g, b, a.
    pub color: Option<(u32, [f32; 4])>,
    pub prop_color: Option<u32>,
    pub visual_desc: Option<u32>,
    pub lifetime: Option<f32>,
    pub color_lifetime: Option<f32>,
    pub behavior: Option<u32>,
    pub parent: Option<u32>,
    pub unparent: Option<u32>,
    pub retarget: Option<u32>,
    pub destroy_target: Option<u32>,
    pub hide_target: Option<u32>,
    pub target_fx: Option<u32>,
    pub day_time: Option<u32>,
    pub night_time: Option<u32>,
    /// `m_channel_keys`: key and value pairs.
    pub channel_keys: Vec<(u32, u32)>,
    pub fire_and_forget: Option<u32>,
    pub water_plane: Option<u32>,
    pub conditional: Option<u32>,
    pub decal: Option<(u32, DecalFx)>,
    pub repulsor: Option<(u32, RepulsorFx)>,
    pub light: Option<(u32, LightSourceFx)>,
    pub player_only: Option<u32>,
    pub fx_script: Option<u32>,
    pub remove_rotation: Option<u32>,
}

const HI: u32 = 32;

impl FxNode {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bits = reader.u64()?;
        let mut n = Self {
            bitfield: bits,
            ..Self::default()
        };
        let has = |bit: u64| bits & bit != 0;
        let list = |reader: &mut Reader<'_>| -> Result<Vec<u32>, ProtocolError> {
            (0..reader.count()?).map(|_| reader.u32()).collect()
        };
        let word = |reader: &mut Reader<'_>, bit: u64| -> Result<Option<u32>, ProtocolError> {
            has(bit).then(|| reader.u32()).transpose()
        };
        let float = |reader: &mut Reader<'_>, bit: u64| -> Result<Option<f32>, ProtocolError> {
            has(bit).then(|| reader.f32()).transpose()
        };
        if has(0x1) {
            n.sounds = list(reader)?;
        }
        if has(0x80 << HI) {
            n.music = list(reader)?;
        }
        if has(0x2) {
            n.particles = list(reader)?;
        }
        n.camera_fx = word(reader, 0x4)?;
        n.appearance = word(reader, 0x8)?;
        if has(0x10) {
            n.appearance_key = Some((reader.u32()?, reader.f32()?));
        }
        n.ramp_time = float(reader, 0x20)?;
        n.intensity = float(reader, 0x40)?;
        n.holding_location = word(reader, 0x80)?;
        n.bone_name = word(reader, 0x100)?;
        if has(0x200) {
            n.bone_offset = Some(reader.vec3()?);
        }
        if has(0x400) {
            let flag = reader.u32()?;
            n.streak = Some((
                flag,
                StreakFx {
                    color: reader.u32()?,
                    start: reader.vec3()?,
                    end: reader.vec3()?,
                    material_instance: reader.u32()?,
                    lifespan: reader.f32()?,
                    trail_duration: reader.f32()?,
                    min_segment_time: reader.f32()?,
                    min_segment_distance: reader.f32()?,
                    texture_scale: reader.f32()?,
                },
            ));
        }
        n.start_streak = word(reader, 0x800)?;
        n.stop_streak = word(reader, 0x1000)?;
        n.slider = word(reader, 0x2000)?;
        n.restore = word(reader, 0x4000)?;
        if has(0x8000) {
            let flag = reader.u32()?;
            let rgba = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
            n.color = Some((flag, rgba));
        }
        n.prop_color = word(reader, 0x1_0000)?;
        n.visual_desc = word(reader, 0x2_0000)?;
        n.lifetime = float(reader, 0x4_0000)?;
        n.color_lifetime = float(reader, 0x20 << HI)?;
        n.behavior = word(reader, 0x8_0000)?;
        n.parent = word(reader, 0x10_0000)?;
        n.unparent = word(reader, 0x20_0000)?;
        n.retarget = word(reader, 0x40_0000)?;
        n.destroy_target = word(reader, 0x80_0000)?;
        n.hide_target = word(reader, 0x8 << HI)?;
        n.target_fx = word(reader, 0x100_0000)?;
        n.day_time = word(reader, 0x200_0000)?;
        n.night_time = word(reader, 0x400_0000)?;
        if has(0x800_0000) {
            n.channel_keys = (0..hash_count(reader)?)
                .map(|_| Ok((reader.u32()?, reader.u32()?)))
                .collect::<Result<_, ProtocolError>>()?;
        }
        n.fire_and_forget = word(reader, 0x1000_0000)?;
        n.water_plane = word(reader, 0x2000_0000)?;
        n.conditional = word(reader, 0x4000_0000)?;
        if has(0x8000_0000) {
            let flag = reader.u32()?;
            n.decal = Some((
                flag,
                DecalFx {
                    color: reader.u32()?,
                    material: reader.u32()?,
                    origin: reader.vec3()?,
                    lifespan: reader.f32()?,
                    size: Waveform::read(reader)?,
                    rotation: Waveform::read(reader)?,
                    position_radius: Waveform::read(reader)?,
                },
            ));
        }
        if has(0x2 << HI) {
            let flag = reader.u32()?;
            n.repulsor = Some((
                flag,
                RepulsorFx {
                    acceleration: reader.f32()?,
                    range: reader.f32()?,
                    lifetime: reader.f32()?,
                    attractor: reader.u32()?,
                    minimum_velocity: reader.f32()?,
                },
            ));
        }
        if has(0x1 << HI) {
            let flag = reader.u32()?;
            n.light = Some((
                flag,
                LightSourceFx {
                    entity: EntityDesc::read(reader)?,
                    lifespan: reader.f32()?,
                    fade_time: reader.f32()?,
                },
            ));
        }
        n.player_only = word(reader, 0x4 << HI)?;
        n.fx_script = word(reader, 0x10 << HI)?;
        n.remove_rotation = word(reader, 0x40 << HI)?;
        Ok(n)
    }
}

/// `FXData::StreamPack`: the default node, then terrain type -> node
/// (`PHashTable<unsigned long, FXNode>`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FxData {
    pub default: FxNode,
    pub terrain: Vec<(u32, FxNode)>,
}

impl FxData {
    fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let default = FxNode::read(reader)?;
        let terrain = (0..hash_count(reader)?)
            .map(|_| Ok((reader.u32()?, FxNode::read(reader)?)))
            .collect::<Result<_, ProtocolError>>()?;
        Ok(Self { default, terrain })
    }
}

/// `PHashTable<unsigned long, PSmartArray<FXData>>::StreamPack`: FX id -> its variants
/// (`PSmartArray<FXData>::StreamPack`: a u32 count, then the entries).
pub(crate) fn fx_overrides(
    reader: &mut Reader<'_>,
) -> Result<Vec<(u32, Vec<FxData>)>, ProtocolError> {
    (0..hash_count(reader)?)
        .map(|_| {
            let key = reader.u32()?;
            let variants = (0..reader.count()?)
                .map(|_| FxData::read(reader))
                .collect::<Result<_, ProtocolError>>()?;
            Ok((key, variants))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{fx_overrides, FxNode};
    use crate::protocol::reader::Reader;
    use crate::protocol::ProtocolError;

    fn words(out: &mut Vec<u8>, values: &[u32]) {
        out.extend(values.iter().flat_map(|v| v.to_le_bytes()));
    }

    fn floats(out: &mut Vec<u8>, values: &[f32]) {
        out.extend(values.iter().flat_map(|v| v.to_le_bytes()));
    }

    #[test]
    fn fields_follow_the_sixty_four_bit_bitfield_in_stream_order() {
        // sound, music (high 0x80), apr key, color, color lifetime (high 0x20), hide target
        // (high 8), channel keys, repulsor (high 2), remove rotation (high 0x40).
        let bits: u64 = 0x1 | 0x10 | 0x8000 | 0x800_0000 | (0x80 | 0x20 | 0x8 | 0x2 | 0x40) << 32;
        let mut b = bits.to_le_bytes().to_vec();
        words(&mut b, &[2, 0x0A00_0001, 0x0A00_0002]); // sounds
        words(&mut b, &[1, 0x0A00_0009]); // music
        words(&mut b, &[0x1000_0003]); // appearance key, scalar
        floats(&mut b, &[0.5]);
        words(&mut b, &[1]); // has colour, r g b a
        floats(&mut b, &[1.0, 0.5, 0.25, 1.0]);
        floats(&mut b, &[2.0]); // colour lifetime
        words(&mut b, &[1]); // hide target
        words(&mut b, &[1 | (2 << 24), 7, 8]); // one channel key
        words(&mut b, &[1]); // has repulsor
        floats(&mut b, &[3.0, 4.0, 5.0]);
        words(&mut b, &[0]);
        floats(&mut b, &[0.1]);
        words(&mut b, &[1]); // remove rotation
        let mut reader = Reader::new(&b);
        let n = FxNode::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!(n.sounds, [0x0A00_0001, 0x0A00_0002]);
        assert_eq!(n.music, [0x0A00_0009]);
        assert_eq!(n.appearance_key, Some((0x1000_0003, 0.5)));
        assert_eq!(n.color, Some((1, [1.0, 0.5, 0.25, 1.0])));
        assert_eq!((n.color_lifetime, n.hide_target), (Some(2.0), Some(1)));
        assert_eq!(n.channel_keys, [(7, 8)]);
        assert_eq!(n.repulsor.unwrap().1.range, 4.0);
        assert_eq!(n.remove_rotation, Some(1));
        for end in 0..b.len() {
            assert!(FxNode::read(&mut Reader::new(&b[..end])).is_err());
        }
    }

    #[test]
    fn overrides_hold_fx_data_arrays_with_terrain_nodes() {
        let mut b = Vec::new();
        words(&mut b, &[1, 0x22, 1]); // one FX id, one FXData
        b.extend((0x20_0000u64 | 0x40 << 32).to_le_bytes()); // default: unparent, remove rot
        words(&mut b, &[1, 0]);
        words(&mut b, &[1, 0x0D]); // one terrain node
        b.extend(0x2_0000u64.to_le_bytes());
        words(&mut b, &[0x1F00_0005]);
        let mut reader = Reader::new(&b);
        let table = fx_overrides(&mut reader).unwrap();
        reader.finish().unwrap();
        let data = &table[0].1[0];
        assert_eq!(table[0].0, 0x22);
        assert_eq!(
            (data.default.unparent, data.default.remove_rotation),
            (Some(1), Some(0))
        );
        assert_eq!(data.terrain[0].0, 0x0D);
        assert_eq!(data.terrain[0].1.visual_desc, Some(0x1F00_0005));
    }

    #[test]
    fn a_light_with_properties_reads_them_with_the_master_property_types() {
        let mut b = (0x1u64 << 32).to_le_bytes().to_vec();
        words(&mut b, &[1, 0x80, 1, 0x2000_0001, 1, 0x30]); // one group, one property
        floats(&mut b, &[2.5, 1.0, 2.0]); // its float value, lifespan, fade
        let types = |name: u32| (name == 0x30).then_some(0x4000_0003);
        let mut reader = Reader::with_property_types(&b, &types);
        let light = FxNode::read(&mut reader).unwrap().light.unwrap().1;
        reader.finish().unwrap();
        let property = &light.entity.properties[0].properties[0];
        assert_eq!(
            (property.name, &property.value),
            (0x30, &crate::protocol::property::PropertyValue::Float(2.5))
        );
        assert_eq!((light.lifespan, light.fade_time), (1.0, 2.0));
    }

    #[test]
    fn a_light_with_properties_needs_the_master_property_table() {
        let mut b = (0x1u64 << 32).to_le_bytes().to_vec();
        words(&mut b, &[1, 0x80]);
        assert!(matches!(
            FxNode::read(&mut Reader::new(&b)),
            Err(ProtocolError::Unsupported(_))
        ));
        let mut b = (0x1u64 << 32).to_le_bytes().to_vec();
        words(&mut b, &[1, 0x0B, 0x2700_0001, 7, 1]); // DID, kind, data id
        floats(&mut b, &[1.0, 2.0]);
        let mut reader = Reader::new(&b);
        let light = FxNode::read(&mut reader).unwrap().light.unwrap().1;
        reader.finish().unwrap();
        assert_eq!(
            (light.entity.did, light.entity.kind, light.entity.data_id),
            (Some(0x2700_0001), Some(7), Some(1))
        );
    }
}
