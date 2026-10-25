//! A free-flying debug camera in world AC axes (x east, y north, z up).

use ac2_math::Vec3;

#[derive(Debug, Default, Clone, Copy)]
pub struct FlyInput {
    pub forward: f32,
    pub right: f32,
    pub up: f32,
    pub fast: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct FlyCamera {
    pub position: Vec3,
    /// Radians clockwise from north.
    pub yaw: f32,
    /// Radians above the horizon.
    pub pitch: f32,
}

impl FlyCamera {
    pub fn looking_at(position: Vec3, target: Vec3) -> Self {
        let d = (target - position).normalize_or_zero();
        Self {
            position,
            yaw: d.x.atan2(d.y),
            pitch: d.z.clamp(-1.0, 1.0).asin(),
        }
    }

    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(sy * cp, cy * cp, sp)
    }

    pub fn turn(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.003;
        self.pitch = (self.pitch - dy * 0.003).clamp(-1.5, 1.5);
    }

    pub fn advance(&mut self, input: FlyInput, seconds: f32) {
        let speed = if input.fast { 400.0 } else { 60.0 } * seconds;
        let forward = self.forward();
        let right = Vec3::new(self.yaw.cos(), -self.yaw.sin(), 0.0);
        self.position +=
            (forward * input.forward + right * input.right + Vec3::Z * input.up) * speed;
    }
}
