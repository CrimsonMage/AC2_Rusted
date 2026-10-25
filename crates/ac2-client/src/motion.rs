//! The server's motion events for a body (`CPhysicsObj::RecvEvt_DoBehavior`,
//! `RecvEvt_StopBehavior`) and the animated part pose as plain data.

use ac2_anim::Ac2BehaviorParams;
use ac2_net::protocol::{BehaviorParams, ObjectAction};
use ac2_physics::PhysicsBody;

/// One Setup bone as the animation left it (`CPartArray::m_boneTransforms`): offset, rotation
/// (w, x, y, z) and scale relative to its parent bone (AC axes). The renderer builds the bone and
/// deform matrices from these (`ac2_render::object::ObjectModel::pose`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartPose {
    pub offset: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl PartPose {
    /// As the `TransformBase` the renderer poses a model with (`ObjectModel::pose`).
    pub fn transform(&self) -> ac2_anim::Ac2Transform {
        let [w, x, y, z] = self.rotation;
        ac2_anim::Ac2Transform {
            offset: self.offset.into(),
            scale: self.scale.into(),
            rotation: ac2_math::Quat::from_xyzw(x, y, z, w),
        }
    }
}

/// The body's bone transforms (`PhysicsBody::bone_transforms`).
pub(crate) fn pose(body: &PhysicsBody) -> Vec<PartPose> {
    body.bone_transforms()
        .iter()
        .map(|t| PartPose {
            offset: [t.offset.x, t.offset.y, t.offset.z],
            rotation: [t.rotation.w, t.rotation.x, t.rotation.y, t.rotation.z],
            scale: [t.scale.x, t.scale.y, t.scale.z],
        })
        .collect()
}

/// `BehaviorParams::StreamPack` over `BehaviorParams::SetDefaults`: the fields the pack bitfield
/// names, the rest at their defaults.
fn behavior(params: &BehaviorParams) -> Ac2BehaviorParams {
    let has = |bit: u32| params.bitfield & bit != 0;
    let mut out = Ac2BehaviorParams::new(if has(0x1) { params.behavior } else { 0 });
    if has(0x4) {
        out.time_scale = params.time_scale;
    }
    if has(0x10) {
        out.hold_cycles = params.hold_cycles;
    }
    if has(0x10_0000) {
        out.fx = params.fx;
    }
    let flag = |bit: u32, value: u32, default: bool| if has(bit) { value != 0 } else { default };
    out.early_callback = flag(0x8, params.early_callback, out.early_callback);
    out.move_to_cancels = flag(0x20, params.moveto_cancels, out.move_to_cancels);
    out.lock_actions = flag(0x800, params.lock_actions, out.lock_actions);
    out.send_event = flag(0x200, params.send_event, out.send_event);
    out.leave_idle_alone = flag(0x2_0000, params.leave_idle_alone, out.leave_idle_alone);
    // `RecvEvt_DoBehavior` runs a copy whose context is 1.
    out.context = 1;
    out
}

/// The motion events a body plays: `RecvEvt_DoBehavior` (`DoBehavior` with context 1; a weenie
/// emote is not run first) and `RecvEvt_StopBehavior`.
pub(crate) fn apply(body: &mut PhysicsBody, action: &ObjectAction) {
    match action {
        ObjectAction::DoBehavior(params) => {
            body.do_behavior(&behavior(params));
        }
        ObjectAction::StopBehavior(id) => body.stop_behavior(*id),
        _ => {}
    }
}
