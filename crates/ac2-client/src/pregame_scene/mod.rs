//! The pregame screens' 3D scenes (`window` feature): each shown viewport's `UIScene`
//! (`UIViewportObject::Render` / `UIScene::Render`) with what its screen stands in it
//! ([`content`]), animated (`UIScenePhysObjEntity::Think`: the objects' animation stepped by the
//! local time since their last step), seen through its camera ([`camera`]) and drawn into the
//! viewport's rectangle among the UI's surfaces.
//!
//! The scenes, objects and models load on worker threads (the scene loader and each viewport's
//! `WorldScene` loader); the frame thread takes what finished and lays out frames.
//!
//! Picks are `UIScene::StartSelection` / `FinishSelection`: the object under the pointer in the
//! viewport (`ac2_render::selection::pick` over every physics object of the scene), answered
//! with what that object stands for (a character slot, an avatar), if anything.

mod camera;
mod content;
mod data;

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use ac2_anim::{Ac2BehaviorParams, Ac2Transform};
use ac2_core::{DataId, Random};
use ac2_db::ResourceStore;
use ac2_gpu::{Gpu, SceneObject, SceneRenderer};
use ac2_math::{Frame, Mat4, Vec3};
use ac2_physics::{PhysicsBody, Position};
use ac2_render::object::ObjectPose;
use ac2_render::scene::{SceneFrame, WorldScene};
use ac2_render::selection::{pick, PickCandidate};
use ac2_render::Camera;
use ac2_ui::pregame::{PregameUi, SceneViewport};
use ac2_world::Ac2WorldSource;

use content::{At, Figure, MODE_AVATAR_SELECTION};
use data::{ObjectData, Request, SceneLoader};

/// `gmAvatarSelectionUI`'s selected animation (`BehaviorParams::mBehaviorID`).
const BEHAVIOR_SELECTED: u32 = 0x4100_0005;
/// The mode an avatar under the pointer goes to (`MouseOnObject`).
const MODE_PRESENTED: u32 = 0x4100_0007;
/// Per-frame time for taking finished model loads (`DBCache::UseTime`'s pump).
const LOAD_BUDGET: Duration = Duration::from_millis(4);

/// One figure laid out this frame.
struct Placed {
    pick: Option<usize>,
    /// The figure's frame (no scale) and scale, for what it holds.
    frame: Frame,
    scale: Vec3,
    /// Its model key and model-to-world matrix and pose, once its model loaded.
    drawn: Option<(u64, Mat4, Option<ObjectPose>)>,
}

/// One viewport's scene.
struct ViewScene {
    viewport: SceneViewport,
    scene: WorldScene,
    renderer: SceneRenderer,
    placed: Vec<Placed>,
    /// The frame's camera, lighting and objects (`prepare`).
    camera: Option<Camera>,
    frame: Option<(SceneFrame, Vec<SceneObject>)>,
}

/// A figure's body (`ICPhysicsObj`'s animation).
struct Animated {
    body: PhysicsBody,
}

/// The pregame viewports' scenes.
pub struct PregameScenes {
    world: Arc<Ac2WorldSource>,
    format: wgpu::TextureFormat,
    loader: SceneLoader,
    views: Vec<ViewScene>,
    bodies: HashMap<u64, Animated>,
    random: Random,
    last: Option<f64>,
    /// The avatar whose selected animation runs (`_waitForAnim`), by figure key.
    selected: Option<u64>,
    /// The avatar under the pointer (`_objectUnderMouse`), by figure key.
    hovered: Option<u64>,
}

fn model_key(visual: DataId, local: Option<&ac2_assets::Ac2VisualDesc>) -> u64 {
    let mut hasher = DefaultHasher::new();
    visual.raw().hash(&mut hasher);
    format!("{local:?}").hash(&mut hasher);
    hasher.finish()
}

/// `CPhysicsObj::add_child`: the child's frame on its parent's location (the parent's scale
/// moves the location, not the child's size).
fn held_frame(parent: &Frame, scale: Vec3, location: &Ac2Transform) -> Frame {
    let offset = parent.orientation * (location.offset * scale);
    Frame::new(
        parent.origin + offset,
        (parent.orientation * location.rotation).normalize(),
    )
}

impl PregameScenes {
    /// Scenes over `store`'s DATs; each viewport's models load through a `WorldScene` over
    /// `world`.
    pub fn new(
        store: Arc<ResourceStore>,
        world: Arc<Ac2WorldSource>,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            world,
            format,
            loader: SceneLoader::new(store),
            views: Vec::new(),
            bodies: HashMap::new(),
            random: Random::default(),
            last: None,
            selected: None,
            hovered: None,
        }
    }

    /// Follows the UI at `now` seconds: the shown viewports, what stands in their scenes, the
    /// loads that finished (all of them when `wait`), the animation and the frame's layout.
    pub fn update(&mut self, gpu: &Gpu, ui: &PregameUi, now: f64, wait: bool) {
        let viewports = ui.viewports();
        self.views.retain(|v| {
            viewports.iter().any(|p| {
                p.root == v.viewport.root
                    && p.scene == v.viewport.scene
                    && p.element == v.viewport.element
            })
        });
        for viewport in viewports {
            self.loader.request(Request::Scene(viewport.scene));
            match self
                .views
                .iter_mut()
                .find(|v| v.viewport.root == viewport.root)
            {
                Some(view) => view.viewport = viewport,
                None => self.views.push(ViewScene {
                    viewport,
                    scene: WorldScene::new(Arc::clone(&self.world), 0),
                    renderer: SceneRenderer::new(gpu, self.format),
                    placed: Vec::new(),
                    camera: None,
                    frame: None,
                }),
            }
        }
        self.loader.poll(wait);
        let mut all: Vec<Vec<Figure>> = Vec::with_capacity(self.views.len());
        for view in &self.views {
            let figures = match self.loader.scene(view.viewport.scene) {
                Some(scene) => content::figures(ui, &view.viewport, scene),
                None => Vec::new(),
            };
            for figure in figures.iter().filter(|f| f.object != 0) {
                self.loader.request(Request::Object(figure.object));
            }
            all.push(figures);
        }
        self.loader.poll(wait);
        self.animate(ui, &all, now);
        for (index, figures) in all.iter().enumerate() {
            self.lay_out(gpu, index, figures, wait);
        }
        for index in 0..self.views.len() {
            self.frame(ui, index);
        }
    }

    /// Bodies for the figures that animate, the mode they go to, then one step
    /// (`UIScenePhysObjEntity::Think`).
    fn animate(&mut self, ui: &PregameUi, all: &[Vec<Figure>], now: f64) {
        let keys: HashSet<u64> = all.iter().flatten().map(|f| f.key).collect();
        self.bodies.retain(|key, _| keys.contains(key));
        for figure in all.iter().flatten() {
            if self.bodies.contains_key(&figure.key) {
                continue;
            }
            let Some(body) = self
                .loader
                .object(figure.object)
                .and_then(|o| o.body.clone())
            else {
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
            self.bodies.insert(figure.key, Animated { body: physics });
        }
        self.hover(ui, all);
        let quantum = self.last.map_or(0.0, |last| (now - last).max(0.0));
        self.last = Some(now);
        if quantum > 0.0 {
            for animated in self.bodies.values_mut() {
                animated
                    .body
                    .update_animation(quantum as f32, now, &mut self.random);
            }
        }
    }

    /// `gmAvatarSelectionUI::MouseOffObject` / `MouseOnObject` when the avatar under the
    /// pointer changes: the one left goes back to the line-up mode, the one under the pointer
    /// to the presented mode 0x41000007 (where its selected animation plays), each unless
    /// already there.
    fn hover(&mut self, ui: &PregameUi, all: &[Vec<Figure>]) {
        let under = ui.avatar_selection().and_then(|s| s.under_mouse());
        let key = under.and_then(|i| {
            all.iter()
                .flatten()
                .find(|f| f.pick == Some(i) && f.mode == Some(MODE_AVATAR_SELECTION))
                .map(|f| f.key)
        });
        if key == self.hovered {
            return;
        }
        let modes = [(self.hovered, MODE_AVATAR_SELECTION), (key, MODE_PRESENTED)];
        for (key, mode) in modes {
            if let Some(animated) = key.and_then(|k| self.bodies.get_mut(&k)) {
                if animated.body.mode() != mode {
                    animated.body.do_mode(0, mode);
                }
            }
        }
        self.hovered = key;
    }

    /// Requests the figures' models, takes finished loads and places the figures.
    fn lay_out(&mut self, gpu: &Gpu, index: usize, figures: &[Figure], wait: bool) {
        let view = &mut self.views[index];
        let mut placed: Vec<Placed> = Vec::with_capacity(figures.len());
        let mut models = Vec::with_capacity(figures.len());
        for figure in figures {
            let object = self.loader.object(figure.object);
            let visual = match (&figure.local, object.and_then(|o| o.visual)) {
                (Some(local), _) => local.parent,
                (None, visual) => visual,
            };
            let model = visual.map(|visual| {
                let key = model_key(visual, figure.local.as_ref());
                view.scene.request_object(key, visual, figure.local.clone());
                key
            });
            models.push(model);
        }
        let update = if wait {
            view.scene.wait()
        } else {
            view.scene.poll(LOAD_BUDGET)
        };
        view.renderer.apply(gpu, update);
        for (figure, model) in figures.iter().zip(models) {
            let object: Option<&Arc<ObjectData>> = self.loader.object(figure.object);
            let body = self.bodies.get(&figure.key).map(|a| &a.body);
            let own_scale = object.map_or(Vec3::ONE, |o| o.scale);
            let (frame, scale) = match figure.at {
                At::Scene(frame, scale) => (frame, scale * own_scale),
                At::Held { parent, location } => {
                    let Some(holder) = placed.get(parent) else {
                        placed.push(Placed {
                            pick: None,
                            frame: Frame::IDENTITY,
                            scale: Vec3::ONE,
                            drawn: None,
                        });
                        continue;
                    };
                    let parent_figure = &figures[parent];
                    let parent_object = self.loader.object(parent_figure.object);
                    let parent_pose = self
                        .bodies
                        .get(&parent_figure.key)
                        .map(|a| a.body.posed_parts());
                    let at =
                        parent_object.and_then(|o| o.location(location, parent_pose.as_deref()));
                    match at {
                        Some(at) => (held_frame(&holder.frame, holder.scale, &at), own_scale),
                        None => {
                            placed.push(Placed {
                                pick: None,
                                frame: holder.frame,
                                scale: Vec3::ONE,
                                drawn: None,
                            });
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
                let pose = rig.pose((!bones.is_empty()).then_some(bones.as_slice()));
                Some((key, world, pose))
            });
            placed.push(Placed {
                pick: figure.pick,
                frame,
                scale,
                drawn,
            });
        }
        view.placed = placed;
    }

    /// The view's camera, lighting and objects (`UIScene::Render`).
    fn frame(&mut self, ui: &PregameUi, index: usize) {
        let view = &mut self.views[index];
        view.camera = None;
        view.frame = None;
        let Some(scene) = self.loader.scene(view.viewport.scene) else {
            return;
        };
        let Some(base) = camera::camera_entity(scene, view.viewport.camera.as_deref()) else {
            return;
        };
        let mut camera_light = None;
        let mut camera = camera::scene_camera(base);
        let modification = ui
            .avatar_modification()
            .filter(|m| m.viewport() == Some(view.viewport.root));
        if let (Some(modification), Some(race)) =
            (modification, ui.data().current().map(|c| c.race))
        {
            let object = self.loader.object(modification.phys_obj());
            let avatar = view.placed.last().and_then(|p| p.drawn.as_ref());
            if let (Some(bounds), Some((key, ..))) = (object.and_then(|o| o.bounds), avatar) {
                let scale_z = view.scene.object_rig(*key).map_or(1.0, |(_, s)| s.z);
                let (c, light) =
                    camera::modification_camera(base, race, bounds, scale_z, modification.camera());
                camera = Some(c);
                camera_light = Some(light);
            }
        }
        let Some(camera) = camera else {
            return;
        };
        let lighting = camera::scene_lighting(scene, camera_light);
        let lights = camera::point_lights(scene, camera_light.is_some());
        let objects = view
            .placed
            .iter()
            .filter_map(|p| {
                let (key, world, pose) = p.drawn.as_ref()?;
                let radius = view
                    .scene
                    .object_pick(*key)
                    .map_or(1.0, |m| (m.max - m.min).length() * 0.5);
                Some(SceneObject {
                    key: *key,
                    world: *world,
                    pose: pose.clone(),
                    tint: [1.0; 4],
                    point_lights: camera::object_lights(&lights, p.frame.origin, radius),
                })
            })
            .collect();
        let frame = SceneFrame {
            lighting,
            ..SceneFrame::default()
        };
        view.camera = Some(camera);
        view.frame = Some((frame, objects));
    }

    /// `UIScene::StartSelection` / `FinishSelection` at screen `point`: what the nearest object
    /// under it in a viewport stands for.
    pub fn pick(&self, point: (i32, i32)) -> Option<usize> {
        for view in &self.views {
            let v = &view.viewport;
            let inside = point.0 >= v.x
                && point.1 >= v.y
                && point.0 < v.x + v.width as i32
                && point.1 < v.y + v.height as i32;
            let Some(camera) = view.camera.as_ref().filter(|_| inside) else {
                continue;
            };
            if !view.placed.iter().any(|p| p.pick.is_some()) {
                continue;
            }
            let meshes: Vec<_> = view
                .placed
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    let (key, world, pose) = p.drawn.as_ref()?;
                    Some((i, *world, pose.as_ref(), view.scene.object_pick(*key)?))
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
            let hit = pick(
                camera.view_projection(aspect),
                camera.eye,
                (v.width, v.height),
                (point.0 - v.x, point.1 - v.y),
                &candidates,
            );
            return hit.and_then(|i| view.placed.get(i as usize)?.pick);
        }
        None
    }

    /// Avatar selection's wait for the chosen avatar's selected animation
    /// (`gmAvatarSelectionUI::ListenToMessage`: `DoBehavior(0x41000005)`, then
    /// `IsDoingBehavior` each loop): whether it ended.
    pub fn animation_done(&mut self, ui: &PregameUi) -> bool {
        let Some(avatar) = ui.selected_animation() else {
            self.selected = None;
            return false;
        };
        let figure = self.views.iter().find_map(|view| {
            let scene = self.loader.scene(view.viewport.scene)?;
            content::figures(ui, &view.viewport, scene)
                .into_iter()
                .find(|f| f.pick == Some(avatar) && f.mode == Some(MODE_AVATAR_SELECTION))
        });
        let Some(figure) = figure else {
            return true;
        };
        match self.selected {
            Some(key) if key == figure.key => self
                .bodies
                .get(&figure.key)
                .is_none_or(|a| !a.body.is_doing_behavior(BEHAVIOR_SELECTED)),
            _ => {
                if !self.loader.objects.contains_key(&figure.object) {
                    // Still loading: the avatar is not made yet.
                    return false;
                }
                self.selected = Some(figure.key);
                match self.bodies.get_mut(&figure.key) {
                    Some(animated) => {
                        animated
                            .body
                            .do_behavior(&Ac2BehaviorParams::new(BEHAVIOR_SELECTED));
                        false
                    }
                    None => true,
                }
            }
        }
    }

    /// Whether loads are under way (headless snapshots wait them out).
    pub fn busy(&self) -> bool {
        self.loader.busy()
    }

    /// Writes each view's frame (`time` is client time for animated materials).
    pub fn prepare(&mut self, gpu: &Gpu, time: f64) {
        for view in &mut self.views {
            let (Some(camera), Some((frame, objects))) = (view.camera.as_ref(), view.frame.take())
            else {
                continue;
            };
            let v = &view.viewport;
            let aspect = v.width as f32 / v.height.max(1) as f32;
            view.renderer
                .prepare(gpu, camera, aspect, time, frame, objects);
        }
    }

    /// The prepared viewports' roots and screen rectangles, in no particular order.
    pub fn drawn(&self) -> Vec<(usize, SceneViewport)> {
        self.views
            .iter()
            .enumerate()
            .filter(|(_, v)| v.camera.is_some())
            .map(|(i, v)| (i, v.viewport.clone()))
            .collect()
    }

    /// Records view `index`'s scene (its viewport is the caller's).
    pub fn draw(&self, index: usize, pass: &mut wgpu::RenderPass<'_>) {
        if let Some(view) = self.views.get(index) {
            view.renderer.draw(pass);
        }
    }

    /// How many objects of the views have their models loaded.
    pub fn drawn_objects(&self) -> usize {
        self.views
            .iter()
            .flat_map(|v| &v.placed)
            .filter(|p| p.drawn.is_some())
            .count()
    }
}
