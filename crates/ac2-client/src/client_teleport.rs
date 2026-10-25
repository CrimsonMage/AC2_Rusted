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
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::{Client, Notice};
use crate::frame::PortalFrame;

/// `gCWInterface::CallIFun_HandleTeleportStart`'s function id: no arguments, a zero return slot.
pub(crate) const IFUN_HANDLE_TELEPORT_START: u32 = 0x007F_001A;
/// `gCWInterface::CallIFun_HandleTeleportEnd`'s function id: `bSuccess`, a zero return slot.
pub(crate) const IFUN_HANDLE_TELEPORT_END: u32 = 0x007F_001B;

impl Client {
    /// `SmartBox::HandleTeleport`'s start, after the player's position is set and the body has
    /// left the world (see the module docs for the order).
    pub(crate) fn teleport_started(&mut self) {
        // CPhysicsObj::HandleTeleportStart: the motion interpreter's constraint ends.
        if let Some(body) = self.player.as_mut().and_then(|p| p.body.as_mut()) {
            body.physics.unconstrain();
        }
        self.teleporting = true;
        // ECM_Physics::SendNotice_PlayerTeleportStart -> gmGamePlayUI::RecvNotice_PlayerTeleportStart
        // -> SmartBox::EnterPortalSpace(0).
        self.notices.push(Notice::PlayerTeleportStart);
        self.enter_portal_space(false);
        self.teleport_ifun(IFUN_HANDLE_TELEPORT_START, &[]);
    }

    /// The teleport's end: the destination is in and the player is placed there.
    /// `CWInterface::HandleTeleportEnd(1)` runs its script, then `SmartBox::ExitPortalSpace`.
    pub(crate) fn teleport_ended(&mut self) {
        self.teleporting = false;
        self.notices.push(Notice::TeleportEnd);
        self.teleport_ifun(IFUN_HANDLE_TELEPORT_END, &[IFunArg::Int(1)]);
        self.exit_portal_space(false);
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

    /// `gmGamePlayUI::RecvNotice_PortalSpace(enter)`: the script's portal-space notices.
    pub(crate) fn portal_notices(&mut self, events: &[SessionEvent]) {
        for event in events {
            if let SessionEvent::Notice(GameplayNotice::PortalSpace(enter)) = event {
                if *enter {
                    self.enter_portal_space(false);
                } else {
                    self.exit_portal_space(false);
                }
            }
        }
    }

    /// The portal effect's state for the frame.
    pub(crate) fn portal_frame(&self) -> PortalFrame {
        PortalFrame {
            in_portal_space: self.portal.in_portal_space(),
            phase: self.portal.phase(),
            draw: *self.portal.draw(),
            teleporting: self.teleporting,
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
