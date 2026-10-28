//! The portal space drawn over the frame (`PortalSpace::Effect::Render`): the full-screen fader
//! (`DrawScreenQuad` of `m_cFaderColor`, SRCALPHA over INVSRCALPHA) and the tunnel rings (the
//! outer, then the inner, textured with their materials from the DAT), as the effect's `Draw`
//! asks.
//!
//! The material textures are uploaded by [`PortalDraw::preload`] at setup, never on the frame
//! thread: a frame only uses what was uploaded.

use ac2_core::DataId;
use ac2_render::portal::{Draw, Tunnel, TunnelFrame};
use ac2_render::Image;

use crate::{Gpu, PortalTunnel, ScreenQuad, TextureCache, TextureKey};

/// The tunnel's field of view (the client's camera, 65 degrees; `Render`'s
/// `SetFOV(mFarDistance * m_fCameraFOVScale)` is not yet resolved).
const TUNNEL_FOV_Y: f32 = 65.0 * std::f32::consts::PI / 180.0;
const TUNNEL_NEAR: f32 = 1.0;
const TUNNEL_FAR: f32 = 2000.0;

/// The portal space's draws: the fader, and the tunnel with its textures.
pub struct PortalDraw {
    fader: ScreenQuad,
    fader_drawn: bool,
    /// `m_bFadeOnlyWorld`: the fader is drawn before the tunnels (the startup fade), else after.
    fader_first: bool,
    tunnel: PortalTunnel,
    tunnel_drawn: bool,
    /// The tunnel's material textures (inner, outer), uploaded by [`PortalDraw::preload`].
    materials: Option<(wgpu::TextureView, wgpu::TextureView)>,
}

impl PortalDraw {
    /// The draws on `format`; the tunnel's index buffer is the effect's, built from a fresh
    /// [`Tunnel`].
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let indices = Tunnel::new().index_buffer();
        Self {
            fader: ScreenQuad::new(gpu, format),
            fader_drawn: false,
            fader_first: true,
            tunnel: PortalTunnel::new(gpu, format, &indices),
            tunnel_drawn: false,
            materials: None,
        }
    }

    /// Uploads the tunnel's material textures (`m_pTunnelMaterial`, `m_pOuterTunnelMaterial`),
    /// decoded by `load`. Called at setup, not per frame.
    pub fn preload(
        &mut self,
        gpu: &Gpu,
        inner: DataId,
        outer: DataId,
        load: &mut dyn FnMut(DataId) -> Option<Image>,
    ) {
        let mut textures = TextureCache::new(&gpu.device, &gpu.queue);
        let mut load = |key: TextureKey| match key {
            TextureKey::Material(did) => load(did),
            _ => None,
        };
        let mut view = |did: DataId| {
            textures.get(
                &gpu.device,
                &gpu.queue,
                TextureKey::Material(did),
                &mut load,
            )
        };
        let inner = view(inner);
        let outer = view(outer);
        self.materials = Some((inner, outer));
    }

    /// The frame's portal state: the fader, and the tunnel's geometry and camera (`tunnel`, while
    /// the effect draws the tunnels) once the materials are uploaded (`aspect` for the projection).
    pub fn update(&mut self, gpu: &Gpu, draw: &Draw, tunnel: Option<&TunnelFrame>, aspect: f32) {
        self.fader_drawn = draw.fader;
        self.fader_first = draw.fade_only_world;
        if draw.fader {
            self.fader.set_color(&gpu.queue, draw.fader_color);
        }
        self.tunnel_drawn = false;
        let (Some(tunnel), Some((inner, outer))) = (tunnel, &self.materials) else {
            return;
        };
        let camera = tunnel
            .camera
            .to_render_camera(TUNNEL_FOV_Y, TUNNEL_NEAR, TUNNEL_FAR);
        let view_proj = camera.view_projection(aspect).to_cols_array();
        self.tunnel.update(
            gpu,
            &view_proj,
            &tunnel.inner,
            &tunnel.outer,
            [inner, outer],
        );
        self.tunnel_drawn = true;
    }

    /// Draws the portal space over the world, in `Render`'s order: the fader first when it covers
    /// only the world (`m_bFadeOnlyWorld`), then the tunnels, then the fader otherwise.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.fader_drawn && self.fader_first {
            self.fader.draw(pass);
        }
        if self.tunnel_drawn {
            self.tunnel.draw(pass);
        }
        if self.fader_drawn && !self.fader_first {
            self.fader.draw(pass);
        }
    }
}
