use glam::{EulerRot, Mat3, Mat4, Quat, Vec3};

const VALID_EPSILON: f32 = 0.001;
const LEGACY_EPSILON: f32 = 0.0002;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackedAFrameError {
    SizeMismatch(usize),
    InvalidFrame,
}

impl std::fmt::Display for PackedAFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SizeMismatch(actual) => write!(
                f,
                "expected {} bytes for packed AFrame, got {}",
                AFrame::PACKED_LEN,
                actual
            ),
            Self::InvalidFrame => write!(f, "packed AFrame contained invalid values"),
        }
    }
}

impl std::error::Error for PackedAFrameError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameAxis {
    X,
    Y,
    Z,
}

impl FrameAxis {
    const fn basis_vector(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }

    const fn reference_axis(self) -> Self {
        match self {
            Self::X => Self::Z,
            Self::Y => Self::X,
            Self::Z => Self::Y,
        }
    }

    const fn fallback_axis(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::Z,
            Self::Z => Self::X,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AFrame {
    pub origin: Vec3,
    pub orientation: Quat,
}

impl Default for AFrame {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl AFrame {
    pub const PACKED_LEN: usize = 28;

    pub const IDENTITY: Self = Self {
        origin: Vec3::ZERO,
        orientation: Quat::IDENTITY,
    };

    pub const fn new(origin: Vec3, orientation: Quat) -> Self {
        Self {
            origin,
            orientation,
        }
    }

    pub fn is_valid(self) -> bool {
        self.origin.is_finite()
            && self.orientation.is_finite()
            && (self.orientation.length_squared() - 1.0).abs() <= VALID_EPSILON
    }

    pub fn local_to_global_point(self, point: Vec3) -> Vec3 {
        self.origin + self.orientation.mul_vec3(point)
    }

    pub fn global_to_local_point(self, point: Vec3) -> Vec3 {
        self.orientation.inverse().mul_vec3(point - self.origin)
    }

    pub fn local_to_global_vector(self, vector: Vec3) -> Vec3 {
        self.orientation.mul_vec3(vector)
    }

    pub fn global_to_local_vector(self, vector: Vec3) -> Vec3 {
        self.orientation.inverse().mul_vec3(vector)
    }

    pub fn to_mat4(self) -> Mat4 {
        Mat4::from_rotation_translation(self.orientation, self.origin)
    }

    pub fn to_packed_bytes(self) -> [u8; Self::PACKED_LEN] {
        let values = [
            self.origin.x,
            self.origin.y,
            self.origin.z,
            self.orientation.w,
            self.orientation.x,
            self.orientation.y,
            self.orientation.z,
        ];
        let mut bytes = [0_u8; Self::PACKED_LEN];

        for (index, value) in values.into_iter().enumerate() {
            let start = index * 4;
            bytes[start..start + 4].copy_from_slice(&value.to_le_bytes());
        }

        bytes
    }

    pub fn from_packed_bytes(bytes: [u8; Self::PACKED_LEN]) -> Result<Self, PackedAFrameError> {
        Self::from_packed_slice(&bytes)
    }

    pub fn from_packed_slice(bytes: &[u8]) -> Result<Self, PackedAFrameError> {
        if bytes.len() != Self::PACKED_LEN {
            return Err(PackedAFrameError::SizeMismatch(bytes.len()));
        }

        let frame = Self::new(
            Vec3::new(read_f32(bytes, 0), read_f32(bytes, 1), read_f32(bytes, 2)),
            Quat::from_xyzw(
                read_f32(bytes, 4),
                read_f32(bytes, 5),
                read_f32(bytes, 6),
                read_f32(bytes, 3),
            ),
        );

        if frame.is_valid() {
            Ok(frame)
        } else {
            Err(PackedAFrameError::InvalidFrame)
        }
    }

    pub fn to_frame(self) -> Frame {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub origin: Vec3,
    pub orientation: Quat,
}

impl Default for Frame {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Frame {
    pub const IDENTITY: Self = Self {
        origin: Vec3::ZERO,
        orientation: Quat::IDENTITY,
    };

    pub const fn new(origin: Vec3, orientation: Quat) -> Self {
        Self {
            origin,
            orientation,
        }
    }

    pub fn from_euler(origin: Vec3, order: EulerRot, angles: Vec3) -> Self {
        Self {
            origin,
            orientation: Quat::from_euler(order, angles.x, angles.y, angles.z),
        }
    }

    pub fn is_valid(self) -> bool {
        AFrame::from(self).is_valid()
    }

    pub fn approximately_eq(self, other: Self, epsilon: f32) -> bool {
        self.origin.abs_diff_eq(other.origin, epsilon)
            && self.quaternion_approximately_eq(other, epsilon)
    }

    pub fn quaternion_approximately_eq(self, other: Self, epsilon: f32) -> bool {
        (self.orientation.w - other.orientation.w).abs() <= epsilon
            && (self.orientation.x - other.orientation.x).abs() <= epsilon
            && (self.orientation.y - other.orientation.y).abs() <= epsilon
            && (self.orientation.z - other.orientation.z).abs() <= epsilon
    }

    pub fn local_to_global_point(self, point: Vec3) -> Vec3 {
        self.origin + self.orientation.mul_vec3(point)
    }

    pub fn global_to_local_point(self, point: Vec3) -> Vec3 {
        self.orientation.inverse().mul_vec3(point - self.origin)
    }

    pub fn local_to_global_vector(self, vector: Vec3) -> Vec3 {
        self.orientation.mul_vec3(vector)
    }

    pub fn global_to_local_vector(self, vector: Vec3) -> Vec3 {
        self.orientation.inverse().mul_vec3(vector)
    }

    pub fn compose_frame(self, child: Self) -> Self {
        Self::new(
            self.local_to_global_point(child.origin),
            self.orientation * child.orientation,
        )
    }

    pub fn compose_aframe(self, child: AFrame) -> Self {
        Self::new(
            self.local_to_global_point(child.origin),
            self.orientation * child.orientation,
        )
    }

    pub fn compose_scaled_aframe(self, child: AFrame, scale: Vec3) -> Self {
        Self::new(
            self.local_to_global_point(child.origin * scale),
            self.orientation * child.orientation,
        )
    }

    pub fn difference_from_frame(self, basis: Self) -> Self {
        Self::new(
            basis.global_to_local_point(self.origin),
            basis.orientation.inverse() * self.orientation,
        )
    }

    pub fn difference_from_aframe(self, basis: AFrame) -> Self {
        Self::new(
            basis.global_to_local_point(self.origin),
            basis.orientation.inverse() * self.orientation,
        )
    }

    pub fn rotate_local(&mut self, rotation: Vec3) {
        if let Some(delta) = delta_quat(rotation) {
            self.orientation *= delta;
        }
    }

    pub fn rotate_global(&mut self, rotation: Vec3) {
        if let Some(delta) = delta_quat(rotation) {
            self.orientation = delta * self.orientation;
        }
    }

    pub fn heading_vector(self) -> Vec3 {
        self.local_to_global_vector(Vec3::Y)
    }

    pub fn heading_degrees(self) -> f32 {
        let heading = self.heading_vector();
        ((450.0 - f64::from(heading.y).atan2(f64::from(heading.x)) * 57.29577951308232) % 360.0)
            as f32
    }

    pub fn set_heading(&mut self, degrees: f32) -> bool {
        let forward_z = self.heading_vector().z;
        if !forward_z.is_finite() || !degrees.is_finite() {
            return false;
        }

        let radians = f64::from(degrees) * 0.0174532925199433;
        // Retail `Frame::set_heading` supplies a unit horizontal pair and the existing
        // forward-vector Z to `set_vector_heading`; that helper normalizes the whole vector.
        // Consequently a pitched frame's resulting Z is `z / sqrt(1 + z*z)`, rather than the
        // original normalized Z.
        self.set_heading_vector(Vec3::new(
            radians.sin() as f32,
            radians.cos() as f32,
            forward_z,
        ))
    }

    pub fn set_heading_vector(&mut self, heading: Vec3) -> bool {
        if !heading.is_finite() {
            return false;
        }

        let length = (f64::from(heading.x) * f64::from(heading.x)
            + f64::from(heading.y) * f64::from(heading.y)
            + f64::from(heading.z) * f64::from(heading.z))
        .sqrt();
        if length < 0.00019999999 || !length.is_finite() {
            return false;
        }
        let normalized = Vec3::new(
            (f64::from(heading.x) / length) as f32,
            (f64::from(heading.y) / length) as f32,
            (f64::from(heading.z) / length) as f32,
        );

        let heading_radians = -((450.0
            - f64::from(normalized.y).atan2(f64::from(normalized.x)) * 57.29577951308232)
            % 360.0)
            * 0.0174532925199433;
        let pitch = f64::from(normalized.z).asin() * 57.29577951308232 * 0.0174532925199433;
        let half_heading = heading_radians * 0.5;
        let half_pitch = pitch * 0.5;
        let heading_cos = half_heading.cos() as f32;
        let heading_sin = half_heading.sin() as f32;
        let pitch_cos = half_pitch.cos() as f32;
        let pitch_sin = half_pitch.sin() as f32;
        let orientation = Quat::from_xyzw(
            heading_cos * pitch_sin,
            heading_sin * pitch_sin,
            heading_sin * pitch_cos,
            heading_cos * pitch_cos,
        );

        self.set_orientation_normalized(orientation)
    }

    /// Matches retail `Frame::rotate_around_axis_to_vector`: preserve one local axis while
    /// rotating the remaining frame basis toward the supplied global direction.
    pub fn rotate_axis_to_vector(&mut self, axis: FrameAxis, direction: Vec3) -> bool {
        if !direction.is_finite() {
            return false;
        }
        let axis_index = match axis {
            FrameAxis::X => 0,
            FrameAxis::Y => 1,
            FrameAxis::Z => 2,
        };
        let previous = [2_usize, 0, 1];
        let basis = [
            self.local_to_global_vector(Vec3::X),
            self.local_to_global_vector(Vec3::Y),
            self.local_to_global_vector(Vec3::Z),
        ];
        let fixed_axis = basis[axis_index];
        let projected = direction - fixed_axis * fixed_axis.dot(direction);
        let normal = if projected.length_squared() <= LEGACY_EPSILON * LEGACY_EPSILON {
            basis[previous[axis_index]]
        } else {
            projected.normalize()
        };
        if normal.dot(fixed_axis).abs() > LEGACY_EPSILON * 5.0 {
            return false;
        }
        let mut columns = basis;
        let adjacent = previous[axis_index];
        columns[adjacent] = normal;
        columns[previous[adjacent]] = normal.cross(fixed_axis);
        self.set_orientation_normalized(Quat::from_mat3(&Mat3::from_cols(
            columns[0], columns[1], columns[2],
        )))
    }

    pub fn set_orientation_normalized(&mut self, orientation: Quat) -> bool {
        if !orientation.is_finite() {
            return false;
        }

        let length = (f64::from(orientation.x) * f64::from(orientation.x)
            + f64::from(orientation.y) * f64::from(orientation.y)
            + f64::from(orientation.z) * f64::from(orientation.z)
            + f64::from(orientation.w) * f64::from(orientation.w))
        .sqrt();
        if !length.is_finite() || length == 0.0 {
            return false;
        }

        let normalized = Quat::from_xyzw(
            (f64::from(orientation.x) / length) as f32,
            (f64::from(orientation.y) / length) as f32,
            (f64::from(orientation.z) / length) as f32,
            (f64::from(orientation.w) / length) as f32,
        );
        if !normalized.is_finite() {
            return false;
        }

        let candidate = Self::new(self.origin, normalized);
        if candidate.is_valid() {
            self.orientation = normalized;
            true
        } else {
            false
        }
    }

    pub fn set_euler_rotation(&mut self, order: EulerRot, angles: Vec3) -> bool {
        self.set_orientation_normalized(Quat::from_euler(order, angles.x, angles.y, angles.z))
    }

    pub fn align_axis_to_vector(&mut self, axis: FrameAxis, direction: Vec3) -> bool {
        if !direction.is_finite() {
            return false;
        }

        let target = direction.normalize_or_zero();
        if target == Vec3::ZERO {
            return false;
        }

        let reference = [axis.reference_axis(), axis.fallback_axis()]
            .into_iter()
            .map(|candidate| self.local_to_global_vector(candidate.basis_vector()))
            .map(|vector| project_onto_plane(vector, target).normalize_or_zero())
            .find(|projected| *projected != Vec3::ZERO);

        let Some(reference) = reference else {
            return false;
        };

        let (x_axis, y_axis, z_axis) = match axis {
            FrameAxis::X => {
                let x_axis = target;
                let y_axis = reference.cross(x_axis).normalize_or_zero();
                if y_axis == Vec3::ZERO {
                    return false;
                }
                let z_axis = x_axis.cross(y_axis).normalize_or_zero();
                (x_axis, y_axis, z_axis)
            }
            FrameAxis::Y => {
                let y_axis = target;
                let z_axis = reference.cross(y_axis).normalize_or_zero();
                if z_axis == Vec3::ZERO {
                    return false;
                }
                let x_axis = y_axis.cross(z_axis).normalize_or_zero();
                (x_axis, y_axis, z_axis)
            }
            FrameAxis::Z => {
                let z_axis = target;
                let x_axis = reference.cross(z_axis).normalize_or_zero();
                if x_axis == Vec3::ZERO {
                    return false;
                }
                let y_axis = z_axis.cross(x_axis).normalize_or_zero();
                (x_axis, y_axis, z_axis)
            }
        };

        self.set_orientation_normalized(Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis)))
    }

    pub fn interpolate_origin(self, to: Self, t: f32) -> Vec3 {
        self.origin.lerp(to.origin, t)
    }

    pub fn interpolate_rotation(self, to: Self, t: f32) -> Quat {
        self.orientation.slerp(to.orientation, t)
    }

    pub fn interpolate(self, to: Self, t: f32) -> Self {
        Self::new(
            self.interpolate_origin(to, t),
            self.interpolate_rotation(to, t),
        )
    }

    pub fn to_mat4(self) -> Mat4 {
        Mat4::from_rotation_translation(self.orientation, self.origin)
    }
}

impl From<AFrame> for Frame {
    fn from(value: AFrame) -> Self {
        Self::new(value.origin, value.orientation)
    }
}

impl From<Frame> for AFrame {
    fn from(value: Frame) -> Self {
        Self::new(value.origin, value.orientation)
    }
}

fn delta_quat(rotation: Vec3) -> Option<Quat> {
    if !rotation.is_finite() {
        return None;
    }

    let angle = rotation.length();
    if angle < LEGACY_EPSILON {
        None
    } else {
        Some(Quat::from_axis_angle(rotation / angle, angle))
    }
}

fn project_onto_plane(vector: Vec3, normal: Vec3) -> Vec3 {
    vector - normal * vector.dot(normal)
}

fn read_f32(bytes: &[u8], index: usize) -> f32 {
    let start = index * 4;
    f32::from_le_bytes(
        bytes[start..start + 4]
            .try_into()
            .expect("slice length matches"),
    )
}

#[cfg(test)]
mod tests {
    use glam::{EulerRot, Quat, Vec3};

    use super::{AFrame, Frame, FrameAxis, PackedAFrameError};

    #[test]
    fn aframe_rejects_invalid_quaternion_norms() {
        let frame = AFrame::new(Vec3::ZERO, Quat::from_xyzw(0.0, 0.0, 0.0, 2.0));
        assert!(!frame.is_valid());
    }

    #[test]
    fn aframe_packed_roundtrip_preserves_values() {
        let frame = AFrame::new(
            Vec3::new(4.0, -3.0, 2.0),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );

        let roundtrip = AFrame::from_packed_bytes(frame.to_packed_bytes()).unwrap();

        assert_eq!(roundtrip.origin, frame.origin);
        assert!(roundtrip.orientation.abs_diff_eq(frame.orientation, 0.0001));
    }

    #[test]
    fn aframe_reports_invalid_packed_values() {
        let bytes = [0_u8; AFrame::PACKED_LEN];
        assert_eq!(
            AFrame::from_packed_bytes(bytes).unwrap_err(),
            PackedAFrameError::InvalidFrame
        );
    }

    #[test]
    fn global_to_local_inverts_local_to_global() {
        let radians = std::f32::consts::FRAC_PI_2;
        let frame = Frame::new(Vec3::new(10.0, 5.0, 0.0), Quat::from_rotation_z(radians));
        let local = Vec3::new(2.0, 0.0, 0.0);

        let world = frame.local_to_global_point(local);
        let roundtrip = frame.global_to_local_point(world);

        assert!((roundtrip - local).length() < 0.0001);
    }

    #[test]
    fn compose_and_difference_roundtrip() {
        let parent = Frame::new(
            Vec3::new(5.0, 1.0, -2.0),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );
        let child = Frame::new(
            Vec3::new(2.0, 0.0, 1.0),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_4),
        );

        let world = parent.compose_frame(child);
        let recovered = world.difference_from_frame(parent);

        assert!(recovered.approximately_eq(child, 0.0001));
    }

    #[test]
    fn compose_scaled_aframe_scales_child_translation() {
        let parent = Frame::new(Vec3::splat(10.0), Quat::IDENTITY);
        let child = AFrame::new(Vec3::new(2.0, 3.0, 4.0), Quat::IDENTITY);

        let combined = parent.compose_scaled_aframe(child, Vec3::new(2.0, 1.0, 0.5));

        assert_eq!(combined.origin, Vec3::new(14.0, 13.0, 12.0));
    }

    #[test]
    fn set_heading_updates_forward_axis() {
        let mut frame = Frame::IDENTITY;

        assert!(frame.set_heading(90.0));

        assert!((frame.heading_vector() - Vec3::X).length() < 0.0001);
        assert!((frame.heading_degrees() - 90.0).abs() < 0.0001);
    }

    #[test]
    fn set_heading_reuses_forward_z_before_retail_normalization() {
        let mut frame = Frame::IDENTITY;
        assert!(frame.set_heading_vector(Vec3::new(0.0, 1.0, 0.5)));

        assert!(frame.set_heading(180.0));

        let forward = frame.heading_vector();
        assert!((forward.x - 0.0).abs() < 0.000_001);
        assert!((forward.y - -0.912_870_9).abs() < 0.000_001);
        assert!((forward.z - 0.408_248_3).abs() < 0.000_001);
        assert!((frame.heading_degrees() - 180.0).abs() < 0.0001);
    }

    #[test]
    fn interpolate_blends_origin_and_rotation() {
        let from = Frame::IDENTITY;
        let to = Frame::new(
            Vec3::new(10.0, 0.0, 0.0),
            Quat::from_rotation_z(std::f32::consts::PI),
        );

        let halfway = from.interpolate(to, 0.5);

        assert_eq!(halfway.origin, Vec3::new(5.0, 0.0, 0.0));
        assert!((halfway.heading_vector() - Vec3::X).length() < 0.0001);
    }

    #[test]
    fn rotate_local_and_global_use_different_spaces() {
        let mut local = Frame::new(
            Vec3::ZERO,
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );
        let mut global = local;

        local.rotate_local(Vec3::X * std::f32::consts::FRAC_PI_2);
        global.rotate_global(Vec3::X * std::f32::consts::FRAC_PI_2);

        assert!(!local.quaternion_approximately_eq(global, 0.0001));
    }

    #[test]
    fn set_euler_rotation_matches_constructor() {
        let mut frame = Frame::IDENTITY;
        assert!(frame.set_euler_rotation(EulerRot::XYZ, Vec3::new(0.25, 0.5, 0.75)));

        let expected = Frame::from_euler(Vec3::ZERO, EulerRot::XYZ, Vec3::new(0.25, 0.5, 0.75));
        assert!(frame.quaternion_approximately_eq(expected, 0.0001));
    }

    #[test]
    fn align_axis_to_vector_points_selected_axis_at_target() {
        let mut frame = Frame::new(
            Vec3::ZERO,
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
        );

        assert!(frame.align_axis_to_vector(FrameAxis::Z, Vec3::X));

        let aligned = frame.local_to_global_vector(Vec3::Z);
        assert!((aligned - Vec3::X).length() < 0.0001);
        assert!(frame.is_valid());
    }

    #[test]
    fn rotate_axis_to_vector_preserves_the_selected_axis() {
        let mut frame = Frame::IDENTITY;

        assert!(frame.rotate_axis_to_vector(FrameAxis::X, Vec3::Y));

        assert!((frame.local_to_global_vector(Vec3::X) - Vec3::X).length() < 0.0001);
        assert!((frame.local_to_global_vector(Vec3::Z) - Vec3::Y).length() < 0.0001);
        assert!(frame.is_valid());
    }

    #[test]
    fn set_orientation_normalized_rejects_zero_quaternion() {
        let mut frame = Frame::IDENTITY;
        assert!(!frame.set_orientation_normalized(Quat::from_xyzw(0.0, 0.0, 0.0, 0.0)));
        assert_eq!(frame, Frame::IDENTITY);
    }
}
