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
use ac2_math::{Frame, Mat4, Quat, Vec3};
use ac2_physics::PartPose;
use ac2_render::object::ObjectPose;
use ac2_render::object_fx::{follow_emitters, object_placements, world_matrix, FxSlot};
use ac2_render::scene::WorldScene;
use ac2_render::Camera;
use ac2_weenie::Appearance;
use ac2_world::Ac2WorldSource;

use crate::frame::{ClientFrame, FxFollow, HoldPoint};
use crate::hud_view::HudView;

use self::overlays::radar_view;

mod held;
mod hud_api;
mod object_fx;
mod overlays;
mod pick;
#[cfg(feature = "test-support")]
mod probe;

/// Near and far planes of the drawn view.
const NEAR: f32 = 0.1;
const FAR: f32 = 4000.0;

/// One object to draw: its id, visual, appearance, where it stands and how it is posed.
struct Drawn<'a> {
    id: u64,
    player: bool,
    visual: u32,
    /// `model_key(visual, appearance)`, computed once when the object is listed.
    key: u64,
    appearance: &'a Appearance,
    cell: u32,
    origin: [f32; 3],
    rotation: [f32; 4],
    pose: &'a [PartPose],
    /// The FX's colour and fade for the object (white for none).
    tint: [f32; 4],
    /// The emitter and attach point of an FX object that hangs from one: it is placed on the
    /// emitter's posed bone each frame.
    follow: Option<FxFollow>,
    /// The holder's id and the holding location the object hangs from: it is placed there each
    /// frame (`CPhysicsObj::UpdateChild`).
    held: Option<(u64, HoldPoint)>,
    /// The placement frame a held object's parts take (`PHYSICSOBJREP::SetPlacementFrame` from the
    /// holder's `orientation_id`); `None` for an object that is not held.
    placement: Option<u32>,
}

/// The view's state.
pub struct ClientView {
    scene: WorldScene,
    renderer: SceneRenderer,
    hud: HudView,
    block_size: f32,
    /// The reticle asked to clear the selection (out of range).
    deselect: bool,
    /// The inventory's paper doll scene (`gmPaperDollUI`).
    doll: crate::paper_doll_scene::PaperDollScene,
    /// The portal space's fader (`PortalSpace::Effect`'s `DrawScreenQuad`).
    fader: crate::view_portal::PortalFader,
    /// The tone map the world and the paper doll reach the window through.
    presenter: ac2_gpu::ScenePresenter,
    /// `WorldRender::Render`'s gate for the frame: the world is drawn (not in portal space, or
    /// the effect draws it).
    world_drawn: bool,
}

/// The landblock (x, y) of a cell id.
fn block_of(cell: u32) -> (u8, u8) {
    ((cell >> 24) as u8, (cell >> 16) as u8)
}

/// Names one object's model: its visual and local description (hashed directly, see
/// `Appearance`'s `Hash`).
fn model_key(visual: u32, appearance: &Appearance) -> u64 {
    let mut hasher = DefaultHasher::new();
    visual.hash(&mut hasher);
    appearance.hash(&mut hasher);
    hasher.finish()
}

/// The FX view of each drawn object (its id, model key, cell and follow).
fn fx_slots(drawn: &[Drawn<'_>]) -> Vec<FxSlot> {
    drawn
        .iter()
        .map(|drawn| FxSlot {
            id: drawn.id,
            key: drawn.key,
            cell: drawn.cell,
            follow: drawn.follow,
        })
        .collect()
}

fn drawn_objects(frame: &ClientFrame) -> Vec<Drawn<'_>> {
    let player = frame.player.iter().filter_map(|p| {
        let visual = p.visual?;
        Some(Drawn {
            id: p.id,
            player: true,
            visual,
            key: model_key(visual, &p.appearance),
            appearance: &p.appearance,
            cell: p.cell,
            origin: p.origin,
            rotation: p.rotation,
            pose: &p.pose,
            tint: frame.fx_looks.tints.get(&p.id).copied().unwrap_or([1.0; 4]),
            follow: None,
            held: None,
            placement: None,
        })
    });
    let others = frame
        .objects
        .iter()
        .filter(|o| match o.holder {
            // A held object is drawn at its holder's holding location, once the holder's Setup
            // has it; it is in no world cell of its own.
            Some(holder) => holder.point.is_some(),
            None => o.in_world,
        })
        .filter(|o| !frame.fx_looks.hidden.contains(&o.id))
        .filter_map(|o| {
            let visual = o.visual?;
            Some(Drawn {
                id: o.id,
                player: false,
                visual,
                key: model_key(visual, &o.appearance),
                appearance: &o.appearance,
                cell: o.cell,
                origin: o.origin,
                rotation: o.rotation,
                pose: &o.pose,
                tint: ac2_smartbox::fx_objects::drawn_tint(
                    frame.fx_looks.tints.get(&o.id).copied().unwrap_or([1.0; 4]),
                    o.anim_scale,
                ),
                follow: o.fx_follow,
                held: o.holder.and_then(|holder| Some((holder.id, holder.point?))),
                placement: o.holder.and_then(|holder| holder.orientation),
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
        let store = Arc::clone(world.resource_store());
        Self {
            block_size: world.block_size(),
            hud: HudView::new(Arc::clone(&store), gpu, format, size),
            deselect: false,
            doll: crate::paper_doll_scene::PaperDollScene::new(store, Arc::clone(&world)),
            scene: WorldScene::new(world, radius),
            renderer: SceneRenderer::new(gpu),
            fader: crate::view_portal::PortalFader::new(gpu, format),
            presenter: ac2_gpu::ScenePresenter::new(gpu, format),
            world_drawn: true,
        }
    }

    /// The scene's graphics options (`WorldScene::set_options`): the landscape draw distance and
    /// the frill distance, as the graphics preferences set them.
    pub fn set_scene_options(&mut self, options: ac2_render::prefs::SceneOptions) {
        self.scene.set_options(options);
    }

    /// What the options pages' applies changed that the window or the scene must take: the scene's
    /// options and the fullscreen state (the frame's owner applies them).
    pub fn take_options_effects(&mut self) -> Vec<ac2_ui::OptionsEffect> {
        self.hud.panels.options.take_effects()
    }

    /// The target changed size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.hud.resize(width, height);
    }

    /// A position relative to `cell`'s landblock, in the world.
    pub fn world_point(&self, cell: u32, origin: [f32; 3]) -> Vec3 {
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
        profile_scope!("view.update");
        if let Some(cell) = frame.cell_id {
            self.scene.set_center(block_of(cell));
        }
        self.scene.time_of_day = frame.server_time;
        for drawn in drawn_objects(frame) {
            let key = drawn.key;
            let local = ac2_physics::local_visual_desc(drawn.appearance);
            self.scene
                .request_object(key, DataId::new(drawn.visual), Some(local));
        }
        let update = self.scene.poll(budget);
        self.renderer.apply(gpu, update);
        self.hud.update(gpu, frame.server_time);
        self.update_doll(gpu, frame, false);
    }

    /// The paper doll's scene while its viewport shows, with the player's appearance.
    fn update_doll(&mut self, gpu: &Gpu, frame: &ClientFrame, wait: bool) {
        let viewports: Vec<_> = self.hud.doll_viewport().into_iter().collect();
        let race = self.hud.player_race;
        self.doll.update(
            gpu,
            viewports,
            frame.player.as_ref(),
            race,
            frame.server_time,
            wait,
        );
    }

    /// Objects appear once their pipelines are compiled rather than stalling the frame that first
    /// shows them (the running client; headless renders wait instead).
    pub fn set_stream_pipelines(&mut self, stream: bool) {
        self.renderer.set_stream_pipelines(stream);
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
        self.update_doll(gpu, frame, true);
        while self.doll.busy() {
            self.update_doll(gpu, frame, true);
        }
        self.update_doll(gpu, frame, true);
    }

    /// Lays out the frame; `false` while there is no camera (before world entry).
    pub fn prepare(&mut self, gpu: &Gpu, frame: &ClientFrame, aspect: f32, time: f64) -> bool {
        profile_scope!("view.prepare");
        self.world_drawn = frame.portal.world_drawn();
        self.fader.update(gpu, &frame.portal);
        let Some(camera) = self.camera(frame) else {
            return false;
        };
        self.scene.set_clock(time);
        let drawn = drawn_objects(frame);
        let mut frames: Vec<Frame> = drawn.iter().map(|drawn| self.drawn_frame(drawn)).collect();
        let mut placed: Vec<_> = drawn
            .iter()
            .zip(&frames)
            .map(|(drawn, at)| self.placed(drawn, *at))
            .collect();
        let slots = fx_slots(&drawn);
        follow_emitters(&self.scene, &slots, &mut frames, &mut placed);
        self.follow_holders(&drawn, &mut frames, &mut placed);
        let placements = object_placements(&self.scene, &slots, &frames, &placed);
        self.scene.place_objects(&placements);
        let scene_frame = self.scene.frame(&camera, aspect);
        let objects = drawn
            .iter()
            .zip(frames)
            .zip(placed)
            .filter_map(|((drawn, at), placed)| {
                let (key, world, pose) = placed?;
                Some(SceneObject {
                    key,
                    world,
                    pose,
                    tint: drawn.tint,
                    point_lights: self.scene.object_point_lights(at.origin, 0.0),
                })
            })
            .collect();
        self.renderer
            .prepare(gpu, &camera, aspect, time, scene_frame, objects);
        self.position_external_container(frame, &camera, aspect);
        self.hud.update_radar(radar_view(frame));
        self.hud.update_navigation(frame);
        self.update_reticle(frame, &camera, aspect);
        self.update_names(frame, &camera, aspect);
        self.update_damage(frame, &camera, aspect);
        self.hud.finish_frame(gpu);
        self.hud.prepare(&gpu.queue);
        self.doll.prepare(gpu, time);
        true
    }

    /// A drawn object's place in the world, from its cell and origin and its rotation.
    fn drawn_frame(&self, drawn: &Drawn<'_>) -> Frame {
        let [w, x, y, z] = drawn.rotation;
        Frame::new(
            self.world_point(drawn.cell, drawn.origin),
            Quat::from_xyzw(x, y, z, w),
        )
    }

    /// A drawn object's model key, model-to-world matrix and pose (`CPartArray::UpdateBones`),
    /// standing at `at`.
    fn placed(&self, drawn: &Drawn<'_>, at: Frame) -> Option<(u64, Mat4, Option<ObjectPose>)> {
        let key = drawn.key;
        let (rig, scale) = self.scene.object_rig(key)?;
        let world = world_matrix(*scale, at);
        let bones: Vec<_> = drawn.pose.iter().map(PartPose::transform).collect();
        let placement = drawn.placement.unwrap_or(0);
        let pose = rig.pose_placed((!bones.is_empty()).then_some(bones.as_slice()), placement);
        Some((key, world, pose))
    }

    /// Whether the reticle cleared the selection since the last call (send
    /// `Command::Select(None)`).
    pub fn take_deselect(&mut self) -> bool {
        std::mem::take(&mut self.deselect)
    }

    /// Object draw calls the last `prepare` laid out for the world (bench statistics).
    pub fn object_draw_count(&self) -> usize {
        self.renderer.object_draw_count()
    }

    /// World instances the last `prepare` culled as outside the view (bench statistics).
    pub fn culled_count(&self) -> usize {
        self.renderer.culled_count()
    }

    /// How many of the frame's objects have their model loaded.
    pub fn loaded_objects(&self, frame: &ClientFrame) -> usize {
        drawn_objects(frame)
            .iter()
            .filter(|d| self.scene.object_rig(d.key).is_some())
            .count()
    }

    /// Draws the frame into `color` (the window) cleared to black: the world when
    /// `prepare` laid it out, then the HUD with the paper doll's scene in its viewport, after
    /// its surface and over a cleared depth ([`crate::ui_scene::render_with_scenes`]).
    pub fn render(
        &self,
        gpu: &Gpu,
        color: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        world: bool,
    ) {
        profile_scope!("view.render");
        let (overlay, roots) = self.hud.layers_for_scenes();
        let draw_world: &dyn Fn(&mut wgpu::RenderPass<'_>) = &|pass| self.renderer.draw(pass);
        crate::ui_scene::render_with_scenes(
            gpu,
            &self.presenter,
            color,
            depth,
            &self.hud.scale,
            [0.0; 3],
            overlay,
            roots,
            &[self.doll.scenes()],
            (world && self.world_drawn).then_some(draw_world),
            |pass| self.fader.draw(pass),
        );
    }
}
