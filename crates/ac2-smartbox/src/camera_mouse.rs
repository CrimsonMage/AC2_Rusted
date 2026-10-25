//! `CameraManager::MouseMove` (admin 0x005778A0, Legions 0x00547840): the mouse-look motion of a
//! frame, smoothed over the last ten frames' deltas (newest first, weights 1, 0.2, 0.04, ...,
//! every partial sum stored as a `float`), turns the camera's viewer offset about the vertical by
//! `-dx * sensitivity` degrees and moves its pitch slope (`m_viewerOffset.z`) by
//! `dy * sensitivity` degrees' worth (inverted unless `m_invertY`), clamped to +-1.3744467; in
//! zoom mode the vertical motion changes the camera distance instead (kept inside 0.2..10).

use crate::camera::{ld, st, CameraManager};

/// The filter's length and decay (`0.2f`).
const FILTER_LENGTH: usize = 10;
const FILTER_DECAY: f32 = 0.2;
/// `-0.017453292f` and `0.017453292f`: degrees to radians.
const NEG_DEGREE: f32 = -0.017_453_292;
const DEGREE: f32 = 0.017_453_292;
/// The viewer-offset slope limits.
const MAX_SLOPE: f32 = 1.374_446_7;
/// `0.06666667f`: zoom-mode distance per filtered unit.
const ZOOM_PER_UNIT: f32 = 0.066_666_67;

impl CameraManager {
    /// `CameraManager::MouseMove(dx, dy)`.
    pub fn mouse_move(&mut self, dx: i32, dy: i32) {
        self.mouse_filter.push_front((dx, dy));
        self.mouse_filter.truncate(FILTER_LENGTH);
        let (mut sum_x, mut sum_y, mut total, mut weight) = (0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32);
        for &(x, y) in &self.mouse_filter {
            sum_x = st(f64::from(x) * ld(weight) + ld(sum_x));
            sum_y = st(f64::from(y) * ld(weight) + ld(sum_y));
            total = st(ld(weight) + ld(total));
            weight = st(ld(FILTER_DECAY) * ld(weight));
        }
        let inverse = 1.0 / ld(total);
        let filtered_x = st(ld(sum_x) * inverse * ld(self.mouse_sensitivity));
        let filtered_y = st(inverse * ld(sum_y) * ld(self.mouse_sensitivity));
        let v = self.viewer_offset;
        let (nx, ny, nz) = (-ld(v.x), -ld(v.y), -ld(v.z));
        let invert = if self.invert_y { 1.0 } else { -1.0 };
        let angle = ld(filtered_x) * ld(NEG_DEGREE);
        let (sin, cos) = angle.sin_cos();
        // The pitch rotation of the original is by zero (`sin(0)`, `cos(0)`).
        let mut slope = st(nz);
        if !self.zoom_mode {
            slope = st(ld(filtered_y) * invert * ld(DEGREE) + ld(slope));
            slope = slope.clamp(-MAX_SLOPE, MAX_SLOPE);
        } else {
            let distance = st(ld(filtered_y) * ld(ZOOM_PER_UNIT) + ld(self.get_camera_distance()));
            if distance > 0.2 && distance < 10.0 {
                self.camera_distance = distance;
            }
        }
        if !self.mapview {
            self.viewer_offset.x = -st(cos * nx - ny * sin);
            self.viewer_offset.z = -slope;
            self.viewer_offset.y = -st(sin * nx + ny * cos);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::camera::CameraManager;

    #[test]
    fn mouse_look_turns_the_offset_and_smooths_over_frames() {
        let mut camera = CameraManager::new();
        camera.viewer_offset = ac2_physics::Vector::new(0.0, -1.0, 0.25);
        // 90 degrees to the right in one frame: the offset swings from south to west of the
        // target (x east, y north).
        camera.mouse_move(90, 0);
        let v = camera.viewer_offset;
        assert!((v.x + 1.0).abs() < 1e-5 && v.y.abs() < 1e-5, "{v:?}");
        assert_eq!(v.z, 0.25);
        // A still mouse keeps easing: the next frame turns by 90 * 0.2 / 1.2 = 15 degrees.
        let before = camera.viewer_offset;
        camera.mouse_move(0, 0);
        let after = camera.viewer_offset;
        let turned = (before.x * after.y - before.y * after.x)
            .asin()
            .to_degrees();
        assert!((turned.abs() - 15.0).abs() < 1e-3, "{turned}");
        // Moving the mouse up looks up (the slope falls) and stops at the limit.
        for _ in 0..200 {
            camera.mouse_move(0, -50);
        }
        assert!((camera.viewer_offset.z + 1.374_446_7).abs() < 1e-6);
    }
}
