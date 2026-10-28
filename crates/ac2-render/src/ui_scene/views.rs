//! The UI's viewport scenes, backend-neutral (`UIViewportObject`, `UIScene::Render`): one view per
//! shown viewport root, with the figures its owner placed in it, their bodies animated
//! (`UIScenePhysObjEntity::Think`), the camera and the frame each view draws. The renderers that
//! draw each view's frame live in `ac2-gpu`.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::Random;
use ac2_math::Vec3;
use ac2_physics::PhysicsBody;
use ac2_world::Ac2WorldSource;

use super::camera;
use super::data::{CameraData, ObjectData, SceneData};
use super::figure::{Figure, Placed};
use super::viewport::SceneViewport;
use crate::scene::{SceneFrame, SceneObject, SceneUpdate, WorldScene};
use crate::Camera;

mod layout;
mod pick;

pub use pick::ScenePick;

/// What the views read from the scenes' loader (a thread that reads the DATs; the client
/// implements it). Requests are asked for once; `poll` takes what finished.
pub trait SceneLoads {
    /// Asks for a scene's data unless it is loaded or under way.
    fn request_scene(&mut self, did: u32);
    /// Asks for an object's data unless it is loaded or under way.
    fn request_object(&mut self, did: u32);
    /// Takes what finished; waits for everything asked when `wait` (headless snapshots).
    fn poll(&mut self, wait: bool);
    /// The scene `did`, once loaded.
    fn scene(&self, did: u32) -> Option<&SceneData>;
    /// The object `did`, once loaded.
    fn object(&self, did: u32) -> Option<&ObjectData>;
    /// Whether `did`'s object data is in (loaded or missing).
    fn object_known(&self, did: u32) -> bool;
    /// Whether loads are under way.
    fn busy(&self) -> bool;
}

/// One viewport's scene.
struct View {
    viewport: SceneViewport,
    scene: WorldScene,
    placed: Vec<Placed>,
    /// The frame's camera (`UIScene::Render`), when the scene and its camera are in.
    camera: Option<Camera>,
    frame: Option<(SceneFrame, Vec<SceneObject>)>,
}

/// A view's frame, ready for its renderer (`prepare`).
#[derive(Debug, Clone)]
pub struct ViewFrame {
    /// The viewport root the frame belongs to.
    pub root: usize,
    pub camera: Camera,
    /// The viewport's width over its height.
    pub aspect: f32,
    pub frame: SceneFrame,
    pub objects: Vec<SceneObject>,
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
    loads: &'a dyn SceneLoads,
}

impl ViewSetup<'_> {
    /// An object's loaded data (its template chain, holding locations, box).
    pub fn object(&self, did: u32) -> Option<&ObjectData> {
        self.loads.object(did)
    }
}

/// The UI's viewport scenes' state.
pub struct UiViews {
    world: Arc<Ac2WorldSource>,
    views: Vec<View>,
    bodies: HashMap<u64, PhysicsBody>,
    random: Random,
    last: Option<f64>,
}

impl UiViews {
    /// No views yet; each viewport's models load through a `WorldScene` over `world`.
    pub fn new(world: Arc<Ac2WorldSource>) -> Self {
        Self {
            world,
            views: Vec::new(),
            bodies: HashMap::new(),
            random: Random::default(),
            last: None,
        }
    }

    /// The viewport roots shown.
    pub fn roots(&self) -> Vec<usize> {
        self.views.iter().map(|v| v.viewport.root).collect()
    }

    /// Follows `viewports` at `now` seconds: their scenes, the figures `figures` stands in each,
    /// the loads that finished (all of them when `wait`), the animation and the frame's layout,
    /// then `direct` adjusts each frame. Returns each view's loads that finished, by root, for
    /// its renderer.
    pub fn update(
        &mut self,
        loads: &mut dyn SceneLoads,
        viewports: Vec<SceneViewport>,
        now: f64,
        wait: bool,
        figures: &mut dyn FnMut(&SceneViewport, &SceneData) -> Vec<Figure>,
        direct: &mut dyn FnMut(&mut ViewSetup<'_>),
    ) -> Vec<(usize, SceneUpdate)> {
        self.views.retain(|v| {
            viewports.iter().any(|p| {
                p.root == v.viewport.root
                    && p.scene == v.viewport.scene
                    && p.element == v.viewport.element
            })
        });
        for viewport in viewports {
            loads.request_scene(viewport.scene);
            match self
                .views
                .iter_mut()
                .find(|v| v.viewport.root == viewport.root)
            {
                Some(view) => view.viewport = viewport,
                None => self.views.push(View {
                    viewport,
                    scene: WorldScene::new(Arc::clone(&self.world), 0),
                    placed: Vec::new(),
                    camera: None,
                    frame: None,
                }),
            }
        }
        loads.poll(wait);
        let mut all: Vec<Vec<Figure>> = Vec::with_capacity(self.views.len());
        for view in &self.views {
            let list = match loads.scene(view.viewport.scene) {
                Some(scene) => figures(&view.viewport, scene),
                None => Vec::new(),
            };
            for figure in list.iter().filter(|f| f.object != 0) {
                loads.request_object(figure.object);
            }
            all.push(list);
        }
        loads.poll(wait);
        self.animate(&all, now, loads);
        let mut updates = Vec::with_capacity(all.len());
        for (index, list) in all.iter().enumerate() {
            let update = self.lay_out(index, list, loads, wait);
            updates.push((self.views[index].viewport.root, update));
        }
        for index in 0..self.views.len() {
            self.frame(index, loads, direct);
        }
        updates
    }

    /// Takes each view's frame, ready to prepare (`UIScene::Render`'s draw list).
    pub fn take_frames(&mut self) -> Vec<ViewFrame> {
        let mut out = Vec::new();
        for view in &mut self.views {
            let (Some(camera), Some((frame, objects))) = (view.camera, view.frame.take()) else {
                continue;
            };
            let v = &view.viewport;
            out.push(ViewFrame {
                root: v.root,
                camera,
                aspect: v.width as f32 / v.height.max(1) as f32,
                frame,
                objects,
            });
        }
        out
    }

    /// The viewports whose camera is set (prepared), in view order.
    pub fn drawn(&self) -> Vec<SceneViewport> {
        self.views
            .iter()
            .filter(|v| v.camera.is_some())
            .map(|v| v.viewport.clone())
            .collect()
    }

    /// How many figures of the views have their models loaded.
    pub fn drawn_objects(&self) -> usize {
        self.views
            .iter()
            .flat_map(|v| &v.placed)
            .filter(|p| p.drawn.is_some())
            .count()
    }

    /// The figures of the viewport of root `root` as last laid out.
    pub fn placed(&self, root: usize) -> Option<&[Placed]> {
        self.views
            .iter()
            .find(|v| v.viewport.root == root)
            .map(|v| v.placed.as_slice())
    }

    /// A figure's body, once made.
    pub fn body_mut(&mut self, key: u64) -> Option<&mut PhysicsBody> {
        self.bodies.get_mut(&key)
    }

    /// The view's camera, lighting and objects (`UIScene::Render`), after its owner's say.
    fn frame(
        &mut self,
        index: usize,
        loads: &dyn SceneLoads,
        direct: &mut dyn FnMut(&mut ViewSetup<'_>),
    ) {
        let view = &mut self.views[index];
        view.camera = None;
        view.frame = None;
        let Some(scene) = loads.scene(view.viewport.scene) else {
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
            loads,
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
                    // UI viewport scenes draw with degrades off.
                    detail: crate::object::DrawDetail::FULL,
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
}
