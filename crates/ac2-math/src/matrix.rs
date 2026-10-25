//! `Matrix4` (Legions `portal/shared/src/geometry/matrix4.cpp`): the 4 x 4 row-major matrix the
//! portal space's tunnel uses, with D3DX's row-vector convention (a point is `v * M`, the
//! translation in elements 12, 13 and 14). The products follow `D3DXMatrixMultiply`'s
//! `out = a * b`. Its elements are `float`, and so are these (`[f32; 16]`).

/// A row-major 4 x 4 matrix (`Matrix4::field_0x0` is element 0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4 {
    pub m: [f32; 16],
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Matrix4 {
    /// The identity.
    pub fn identity() -> Self {
        let mut m = [0.0; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[15] = 1.0;
        Self { m }
    }

    /// `Matrix4::RotateX(radians)`: the rotation about x (`cos` and `sin` taken in double
    /// precision, stored as float).
    pub fn rotate_x(radians: f32) -> Self {
        let (s, c) = f64::from(radians).sin_cos();
        let mut out = Self::identity();
        out.m[5] = c as f32;
        out.m[6] = s as f32;
        out.m[9] = -s as f32;
        out.m[10] = c as f32;
        out
    }

    /// `Matrix4::RotateY(radians)`.
    pub fn rotate_y(radians: f32) -> Self {
        let (s, c) = f64::from(radians).sin_cos();
        let mut out = Self::identity();
        out.m[0] = c as f32;
        out.m[2] = -s as f32;
        out.m[8] = s as f32;
        out.m[10] = c as f32;
        out
    }

    /// `Matrix4::RotateZ(radians)`.
    pub fn rotate_z(radians: f32) -> Self {
        let (s, c) = f64::from(radians).sin_cos();
        let mut out = Self::identity();
        out.m[0] = c as f32;
        out.m[1] = s as f32;
        out.m[4] = -s as f32;
        out.m[5] = c as f32;
        out
    }

    /// `D3DXMatrixMultiply(out, self, rhs)`: `self * rhs`, each element a float sum of four
    /// products in the order D3DX takes them.
    pub fn multiply(&self, rhs: &Matrix4) -> Matrix4 {
        let mut out = [0.0f32; 16];
        for row in 0..4 {
            for col in 0..4 {
                let a = |k: usize| self.m[row * 4 + k];
                let b = |k: usize| rhs.m[k * 4 + col];
                out[row * 4 + col] = a(0) * b(0) + a(1) * b(1) + a(2) * b(2) + a(3) * b(3);
            }
        }
        Matrix4 { m: out }
    }

    /// The homogeneous point `(x, y, z, 1) * M`: `(x', y', z', w)`, before the perspective
    /// divide. The tunnel divides each `x', y', z'` by `w`.
    pub fn transform(&self, x: f32, y: f32, z: f32) -> [f32; 4] {
        let m = &self.m;
        let col = |j: usize| x * m[j] + y * m[4 + j] + z * m[8 + j] + m[12 + j];
        [col(0), col(1), col(2), col(3)]
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix4;

    #[test]
    fn a_quarter_turn_about_z_maps_x_onto_y() {
        let turn = Matrix4::rotate_z(std::f32::consts::FRAC_PI_2);
        let [x, y, z, w] = turn.transform(1.0, 0.0, 0.0);
        assert!(x.abs() < 1e-6 && (y - 1.0).abs() < 1e-6 && z == 0.0 && w == 1.0);
    }

    #[test]
    fn products_are_row_vector_products() {
        // The translation (1, 2, 3) and the scale of x by 2. A row vector meets the left factor
        // first: `v * (A * B)` is `(v * A) * B`.
        let mut t = Matrix4::identity();
        t.m[12] = 1.0;
        t.m[13] = 2.0;
        t.m[14] = 3.0;
        let mut s = Matrix4::identity();
        s.m[0] = 2.0;
        // Translated first, then scaled: (1, 1, 1) -> (2, 3, 4) -> (4, 3, 4).
        let [x, y, z, _] = t.multiply(&s).transform(1.0, 1.0, 1.0);
        assert_eq!((x, y, z), (4.0, 3.0, 4.0));
        // Scaled first, then translated: (1, 1, 1) -> (2, 1, 1) -> (3, 3, 4).
        let [x, y, z, _] = s.multiply(&t).transform(1.0, 1.0, 1.0);
        assert_eq!((x, y, z), (3.0, 3.0, 4.0));
    }

    #[test]
    fn the_identity_is_neutral() {
        let r = Matrix4::rotate_x(0.3);
        assert_eq!(Matrix4::identity().multiply(&r), r);
        assert_eq!(r.multiply(&Matrix4::identity()), r);
    }
}
