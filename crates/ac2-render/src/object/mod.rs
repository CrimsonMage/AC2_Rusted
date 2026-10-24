//! Static objects (buildings, props, scenery, grass) through the retail material path: layer
//! selection and `RenderDeviceD3D::SetupState` per layer, the vertex program `fragment.dll` links
//! for it, and the outdoor sun and ambient it is lit by.

mod lights;
mod model;
mod pass;
#[cfg(test)]
mod pass_tests;
mod texture_matrix;

pub use lights::{ObjectLights, SunLight};
pub use model::{
    load_object_model, ObjectBatch, ObjectMaterial, ObjectModel, ObjectPart, ObjectVertex,
};
pub use pass::{
    blend, layer_drawn, layer_pass, texture_matrices, LayerPass, ProgramFlags, RenderSettings,
    StagePass, VertexProgram,
};
