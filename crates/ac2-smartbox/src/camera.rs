//! `CameraManager` (Legions `portal/client/src/smartbox/camera/cameramanager.cpp`; the ported
//! functions are exact instruction matches of admin 7148): the player camera's orbit
//! (`m_viewerOffset` holds the horizontal direction from the player to the camera in x/y and
//! the pitch in z), its distance and shoulder offset, and the camera actions.
//!
//! The x87 ran at 53-bit precision, so expressions are evaluated in `f64` in the retail
//! operation order and rounded to `f32` exactly where the retail code stores a `float`.

use ac2_physics::{Heading, Position, Vector};

use crate::prefs::CameraValues;
use crate::targeting::TargetingModule;

/// Loads a `float` onto the x87 stack.
#[inline(always)]
pub fn ld(value: f32) -> f64 {
    f64::from(value)
}

/// Stores an x87 value into `float` memory.
#[inline(always)]
pub fn st(value: f64) -> f32 {
    value as f32
}

/// `F_EPSILON` (`0x3951B717`).
pub const F_EPSILON: f32 = 0.0002;
/// The pitch limit (`0x3FAFEDDF`, about 78.75 degrees).
const MAX_PITCH: f32 = 1.374_446_7;
/// The farthest camera distance.
const MAX_DISTANCE: f32 = 160.0;
/// `5*pi/2` as the double the heading formulas subtract `fpatan` from.
const FIVE_HALVES_PI: f64 = 7.853_981_633_974_483;
/// `Heading::get_radians`' `float` factor (`0x3C8EFA35`).
const DEGREES_TO_RADIANS: f32 = 0.017_453_292;
/// The `float` degrees-per-radian factor of the heading animation (`0x42652EE1`).
const RADIANS_TO_DEGREES: f32 = 57.295_78;
/// The first-person camera distance.
pub const FIRST_PERSON_DISTANCE: f32 = 0.01;

/// Camera actions (`CameraManager::StartAction`/`StopAction`), `ActionInterpreter` ids.
pub mod action {
    pub const FAKE_TRANSITION: u32 = 0x4000_0028;
    pub const SHOULDER_VIEW: u32 = 0x4000_002A;
    pub const MOUSE_LOOK_HOLD: u32 = 0x4000_002C;
    pub const ZOOM_IN: u32 = 0x4000_002D;
    pub const ROTATE_LEFT: u32 = 0x4000_002E;
    pub const ROTATE_RIGHT: u32 = 0x4000_002F;
    pub const RESET: u32 = 0x4000_0030;
    pub const TOGGLE_FIRST_PERSON: u32 = 0x4000_00A2;
    pub const LOOK_AT_GROUND: u32 = 0x4000_00AE;
    pub const ZOOM_OUT: u32 = 0x4000_00B2;
    pub const PITCH_UP: u32 = 0x4000_00B6;
    pub const PITCH_DOWN: u32 = 0x4000_00B7;
    pub const TOGGLE_MOUSE_LOOK: u32 = 0x4000_00BA;
}

/// What the camera reads from the player's physics object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSubject {
    /// `GetPosition`.
    pub position: Position,
    /// `GetHeight` and `GetRadius` (x87 results, not stored).
    pub height: f64,
    pub radius: f64,
    /// `GetTotalScale` z.
    pub scale_z: f32,
    /// `get_local_velocity`.
    pub local_velocity: Vector,
    /// `GetElement` and `is_nodraw`.
    pub element: u32,
    pub no_draw: bool,
}

/// The heading of a horizontal direction: `Heading(5*pi/2 - atan2(y, x), RADIANS)` of the
/// normalised x/y (x stored before the `fpatan`), zero under `F_EPSILON`.
fn direction_heading(x: f32, y: f32) -> Heading {
    let (x, y) = (ld(x), ld(y));
    let length = (y * y + x * x).sqrt();
    if length < ld(F_EPSILON) {
        return Heading::ZERO;
    }
    let inverse = 1.0 / length;
    let nx = ld(st(inverse * x));
    Heading::from_radians(st(FIVE_HALVES_PI - (inverse * y).atan2(nx)))
}

/// `Heading::get_radians` (left on the x87 stack).
fn radians(heading: Heading) -> f64 {
    ld(DEGREES_TO_RADIANS) * ld(heading.degrees())
}

/// `Matrix4::RotateAngles` then the forward row (`m[4..7]` with the transform's `w` of 1):
/// the unit direction for pitch `-angles.x` and heading `angles.y`.
fn rotate_angles_forward(angles: Vector) -> Vector {
    let (ax, ay, az) = (-ld(angles.x), -ld(angles.y), -ld(angles.z));
    let (sx, cx) = (ax.sin(), ax.cos());
    let (sy, cy) = (ay.sin(), ld(st(ay.cos())));
    let (sz, cz) = (ld(st(az.sin())), ld(st(az.cos())));
    let cz_sy = ld(st(cz * sy));
    let forward = (
        st(sz * sx - cz_sy * cx),
        st(cy * cx),
        st(cz * sx + sy * sz * cx),
    );
    // `UpdateCamera` transforms (0, 1, 0) by the matrix: `RotateBasis` leaves m[3], m[7] and
    // m[11] at 0, so `w` is exactly 1 and each component is `(a + b) * 0 + forward`, which
    // only turns a negative zero positive.
    Vector::new(
        st(ld(forward.0) + 0.0),
        st(ld(forward.1) + 0.0),
        st(ld(forward.2) + 0.0),
    )
}

/// `CameraManager`, with the preference values `SetupPrefs` blesses applied.
#[derive(Debug, Clone)]
pub struct CameraManager {
    pub fov: f32,
    anim_fov: f32,
    /// `m_targetOffset`: the look-at offset above the player (`SetCameraDefaults`).
    pub target_offset: Vector,
    pub viewer_offset: Vector,
    anim_pitch: f32,
    anim_heading: f32,
    pub mouse_look: bool,
    pub first_person: bool,
    pub mapview: bool,
    pub camera_distance: f32,
    prefs_distance: f32,
    prefs_pitch: f32,
    pub third_person_steers: bool,
    anim_scale: f32,
    rotate_scale: f32,
    align_to_slope: bool,
    start_shoulder: Vector,
    current_shoulder: Vector,
    target_shoulder: Vector,
    interp_time: f32,
    pitch_from_slope: f32,
    z_velocity: f32,
    last_z: f32,
    pub targeting: TargetingModule,
    /// `m_mouseSensitivity`, `m_invertY` and `m_zoomMode` (constructor defaults: 1, off, off).
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub zoom_mode: bool,
    /// `MouseMove`'s static `mouseFilterDeque`: the last ten mouse-look deltas, newest first.
    pub mouse_filter: std::collections::VecDeque<(i32, i32)>,
}

impl Default for CameraManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraManager {
    /// `CameraManager::CameraManager`, then the `SetupPrefs` defaults through `HandleChange`:
    /// camera height 0, distance 4.5, tilt 0.25 (which re-runs `SetDefaults`), transition time
    /// 1, rotate and zoom speed 1, auto distance, align to slope, and third person steers.
    pub fn new() -> Self {
        Self {
            fov: 65.0,
            anim_fov: 0.0,
            target_offset: Vector::new(0.0, 0.0, 0.0),
            viewer_offset: Vector::new(0.0, -3.5, 0.0),
            anim_pitch: 0.0,
            anim_heading: 0.0,
            mouse_look: false,
            first_person: false,
            mapview: false,
            camera_distance: 3.5,
            prefs_distance: 4.5,
            prefs_pitch: 0.25,
            third_person_steers: true,
            anim_scale: 1.0,
            rotate_scale: 1.0,
            align_to_slope: true,
            start_shoulder: Vector::new(0.0, 0.0, 0.0),
            current_shoulder: Vector::new(0.0, 0.0, 0.0),
            target_shoulder: Vector::new(0.0, 0.0, 0.0),
            interp_time: 0.0,
            pitch_from_slope: 0.0,
            z_velocity: 0.0,
            last_z: 0.0,
            targeting: TargetingModule::default(),
            mouse_sensitivity: 1.0,
            invert_y: false,
            zoom_mode: false,
            mouse_filter: std::collections::VecDeque::new(),
        }
    }

    /// `SmartBox::SetCameraDefaults` for a new player: `SetTargetObject(player, 1, 1)`, the
    /// look-at offset `height * mInitialCameraHeight` above it, then `SetDefaults`.
    pub fn set_player(&mut self, subject: &CameraSubject, initial_camera_height: f32) {
        let height = st(subject.height * ld(initial_camera_height));
        // `SetTargetObject`: `StoreTargetPos` from no target finds no old position to blend
        // from; the first target becomes the player.
        self.targeting
            .store_target_pos(Some(subject), true, 1.0, 1.0);
        self.targeting.set_player(true);
        self.targeting.camera_distance = self.camera_distance;
        self.target_offset = Vector::new(0.0, 0.0, height);
        self.set_defaults(Some(subject));
    }

    /// `CameraManager::ResetCamera` (the player left).
    pub fn reset(&mut self) {
        self.targeting.set_player(false);
    }

    /// `CameraManager::GetCameraDistance` (no camera effects run, so their distance sum is 0).
    pub fn get_camera_distance(&self) -> f32 {
        self.camera_distance
    }

    /// `CameraManager::GetGlobalDirection`: `-m_viewerOffset`, the pitch in z included.
    pub fn global_direction(&self) -> [f32; 3] {
        let v = self.viewer_offset;
        [-v.x, -v.y, -v.z]
    }

    /// `CameraManager::SetDefaults`: behind the player at the default pitch and distance.
    fn set_defaults(&mut self, subject: Option<&CameraSubject>) {
        if self.mouse_look {
            let pitched = Vector::new(0.0, -1.0, self.prefs_pitch).d3dx_normalize();
            let heading = direction_heading(self.viewer_offset.x, self.viewer_offset.y);
            self.viewer_offset = Vector::new(
                st(radians(heading).sin()),
                st(radians(heading).cos()),
                pitched.z,
            );
        } else {
            self.viewer_offset = Vector::new(0.0, -1.0, self.prefs_pitch);
        }
        if self.first_person {
            self.camera_distance = FIRST_PERSON_DISTANCE;
        } else {
            let _ = subject;
            self.targeting.fake_transition();
            self.camera_distance = self.prefs_distance;
        }
    }

    /// `CameraManager::StartAction`.
    pub fn start_action(&mut self, action: u32, extent: f32, subject: Option<&CameraSubject>) {
        match action {
            action::FAKE_TRANSITION => self.targeting.fake_transition(),
            action::SHOULDER_VIEW => self.shoulder_view(),
            action::MOUSE_LOOK_HOLD => self.set_mouse_look(true, subject),
            action::ZOOM_IN if !self.first_person => {
                // `ZoomIn`: the floor is `0.01f + 0.1f`, stored; the compare is unrounded.
                let closer = ld(self.camera_distance) - 1.5;
                self.camera_distance = st(closer);
                let floor = st(ld(FIRST_PERSON_DISTANCE) + ld(0.1_f32));
                if closer < ld(floor) {
                    self.camera_distance = floor;
                }
                self.targeting.camera_distance = self.camera_distance;
            }
            action::ZOOM_OUT if !self.first_person => {
                let farther = ld(self.camera_distance) + 1.5;
                self.camera_distance = st(farther);
                if farther > ld(MAX_DISTANCE) {
                    self.camera_distance = MAX_DISTANCE;
                }
                self.targeting.camera_distance = self.camera_distance;
            }
            action::ROTATE_LEFT if !self.first_person => {
                self.anim_heading = st(ld(extent) * ld(self.rotate_scale));
            }
            action::ROTATE_RIGHT if !self.first_person => {
                self.anim_heading = -st(ld(extent) * ld(self.rotate_scale));
            }
            action::RESET => {
                self.set_defaults(subject);
                self.anim_scale = 1.0;
                self.anim_pitch = 0.0;
                self.anim_heading = 0.0;
            }
            action::TOGGLE_FIRST_PERSON => self.set_first_person(!self.first_person),
            action::LOOK_AT_GROUND => self.look_at_ground(),
            action::PITCH_UP => self.anim_pitch = st(ld(extent) * ld(self.rotate_scale)),
            action::PITCH_DOWN => self.anim_pitch = -st(ld(extent) * ld(self.rotate_scale)),
            action::TOGGLE_MOUSE_LOOK => self.set_mouse_look(!self.mouse_look, subject),
            _ => {}
        }
    }

    /// `CameraManager::StopAction`.
    pub fn stop_action(&mut self, action: u32, subject: Option<&CameraSubject>) {
        match action {
            action::MOUSE_LOOK_HOLD => self.set_mouse_look(false, subject),
            action::ROTATE_LEFT | action::ROTATE_RIGHT if !self.first_person => {
                self.anim_heading = 0.0;
            }
            action::PITCH_UP | action::PITCH_DOWN => self.anim_pitch = 0.0,
            _ => {}
        }
    }

    /// `CameraManager::ShoulderView`: toggles a close over-the-shoulder view (1.3 m).
    fn shoulder_view(&mut self) {
        if self.mapview || self.first_person || self.mouse_look {
            return;
        }
        if self.camera_distance == 1.3 {
            self.camera_distance = self.prefs_distance;
        } else {
            self.viewer_offset = Vector::new(0.0, -1.0, self.prefs_pitch);
            self.camera_distance = 1.3;
        }
        self.targeting.camera_distance = self.camera_distance;
        self.targeting.fake_transition();
    }

    /// `CameraManager::LookAtGround`: steep down (1.37), or back to the default pitch when
    /// already there.
    fn look_at_ground(&mut self) {
        if self.first_person {
            return;
        }
        let v = self.viewer_offset;
        let (dx, dy, dz) = (ld(v.x), ld(v.y) - -1.0, ld(v.z) - ld(1.37_f32));
        let length = ((dz * dz + dy * dy) + dx * dx).sqrt();
        let pitch = if length > ld(F_EPSILON) {
            1.37
        } else {
            self.prefs_pitch
        };
        self.viewer_offset = Vector::new(0.0, -1.0, pitch);
        self.targeting.fake_transition();
    }

    /// `CameraManager::SetMouseLookState`: the viewer offset turns between the player's frame
    /// (orbiting with it) and the world's (mouse look).
    pub fn set_mouse_look(&mut self, state: bool, subject: Option<&CameraSubject>) {
        if self.mouse_look == state {
            return;
        }
        self.mouse_look = state;
        if let Some(player) = subject {
            let frame = player.position.frame;
            self.viewer_offset = if state {
                frame.local_to_global_vec(self.viewer_offset)
            } else {
                frame.global_to_local_vec(self.viewer_offset)
            };
        }
    }

    /// `CameraManager::SetFirstPersonState`.
    pub fn set_first_person(&mut self, state: bool) {
        if self.first_person == state {
            return;
        }
        if state {
            self.camera_distance = FIRST_PERSON_DISTANCE;
            self.viewer_offset = Vector::new(0.0, -1.0, self.prefs_pitch);
            self.first_person = true;
            return;
        }
        if (ld(self.camera_distance) - ld(FIRST_PERSON_DISTANCE)).abs() < ld(F_EPSILON) {
            self.camera_distance = self.prefs_distance;
        }
        self.first_person = false;
    }

    /// `CameraManager::CalcAlignToSlope`: the pitch eases toward the player's vertical motion
    /// so the camera looks along slopes.
    fn calc_align_to_slope(&mut self, subject: &CameraSubject, quantum: f32) {
        let z = subject.position.frame.origin.z;
        let q = ld(quantum);
        let rise = st(ld(z) - ld(self.last_z));
        if quantum > 1.0 {
            self.z_velocity = rise;
        } else {
            let keep = ld(st(1.0 - q));
            let decayed = keep * ld(self.z_velocity);
            self.z_velocity = st(decayed);
            let size = ld(rise).abs();
            if size < ld(F_EPSILON) {
                self.z_velocity = st(decayed * keep);
            } else if size < 5.0 {
                self.z_velocity = st(ld(rise) * q + decayed);
            }
        }
        self.z_velocity = self.z_velocity.clamp(-MAX_PITCH, MAX_PITCH);
        let wanted = -self.z_velocity;
        self.last_z = subject.position.frame.origin.z;
        let gap = ld(self.pitch_from_slope) - ld(wanted);
        if gap.abs() > 0.02 {
            self.pitch_from_slope = st(ld(self.pitch_from_slope) - gap * q);
            return;
        }
        if self.pitch_from_slope > wanted {
            let eased = ld(self.pitch_from_slope) - q * ld(0.05_f32);
            self.pitch_from_slope = st(eased);
            if eased < ld(wanted) {
                self.pitch_from_slope = wanted;
            }
        } else if self.pitch_from_slope < wanted {
            let eased = q * ld(0.05_f32) + ld(self.pitch_from_slope);
            self.pitch_from_slope = st(eased);
            if eased > ld(wanted) {
                self.pitch_from_slope = wanted;
            }
        }
    }

    /// `CameraManager::CalcShoulderOffset`: a sideways offset of up to 70% of the player's
    /// radius that grows as the camera closes in under 2.9 m (to the side the camera orbits
    /// on), eased over a second when it jumps; `distance` 0 asks without easing.
    pub fn calc_shoulder_offset(
        &mut self,
        subject: Option<&CameraSubject>,
        distance: f32,
    ) -> Vector {
        let Some(player) = subject.filter(|_| !self.first_person) else {
            return Vector::new(0.0, 0.0, 0.0);
        };
        let reach = st(player.radius * ld(0.7_f32));
        // The bias stays on the x87 stack; its stored copy feeds the facing scale.
        let mut bias = (1.0 - ld(distance) / ld(2.9_f32)) * ld(reach);
        let mut stored = st(bias);
        if bias > ld(reach) {
            bias = ld(reach);
            stored = reach;
        }
        if bias < 0.0 {
            bias = 0.0;
            stored = 0.0;
        }
        let mut side = bias;
        let v = self.viewer_offset;
        if !self.mouse_look && v.y > 0.0 {
            let unit = Vector::new(v.x, v.y, 0.0).d3dx_normalize();
            side = (1.0 - ld(unit.y)) * ld(stored);
        }
        if v.x < 0.0 {
            side = -side;
        }
        let offset = player
            .position
            .frame
            .local_to_global_vec(Vector::new(st(side), 0.0, 0.0));
        if distance == 0.0 {
            return offset;
        }
        let dx = ld(offset.x) - ld(self.current_shoulder.x);
        let dy = ld(offset.y) - ld(self.current_shoulder.y);
        if dx * dx + dy * dy <= ld(0.05_f32) {
            let settled = |a: f32, b: f32| (ld(a) - ld(b)).abs() < ld(F_EPSILON);
            if self.interp_time == 0.0
                && settled(self.target_shoulder.x, self.current_shoulder.x)
                && settled(self.target_shoulder.y, self.current_shoulder.y)
                && settled(self.target_shoulder.z, self.current_shoulder.z)
            {
                self.current_shoulder = offset;
                self.target_shoulder = offset;
                self.start_shoulder = offset;
            }
        } else {
            self.target_shoulder = offset;
            self.start_shoulder = self.current_shoulder;
            self.interp_time = 0.0;
        }
        self.current_shoulder
    }

    /// `CameraManager::UpdateCamera`: advances the zoom, pitch, slope and heading animations
    /// and the shoulder easing by `quantum`, then the viewer position for the player
    /// (`TargetingModule::CalcViewerPos`). `current` is where the camera is now (a blend from
    /// no target starts there).
    pub fn update_camera(
        &mut self,
        quantum: f32,
        subject: Option<&CameraSubject>,
        current: &Position,
    ) -> Position {
        let q = ld(quantum);
        self.fov = st(ld(self.anim_fov) + ld(self.fov));
        let distance = ld(self.anim_scale).powf(q) * ld(self.camera_distance);
        self.camera_distance = st(distance);
        if distance == 0.0 {
            self.camera_distance = 0.1;
        }
        if self.camera_distance > MAX_DISTANCE {
            self.camera_distance = MAX_DISTANCE;
        }
        if !self.mapview && self.anim_pitch != 0.0 {
            self.viewer_offset.z = st(q * ld(self.anim_pitch) + ld(self.viewer_offset.z));
            self.viewer_offset.z = self.viewer_offset.z.clamp(-MAX_PITCH, MAX_PITCH);
        }
        let mut pitch = self.viewer_offset.z;
        if self.align_to_slope && !self.mapview && !self.mouse_look {
            if let Some(player) = subject {
                self.calc_align_to_slope(player, quantum);
            }
            let raised = ld(pitch) + ld(self.pitch_from_slope);
            pitch = st(raised);
            if raised > ld(MAX_PITCH) {
                pitch = MAX_PITCH;
            }
            if ld(pitch) < -ld(MAX_PITCH) {
                pitch = -MAX_PITCH;
            }
        }
        if !self.mapview && self.anim_heading != 0.0 {
            let heading = direction_heading(self.viewer_offset.x, self.viewer_offset.y);
            let degrees =
                ld(RADIANS_TO_DEGREES) * q * ld(self.anim_heading) + ld(heading.degrees());
            let turned = Heading::from_degrees(st(degrees));
            let (x, y) = (ld(self.viewer_offset.x), ld(self.viewer_offset.y));
            let length = ld(st((x * x + y * y).sqrt()));
            self.viewer_offset.x = st(radians(turned).sin() * length);
            self.viewer_offset.y = st(radians(turned).cos() * length);
        }
        self.ease_shoulder(quantum);
        if let Some(player) = subject {
            let mut targeting = std::mem::take(&mut self.targeting);
            targeting.update(self, player, quantum);
            self.targeting = targeting;
        }
        let heading = direction_heading(self.viewer_offset.x, self.viewer_offset.y);
        let angles = Vector::new(-pitch, st(radians(heading)), 0.0);
        let viewer = rotate_angles_forward(angles);
        let mut targeting = std::mem::take(&mut self.targeting);
        let position = targeting.calc_viewer_pos(self, subject, viewer, current);
        self.targeting = targeting;
        position
    }

    /// `UpdateCamera`'s shoulder easing: `m_currentShoulderOffset` moves from the start to the
    /// target offset over a second; at the end the target snaps back to the current offset.
    fn ease_shoulder(&mut self, quantum: f32) {
        let far = |a: f32, b: f32| (ld(a) - ld(b)).abs() >= ld(F_EPSILON);
        let (t, c) = (self.target_shoulder, self.current_shoulder);
        if !(far(t.x, c.x) || far(t.y, c.y) || far(t.z, c.z)) {
            return;
        }
        let time = ld(quantum) + ld(self.interp_time);
        self.interp_time = st(time);
        if time >= 1.0 {
            self.target_shoulder = self.current_shoulder;
            self.interp_time = 0.0;
            return;
        }
        let s = self.start_shoulder;
        let lerp = |target: f32, start: f32| st((ld(target) - ld(start)) * time + ld(start));
        self.current_shoulder = Vector::new(lerp(t.x, s.x), lerp(t.y, s.y), lerp(t.z, s.z));
    }

    /// The camera's preference values, as the bank holds them (`CameraManager::HandleApply` and
    /// the handlers of the keys [`crate::prefs`] registers). The distance and the tilt are the
    /// defaults the camera returns to at its next reset (`SetDefaults`), not the live camera.
    pub fn apply_prefs(&mut self, values: &CameraValues) {
        self.prefs_distance = values.distance;
        self.prefs_pitch = values.tilt;
        self.rotate_scale = values.rotate_speed;
        self.mouse_sensitivity = values.mouse_sensitivity;
        self.invert_y = values.invert_y;
        self.third_person_steers = values.steer_player;
        self.align_to_slope = values.align_to_slope;
    }

    /// `CameraManager::UseTime`'s field of view: 65 plus the camera effects' sum (none run).
    pub fn fov(&self) -> f32 {
        self.fov
    }

    /// `CameraManager::NoRayTrace`: first person, or a target other than the player.
    pub fn no_ray_trace(&self) -> bool {
        self.first_person || !self.targeting.target_is_player
    }

    /// `GetFirstPersonIsReady`.
    pub fn first_person_ready(&self) -> bool {
        self.first_person && self.targeting.first_person_ready()
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
