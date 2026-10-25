//! `ParticleEmitter::FillVertexBuffer` and `FillVerts`: each particle becomes a camera-facing
//! quad (or triangle), stretched along its streak, turned by its rotation, coloured by its
//! keyframes and the emitter's fade, as vertices the object pass draws.
//!
//! Retail builds the corners in view space (x right, y depth, z up) and the renderer carries
//! them back to the world by the inverse view matrix; here the corners are built in that basis
//! and placed in the world straight away.

use ac2_assets::Ac2ParticleSystemDesc;
use ac2_math::Vec3;

use super::emitter::Emitter;
use super::keyframes::{blend, flag, Particle};
use super::EPSILON;
use crate::object::ObjectVertex;

/// `QUAD_U` and `QUAD_V`: the corners' texture coordinates, top left first.
const QUAD_UV: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

/// The camera as the particles see it: its place and the view-space axes in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraBasis {
    pub eye: Vec3,
    pub right: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

impl CameraBasis {
    /// The basis of a camera at `eye` looking at `target`, with the world's z up.
    pub fn look_at(eye: Vec3, target: Vec3) -> Option<Self> {
        let forward = (target - eye).try_normalize()?;
        let right = forward.cross(Vec3::Z).try_normalize().unwrap_or(Vec3::X);
        Some(Self {
            eye,
            right,
            forward,
            up: right.cross(forward),
        })
    }

    /// A world position in view space: x right, y depth, z up.
    fn view_of(&self, world: Vec3) -> Vec3 {
        let d = world - self.eye;
        Vec3::new(d.dot(self.right), d.dot(self.forward), d.dot(self.up))
    }

    /// A view-space position in the world.
    fn world_of(&self, view: Vec3) -> Vec3 {
        self.eye + self.right * view.x + self.forward * view.y + self.up * view.z
    }
}

/// The vertices and triangle list of a system's particles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParticleGeometry {
    pub vertices: Vec<ObjectVertex>,
    pub indices: Vec<u32>,
}

impl ParticleGeometry {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// What the fill reads from the system.
pub(crate) struct FillContext<'a> {
    pub basis: &'a CameraBasis,
    pub now: f64,
    /// The system's distance fade.
    pub alpha: f32,
    /// The scale the particles are drawn at (`RenderAtPosition`'s `vScale`).
    pub scale: Vec3,
    pub system_start: f64,
    /// Where a system that keeps its particles relative to itself stands.
    pub placement: Option<Vec3>,
}

/// `FillVerts::GetVertexPositions`: the quad's corners about `position` (view space).
fn quad(position: Vec3, scale_x: f32, scale_y: f32) -> [Vec3; 4] {
    let (hx, hy) = (scale_x * 0.5, scale_y * 0.5);
    [
        Vec3::new(position.x - hx, position.y, position.z + hy),
        Vec3::new(position.x - hx, position.y, position.z - hy),
        Vec3::new(position.x + hx, position.y, position.z - hy),
        Vec3::new(position.x + hx, position.y, position.z + hy),
    ]
}

/// `FillVerts::GetStreakedVertexPositions`: the quad stretched from the particle to its streak
/// origin (both in view space), its corners picked by the streak's direction on the screen.
fn streaked(position: Vec3, streak: Vec3, scale_x: f32, scale_y: f32) -> [Vec3; 4] {
    let (hx, hy) = (scale_x * 0.5, scale_y * 0.5);
    let (dx, dz) = (position.x - streak.x, position.z - streak.z);
    let length = (dx * dx + dz * dz).sqrt();
    if length < EPSILON {
        return quad(position, scale_x, scale_y);
    }
    let (ux, uz) = (dx / length, dz / length);
    let at = |origin: Vec3, x: f32, z: f32| Vec3::new(origin.x + x, origin.y, origin.z + z);
    let ax = ux.abs();
    if ax > 0.923_879_5 || ax < 0.382_683_4 {
        if ax <= uz.abs() {
            if uz > 0.0 {
                [
                    at(position, hx, hy),
                    at(position, -hx, hy),
                    at(streak, -hx, -hy),
                    at(streak, hx, -hy),
                ]
            } else {
                [
                    at(position, hx, -hy),
                    at(position, -hx, -hy),
                    at(streak, -hx, hy),
                    at(streak, hx, hy),
                ]
            }
        } else if ux <= 0.0 {
            [
                at(position, -hx, hy),
                at(position, -hx, -hy),
                at(streak, hx, -hy),
                at(streak, hx, hy),
            ]
        } else {
            [
                at(position, hx, hy),
                at(position, hx, -hy),
                at(streak, -hx, -hy),
                at(streak, -hx, hy),
            ]
        }
    } else {
        // A diagonal streak: the signs of its two components, zero counting as one class.
        let class = |u: f32| {
            if u.abs() < EPSILON {
                0
            } else if u <= 0.0 {
                2
            } else {
                1
            }
        };
        if class(ux) == class(uz) {
            [
                at(position, hx, -hy),
                at(position, -hx, hy),
                at(streak, -hx, hy),
                at(streak, hx, -hy),
            ]
        } else {
            [
                at(position, hx, hy),
                at(position, -hx, -hy),
                at(streak, -hx, -hy),
                at(streak, hx, hy),
            ]
        }
    }
}

/// `Matrix4::RotateAngles`' basis for `angles`: where the x, y and z axes go.
fn rotation_basis(angles: Vec3) -> [Vec3; 3] {
    let (sx, cx) = (-angles.x).sin_cos();
    let (sy, cy) = (-angles.y).sin_cos();
    let (sz, cz) = (-angles.z).sin_cos();
    [
        Vec3::new(cz * cy, sy, -sz * cy),
        Vec3::new(sz * sx - cz * sy * cx, cy * cx, cz * sx + sy * sz * cx),
        Vec3::new(sz * cx + cz * sy * sx, -cy * sx, cz * cx - sy * sz * sx),
    ]
}

/// `FillVerts::RotateVertex`: `v` turned by the basis.
fn turn(basis: &[Vec3; 3], v: Vec3) -> Vec3 {
    basis[0] * v.x + basis[1] * v.y + basis[2] * v.z
}

impl Emitter {
    /// The emitter's fade in and out (`FillVertexBuffer`'s `fEmitterOpacity`).
    fn opacity(&self, system: &Ac2ParticleSystemDesc, context: &FillContext<'_>) -> f32 {
        let desc = &system.emitters[self.index];
        let fast_forward = system.fast_forward_time;
        let fade_in = fast_forward + desc.fade_in;
        let fade_out = fast_forward + desc.fade_out;
        let t = (context.now - (f64::from(desc.start_time) + context.system_start)) as f32;
        let mut opacity = 1.0;
        if fade_in > EPSILON && t > fast_forward && t < fade_in {
            opacity = if (fade_in - fast_forward).abs() >= EPSILON {
                ((t - fast_forward) / (fade_in - fast_forward)).clamp(0.0, 1.0)
            } else {
                0.0
            };
        }
        if desc.time_limit > EPSILON && fade_out > EPSILON && fade_out < t {
            let span = desc.time_limit - fade_out;
            opacity = if span.abs() < EPSILON {
                1.0
            } else {
                (1.0 - (t - fade_out) / span).clamp(0.0, 1.0)
            };
        }
        opacity
    }

    /// The particle's size and colour from its keyframes.
    fn look(
        &self,
        desc: &ac2_assets::Ac2ParticleEmitterDesc,
        p: &Particle,
        time_alive: f32,
        object_scale: f32,
        opacity: f32,
    ) -> ([f32; 2], [f32; 4]) {
        let (mut size, mut color) = ([1.0_f32, 1.0], [1.0_f32; 4]);
        if self.flags & 0x82 != 0 {
            if let Some(last) = p.key.last[0] {
                let from = &desc.keyframes[self.keys[last].desc];
                let to = &desc.keyframes[self.keys[self.keys[last].next[0]].desc];
                size = if to.key_flags & 1 == 0 {
                    [from.scale_x, from.scale_y]
                } else {
                    let f = blend(from.time, to.time, time_alive);
                    [
                        f * to.scale_x + (1.0 - f) * from.scale_x,
                        f * to.scale_y + (1.0 - f) * from.scale_y,
                    ]
                };
            }
            size = [size[0] * object_scale, size[1] * object_scale];
            if self.flags & flag::SCALE != 0 {
                size = [size[0] * p.scale, size[1] * p.scale];
            }
            if let Some(last) = p.key.last[2] {
                let from = &desc.keyframes[self.keys[last].desc];
                let to = &desc.keyframes[self.keys[self.keys[last].next[2]].desc];
                color = if to.key_flags & 2 == 0 {
                    from.color
                } else {
                    let f = blend(from.time, to.time, time_alive);
                    std::array::from_fn(|i| to.color[i] * f + from.color[i] * (1.0 - f))
                };
            }
            color[3] *= opacity;
        } else {
            size = [size[0] * object_scale, size[1] * object_scale];
        }
        (size, color)
    }

    /// Appends this emitter's particles as quads or triangles.
    pub(crate) fn fill(
        &self,
        system: &Ac2ParticleSystemDesc,
        context: &FillContext<'_>,
        out: &mut ParticleGeometry,
    ) {
        let desc = &system.emitters[self.index];
        let corners = match desc.particle_shape {
            3 => 3,
            4 => 4,
            _ => return,
        };
        let basis = context.basis;
        let opacity = self.opacity(system, context);
        let blend_alpha = ((context.now - self.last_think) / f64::from(system.physics_time_step))
            .clamp(0.0, 1.0) as f32;
        let object_scale = context.scale.max_element();
        let normal = -basis.forward;
        for p in self.particles() {
            // Where the particle is, blended from its previous position when it keeps one.
            let mut origin = if self.flags & flag::PREVIOUS_ORIGIN != 0 {
                p.previous_origin.lerp(p.origin, blend_alpha)
            } else {
                p.origin
            };
            let streak_origin = if self.flags & flag::STREAK_ORIGIN == 0 {
                Vec3::ZERO
            } else if self.flags & flag::PREVIOUS_STREAK_ORIGIN != 0 {
                p.previous_streak_origin.lerp(p.streak_origin, blend_alpha)
            } else {
                p.streak_origin
            };
            let time_alive = (context.now - p.time_created) as f32;
            let (size, mut color) = self.look(desc, p, time_alive, object_scale, opacity);
            if self.flags & flag::WORLD_ORIGIN != 0 {
                origin = (origin - p.world_origin) * context.scale + p.world_origin;
            }
            // A system that keeps its particles about itself is placed by translation alone.
            let place = context.placement.unwrap_or(Vec3::ZERO);
            let origin = origin
                * if context.placement.is_some() {
                    context.scale
                } else {
                    Vec3::ONE
                }
                + place;
            let streak_origin = streak_origin
                * if context.placement.is_some() {
                    context.scale
                } else {
                    Vec3::ONE
                }
                + place;
            let center = basis.view_of(origin);
            color[3] *= context.alpha;
            let color = color.map(|c| c.clamp(0.0, 1.0));
            let world = if self.flags & flag::STREAK != 0 {
                let streak = if self.flags & flag::WORLD_ROTATION != 0 {
                    streak_origin - origin
                } else {
                    basis.view_of(streak_origin)
                };
                let view = streaked(center, streak, size[0], size[1]);
                view.map(|v| basis.world_of(v))
            } else if self.flags & flag::WORLD_ROTATION != 0 {
                let turned = rotation_basis(p.world_rotation);
                quad(Vec3::ZERO, size[0], size[1]).map(|v| origin + turn(&turned, v))
            } else if self.flags & flag::ROTATION != 0 {
                let turned = rotation_basis(p.rotation);
                quad(Vec3::ZERO, size[0], size[1])
                    .map(|v| basis.world_of(center + turn(&turned, v)))
            } else {
                quad(center, size[0], size[1]).map(|v| basis.world_of(v))
            };
            let first = out.vertices.len() as u32;
            for (corner, position) in world.iter().take(corners).enumerate() {
                out.vertices.push(ObjectVertex {
                    position: position.to_array(),
                    normal: normal.to_array(),
                    color,
                    uvs: [QUAD_UV[corner], [0.0; 2], [0.0; 2], [0.0; 2]],
                    tangents: [[0.0; 3]; 2],
                });
            }
            out.indices
                .extend_from_slice(&[first, first + 1, first + 2]);
            if corners == 4 {
                out.indices
                    .extend_from_slice(&[first, first + 2, first + 3]);
            }
        }
    }
}
