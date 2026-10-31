//! The texture filtering of the world's samplers, from the `TrilinearFiltering` and
//! `AnisotropicFiltering` preferences.

/// `capsD3D.MaxAnisotropy` as the client asks for it: the highest level wgpu allows.
pub const MAX_ANISOTROPY: u16 = 16;

/// The mip filter and anisotropy the stage and terrain samplers are made with.
///
/// `RenderDeviceD3D::SetStageFilterMode` makes the mip filter LINEAR only while
/// `Render::m_Prefs.TrilinearFiltering` is on (else POINT), and turns a LINEAR min or mag filter
/// into ANISOTROPIC while `AnisotropicFiltering` is on and the caps allow it, at
/// `D3DTSS_MAXANISOTROPY` = `capsD3D.MaxAnisotropy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureFiltering {
    /// `TrilinearFiltering`: the mip filter is linear (else nearest).
    pub trilinear: bool,
    /// The anisotropy clamp for stages whose min and mag filters are linear: 1 is off.
    pub anisotropy: u16,
}

impl Default for TextureFiltering {
    /// The registered defaults: trilinear on, anisotropic off.
    fn default() -> Self {
        Self {
            trilinear: true,
            anisotropy: 1,
        }
    }
}

impl TextureFiltering {
    /// The filtering for the preferences on an adapter with `downlevel` capabilities. Without
    /// `ANISOTROPIC_FILTERING` the clamp stays 1, as retail's caps check left it.
    pub fn new(trilinear: bool, anisotropic: bool, downlevel: wgpu::DownlevelFlags) -> Self {
        let supported = downlevel.contains(wgpu::DownlevelFlags::ANISOTROPIC_FILTERING);
        Self {
            trilinear,
            anisotropy: if anisotropic && supported {
                MAX_ANISOTROPY
            } else {
                1
            },
        }
    }

    /// [`TextureFiltering::new`] for `gpu`'s adapter.
    pub fn for_gpu(gpu: &crate::Gpu, trilinear: bool, anisotropic: bool) -> Self {
        Self::new(
            trilinear,
            anisotropic,
            gpu.adapter.get_downlevel_capabilities().flags,
        )
    }

    /// The mip filter and anisotropy clamp of a sampler whose min and mag filters are
    /// `min_linear` and `mag_linear`. wgpu allows a clamp above 1 only with min, mag and mip all
    /// linear, so an anisotropic stage takes a linear mip even when trilinear is off (retail's
    /// POINT mip filter stays on the stages that are not anisotropic).
    pub fn for_stage(&self, min_linear: bool, mag_linear: bool) -> (wgpu::MipmapFilterMode, u16) {
        if self.anisotropy > 1 && min_linear && mag_linear {
            (wgpu::MipmapFilterMode::Linear, self.anisotropy)
        } else if self.trilinear {
            (wgpu::MipmapFilterMode::Linear, 1)
        } else {
            (wgpu::MipmapFilterMode::Nearest, 1)
        }
    }
}

#[cfg(test)]
#[path = "filtering_tests.rs"]
mod tests;
