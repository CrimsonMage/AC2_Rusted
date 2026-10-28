//! `UIScene::StartSelection` / `FinishSelection`: the figure under a pointer in a viewport.

use super::UiViews;
use crate::selection::{pick_part, PickCandidate};

/// A pick's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenePick {
    /// The viewport's root and the figure hit (its index in the frame's figures).
    pub root: usize,
    pub figure: usize,
    pub key: u64,
    /// What the figure answers (`Figure::pick`).
    pub pick: Option<usize>,
    /// `ObjectSelection::meshType` of the part hit.
    pub mesh_type: u32,
}

impl UiViews {
    /// `UIScene::StartSelection` / `FinishSelection` at screen `point` in the viewport of root
    /// `root` (any viewport under the point when `None`): the nearest figure under it.
    pub fn pick(&self, point: (i32, i32), root: Option<usize>) -> Option<ScenePick> {
        for view in &self.views {
            let v = &view.viewport;
            if root.is_some_and(|r| r != v.root) {
                continue;
            }
            let inside = point.0 >= v.x
                && point.1 >= v.y
                && point.0 < v.x + v.width as i32
                && point.1 < v.y + v.height as i32;
            let Some(camera) = view.camera.as_ref().filter(|_| inside) else {
                continue;
            };
            let meshes: Vec<_> = view
                .placed
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    let drawn = p.drawn.as_ref()?;
                    let mesh = view.scene.object_pick(drawn.key)?;
                    Some((i, drawn.world, drawn.pose.as_ref(), mesh))
                })
                .collect();
            let candidates: Vec<_> = meshes
                .iter()
                .map(|(i, world, pose, mesh)| PickCandidate {
                    id: *i as u64,
                    world: *world,
                    pose: *pose,
                    mesh,
                    third_person_player: false,
                })
                .collect();
            let aspect = v.width as f32 / v.height.max(1) as f32;
            let hit = pick_part(
                camera.view_projection(aspect),
                camera.eye,
                (v.width, v.height),
                (point.0 - v.x, point.1 - v.y),
                &candidates,
            );
            return hit.and_then(|(i, mesh_type)| {
                let placed = view.placed.get(i as usize)?;
                Some(ScenePick {
                    root: v.root,
                    figure: i as usize,
                    key: placed.key,
                    pick: placed.pick,
                    mesh_type,
                })
            });
        }
        None
    }
}
