//! The figures' bodies and layout (`UIScenePhysObjEntity::Think`, `UIScene::AddEntity`'s
//! holding, `UISceneEntity::SetFrame`): each figure's model requested, the loads taken, and the
//! figure placed in the scene.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use ac2_assets::Ac2VisualDesc;
use ac2_core::DataId;
use ac2_math::{Frame, Mat4, Vec3};
use ac2_physics::{PhysicsBody, Position};

use super::{SceneLoads, UiViews};
use crate::scene::SceneUpdate;
use crate::ui_scene::figure::{held_frame, At, Drawn, Figure, Placed};

/// Per-frame time for taking finished model loads (`DBCache::UseTime`'s pump).
const LOAD_BUDGET: Duration = Duration::from_millis(4);

/// A model's key: its visual and its local appearance.
fn model_key(visual: DataId, local: Option<&Ac2VisualDesc>) -> u64 {
    let mut hasher = DefaultHasher::new();
    visual.raw().hash(&mut hasher);
    format!("{local:?}").hash(&mut hasher);
    hasher.finish()
}

impl UiViews {
    /// Bodies for the figures that animate and the mode they start in, then one step
    /// (`UIScenePhysObjEntity::Think`).
    pub(super) fn animate(&mut self, all: &[Vec<Figure>], now: f64, loads: &dyn SceneLoads) {
        let keys: HashSet<u64> = all.iter().flatten().map(|f| f.key).collect();
        self.bodies.retain(|key, _| keys.contains(key));
        for figure in all.iter().flatten() {
            if self.bodies.contains_key(&figure.key) {
                continue;
            }
            let Some(body) = loads.object(figure.object).and_then(|o| o.body.clone()) else {
                continue;
            };
            let mut physics = PhysicsBody::new(Position::default(), body.shape, body.motion);
            physics.set_radius(body.radius);
            if let Some(animation) = body.animation {
                physics.set_animation(animation);
            }
            if let Some(mode) = figure.mode {
                physics.set_mode(mode);
            }
            self.bodies.insert(figure.key, physics);
        }
        let quantum = self.last.map_or(0.0, |last| (now - last).max(0.0));
        self.last = Some(now);
        if quantum > 0.0 {
            for body in self.bodies.values_mut() {
                body.update_animation(quantum as f32, now, &mut self.random);
            }
        }
    }

    /// Requests the figures' models, takes finished loads and places the figures. Returns the
    /// loads the view's renderer uploads.
    pub(super) fn lay_out(
        &mut self,
        index: usize,
        figures: &[Figure],
        loads: &dyn SceneLoads,
        wait: bool,
    ) -> SceneUpdate {
        let view = &mut self.views[index];
        let mut placed: Vec<Placed> = Vec::with_capacity(figures.len());
        let mut models = Vec::with_capacity(figures.len());
        for figure in figures {
            let object_visual = loads.object(figure.object).and_then(|o| o.visual);
            // A figure's local description hangs from its object's visual unless it names one.
            let local = figure.local.clone().map(|mut local| {
                local.parent = local.parent.or(object_visual);
                local
            });
            let visual = match &local {
                Some(local) => local.parent,
                None => object_visual,
            };
            let model = visual.map(|visual| {
                let key = model_key(visual, local.as_ref());
                view.scene.request_object(key, visual, local.clone());
                key
            });
            models.push(model);
        }
        let update = if wait {
            view.scene.wait()
        } else {
            view.scene.poll(LOAD_BUDGET)
        };
        for (figure, model) in figures.iter().zip(models) {
            let object = loads.object(figure.object);
            let body = self.bodies.get(&figure.key);
            let own_scale = object.map_or(Vec3::ONE, |o| o.scale);
            let unplaced = |frame| Placed {
                key: figure.key,
                object: figure.object,
                pick: None,
                frame,
                scale: Vec3::ONE,
                drawn: None,
            };
            let (frame, scale) = match figure.at {
                At::Scene(frame, scale) => (frame, scale * own_scale),
                At::Held { parent, location } => {
                    let Some(holder) = placed.get(parent) else {
                        placed.push(unplaced(Frame::IDENTITY));
                        continue;
                    };
                    let parent_figure = &figures[parent];
                    let parent_object = loads.object(parent_figure.object);
                    let parent_pose = self.bodies.get(&parent_figure.key).map(|b| b.posed_parts());
                    let at =
                        parent_object.and_then(|o| o.location(location, parent_pose.as_deref()));
                    match at {
                        Some(at) => (held_frame(&holder.frame, holder.scale, &at), own_scale),
                        None => {
                            let frame = holder.frame;
                            placed.push(unplaced(frame));
                            continue;
                        }
                    }
                }
            };
            let drawn = model.and_then(|key| {
                let (rig, rig_scale) = view.scene.object_rig(key)?;
                let world = Mat4::from_scale_rotation_translation(
                    scale * *rig_scale,
                    frame.orientation,
                    frame.origin,
                );
                let bones = body.map(|b| b.bone_transforms()).unwrap_or_default();
                let children = body.map(|b| b.child_bone_transforms()).unwrap_or_default();
                let pose = rig.pose_groups(
                    (!bones.is_empty()).then_some(bones.as_slice()),
                    &children,
                    0,
                );
                let bounds = view.scene.object_pick(key).map(|m| (m.min, m.max));
                Some(Drawn {
                    key,
                    world,
                    pose,
                    rig_scale: *rig_scale,
                    bounds,
                })
            });
            placed.push(Placed {
                key: figure.key,
                object: figure.object,
                pick: figure.pick,
                frame,
                scale,
                drawn,
            });
        }
        view.placed = placed;
        update
    }
}
