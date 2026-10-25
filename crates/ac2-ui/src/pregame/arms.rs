//! Character selection's arms: one `UISceneEntity` per character slot, turned about the scene's
//! vertical axis so the selected slot faces the camera. The first arm is unturned, the second
//! turned back by the small angle (45 degrees) and each further one by the large angle more,
//! the large angle sharing what the two small angles leave of the circle among the other slots
//! (`gmCharacterSelectionUI::InitArms`). Moving the selection one slot turns every arm by the
//! large angle, except the selected arm and its neighbour on the side it moves to, which turn by
//! the small angle (`MoveArm`, `MoveArms`); the turn slerps over `_moveDuration` (1.5 s), or is
//! immediate. Each UI loop the arms step toward the target slot by the shorter way round while
//! the first arm is not turning, or all the way at once after a direct pick
//! (`ListenToMessage(UIMESSAGE_GLOBAL_LOOP)`).
//!
//! The entities' slerp is `UISceneEntity::SLERPTo` / `Think` over `Frame::slerp`
//! (`D3DXQuaternionSlerp`); headings follow `Heading`'s arithmetic (`fmod` 360, kept in
//! [0, 360) and stored as a `float`).

use ac2_math::{Frame, Quat, Vec3};

/// `Heading::set`: the angle `fmod` 360, a negative one moved up by 360 (to 0 if that rounds
/// to 360), stored as a `float`.
fn heading(degrees: f64) -> f32 {
    let mut value = degrees % 360.0;
    if value < 0.0 {
        value += 360.0;
        if value >= 360.0 {
            value = 0.0;
        }
    }
    value as f32
}

/// `D3DXQuaternionSlerp`: the shorter way round, linear when the two are within 0.001.
fn slerp(from: Quat, to: Quat, t: f32) -> Quat {
    let a = [from.x, from.y, from.z, from.w].map(f64::from);
    let b = [to.x, to.y, to.z, to.w].map(f64::from);
    let t = f64::from(t);
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let mut sign = 1.0;
    if dot < 0.0 {
        sign = -1.0;
        dot = -dot;
    }
    let (mut p, mut q) = (1.0 - t, t);
    if 1.0 - dot > 0.001 {
        let theta = dot.acos();
        p = (theta * p).sin() / theta.sin();
        q = (theta * q).sin() / theta.sin();
    }
    let lane = |i: usize| (p * a[i] + sign * q * b[i]) as f32;
    Quat::from_xyzw(lane(0), lane(1), lane(2), lane(3))
}

/// `Frame::slerp`: origins lerped, rotations slerped.
fn slerp_frame(from: &Frame, to: &Frame, t: f32) -> Frame {
    let lerp =
        |a: f32, b: f32| ((f64::from(b) - f64::from(a)) * f64::from(t) + f64::from(a)) as f32;
    Frame::new(
        Vec3::new(
            lerp(from.origin.x, to.origin.x),
            lerp(from.origin.y, to.origin.y),
            lerp(from.origin.z, to.origin.z),
        ),
        slerp(from.orientation, to.orientation, t),
    )
}

/// `UISceneEntity`'s frame and its timed move to another (`m_CurrentFrame`, `m_StartFrame`,
/// `m_EndFrame`, `m_StartUpdateTime`, `m_UpdateDuration`, `m_bSLERPing`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneEntity {
    frame: Frame,
    start: Frame,
    end: Frame,
    start_time: f64,
    duration: f64,
    slerping: bool,
}

impl Default for SceneEntity {
    fn default() -> Self {
        Self {
            frame: Frame::IDENTITY,
            start: Frame::IDENTITY,
            end: Frame::IDENTITY,
            start_time: 0.0,
            duration: 0.0,
            slerping: false,
        }
    }
}

impl SceneEntity {
    /// The entity's frame in its scene.
    pub fn frame(&self) -> Frame {
        self.frame
    }

    /// Whether a move is under way (`m_bSLERPing`).
    pub fn slerping(&self) -> bool {
        self.slerping
    }

    /// `UISceneEntity::SetFrame`.
    pub fn set_frame(&mut self, frame: Frame) {
        self.frame = frame;
    }

    /// `UISceneEntity::SLERPTo`: from the current frame to `frame` over `duration` seconds from
    /// `now` (`Timer::local_time`); a zero duration sets it at once.
    pub fn slerp_to(&mut self, frame: Frame, duration: f64, now: f64) {
        if duration == 0.0 {
            self.set_frame(frame);
            return;
        }
        self.start_time = now;
        self.duration = duration;
        self.start = self.frame;
        self.end = frame;
        self.slerping = true;
    }

    /// `UISceneEntity::Think`: the frame along the move; past its end, the end frame.
    pub fn think(&mut self, now: f64) {
        if !self.slerping {
            return;
        }
        let t = (now - self.start_time) / self.duration;
        if t > 1.0 {
            let end = self.end;
            self.set_frame(end);
            self.slerping = false;
            return;
        }
        self.frame = slerp_frame(&self.start, &self.end, t as f32);
    }
}

/// The arms and the selection they show (`_Arms`, `_currSelected`, `_targetSelected`,
/// `_numCharSlots`, `_smallAngle`, `_largeAngle`, `_moveDuration`, `_moveImmediate`).
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterArms {
    arms: Vec<SceneEntity>,
    small: f32,
    large: f32,
    move_duration: f32,
    slots: usize,
    current: usize,
    target: usize,
    move_immediate: bool,
}

impl Default for CharacterArms {
    /// `gmCharacterSelectionUI::gmCharacterSelectionUI`: 45 and 0 degrees, 1.5 s.
    fn default() -> Self {
        Self {
            arms: Vec::new(),
            small: 45.0,
            large: 0.0,
            move_duration: 1.5,
            slots: 0,
            current: 0,
            target: 0,
            move_immediate: false,
        }
    }
}

impl CharacterArms {
    /// `gmCharacterSelectionUI::InitArms` for `slots` character slots (the arms of a previous
    /// set are dropped first, `UnParentandDeleteArms`).
    pub fn init(&mut self, slots: usize) {
        self.arms.clear();
        self.slots = slots;
        let small = f64::from(heading(f64::from(self.small)));
        // `(float)(numCharSlots - 2)`, unsigned.
        let others = f64::from((slots as u32).wrapping_sub(2));
        self.large = heading(f64::from(((360.0 - (small + small)) / others) as f32));
        for arm in 1..=slots as u32 {
            let mut frame = Frame::IDENTITY;
            if arm != 1 {
                let scaled = heading(f64::from(
                    (f64::from(arm.wrapping_sub(2)) * f64::from(self.large)) as f32,
                ));
                let back = heading(f64::from(frame.heading_degrees()) - small);
                frame.set_heading(heading(f64::from(back) - f64::from(scaled)));
            }
            let mut entity = SceneEntity::default();
            entity.set_frame(frame);
            self.arms.push(entity);
        }
    }

    pub fn arms(&self) -> &[SceneEntity] {
        &self.arms
    }

    /// The slot the arms show (`_currSelected`) and the one they turn toward (`_targetSelected`).
    pub fn selected(&self) -> (usize, usize) {
        (self.current, self.target)
    }

    pub fn set_current(&mut self, slot: usize) {
        self.current = slot;
    }

    pub fn set_target(&mut self, slot: usize) {
        self.target = slot;
    }

    /// `_moveImmediate`: the next loop turns straight to the target.
    pub fn set_move_immediate(&mut self, immediate: bool) {
        self.move_immediate = immediate;
    }

    /// `gmCharacterSelectionUI::MoveArm`: arm `index` turned by the small or large angle, left
    /// (positive) or right.
    fn move_arm(
        &mut self,
        index: usize,
        small_move: bool,
        shift_left: bool,
        immediate: bool,
        now: f64,
    ) {
        let angle = if small_move { self.small } else { self.large };
        let angle = if shift_left {
            angle
        } else {
            heading(-f64::from(angle))
        };
        let duration = f64::from(self.move_duration);
        let Some(arm) = self.arms.get_mut(index) else {
            return;
        };
        let mut frame = arm.frame;
        let turned = heading(f64::from(frame.heading_degrees()) + f64::from(angle));
        frame.set_heading(turned);
        if immediate {
            arm.set_frame(frame);
        } else {
            arm.slerp_to(frame, duration, now);
        }
    }

    /// `gmCharacterSelectionUI::MoveArms`: every arm one slot along; nothing while the first
    /// arm is still turning.
    pub fn move_arms(&mut self, shift_left: bool, immediate: bool, now: f64) -> bool {
        if self.arms.first().is_none_or(|arm| arm.slerping) || self.slots == 0 {
            return false;
        }
        let n = self.slots;
        let neighbour = if shift_left {
            (n + 1 + self.current) % n
        } else {
            (n - 1 + self.current) % n
        };
        for index in 0..self.arms.len() {
            let small = index == self.current || index == neighbour;
            self.move_arm(index, small, shift_left, immediate, now);
        }
        let step = if shift_left { 1 } else { n - 1 };
        self.current = (self.current + step) % n;
        true
    }

    /// The arms' part of `ListenToMessage(UIMESSAGE_GLOBAL_LOOP)`: the entities move on
    /// (`UIScene` think), then the arms step toward the target slot.
    pub fn global_loop(&mut self, now: f64) {
        for arm in &mut self.arms {
            arm.think(now);
        }
        let Some(first) = self.arms.first() else {
            return;
        };
        if self.current != self.target && !first.slerping {
            if !self.move_immediate {
                // The shorter way round (`float` arithmetic).
                let current = self.current as f32;
                let target = self.target as f32;
                let count = self.arms.len() as f32;
                let mut shift_left = (current - target).abs() < (count * 0.5).abs();
                if current - target >= 0.0 {
                    shift_left = !shift_left;
                }
                self.move_arms(shift_left, false, now);
            } else {
                while self.current != self.target {
                    if !self.move_arms(false, true, now) {
                        self.move_immediate = false;
                        return;
                    }
                }
            }
        }
        self.move_immediate = false;
    }
}
