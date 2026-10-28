//! Decal FX: the projected decals `FXManager::DoDecalFX` makes (`DecalEffectManager::CreateDecalEffect`),
//! as the scene holds them. A decal is a box centred on its origin, its tangent along the decal's
//! direction, its normal up (`DecalEffect::ConstructLandscapeDecal`). The ground under the box is
//! clipped against its six faces (`DecalEffect::ConstructDecal`, `DecalEffect::ClipPolygonAgainstPlane`)
//! and the pieces that survive become the decal's triangles, lifted 0.015 m off the ground, with
//! texture coordinates over the box. A decal fades in and out over its lifespan (`DecalEffect::Think`),
//! its colour is the node's colour at that opacity (`DecalEffect::CreateGeometry`), and the decals
//! of one material draw as one geometry (`DecalEffectGroup::Render`).
//!
//! Positions are in the scene's world space (AC axes: x east, y north, z up). The triangles are
//! built once, when the decal starts; each frame only its opacity changes.

use ac2_core::DataId;
use ac2_math::Vec3;

use crate::object::ObjectVertex;
use crate::particles::ParticleGeometry;

/// The lift off the ground each decal triangle gets (`DecalEffect::ConstructDecal`, `+ 0.015`).
pub const DECAL_LIFT: f32 = 0.015;

/// A decal group draws only while its triangles' vertices stay under this count
/// (`DecalEffectGroup::Render`: `nNumTotalTriangles * 3 < 0x2000`).
pub const MAX_GROUP_VERTICES: usize = 0x2000;

/// A ground triangle's corner: its world position and normal (`LScapeVertex`'s origin and normal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// One triangle of the ground a decal may land on.
pub type GroundTriangle = [GroundVertex; 3];

/// A decal triangle's corner (`DecalTriangle`'s origin, normal and texture coordinate).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecalCorner {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// One decal triangle, three corners.
pub type DecalTriangle = [DecalCorner; 3];

/// What `FXManager::DoDecalFX` hands `CreateDecalEffect` (`DecalEffectDesc`): the material, the
/// lifespan, the box edge (the same on all three axes), the tangent direction, the colour, whether
/// the decal draws once, and the decal's origin in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecalSpec {
    pub material: DataId,
    pub lifespan: f32,
    pub size: f32,
    pub direction: Vec3,
    pub color: [f32; 4],
    pub draw_once: bool,
    pub origin: Vec3,
}

/// One face of the decal's box: `n . p + d >= 0` is inside (`Plane`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Plane {
    n: [f32; 3],
    d: f32,
}

impl Plane {
    /// The signed distance of `p`, evaluated in f64 as the x87 stack did, then compared at zero.
    fn distance(&self, p: [f32; 3]) -> f64 {
        f64::from(self.n[0]) * f64::from(p[0])
            + f64::from(self.n[1]) * f64::from(p[1])
            + f64::from(self.n[2]) * f64::from(p[2])
            + f64::from(self.d)
    }
}

/// A clip-polygon corner: position and normal, both interpolated along a clipped edge.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Corner {
    position: [f32; 3],
    normal: [f32; 3],
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f64 {
    f64::from(a[0]) * f64::from(b[0])
        + f64::from(a[1]) * f64::from(b[1])
        + f64::from(a[2]) * f64::from(b[2])
}

/// `N x T`, the decal's bitangent (`ConstructDecal`'s `fVar3..fVar5`), stored as floats.
fn cross(n: [f32; 3], t: [f32; 3]) -> [f32; 3] {
    let (n, t) = (n.map(f64::from), t.map(f64::from));
    [
        (n[1] * t[2] - n[2] * t[1]) as f32,
        (n[2] * t[0] - n[0] * t[2]) as f32,
        (n[0] * t[1] - n[1] * t[0]) as f32,
    ]
}

fn negate(v: [f32; 3]) -> [f32; 3] {
    v.map(|c| -c)
}

/// The point where the edge from `prev` to `cur` crosses the plane: `prev + s (cur - prev)` with
/// `s = d_prev / (d_prev - d_cur)`. Positions and normals interpolate the same way.
fn crossing(prev: &Corner, cur: &Corner, d_prev: f64, d_cur: f64) -> Corner {
    let s = d_prev / (d_prev - d_cur);
    let lerp = |a: [f32; 3], b: [f32; 3]| {
        let mut out = [0.0_f32; 3];
        for k in 0..3 {
            let (a, b) = (f64::from(a[k]), f64::from(b[k]));
            out[k] = (a + s * (b - a)) as f32;
        }
        out
    };
    Corner {
        position: lerp(prev.position, cur.position),
        normal: lerp(prev.normal, cur.normal),
    }
}

/// `DecalEffect::ClipPolygonAgainstPlane`: the polygon clipped to the inside of `plane` (Sutherland
/// Hodgman, the inside test `distance >= 0`). A polygon with no inside corner is gone.
fn clip_polygon(plane: &Plane, poly: &[Corner]) -> Vec<Corner> {
    let n = poly.len();
    let dist: Vec<f64> = poly.iter().map(|c| plane.distance(c.position)).collect();
    if n == 0 || dist.iter().all(|d| *d < 0.0) {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n + 2);
    for cur in 0..n {
        let prev = (cur + n - 1) % n;
        let (d_cur, d_prev) = (dist[cur], dist[prev]);
        match (d_cur >= 0.0, d_prev >= 0.0) {
            (true, true) => out.push(poly[cur]),
            (true, false) => {
                out.push(crossing(&poly[prev], &poly[cur], d_prev, d_cur));
                out.push(poly[cur]);
            }
            (false, true) => out.push(crossing(&poly[prev], &poly[cur], d_prev, d_cur)),
            (false, false) => {}
        }
    }
    out
}

/// `DecalEffect::ConstructDecal`: the decal's triangles, from the ground triangles it lands on.
/// `normal` is the decal's up axis (the world's z for a landscape decal), `tangent` its direction
/// (the caller's, not normalised here), and `size` its box edges. Each ground triangle is clipped
/// against the six faces in turn; the surviving polygon fans into triangles whose texture
/// coordinates run `0..1` across the box, each lifted by [`DECAL_LIFT`].
pub fn construct_decal(
    origin: [f32; 3],
    normal: [f32; 3],
    tangent: [f32; 3],
    size: [f32; 3],
    ground: &[GroundTriangle],
) -> Vec<DecalTriangle> {
    let bitangent = cross(normal, tangent);
    let half = size.map(|s| f64::from(s) * 0.5);
    let planes = [
        plane(tangent, half[0] - dot(tangent, origin)),
        plane(negate(tangent), half[0] + dot(tangent, origin)),
        plane(bitangent, half[1] - dot(bitangent, origin)),
        plane(negate(bitangent), half[1] + dot(bitangent, origin)),
        plane(normal, half[2] - dot(normal, origin)),
        plane(negate(normal), half[2] + dot(normal, origin)),
    ];
    let inv = [1.0 / f64::from(size[0]), 1.0 / f64::from(size[1])];
    let uv = |p: [f32; 3]| {
        let d = [
            f64::from(p[0]) - f64::from(origin[0]),
            f64::from(p[1]) - f64::from(origin[1]),
            f64::from(p[2]) - f64::from(origin[2]),
        ];
        let along = |axis: [f32; 3]| dot(axis, [d[0] as f32, d[1] as f32, d[2] as f32]);
        [
            (along(tangent) * inv[0] + 0.5) as f32,
            (along(bitangent) * inv[1] + 0.5) as f32,
        ]
    };
    let mut triangles = Vec::new();
    for tri in ground {
        let mut poly: Vec<Corner> = tri
            .iter()
            .map(|v| Corner {
                position: v.position,
                normal: v.normal,
            })
            .collect();
        for face in &planes {
            poly = clip_polygon(face, &poly);
            if poly.is_empty() {
                break;
            }
        }
        if poly.len() < 3 {
            continue;
        }
        for i in 1..poly.len() - 1 {
            let corners = [poly[0], poly[i], poly[i + 1]].map(|c| DecalCorner {
                position: [c.position[0], c.position[1], c.position[2] + DECAL_LIFT],
                normal: c.normal,
                uv: uv(c.position),
            });
            triangles.push(corners);
        }
    }
    triangles
}

fn plane(n: [f32; 3], d: f64) -> Plane {
    Plane { n, d: d as f32 }
}

/// `DecalEffect::Think`'s opacity at `age` seconds into a decal of `lifespan`: it fades in over
/// 10% of its life (at most 0.25 s) and out over 20% (at most 1 s), and a draw-once decal is
/// never faded in. `None` once the lifespan is over.
pub fn opacity_at(age: f64, lifespan: f32, draw_once: bool) -> Option<f32> {
    let life = f64::from(lifespan);
    if age >= life {
        return None;
    }
    let fade_out = (life * 0.2).min(1.0);
    let fade_in = (life * 0.1).min(0.25);
    let rise = if draw_once || age / fade_in > 1.0 {
        1.0
    } else {
        age / fade_in
    };
    let opacity = (rise / fade_out) * (life - age);
    Some(opacity.clamp(0.0, 1.0) as f32)
}

/// One decal in the world: its spec, its triangles and its fade (`DecalEffect`).
#[derive(Debug, Clone, PartialEq)]
pub struct DecalEffect {
    spec: DecalSpec,
    created: f64,
    opacity: f32,
    triangles: Vec<DecalTriangle>,
    alive: bool,
}

impl DecalEffect {
    /// `DecalEffect::Init`: the decal starts at `now` on the ground triangles it lands on. A
    /// decal with no ground under it is alive all the same and draws nothing (`ConstructDecal`
    /// returns success with no triangles).
    pub fn new(spec: DecalSpec, now: f64, ground: &[GroundTriangle]) -> Self {
        let triangles = construct_decal(
            spec.origin.to_array(),
            [0.0, 0.0, 1.0],
            spec.direction.to_array(),
            [spec.size; 3],
            ground,
        );
        Self {
            spec,
            created: now,
            opacity: 1.0,
            triangles,
            alive: true,
        }
    }

    /// `DecalEffect::Think` at client time `now`: its opacity, or dead once its lifespan is over.
    pub fn think(&mut self, now: f64) {
        if !self.alive {
            return;
        }
        match opacity_at(now - self.created, self.spec.lifespan, self.spec.draw_once) {
            Some(opacity) => self.opacity = opacity,
            None => self.kill(),
        }
    }

    fn kill(&mut self) {
        self.alive = false;
        self.triangles.clear();
    }

    pub fn alive(&self) -> bool {
        self.alive
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn triangles(&self) -> &[DecalTriangle] {
        &self.triangles
    }

    /// The decal's diffuse colour as `DecalEffect::CreateGeometry` packs it: alpha is the node's
    /// alpha at the current opacity, red, green and blue are the node's, each truncated to a byte.
    fn diffuse(&self) -> [f32; 4] {
        let byte = |c: f32| (f64::from(c) * 255.0) as u8;
        let [r, g, b, a] = self.spec.color;
        let alpha = (f64::from(a) * f64::from(self.opacity) * 255.0) as u8;
        [byte(r), byte(g), byte(b), alpha].map(|c| f32::from(c) / 255.0)
    }

    /// `DecalEffect::CreateGeometry`: this decal's vertices, three per triangle, appended to
    /// `out`. A draw-once decal is spent by its one draw.
    fn append_geometry(&mut self, out: &mut Vec<ObjectVertex>) {
        let [r, g, b, a] = self.diffuse();
        for tri in &self.triangles {
            for corner in tri {
                out.push(ObjectVertex {
                    position: corner.position,
                    normal: corner.normal,
                    color: [r, g, b, a],
                    uvs: [corner.uv, [0.0; 2], [0.0; 2], [0.0; 2]],
                    tangents: [[0.0; 3]; 2],
                });
            }
        }
        if self.spec.draw_once {
            self.kill();
        }
    }

    fn vertex_count(&self) -> usize {
        self.triangles.len() * 3
    }
}

/// The decals of one material (`DecalEffectGroup`), in the order they were made.
#[derive(Debug, Clone, PartialEq)]
struct DecalGroup {
    material: DataId,
    effects: Vec<DecalEffect>,
}

/// The scene's decals (`DecalEffectManager`): groups by material, thought and drawn each frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DecalEffects {
    groups: Vec<DecalGroup>,
}

impl DecalEffects {
    /// `DecalEffectManager::CreateDecalEffect`: a decal of `spec`, started now on `ground`, in
    /// its material's group (a new group when the material has none yet).
    pub fn create(&mut self, spec: DecalSpec, now: f64, ground: &[GroundTriangle]) {
        let effect = DecalEffect::new(spec, now, ground);
        match self
            .groups
            .iter_mut()
            .find(|group| group.material == spec.material)
        {
            Some(group) => group.effects.push(effect),
            None => self.groups.push(DecalGroup {
                material: spec.material,
                effects: vec![effect],
            }),
        }
    }

    /// `DecalEffectManager::Think`: every decal thinks, and the dead ones leave.
    pub fn think(&mut self, now: f64) {
        for group in &mut self.groups {
            for effect in &mut group.effects {
                effect.think(now);
            }
            group.effects.retain(DecalEffect::alive);
        }
        self.groups.retain(|group| !group.effects.is_empty());
    }

    /// `DecalEffectManager::Render`: each material's live decals as one triangle list, in group
    /// order. A group whose vertices reach [`MAX_GROUP_VERTICES`] draws nothing this frame.
    /// Draw-once decals are spent by this call.
    pub fn geometry(&mut self) -> Vec<(DataId, ParticleGeometry)> {
        let mut out = Vec::new();
        for group in &mut self.groups {
            let total: usize = group
                .effects
                .iter()
                .filter(|effect| effect.alive())
                .map(DecalEffect::vertex_count)
                .sum();
            if total == 0 || total >= MAX_GROUP_VERTICES {
                continue;
            }
            let mut vertices = Vec::with_capacity(total);
            for effect in &mut group.effects {
                if effect.alive() {
                    effect.append_geometry(&mut vertices);
                }
            }
            let indices = (0..vertices.len() as u32).collect();
            out.push((group.material, ParticleGeometry { vertices, indices }));
            group.effects.retain(DecalEffect::alive);
        }
        self.groups.retain(|group| !group.effects.is_empty());
        out
    }

    /// The live decals, over every material.
    pub fn count(&self) -> usize {
        self.groups.iter().map(|group| group.effects.len()).sum()
    }
}

#[cfg(test)]
#[path = "decal_tests.rs"]
mod tests;
