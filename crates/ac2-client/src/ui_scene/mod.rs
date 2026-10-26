//! The UI's 3D scenes (`window` feature): each shown viewport's `UIScene`
//! (`UIViewportObject::Render` / `UIScene::Render`) with the figures its owner stands in it,
//! animated (`UIScenePhysObjEntity::Think`: the objects' animation stepped by the local time
//! since their last step), seen through its camera ([`camera`]) and drawn into the viewport's
//! rectangle among the UI's surfaces. The pregame screens ([`crate::pregame_scene`]) own one each.
//!
//! The scenes, objects and models load on worker threads (the scene loader and each viewport's
//! `WorldScene` loader); the frame thread takes what finished and lays out frames.
//!
//! Picks are `UIScene::StartSelection` / `FinishSelection`: the object under the pointer in the
//! viewport (`ac2_render::selection::pick_part` over every physics object of the scene) and the
//! mesh type of the part hit (`ObjectSelection::meshType`).

pub(crate) mod camera;
pub(crate) mod data;
mod figure;
mod render;

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use ac2_core::{DataId, Random};
use ac2_db::ResourceStore;
use ac2_gpu::{Gpu, SceneObject, SceneRenderer};
use ac2_math::{Frame, Mat4, Vec3};
use ac2_physics::{PhysicsBody, Position};
use ac2_render::scene::{SceneFrame, WorldScene};
use ac2_render::selection::{pick_part, PickCandidate};
use ac2_render::Camera;
use ac2_ui::SceneViewport;
use ac2_world::Ac2WorldSource;

pub use data::{CameraData, EntityData, LightData, ObjectData, SceneData};
use data::{Request, SceneLoader};
use figure::held_frame;
pub use figure::{figure_key, scene_figures, At, Drawn, Figure, Placed};
pub use render::render_with_scenes;

/// Per-frame time for taking finished model loads (`DBCache::UseTime`'s pump).
const LOAD_BUDGET: Duration = Duration::from_millis(4);

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

/// What a frame's owner may change once the figures are placed: the figures (moved with
/// [`Placed::move_to`]), the camera and a light from the camera (`Light_1`).
pub struct ViewSetup<'a> {
    pub viewport: &'a SceneViewport,
    /// The camera entity the viewport names.
    pub base: &'a CameraData,
    pub camera: Camera,
    /// How far the eye stands behind the camera's origin (an orthographic camera's).
    pub eye_offset: f32,
    pub camera_light: Option<Vec3>,
    pub figures: &'a mut [Placed],
    loader: &'a SceneLoader,
}

impl ViewSetup<'_> {
    /// An object's loaded data (its template chain, holding locations, box).
    pub(crate) fn object(&self, did: u32) -> Option<&ObjectData> {
        self.loader.object(did).map(|o| o.as_ref())
    }
}

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

/// The UI's viewport scenes.
pub struct UiScenes {
    world: Arc<Ac2WorldSource>,
    loader: SceneLoader,
    views: Vec<ViewScene>,
    bodies: HashMap<u64, PhysicsBody>,
    random: Random,
    last: Option<f64>,
}

fn model_key(visual: DataId, local: Option<&ac2_assets::Ac2VisualDesc>) -> u64 {
    let mut hasher = DefaultHasher::new();
    visual.raw().hash(&mut hasher);
    format!("{local:?}").hash(&mut hasher);
    hasher.finish()
}

impl UiScenes {
    /// Scenes over `store`'s DATs; each viewport's models load through a `WorldScene` over
    /// `world`.
    pub fn new(store: Arc<ResourceStore>, world: Arc<Ac2WorldSource>) -> Self {
        Self {
            world,
            loader: SceneLoader::new(store),
            views: Vec::new(),
            bodies: HashMap::new(),
            random: Random::default(),
            last: None,
        }
    }

    /// Follows `viewports` at `now` seconds: their scenes, the figures `figures` stands in each,
    /// the loads that finished (all of them when `wait`), the animation and the frame's layout,
    /// then `direct` adjusts each frame.
    pub fn update(
        &mut self,
        gpu: &Gpu,
        viewports: Vec<SceneViewport>,
        now: f64,
        wait: bool,
        figures: &mut dyn FnMut(&SceneViewport, &SceneData) -> Vec<Figure>,
        direct: &mut dyn FnMut(&mut ViewSetup<'_>),
    ) {
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
                    renderer: SceneRenderer::new(gpu),
                    placed: Vec::new(),
                    camera: None,
                    frame: None,
                }),
            }
        }
        self.loader.poll(wait);
        let mut all: Vec<Vec<Figure>> = Vec::with_capacity(self.views.len());
        for view in &self.views {
            let list = match self.loader.scene(view.viewport.scene) {
                Some(scene) => figures(&view.viewport, scene),
                None => Vec::new(),
            };
            for figure in list.iter().filter(|f| f.object != 0) {
                self.loader.request(Request::Object(figure.object));
            }
            all.push(list);
        }
        self.loader.poll(wait);
        self.animate(&all, now);
        for (index, list) in all.iter().enumerate() {
            self.lay_out(gpu, index, list, wait);
        }
        for index in 0..self.views.len() {
            self.frame(index, direct);
        }
    }

    /// Bodies for the figures that animate and the mode they start in, then one step
    /// (`UIScenePhysObjEntity::Think`).
    fn animate(&mut self, all: &[Vec<Figure>], now: f64) {
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

    /// Requests the figures' models, takes finished loads and places the figures.
    fn lay_out(&mut self, gpu: &Gpu, index: usize, figures: &[Figure], wait: bool) {
        let view = &mut self.views[index];
        let mut placed: Vec<Placed> = Vec::with_capacity(figures.len());
        let mut models = Vec::with_capacity(figures.len());
        for figure in figures {
            let object = self.loader.object(figure.object);
            let object_visual = object.and_then(|o| o.visual);
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
        view.renderer.apply(gpu, update);
        for (figure, model) in figures.iter().zip(models) {
            let object = self.loader.object(figure.object);
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
                    let parent_object = self.loader.object(parent_figure.object);
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
                let pose = rig.pose((!bones.is_empty()).then_some(bones.as_slice()));
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
    }

    /// The view's camera, lighting and objects (`UIScene::Render`), after its owner's say.
    fn frame(&mut self, index: usize, direct: &mut dyn FnMut(&mut ViewSetup<'_>)) {
        let view = &mut self.views[index];
        view.camera = None;
        view.frame = None;
        let Some(scene) = self.loader.scene(view.viewport.scene) else {
            return;
        };
        let Some(base) = camera::camera_entity(scene, view.viewport.camera.as_deref()) else {
            return;
        };
        let (camera, eye_offset) = camera::scene_camera(base);
        let mut setup = ViewSetup {
            viewport: &view.viewport,
            base,
            camera,
            eye_offset,
            camera_light: None,
            figures: &mut view.placed,
            loader: &self.loader,
        };
        direct(&mut setup);
        let (camera, eye_offset, camera_light) =
            (setup.camera, setup.eye_offset, setup.camera_light);
        let mut lighting = camera::scene_lighting(scene, camera_light);
        lighting.fog_range.0 += eye_offset;
        lighting.fog_range.1 += eye_offset;
        let lights = camera::point_lights(scene, camera_light.is_some());
        let objects = view
            .placed
            .iter()
            .filter_map(|p| {
                let drawn = p.drawn.as_ref()?;
                let radius = drawn
                    .bounds
                    .map_or(1.0, |(min, max)| (max - min).length() * 0.5);
                Some(SceneObject {
                    key: drawn.key,
                    world: drawn.world,
                    pose: drawn.pose.clone(),
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

    /// A figure's body, once made.
    pub fn body_mut(&mut self, key: u64) -> Option<&mut PhysicsBody> {
        self.bodies.get_mut(&key)
    }

    /// Whether `object`'s data is in (loaded or missing).
    pub fn object_known(&self, object: u32) -> bool {
        self.loader.objects.contains_key(&object)
    }

    /// The scene `did`, once loaded.
    pub(crate) fn scene(&self, did: u32) -> Option<&Arc<SceneData>> {
        self.loader.scene(did)
    }

    /// The figures of the viewport of root `root` as last laid out.
    pub fn placed(&self, root: usize) -> Option<&[Placed]> {
        self.views
            .iter()
            .find(|v| v.viewport.root == root)
            .map(|v| v.placed.as_slice())
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

    /// The prepared viewports, by index.
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

    /// How many figures of the views have their models loaded.
    pub fn drawn_objects(&self) -> usize {
        self.views
            .iter()
            .flat_map(|v| &v.placed)
            .filter(|p| p.drawn.is_some())
            .count()
    }
}
