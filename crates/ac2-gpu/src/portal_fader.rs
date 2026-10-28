//! The portal space's full-screen fader (`PortalSpace::Effect::Render`'s `DrawScreenQuad` of
//! `m_cFaderColor`, SRCALPHA over INVSRCALPHA), drawn over the world while the effect asks for it.

use ac2_render::portal::Draw;

use crate::{Gpu, ScreenQuad};

/// The fader's quad and the colour it draws this frame, if any.
pub struct PortalFader {
    quad: ScreenQuad,
    drawn: bool,
}

impl PortalFader {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        Self {
            quad: ScreenQuad::new(gpu, format),
            drawn: false,
        }
    }

    /// The frame's fader: `m_bDrawFader`, with its colour (the portal effect's `draw`).
    pub fn update(&mut self, gpu: &Gpu, draw: &Draw) {
        self.drawn = draw.fader;
        if draw.fader {
            self.quad.set_color(&gpu.queue, draw.fader_color);
        }
    }

    /// Draws the fader into `pass` when the frame has one.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.drawn {
            self.quad.draw(pass);
        }
    }
}
