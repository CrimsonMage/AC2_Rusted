//! The teleport flow (Legions `SmartBox::HandleTeleport` and what it starts) and the portal
//! space around it.
//!
//! A teleport is the server's position for the player with a newer teleport stamp
//! (`CPhysicsObj::RecvEvt_Position`, which calls the player's `PlayerPhysicsEventHandler` for
//! `SmartBox::HandleTeleport`). There is no separate teleport message in the Legions opcode table.
//! The start, in the order retail runs it:
//!
//! 1. `CPhysicsObj::HandleTeleportStart`: the body's own movement constraint ends.
//! 2. `SmartBox::EnterPortalSpace`, through `ECM_Physics::SendNotice_PlayerTeleportStart` and
//!    `gmGamePlayUI::RecvNotice_PlayerTeleportStart`: the portal effect starts.
//! 3. The script's `CWInterface::HandleTeleportStart` (`gCWInterface::CallIFun_HandleTeleportStart`,
//!    `0x7F001A`). The script also raises `CM_UI::SendNotice_PortalSpace`, which the UI routes to
//!    `gmGamePlayUI::RecvNotice_PortalSpace` (`GameplayNotice::PortalSpace`).
//!
//! The end, when the destination's cells are in and the player is placed there: the script's
//! `CWInterface::HandleTeleportEnd(1)` (`0x7F001B`), and the effect exits (`SmartBox::ExitPortalSpace`).
//! The frame drawn after that is acknowledged (`SceneRenderingComplete`) by the client's render step.

use ac2_script::host::IFunArg;
use ac2_ui::PortalSpaceCall;
use ac2_weenie::{GameplayNotice, SessionEvent, STAMP_TELEPORT};

use crate::client::{Client, Notice};
use crate::frame::PortalFrame;

/// `FileArray::GetDIDFromEnum`'s group `PortalSpace::Effect::Initialize` reads its DIDs from
/// (`0x40000002`).
const DID_GROUP: u32 = 0x4000_0002;
/// `Initialize`'s material enums: the inner (`m_pTunnelMaterial`) and the outer tunnel
/// (`m_pOuterTunnelMaterial`), `DBObj::GetByEnum(enum, 0x40000002, 0x40000032)`.
const INNER_MATERIAL_ENUM: u32 = 0x4000_0013;
const OUTER_MATERIAL_ENUM: u32 = 0x4000_0014;

/// The tunnel's material DIDs: the inner and the outer material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalMaterials {
    pub inner: u32,
    pub outer: u32,
}

/// Resolves `PortalSpace::Effect::Initialize`'s tunnel materials through the DAT's enum mapper
/// (0 where the DAT has none).
pub fn portal_materials(store: &ac2_db::ResourceStore) -> PortalMaterials {
    let did = |value: u32| {
        ac2_assets::ac2_enum_did(store, DID_GROUP, value)
            .ok()
            .flatten()
            .map_or(0, |id| id.raw())
    };
    PortalMaterials {
        inner: did(INNER_MATERIAL_ENUM),
        outer: did(OUTER_MATERIAL_ENUM),
    }
}

/// `gCWInterface::CallIFun_HandleTeleportStart`'s function id: no arguments, a zero return slot.
pub(crate) const IFUN_HANDLE_TELEPORT_START: u32 = 0x007F_001A;
/// `gCWInterface::CallIFun_HandleTeleportEnd`'s function id: `bSuccess`, a zero return slot.
pub(crate) const IFUN_HANDLE_TELEPORT_END: u32 = 0x007F_001B;

impl Client {
    /// `SmartBox::HandleTeleport`'s start, after the player's position is set and the body has
    /// left the world (see the module docs for the order).
    pub(crate) fn teleport_started(&mut self) {
        // CPhysicsObj::HandleTeleportStart: the motion interpreter stops its interpolation
        // (vtable 0x78, CMotionInterp::StopInterpolating) and its constraint ends (0x84,
        // CMotionInterp::UnConstrain). Its TargetManager calls (ClearTarget, NotifyVoyeurOfEvent
        // with Teleported_TargetStatus) have no port yet.
        if let Some(body) = self.player.as_mut().and_then(|p| p.body.as_mut()) {
            body.physics.stop_interpolating();
            body.physics.unconstrain();
        }
        // The vtable 0x53C call: CPhysicsObj::inc_update_time(StampID::TELEPORT = 1).
        self.session.inc_player_update_time(STAMP_TELEPORT);
        self.teleporting = true;
        // ECM_Physics::SendNotice_PlayerTeleportStart -> gmGamePlayUI::RecvNotice_PlayerTeleportStart,
        // whose gate enters the portal space.
        self.notices.push(Notice::PlayerTeleportStart);
        let call = self.story.player_teleport_start();
        self.portal_call(call);
        self.teleport_ifun(IFUN_HANDLE_TELEPORT_START, &[]);
    }

    /// The teleport's end: the destination is in and the player is placed there.
    /// `CWInterface::HandleTeleportEnd(1)` runs its script. The portal space does not exit here:
    /// retail's only `SmartBox::ExitPortalSpace` callers are the gameplay UI's notice handlers, so
    /// the exit comes from [`Client::player_ready_to_enter_world`] and the weenie's leave notice.
    pub(crate) fn teleport_ended(&mut self) {
        self.teleporting = false;
        self.notices.push(Notice::TeleportEnd);
        self.teleport_ifun(IFUN_HANDLE_TELEPORT_END, &[IFunArg::Int(1)]);
    }

    /// `gmGamePlayUI::RecvNotice_PlayerReadyToEnterWorld`: the gate runs, or a portal scene ends
    /// (its story events come back for the frame).
    pub(crate) fn player_ready_to_enter_world(&mut self) -> Vec<SessionEvent> {
        let (call, events) = self.story.player_ready_to_enter_world();
        self.portal_call(call);
        let mut out = Vec::new();
        self.story_events(events, &[], &mut out);
        out
    }

    /// `gmGamePlayUI::RecvNotice_LoginFinished`: `Init`, then `RecvNotice_PortalSpace(0)` (the
    /// vtable call at `+0x22C` on the notice handler, which is the portal notice).
    pub(crate) fn login_finished(&mut self) {
        let call = self.story.portal_space(false);
        self.portal_call(call);
    }

    /// The gate's call on the `SmartBox`, if the gate makes one.
    fn portal_call(&mut self, call: Option<PortalSpaceCall>) {
        match call {
            Some(PortalSpaceCall::Enter) => self.enter_portal_space(false),
            Some(PortalSpaceCall::Exit) => self.exit_portal_space(false),
            None => {}
        }
    }

    /// `PortalSpace::Effect::Initialize`'s cloud system (`PSManager::CreateParticleSystem` of PSDesc
    /// `0x40000015`), from the description read at setup (`ac2_render::portal::load_clouds_desc`).
    pub fn set_portal_clouds(
        &mut self,
        desc: Option<std::sync::Arc<ac2_assets::Ac2ParticleSystemDesc>>,
    ) {
        let now = self.session.local_time();
        self.portal.set_clouds(desc, now);
    }

    /// `SmartBox::EnterPortalSpace(bInstantly)` (`WorldRender::EnterPortalSpace`).
    pub(crate) fn enter_portal_space(&mut self, instantly: bool) {
        let now = self.session.local_time();
        self.portal.enter(instantly, now, &mut self.random);
    }

    /// `SmartBox::ExitPortalSpace(bInstantly)` (`WorldRender::ExitPortalSpace`).
    pub(crate) fn exit_portal_space(&mut self, instantly: bool) {
        let now = self.session.local_time();
        self.portal.exit(instantly, now);
    }

    /// `gmGamePlayUI::RecvNotice_PortalSpace(enter)`: the script's portal-space notices, through the
    /// gate.
    pub(crate) fn portal_notices(&mut self, events: &[SessionEvent]) {
        for event in events {
            if let SessionEvent::Notice(GameplayNotice::PortalSpace(enter)) = event {
                let call = self.story.portal_space(*enter);
                self.portal_call(call);
            }
        }
    }

    /// The portal effect's state for the frame.
    pub(crate) fn portal_frame(&self) -> PortalFrame {
        let drawn = self.portal.draw();
        PortalFrame {
            in_portal_space: self.portal.in_portal_space(),
            phase: self.portal.phase(),
            draw: *drawn,
            teleporting: self.teleporting,
            camera: self.portal.camera(),
            tunnel: drawn.tunnels.then(|| self.portal.tunnel_frame()),
            clouds: self.portal.cloud_geometry().clone(),
        }
    }

    /// A teleport IFun on the player's process (`CWInterface`'s script). Without a running
    /// process there is no script to run; its output is queued for the frame's events.
    fn teleport_ifun(&mut self, function: u32, args: &[IFunArg]) {
        let Some(player) = self.session.player() else {
            return;
        };
        let running = self
            .scripts
            .as_ref()
            .is_some_and(|s| s.running.contains(&player));
        if !running {
            return;
        }
        let mut out = Vec::new();
        if let Err(error) = self.ifun_into(function, args, &mut out) {
            if let Some(scripts) = self.scripts.as_mut() {
                scripts.problems.push(error);
            }
        }
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(out);
        }
    }
}
