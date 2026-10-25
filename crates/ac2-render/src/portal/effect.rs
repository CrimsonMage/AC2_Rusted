//! `PortalSpace::Effect` (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`):
//! the phase machine that fades the world out when a teleport starts, runs the tunnel while the
//! player is in transit, and fades back in once the destination is in. This module is the
//! timing and the draw flags (`Start`, `StartNow`, `Stop`, `StopNow`, `Think`); the tunnel
//! geometry is in `tunnel`.
//!
//! The timing is in client seconds (`Timer::local_time`, f64). The admin 7148 decompile carries
//! the fader's channels through a double's halves (`dTimeNow._0_4_`, `._4_4_`): the Shutdown
//! phases' fader is (0, 1, 1, alpha), the Startup phase 0's (0, 0, 0, alpha).

use ac2_core::Random;

/// `PortalSpace::Effect::StateType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Inactive,
    StartupPhase0,
    StartupPhase1,
    StartupPhase2,
    Active,
    ShutdownPhase0,
    ShutdownPhase1,
}

/// The draw flags and scales `Think` sets for the frame (`m_bDraw*`, `m_f*Scale`, the fader).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Draw {
    /// `m_bDrawWorld`: the world is drawn (fading in or out, at its own field of view).
    pub world: bool,
    /// `m_bDrawBackground`: the portal background (the sky tunnel) is drawn.
    pub background: bool,
    /// `m_bDrawTunnels`: the tunnel rings are drawn.
    pub tunnels: bool,
    /// `m_bDrawFader`: the full-screen fader quad is drawn.
    pub fader: bool,
    /// `m_bFadeOnlyWorld`: the fader covers only the world (drawn before the tunnels).
    pub fade_only_world: bool,
    /// `m_fWorldFOVScale`.
    pub world_fov_scale: f32,
    /// `m_fCameraFOVScale`.
    pub camera_fov_scale: f32,
    /// `m_fBackgroundOpacity`, in [0, 1].
    pub background_opacity: f32,
    /// `m_fTunnelVisibility`, in [0, 1].
    pub tunnel_visibility: f32,
    /// `m_cFaderColor` (r, g, b, a), each in [0, 1].
    pub fader_color: [f32; 4],
}

impl Default for Draw {
    /// The effect's constructor: nothing drawn, unit scales, black fader.
    fn default() -> Self {
        Self {
            world: false,
            background: false,
            tunnels: false,
            fader: false,
            fade_only_world: false,
            world_fov_scale: 1.0,
            camera_fov_scale: 1.0,
            background_opacity: 1.0,
            tunnel_visibility: 1.0,
            fader_color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// The phase durations in seconds: `Think` moves on once the phase has run longer than this.
const PHASE_SECONDS: f64 = 1.0;
/// The `dTimeNow` constant `Think` writes for the shutdown faders (the double 2^-7, whose high
/// word is 1.0f: the fader's g channel is 1, its r channel 0).
const SHUTDOWN_FADER: [f32; 3] = [0.0, 1.0, 1.0];

/// `PortalSpace::Effect`: the teleport's portal-space effect, without the tunnel geometry.
#[derive(Debug, Clone)]
pub struct PortalSpace {
    /// `m_bWantPortalSpace`: the effect is asked to run (`Start`) or to stop (`Stop`).
    want: bool,
    /// `m_State`.
    phase: Phase,
    /// `m_dStateStartTime`.
    phase_start: f64,
    /// `m_dLastThinkTime`.
    last_think: f64,
    /// `m_fRandomFactor`: the tunnel's noise offset, rolled when the effect starts.
    random_factor: f32,
    draw: Draw,
}

impl Default for PortalSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl PortalSpace {
    /// `PortalSpace::Effect::Effect`.
    pub fn new() -> Self {
        Self {
            want: false,
            phase: Phase::Inactive,
            phase_start: 0.0,
            last_think: 0.0,
            random_factor: 0.0,
            draw: Draw::default(),
        }
    }

    /// `WorldRender::EnterPortalSpace(instantly)`: starts the effect unless it is wanted already.
    pub fn enter(&mut self, instantly: bool, now: f64, random: &mut Random) {
        if !self.want {
            self.start(instantly, now, random);
        }
    }

    /// `WorldRender::ExitPortalSpace(instantly)`: stops the effect if it is wanted.
    pub fn exit(&mut self, instantly: bool, now: f64) {
        if self.want {
            self.stop(instantly, now);
        }
    }

    /// `WorldRender::InPortalSpace`: the effect is running or wanted.
    pub fn in_portal_space(&self) -> bool {
        self.phase != Phase::Inactive || self.want
    }

    /// `PortalSpace::Effect::m_bWantPortalSpace`.
    pub fn wanted(&self) -> bool {
        self.want
    }

    /// `PortalSpace::Effect::m_State`.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The draw flags and scales of the last `think`.
    pub fn draw(&self) -> &Draw {
        &self.draw
    }

    /// `PortalSpace::Effect::m_fRandomFactor`.
    pub fn random_factor(&self) -> f32 {
        self.random_factor
    }

    /// `PortalSpace::Effect::Start(bStartInstantly)`.
    fn start(&mut self, instantly: bool, now: f64, random: &mut Random) {
        self.want = true;
        if instantly {
            self.start_now(Phase::Active, now, random);
        }
    }

    /// `PortalSpace::Effect::Stop(bStopInstantly)`.
    fn stop(&mut self, instantly: bool, now: f64) {
        self.want = false;
        if instantly {
            self.stop_now(Phase::Inactive, now);
        }
    }

    /// `PortalSpace::Effect::StartNow(_State)`: the phase, its start time, and a fresh random
    /// factor (`Random::RollDice(0, 10000)`).
    fn start_now(&mut self, phase: Phase, now: f64, random: &mut Random) {
        self.phase_start = now;
        self.last_think = now;
        self.phase = phase;
        self.random_factor = random.roll_dice_f32(0.0, 10000.0);
    }

    /// `PortalSpace::Effect::StopNow(_State)`: the phase and its start time.
    fn stop_now(&mut self, phase: Phase, now: f64) {
        self.phase = phase;
        self.phase_start = now;
    }

    /// `PortalSpace::Effect::Think`: advances the phase on the clock, then computes this frame's
    /// draw flags and scales.
    pub fn think(&mut self, now: f64, random: &mut Random) {
        self.last_think = now;
        if self.phase == Phase::Inactive && self.want {
            self.start_now(Phase::StartupPhase0, now, random);
        }
        if self.phase == Phase::Active && !self.want {
            self.stop_now(Phase::ShutdownPhase0, now);
        }
        let elapsed = now - self.phase_start;
        match self.phase {
            Phase::StartupPhase0 | Phase::StartupPhase2 if elapsed > PHASE_SECONDS => {
                self.phase_start = now;
                self.phase = Phase::Active;
            }
            Phase::StartupPhase1 if elapsed > PHASE_SECONDS => {
                self.phase_start = now;
                self.phase = Phase::StartupPhase2;
            }
            Phase::ShutdownPhase0 if elapsed > PHASE_SECONDS => {
                self.phase_start = now;
                self.phase = Phase::ShutdownPhase1;
            }
            Phase::ShutdownPhase1 if elapsed > PHASE_SECONDS => {
                self.phase_start = now;
                self.phase = Phase::Inactive;
            }
            _ => {}
        }
        let elapsed = now - self.phase_start;
        self.draw = self.draw_for(elapsed);
    }

    /// The draw flags of `Think` for the phase at `elapsed` seconds into it.
    fn draw_for(&self, elapsed: f64) -> Draw {
        let mut d = Draw::default();
        match self.phase {
            Phase::StartupPhase0 => {
                d.world = true;
                d.background = true;
                d.tunnels = true;
                d.fade_only_world = true;
                // `(float)(1.0 - e*e*0.8)`, computed in extended precision.
                d.world_fov_scale = (1.0 - elapsed * elapsed * 0.8) as f32;
                d.background_opacity = elapsed as f32;
                d.tunnel_visibility = elapsed as f32;
                d.fader = true;
                d.fader_color = [0.0, 0.0, 0.0, elapsed as f32];
            }
            Phase::Active => {
                d.background = true;
                d.tunnels = true;
                d.tunnel_visibility = 1.0;
            }
            Phase::ShutdownPhase0 => {
                d.background = true;
                d.tunnels = true;
                d.tunnel_visibility = 1.0;
                d.camera_fov_scale = (elapsed * elapsed * 1.4 + 1.0) as f32;
                d.fader = true;
                d.fader_color = fader(elapsed);
            }
            Phase::ShutdownPhase1 => {
                let remaining = 1.0 - elapsed;
                d.world = true;
                d.world_fov_scale = (remaining * remaining * 1.4 + 1.0) as f32;
                d.fader = true;
                d.fader_color = fader(remaining);
            }
            Phase::Inactive | Phase::StartupPhase1 | Phase::StartupPhase2 => {}
        }
        // `RGBAColor::CapLow` / `CapHigh` and the [0, 1] caps on the visibility and opacity.
        if self.phase != Phase::Inactive {
            d.fader_color = d.fader_color.map(|c| c.clamp(0.0, 1.0));
            d.tunnel_visibility = d.tunnel_visibility.clamp(0.0, 1.0);
            d.background_opacity = d.background_opacity.clamp(0.0, 1.0);
        }
        d
    }
}

/// The shutdown fader: the constant channels with the alpha.
fn fader(alpha: f64) -> [f32; 4] {
    [
        SHUTDOWN_FADER[0],
        SHUTDOWN_FADER[1],
        SHUTDOWN_FADER[2],
        alpha as f32,
    ]
}
