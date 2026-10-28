//! The plain-data snapshot of one client frame, for the renderer.

use std::sync::Arc;

pub use ac2_physics::HoldPoint;
pub use ac2_ui::{MapMarker, RadarBlip, RadarFrame};

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
    pub pose: Vec<ac2_physics::PartPose>,
    /// The player's local `VisualDesc` (`CPhysicsObj::m_local_vdesc`), as for other objects.
    pub appearance: Arc<ac2_weenie::Appearance>,
}

/// An object other than the player: where its physics object is (in the client's world), or
/// else where the server last placed it, with what to draw it with.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectFrame {
    pub id: u64,
    /// The `VisualDesc` parent DID (its appearance) and the root Setup.
    pub visual: Option<u32>,
    pub setup: Option<u32>,
    pub name: Option<Arc<str>>,
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
    /// `CPhysicsObj::GetMode`: the mode the body's animation has reached (a `DoMode`'s transition
    /// ends in it); `None` without a body.
    pub animated_mode: Option<u32>,
    pub motion: [f32; 3],
    /// The object holding this one at a holding location (`CPhysicsObj::set_parent`: a wielded
    /// or carried item), drawn there rather than at `origin`.
    pub holder: Option<Holder>,
    /// The FX object's emitter and attach point, when the object hangs from one
    /// (`CPhysicsObj::FXSetParent`): drawn on the emitter's posed bone each frame.
    pub fx_follow: Option<FxFollow>,
    /// `gmWeenieDesc::m_containerID` and `m_wielderID` (qualities `0x102`, `0x105`): the weenie
    /// graph's container and wielder.
    pub container: Option<u64>,
    pub wielder: Option<u64>,
    /// `CPhysicsObj::m_local_vdesc`: the server's appearance on top of `visual`'s DAT chain
    /// (Setup, part-group, attachment and appearance-table overrides).
    pub appearance: Arc<ac2_weenie::Appearance>,
    /// The root Setup's bones relative to their parents as the animation left them, when its
    /// physics object exists.
    pub pose: Vec<ac2_physics::PartPose>,
    /// `PHYSICSOBJREP::Draw`'s colour scale ([`ac2_physics::PhysicsBody::draw_scale`]): the
    /// animation colour and opacity sliders, white until moved. The drawn tint multiplies it.
    pub anim_scale: [f32; 4],
    /// `CPhysicsObj::Draw`'s `_bUseAdaptiveDegrade`: a server object (not temporary) whose body
    /// is mobile and not static (`is_mobile`, `is_dynamic`).
    pub adaptive_degrade: bool,
}

pub use ac2_physics::FxFollow;

/// Where a held object hangs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Holder {
    pub id: u64,
    /// The holder's holding location (`PhysicsDesc` location / `RecvEvt_Parent`).
    pub location: u32,
    /// The placement frame the held object takes there (`SetPlacementFrame`).
    pub orientation: Option<u32>,
    /// The holder's Setup holding location (`CPartArray::GetHoldingLocation`): the root bone it
    /// rides and its transform. `None` while the holder's body is not loaded, so the object has
    /// no place to be drawn at.
    pub point: Option<HoldPoint>,
    /// `CHILDINFO::m_offset`: the child's frame in the placed frame. Identity for a holding
    /// location child; a stuck missile (`CPhysicsObj::StickySetParent`) keeps its rotation to
    /// the bone it hangs from.
    pub offset: ac2_math::Frame,
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
    /// The tunnel as drawn this frame (the rings and the camera), while the tunnel is drawn.
    pub tunnel: Option<ac2_render::portal::TunnelFrame>,
    /// `Render`'s camera, while the effect is drawn (the background, rings and clouds use it).
    pub camera: Option<ac2_render::portal::PortalCamera>,
    /// The clouds' particles `Render` queued this frame (empty when the tunnels are not drawn).
    pub clouds: ac2_render::particles::ParticleGeometry,
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
    pub markers: Arc<Vec<MapMarker>>,
    /// The objects an FX hid or tinted (`FXManager`'s hides and colours, and fades).
    pub fx_looks: ac2_smartbox::fx_objects::FxLooks,
}
