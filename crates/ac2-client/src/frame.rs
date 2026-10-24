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

/// A remote object as the server last placed it.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectFrame {
    pub id: u64,
    /// The `VisualDesc` parent DID (its appearance) and the root Setup.
    pub visual: Option<u32>,
    pub setup: Option<u32>,
    pub cell: u32,
    pub origin: [f32; 3],
    pub rotation: [f32; 4],
    pub heading_degrees: Option<f32>,
}

/// Everything a renderer needs from one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientFrame {
    /// `Timer::cur_time` (server time).
    pub server_time: f64,
    /// The region calendar's time of day.
    pub time_of_day: Option<f32>,
    /// The player's cell, when in the world.
    pub cell_id: Option<u32>,
    pub player: Option<PlayerFrame>,
    /// Placed remote objects (held or contained ones have no position and are left out).
    pub objects: Vec<ObjectFrame>,
    /// The resident landblocks (`xxyy0000`).
    pub blocks: Vec<u32>,
}
