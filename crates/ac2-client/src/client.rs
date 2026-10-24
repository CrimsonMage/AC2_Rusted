//! `Turbine::Client::UseTime` (Legions 0x006A32A0, admin 0x00722F20) without rendering, sound
//! or UI: one call is one client frame, in retail's order.
//!
//! 1. `Timer::update_time`, `ClientNet::UseTime`: datagrams in and out.
//! 2. `PlayerMovementModule::UseTime`: the actions drive the player's motion interpreter, then
//!    the movement and head reports.
//! 3. `SmartBox::UseTime`: `UpdatePlayer` makes the landblocks around the player resident
//!    (`CellManager::UpdateVisibleCells`), then `WorldManager::Update` steps physics
//!    (`CPhysics::UpdatePhysics` on `Timer::local_time`, `PHYSICS_QUANTUM` steps).
//! 4. `CEventQueue::UseTime` and the logon queue: the server's events, which create and move
//!    the player body.

use std::time::Duration;

use ac2_physics::{
    Frame, Heading, MovementInput, MovementState, PhysicsBody, PhysicsClock, Position, Quaternion,
    Terrain, Vector,
};
use ac2_weenie::{
    ActiveAction, BodyFacts, BodyState, CameraState, CellPosition, MovementCommands,
    MovementModule, OnlineSession, SessionEvent, WorldEntry,
};

use crate::cells::CellManager;
use crate::frame::{ClientFrame, ObjectFrame, PlayerFrame};
use crate::world_data::WorldData;

/// `SetPositionSimple(p, sliding = 1)`'s flags (`SyncPositionSimple(pos, 1)`): `0x1000` and
/// `SLIDE_SPF`.
pub const SYNC_POSITION_FLAGS: u32 = 0x1010;

/// One frame's input: the actions the action interpreter resolved, the camera, and the head
/// direction (head x and z sliders) with whether the player looks at an object.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameInput {
    pub actions: Vec<ActiveAction>,
    pub camera: CameraState,
    pub head: Option<((f32, f32), bool)>,
}

/// The player's simulated body.
struct Player {
    id: u64,
    visual: Option<u32>,
    body: PhysicsBody,
    clock: PhysicsClock,
    /// The heading the motion interpreter turns to, held across frames as `TurnTo` holds it.
    turn_to: Option<f32>,
}

/// The headless client: an online session, the resident world and the player body.
pub struct Client {
    session: OnlineSession,
    data: Box<dyn WorldData>,
    terrain: Terrain,
    cells: CellManager,
    player: Option<Player>,
    module: MovementModule,
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
    pub fn new(session: OnlineSession, data: Box<dyn WorldData>) -> Self {
        Self {
            session,
            data,
            terrain: Terrain::new(),
            cells: CellManager::default(),
            player: None,
            module: MovementModule::default(),
        }
    }

    pub fn session(&self) -> &OnlineSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut OnlineSession {
        &mut self.session
    }

    /// Waits between frames on the session transport's clock.
    pub fn sleep(&mut self, duration: Duration) {
        self.session.sleep(duration);
    }

    /// One client frame (see the module docs); returns the session's events.
    pub fn use_time(&mut self, input: &FrameInput) -> Result<Vec<SessionEvent>, String> {
        self.session.net_use_time();
        self.movement(input);
        self.smart_box()?;
        let events = self.session.dispatch_events();
        for event in &events {
            self.apply(event)?;
        }
        // `gmGamePlayUI::RecvNotice_NextFrameDrawn`: once a frame has run with the player in the
        // world, the scene counts as rendered (the session sends it once).
        if self.player.is_some() {
            self.session.scene_ready();
        }
        Ok(events)
    }

    /// `PlayerMovementModule::UseTime`.
    fn movement(&mut self, input: &FrameInput) {
        let Some(player) = &mut self.player else {
            return;
        };
        let state = player.body.movement_state();
        let facts = BodyFacts {
            moving_to: false,
            turning_to: state.turning_to,
            interpolating: false,
            heading_degrees: state.heading_degrees,
        };
        let commands = self.module.use_time(&facts, &input.actions, &input.camera);
        apply_commands(player, &commands, &state);
        let report = body_state(&player.body.movement_state(), commands.actively_moving);
        self.session.report_body(&report);
        if let Some((head, looking_at_object)) = input.head {
            self.session.report_head(looking_at_object, head);
        }
    }

    /// `SmartBox::UseTime`: `UpdatePlayer`, then `WorldManager::Update`'s physics step.
    fn smart_box(&mut self) -> Result<(), String> {
        let Some(player) = &mut self.player else {
            return Ok(());
        };
        let cell = player.body.position().cell_id;
        self.cells
            .update_visible_cells(cell, &mut self.terrain, &*self.data)?;
        let now = self.session.local_time();
        player.body.update(&mut player.clock, now, &self.terrain);
        Ok(())
    }

    /// The events that create, move and remove the player body.
    fn apply(&mut self, event: &SessionEvent) -> Result<(), String> {
        match event {
            SessionEvent::EnteredWorld(entry) => self.create_player(entry)?,
            SessionEvent::PlayerMoved {
                position,
                heading_degrees,
                teleported,
                impulse,
            } => {
                let Some(player) = &mut self.player else {
                    return Ok(());
                };
                // `RecvEvt_Position`: the received position faces the body's own heading unless
                // this was a teleport (`Position::set_heading`).
                let heading = if *teleported {
                    *heading_degrees
                } else {
                    player.body.movement_state().heading_degrees
                };
                let mut frame = Frame::new(
                    Vector::new(position.origin[0], position.origin[1], position.origin[2]),
                    Quaternion::new(1.0, 0.0, 0.0, 0.0),
                );
                frame.set_heading(Heading::from_degrees(heading));
                let target = Position::new(position.cell, frame);
                self.cells
                    .update_visible_cells(position.cell, &mut self.terrain, &*self.data)?;
                player
                    .body
                    .place(target, SYNC_POSITION_FLAGS, &self.terrain);
                if let Some(velocity) = impulse {
                    player.body.set_velocity(*velocity);
                }
            }
            SessionEvent::LeftWorld | SessionEvent::Disconnected(_) => {
                self.player = None;
                self.cells
                    .update_visible_cells(0, &mut self.terrain, &*self.data)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// The player's `CreateObject` (the session's world entry): its cells, then the body placed
    /// at the server position.
    fn create_player(&mut self, entry: &WorldEntry) -> Result<(), String> {
        let cell = entry.position.cell;
        self.cells
            .update_visible_cells(cell, &mut self.terrain, &*self.data)?;
        let body = self.data.body(entry.visual)?;
        let position = physics_position(cell, entry.position.origin, entry.orientation);
        let mut physics = PhysicsBody::new(position, body.shape, body.motion);
        physics.place(position, SYNC_POSITION_FLAGS, &self.terrain);
        self.player = Some(Player {
            id: entry.player,
            visual: entry.visual,
            body: physics,
            clock: PhysicsClock::new(self.session.local_time()),
            turn_to: None,
        });
        Ok(())
    }

    /// The frame for the renderer.
    pub fn frame(&self) -> ClientFrame {
        let server_time = self.session.cur_time();
        let player = self.player.as_ref().map(|p| {
            let state = p.body.movement_state();
            PlayerFrame {
                id: p.id,
                cell: state.cell_id,
                origin: state.origin,
                rotation: state.rotation,
                heading_degrees: state.heading_degrees,
                visual: p.visual,
                velocity: state.velocity,
                contact: state.contact,
            }
        });
        let player_id = player.map(|p| p.id);
        let objects = self
            .session
            .objects()
            .iter()
            .filter(|o| Some(o.id) != player_id)
            .filter_map(|o| {
                let position = o.position?;
                Some(ObjectFrame {
                    id: o.id,
                    visual: o.visual,
                    setup: o.setup,
                    cell: position.cell,
                    origin: position.origin,
                    rotation: position.rotation,
                    heading_degrees: o.heading_degrees,
                })
            })
            .collect();
        ClientFrame {
            server_time,
            time_of_day: self.data.time_of_day(server_time),
            cell_id: player.map(|p| p.cell),
            player,
            objects,
            blocks: self.cells.resident().collect(),
        }
    }
}

/// The `UseTime` calls on the body: jumps during the action loop, then `DoMotion` with the turn
/// (a `TurnTo` target stays until the turn ends; `SetOmegaVelocity` otherwise).
fn apply_commands(player: &mut Player, commands: &MovementCommands, state: &MovementState) {
    if commands.turn_off_run_lock || !commands.movement_active {
        return;
    }
    for _ in 0..commands.jumps {
        player.body.jump();
    }
    if let Some(heading) = commands.steer.or(commands.turn_to) {
        player.turn_to = Some(heading);
    } else if !state.turning_to {
        player.turn_to = None;
    }
    player.body.set_input(MovementInput {
        motion: commands.do_motion.unwrap_or(state.do_motion),
        heading: player.turn_to,
        turn: commands.omega.unwrap_or(0.0),
    });
}
