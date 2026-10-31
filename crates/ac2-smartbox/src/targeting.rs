//! `TargetingModule` (Legions `portal/client/src/smartbox/camera/targetingmodule.cpp`, every
//! function an exact instruction match of admin 7148): where the camera stands for its parent
//! and its target, and the timed blend between the old and the new ones (`TurnOn`,
//! `FakeTransition`). The player camera targets the player with no parent; behaviours and camera
//! effects move the target and the parent to other objects, to bone types on them, and to the
//! camera's two dummy objects on the player (`CameraManager::SetupCameraDummyObjects`), and the
//! camera returns to the player when their period runs out.
//!
//! Where an object stands is the caller's: `GetPosition`'s `_bDrawingPos` choice between the
//! object's drawing position (its previous and current positions slerped by the render
//! interpolation) and its physics position is made by whoever builds the [`CameraView`] and
//! [`CameraObjects`]: the client hands over drawn positions every frame (`AGENTS.md`'s camera
//! exception) and physics ones for the behaviour events. A bone is always the drawn one
//! (`GetBonePosition`). The one case not modelled is a child object (`CPhysicsObj::m_pParent`
//! set, vtable `0x560`), whose drawing position `GetPosition` takes whatever the flag. Float
//! order follows the x87 code (53-bit precision): `ld`/`st` mark loads and `float` stores.

use ac2_physics::{Frame, Position, Quaternion, Vector};

use crate::camera::{ld, st, CameraManager, CameraSubject, F_EPSILON};

/// `GetElement`'s water element: the first-person camera leans a whole metre forward in it.
const WATER_ELEMENT: u32 = 0x4000_0003;
/// `INVALID_IID`.
pub const INVALID_IID: u64 = 0;
/// `IIDUtils::GetCameraParent` and `GetCameraTarget`: the camera's dummy objects.
pub const CAMERA_PARENT_ID: u64 = 0x1200_0000_0000_0FFF;
pub const CAMERA_TARGET_ID: u64 = 0x1200_0000_0000_0FFE;
/// The player's bone types the dummies are attached to (`SetupCameraDummyObjects`); the
/// target's is also the head the dynamic first-person camera rides (`UpdateCamera`).
pub const CAMERA_PARENT_BONE: u32 = 0x4000_0004;
pub const CAMERA_TARGET_BONE: u32 = 0x4000_0005;

/// The objects other than the player the camera can follow.
pub trait CameraObjects {
    /// What the camera reads from object `id`'s physics object, `None` when it is not known
    /// (`ICPhysicsObj::GetObjectA`). Never asked for the player.
    fn subject(&self, id: u64) -> Option<CameraSubject>;
    /// `CPhysicsObj::GetBonePosition(bone, offset)`: object `id`'s bone of type `bone` (offset by
    /// `offset` in the bone's space), placed by the object's position and scale. `None` when the
    /// object or the bone type is missing. Asked for the player too.
    fn bone_position(&self, id: u64, bone: u32, offset: Vector) -> Option<Position>;
}

/// No objects but the player.
pub struct NoObjects;

impl CameraObjects for NoObjects {
    fn subject(&self, _id: u64) -> Option<CameraSubject> {
        None
    }
    fn bone_position(&self, _id: u64, _bone: u32, _offset: Vector) -> Option<Position> {
        None
    }
}

/// What the camera sees this update: the player and the other objects.
#[derive(Clone, Copy)]
pub struct CameraView<'a> {
    pub player: Option<&'a CameraSubject>,
    pub objects: &'a dyn CameraObjects,
}

/// An object as `InternalCalcViewerPos` reads it.
#[derive(Debug, Clone, Copy)]
struct Seen {
    subject: CameraSubject,
    /// `IsThePlayer` and `IsDummyObject`.
    is_player: bool,
    is_dummy: bool,
}

impl CameraView<'_> {
    /// `ICPhysicsObj::GetObjectA(id)`. A camera dummy is there while the player has its bone
    /// type, standing on that bone.
    fn object(&self, id: u64, player_id: u64) -> Option<Seen> {
        if id == INVALID_IID {
            return None;
        }
        if let Some(player) = self.player.filter(|p| p.id == id && id == player_id) {
            return Some(Seen {
                subject: *player,
                is_player: true,
                is_dummy: false,
            });
        }
        let bone = match id {
            CAMERA_PARENT_ID => Some(CAMERA_PARENT_BONE),
            CAMERA_TARGET_ID => Some(CAMERA_TARGET_BONE),
            _ => None,
        };
        if let Some(bone) = bone {
            let zero = Vector::new(0.0, 0.0, 0.0);
            let position = self.objects.bone_position(player_id, bone, zero)?;
            return Some(Seen {
                subject: CameraSubject {
                    id,
                    position,
                    height: 0.0,
                    radius: 0.0,
                    scale_z: 1.0,
                    local_velocity: zero,
                    element: 0,
                    no_draw: true,
                },
                is_player: false,
                is_dummy: true,
            });
        }
        Some(Seen {
            subject: self.objects.subject(id)?,
            is_player: false,
            is_dummy: false,
        })
    }
}

/// `TargetingModule`.
#[derive(Debug, Clone)]
pub struct TargetingModule {
    /// `mPlayerID`, `mTargetID`/`mTargetBone`, `mOldTargetID`/`mOldTargetBone`, `mParentID`/
    /// `mParentBone` and `mOldParentID`/`mOldParentBone`.
    pub player_id: u64,
    pub target_id: u64,
    pub target_bone: u32,
    old_target_id: u64,
    old_target_bone: u32,
    pub parent_id: u64,
    pub parent_bone: u32,
    old_parent_id: u64,
    old_parent_bone: u32,
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
    /// `mLastTargetPos` and `mLastParentPos`; `None` while they hold no valid cell
    /// (`Position::IsValid`).
    last_target_pos: Option<Position>,
    last_parent_pos: Option<Position>,
    pub auto_distance: bool,
    /// `InternalCalcViewerPos`' `player->LookAt(mTargetID)` (vtable `0x4f8`): the target the
    /// player's head turned to this update, for the client to apply ([`Self::take_look_at`]).
    look_at: std::cell::Cell<Option<u64>>,
}

impl Default for TargetingModule {
    /// `TargetingModule::TargetingModule`.
    fn default() -> Self {
        Self {
            player_id: INVALID_IID,
            target_id: INVALID_IID,
            target_bone: 0,
            old_target_id: INVALID_IID,
            old_target_bone: 0,
            parent_id: INVALID_IID,
            parent_bone: 0,
            old_parent_id: INVALID_IID,
            old_parent_bone: 0,
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
            last_parent_pos: None,
            auto_distance: true,
            look_at: std::cell::Cell::new(None),
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
    pub fn set_player(&mut self, id: u64) {
        self.player_id = id;
        self.old_target_id = id;
        self.target_id = id;
    }

    /// Whether the camera targets the player (`mTargetID == mPlayerID`).
    pub fn target_is_player(&self) -> bool {
        self.target_id == self.player_id
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

    /// `TargetingModule::FakeTransition`: forget the old target, drop the parent, and blend in
    /// from the camera's current position over `mFakeTransTime`.
    pub fn fake_transition(&mut self) {
        self.old_target_id = INVALID_IID;
        self.old_parent_id = self.parent_id;
        self.parent_id = INVALID_IID;
        self.turn_on(self.fake_trans_time, self.fake_trans_time);
    }

    /// `TargetingModule::GetPosition`: the object's position, or its bone's
    /// (`GetBonePosition` with no offset). A bone retail cannot find leaves the position
    /// unwritten; here it is missing.
    fn get_position(&self, view: &CameraView<'_>, id: u64, bone: u32) -> Option<Position> {
        let seen = view.object(id, self.player_id)?;
        if bone != 0 {
            return view
                .objects
                .bone_position(id, bone, Vector::new(0.0, 0.0, 0.0));
        }
        Some(seen.subject.position)
    }

    /// `TargetingModule::StoreTargetPos`: a new target (or bone) keeps where the old one was and
    /// blends from it over `ramp_time`, holding the new one for `period` (`-1` holds it).
    pub fn store_target_pos(
        &mut self,
        view: &CameraView<'_>,
        id: u64,
        bone: u32,
        period: f32,
        ramp_time: f32,
    ) {
        if self.target_id == id && self.target_bone == bone {
            return;
        }
        if let Some(position) = self.get_position(view, self.target_id, self.target_bone) {
            self.last_target_pos = Some(position);
        }
        self.old_target_id = self.target_id;
        self.old_target_bone = self.target_bone;
        self.target_id = id;
        self.target_bone = bone;
        if self.last_target_pos.is_some() {
            self.turn_on(period, ramp_time);
        }
    }

    /// `TargetingModule::StoreParentPos`: as [`Self::store_target_pos`] for the parent, which
    /// changes only with its id.
    pub fn store_parent_pos(
        &mut self,
        view: &CameraView<'_>,
        id: u64,
        bone: u32,
        period: f32,
        ramp_time: f32,
    ) {
        if self.parent_id == id {
            return;
        }
        if let Some(position) = self.get_position(view, self.parent_id, self.parent_bone) {
            self.last_parent_pos = Some(position);
        }
        self.old_parent_id = self.parent_id;
        self.parent_id = id;
        self.old_parent_bone = self.parent_bone;
        self.parent_bone = bone;
        if self.last_parent_pos.is_some() {
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
    /// done (or has nothing to blend from) turns off, and a timed (not held) target or parent
    /// other than the player returns the camera to the player over 2 seconds.
    pub fn update(
        &mut self,
        camera: &CameraManager,
        view: &CameraView<'_>,
        subject: &CameraSubject,
        quantum: f32,
    ) {
        self.calc_camera_distance(camera, subject, quantum);
        if !self.is_on {
            if let Some(position) = self.get_position(view, self.parent_id, self.parent_bone) {
                self.last_parent_pos = Some(position);
            }
            if let Some(position) = self.get_position(view, self.target_id, self.target_bone) {
                self.last_target_pos = Some(position);
            }
        }
        self.time = st(ld(quantum) + ld(self.time));
        if !(self.weight < 1.0
            && (self.last_parent_pos.is_some() || self.last_target_pos.is_some()))
        {
            self.is_on = false;
        }
        if ld(self.period) - ld(self.ramp_time) < ld(self.time)
            && !self.hold
            && (self.target_id != self.player_id || self.parent_id != INVALID_IID)
        {
            self.store_target_pos(view, self.player_id, 0, -1.0, 2.0);
            self.store_parent_pos(view, INVALID_IID, 0, -1.0, 2.0);
        }
        if self.ramp_time > self.time {
            self.weight = st(ld(self.time) / ld(self.ramp_time));
        } else if self.ramp_time < self.time {
            self.weight = 1.0;
        }
        self.weight = self.weight.clamp(0.0, 1.0);
    }

    /// `TargetingModule::InternalCalcViewerPos` for one parent and target (`use_last` takes the
    /// last known positions of ones that are gone, for the old pair of a blend):
    /// - no parent: behind the target along `-viewer_offset`, at the camera distance (scaled by
    ///   the body's z scale), lifted by the camera height and the body's height, plus the
    ///   shoulder offset and kept at least 0.5 m from it horizontally; on a target bone, at the
    ///   bone. First person sits at 90% of the player's height, leaning forward.
    /// - a parent and a target: at the parent looking at the target; on a parent that is the
    ///   player (with no target bone), a third of the camera distance back from the player
    ///   along the line from the target, lifted as above; then the parent's height.
    /// - a parent alone: at the parent facing its heading, a third of the way along the viewer
    ///   offset, then its height.
    ///
    /// With neither, the `fallback` position is the answer.
    #[allow(clippy::too_many_arguments)]
    fn internal_calc_viewer_pos(
        &self,
        camera: &mut CameraManager,
        view: &CameraView<'_>,
        use_last: bool,
        (parent_id, parent_bone): (u64, u32),
        (target_id, target_bone): (u64, u32),
        fallback: &Position,
    ) -> Position {
        let target = view.object(target_id, self.player_id);
        let parent = (parent_id != INVALID_IID)
            .then(|| view.object(parent_id, self.player_id))
            .flatten();
        let mut parent_pos = self.get_position(view, parent_id, parent_bone);
        if parent_pos.is_none() && use_last && parent_id != INVALID_IID {
            parent_pos = self.last_parent_pos;
        }
        let mut target_pos = self.get_position(view, target_id, target_bone);
        if target_pos.is_none() && use_last && target_id != INVALID_IID {
            target_pos = self.last_target_pos;
        }
        let none = Position::new(0, Frame::default());
        let (parent_pos, target_pos) = (parent_pos.unwrap_or(none), target_pos.unwrap_or(none));
        let mut global = self.viewer_offset;
        if !camera.mouse_look {
            if let Some(player) = view.player {
                global = player
                    .position
                    .frame
                    .local_to_global_vec(self.viewer_offset);
            }
        }
        let forward_scale = target
            .filter(|t| t.is_player)
            .map_or(1.0, |t| t.subject.scale_z);
        let parent_known = parent.is_some() || (use_last && parent_id != INVALID_IID);
        let target_known = target.is_some() || (use_last && target_id != INVALID_IID);
        if !parent_known {
            // Retail reads the target object here; one known only by its last position is not
            // there to read (a crash in retail), so the fallback stands.
            let Some(target) = target.filter(|_| target_known) else {
                return *fallback;
            };
            return self.behind_target(camera, view, &target, &target_pos, global);
        }
        let identity = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let mut frame = Frame::new(parent_pos.frame.origin, identity);
        let mut origin = parent_pos.frame.origin;
        let d = ld(self.camera_distance);
        let third = ld(0.333_333_34_f32);
        let fwd = ld(forward_scale);
        let lift = match target_known {
            false => {
                // A parent alone: its heading, a third of the way out along the viewer offset.
                frame.set_vector_heading(parent_pos.frame.vector_heading());
                let Some(_) = parent else {
                    return Position::new(parent_pos.cell_id, frame);
                };
                let y = st(ld(global.y) * d);
                let z = st(ld(global.z) * d * fwd);
                origin.x = st(ld(origin.x) + ld(st(ld(global.x) * d * fwd * third)));
                origin.y = st(ld(origin.y) + ld(y) * fwd * third);
                origin.z = st(ld(origin.z) + ld(z) * third);
                true
            }
            true => {
                // Looking from the parent's top at the target.
                let mut top = parent_pos.frame.origin;
                if let Some(p) = parent.filter(|p| !p.is_dummy) {
                    top.z = st(p.subject.height + ld(top.z));
                }
                let t = target_pos.frame.origin;
                let dir = Vector::new(
                    st(ld(t.x) - ld(top.x)),
                    st(ld(t.y) - ld(top.y)),
                    st(ld(t.z) - ld(top.z)),
                );
                frame.set_vector_heading(dir);
                let Some(p) = parent else {
                    return Position::new(parent_pos.cell_id, frame);
                };
                // On the player, whose head turns to the target (`LookAt`, for the current pair
                // only), back from it along the line.
                if p.is_player && !use_last {
                    self.look_at.set(Some(self.target_id));
                }
                if p.is_player && self.target_bone == 0 {
                    let away = normalize(Vector::new(-dir.x, -dir.y, -dir.z));
                    let y = st(ld(away.y) * d);
                    let z = st(ld(away.z) * d * fwd);
                    origin.x = st(ld(st(ld(away.x) * d * fwd * third)) + ld(origin.x));
                    origin.y = st(ld(y) * fwd * third + ld(origin.y));
                    origin.z = st(ld(st(ld(z) * third)) + ld(origin.z) + ld(self.camera_height));
                    true
                } else {
                    false
                }
            }
        };
        if lift {
            let shoulder = camera.calc_shoulder_offset(view.player, self.camera_distance);
            origin.x = st(ld(origin.x) + ld(shoulder.x));
            origin.y = st(ld(origin.y) + ld(shoulder.y));
            origin.z = st(ld(origin.z) + ld(shoulder.z));
        }
        if let Some(p) = parent.filter(|p| !p.is_dummy) {
            origin.z = st(p.subject.height + ld(origin.z));
        }
        frame.origin = origin;
        Position::new(parent_pos.cell_id, frame)
    }

    /// `InternalCalcViewerPos` with no parent: behind `target` (see above).
    fn behind_target(
        &self,
        camera: &mut CameraManager,
        view: &CameraView<'_>,
        target: &Seen,
        target_pos: &Position,
        global: Vector,
    ) -> Position {
        let subject = &target.subject;
        let mut frame = Frame::new(target_pos.frame.origin, Quaternion::new(1.0, 0.0, 0.0, 0.0));
        frame.set_vector_heading(Vector::new(-global.x, -global.y, -global.z));
        let mut origin = frame.origin;
        if !camera.first_person {
            if self.target_bone == 0 {
                let d = ld(self.camera_distance);
                let scale = ld(subject.scale_z);
                let ay = ld(st(ld(global.y) * d));
                let az = st(ld(st(ld(global.z) * d)) * scale);
                origin.x = st(ld(global.x) * d * scale + ld(origin.x));
                origin.y = st(ay * scale + ld(origin.y));
                origin.z = st(ld(az) + ld(origin.z) + ld(self.camera_height));
                if !target.is_dummy {
                    origin.z = st(subject.height + ld(origin.z));
                }
                let shoulder = camera.calc_shoulder_offset(view.player, self.camera_distance);
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
            }
        } else if !subject.no_draw && target.is_player {
            let forward = normalize(Vector::new(frame.m[3], frame.m[4], 0.0));
            origin.z = st(subject.height * ld(0.9_f32) + ld(origin.z));
            let speed = subject.local_velocity.y;
            let lean = if subject.element == WATER_ELEMENT {
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

    /// `TargetingModule::GetInterpViewerOffset` (admin 0x00579420): the viewer offset, during a
    /// blend (`mIsOn`) the old and the new ones lerped by the blend's time.
    pub fn interp_viewer_offset(&self) -> Vector {
        if !self.is_on {
            return self.viewer_offset;
        }
        let (t, u) = (self.time, 1.0 - self.time);
        let (a, b) = (self.viewer_offset, self.old_viewer_offset);
        Vector::new(t * a.x + u * b.x, t * a.y + u * b.y, t * a.z + u * b.z)
    }

    /// `TargetingModule::CalcViewerPos`: the viewer position for `viewer_offset`; during a blend
    /// (`mIsOn`) the old pair's (from their last positions if gone) and the new one's, origin
    /// lerped and rotation slerped by the blend weight.
    pub fn calc_viewer_pos(
        &mut self,
        camera: &mut CameraManager,
        view: &CameraView<'_>,
        viewer_offset: Vector,
        current: &Position,
    ) -> Position {
        self.viewer_offset = viewer_offset;
        let new = (
            (self.parent_id, self.parent_bone),
            (self.target_id, self.target_bone),
        );
        if !self.is_on {
            let position =
                self.internal_calc_viewer_pos(camera, view, false, new.0, new.1, current);
            self.old_viewer_offset = viewer_offset;
            return position;
        }
        let old = (
            (self.old_parent_id, self.old_parent_bone),
            (self.old_target_id, self.old_target_bone),
        );
        let from = self.internal_calc_viewer_pos(camera, view, true, old.0, old.1, current);
        let to = self.internal_calc_viewer_pos(camera, view, false, new.0, new.1, current);
        let mut result = Position::new(0, Frame::default());
        result.lerp_origin(&from, &to, self.weight);
        result
            .frame
            .slerp_rotation(&from.frame, &to.frame, self.weight);
        result
    }

    /// The target the player's head turned to at the last viewer update
    /// (`InternalCalcViewerPos`' `player->LookAt(mTargetID)`), taken once.
    pub fn take_look_at(&self) -> Option<u64> {
        self.look_at.take()
    }

    /// `GetFirstPersonIsReady`'s test: the player targeted with no bone at the first-person
    /// distance.
    pub fn first_person_ready(&self) -> bool {
        self.target_is_player() && self.target_bone == 0 && self.camera_distance == 0.01
    }
}
