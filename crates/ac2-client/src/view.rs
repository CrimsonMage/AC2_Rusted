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
use ac2_render::object::ObjectPose;
use ac2_render::scene::WorldScene;
use ac2_render::selection::{pick, PickCandidate};
use ac2_render::Camera;
use ac2_weenie::Appearance;
use ac2_world::Ac2WorldSource;

use crate::frame::ClientFrame;
use crate::hud_view::HudView;
use crate::motion::PartPose;

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
    /// The reticle asked to clear the selection (out of range).
    deselect: bool,
    /// The inventory's paper doll scene (`gmPaperDollUI`).
    doll: crate::paper_doll_scene::PaperDollScene,
    /// The portal space's fader (`PortalSpace::Effect`'s `DrawScreenQuad`).
    fader: crate::view_portal::PortalFader,
    /// `WorldRender::Render`'s gate for the frame: the world is drawn (not in portal space, or
    /// the effect draws it).
    world_drawn: bool,
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
            id: p.id,
            player: true,
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
                id: o.id,
                player: false,
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
        let store = Arc::clone(world.resource_store());
        Self {
            block_size: world.block_size(),
            hud: HudView::new(Arc::clone(&store), gpu, format, size),
            deselect: false,
            doll: crate::paper_doll_scene::PaperDollScene::new(store, Arc::clone(&world), format),
            scene: WorldScene::new(world, radius),
            renderer: SceneRenderer::new(gpu, format),
            fader: crate::view_portal::PortalFader::new(gpu, format),
            world_drawn: true,
        }
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
        self.world_drawn = frame.portal.world_drawn();
        self.fader.update(gpu, &frame.portal);
        let Some(camera) = self.camera(frame) else {
            return false;
        };
        let scene_frame = self.scene.frame(&camera, aspect);
        let objects = drawn_objects(frame)
            .iter()
            .filter_map(|drawn| {
                let (key, world, pose) = self.placed(drawn)?;
                Some(SceneObject {
                    key,
                    world,
                    pose,
                    tint: [1.0; 4],
                    point_lights: Vec::new(),
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

    /// A drawn object's model key, model-to-world matrix and pose (`CPartArray::UpdateBones`).
    fn placed(&self, drawn: &Drawn<'_>) -> Option<(u64, Mat4, Option<ObjectPose>)> {
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
        Some((key, world, pose))
    }

    /// `gmSelectionUI::Start3DSelection` / `Finish3DSelection`: the object under `cursor` on a
    /// `screen`-sized view (`ac2_render::selection::pick`); the player is not selectable in
    /// first person.
    pub fn pick(&self, frame: &ClientFrame, screen: (u32, u32), cursor: (i32, i32)) -> Option<u64> {
        let camera = self.camera(frame)?;
        let first_person = frame.camera.is_some_and(|c| c.first_person);
        let aspect = screen.0 as f32 / screen.1.max(1) as f32;
        let drawn = drawn_objects(frame);
        let placed: Vec<_> = drawn
            .iter()
            .filter(|d| !(d.player && first_person))
            .filter_map(|d| {
                let (key, world, pose) = self.placed(d)?;
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
        pick(
            camera.view_projection(aspect),
            camera.eye,
            screen,
            cursor,
            &candidates,
        )
    }

    /// `gm3DTextUI::Update`: the reticle around the selected object, from its model box, its
    /// placement and the camera.
    fn update_reticle(&mut self, frame: &ClientFrame, camera: &Camera, aspect: f32) {
        let selection = frame.selection.as_ref();
        let drawn = drawn_objects(frame);
        let placed = selection
            .and_then(|s| drawn.iter().find(|d| d.id == s.id))
            .and_then(|d| {
                let (key, world, _) = self.placed(d)?;
                let mesh = self.scene.object_pick(key)?;
                Some((
                    world,
                    mesh.min,
                    mesh.max,
                    self.world_point(d.cell, d.origin),
                ))
            });
        let player = frame
            .player
            .as_ref()
            .map(|p| self.world_point(p.cell, p.origin));
        let view_projection = camera.view_projection(aspect);
        let project = |world: Mat4| {
            move |p: [f32; 3]| {
                let clip = view_projection * world * Vec3::from_array(p).extend(1.0);
                let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
                ([clip.x / w, clip.y / w], clip.w > 0.0)
            }
        };
        let projector = placed.map(|(world, ..)| project(world));
        let target = match (selection, placed, &projector) {
            (Some(s), Some((_, min, max, at)), Some(projector)) => Some(ac2_ui::ReticleTarget {
                id: s.id,
                bounds: (min.to_array(), max.to_array()),
                project: projector,
                planar_distance: player.map(|p| (at - p).truncate().length()),
            }),
            _ => None,
        };
        let outcome = self.hud.reticle(
            selection.map(|s| (s.id, s.name.as_deref())),
            selection.and_then(|s| s.info.as_ref()),
            target,
        );
        self.deselect |= outcome.deselect;
    }

    /// `gmExternalContainerUI::PositionWindow` for a container that just opened: its model's
    /// box centre projected (`SceneTool::TransformVector`); centred when it is not drawn or is
    /// behind the camera.
    fn position_external_container(&mut self, frame: &ClientFrame, camera: &Camera, aspect: f32) {
        let Some(id) = self.hud.external_container_position() else {
            return;
        };
        let view_projection = camera.view_projection(aspect);
        let clip = drawn_objects(frame)
            .iter()
            .find(|d| d.id == id)
            .and_then(|d| {
                let (key, world, _) = self.placed(d)?;
                let mesh = self.scene.object_pick(key)?;
                let centre = (mesh.min + mesh.max) * 0.5;
                let clip = view_projection * world * centre.extend(1.0);
                (clip.w > 0.0).then(|| [clip.x / clip.w, clip.y / clip.w])
            });
        self.hud.position_external_container(clip);
    }

    /// `gm3DTextUI::UpdateNames`: each named object's model box and projection, and its ground
    /// distance from the player (its radar offset).
    fn update_names(&mut self, frame: &ClientFrame, camera: &Camera, aspect: f32) {
        let named = self.hud.named();
        if named.is_empty() {
            return;
        }
        let view_projection = camera.view_projection(aspect);
        let drawn = drawn_objects(frame);
        let mut objects = Vec::new();
        for id in named {
            let Some(blip) = frame.radar.blips.iter().find(|b| b.id == id) else {
                continue;
            };
            let Some(d) = drawn.iter().find(|d| d.id == id) else {
                continue;
            };
            let Some((key, world, _)) = self.placed(d) else {
                continue;
            };
            let Some(mesh) = self.scene.object_pick(key) else {
                continue;
            };
            let (x, y) = (f64::from(blip.offset[0]), f64::from(blip.offset[1]));
            let distance = (x * x + y * y) as f32;
            objects.push((
                id,
                world,
                mesh.min.to_array(),
                mesh.max.to_array(),
                distance,
            ));
        }
        let projectors: Vec<_> = objects
            .iter()
            .map(|&(_, world, ..)| {
                move |p: [f32; 3]| {
                    let clip = view_projection * world * Vec3::from_array(p).extend(1.0);
                    let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
                    ([clip.x / w, clip.y / w], clip.w > 0.0)
                }
            })
            .collect();
        let places: Vec<_> = objects
            .iter()
            .zip(&projectors)
            .map(
                |(&(id, _, min, max, distance), project)| ac2_ui::NamePlace {
                    id,
                    bounds: (min, max),
                    project,
                    xy_distance_sq: distance,
                },
            )
            .collect();
        let selection = frame.selection.as_ref().map_or(0, |s| s.id);
        self.hud.update_names(&places, selection, frame.radar.range);
    }

    /// `gm3DTextUI::UpdateDamage`: each floating damage text's object box and projection.
    fn update_damage(&mut self, frame: &ClientFrame, camera: &Camera, aspect: f32) {
        let objects = self.hud.damage_objects();
        if objects.is_empty() {
            return;
        }
        let view_projection = camera.view_projection(aspect);
        let drawn = drawn_objects(frame);
        let mut boxes = Vec::new();
        for id in objects {
            let Some(d) = drawn.iter().find(|d| d.id == id) else {
                continue;
            };
            let Some((key, world, _)) = self.placed(d) else {
                continue;
            };
            let Some(mesh) = self.scene.object_pick(key) else {
                continue;
            };
            boxes.push((id, world, mesh.min.to_array(), mesh.max.to_array()));
        }
        let projectors: Vec<_> = boxes
            .iter()
            .map(|&(_, world, ..)| {
                move |p: [f32; 3]| {
                    let clip = view_projection * world * Vec3::from_array(p).extend(1.0);
                    let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
                    ([clip.x / w, clip.y / w], clip.w > 0.0)
                }
            })
            .collect();
        let places: Vec<_> = boxes
            .iter()
            .zip(&projectors)
            .map(|(&(id, _, min, max), project)| ac2_ui::DamagePlace {
                id,
                bounds: (min, max),
                project,
            })
            .collect();
        let selection = frame.selection.as_ref().map_or(0, |s| s.id);
        self.hud.update_damage(&places, selection);
    }

    /// Whether the reticle cleared the selection since the last call (send
    /// `Command::Select(None)`).
    pub fn take_deselect(&mut self) -> bool {
        std::mem::take(&mut self.deselect)
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

    /// The sounds the HUD's media played since the last call (`MD_Data_Sound`), for
    /// [`crate::play_ui_sounds`].
    pub fn take_ui_sounds(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.hud.sounds)
    }

    /// The frame's UI actions for the HUD ([`ac2_ui::Controls::take_ui_input`]); returns the
    /// messages they broadcast, for [`ac2_ui::Controls::ui_messages`].
    pub fn ui_input(
        &mut self,
        actions: &[(u32, f32, bool)],
        modifiers: ac2_ui::Modifiers,
    ) -> Vec<ac2_ui::UiMessage> {
        self.hud.input(actions, modifiers)
    }

    /// The typing for the HUD's active text element ([`ac2_ui::Controls::take_text_input`]);
    /// returns the messages it broadcast.
    pub fn text_input(
        &mut self,
        actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        modifiers: ac2_ui::Modifiers,
    ) -> Vec<ac2_ui::UiMessage> {
        self.hud.text_input(actions, characters, modifiers)
    }

    /// What the HUD's chat asked since the last call ([`ac2_ui::ChatRequest`]).
    pub fn take_chat_requests(&mut self) -> Vec<ac2_ui::ChatRequest> {
        self.hud.take_chat_requests()
    }

    /// The HUD frameworks that ask the player (skills panel, shortcut bar, chat options), once a
    /// frame after `Client::use_time`; returns the gameplay commands they ask for.
    pub fn frameworks(
        &mut self,
        client: &mut crate::Client,
        frame: &crate::ClientFrame,
        events: &[ac2_weenie::SessionEvent],
        script_notices: &[ac2_script::host::ScriptNotice],
    ) -> Vec<crate::Command> {
        // The paper doll's pick from the scene as last drawn (`FinishPaperDollClick` runs on
        // the loop after the selection render).
        let doll_pick = self.hud.doll_wants_pick().map(|point| {
            let viewport = self.hud.doll_viewport()?;
            self.doll.pick(point, &viewport)
        });
        // `gm3DItemsUI`: a drop's target search finishes with the object under the pointer.
        let world_pick = self.hud.world_pick_point().map(|point| {
            let size = self.hud.window_size();
            self.pick(frame, size, point)
        });
        let mut commands = self.hud.items_3d_frame(client, frame, world_pick);
        commands.extend(self.hud.craft_frameworks(client, frame, events));
        commands.extend(self.hud.ingredient_frameworks(client, frame, events));
        commands.extend(
            self.hud
                .frameworks(client, frame, events, script_notices, doll_pick),
        );
        commands
    }

    /// `CM_UI::SendNotice_OpenExaminationPanel`: the examination panel shows.
    pub fn open_examination(&mut self) {
        self.hud.open_examination();
    }

    /// `RecvNotice_ReloadChatSettings`: the player's chat options (filters, fonts, popups).
    pub fn reload_chat_settings(&mut self, settings: &ac2_ui::ChatSettings) {
        self.hud.reload_chat_settings(settings);
    }

    /// The chat's command history answer: the entry line shows `text`.
    pub fn set_chat_entry(&mut self, text: &str) {
        self.hud.set_chat_entry(text);
    }

    /// The chat's retell / reply history answer (`ChatWindow::DoAssistedTell`).
    pub fn assisted_tell(&mut self, target: &str) {
        self.hud.assisted_tell(target);
    }

    /// The skills panel's panel and selected skill.
    pub fn skills_current(&self) -> Option<(u32, u32)> {
        self.hud.skills_current()
    }

    /// The HUD's text element `name` (`IUIManager` lookup by element id).
    pub fn element_text(&self, name: &str) -> Option<String> {
        self.hud.element_text(name)
    }

    /// The secure trade panel: whether it shows, the player's offers and the partner's.
    pub fn trade(&self) -> Option<(bool, Vec<u64>, Vec<u64>)> {
        let panel = self.hud.secure_trade.as_ref()?;
        Some((panel.is_shown(), panel.own_items(), panel.partner_items()))
    }

    /// The objects with a name plate (`gm3DTextUI`).
    pub fn named(&self) -> Vec<u64> {
        self.hud.named()
    }

    /// The character options page: its tab opens.
    pub fn character_open(&mut self) {
        self.hud.character_open();
    }

    /// The character options page: an option's checkbox is ticked or unticked.
    pub fn character_tick(&mut self, option: &str, on: bool) {
        self.hud.character_tick(option, on);
    }

    /// The character options page: one of its buttons is pressed (`ApplyButton`, ...); its
    /// requests become commands on the next [`View::frameworks`].
    pub fn character_press(&mut self, button: &str) {
        self.hud.character_press(button);
    }

    /// The character options page: an option's value as the page shows it.
    pub fn character_value(&self, option: &str) -> Option<f64> {
        self.hud.character_value(option)
    }

    /// A press on one of the navigation panel's buttons (`LockPositionButton`, `NavigateLeftButton`,
    /// ...), with Shift held or not (the short pan step). Returns whether the panel took it.
    pub fn navigation_press(&mut self, button: &str, shift: bool) -> bool {
        self.hud.navigation_press(button, shift)
    }

    /// A press on the navigation panel's map (`MapField`, `MapParentField`, `PlayerMarker`):
    /// the element's data code and the pixel relative to the map field.
    pub fn navigation_map_press(&mut self, element: &str, data: i32, pixel: (i32, i32)) -> bool {
        self.hud.navigation_map_press(element, data, pixel)
    }

    /// The navigation panel's lock to the character (`gmNavigationUI::ListenToMessage`'s lock):
    /// locking zooms to the character, unlocking leaves the window where it is.
    pub fn navigation_lock(&mut self, locked: bool) {
        self.hud.navigation_lock(locked);
    }

    /// The navigation panel and the tree it lives in, once the HUD is built.
    pub fn navigation(&mut self) -> Option<(&mut ac2_ui::UiTree, &mut ac2_ui::NavigationUi)> {
        self.hud.navigation_parts()
    }

    /// The radar's coordinates text.
    pub fn radar_coordinates(&self) -> Option<String> {
        self.hud.radar_coordinates()
    }

    /// The backpack's first page: each item and its slot's tooltip.
    pub fn backpack_items(&self) -> Vec<(u64, String)> {
        self.hud.backpack_items()
    }

    /// The external container panel's container and items, while it shows one.
    pub fn external_container_items(&self) -> Option<(u64, Vec<u64>)> {
        self.hud.external_container_items()
    }

    /// The item the paper doll shows worn at an inventory location.
    pub fn paper_doll_item(&self, location: u32) -> Option<u64> {
        self.hud.paper_doll.as_ref()?.item_at(location)
    }

    /// Shows or hides a toolbar panel, as its button does.
    pub fn show_panel(&mut self, child: ac2_ui::ToolbarChild, shown: bool) {
        self.hud.show_panel(child, shown);
    }

    /// The paper doll's viewport while it shows.
    pub fn paper_doll_viewport(&self) -> Option<ac2_ui::SceneViewport> {
        self.hud.doll_viewport()
    }

    /// The paper doll scene's pick at a screen point: the mesh type of the avatar's part there.
    pub fn paper_doll_pick(&self, point: (i32, i32)) -> Option<u32> {
        self.doll.pick(point, &self.hud.doll_viewport()?)
    }

    /// The allegiance page's member list texts, in order.
    pub fn allegiance_texts(&self) -> Vec<String> {
        self.hud.allegiance_texts()
    }

    /// The fellowship page's rows, top first: each member's id and name.
    pub fn fellowship_rows(&self) -> Vec<(u64, String)> {
        self.hud.fellowship_rows()
    }

    /// The examination panel's texts, in order.
    pub fn examination_texts(&self) -> Vec<String> {
        self.hud.examination_texts()
    }

    /// The shortcut bar's slots.
    pub fn shortcut_slots(&self) -> usize {
        self.hud.shortcut_slots()
    }

    /// The first chat window's log text.
    pub fn chat_log(&self) -> Option<String> {
        self.hud.chat_log()
    }

    /// Whether a toolbar panel is shown.
    pub fn panel_shown(&self, child: ac2_ui::ToolbarChild) -> bool {
        self.hud.panel_shown(child)
    }

    /// `RecvNotice_DisplayStringInfo`: a `StringInfo` line for the chat windows, rendered off the
    /// frame thread.
    pub fn chat_display_info(&mut self, kind: u32, info: &ac2_net::protocol::StringInfo) {
        self.hud.chat_display_info(kind, info);
    }

    /// A line for the chat windows, with its display type (`RecvNotice_DisplayStringInfo`).
    pub fn chat_display(&mut self, kind: u32, text: &str) {
        self.hud.chat_display(kind, text);
    }

    /// Whether the HUD takes typing (`CInputManager::SetTextMode`).
    pub fn text_mode(&self) -> bool {
        self.hud.text_mode()
    }

    /// The pointer as the HUD last saw it (`ICIDM` mouse position), in window pixels.
    pub fn pointer(&self) -> (i32, i32) {
        self.hud.pointer()
    }

    /// How many HUD surfaces are shown.
    pub fn hud_shown_layers(&self) -> usize {
        self.hud.shown_layers()
    }

    /// How many HUD surfaces are drawn.
    pub fn hud_layers(&self) -> usize {
        self.hud.layers()
    }

    /// Records the frame into a pass cleared to `SceneTool::backgroundColor` (black): the
    /// world, then the HUD.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.world_drawn {
            self.renderer.draw(pass);
        }
        self.fader.draw(pass);
        self.hud.draw(pass);
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
        let (overlay, roots) = self.hud.layers_for_scenes();
        crate::ui_scene::render_with_scenes(
            gpu,
            color,
            depth,
            &self.hud.scale,
            [0.0; 3],
            overlay,
            roots,
            &[self.doll.scenes()],
            |pass| {
                if world && self.world_drawn {
                    self.renderer.draw(pass);
                }
                self.fader.draw(pass);
            },
        );
    }
}

/// `gmRadarUI::Update`'s input from the frame: the range, the heading, each blip with its
/// object's name, and the selection.
fn radar_view(frame: &ClientFrame) -> ac2_ui::RadarView {
    let radar = &frame.radar;
    let name = |id| {
        frame
            .objects
            .iter()
            .find(|o| o.id == id)
            .and_then(|o| o.name.clone())
            .unwrap_or_default()
    };
    ac2_ui::RadarView {
        range: radar.range,
        heading_degrees: radar.heading_degrees,
        dots: radar
            .blips
            .iter()
            .map(|b| ac2_ui::RadarDot {
                id: b.id,
                color: b.color,
                shape: b.shape,
                offset: [b.offset[0], b.offset[1]],
                in_range: b.in_range,
                name: name(b.id),
            })
            .collect(),
        selection: frame.selection.as_ref().map_or(0, |s| s.id),
        coordinates: None,
    }
}
