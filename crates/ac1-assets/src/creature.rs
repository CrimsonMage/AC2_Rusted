//! An ACE weenie as an `ac2-render` model: its AC1 Setup's parts after the ClothingBase's and
//! the weenie's own part and texture changes (`CPartArray::SetPart`/`SetTexture` over the
//! server's `ObjDesc`), each `CGfxObj`'s polygons fanned into triangle lists by surface, each
//! `CSurface` drawn as one lit textured material layer, its image through the object's composed
//! palette (`CPartArray::SetPalette`), and the motion table's idle cycle as per-part poses.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{Ac2Bone, Ac2DegradeInfo, Ac2PlacementFrame, Ac2Setup, Ac2Transform};
use ac2_assets::{Ac2LayerLighting, Ac2LayerStage, Ac2MaterialLayer, Ac2Waveform};
use ac2_core::DataId;
use ac2_math::{Quat, Vec3};
use ac2_physics::PartPose;
use ac2_render::object::{
    blend, ObjectBatch, ObjectMaterial, ObjectModel, ObjectPart, ObjectSkeleton, ObjectVertex,
    RenderSettings, SkeletonGroup,
};
use ac2_render::scene::{ForeignModel, ForeignModels, Images};
use ac2_render::TextureKey;

use crate::dats::Ac1Dats;
use crate::formats::{
    decode_degrade_closest, decode_palette, decode_palette_set, decode_surface_texture,
    palette_for_shade, surface_type as st, Ac1Error, Ac1Result, Animation, ClothingTable, Frame,
    GfxObj, MotionTable, RenderSurface, Setup, Surface,
};
use crate::image::{decode_rgba, mip_chain, solid};
use crate::weenie::AceWeenie;

/// The visual DIDs the AC1 creatures are drawn under: a range no AC2 DAT uses.
pub const AC1_VISUAL_BASE: u32 = 0xAC00_0000;
/// The texture DIDs their images are keyed by.
const AC1_TEXTURE_BASE: u32 = 0xAC80_0000;
const PALETTE_SET: u32 = 0x0F;

/// D3D values of the one layer an AC1 surface draws as.
mod d3d {
    pub const TOP_MODULATE: u32 = 4;
    pub const TOP_SELECTARG1: u32 = 2;
    pub const TA_DIFFUSE: u32 = 0;
    pub const TA_TEXTURE: u32 = 2;
    pub const TADDRESS_WRAP: u32 = 1;
    pub const TEXF_LINEAR: u32 = 2;
    pub const CMP_LESSEQUAL: u32 = 4;
    /// `D3DCULL_CW`, as AC2's object materials draw: the triangles below are wound
    /// counter-clockwise seen from their front, as AC2's meshes are.
    pub const CULL_CW: u32 = 2;
}

/// An AC1 creature ready to draw.
#[derive(Debug, Clone)]
pub struct Ac1Creature {
    pub weenie: AceWeenie,
    /// The visual DID its model is drawn under.
    pub visual: DataId,
    pub model: ObjectModel,
    pub images: Images,
    /// The motion table's idle cycle, when it has one.
    pub idle: Option<IdleCycle>,
}

/// A looping cycle of per-part poses (`CSequence` over the cycle's `AnimData`s).
#[derive(Debug, Clone, PartialEq)]
pub struct IdleCycle {
    /// Each frame's part poses and how long it shows.
    frames: Vec<(Vec<PartPose>, f64)>,
    length: f64,
}

impl IdleCycle {
    /// The poses at `time` seconds into the loop (each frame held, as `CSequence` steps whole
    /// frames).
    pub fn pose(&self, time: f64) -> &[PartPose] {
        let mut t = time.rem_euclid(self.length);
        for (pose, duration) in &self.frames {
            if t < *duration {
                return pose;
            }
            t -= duration;
        }
        &self.frames[self.frames.len() - 1].0
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// The part changes and palette an `ObjDesc` makes of a weenie.
struct Appearance {
    parts: Vec<u32>,
    /// `SurfaceTexture` swaps per part.
    textures: Vec<HashMap<u32, u32>>,
    /// (palette, start, count) copied over a texture's palette, in order.
    subpalettes: Vec<(u32, u32, u32)>,
}

impl Ac1Dats {
    /// Builds `weenie` drawn under `visual`.
    pub fn creature(&self, weenie: &AceWeenie, visual: DataId) -> Ac1Result<Ac1Creature> {
        let setup = self.decode(weenie.setup, Setup::decode)?;
        let appearance = self.appearance(weenie, &setup)?;
        let mut builder = ModelBuilder {
            dats: self,
            subpalettes: &appearance.subpalettes,
            materials: HashMap::new(),
            images: Vec::new(),
            next_texture: AC1_TEXTURE_BASE | ((visual.raw() & 0xFFF) << 11),
        };
        let rest = setup
            .placement(0)
            .map(<[Frame]>::to_vec)
            .unwrap_or_default();
        let mut parts = Vec::new();
        for (index, gfx) in appearance.parts.iter().enumerate() {
            let scale = setup.scales.get(index).copied().unwrap_or([1.0; 3]);
            let frame = rest.get(index).copied().unwrap_or(Frame::IDENTITY);
            if let Some(part) =
                builder.part(index, *gfx, scale, frame, &appearance.textures[index])?
            {
                parts.push(part);
            }
        }
        let bones = appearance.parts.len();
        let ac2_setup = Ac2Setup {
            meshes: Arc::from(vec![DataId::new(0); bones]),
            bones: Arc::from(vec![
                Ac2Bone {
                    parent: None,
                    name: 0
                };
                bones
            ]),
            placements: Arc::from(vec![Ac2PlacementFrame {
                kind: 0,
                bones: Arc::from(
                    (0..bones)
                        .map(|i| transform(rest.get(i).copied().unwrap_or(Frame::IDENTITY)))
                        .collect::<Vec<_>>(),
                ),
            }]),
            holding_points: Arc::from([]),
            connection_points: Arc::from([]),
            degrade: Ac2DegradeInfo::default(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        };
        let skeleton = ObjectSkeleton {
            groups: vec![SkeletonGroup::new(visual, &ac2_setup, false, None)],
        };
        let model = ObjectModel {
            visual,
            parts,
            degrade: None,
            skeleton: Some(Arc::new(skeleton)),
        };
        let motion_table = if weenie.motion_table != 0 {
            weenie.motion_table
        } else {
            setup.default_motion_table
        };
        let idle = (motion_table != 0)
            .then(|| self.idle_cycle(motion_table, bones))
            .transpose()?
            .flatten();
        Ok(Ac1Creature {
            weenie: weenie.clone(),
            visual,
            model,
            images: builder.images,
            idle,
        })
    }

    /// `WorldObject.CalculateObjDesc` (ACE) as the client applies it: the ClothingBase's
    /// object effects for this Setup, the PaletteTemplate's sub-palettes at the weenie's shade,
    /// then the weenie's own anim parts, texture maps and palettes.
    fn appearance(&self, weenie: &AceWeenie, setup: &Setup) -> Ac1Result<Appearance> {
        let mut parts = setup.parts.clone();
        let mut textures = vec![HashMap::new(); parts.len()];
        let mut subpalettes = Vec::new();
        if weenie.clothing_base != 0 {
            let table = self.decode(weenie.clothing_base, ClothingTable::decode)?;
            for effect in table.bases.get(&weenie.setup).into_iter().flatten() {
                let part = effect.part as usize;
                if let Some(slot) = parts.get_mut(part) {
                    *slot = effect.gfx_obj;
                    textures[part].extend(effect.textures.iter().copied());
                }
            }
            let template = weenie.palette_template.unwrap_or(0);
            for sub in table.templates.get(&template).into_iter().flatten() {
                let palette = self.shaded_palette(sub.palette_set, weenie.shade)?;
                for (start, count) in &sub.ranges {
                    subpalettes.push((palette, *start, *count));
                }
            }
        }
        for (part, gfx) in &weenie.anim_parts {
            if let Some(slot) = parts.get_mut(*part as usize) {
                *slot = *gfx;
            }
        }
        for (part, old, new) in &weenie.texture_maps {
            if let Some(map) = textures.get_mut(*part as usize) {
                map.insert(*old, *new);
            }
        }
        for (palette, offset, length) in &weenie.palettes {
            let palette = self.shaded_palette(*palette, weenie.shade)?;
            subpalettes.push((palette, offset * 8, length * 8));
        }
        Ok(Appearance {
            parts,
            textures,
            subpalettes,
        })
    }

    /// A palette, or a palette set's palette at `shade` (`PalSet::GetPaletteID`).
    fn shaded_palette(&self, id: u32, shade: f64) -> Ac1Result<u32> {
        if id >> 24 != PALETTE_SET {
            return Ok(id);
        }
        let set = self.decode(id, decode_palette_set)?;
        palette_for_shade(&set, shade).ok_or(Ac1Error {
            did: id,
            context: "the palette set has no palette at this shade",
        })
    }

    /// `CMotionTable::SetDefaultState`'s cycle as poses for `parts` parts.
    fn idle_cycle(&self, motion_table: u32, parts: usize) -> Ac1Result<Option<IdleCycle>> {
        let table = self.decode(motion_table, MotionTable::decode)?;
        let Some(anims) = table.idle_cycle() else {
            return Ok(None);
        };
        let mut frames = Vec::new();
        for data in anims {
            let animation = self.decode(data.animation, Animation::decode)?;
            let last = animation.frames.len() as i32 - 1;
            let high = if data.high_frame < 0 {
                last
            } else {
                data.high_frame.min(last)
            };
            let low = data.low_frame.clamp(0, high.max(0));
            let rate = f64::from(data.framerate.abs()).max(1.0);
            let mut range: Vec<i32> = (low..=high).collect();
            if data.framerate < 0.0 {
                range.reverse();
            }
            for index in range {
                let pose = (0..parts)
                    .map(|part| {
                        let frame = animation.frames[index as usize]
                            .get(part)
                            .copied()
                            .unwrap_or(Frame::IDENTITY);
                        part_pose(frame)
                    })
                    .collect();
                frames.push((pose, 1.0 / rate));
            }
        }
        let length = frames.iter().map(|(_, d)| d).sum::<f64>();
        Ok((!frames.is_empty() && length > 0.0).then_some(IdleCycle { frames, length }))
    }
}

/// A frame as the bone transform the renderer poses a part with.
fn transform(frame: Frame) -> Ac2Transform {
    let [w, x, y, z] = frame.rotation;
    let rotation = Quat::from_xyzw(x, y, z, w);
    Ac2Transform {
        offset: Vec3::from(frame.origin),
        scale: Vec3::ONE,
        rotation: if rotation.length_squared() > f32::EPSILON {
            rotation.normalize()
        } else {
            Quat::IDENTITY
        },
    }
}

fn part_pose(frame: Frame) -> PartPose {
    let t = transform(frame);
    PartPose {
        offset: t.offset.to_array(),
        rotation: [t.rotation.w, t.rotation.x, t.rotation.y, t.rotation.z],
        scale: [1.0; 3],
    }
}

struct ModelBuilder<'a> {
    dats: &'a Ac1Dats,
    subpalettes: &'a [(u32, u32, u32)],
    /// Each (surface, texture) pair's material and texture key.
    materials: HashMap<(u32, u32), Option<Arc<ObjectMaterial>>>,
    images: Images,
    next_texture: u32,
}

impl ModelBuilder<'_> {
    /// Part `index`: GfxObj `gfx` (its closest degrade), its vertices scaled by the Setup's
    /// default scale, at its rest frame.
    fn part(
        &mut self,
        index: usize,
        gfx: u32,
        scale: [f32; 3],
        frame: Frame,
        swaps: &HashMap<u32, u32>,
    ) -> Ac1Result<Option<ObjectPart>> {
        if gfx == 0 {
            return Ok(None);
        }
        let mut obj = self.dats.decode(gfx, GfxObj::decode)?;
        if let Some(degrade) = obj.degrade {
            let closest = self.dats.decode(degrade, decode_degrade_closest)?;
            if let Some(closest) = closest.filter(|id| *id != gfx) {
                obj = self.dats.decode(closest, GfxObj::decode)?;
            }
        }
        let mut vertices: Vec<ObjectVertex> = Vec::new();
        let mut corners: HashMap<(u16, u8, bool), u32> = HashMap::new();
        let mut lists: HashMap<i16, Vec<u32>> = HashMap::new();
        let scale = Vec3::from(scale);
        for polygon in &obj.polygons {
            let sides: &[(bool, i16, &[u8])] = match polygon.sides {
                0 => &[(false, polygon.pos_surface, &polygon.pos_uvs[..])],
                _ => &[
                    (false, polygon.pos_surface, &polygon.pos_uvs[..]),
                    (true, polygon.neg_surface, &polygon.neg_uvs[..]),
                ],
            };
            for &(back, surface, uvs) in sides {
                let mut ring = Vec::with_capacity(polygon.vertices.len());
                for (corner, id) in polygon.vertices.iter().enumerate() {
                    let uv = uvs.get(corner).copied().unwrap_or(0);
                    let key = (*id, uv, back);
                    let at = match corners.get(&key) {
                        Some(at) => *at,
                        None => {
                            let Some(v) = obj.vertices.get(id) else {
                                ring.clear();
                                break;
                            };
                            let normal = Vec3::from(v.normal) * if back { -1.0 } else { 1.0 };
                            let at = vertices.len() as u32;
                            let mut set = [[0.0; 2]; 4];
                            set[0] = v.uvs.get(uv as usize).copied().unwrap_or([0.0; 2]);
                            vertices.push(ObjectVertex {
                                position: (Vec3::from(v.origin) * scale).to_array(),
                                normal: normal.normalize_or_zero().to_array(),
                                color: [1.0; 4],
                                uvs: set,
                                tangents: [[0.0; 3]; 2],
                            });
                            corners.insert(key, at);
                            at
                        }
                    };
                    ring.push(at);
                }
                if ring.len() < 3 {
                    continue;
                }
                let list = lists.entry(surface).or_default();
                for i in 1..ring.len() - 1 {
                    list.extend(oriented(&vertices, [ring[0], ring[i], ring[i + 1]]));
                }
            }
        }
        let mut surfaces: Vec<_> = lists.into_iter().collect();
        surfaces.sort_by_key(|(surface, _)| *surface);
        let mut batches = Vec::new();
        for (surface, indices) in surfaces {
            let Some(did) = obj.surfaces.get(surface as usize).copied() else {
                continue;
            };
            if let Some(material) = self.material(did, swaps)? {
                batches.push(ObjectBatch {
                    indices: Arc::from(indices),
                    material,
                });
            }
        }
        if batches.is_empty() {
            return Ok(None);
        }
        Ok(Some(ObjectPart {
            mesh: DataId::new(gfx),
            transform: transform(frame).to_mat4(),
            vertices: Arc::from(vertices),
            batches,
            degrades: None,
            slot: (0, index),
            skin: None,
            mesh_type: 0,
        }))
    }

    /// Surface `did` as one material layer, its texture swapped by `swaps`.
    fn material(
        &mut self,
        did: u32,
        swaps: &HashMap<u32, u32>,
    ) -> Ac1Result<Option<Arc<ObjectMaterial>>> {
        let surface = self.dats.decode(did, Surface::decode)?;
        let texture = swaps
            .get(&surface.texture)
            .copied()
            .unwrap_or(surface.texture);
        if let Some(material) = self.materials.get(&(did, texture)) {
            return Ok(material.clone());
        }
        let image = if surface.flags & (st::BASE1_IMAGE | st::BASE1_CLIPMAP) != 0 {
            self.texture_image(&surface, texture)?
        } else if surface.flags & st::BASE1_SOLID != 0 {
            Some(solid(surface.color))
        } else {
            None
        };
        let material = image.map(|image| {
            let key = DataId::new(self.next_texture);
            self.next_texture += 1;
            self.images.push((TextureKey::Texture(key), Some(image)));
            Arc::new(ObjectMaterial::new(vec![layer(&surface, key)]))
        });
        self.materials.insert((did, texture), material.clone());
        Ok(material)
    }

    /// The largest `RenderSurface` of `SurfaceTexture` `texture`, an indexed one through the
    /// surface's palette with the object's sub-palettes copied over it.
    fn texture_image(
        &self,
        surface: &Surface,
        texture: u32,
    ) -> Ac1Result<Option<ac2_render::Image>> {
        let levels = self.dats.decode(texture, decode_surface_texture)?;
        let Some(level) = levels.first().copied() else {
            return Ok(None);
        };
        let Some(bytes) = self.dats.image(level)? else {
            return Ok(None);
        };
        let render = RenderSurface::decode(level, &bytes)?;
        let base = if render.palette != 0 {
            render.palette
        } else {
            surface.palette
        };
        let mut palette = if base != 0 {
            self.dats.decode(base, decode_palette)?
        } else {
            Vec::new()
        };
        for (id, start, count) in self.subpalettes {
            let colors = self.dats.decode(*id, decode_palette)?;
            for i in *start as usize..(*start + *count) as usize {
                if let (Some(slot), Some(color)) = (palette.get_mut(i), colors.get(i)) {
                    *slot = *color;
                }
            }
        }
        let clip = surface.flags & st::BASE1_CLIPMAP != 0;
        Ok(decode_rgba(&render, &palette, clip)
            .map(|rgba| mip_chain(render.width, render.height, rgba)))
    }
}

/// A triangle wound counter-clockwise seen from the side its vertex normals face.
fn oriented(vertices: &[ObjectVertex], [a, b, c]: [u32; 3]) -> [u32; 3] {
    let p = |i: u32| Vec3::from(vertices[i as usize].position);
    let n = |i: u32| Vec3::from(vertices[i as usize].normal);
    let face = (p(b) - p(a)).cross(p(c) - p(a));
    if face.dot(n(a) + n(b) + n(c)) < 0.0 {
        [a, c, b]
    } else {
        [a, b, c]
    }
}

/// `CSurface` as a material layer: its texture modulated by the lit diffuse; translucent and
/// alpha surfaces blend, additive ones add, clip maps alpha-test; luminous surfaces are unlit.
fn layer(surface: &Surface, texture: DataId) -> Ac2MaterialLayer {
    let flags = surface.flags;
    let translucent = flags & st::TRANSLUCENT != 0 || surface.translucency > 0.0002;
    let alpha = flags & (st::ALPHA | st::INV_ALPHA) != 0 || translucent;
    let additive = flags & st::ADDITIVE != 0;
    let (source_blend, dest_blend) = if additive {
        (blend::SRCALPHA, blend::ONE)
    } else if alpha {
        (blend::SRCALPHA, blend::INVSRCALPHA)
    } else {
        (blend::ONE, blend::ZERO)
    };
    let opacity = if translucent {
        (1.0 - surface.translucency).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let clip = flags & st::BASE1_CLIPMAP != 0;
    Ac2MaterialLayer {
        true_flags: 0,
        false_flags: 0,
        stages: vec![Ac2LayerStage {
            texture: Some(texture),
            color_op: d3d::TOP_MODULATE,
            color_arg1: d3d::TA_TEXTURE,
            color_arg2: d3d::TA_DIFFUSE,
            alpha_op: if alpha {
                d3d::TOP_MODULATE
            } else {
                d3d::TOP_SELECTARG1
            },
            alpha_arg1: d3d::TA_TEXTURE,
            alpha_arg2: d3d::TA_DIFFUSE,
            address_u: d3d::TADDRESS_WRAP,
            address_v: d3d::TADDRESS_WRAP,
            min_filter: d3d::TEXF_LINEAR,
            mag_filter: d3d::TEXF_LINEAR,
            tex_coord: 0,
            use_projection: false,
            special_texture: 0,
        }],
        modifiers: Vec::new(),
        source_blend,
        dest_blend,
        depth_test: d3d::CMP_LESSEQUAL,
        depth_write: u32::from(!(alpha || additive)),
        cull_mode: d3d::CULL_CW,
        alpha_test_mode: u32::from(clip),
        alpha_test_ref: Ac2Waveform::constant(0.5),
        lighting: Ac2LayerLighting {
            dynamic: flags & st::LUMINOUS == 0 && surface.luminosity <= 0.0,
            material_diffuse: true,
            depth_fog: true,
            ..Ac2LayerLighting::default()
        },
        ambient: [1.0; 4],
        diffuse: [1.0, 1.0, 1.0, opacity],
        specular: [0.0; 4],
        specular_power: Ac2Waveform::constant(0.0),
        options: 0,
        vertex_format: 0,
        vertex_shader: Arc::from([]),
        pixel_shader: Arc::from([]),
    }
}

/// The AC1 creatures the scene draws, by visual DID (`ForeignModels`).
#[derive(Debug, Default)]
pub struct Ac1Models {
    models: HashMap<DataId, (ObjectModel, Vec3, Images)>,
}

impl Ac1Models {
    pub fn insert(&mut self, creature: &Ac1Creature) {
        self.models.insert(
            creature.visual,
            (creature.model.clone(), Vec3::ONE, creature.images.clone()),
        );
    }
}

impl ForeignModels for Ac1Models {
    fn object_model(&self, visual: DataId, _settings: &RenderSettings) -> Option<ForeignModel> {
        let (model, scale, images) = self.models.get(&visual)?;
        Some(ForeignModel {
            model: Some(model.clone()),
            scale: *scale,
            images: images.clone(),
        })
    }
}

/// A placement matrix's translation, for tests.
#[cfg(test)]
fn origin(m: &ac2_math::Mat4) -> Vec3 {
    m.w_axis.truncate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_triangle_turns_counter_clockwise_towards_its_normals() {
        let vertex = |x: f32, y: f32| ObjectVertex {
            position: [x, y, 0.0],
            normal: [0.0, 0.0, 1.0],
            color: [1.0; 4],
            uvs: [[0.0; 2]; 4],
            tangents: [[0.0; 3]; 2],
        };
        // Counter-clockwise seen from +z, where the normals face.
        let vertices = [vertex(0.0, 0.0), vertex(1.0, 0.0), vertex(0.0, 1.0)];
        assert_eq!(oriented(&vertices, [0, 1, 2]), [0, 1, 2]);
        assert_eq!(oriented(&vertices, [0, 2, 1]), [0, 1, 2]);
    }

    #[test]
    fn the_idle_cycle_holds_each_frame_and_loops() {
        let pose = |x: f32| {
            vec![PartPose {
                offset: [x, 0.0, 0.0],
                rotation: [1.0, 0.0, 0.0, 0.0],
                scale: [1.0; 3],
            }]
        };
        let cycle = IdleCycle {
            frames: vec![(pose(0.0), 0.1), (pose(1.0), 0.1)],
            length: 0.2,
        };
        assert_eq!(cycle.pose(0.05)[0].offset[0], 0.0);
        assert_eq!(cycle.pose(0.15)[0].offset[0], 1.0);
        assert_eq!(cycle.pose(0.25)[0].offset[0], 0.0);
        assert_eq!(origin(&transform(Frame::IDENTITY).to_mat4()), Vec3::ZERO);
    }
}
