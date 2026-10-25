//! Static objects (buildings, props, scenery, grass) through the retail material path: layer
//! selection and `RenderDeviceD3D::SetupState` per layer, the vertex program `fragment.dll` links
//! for it, and the outdoor sun and ambient it is lit by.

mod cell;
mod lights;
mod model;
mod pass;
#[cfg(test)]
mod pass_tests;
mod pixel_shader;
mod point_lights;
mod skeleton;
#[cfg(test)]
mod skeleton_tests;
mod texture_matrix;
mod water;

pub use cell::{bake_cell_colors, load_cell_model, CellLighting};
pub use lights::{ObjectEnvironment, ObjectLights, SunLight};
pub use model::{
    load_object_model, ObjectBatch, ObjectMaterial, ObjectModel, ObjectPart, ObjectPose,
    ObjectVertex,
};
pub use pass::{
    blend, layer_drawn, layer_pass, texture_matrices, LayerPass, ProgramFlags, RenderSettings,
    StagePass, VertexProgram,
};
pub use pixel_shader::{
    parse_pixel_shader, PixelProgram, PsInstruction, PsOp, PsRegister, PsSource, PsSourceModifier,
};
pub use point_lights::{object_point_lights, PointLight, SceneLight, MAX_POINT_LIGHTS};
pub use skeleton::{blend_vertex, GroupPose, ObjectSkeleton, SkeletonGroup};
pub use water::load_water_model;
