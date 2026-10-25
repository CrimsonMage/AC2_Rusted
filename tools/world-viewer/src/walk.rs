//! Walking the world offline: the player's body on the resident terrain (`ac2-physics`), the
//! cells around it kept resident on loader threads (`ac2_smartbox::CellManager`) and the player
//! camera (`CameraManager`). It stands in for the client's `SmartBox` without a session: the
//! keyboard becomes `MovementInput`, and a teleport is a world entry into a new cell.

use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_client::{PartPose, PlayerFrame};
use ac2_core::{DataId, Random};
use ac2_db::ResourceStore;
use ac2_physics::landdefs::lcoord_to_gid;
use ac2_physics::{
    AnimationClock, Frame, MovementInput, PhysicsClock, Position, Quaternion, Terrain, Vector,
    PHYSICS_QUANTUM,
};
use ac2_smartbox::{
    camera_action, load_body_data, BlockPool, BodyData, CameraFrame, CameraManager, CellManager,
    PlayerBody, SceneCamera,
};
use ac2_weenie::Appearance;
use ac2_world::Ac2WorldSource;

/// `SmartBox::SetCameraDefaults`' initial camera height factor and the entry camera's quantum.
const INITIAL_CAMERA_HEIGHT: f32 = 0.75;
const ENTRY_CAMERA_QUANTUM: f32 = 0.1;
/// `ENTER_WORLD_FLAGS`: how the client places a new player (slide onto the terrain).
const ENTER_WORLD_FLAGS: u32 = 0x11;
/// How far above the ground a new player drops from.
const DROP_HEIGHT: f32 = 1.0;
/// Frame-thread time a frame spends taking finished block loads.
const LOAD_BUDGET: Duration = Duration::from_millis(4);
const LOADER_THREADS: usize = 2;
/// The player's object id in the synthetic frame.
const PLAYER_ID: u64 = 1;

/// One frame's keys as movement: each axis in `[-1, 1]`.
#[derive(Debug, Default, Clone, Copy)]
pub struct WalkInput {
    pub forward: f32,
    pub strafe: f32,
    /// Positive turns left (counter-clockwise).
    pub turn: f32,
    pub jump: bool,
}

/// What the status line shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkStatus {
    pub cell: u32,
    pub origin: [f32; 3],
    pub heading_degrees: f32,
    pub contact: bool,
    pub resident_blocks: usize,
    pub in_world: bool,
}

/// Where a teleport or an entry lands: a block and the position within it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Landing {
    pub block: (u8, u8),
    pub x: f32,
    pub y: f32,
}

impl Landing {
    /// The outdoor cell and the position at the landing's ground (z is set once the terrain is
    /// resident).
    fn position(&self) -> Position {
        let square = ac2_physics::landdefs::SQUARE_LENGTH;
        let side = ac2_physics::landdefs::LBLOCK_SIDE;
        let cell_x = i32::from(self.block.0) * side + ((self.x / square) as i32).clamp(0, side - 1);
        let cell_y = i32::from(self.block.1) * side + ((self.y / square) as i32).clamp(0, side - 1);
        Position::new(
            lcoord_to_gid(cell_x, cell_y),
            Frame::new(Vector::new(self.x, self.y, 0.0), Quaternion::IDENTITY),
        )
    }
}

pub struct Walk {
    store: Arc<ResourceStore>,
    block_size: f32,
    terrain: Terrain,
    cells: CellManager,
    pool: BlockPool,
    player: Option<PlayerBody>,
    /// The body being loaded for a new model, and the position it takes over.
    loading: Option<Receiver<Result<BodyData, String>>>,
    /// Where the player enters once the cells around it are resident.
    entry: Option<Position>,
    in_world: bool,
    pub camera: CameraManager,
    scene: SceneCamera,
    physics_clock: PhysicsClock,
    anim_clock: AnimationClock,
    random: Random,
    /// Physics quanta run per quantum owed (a speed-up of the whole body, animation included).
    pub speed: u32,
    turn_to: Option<f32>,
    now: f64,
    /// The last load or entry failure.
    pub message: Option<String>,
}

impl Walk {
    pub fn new(world: &Arc<Ac2WorldSource>, store: Arc<ResourceStore>, now: f64) -> Self {
        Self {
            store,
            block_size: world.block_size(),
            terrain: Terrain::new(),
            cells: CellManager::default(),
            pool: BlockPool::new(Arc::clone(world) as _, LOADER_THREADS),
            player: None,
            loading: None,
            entry: None,
            in_world: false,
            camera: CameraManager::new(),
            scene: SceneCamera::default(),
            physics_clock: PhysicsClock::new(now),
            anim_clock: AnimationClock::new(now),
            random: Random::default(),
            speed: 1,
            turn_to: None,
            now,
            message: None,
        }
    }

    /// Loads the body for `visual` on a thread; the player takes it over where they stand (or
    /// at `landing` for the first body).
    pub fn set_model(&mut self, visual: DataId) {
        let (sender, receiver) = channel();
        let store = Arc::clone(&self.store);
        std::thread::spawn(move || {
            let _ = sender.send(load_body_data(&store, visual));
        });
        self.loading = Some(receiver);
    }

    /// World entry at `landing`: the cells there load, then the player drops onto the ground.
    pub fn teleport(&mut self, landing: Landing) {
        self.entry = Some(landing.position());
        self.in_world = false;
        self.message = None;
    }

    pub fn in_world(&self) -> bool {
        self.in_world
    }

    /// The player's position, for placing the fly camera or the status line.
    pub fn position(&self) -> Option<Position> {
        self.player.as_ref().map(|p| p.physics.position())
    }

    pub fn status(&self) -> Option<WalkStatus> {
        let player = self.player.as_ref()?;
        let state = player.physics.movement_state();
        Some(WalkStatus {
            cell: state.cell_id,
            origin: state.origin,
            heading_degrees: state.heading_degrees,
            contact: state.contact,
            resident_blocks: self.cells.resident().count(),
            in_world: self.in_world,
        })
    }

    /// The renderer's camera for this frame, while the player is in the world.
    pub fn camera_frame(&self) -> Option<CameraFrame> {
        let player = self.player.as_ref().filter(|_| self.in_world)?;
        Some(self.scene.frame(&self.camera, player.scale))
    }

    /// The player as the client's frame carries it, once the body is in the world: drawn at
    /// the render interpolation between its last two quanta, posed by its animation.
    pub fn player_frame(&self, visual: DataId) -> Option<PlayerFrame> {
        let player = self.player.as_ref().filter(|_| self.in_world)?;
        let t = self.physics_clock.render_interpolation(self.now);
        let position = player.physics.drawing_position(t);
        let state = player.physics.movement_state();
        let (o, q) = (position.frame.origin, position.frame.q);
        Some(PlayerFrame {
            id: PLAYER_ID,
            cell: position.cell_id,
            origin: [o.x, o.y, o.z],
            rotation: [q.w, q.x, q.y, q.z],
            heading_degrees: state.heading_degrees,
            visual: Some(visual.raw()),
            velocity: state.velocity,
            contact: state.contact,
            pose: player
                .physics
                .bone_transforms()
                .iter()
                .map(|t| PartPose {
                    offset: [t.offset.x, t.offset.y, t.offset.z],
                    rotation: [t.rotation.w, t.rotation.x, t.rotation.y, t.rotation.z],
                    scale: [t.scale.x, t.scale.y, t.scale.z],
                })
                .collect(),
            // `CPhysicsObj::m_local_vdesc`: the server names the DAT `VisualDesc` it adds to.
            appearance: Appearance {
                parent: visual.raw(),
                ..Appearance::default()
            },
        })
    }

    /// The resident landblocks (`xxyy0000`), as the frame lists them.
    pub fn resident_blocks(&self) -> Vec<u32> {
        self.cells.resident().collect()
    }

    pub fn block_size(&self) -> f32 {
        self.block_size
    }

    fn take_loads(&mut self) {
        let start = Instant::now();
        while let Some(loaded) = self.pool.try_next() {
            if let Err(error) = self.cells.loaded(loaded, &mut self.terrain, &mut self.pool) {
                self.message = Some(error);
            }
            if start.elapsed() >= LOAD_BUDGET {
                break;
            }
        }
    }

    fn take_body(&mut self) {
        let Some(receiver) = &self.loading else {
            return;
        };
        let data = match receiver.try_recv() {
            Ok(data) => data,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("the body loader stopped".to_string()),
        };
        self.loading = None;
        match data {
            Ok(data) => {
                // The old body's place, or the entry's, until the cells are resident.
                let position = self
                    .player
                    .as_ref()
                    .map(|p| p.physics.position())
                    .or(self.entry)
                    .unwrap_or_default();
                let player = PlayerBody::new(position, data);
                if self.in_world {
                    let mut player = player;
                    let placed = player
                        .physics
                        .place(position, ENTER_WORLD_FLAGS, &self.terrain);
                    if placed != ac2_physics::SetPositionError::Ok {
                        self.message = Some(format!("the new body cannot stand here: {placed:?}"));
                    }
                    self.camera
                        .set_player(&player.camera_subject(), INITIAL_CAMERA_HEIGHT);
                    self.player = Some(player);
                } else {
                    self.player = Some(player);
                }
            }
            Err(error) => self.message = Some(error),
        }
    }

    /// `SmartBox::UpdatePlayer`: the world entry, once the cells around the entry are resident.
    fn enter_world(&mut self) {
        let (Some(entry), Some(player)) = (self.entry, self.player.as_mut()) else {
            return;
        };
        if self.cells.is_blocking() {
            return;
        }
        let Some(ground) = self.terrain.terrain_height(&entry) else {
            self.message = Some("there is no outdoor terrain at that spot".into());
            self.entry = None;
            return;
        };
        let mut at = entry;
        at.frame.origin.z = ground + DROP_HEIGHT;
        let placed = player.physics.place(at, ENTER_WORLD_FLAGS, &self.terrain);
        self.entry = None;
        if placed != ac2_physics::SetPositionError::Ok {
            self.message = Some(format!("cannot enter the world here: {placed:?}"));
            return;
        }
        self.in_world = true;
        let subject = player.camera_subject();
        self.camera.set_player(&subject, INITIAL_CAMERA_HEIGHT);
        let desired = self
            .camera
            .update_camera(ENTRY_CAMERA_QUANTUM, Some(&subject), &at);
        self.scene.set_position(desired, true);
        self.scene.render = self.scene.current;
        self.physics_clock = PhysicsClock::new(self.now);
    }

    /// Turns the camera with the mouse while mouse look is held.
    pub fn mouse_look(&mut self, dx: f32, dy: f32) {
        if self.camera.mouse_look {
            self.camera.mouse_move(dx as i32, dy as i32);
        }
    }

    /// A camera action (`camera_action::*`) started or stopped.
    pub fn camera_action(&mut self, action: u32, start: bool) {
        let subject = self.player.as_ref().map(|p| p.camera_subject());
        if start {
            self.camera.start_action(action, 1.0, subject.as_ref());
        } else {
            self.camera.stop_action(action, subject.as_ref());
        }
    }

    /// The player's body turns to the camera's heading while it looks around and walks.
    fn steer(&mut self, moving: bool) -> Option<f32> {
        if self.camera.mouse_look && self.camera.third_person_steers && moving {
            let [x, y, _] = self.camera.global_direction();
            let degrees = x.atan2(y).to_degrees().rem_euclid(360.0);
            self.turn_to = Some(degrees);
        } else if !self.player.as_ref()?.physics.movement_state().turning_to {
            self.turn_to = None;
        }
        self.turn_to
    }

    /// One frame: finished loads, the cells around the player, then physics, animation and the
    /// camera for the quanta owed at `now`.
    pub fn update(&mut self, dt: f32, now: f64, input: WalkInput) {
        self.now = now;
        self.take_loads();
        self.take_body();
        let focus = self
            .entry
            .map(|p| p.cell_id)
            .or_else(|| self.position().map(|p| p.cell_id));
        if let Some(cell) = focus {
            self.cells.update_visible_cells(
                cell,
                self.entry.is_some(),
                &mut self.terrain,
                &mut self.pool,
            );
        }
        if self.entry.is_some() {
            self.enter_world();
        }
        if !self.in_world || self.cells.is_blocking() {
            return;
        }
        let moving = input.forward != 0.0 || input.strafe != 0.0;
        let heading = self.steer(moving);
        let Some(player) = self.player.as_mut() else {
            return;
        };
        if input.jump {
            player.physics.jump();
        }
        let state = player.physics.movement_state();
        player.physics.set_input(MovementInput {
            motion: [input.strafe, input.forward, state.do_motion[2]],
            heading,
            turn: if heading.is_some() { 0.0 } else { input.turn },
        });
        player.physics.set_time(now);
        let quanta = self.physics_clock.advance(now) * self.speed.max(1);
        for _ in 0..quanta {
            player.physics.step_quantum(&self.terrain);
            if !self.camera.mouse_look {
                let subject = player.camera_subject();
                self.scene
                    .update(&mut self.camera, &subject, PHYSICS_QUANTUM, true);
            }
        }
        if let Some(step) = self.anim_clock.advance(now) {
            for _ in 0..self.speed.max(1) {
                player.physics.update_animation(step, now, &mut self.random);
            }
        }
        if self.camera.mouse_look {
            let subject = player.camera_subject();
            self.scene.update(&mut self.camera, &subject, dt, false);
        }
        self.scene
            .slerp(self.physics_clock.render_interpolation(now));
    }
}

/// Camera actions the viewer maps keys to.
pub mod actions {
    pub use super::camera_action::{
        MOUSE_LOOK_HOLD, RESET, ROTATE_LEFT, ROTATE_RIGHT, TOGGLE_FIRST_PERSON, ZOOM_IN, ZOOM_OUT,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_landing_is_an_outdoor_cell_of_its_block() {
        let landing = Landing {
            block: (0x7F, 0x7F),
            x: 80.0,
            y: 80.0,
        };
        let position = landing.position();
        assert_eq!(position.cell_id >> 16, 0x7F7F);
        assert!((1..=64).contains(&(position.cell_id & 0xFFFF)));
        assert_eq!(position.frame.origin.x, 80.0);
        // The block's far corner stays inside its last cell.
        let corner = Landing {
            block: (0x10, 0x20),
            x: 159.9,
            y: 159.9,
        }
        .position();
        assert_eq!(corner.cell_id >> 16, 0x1020);
        assert_eq!(corner.cell_id & 0xFFFF, 64);
    }
}
