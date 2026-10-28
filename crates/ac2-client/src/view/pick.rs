//! Object picking: the object under the pointer (`gmSelectionUI::Start3DSelection` /
//! `Finish3DSelection`), by the scene's pick meshes.
use std::sync::Arc;

use ac2_render::selection::{pick_list, PickCandidate};

use super::{drawn_objects, ClientView};
use crate::frame::ClientFrame;

impl ClientView {
    /// `gmSelectionUI::Start3DSelection` / `Finish3DSelection`: the object under `cursor` on a
    /// `screen`-sized view (`ac2_render::selection::pick`); the player is not selectable in
    /// first person.
    pub fn pick(&self, frame: &ClientFrame, screen: (u32, u32), cursor: (i32, i32)) -> Option<u64> {
        self.pick_list(frame, screen, cursor).first().copied()
    }

    /// The potential list under `cursor`, nearest first (`SmartBox`'s pick list that
    /// `Finish3DSelection` reads).
    pub fn pick_list(
        &self,
        frame: &ClientFrame,
        screen: (u32, u32),
        cursor: (i32, i32),
    ) -> Vec<u64> {
        let Some(camera) = self.camera(frame) else {
            return Vec::new();
        };
        let first_person = frame.camera.is_some_and(|c| c.first_person);
        let aspect = screen.0 as f32 / screen.1.max(1) as f32;
        let drawn = drawn_objects(frame);
        let placed: Vec<_> = drawn
            .iter()
            .filter(|d| !(d.player && first_person))
            .filter_map(|d| {
                let (key, world, pose) = self.placed(d, self.drawn_frame(d))?;
                let mesh = self.scene.object_pick(key)?;
                Some((d.id, d.player, world, pose, Arc::clone(mesh)))
            })
            .collect();
        let candidates: Vec<_> = placed
            .iter()
            .map(|(id, player, world, pose, mesh)| PickCandidate {
                id: *id,
                world: *world,
                pose: pose.as_ref(),
                mesh,
                third_person_player: *player,
            })
            .collect();
        pick_list(
            camera.view_projection(aspect),
            camera.eye,
            screen,
            cursor,
            &candidates,
        )
        .into_iter()
        .map(|(id, _)| id)
        .collect()
    }
}
