//! The landscape's self shadow, the part of `LScapeTerrainMesh::GetShadowColor` that a grass frill
//! takes from the terrain under it (`FrillRenderType::AddPos`). The map term is the static shadow
//! map (`landscape_shadow` in the object shader); this is the self term.
//!
//! `LScapeRender::UpdateLighting` picks one of nine light directions (`SHADOWGEN_LIGHT_VECTORS`,
//! through `LScapeRender::GetSunLightDirectionBits`, whose best dot product with the sun wins) and
//! `LScapeTerrainMesh::SetSelfShadowBit` stores it as that light's bit: index 4 is the zero vector
//! and means no bit, indices above 4 shift down by one. Each terrain vertex then has alpha 255 when
//! its colour bits (`CLandBlockData::GetColor`) include that bit, else 0 (`UpdateNearTerrain`).

use ac2_math::Vec3;

/// `SHADOWGEN_LIGHT_VECTORS` of the admin 7148 build (the table `GetSunLightDirectionBits` reads;
/// the Legions 7160 build uses the same light set). Index 4 is the zero vector, which only wins
/// when no other direction faces the sun.
const SHADOW_LIGHT_VECTORS: [[f32; 3]; 9] = [
    [-0.45, 0.13, -0.32],
    [-0.36, 0.12, -0.44],
    [-0.28, 0.12, -0.57],
    [-0.14, 0.06, -0.58],
    [0.0, 0.0, 0.0],
    [0.11, -0.05, -0.59],
    [0.22, -0.10, -0.60],
    [0.33, -0.11, -0.49],
    [0.40, -0.12, -0.38],
];

/// The index of `SHADOW_LIGHT_VECTORS` that stands for no shadow bit.
const NO_BIT_INDEX: usize = 4;

/// `LScapeRender::GetSunLightDirectionBits` with `LScapeTerrainMesh::SetSelfShadowBit`: the shadow
/// bit (0 to 7) of the light direction closest to `light_dir` (the direction light travels), or
/// `None` where the zero vector wins. Dot products are f64; the first best index wins a tie.
pub fn shadow_light_bit(light_dir: Vec3) -> Option<u32> {
    let light = [
        f64::from(light_dir.x),
        f64::from(light_dir.y),
        f64::from(light_dir.z),
    ];
    let mut best = -1.0_f64;
    let mut index = None;
    for (i, vector) in SHADOW_LIGHT_VECTORS.iter().enumerate() {
        let unit = normalise(*vector);
        let dot = unit[0] * light[0] + unit[1] * light[1] + unit[2] * light[2];
        if best < dot {
            best = dot;
            index = Some(i);
        }
    }
    match index? {
        NO_BIT_INDEX => None,
        i if i > NO_BIT_INDEX => Some(i as u32 - 1),
        i => Some(i as u32),
    }
}

/// `D3DXVec3Normalize` in f64; the zero vector stays zero.
fn normalise(v: [f32; 3]) -> [f64; 3] {
    let v = v.map(f64::from);
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length == 0.0 {
        return [0.0; 3];
    }
    v.map(|c| c / length)
}

/// The vertex alpha `LScapeTerrainMesh::UpdateNearTerrain` gives a vertex: 255 when its colour
/// bits (`CLandBlockData::GetColor`) include `shadow_bit`, else 0.
pub fn vertex_shadow_alpha(colour: u32, shadow_bit: u32) -> u8 {
    if (colour >> (shadow_bit & 31)) & 1 == 1 {
        255
    } else {
        0
    }
}

/// `LScapeTerrainMesh::GetShadowColor`'s self term from the alphas of the polygon's four corner
/// vertices: their mean, `alpha / 255`, accumulated in f64 and stored as f32. Below 0.1 the
/// colour goes black; from 0.1 to 0.9 it scales by that factor; above 0.9 it is unchanged.
pub fn self_shadow_factor(alphas: [u8; 4]) -> f32 {
    let mean = alphas.iter().map(|a| f64::from(*a) / 255.0).sum::<f64>() / 4.0;
    if mean < 0.1 {
        0.0
    } else if mean > 0.9 {
        1.0
    } else {
        mean as f32
    }
}

#[cfg(test)]
mod tests {
    use super::{self_shadow_factor, shadow_light_bit, vertex_shadow_alpha};
    use ac2_math::Vec3;

    #[test]
    fn self_term_quarter_steps() {
        // Corners in shadow: 0, 1, 2, 3 and 4 of four.
        assert_eq!(self_shadow_factor([0; 4]), 0.0);
        assert!((self_shadow_factor([255, 0, 0, 0]) - 0.25).abs() < 1e-6);
        assert!((self_shadow_factor([255, 255, 0, 0]) - 0.5).abs() < 1e-6);
        assert!((self_shadow_factor([255, 255, 255, 0]) - 0.75).abs() < 1e-6);
        assert_eq!(self_shadow_factor([255; 4]), 1.0);
    }

    #[test]
    fn self_term_thresholds() {
        // Mean 0.0255 is below 0.1: black. Mean 0.1 exactly scales by 0.1.
        assert_eq!(self_shadow_factor([26, 0, 0, 0]), 0.0);
        assert!((self_shadow_factor([102, 0, 0, 0]) - 0.1).abs() < 1e-6);
        // Mean 0.9745 is above 0.9: unchanged.
        assert_eq!(self_shadow_factor([255, 255, 255, 229]), 1.0);
    }

    #[test]
    fn vertex_alpha_follows_the_colour_bit() {
        assert_eq!(vertex_shadow_alpha(0b0100, 2), 255);
        assert_eq!(vertex_shadow_alpha(0b0100, 1), 0);
        assert_eq!(vertex_shadow_alpha(0, 0), 0);
    }

    #[test]
    fn sun_picks_the_best_light_bit() {
        // Index 0 ((-0.45, 0.13, -0.32)) is the best match for its own direction: bit 0.
        let own = Vec3::new(-0.45, 0.13, -0.32).normalize();
        assert_eq!(shadow_light_bit(own), Some(0));
        // Index 7 ((0.33, -0.11, -0.49)) shifts down past the zero vector: bit 6.
        let seventh = Vec3::new(0.33, -0.11, -0.49).normalize();
        assert_eq!(shadow_light_bit(seventh), Some(6));
    }

    #[test]
    fn zero_vector_wins_when_no_light_faces_the_sun() {
        // Straight up: every table direction is below the horizon, so the zero vector (dot 0) wins.
        assert_eq!(shadow_light_bit(Vec3::new(0.0, 0.0, 1.0)), None);
    }
}
