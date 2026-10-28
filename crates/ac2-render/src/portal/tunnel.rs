//! The portal space's tunnel (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`):
//! a ring of 150 portals spaced 3 units apart along the tunnel, each with a radius and a world
//! matrix that twists it (`GeneratePortals`), and the geometry the rings become
//! (`Initialize`'s index buffer and points, `UpdateGeometry`'s vertices).
//!
//! The portals are a ring buffer: `MakeSafePortalIndex` wraps an index into `0..150`. A portal
//! behind the camera is regenerated at the far end (`GeneratePortals` from the previous index).

use ac2_math::matrix::Matrix4;
use ac2_math::waveform::kind;
use ac2_math::Waveform;

/// `TunnelPortalType` count (`0x96`).
pub const PORTALS: usize = 150;
/// The points of one ring (`m_pTunnelPortalPoints`, `0xC0` bytes of vectors).
pub const POINTS: usize = 16;
/// The vertices of one ring: the points and the first again, closing it (`0x11`).
pub const RING_VERTICES: usize = POINTS + 1;
/// The distance between two portals (`fPosition + 3.0`).
pub const PORTAL_SPACING: f64 = 3.0;

/// One tunnel portal (`TunnelPortalType`): its place along the tunnel, the ring's radius, and
/// the world matrix of the ring (the product of all the rings before it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Portal {
    /// `fPosition`: the distance along the tunnel.
    pub position: f32,
    /// `fRadius`: the ring's radius, a sine of the position.
    pub radius: f32,
    /// `mWorld`.
    pub world: Matrix4,
    /// `bHasParticle`: whether the portal carries a particle emitter (never set here).
    pub has_particle: bool,
}

impl Default for Portal {
    /// `StartNow`'s reset: no distance, no radius, the identity matrix.
    fn default() -> Self {
        Self {
            position: 0.0,
            radius: 0.0,
            world: Matrix4::identity(),
            has_particle: false,
        }
    }
}

/// One vertex of a tunnel ring: the position (after the perspective divide), the diffuse colour
/// (`ARGB`, the alpha is the ring's fade) and the texture coordinates (`u` along the tunnel, `v`
/// around the ring).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TunnelVertex {
    pub position: [f32; 3],
    pub color: u32,
    pub uv: [f32; 2],
}

/// `MakeSafePortalIndex`: the ring-buffer index `index` wraps to, in `0..PORTALS`.
pub fn make_safe_index(index: i64) -> usize {
    index.rem_euclid(PORTALS as i64) as usize
}

/// The tunnel's portals and its ring's points (`m_pTunnelPortals`, `m_pTunnelPortalPoints`).
#[derive(Debug, Clone, PartialEq)]
pub struct Tunnel {
    pub portals: Vec<Portal>,
    points: [[f32; 3]; POINTS],
}

impl Default for Tunnel {
    fn default() -> Self {
        Self::new()
    }
}

impl Tunnel {
    /// `Initialize`'s points: `(cos a, 0, sin a)` for `a = k * pi/8`, the unit circle's sixteen
    /// points (the angle sums in double precision, the coordinates stored as float).
    pub fn new() -> Self {
        let mut points = [[0.0f32; 3]; POINTS];
        for (k, point) in points.iter_mut().enumerate() {
            let angle = k as f64 * std::f64::consts::FRAC_PI_8;
            point[0] = angle.cos() as f32;
            point[2] = angle.sin() as f32;
        }
        Self {
            portals: vec![Portal::default(); PORTALS],
            points,
        }
    }

    /// The ring's points, each `(x, 0, z)`.
    pub fn points(&self) -> &[[f32; 3]; POINTS] {
        &self.points
    }

    /// `StartNow`'s reset of every portal.
    pub fn reset(&mut self) {
        self.portals.iter_mut().for_each(|p| *p = Portal::default());
    }

    /// `PortalSpace::Effect::GeneratePortals(start, count)`: the portals `start..start+count`,
    /// each after the one before it (wrapping). The position is the previous one plus 3; the
    /// radius is the sine waveform of the position; the world matrix is the ring's rotation
    /// (`mRotZ`, `mRotX` and `mRotY` from the three Perlin waveforms, `C * B * A`) times the
    /// previous portal's matrix.
    pub fn generate(&mut self, start: usize, count: usize, random_factor: f32) {
        for i in start..start + count {
            let prev = make_safe_index(i as i64 - 1);
            let previous = self.portals[prev];
            let position = (f64::from(previous.position) + PORTAL_SPACING) as f32;
            let mut rotation = ring_rotation(position, random_factor);
            // The local matrix is the rotation with the translation (0, 3, 0): the `fstp` of the
            // spacing lands on m[13] after the copy (the x87 chain's 0x0A6D56C store), so each
            // ring sits 3 units along its own y from the one before.
            let radius = sine_radius().value(f64::from(position));
            rotation.m[13] = PORTAL_SPACING as f32;
            self.portals[i] = Portal {
                position,
                radius,
                world: rotation.multiply(&previous.world),
                has_particle: false,
            };
        }
    }

    /// The buffer's indices: sixteen quads between each ring and the next (`Initialize`).
    pub fn index_buffer(&self) -> Vec<u16> {
        let mut indices = Vec::with_capacity(PORTALS * POINTS * 6);
        for ring in 0..PORTALS {
            let next = make_safe_index(ring as i64 + 1);
            let (here, there) = ((ring * RING_VERTICES) as u16, (next * RING_VERTICES) as u16);
            for j in 0..POINTS as u16 {
                indices.extend([
                    here + j,
                    here + j + 1,
                    there + j,
                    here + j + 1,
                    there + j + 1,
                    there + j,
                ]);
            }
        }
        indices
    }

    /// `UpdateGeometry(current, start, buffer, scale)`: the vertices of the tunnel's rings, scaled
    /// by `scale` (1.0 for the inner tunnel, 1.2 for the outer). Ring step `k` (0..150) is portal
    /// `current + k` (wrapping, `current` being `m_nLastPortalPassed`). The buffer fills from its
    /// last ring backwards, so buffer ring `b` holds step `149 - b`. Within a ring the memory runs
    /// the other way: vertex `m` (0..17) is point `16 - m` with `v = (16 - m) / 16`, so the
    /// closing vertex (`v = 1`) is the ring's first. A step before the start (`k < start`) is
    /// transparent; the others fade with their distance from the start (`X = k - start`, see
    /// [`fade_color`]).
    pub fn update_geometry(&self, current: usize, start: usize, scale: f32) -> Vec<TunnelVertex> {
        let mut vertices = Vec::with_capacity(PORTALS * RING_VERTICES);
        for ring in 0..PORTALS {
            let step = PORTALS - 1 - ring;
            let portal = &self.portals[make_safe_index((current + step) as i64)];
            let color = if step < start {
                0x00FF_FFFF
            } else {
                fade_color(step - start)
            };
            for m in 0..RING_VERTICES {
                let j = RING_VERTICES - 1 - m;
                let point = self.points[j % POINTS];
                vertices.push(TunnelVertex {
                    position: project(portal, point, scale),
                    color,
                    uv: [portal.position, j as f32 * 0.0625],
                });
            }
        }
        vertices
    }
}

/// `Render`'s first portal of the tunnel: `(int)((1 - m_fTunnelVisibility) * 149)`, the
/// visibility being the tunnel's fade in (0 nothing visible, 1 all of it).
pub fn start_for_visibility(visibility: f32) -> usize {
    ((1.0 - f64::from(visibility)) * 149.0) as usize
}

/// The ring's rotation: `GeneratePortals`' three Perlin waveforms at the portal's position
/// (the x twist at `0.005 p`, the y at `0.002 p`, the z at `0.005 p`), composed `C * B * A`.
fn ring_rotation(position: f32, random_factor: f32) -> Matrix4 {
    let p = f64::from(position);
    let x = perlin_wave(0.35, random_factor).value(p * 0.005);
    let y = perlin_wave(0.25, random_factor).value(p * 0.002);
    let z = perlin_wave(0.35, random_factor).value(p * 0.005);
    let rotate_x = Matrix4::rotate_x(x);
    let rotate_y = Matrix4::rotate_y(y);
    let rotate_z = Matrix4::rotate_z(z);
    rotate_z.multiply(&rotate_y).multiply(&rotate_x)
}

/// `Waveform(WAVEFORM_PERLIN, 0, amplitude, random factor, 1, 1)`.
fn perlin_wave(amplitude: f32, random_factor: f32) -> Waveform {
    Waveform {
        kind: kind::PERLIN,
        base: 0.0,
        amplitude,
        phase: random_factor,
        frequency: 1.0,
        ..Waveform::default()
    }
}

/// `Waveform(WAVEFORM_SINE, 10, 6, 0, 0.01, 1)`: the radius.
fn sine_radius() -> Waveform {
    Waveform {
        kind: kind::SINE,
        base: 10.0,
        amplitude: 6.0,
        phase: 0.0,
        frequency: 0.01,
        ..Waveform::default()
    }
}

/// One ring vertex's position (`UpdateGeometry`'s x87 chain): the point's x, scaled by the
/// portal's radius and the ring's scale, is stored as a float (`fstp`), the y is unscaled, the z
/// is scaled in extended precision; the matrix product and `1 / w` stay in extended precision,
/// and the divided coordinates round to float at the store.
fn project(portal: &Portal, point: [f32; 3], scale: f32) -> [f32; 3] {
    let radius = f64::from(portal.radius);
    let s = f64::from(scale);
    let x = f64::from((f64::from(point[0]) * radius * s) as f32);
    let z = f64::from(point[2]) * radius * s;
    let [px, py, pz, w] = portal.world.transform_f64(x, f64::from(point[1]), z);
    let inverse = 1.0 / w;
    [
        (px * inverse) as f32,
        (py * inverse) as f32,
        (pz * inverse) as f32,
    ]
}

/// The ring's colour for `X`, its distance in rings from the start (`X = k - start`, the counter
/// `UpdateGeometry` compares with 16): `0.8 + 0.2 (1 - X/15)` below 16, `0.8 (1 - X/149)^2`
/// beyond. The constants are the float literals of the image; the alpha stays in extended
/// precision and the alpha byte is the low eight bits of `ftol(255 * alpha)`.
pub(crate) fn fade_color(distance: usize) -> u32 {
    let x = distance as f64;
    let alpha = if distance < 16 {
        f64::from(0.8f32) + f64::from(0.2f32) * (1.0 - x * f64::from(1.0f32 / 15.0))
    } else {
        let t = 1.0 - x * f64::from(1.0f32 / 149.0);
        f64::from(0.8f32) * (t * t)
    };
    let byte = (alpha * 255.0) as i32 as u32 & 0xFF;
    (byte << 24) | 0x00FF_FFFF
}
