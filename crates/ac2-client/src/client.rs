//! `Turbine::Client::UseTime` (Legions 0x006A32A0, admin 0x00722F20) without rendering, sound
//! or UI: one call is one client frame, in retail's order.
//!
//! 1. `Timer::update_time`, `ClientNet::UseTime`: datagrams in and out.
//! 2. `DBCache::UseTime`: finished loader work, within a time budget (the cells' rings, the
//!    player body).
//! 3. `PlayerMovementModule::UseTime`: the actions drive the player's motion interpreter, then
//!    the movement and head reports.
//! 4. `SmartBox::UseTime`: `UpdatePlayer` (cells, world entry), `WorldManager::Update`
//!    (`GameTime::UseTime`, then unless blocking for cells `CObjectMaint::UseTime` and
//!    `CPhysics::UpdatePhysics`, whose every player quantum runs
//!    `SmartBox::PlayerPhysicsUpdatedCallback`), the mouse-look camera and `SceneCamera::slerp`.
//! 5. `CEventQueue::UseTime` and the logon queue: the server's events.
//! 6. The render step: `CPhysics::UpdateRenderInterpolation`, and the frame-drawn notice
//!    (`SceneRenderingComplete`) a world entry asked for.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ac2_physics::{
    AnimationClock, Frame, Heading, MovementInput, MovementState, PhysicsBody, PhysicsClock,
    Position, Quaternion, SetPositionError, Terrain, Vector, PHYSICS_QUANTUM,
};
use ac2_weenie::{
    ActiveAction, BodyFacts, BodyState, CellPosition, MovementCommands, MovementModule,
    OnlineSession, SessionEvent, WorldEntry,
};
use ac2_world::GameClock;

use ac2_render::portal::PortalSpace;

use crate::camera::{CameraManager, CameraSubject};
use crate::cells::CellManager;
use crate::loader::{Loaded, Loader, Request};
use crate::remote::Remotes;
use crate::scene::SceneCamera;
use crate::world_data::{BodyData, WorldData};

/// `SetPositionSimple(p, sliding = 1)`'s flags (`SyncPositionSimple(pos, 1)`): `0x1000` and
/// `SLIDE_SPF`.
pub const SYNC_POSITION_FLAGS: u32 = 0x1010;
/// `CPhysicsObj::EnterWorldSimple(pos, sliding = 1)`'s `SetPositionStruct` flags: `PLACEMENT`
/// and `SLIDE_SPF`.
pub const ENTER_WORLD_FLAGS: u32 = 0x11;
/// `SmartBox::mInitialCameraHeight`: the look-at point sits at this fraction of the height.
const INITIAL_CAMERA_HEIGHT: f32 = 0.75;
/// `SmartBox::UpdatePlayer`'s camera step on world entry.
const ENTRY_CAMERA_QUANTUM: f32 = 0.1;

/// A camera action from the action interpreter (`CameraManager::StartAction`/`StopAction`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraAction {
    pub action: u32,
    pub extent: f32,
    pub start: bool,
}

/// One frame's input: the actions the action interpreter resolved, the camera actions, and the
/// head direction (head x and z sliders) with whether the player looks at an object.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameInput {
    pub actions: Vec<ActiveAction>,
    pub camera_actions: Vec<CameraAction>,
    pub head: Option<((f32, f32), bool)>,
    /// The frame's mouse-look motion (`CallMouseLookHandler` -> `CameraManager::MouseMove`).
    pub mouse_look: Option<(i32, i32)>,
    /// `gmCameraUI::UpdateCamera`'s `SetMouseLookState` and `SetFirstPersonState`.
    pub mouse_look_state: Option<bool>,
    pub first_person_state: Option<bool>,
    /// `PlayerMovementModule::DoBehavior` / `StopBehavior` (behaviour, start) from behaviour-class
    /// actions, and `InterruptMoveTo` from a movement action starting.
    pub behaviors: Vec<(u32, bool)>,
    pub interrupt_move_to: bool,
    /// The UI's gameplay commands (selection, examination).
    pub commands: Vec<crate::gameplay::Command>,
}

/// Client-side notices of the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// `SendNotice_PlayerReadyToEnterWorld` (`SmartBox::UpdatePlayer`).
    PlayerReadyToEnterWorld,
    /// `SendNotice_NextFrameDrawn` (`WorldRender::Render`), which sent `SceneRenderingComplete`.
    NextFrameDrawn,
    /// `gmUIFlow::RecvNotice_ServerDied`: the connection was lost; retail switches to the
    /// Disconnected UI mode with `ID_NetErr_ConnectionLost` (table `0x41000018`).
    ConnectionLost,
    /// `ECM_Physics::SendNotice_PlayerTeleportStart` (`0x10419`): the teleport began.
    PlayerTeleportStart,
    /// The teleport ended: the destination is in and the player is placed there.
    TeleportEnd,
}

/// The player's simulated body, once its Setup has loaded.
pub(crate) struct Body {
    pub(crate) physics: PhysicsBody,
    /// `GetHeight` and `GetRadius` (x87 values) and `GetTotalScale` z.
    height: f64,
    radius: f64,
    scale: f32,
}

/// The player object (`CreateObject` of the player).
pub(crate) struct Player {
    pub(crate) id: u64,
    pub(crate) visual: Option<u32>,
    /// `get_position` until the body exists.
    position: Position,
    pub(crate) body: Option<Body>,
    /// `CPhysicsObj::IsInWorld`.
    pub(crate) in_world: bool,
    /// The heading the motion interpreter turns to, held across frames as `TurnTo` holds it.
    turn_to: Option<f32>,
    /// The object the server's move-to heads for (`RecvEvt_MoveTo` with an object target).
    pub(crate) move_to_target: Option<u64>,
    /// The player's `EntityDesc` ethereal bits were asked for (once its create is known).
    pub(crate) entity_mask_requested: bool,
}

impl Player {
    pub(crate) fn position(&self) -> Position {
        self.body
            .as_ref()
            .map_or(self.position, |body| body.physics.position())
    }
}

/// The headless client: an online session, the resident world and the player body.
pub struct Client {
    pub(crate) session: OnlineSession,
    pub(crate) loader: Loader,
    pub(crate) terrain: Terrain,
    pub(crate) cells: CellManager,
    pub(crate) player: Option<Player>,
    module: MovementModule,
    pub(crate) camera: CameraManager,
    pub(crate) scene: SceneCamera,
    pub(crate) game_clock: Option<GameClock>,
    /// The region, for its weather (`PhysicsGlobals::SetWind`).
    pub(crate) region: Option<Arc<ac2_world::RegionDesc>>,
    physics_clock: PhysicsClock,
    /// `PhysicsModule::m_tAnimationUpdateTime`.
    anim_clock: AnimationClock,
    /// `Random`, the client's shared generator (animation effect triggers roll on it).
    pub(crate) random: ac2_core::Random,
    /// `SmartBox::m_bBlockingLoad`.
    blocking_load: bool,
    /// `SmartBox::SetWantNoticeOnNextFrameDrawn`.
    want_frame_drawn: bool,
    /// `SmartBox::mtCameraUpdate`.
    camera_update_time: f64,
    /// `CPhysics::sRenderInterpolation`, from the last render step.
    pub(crate) render_interpolation: f32,
    /// `Timer::local_time` of the last render step: the time the poses are drawn at.
    render_time: f64,
    /// Whether the drawn pose blends between the last two bone refreshes (not retail; the
    /// retail pose is the last refresh, drawn as it is).
    pose_blend: bool,
    load_budget: Duration,
    pub(crate) notices: Vec<Notice>,
    /// Where the per-character window position files are (`None`: the player's AC2 directory).
    pub(crate) window_positions_dir: Option<std::path::PathBuf>,
    /// The physics objects of everything but the player.
    pub(crate) remotes: Remotes,
    /// The weenies' script processes, when a script VM is attached.
    pub(crate) scripts: Option<crate::scripts::Scripts>,
    pub(crate) audio: Option<crate::client_audio::ClientAudio>,
    /// The FX-made objects, colours and hides (`FXManager`'s object parts); no sound device needed.
    pub(crate) fx_objects: crate::fx_objects::FxObjects,
    /// The radar (`gmRadarUI`) and the map notes (`gmNavigationUI`).
    pub(crate) radar: crate::client_radar::RadarState,
    /// The GUN list and status events for the friends page (`gmFriendsUI`), in order.
    pub(crate) friend_events: Vec<crate::friends::FriendEvent>,
    /// `SmartBox`'s portal space (`WorldRender::m_PortalSpaceEffect`): the teleport's fade and tunnel.
    pub(crate) portal: PortalSpace,
    /// The DAT's FX scripts a body's `DoFXScript` plays (read when the client attaches its DAT).
    pub(crate) fx_scripts: crate::FxScripts,
    /// Between a teleport's start and its end (`SmartBox::HandleTeleport` to the placement).
    pub(crate) teleporting: bool,
    /// `gmGamePlayUI`'s story slot (`_storyUI`) and its portal-scene flag.
    pub(crate) story: ac2_ui::GameplayStory,
    /// The scenes the story holding the slot was made from, for `HandlePlayScenesDone`.
    pub(crate) story_scenes: Vec<ac2_weenie::SceneInfo>,
    /// The story slot's events the HUD draws (`StoryEvent::Show`, `Hide`, `StopAudio`), taken by
    /// [`HudView::story_frameworks`](crate::hud_view::HudView).
    pub(crate) story_queue: Vec<ac2_ui::StoryEvent>,
    /// Session events the story slot made outside a frame (the skip key, the session's end);
    /// they join the next frame's events.
    pub(crate) story_outbox: Vec<SessionEvent>,
    /// Session events a script resumed outside a frame made (a latent mode or behaviour that
    /// finished on its body); they join the next frame's events.
    pub(crate) script_outbox: Vec<SessionEvent>,
    /// Whether the HUD's background has a fade to report (`StoryView::fades_by_media`): the
    /// story waits for its `token_play` ([`Client::story_fade_done`]) instead of starting at once.
    pub(crate) story_fades_on_hud: bool,
    /// Whether mouse look may be on: a story holding the slot turns it off
    /// (`CM_UI::SendNotice_UpdateCanMouseLook`).
    pub(crate) mouse_look_allowed: bool,
    /// The client's preferences (`Preferences.ini`): the bank the options pages write, and the file.
    pub(crate) prefs: Option<crate::client_prefs::ClientPrefs>,
    /// `Turbine::Client`'s world name (`Turbine::Client::SetWorldName`, from the world-name
    /// notice); the scripts' `gTurbineGunUtils::CallEFun_GetWorldName` and the assistance page
    /// read it. Empty until the notice arrives.
    pub(crate) world_name: String,
}

fn physics_position(cell: u32, origin: [f32; 3], rotation: [f32; 4]) -> Position {
    let [w, x, y, z] = rotation;
    Position::new(
        cell,
        Frame::new(
            Vector::new(origin[0], origin[1], origin[2]),
            Quaternion::new(w, x, y, z),
        ),
    )
}

/// The report's view of the body (`PlayerMovementModule` reads it from `CPhysicsObj` and its
/// `CMotionInterp`).
fn body_state(state: &MovementState, actively_moving: bool) -> BodyState {
    BodyState {
        position: CellPosition {
            cell: state.cell_id,
            origin: state.origin,
        },
        heading_degrees: state.heading_degrees,
        turn_to_heading: state.turning_to.then_some(state.turn_to_degrees),
        motion: state.do_motion,
        times_jumped: state.times_jumped,
        velocity: state.velocity,
        actively_moving,
    }
}

impl Client {
    /// A client over `session` reading `data` on two loader threads.
    pub fn new(session: OnlineSession, data: Arc<dyn WorldData>) -> Self {
        let game_clock = data.game_clock();
        let region = data.region();
        let now = session.local_time();
        Self {
            session,
            loader: Loader::new(data, 2),
            terrain: Terrain::new(),
            cells: CellManager::default(),
            player: None,
            module: MovementModule::default(),
            camera: CameraManager::new(),
            scene: SceneCamera::default(),
            game_clock,
            region,
            physics_clock: PhysicsClock::new(now),
            anim_clock: AnimationClock::new(now),
            random: ac2_core::Random::default(),
            blocking_load: false,
            want_frame_drawn: false,
            camera_update_time: now,
            render_interpolation: 0.0,
            render_time: now,
            pose_blend: true,
            load_budget: Duration::from_millis(4),
            notices: Vec::new(),
            window_positions_dir: None,
            remotes: Remotes::default(),
            scripts: None,
            audio: None,
            fx_objects: crate::fx_objects::FxObjects::default(),
            radar: Default::default(),
            friend_events: Vec::new(),
            portal: PortalSpace::new(),
            fx_scripts: crate::FxScripts::default(),
            teleporting: false,
            story: Default::default(),
            story_scenes: Vec::new(),
            story_queue: Vec::new(),
            story_outbox: Vec::new(),
            script_outbox: Vec::new(),
            story_fades_on_hud: false,
            mouse_look_allowed: true,
            world_name: String::new(),
            prefs: None,
        }
    }

    pub fn session(&self) -> &OnlineSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut OnlineSession {
        &mut self.session
    }

    pub fn camera(&self) -> &CameraManager {
        &self.camera
    }

    /// Gives the client its preferences, read before the frame loop; the options pages write them.
    pub fn attach_prefs(&mut self, prefs: crate::client_prefs::ClientPrefs) {
        self.prefs = Some(prefs);
    }

    pub fn prefs(&self) -> Option<&crate::client_prefs::ClientPrefs> {
        self.prefs.as_ref()
    }

    pub fn prefs_mut(&mut self) -> Option<&mut crate::client_prefs::ClientPrefs> {
        self.prefs.as_mut()
    }

    /// The applied preferences as the live client reads them: the sound manager's volumes
    /// (`SoundManager::HandleApply`) and the camera's values (`CameraManager::HandleApply`). Returns
    /// the scene's graphics options for the view (`WorldScene::set_options`).
    pub fn apply_live_preferences(&mut self) -> Option<ac2_render::prefs::SceneOptions> {
        let prefs = self.prefs.take()?;
        self.apply_sound_preferences(prefs.bank());
        self.apply_light_preferences(prefs.bank());
        self.apply_streak_preferences(prefs.bank());
        self.camera
            .apply_prefs(&ac2_smartbox::prefs::values(prefs.bank()));
        let scene = prefs.scene_options();
        self.prefs = Some(prefs);
        Some(scene)
    }

    /// Whether drawn poses blend between the last two bone refreshes (on by default). Off draws
    /// the retail pose, the last refresh, as it is.
    pub fn set_pose_blend(&mut self, on: bool) {
        self.pose_blend = on;
    }

    /// The player's physics state: what the movement reports carry. The frame's player is drawn
    /// between physics quanta, so it is not the state the server is told.
    pub fn player_movement_state(&self) -> Option<MovementState> {
        let body = self.player.as_ref()?.body.as_ref()?;
        Some(body.physics.movement_state())
    }

    /// The time poses are blended at, or `None` when pose blending is off.
    pub(crate) fn pose_blend_time(&self) -> Option<f64> {
        self.pose_blend.then_some(self.render_time)
    }

    /// How long one frame may spend taking in finished loads.
    pub fn set_load_budget(&mut self, budget: Duration) {
        self.load_budget = budget;
    }

    /// Waits between frames on the session transport's clock.
    pub fn sleep(&mut self, duration: Duration) {
        self.session.sleep(duration);
    }

    /// Waits (on this thread) until the loader threads have finished everything asked of them,
    /// taking each result in as it comes (so a finished ring's follow-up is waited for too): a
    /// deterministic frame sequence for tests and probes. A frame never waits like this.
    pub fn settle_loads(&mut self, timeout: Duration) -> Result<(), String> {
        while self.loader.busy() {
            let loaded = self
                .loader
                .wait(timeout)
                .ok_or("the loader threads did not finish")?;
            self.take_loaded(loaded)?;
        }
        Ok(())
    }

    /// The notices since the last call.
    pub fn take_notices(&mut self) -> Vec<Notice> {
        std::mem::take(&mut self.notices)
    }

    /// One client frame (see the module docs); returns the session's events.
    pub fn use_time(&mut self, input: &FrameInput) -> Result<Vec<SessionEvent>, String> {
        {
            profile_scope!("net_use_time");
            self.session.net_use_time();
        }
        {
            profile_scope!("db_cache");
            self.db_cache()?;
        }
        {
            profile_scope!("input_handlers");
            self.input_handlers(input);
        }
        let commanded = {
            profile_scope!("commands");
            self.commands(&input.commands)
        };
        {
            profile_scope!("movement");
            self.movement(input);
        }
        let mut events = {
            profile_scope!("smart_box");
            self.smart_box()?
        };
        events.extend(commanded);
        let mut dispatched = {
            profile_scope!("dispatch_events");
            self.session.dispatch_events()
        };
        {
            profile_scope!("apply");
            for event in &dispatched {
                self.apply(event)?;
            }
        }
        self.apply_stories(&dispatched);
        {
            profile_scope!("run_scripts");
            self.run_scripts(&mut dispatched);
        }
        events.extend(dispatched);
        // The script notices of this frame reach the story slot (`CM_Quest`, `CM_UI`).
        let mut story = self.apply_story_notices(&events);
        // `gmGamePlayUI::EndSession`: the story slot is deleted when the session drops.
        if events
            .iter()
            .any(|event| matches!(event, SessionEvent::Disconnected(_)))
        {
            self.story_session_end();
        }
        {
            profile_scope!("story_frame");
            story.extend(self.story_frame());
        }
        story.append(&mut self.story_outbox);
        story.append(&mut self.script_outbox);
        events.extend(story);
        let produced = {
            profile_scope!("radar_use_time");
            self.radar_use_time(&events)
        };
        events.extend(produced);
        self.notices_seen(&events);
        self.portal_notices(&events);
        self.camera_actions(input);
        self.render();
        Ok(events)
    }

    /// `PlayerMovementModule::LockActions` (`m_lock_actions`): the player's movement keys do
    /// nothing while a behaviour that locks actions plays. The animation's `LockActions` and
    /// `UnlockActions` events for the player set it (`client_audio`).
    pub(crate) fn lock_player_actions(&mut self, locked: bool) {
        self.module.locked = locked;
    }

    /// What the input poll's handlers did this frame: mouse look (`CameraManager::MouseMove`),
    /// the camera UI's mouse-look and first-person states, and the player's behaviours and
    /// move-to interruption (`PlayerMovementModule`).
    fn input_handlers(&mut self, input: &FrameInput) {
        let subject = self.camera_subject(false);
        if let Some(state) = input.mouse_look_state {
            let state = state && self.mouse_look_allowed;
            self.camera.set_mouse_look(state, subject.as_ref());
        }
        if let Some(state) = input.first_person_state {
            self.camera.set_first_person(state);
        }
        if let Some((dx, dy)) = input.mouse_look {
            if self.camera.mouse_look && self.player.as_ref().is_some_and(|p| p.body.is_some()) {
                self.camera.mouse_move(dx, dy);
            }
        }
        let Some(body) = self.player.as_mut().and_then(|p| p.body.as_mut()) else {
            return;
        };
        if input.interrupt_move_to {
            body.physics.interrupt_move_to();
        }
        for &(behavior, start) in &input.behaviors {
            if start {
                body.physics
                    .do_behavior(&ac2_anim::Ac2BehaviorParams::new(behavior));
            } else {
                body.physics.stop_behavior(behavior);
            }
        }
    }

    /// `ActionInterpreter::UseTime`'s camera actions (`CameraManager::StartAction` and
    /// `StopAction`), which the next frame's camera update sees.
    fn camera_actions(&mut self, input: &FrameInput) {
        let subject = self.camera_subject(false);
        for camera_action in &input.camera_actions {
            if camera_action.start {
                self.camera.start_action(
                    camera_action.action,
                    camera_action.extent,
                    subject.as_ref(),
                );
            } else {
                self.camera
                    .stop_action(camera_action.action, subject.as_ref());
            }
        }
    }

    /// `DBCache::UseTime`: finished loads, within the frame's budget.
    fn db_cache(&mut self) -> Result<(), String> {
        let start = std::time::Instant::now();
        while let Some(loaded) = self.loader.try_next() {
            self.take_loaded(loaded)?;
            if start.elapsed() >= self.load_budget {
                break;
            }
        }
        Ok(())
    }

    fn take_loaded(&mut self, loaded: Loaded) -> Result<(), String> {
        match loaded {
            Loaded::Body(visual, body) => {
                self.remote_body_loaded(visual, body.clone());
                let for_player = self
                    .player
                    .as_ref()
                    .is_some_and(|p| p.visual == visual && p.body.is_none());
                if for_player {
                    self.body_loaded(visual, body?);
                }
            }
            Loaded::ScriptState(id, pid, did) => self.script_state_loaded(id, pid, did),
            Loaded::MotionDesc(object, desc) => self.motion_desc_loaded(object, desc),
            Loaded::EntityMask(object, mask) => self.entity_mask_loaded(object, mask),
            Loaded::ScriptData(answered) => self.script_data_loaded(answered),
            Loaded::Ambient(loaded) => self.ambient_loaded(loaded),
            Loaded::Block(block, result) => {
                let loaded = ac2_smartbox::BlockLoaded::Block(block, result);
                self.cells
                    .loaded(loaded, &mut self.terrain, &mut self.loader)?;
                self.object_cells();
            }
            Loaded::FarBlock(block, result) => {
                let loaded = ac2_smartbox::BlockLoaded::FarBlock(block, result);
                self.cells
                    .loaded(loaded, &mut self.terrain, &mut self.loader)?;
                self.object_cells();
            }
        }
        Ok(())
    }

    /// The player's Setup arrived: the body is built at the stored position, out of the world,
    /// and the camera targets it (`SmartBox::InitPlayer` -> `SetCameraDefaults`).
    fn body_loaded(&mut self, visual: Option<u32>, data: BodyData) {
        let Some(player) = self.player.as_mut().filter(|p| p.visual == visual) else {
            return;
        };
        if player.body.is_some() {
            return;
        }
        let scale = data.shape.scale;
        let height = f64::from(data.shape.height) * f64::from(scale);
        let radius = f64::from(data.radius) * f64::from(scale);
        let mut physics = PhysicsBody::new(player.position, data.shape, data.motion);
        // `CPhysicsObj::IsThePlayer`: the player's id is `ICPhysicsObj::player_id`.
        physics.set_player(true);
        physics.set_radius(data.radius);
        if let Some(animation) = data.animation {
            physics.set_animation(animation);
        }
        player.body = Some(Body {
            physics,
            height,
            radius,
            scale,
        });
        if let Some(subject) = self.camera_subject(false) {
            self.camera.set_player(&subject, INITIAL_CAMERA_HEIGHT);
        }
    }

    /// What the camera reads from the player's physics object.
    pub(crate) fn camera_subject(&self, require_in_world: bool) -> Option<CameraSubject> {
        let player = self.player.as_ref()?;
        if require_in_world && !player.in_world {
            return None;
        }
        let body = player.body.as_ref()?;
        let position = body.physics.position();
        let state = body.physics.movement_state();
        let [vx, vy, vz] = state.velocity;
        Some(CameraSubject {
            position,
            height: body.height,
            radius: body.radius,
            scale_z: body.scale,
            local_velocity: position.frame.global_to_local_vec(Vector::new(vx, vy, vz)),
            element: state.element,
            no_draw: self.session.player().is_some_and(|id| self.is_no_draw(id)),
        })
    }

    /// `PlayerMovementModule::UseTime`.
    fn movement(&mut self, input: &FrameInput) {
        let camera = camera_state(&self.camera);
        let Some(player) = self.player.as_mut().filter(|p| p.in_world) else {
            return;
        };
        let Some(body) = &mut player.body else {
            return;
        };
        let state = body.physics.movement_state();
        let facts = BodyFacts {
            moving_to: false,
            turning_to: state.turning_to,
            interpolating: false,
            heading_degrees: state.heading_degrees,
        };
        let commands = self.module.use_time(&facts, &input.actions, &camera);
        apply_commands(&mut player.turn_to, &mut body.physics, &commands, &state);
        let report = body_state(&body.physics.movement_state(), commands.actively_moving);
        self.session.report_body(&report);
        if let Some((head, looking_at_object)) = input.head {
            self.session.report_head(looking_at_object, head);
        }
    }

    /// `SmartBox::UseTime`.
    fn smart_box(&mut self) -> Result<Vec<SessionEvent>, String> {
        let mut events = self.update_player();
        events.extend(self.world_manager_update()?);
        let in_world = self.player.as_ref().is_some_and(|p| p.in_world);
        if in_world && self.camera.mouse_look {
            let now = self.session.local_time();
            let quantum = (now - self.camera_update_time) as f32;
            self.update_camera(quantum, false);
            self.camera_update_time = now;
        }
        self.scene.slerp(self.render_interpolation);
        // `CameraManager::UseTime`: no camera effects run, so the field of view is 65 and the
        // current position gains no distortion.
        self.scene.fov = self.camera.fov();
        Ok(events)
    }

    /// `SmartBox::UpdatePlayer`: the cells around the player, then (unless blocking for them)
    /// world entry: `ReEnterVisibility` places the body (`EnterWorldSimple`, sliding), the
    /// player is ready to enter the world, and the camera starts behind it.
    fn update_player(&mut self) -> Vec<SessionEvent> {
        let Some(player) = &self.player else {
            return Vec::new();
        };
        let cell = player.position().cell_id;
        self.cells.update_visible_cells(
            cell,
            self.blocking_load,
            &mut self.terrain,
            &mut self.loader,
        );
        self.blocking_load = false;
        self.object_cells();
        if self.cells.is_blocking() {
            return Vec::new();
        }
        let Some(player) = self.player.as_mut().filter(|p| !p.in_world) else {
            return Vec::new();
        };
        let entered = match &mut player.body {
            Some(body) if cell != 0 => {
                let position = body.physics.position();
                let placed = body
                    .physics
                    .place(position, ENTER_WORLD_FLAGS, &self.terrain);
                placed == SetPositionError::Ok
            }
            _ => false,
        };
        let mut events = Vec::new();
        if cell == 0 || entered {
            player.in_world = entered;
            self.notices.push(Notice::PlayerReadyToEnterWorld);
            // `gmGamePlayUI::RecvNotice_PlayerReadyToEnterWorld`: the portal gate.
            events = self.player_ready_to_enter_world();
            self.want_frame_drawn = true;
        }
        if entered && self.teleporting {
            self.teleport_ended();
        }
        if entered {
            let at = self
                .player
                .as_ref()
                .map(Player::position)
                .unwrap_or_default();
            let subject = self.camera_subject(true);
            let desired = self
                .camera
                .update_camera(ENTRY_CAMERA_QUANTUM, subject.as_ref(), &at);
            self.set_camera_position(desired, true);
        }
        events
    }

    /// `WorldRender::Update` (the SmartBox's world manager): `WorldManager::Update` (game time,
    /// then unless blocking for cells `CObjectMaint::UseTime` and `CPhysics::UpdatePhysics`),
    /// `CPhysics::UpdateRenderInterpolation`, then `CPhysics::UpdateAnimation`.
    fn world_manager_update(&mut self) -> Result<Vec<SessionEvent>, String> {
        if let Some(clock) = &mut self.game_clock {
            clock.use_time(self.session.cur_time());
        }
        let now = self.session.local_time();
        let mut events = Vec::new();
        if !self.cells.is_blocking() {
            events = self.session.object_maint_use_time();
            for event in &events {
                self.apply(event)?;
            }
            self.remotes.sync_shadows(&mut self.terrain);
            // `CPhysics::UpdatePhysics` on `Timer::local_time`.
            let quanta = self.physics_clock.advance(now);
            let cur_time = self.session.cur_time();
            // The player's impacts, in the order its quanta recorded them.
            let mut impacts = Vec::new();
            for _ in 0..quanta {
                self.track_move_to_target();
                let player = self
                    .player
                    .as_mut()
                    .filter(|p| p.in_world)
                    .and_then(|p| p.body.as_mut());
                if let Some(body) = player {
                    body.physics.step_quantum(&self.terrain);
                    impacts.extend(body.physics.take_impact_report());
                    // `SmartBox::PlayerPhysicsUpdatedCallback`.
                    if !self.camera.mouse_look {
                        self.update_camera(PHYSICS_QUANTUM, true);
                    }
                }
                // `CPhysics::UpdateAllPhysics`: the other objects.
                self.remotes.step_quantum(&mut self.terrain, cur_time);
            }
            // `CPhysicsObj::ReportCollisions` for the player: its script hears each impact.
            if let Some(id) = self.player.as_ref().map(|p| p.id) {
                for report in impacts {
                    self.collision_reported(id, report, &mut events);
                }
            }
        }
        self.render_interpolation = self.physics_clock.render_interpolation(now);
        self.render_time = now;
        // `WorldRender::Update` -> `PortalSpace::Effect::Think`.
        self.portal.think(now, &mut self.random);
        self.play_portal_sounds();
        // `PhysicsModule::UpdateAnimation` -> `CPhysics::UpdateAllAnimation`.
        if let Some(step) = self.anim_clock.advance(now) {
            // `WorldRender::UpdateTimeOfDay`'s `PhysicsGlobals::SetWind`.
            let wind = self.region.as_ref().map_or([0.0; 3], |region| {
                region.physics_wind(self.session.cur_time())
            });
            // `FXTrack::FillCache`: the pending FX scripts take their records before the step.
            let scripts = &self.fx_scripts;
            // `LookAtManager::Update` reads the look target's position and size each step.
            let mut looks = HashMap::new();
            self.remotes.look_targets(&mut looks);
            if let Some(player) = self
                .player
                .as_ref()
                .and_then(|p| Some((p.id, p.body.as_ref()?)))
            {
                looks.insert(player.0, crate::remote::look_snapshot(&player.1.physics));
            }
            if let Some(body) = self.player.as_mut().and_then(|p| p.body.as_mut()) {
                body.physics.resolve_fx_scripts(|id| scripts.get(id));
                body.physics.set_wind(wind);
                body.physics.set_look_snapshot(
                    body.physics
                        .look_target()
                        .and_then(|t| looks.get(&t).copied()),
                );
                body.physics.update_animation(step, now, &mut self.random);
            }
            self.remotes
                .update_animation(step, now, wind, &mut self.random, scripts, &looks);
            self.animation_sounds(step, now);
        }
        self.fx_objects_use_time(now);
        self.audio_use_time(now);
        self.rain_use_time(now);
        Ok(events)
    }

    /// `SmartBox::UpdateCamera(quantum, _bInterpolate)`: the camera manager's viewer position
    /// becomes the scene camera's current one, the old current its previous.
    fn update_camera(&mut self, quantum: f32, interpolate: bool) {
        let Some(subject) = self.camera_subject(true) else {
            return;
        };
        self.scene
            .update(&mut self.camera, &subject, quantum, interpolate);
    }

    /// `SmartBox::SetCameraPosition`.
    fn set_camera_position(&mut self, position: Position, set_desired: bool) {
        self.scene.set_position(position, set_desired);
    }

    /// The render step: `WorldRender::Render`'s
    /// frame-drawn notice (`gmGamePlayUI::RecvNotice_NextFrameDrawn` sends
    /// `SceneRenderingComplete`).
    fn render(&mut self) {
        if std::mem::take(&mut self.want_frame_drawn) {
            self.session.scene_ready();
            self.notices.push(Notice::NextFrameDrawn);
        }
    }

    /// `Turbine::Client::SetWorldName` (the name the notice carries; a non-literal name is
    /// kept as empty), and the script VM's copy of it.
    fn set_world_name(&mut self, info: &ac2_net::protocol::StringInfo) {
        self.world_name = info.literal.clone().unwrap_or_default();
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.host.set_world_name(&self.world_name);
        }
    }

    /// `Turbine::Client::GetWorldName`: the world's name ("" before the notice).
    pub fn world_name(&self) -> &str {
        &self.world_name
    }

    /// The events that create, move and remove the player.
    fn apply(&mut self, event: &SessionEvent) -> Result<(), String> {
        match event {
            SessionEvent::EnteredWorld(entry) => self.create_player(entry),
            SessionEvent::PlayerMoved {
                position,
                heading_degrees,
                teleported,
                impulse,
            } => self.player_moved(position, *heading_degrees, *teleported, *impulse),
            SessionEvent::ServerDied => self.notices.push(Notice::ConnectionLost),
            // `gmGamePlayUI::RecvNotice_LoginFinished` (Init, then the portal gate's leave).
            SessionEvent::LoginFinished => self.login_finished(),
            // `GlobalEventHandler::RecvEvt_WorldName` -> `gmClient::RecvNotice_WorldName`.
            SessionEvent::WorldName(info) => self.set_world_name(info),
            SessionEvent::LeftWorld | SessionEvent::Disconnected(_) => {
                self.player = None;
                self.camera.reset();
                self.remotes.clear(&mut self.terrain);
                self.cells
                    .update_visible_cells(0, false, &mut self.terrain, &mut self.loader);
                self.cells.take_changes();
            }
            other => self.apply_remote(other),
        }
        Ok(())
    }

    /// `CPhysicsObj::RecvEvt_Position`'s player branch: `SyncPositionSimple(pos, 1)` keeping the
    /// body's heading unless teleported, the impulse, then on a teleport
    /// `SmartBox::HandleTeleport` (out of the world at the new position, a blocking load).
    fn player_moved(
        &mut self,
        position: &CellPosition,
        heading_degrees: f32,
        teleported: bool,
        impulse: Option<[f32; 3]>,
    ) {
        let Some(player) = &mut self.player else {
            return;
        };
        let current = player.position();
        let heading = if teleported {
            heading_degrees
        } else {
            current.frame.heading().degrees()
        };
        let mut frame = Frame::new(
            Vector::new(position.origin[0], position.origin[1], position.origin[2]),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        );
        frame.set_heading(Heading::from_degrees(heading));
        let target = Position::new(position.cell, frame);
        match &mut player.body {
            Some(body) if player.in_world => {
                body.physics
                    .place(target, SYNC_POSITION_FLAGS, &self.terrain);
                if let Some(velocity) = impulse {
                    body.physics.set_velocity(velocity);
                }
            }
            Some(body) => body.physics.set_position(target),
            None => player.position = target,
        }
        if !teleported {
            return;
        }
        // `SmartBox::HandleTeleport`: the camera jumps to its place around the new position,
        // then the body leaves the world there (`LeaveWorld`, `StorePosition`).
        let subject = self.camera_subject(false);
        let desired = self
            .camera
            .update_camera(ENTRY_CAMERA_QUANTUM, subject.as_ref(), &target);
        self.set_camera_position(desired, true);
        if let Some(player) = &mut self.player {
            player.in_world = false;
            match &mut player.body {
                Some(body) => body.physics.set_position(target),
                None => player.position = target,
            }
        }
        self.blocking_load = true;
        self.teleport_started();
    }

    /// `SmartBox::RecvNotice_CreatePlayerObject`: the player object at the server position;
    /// its Setup loads on a loader thread, and the first cells load blocking (`m_bBlockingLoad`).
    fn create_player(&mut self, entry: &WorldEntry) {
        let position = physics_position(
            entry.position.cell,
            entry.position.origin,
            entry.orientation,
        );
        self.player = Some(Player {
            id: entry.player,
            visual: entry.visual,
            position,
            body: None,
            in_world: false,
            turn_to: None,
            move_to_target: None,
            entity_mask_requested: false,
        });
        self.loader.request(Request::Body(entry.visual));
        self.blocking_load = true;
    }
}

/// The `UseTime` calls on the body: jumps during the action loop, then `DoMotion` with the turn
/// (a `TurnTo` target stays until the turn ends; `SetOmegaVelocity` otherwise).
fn apply_commands(
    turn_to: &mut Option<f32>,
    body: &mut PhysicsBody,
    commands: &MovementCommands,
    state: &MovementState,
) {
    if commands.turn_off_run_lock || !commands.movement_active {
        return;
    }
    for _ in 0..commands.jumps {
        body.jump();
    }
    if let Some(heading) = commands.steer.or(commands.turn_to) {
        *turn_to = Some(heading);
    } else if !state.turning_to {
        *turn_to = None;
    }
    body.set_input(MovementInput {
        motion: commands.do_motion.unwrap_or(state.do_motion),
        heading: *turn_to,
        turn: commands.omega.unwrap_or(0.0),
    });
}

/// The `CameraState` `PlayerMovementModule::UseTime` reads.
pub fn camera_state(camera: &CameraManager) -> ac2_weenie::CameraState {
    ac2_weenie::CameraState {
        mouse_look: camera.mouse_look,
        first_person: camera.first_person,
        third_person_steers: camera.third_person_steers,
        direction: camera.global_direction(),
    }
}
