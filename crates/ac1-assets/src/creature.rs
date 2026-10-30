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
    palette_for_shade, surface_type as st, Ac1Error, Ac1Result, ClothingTable, Frame, GfxObj,
    RenderSurface, Setup, Surface,
};
use crate::idle::IdleBehaviour;
use crate::image::{decode_rgba, mip_chain, solid};
use crate::weenie::AceWeenie;

/// The visual DIDs the AC1 creatures are drawn under: a range no AC2 DAT uses.
pub const AC1_VISUAL_BASE: u32 = 0xAC00_0000;
/// The texture DIDs their images are keyed by.
const AC1_TEXTURE_BASE: u32 = 0xAC80_0000;
const PALETTE_SET: u32 = 0x0F;
/// AC1's sky over the AC2 one an AC1 creature stands under, as material light scales (the
/// fixed-function device multiplies the ambient light by the material ambient and each light by
/// the material diffuse). The AC1 Region (`0x13000000`) lights noon with a sun of 0.8 and an
/// ambient of 0.35 times its (0.9, 0.9, 1.0) colour; AC2's Dereth region with 0.65 and 0.25.
const AC1_AMBIENT_SCALE: f32 = 1.3;
const AC1_SUN_SCALE: f32 = 1.25;

/// D3D values of the one layer an AC1 surface draws as.
mod d3d {
    pub const TOP_MODULATE: u32 = 4;
    pub const TOP_SELECTARG1: u32 = 2;
    pub const TA_DIFFUSE: u32 = 0;
    pub const TA_CURRENT: u32 = 1;
    pub const TA_TEXTURE: u32 = 2;
    pub const TA_ALPHAREPLICATE: u32 = 0x20;
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
    /// The idle cycle and heartbeat actions, when the motion table has a default cycle.
    pub idle: Option<IdleBehaviour>,
    /// The Setup's `default_script` (`PhysicsScript`): its glow and particles; 0 for none.
    pub script: u32,
    /// The icon ACE gives it (`CalculateObjDesc`: the ClothingBase template's icon when it has
    /// one, else `PropertyDataId.Icon`); 0 for none.
    pub icon: u32,
}

/// The part changes and palette an `ObjDesc` makes of a weenie.
struct Appearance {
    parts: Vec<u32>,
    /// `SurfaceTexture` swaps per part.
    textures: Vec<HashMap<u32, u32>>,
    /// The `ObjDesc` palette (`PaletteBase` with the sub-palettes copied over it) that replaces
    /// every indexed surface's own (`CPartArray::SetPalette`); `None` without a base palette or
    /// sub-palettes, when each surface keeps its own.
    palette: Option<Vec<u32>>,
    /// The palette template's icon, when it has one.
    icon: Option<u32>,
}

impl Ac1Dats {
    /// Builds `weenie` drawn under `visual`.
    pub fn creature(&self, weenie: &AceWeenie, visual: DataId) -> Ac1Result<Ac1Creature> {
        let setup = self.decode(weenie.setup, Setup::decode)?;
        let appearance = self.appearance(weenie, &setup)?;
        let mut builder = ModelBuilder {
            dats: self,
            palette: appearance.palette.as_deref(),
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
        let placements: Vec<(u32, &[Frame])> = std::iter::once((0, &rest[..]))
            .chain(
                setup
                    .placements
                    .iter()
                    .filter(|(key, _)| *key != 0)
                    .map(|(key, frames)| (*key, &frames[..])),
            )
            .collect();
        let model = rigid_model(visual, parts, bones, &placements);
        let motion_table = if weenie.motion_table != 0 {
            weenie.motion_table
        } else {
            setup.default_motion_table
        };
        let idle = (motion_table != 0)
            .then(|| self.idle_behaviour(weenie, motion_table, bones))
            .transpose()?
            .flatten();
        Ok(Ac1Creature {
            weenie: weenie.clone(),
            visual,
            model,
            images: builder.images,
            idle,
            script: setup.default_script,
            icon: appearance.icon.unwrap_or(weenie.icon),
        })
    }

    /// Icon `did` (a `RenderSurface`) as RGBA: its width, height and pixels.
    pub fn icon_rgba(&self, did: u32) -> Ac1Result<Option<(u32, u32, Vec<u8>)>> {
        let Some(bytes) = self.image(did)? else {
            return Ok(None);
        };
        let render = RenderSurface::decode(did, &bytes)?;
        let palette = if render.palette != 0 {
            self.decode(render.palette, decode_palette)?
        } else {
            Vec::new()
        };
        Ok(decode_rgba(&render, &palette, false).map(|rgba| (render.width, render.height, rgba)))
    }

    /// A particle's GfxObj `gfx` as a model drawn under `visual`: one part at the origin, its
    /// surfaces' own palettes.
    pub fn particle_model(&self, gfx: u32, visual: DataId) -> Ac1Result<(ObjectModel, Images)> {
        let mut builder = ModelBuilder {
            dats: self,
            palette: None,
            materials: HashMap::new(),
            images: Vec::new(),
            next_texture: AC1_TEXTURE_BASE | ((visual.raw() & 0xFFF) << 11),
        };
        let parts = builder
            .part(0, gfx, [1.0; 3], Frame::IDENTITY, &HashMap::new())?
            .into_iter()
            .collect();
        let model = rigid_model(visual, parts, 1, &[(0, &[Frame::IDENTITY][..])]);
        Ok((model, builder.images))
    }

    /// `WorldObject.CalculateObjDesc` (ACE) as the client applies it: the ClothingBase's
    /// object effects for this Setup, the PaletteTemplate's sub-palettes at the weenie's shade,
    /// then the weenie's own anim parts, texture maps and palettes; the `ObjDesc` palette is the
    /// weenie's `PaletteBase` with every sub-palette's ranges copied over it, in order.
    fn appearance(&self, weenie: &AceWeenie, setup: &Setup) -> Ac1Result<Appearance> {
        let mut parts = setup.parts.clone();
        let mut textures = vec![HashMap::new(); parts.len()];
        let mut subpalettes = Vec::new();
        let mut icon = None;
        if weenie.clothing_base != 0 {
            let table = self.decode(weenie.clothing_base, ClothingTable::decode)?;
            for effect in table.bases.get(&weenie.setup).into_iter().flatten() {
                let part = effect.part as usize;
                if let Some(slot) = parts.get_mut(part) {
                    *slot = effect.gfx_obj;
                    textures[part].extend(effect.textures.iter().copied());
                }
            }
            // ACE (`CalculateObjDesc`) applies the sub-palettes only when the weenie has a shade or
            // a template, taking the table's first template when the weenie's is not in it.
            icon = weenie
                .palette_template
                .and_then(|t| table.template_icons.get(&t))
                .copied()
                .filter(|icon| *icon != 0);
            let template = weenie
                .palette_template
                .filter(|t| table.templates.contains_key(t))
                .or(table.template_order.first().copied());
            let subs = (weenie.palette_template.is_some() || weenie.has_shade)
                .then(|| template.and_then(|t| table.templates.get(&t)))
                .flatten();
            for sub in subs.into_iter().flatten() {
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
            // The wire's length byte 0 is the whole palette (`Subpalette::UnPack`).
            let length = if *length == 0 { 256 } else { *length };
            subpalettes.push((palette, offset * 8, length * 8));
        }
        let palette = if weenie.palette_base != 0 && !subpalettes.is_empty() {
            let mut base = self.decode(weenie.palette_base, decode_palette)?;
            for (id, start, count) in &subpalettes {
                let colors = self.decode(*id, decode_palette)?;
                for i in *start as usize..(*start + *count) as usize {
                    if let (Some(slot), Some(color)) = (base.get_mut(i), colors.get(i)) {
                        *slot = *color;
                    }
                }
            }
            Some(base)
        } else {
            None
        };
        Ok(Appearance {
            parts,
            textures,
            palette,
            icon,
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
}

/// A model of rigid parts, one bone per part (AC1 poses every part in object space), with the
/// placement frames `placements` by AC1 `Placement` key (0 the rest frame).
fn rigid_model(
    visual: DataId,
    parts: Vec<ObjectPart>,
    bones: usize,
    placements: &[(u32, &[Frame])],
) -> ObjectModel {
    let ac2_setup = Ac2Setup {
        meshes: Arc::from(vec![DataId::new(0); bones]),
        bones: Arc::from(vec![
            Ac2Bone {
                parent: None,
                name: 0
            };
            bones
        ]),
        placements: Arc::from(
            placements
                .iter()
                .map(|(kind, frames)| Ac2PlacementFrame {
                    kind: *kind,
                    bones: Arc::from(
                        (0..bones)
                            .map(|i| transform(frames.get(i).copied().unwrap_or(Frame::IDENTITY)))
                            .collect::<Vec<_>>(),
                    ),
                })
                .collect::<Vec<_>>(),
        ),
        holding_points: Arc::from([]),
        connection_points: Arc::from([]),
        degrade: Ac2DegradeInfo::default(),
        bounds_min: Vec3::ZERO,
        bounds_max: Vec3::ZERO,
    };
    ObjectModel {
        visual,
        parts,
        degrade: None,
        skeleton: Some(Arc::new(ObjectSkeleton {
            groups: vec![SkeletonGroup::new(visual, &ac2_setup, false, None)],
        })),
    }
}

/// What turns an AC1 object placed for an AC1 hand (its Setup's hand placement frame) into the
/// axes an AC2 hand holds by: an AC2 weapon's blade runs along its +y with the guard across x,
/// an AC1 weapon's (after `RightHandCombat`) along +z with the guard across y. The AC1 axes
/// x, y, z become the AC2 z, x, y: a turn of -120 degrees about (1, 1, 1). A held AC1 object
/// takes it as its offset in the holding location (`CHILDINFO::m_offset`).
pub const AC1_HAND_TO_AC2: Quat = Quat::from_xyzw(-0.5, -0.5, -0.5, 0.5);

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

pub(crate) fn part_pose(frame: Frame) -> PartPose {
    let t = transform(frame);
    PartPose {
        offset: t.offset.to_array(),
        rotation: [t.rotation.w, t.rotation.x, t.rotation.y, t.rotation.z],
        scale: [1.0; 3],
    }
}

struct ModelBuilder<'a> {
    dats: &'a Ac1Dats,
    /// The `ObjDesc` palette (`Appearance::palette`).
    palette: Option<&'a [u32]>,
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
    /// object's `ObjDesc` palette, else its own (`CSurface::orig_palette_id`, else the
    /// `RenderSurface`'s default).
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
        let own;
        let palette = match self.palette {
            Some(palette) => palette,
            None => {
                let authored = if surface.palette != 0 {
                    surface.palette
                } else {
                    render.palette
                };
                own = if authored != 0 {
                    self.dats.decode(authored, decode_palette)?
                } else {
                    Vec::new()
                };
                &own[..]
            }
        };
        let clip = surface.flags & st::BASE1_CLIPMAP != 0;
        Ok(decode_rgba(&render, palette, clip)
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
/// alpha surfaces blend, clip maps alpha-test; luminous surfaces are unlit, their colour the
/// texel times the material diffuse (which a tint fades). Additive surfaces (glows, flames, the
/// particles) add the texel premultiplied by its alpha (`ONE`, `ONE`: what retail's
/// `SRCALPHA`, `ONE` adds) and write no depth, so they cast no sun shadow.
fn layer(surface: &Surface, texture: DataId) -> Ac2MaterialLayer {
    let flags = surface.flags;
    let translucent = flags & st::TRANSLUCENT != 0 || surface.translucency > 0.0002;
    let alpha = flags & (st::ALPHA | st::INV_ALPHA) != 0 || translucent;
    let additive = flags & st::ADDITIVE != 0;
    let lit = flags & st::LUMINOUS == 0 && surface.luminosity <= 0.0;
    let (source_blend, dest_blend) = if additive {
        (blend::ONE, blend::ONE)
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
    let stage = |color_op, color_arg1, color_arg2, texture| Ac2LayerStage {
        texture,
        color_op,
        color_arg1,
        color_arg2,
        alpha_op: if alpha || additive {
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
    };
    let mut stages = vec![if lit {
        // The texel itself: the light multiplies it in `display_lit`.
        stage(
            d3d::TOP_SELECTARG1,
            d3d::TA_TEXTURE,
            d3d::TA_DIFFUSE,
            Some(texture),
        )
    } else {
        stage(
            d3d::TOP_MODULATE,
            d3d::TA_TEXTURE,
            d3d::TA_DIFFUSE,
            Some(texture),
        )
    }];
    if additive {
        let mut premultiply = stage(
            d3d::TOP_MODULATE,
            d3d::TA_CURRENT,
            d3d::TA_CURRENT | d3d::TA_ALPHAREPLICATE,
            None,
        );
        premultiply.alpha_op = d3d::TOP_SELECTARG1;
        premultiply.alpha_arg1 = d3d::TA_CURRENT;
        stages.push(premultiply);
    }
    let light = if lit { AC1_SUN_SCALE } else { 1.0 };
    Ac2MaterialLayer {
        true_flags: 0,
        false_flags: 0,
        stages,
        modifiers: Vec::new(),
        source_blend,
        dest_blend,
        depth_test: d3d::CMP_LESSEQUAL,
        depth_write: u32::from(!(alpha || additive)),
        cull_mode: d3d::CULL_CW,
        alpha_test_mode: u32::from(clip),
        alpha_test_ref: Ac2Waveform::constant(0.5),
        lighting: Ac2LayerLighting {
            dynamic: lit,
            material_diffuse: true,
            depth_fog: true,
            display_lit: true,
            display_glow: additive,
            ..Ac2LayerLighting::default()
        },
        ambient: [AC1_AMBIENT_SCALE, AC1_AMBIENT_SCALE, AC1_AMBIENT_SCALE, 1.0],
        diffuse: [light, light, light, opacity],
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
    /// A model drawn under `visual` (a particle's).
    pub fn insert_model(&mut self, visual: DataId, model: ObjectModel, images: Images) {
        self.models.insert(visual, (model, Vec3::ONE, images));
    }

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
    fn an_identity_frame_places_a_part_at_the_origin() {
        assert_eq!(origin(&transform(Frame::IDENTITY).to_mat4()), Vec3::ZERO);
    }
}
