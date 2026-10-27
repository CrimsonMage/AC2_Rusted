//! `CPhysicsObj`'s look-at entry points over [`LookAtManager`]: `LookAt` (the server's or a
//! script's target), `LookAtDir` (the head direction), and the update the animation step runs.

use crate::look_at::{LookSelf, LOOK_RAMP};

use super::PhysicsObj;

impl PhysicsObj {
    /// `CPhysicsObj::LookAt(target)`: the body looks at `target`; a target that is the body
    /// itself (`own`) clears the look, and so does an empty target. Retail then calls
    /// `LookAtManager::LookAt(target, 0.5)` or `Release(0.5)`.
    pub(crate) fn look_at(&mut self, target: u64, own: u64) {
        let target = if target == own { 0 } else { target };
        self.look.parent = own;
        if target != 0 {
            let yaw = self.anim.sliders.value(crate::look_at::LOOK_YAW_SLIDER);
            self.look.look_at(target, LOOK_RAMP, yaw);
        } else {
            self.look.release(LOOK_RAMP);
        }
    }

    /// `CPhysicsObj::LookAtDir(x, z)`: a non-player's head direction (a player's head follows
    /// its camera, so a server direction does not move it).
    pub(crate) fn look_at_dir(&mut self, x: f32, z: f32) {
        if self.anim.is_player() {
            return;
        }
        self.head_x = x;
        self.head_z = z;
        self.look.look_at_dir();
    }

    /// The id of the object the body is looking at, if any.
    pub(crate) fn look_target(&self) -> Option<u64> {
        (self.look.target != 0).then_some(self.look.target)
    }

    /// `LookAtManager::Update` for this quantum, with the target's snapshot the client gave.
    pub(crate) fn update_look_at(&mut self, quantum: f32) {
        let me = LookSelf {
            position: self.position,
            height: self.height(),
            is_player: self.anim.is_player(),
            head_x: self.head_x,
        };
        let target = if self.look.target != 0 {
            self.look_snapshot
        } else {
            None
        };
        for slider in self.look.update(quantum, &me, target) {
            self.do_slider(0, slider.slider, slider.value, slider.time);
        }
    }
}
