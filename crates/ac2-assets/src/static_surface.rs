/// Fixed-function blend intent of one decoded surface batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedSurfaceBlendMode {
    Opaque,
    /// `ONE, INVSRCALPHA` plus alpha test.
    ClipMap,
    Alpha,
    AlphaAdditive,
    InverseAlpha,
    InverseAlphaAdditive,
    Additive,
}

/// Submission lane of one decoded surface batch: drawn immediately, after clipped geometry, or
/// with the sorted alpha pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DecodedConstructedSurfaceDelay {
    #[default]
    Immediate,
    Clip,
    Alpha,
}

/// Surface material inputs needed by static-model consumers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecodedStaticSurfaceMaterial {
    /// Diffuse ARGB baked into the batch's vertices.
    pub mesh_diffuse: [f32; 4],
    pub base_color: [f32; 4],
    pub translucency: f32,
    pub luminosity: f32,
    pub diffuse: f32,
    pub blend_mode: DecodedSurfaceBlendMode,
    pub alpha_test: bool,
    /// Whether the batch draws as part of built (merged) building and interior meshes.
    pub draws_in_built_mesh: bool,
}

impl Default for DecodedStaticSurfaceMaterial {
    fn default() -> Self {
        Self {
            mesh_diffuse: [1.0; 4],
            base_color: [1.0; 4],
            translucency: 0.0,
            luminosity: 0.0,
            diffuse: 1.0,
            blend_mode: DecodedSurfaceBlendMode::Opaque,
            alpha_test: false,
            draws_in_built_mesh: false,
        }
    }
}
