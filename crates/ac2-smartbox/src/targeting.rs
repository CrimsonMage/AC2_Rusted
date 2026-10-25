//! `TargetingModule` (Legions `portal/client/src/smartbox/camera/targetingmodule.cpp`, every
//! function an exact instruction match of admin 7148): where the camera stands for its target,
//! and the timed blend between an old and a new target (`TurnOn`, `FakeTransition`).
//!
//! The client's camera only ever targets the player with no parent and no bone, so the parent
//! and bone branches of `InternalCalcViewerPos` are not reachable here and are left out.
//! Float order follows the x87 code (53-bit precision): `ld`/`st` mark loads and `float` stores.

use ac2_physics::{Frame, Position, Quaternion, Vector};

use crate::camera::{ld, st, CameraManager, CameraSubject, F_EPSILON};

/// `GetElement`'s water element: the first-person camera leans a whole metre forward in it.
const WATER_ELEMENT: u32 = 0x4000_0003;

/// `TargetingModule`, reduced to a player target.
#[derive(Debug, Clone)]
pub struct TargetingModule {
    /// `mTargetID == mPlayerID` (otherwise `INVALID_IID`), and the same for `mOldTargetID`.
    pub target_is_player: bool,
    old_target_is_player: bool,
    pub has_player: bool,
    is_on: bool,
    time: f32,
    period: f32,
    hold: bool,
    ramp_time: f32,
    weight: f32,
    weight_vel: f32,
    pub camera_distance: f32,
    pub camera_height: f32,
    time_running: f32,
    time_not_running: f32,
    pub fake_trans_time: f32,
    viewer_offset: Vector,
    old_viewer_offset: Vector,
    /// `mLastTargetPos`; `None` while it holds no valid cell (`Position::IsValid`).
    last_target_pos: Option<Position>,
    pub auto_distance: bool,
}

impl Default for TargetingModule {
    /// `TargetingModule::TargetingModule`.
    fn default() -> Self {
        Self {
            target_is_player: false,
            old_target_is_player: false,
            has_player: false,
            is_on: false,
            time: 0.0,
            period: 1.0,
            hold: false,
            ramp_time: 0.1,
            weight: 0.0,
            weight_vel: 0.0,
            camera_distance: 3.5,
            camera_height: 0.0,
            time_running: 0.0,
            time_not_running: 0.0,
            fake_trans_time: 1.0,
            viewer_offset: Vector::new(0.0, -3.5, 0.0),
            old_viewer_offset: Vector::new(0.0, -3.5, 0.0),
            last_target_pos: None,
            auto_distance: true,
        }
    }
}

/// `D3DXVec3Normalize` of a vector given as stored floats.
fn normalize(v: Vector) -> Vector {
    v.d3dx_normalize()
}

impl TargetingModule {
    /// `TargetingModule::SetPlayer`: the player becomes the target, the old target and the
    /// player id.
    pub fn set_player(&mut self, present: bool) {
        self.has_player = present;
        self.target_is_player = present;
        self.old_target_is_player = present;
    }

    /// `TargetingModule::TurnOn`.
    fn turn_on(&mut self, period: f32, ramp_time: f32) {
        self.is_on = true;
        self.time = 0.0;
        if period == -1.0 {
            self.period = 1.0;
            self.hold = true;
        } else {
            self.period = period;
            self.hold = false;
        }
        self.ramp_time = ramp_time;
        if ramp_time == 0.0 {
            self.weight = 1.0;
            self.weight_vel = 0.0;
        } else {
            self.weight = 0.0;
            self.weight_vel = st(1.0 / ld(ramp_time));
        }
    }

    /// `TargetingModule::FakeTransition`: forget the old target and blend in from the
    /// camera's current position over `mFakeTransTime`.
    pub fn fake_transition(&mut self) {
        self.old_target_is_player = false;
        self.turn_on(self.fake_trans_time, self.fake_trans_time);
    }

    /// `TargetingModule::GetPosition` for the target (`_bDrawingPos` reads the render-slerped
    /// position, which the headless client does not keep: the physics position stands in).
    fn target_position(
        &self,
        subject: Option<&CameraSubject>,
        is_player: bool,
    ) -> Option<Position> {
        is_player.then(|| subject.map(|s| s.position)).flatten()
    }

    /// `TargetingModule::StoreTargetPos` with bone 0.
    pub fn store_target_pos(
        &mut self,
        subject: Option<&CameraSubject>,
        target_is_player: bool,
        period: f32,
        ramp_time: f32,
    ) {
        if self.target_is_player == target_is_player {
            return;
        }
        if let Some(position) = self.target_position(subject, self.target_is_player) {
            self.last_target_pos = Some(position);
        }
        self.old_target_is_player = self.target_is_player;
        self.target_is_player = target_is_player;
        if self.last_target_pos.is_some() {
            self.turn_on(period, ramp_time);
        }
    }

    /// `TargetingModule::CalcCameraDistance`: the camera distance, which the auto-distance
    /// option eases out (to at most 5 m) while the player runs and back in after a second at
    /// rest, when the chosen distance is 1.5 m or less.
    fn calc_camera_distance(
        &mut self,
        camera: &CameraManager,
        subject: &CameraSubject,
        quantum: f32,
    ) {
        let distance = camera.get_camera_distance();
        if distance > 1.5 || !self.auto_distance {
            self.camera_distance = distance;
            return;
        }
        let v = subject.local_velocity;
        let speed2 = (ld(v.z) * ld(v.z) + ld(v.y) * ld(v.y)) + ld(v.x) * ld(v.x);
        let running = speed2 > 2.0;
        if running {
            self.time_running = st(ld(quantum) + ld(self.time_running));
            self.time_not_running = 0.0;
        } else {
            self.time_not_running = st(ld(quantum) + ld(self.time_not_running));
            self.time_running = 0.0;
        }
        self.time_running = self.time_running.clamp(0.0, 6.0);
        if camera.first_person || camera.mapview {
            self.camera_distance = distance;
            return;
        }
        let wanted = st(ld(distance) + 1.0);
        let current = st(ld(self.camera_distance) + 1.0);
        self.camera_distance = current;
        if ld(self.time_running) > 2.5 && ld(current) <= 5.5 {
            if running {
                let eased = ld(current).powf(ld(1.07_f32));
                self.camera_distance = st(eased);
                if eased > 5.0 && ld(wanted) < 5.0 {
                    self.camera_distance = 5.0;
                }
            }
        } else if !running && current != wanted && ld(self.time_not_running) > 1.0 {
            let eased = ld(current).powf(ld(0.97_f32));
            self.camera_distance = st(eased);
            if eased < ld(wanted) {
                self.camera_distance = wanted;
            }
        }
        self.camera_distance = st(ld(self.camera_distance) - 1.0);
    }

    /// `TargetingModule::Update`: the camera distance, then the blend clock; a blend that is
    /// done (or has nothing to blend from) turns off, and a timed (not held) target returns to
    /// the player.
    pub fn update(&mut self, camera: &CameraManager, subject: &CameraSubject, quantum: f32) {
        self.calc_camera_distance(camera, subject, quantum);
        if !self.is_on {
            if let Some(position) = self.target_position(Some(subject), self.target_is_player) {
                self.last_target_pos = Some(position);
            }
        }
        self.time = st(ld(quantum) + ld(self.time));
        if !(self.weight < 1.0 && self.last_target_pos.is_some()) {
            self.is_on = false;
        }
        if ld(self.period) - ld(self.ramp_time) < ld(self.time)
            && !self.hold
            && !self.target_is_player
            && self.has_player
        {
            self.store_target_pos(Some(subject), true, -1.0, 2.0);
        }
        if self.ramp_time > self.time {
            self.weight = st(ld(self.time) / ld(self.ramp_time));
        } else if self.ramp_time < self.time {
            self.weight = 1.0;
        }
        self.weight = self.weight.clamp(0.0, 1.0);
    }

    /// `TargetingModule::InternalCalcViewerPos` for a player target without parent or bone:
    /// behind the player along `-viewer_offset`, at the camera distance (scaled by the body's
    /// z scale), lifted by the camera height and the body's height, plus the shoulder offset and
    /// kept at least 0.5 m from the player horizontally. First person sits at 90% of the body's
    /// height, leaning forward. With no target the `fallback` position is the answer.
    fn internal_calc_viewer_pos(
        &self,
        camera: &mut CameraManager,
        subject: Option<&CameraSubject>,
        target_is_player: bool,
        fallback: &Position,
    ) -> Position {
        let target = if target_is_player { subject } else { None };
        let mut global = self.viewer_offset;
        if !camera.mouse_look {
            if let Some(player) = subject {
                global = player
                    .position
                    .frame
                    .local_to_global_vec(self.viewer_offset);
            }
        }
        let Some(target) = target else {
            return *fallback;
        };
        let target_pos = target.position;
        let mut frame = Frame::new(target_pos.frame.origin, Quaternion::new(1.0, 0.0, 0.0, 0.0));
        frame.set_vector_heading(Vector::new(-global.x, -global.y, -global.z));
        let mut origin = frame.origin;
        if !camera.first_person {
            let d = ld(self.camera_distance);
            let scale = ld(target.scale_z);
            let ay = ld(st(ld(global.y) * d));
            let az = st(ld(st(ld(global.z) * d)) * scale);
            origin.x = st(ld(global.x) * d * scale + ld(origin.x));
            origin.y = st(ay * scale + ld(origin.y));
            origin.z = st(ld(az) + ld(origin.z) + ld(self.camera_height));
            // `IsDummyObject` is false for the player: its height lifts the camera.
            origin.z = st(target.height + ld(origin.z));
            let shoulder = camera.calc_shoulder_offset(Some(target), self.camera_distance);
            origin.x = st(ld(origin.x) + ld(shoulder.x));
            origin.y = st(ld(origin.y) + ld(shoulder.y));
            origin.z = st(ld(origin.z) + ld(shoulder.z));
            let (tx, ty) = (target_pos.frame.origin.x, target_pos.frame.origin.y);
            let dx = ld(origin.x) - ld(tx);
            let dy = ld(origin.y) - ld(ty);
            let length = (dy * dy + dx * dx).sqrt();
            if length > 0.0 && length < 0.5 {
                let away = normalize(Vector::new(st(dx), st(dy), 0.0));
                origin.x = st(ld(away.x) * 0.5 + ld(tx));
                origin.y = st(ld(away.y) * 0.5 + ld(ty));
            }
        } else if !target.no_draw {
            let forward = normalize(Vector::new(frame.m[3], frame.m[4], 0.0));
            // `IsThePlayer`.
            origin.z = st(target.height * ld(0.9_f32) + ld(origin.z));
            let speed = target.local_velocity.y;
            let lean = if target.element == WATER_ELEMENT {
                1.0
            } else if ld(speed) > ld(F_EPSILON) || -ld(F_EPSILON) > ld(speed) {
                (ld(speed).abs() * ld(0.1_f32) + 1.0) * ld(0.4_f32)
            } else {
                ld(0.4_f32)
            };
            let y = st(ld(forward.y) * lean);
            let z = st(lean * ld(forward.z));
            origin.x = st(ld(forward.x) * lean + ld(origin.x));
            origin.y = st(ld(y) + ld(origin.y));
            origin.z = st(ld(z) + ld(origin.z));
        }
        frame.origin = origin;
        Position::new(target_pos.cell_id, frame)
    }

    /// `TargetingModule::CalcViewerPos`: the viewer position for `viewer_offset`; during a blend
    /// (`mIsOn`) the old target's and the new one's, origin lerped and rotation slerped by the
    /// blend weight.
    pub fn calc_viewer_pos(
        &mut self,
        camera: &mut CameraManager,
        subject: Option<&CameraSubject>,
        viewer_offset: Vector,
        current: &Position,
    ) -> Position {
        self.viewer_offset = viewer_offset;
        if !self.is_on {
            let position =
                self.internal_calc_viewer_pos(camera, subject, self.target_is_player, current);
            self.old_viewer_offset = viewer_offset;
            return position;
        }
        let from =
            self.internal_calc_viewer_pos(camera, subject, self.old_target_is_player, current);
        let to = self.internal_calc_viewer_pos(camera, subject, self.target_is_player, current);
        let mut result = Position::new(0, Frame::default());
        result.lerp_origin(&from, &to, self.weight);
        result
            .frame
            .slerp_rotation(&from.frame, &to.frame, self.weight);
        result
    }

    /// `GetFirstPersonIsReady`'s test: the player targeted at the first-person distance.
    pub fn first_person_ready(&self) -> bool {
        self.target_is_player && self.camera_distance == 0.01
    }
}
