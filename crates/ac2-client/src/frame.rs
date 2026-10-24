//! The plain-data snapshot of one client frame, for the renderer.

/// The player's body as physics left it this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerFrame {
    pub id: u64,
    pub cell: u32,
    /// AC axes relative to the cell's landblock.
    pub origin: [f32; 3],
    /// (w, x, y, z).
    pub rotation: [f32; 4],
    pub heading_degrees: f32,
    pub visual: Option<u32>,
    pub velocity: [f32; 3],
    pub contact: bool,
}

/// An object other than the player: where its physics object is (in the client's world), or
/// else where the server last placed it, with what to draw it with.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectFrame {
    pub id: u64,
    /// The `VisualDesc` parent DID (its appearance) and the root Setup.
    pub visual: Option<u32>,
    pub setup: Option<u32>,
    pub name: Option<String>,
    pub cell: u32,
    /// AC axes relative to the cell's landblock.
    pub origin: [f32; 3],
    /// (w, x, y, z).
    pub rotation: [f32; 4],
    pub heading_degrees: Option<f32>,
    /// `CPhysicsObj::IsInWorld`: the object stands in a resident cell (draw it); otherwise it
    /// waits in a lost cell at its last position.
    pub in_world: bool,
    /// `CBookKeep::m_lostCellID`: the cell it waits for while out of the world.
    pub lost_cell: Option<u32>,
    /// Within physics range of the player (96 m); out of range it snaps to its positions.
    pub active: bool,
    /// The physics object's velocity (world axes, m/s) and ground contact.
    pub velocity: [f32; 3],
    pub contact: bool,
    /// `PhysicsDesc::mode` / the latest `SetMode`, and the do-motion values of the latest
    /// position (`PositionPack`): what its motion table would play.
    pub mode: Option<u32>,
    pub motion: [f32; 3],
    /// Holder and location of a held object.
    pub parent: Option<(u64, u32)>,
}

/// Everything a renderer needs from one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientFrame {
    /// `Timer::cur_time` (server time).
    pub server_time: f64,
    /// `GameTime::present_time_of_day`.
    pub time_of_day: Option<f32>,
    /// The player's cell, when its body exists.
    pub cell_id: Option<u32>,
    /// `CPhysicsObj::IsInWorld` for the player.
    pub in_world: bool,
    /// `CellManager::IsBlockingForCells`: the player's physics waits for its cells.
    pub blocking_for_cells: bool,
    pub player: Option<PlayerFrame>,
    /// The camera, while the player is in the world.
    pub camera: Option<crate::scene::CameraFrame>,
    /// Placed remote objects (held or contained ones have no position and are left out).
    pub objects: Vec<ObjectFrame>,
    /// The resident landblocks (`xxyy0000`).
    pub blocks: Vec<u32>,
}
