//! The FX objects that hang from an emitter follow it (`CPhysicsObj::UpdateChild`): each frame
//! their drawn place is the emitter's posed bone or holding point, not the place they were made at.

use std::collections::HashMap;

use ac2_math::{Frame, Mat4};
use ac2_render::object::ObjectPose;

use super::{world_matrix, ClientView, Drawn};
use crate::client_fx_objects::follow_frame;

impl ClientView {
    /// `CPhysicsObj::UpdateChild`: each FX object that hangs from an emitter stands where that
    /// emitter's posed bone or holding point puts it this frame. The emitter's place is the one it
    /// is drawn at; an emitter that is not drawn leaves its objects where they were.
    pub(super) fn follow_emitters(
        &self,
        drawn: &[Drawn<'_>],
        frames: &mut [Frame],
        placed: &mut [Option<(u64, Mat4, Option<ObjectPose>)>],
    ) {
        let index: HashMap<u64, usize> = drawn
            .iter()
            .enumerate()
            .map(|(i, drawn)| (drawn.id, i))
            .collect();
        for (i, child) in drawn.iter().enumerate() {
            let Some(follow) = child.follow else {
                continue;
            };
            let Some(&emitter) = index.get(&follow.emitter) else {
                continue;
            };
            let Some((_, emitter_scale)) = self.scene.object_rig(drawn[emitter].key) else {
                continue;
            };
            let Some(pose) = placed[emitter]
                .as_ref()
                .and_then(|placed| placed.2.as_ref())
            else {
                continue;
            };
            let Some(at) = follow_frame(
                frames[emitter],
                *emitter_scale,
                &pose.bones,
                &pose.named_bones,
                &follow.attach,
            ) else {
                continue;
            };
            frames[i] = at;
            let Some((_, scale)) = self.scene.object_rig(child.key) else {
                continue;
            };
            if let Some((key, _, pose)) = placed[i].take() {
                placed[i] = Some((key, world_matrix(*scale, at), pose));
            }
        }
    }
}
