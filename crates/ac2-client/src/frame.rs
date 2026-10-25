//! The plain-data snapshot of one client frame, for the renderer.

use ac2_net::protocol::StringInfo;

/// The player's body as physics left it this frame.
#[derive(Debug, Clone, PartialEq)]
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
    /// The root Setup's bones relative to their parents as the animation left them (empty
    /// without a Setup).
    pub pose: Vec<crate::motion::PartPose>,
    /// The player's local `VisualDesc` (`CPhysicsObj::m_local_vdesc`), as for other objects.
    pub appearance: ac2_weenie::Appearance,
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
    /// The object holding this one at a holding location (`CPhysicsObj::set_parent`: a wielded
    /// or carried item), drawn there rather than at `origin`.
    pub holder: Option<Holder>,
    /// `gmWeenieDesc::m_containerID` and `m_wielderID` (qualities `0x102`, `0x105`): the weenie
    /// graph's container and wielder.
    pub container: Option<u64>,
    pub wielder: Option<u64>,
    /// `CPhysicsObj::m_local_vdesc`: the server's appearance on top of `visual`'s DAT chain
    /// (Setup, part-group, attachment and appearance-table overrides).
    pub appearance: ac2_weenie::Appearance,
    /// The root Setup's bones relative to their parents as the animation left them, when its
    /// physics object exists.
    pub pose: Vec<crate::motion::PartPose>,
}

/// Where a held object hangs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Holder {
    pub id: u64,
    /// The holder's holding location (`PhysicsDesc` location / `RecvEvt_Parent`).
    pub location: u32,
    /// The placement frame the held object takes there (`SetPlacementFrame`).
    pub orientation: Option<u32>,
}

/// One object on the radar (`gmRadarUI::RadarInfo`): its look, and where it is from the player.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarBlip {
    pub id: u64,
    /// The `RGBAColor` the object's script gives (r, g, b, a).
    pub color: [f32; 4],
    /// The blip's shape, `0x40000001` (point) to `0x4000000A` (face).
    pub shape: u32,
    /// The object's cell and AC origin.
    pub cell: u32,
    pub origin: [f32; 3],
    /// The object's offset in the player's frame (`Position::localtolocal`).
    pub offset: [f32; 3],
    /// Within the radar's range (`gmRadarUI::Update` draws only these).
    pub in_range: bool,
    /// The selected object (`gmRadarUI::_currentSelection`).
    pub selected: bool,
}

/// What the radar shows (`gmRadarUI`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadarFrame {
    /// `gmRadarUI::_grooveLevel`: the danger the music's groove follows.
    pub groove: i8,
    /// The range the radar shows where the player stands (indoor or outdoor).
    pub range: f32,
    /// The player's heading, which the radar's heading field turns with.
    pub heading_degrees: Option<f32>,
    /// The objects with a blip, in the radar's own order (`gmRadarUI::_infoArray`).
    pub blips: Vec<RadarBlip>,
}

/// A map note (`gmNavigationUI`'s marker) as `CM_Character::SendNotice_AddMarker` sends it. The
/// navigation panel draws these, not the radar.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapMarker {
    /// The marker's type (a `CMapNoteRow`); type 3 is never shown.
    pub kind: u32,
    /// The marked object's instance id (0 for none).
    pub id: u64,
    pub cell: u32,
    /// The marker's image DID (0 for none).
    pub image: u32,
    /// The marker's text.
    pub message: StringInfo,
}

/// The portal space (`PortalSpace::Effect`): the phase and the draw flags for the frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PortalFrame {
    /// `WorldRender::InPortalSpace`.
    pub in_portal_space: bool,
    /// `PortalSpace::Effect::m_State`.
    pub phase: ac2_render::portal::Phase,
    /// The draw flags: the world is drawn only while `draw.world` (the fades).
    pub draw: ac2_render::portal::Draw,
    /// A teleport has started and its destination is not yet in.
    pub teleporting: bool,
}

impl PortalFrame {
    /// `WorldRender::Render`'s gate: the world is drawn unless the player is in portal space and
    /// the effect does not draw it (then only the effect is drawn).
    pub fn world_drawn(&self) -> bool {
        !self.in_portal_space || self.draw.world
    }
}

/// Everything a renderer needs from one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientFrame {
    /// `Timer::cur_time` (server time).
    pub server_time: f64,
    /// `GameTime::present_time_of_day`.
    pub time_of_day: Option<f32>,
    /// The game's calendar at this frame (`GameTime::InqCurrentGameDateTimeString`'s variables).
    pub date_time: Option<ac2_world::GameDateTime>,
    /// The player's cell, when its body exists.
    pub cell_id: Option<u32>,
    /// `CPhysicsObj::IsInWorld` for the player.
    pub in_world: bool,
    /// `CellManager::IsBlockingForCells`: the player's physics waits for its cells.
    pub blocking_for_cells: bool,
    /// The portal space (the teleport's fade and tunnel).
    pub portal: PortalFrame,
    pub player: Option<PlayerFrame>,
    /// The camera, while the player is in the world.
    pub camera: Option<crate::scene::CameraFrame>,
    /// Placed remote objects (held or contained ones have no position and are left out).
    pub objects: Vec<ObjectFrame>,
    /// The resident landblocks (`xxyy0000`).
    pub blocks: Vec<u32>,
    /// The selected object (`gmSelectionUI`).
    pub selection: Option<crate::gameplay::SelectionFrame>,
    /// The player's vitals as the status bar shows them (`gmVitalsUI`).
    pub player_vitals: ac2_weenie::PlayerVitals,
    /// The player's inventory and open containers (`gmInventoryUI`, `gmPaperDollUI`,
    /// `gmExternalContainerUI`).
    pub inventory: crate::gameplay::InventoryFrame,
    /// Experience, skills, shortcuts, fellowship and allegiance (the advancement, shortcut and
    /// social panels).
    pub progress: crate::gameplay::ProgressFrame,
    /// The radar (`gmRadarUI`): the blips and the groove.
    pub radar: RadarFrame,
    /// The map notes (`gmNavigationUI`), in arrival order.
    pub markers: Vec<MapMarker>,
}
