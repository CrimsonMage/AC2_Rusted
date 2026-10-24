//! `LM_UVTranslate`, `LM_UVRotate`, `LM_UVScale` and `LM_UVTransform::ApplyTextureTransform` on
//! the vertex-shader path, as column-vector transposes of the D3DX row-vector matrices retail
//! multiplies into a stage's texture matrix.

use ac2_assets::Ac2LayerModifier;
use ac2_math::{Mat4, Vec3};

/// The stage a modifier targets and its transform at client time `time`.
pub fn modifier_transform(modifier: &Ac2LayerModifier, time: f64) -> (u32, Mat4) {
    match modifier {
        // `Matrix4::Translate(u, v, 0)` (the `Translate3` form is the no-vertex-shader path).
        Ac2LayerModifier::UvTranslate { tex_coord, u, v } => (
            *tex_coord,
            Mat4::from_translation(Vec3::new(u.value(time), v.value(time), 0.0)),
        ),
        // `Matrix4::RotateZ(degrees * 0.017453292)`.
        Ac2LayerModifier::UvRotate { tex_coord, rotate } => (
            *tex_coord,
            Mat4::from_rotation_z(rotate.value(time) * 0.017_453_292),
        ),
        Ac2LayerModifier::UvScale { tex_coord, u, v } => (
            *tex_coord,
            Mat4::from_scale(Vec3::new(u.value(time), v.value(time), 1.0)),
        ),
        // `m_mTransform` is stored row-major for row vectors; reading its rows as columns is the
        // transpose.
        Ac2LayerModifier::UvTransform { stage, matrix } => (*stage, Mat4::from_cols_array(matrix)),
    }
}

#[cfg(test)]
mod tests {
    use super::modifier_transform;
    use ac2_assets::{Ac2LayerModifier, Ac2Waveform};
    use ac2_math::Vec4;

    #[test]
    fn translate_and_rotate_follow_d3dx_row_vector_order() {
        let translate = Ac2LayerModifier::UvTranslate {
            tex_coord: 1,
            u: Ac2Waveform::constant(0.25),
            v: Ac2Waveform::constant(-0.5),
        };
        let (stage, m) = modifier_transform(&translate, 0.0);
        assert_eq!(stage, 1);
        assert_eq!(
            m * Vec4::new(0.5, 0.5, 0.0, 1.0),
            Vec4::new(0.75, 0.0, 0.0, 1.0)
        );

        let rotate = Ac2LayerModifier::UvRotate {
            tex_coord: 0,
            rotate: Ac2Waveform::constant(90.0),
        };
        // `RotateZ`: row 0 = (cos, sin), row 1 = (-sin, cos); (1, 0) * R = (cos, sin).
        let (_, m) = modifier_transform(&rotate, 0.0);
        let v = m * Vec4::new(1.0, 0.0, 0.0, 1.0);
        assert!((v.x).abs() < 1e-6 && (v.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn stored_transform_is_row_major() {
        let mut matrix = [0.0_f32; 16];
        for i in [0, 5, 10, 15] {
            matrix[i] = 1.0;
        }
        matrix[12] = 0.5; // row 3, column 0: u translation for row vectors
        let (stage, m) =
            modifier_transform(&Ac2LayerModifier::UvTransform { stage: 2, matrix }, 0.0);
        assert_eq!(stage, 2);
        assert_eq!(
            m * Vec4::new(0.0, 0.0, 0.0, 1.0),
            Vec4::new(0.5, 0.0, 0.0, 1.0)
        );
    }
}
