//! The client drawn (`window` feature): the shared world scene (`ac2_render::scene`) kept
//! resident around the player's landblock, every in-world object with a visual drawn under its
//! own appearance (`VisualDesc::GetTotalVisualDesc` of its local description) and posed by its
//! animation (`CPartArray::UpdateBones` over `ObjectFrame.pose`), seen from the client's scene
//! camera (`SceneCamera::mRenderPosition`) and lit by the region sky at the server time, with the
//! gameplay HUD over it (`hud_view.rs`). All loading runs on worker threads; this only drains
//! finished loads and lays out frames.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use ac2_core::DataId;
use ac2_gpu::{Gpu, SceneObject, SceneRenderer};
use ac2_math::{Mat4, Quat, Vec3};
use ac2_render::scene::WorldScene;
use ac2_render::Camera;
use ac2_weenie::Appearance;
use ac2_world::Ac2WorldSource;

use crate::frame::ClientFrame;
use crate::hud_view::HudView;
use crate::motion::PartPose;

/// Near and far planes of the drawn view.
const NEAR: f32 = 0.1;
const FAR: f32 = 4000.0;

/// One object to draw: its id, visual, appearance, where it stands and how it is posed.
struct Drawn<'a> {
    visual: u32,
    appearance: &'a Appearance,
    cell: u32,
    origin: [f32; 3],
    rotation: [f32; 4],
    pose: &'a [PartPose],
}

/// The view's state.
pub struct ClientView {
    scene: WorldScene,
    renderer: SceneRenderer,
    hud: HudView,
    block_size: f32,
}

/// The landblock (x, y) of a cell id.
fn block_of(cell: u32) -> (u8, u8) {
    ((cell >> 24) as u8, (cell >> 16) as u8)
}

/// Names one object's model: its visual and local description.
fn model_key(visual: u32, appearance: &Appearance) -> u64 {
    let mut hasher = DefaultHasher::new();
    visual.hash(&mut hasher);
    format!("{appearance:?}").hash(&mut hasher);
    hasher.finish()
}

fn drawn_objects(frame: &ClientFrame) -> Vec<Drawn<'_>> {
    let player = frame.player.iter().filter_map(|p| {
        Some(Drawn {
            visual: p.visual?,
            appearance: &p.appearance,
            cell: p.cell,
            origin: p.origin,
            rotation: p.rotation,
            pose: &p.pose,
        })
    });
    let others = frame
        .objects
        .iter()
        .filter(|o| o.in_world && o.holder.is_none())
        .filter_map(|o| {
            Some(Drawn {
                visual: o.visual?,
                appearance: &o.appearance,
                cell: o.cell,
                origin: o.origin,
                rotation: o.rotation,
                pose: &o.pose,
            })
        });
    player.chain(others).collect()
}

impl ClientView {
    /// A view with a near ring of `radius` landblocks on a `size` (pixels) target.
    pub fn new(
        world: Arc<Ac2WorldSource>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        radius: i32,
        size: (u32, u32),
    ) -> Self {
        Self {
            block_size: world.block_size(),
            hud: HudView::new(Arc::clone(world.resource_store()), gpu, format, size),
            scene: WorldScene::new(world, radius),
            renderer: SceneRenderer::new(gpu, format),
        }
    }

    /// The target changed size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.hud.resize(width, height);
    }

    /// A position relative to `cell`'s landblock, in the world.
    fn world_point(&self, cell: u32, origin: [f32; 3]) -> Vec3 {
        let (x, y) = block_of(cell);
        Vec3::new(
            f32::from(x) * self.block_size,
            f32::from(y) * self.block_size,
            0.0,
        ) + Vec3::from_array(origin)
    }

    /// The frame's camera, while the player is in the world.
    pub fn camera(&self, frame: &ClientFrame) -> Option<Camera> {
        let camera = frame.camera?;
        Some(Camera {
            eye: self.world_point(camera.cell, camera.origin),
            target: self.world_point(camera.cell, camera.target),
            fov_y: camera.fov.to_radians(),
            near: NEAR,
            far: FAR,
        })
    }

    /// Follows the client: the ring around the player's landblock, the region sky at the server
    /// time, models for objects not yet requested; then takes the loads that finished within
    /// `budget` and uploads them.
    pub fn update(&mut self, gpu: &Gpu, frame: &ClientFrame, budget: Duration) {
        if let Some(cell) = frame.cell_id {
            self.scene.set_center(block_of(cell));
        }
        self.scene.time_of_day = frame.server_time;
        for drawn in drawn_objects(frame) {
            let key = model_key(drawn.visual, drawn.appearance);
            let local = crate::appearance::local_visual_desc(drawn.appearance);
            self.scene
                .request_object(key, DataId::new(drawn.visual), Some(local));
        }
        let update = self.scene.poll(budget);
        self.renderer.apply(gpu, update);
        self.hud.update(gpu);
    }

    /// Waits for every outstanding load (headless snapshots).
    pub fn settle(&mut self, gpu: &Gpu, frame: &ClientFrame, aspect: f32) {
        self.hud.settle(gpu);
        let update = self.scene.wait();
        self.renderer.apply(gpu, update);
        if let Some(camera) = self.camera(frame) {
            self.scene.frame(&camera, aspect);
            let update = self.scene.wait();
            self.renderer.apply(gpu, update);
        }
    }

    /// Lays out the frame; `false` while there is no camera (before world entry).
    pub fn prepare(&mut self, gpu: &Gpu, frame: &ClientFrame, aspect: f32, time: f64) -> bool {
        let Some(camera) = self.camera(frame) else {
            return false;
        };
        let scene_frame = self.scene.frame(&camera, aspect);
        let objects = drawn_objects(frame)
            .into_iter()
            .filter_map(|drawn| {
                let key = model_key(drawn.visual, drawn.appearance);
                let (rig, scale) = self.scene.object_rig(key)?;
                let [w, x, y, z] = drawn.rotation;
                let world = Mat4::from_scale_rotation_translation(
                    *scale,
                    Quat::from_xyzw(x, y, z, w),
                    self.world_point(drawn.cell, drawn.origin),
                );
                let bones: Vec<_> = drawn.pose.iter().map(PartPose::transform).collect();
                let pose = rig.pose((!bones.is_empty()).then_some(bones.as_slice()));
                Some(SceneObject {
                    key,
                    world,
                    pose,
                    tint: [1.0; 4],
                })
            })
            .collect();
        self.renderer
            .prepare(gpu, &camera, aspect, time, scene_frame, objects);
        self.hud.prepare(&gpu.queue);
        true
    }

    /// How many of the frame's objects have their model loaded.
    pub fn loaded_objects(&self, frame: &ClientFrame) -> usize {
        drawn_objects(frame)
            .iter()
            .filter(|d| {
                self.scene
                    .object_rig(model_key(d.visual, d.appearance))
                    .is_some()
            })
            .count()
    }

    /// The frame's UI actions for the HUD ([`ac2_ui::Controls::take_ui_input`]); returns the
    /// messages they broadcast, for [`ac2_ui::Controls::ui_messages`].
    pub fn ui_input(&mut self, actions: &[(u32, f32, bool)]) -> Vec<ac2_ui::UiMessage> {
        self.hud.input(actions)
    }

    /// How many HUD surfaces are drawn.
    pub fn hud_layers(&self) -> usize {
        self.hud.layers()
    }

    /// Records the frame into a pass cleared to `SceneTool::backgroundColor` (black): the
    /// world, then the HUD.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.renderer.draw(pass);
        self.hud.draw(pass);
    }
}
