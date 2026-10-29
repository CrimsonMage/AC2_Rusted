//! Static objects (buildings, props, scenery, grass) through the retail material path: layer
//! selection and `RenderDeviceD3D::SetupState` per layer, the vertex program `fragment.dll` links
//! for it, and the outdoor sun and ambient it is lit by.

mod cell;
mod degrade;
#[cfg(test)]
mod fx_bone_tests;
mod lights;
mod mesh_model;
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

pub use ac2_assets::{drawn_groups, resolve_drawn_groups, total_visual_desc, DrawnGroup};
pub use cell::{bake_cell_colors, load_cell_model, CellLighting};
pub use degrade::{
    degrade_level, degrade_scale, mesh_degrade_factor, object_lod, part_array_lod, DegradeLists,
    DegradeOptions, DrawDetail, ObjectLod, PartDegrades, HIGH_DETAIL,
};
pub use lights::{ObjectEnvironment, ObjectLights, SunLight};
pub use mesh_model::{load_mesh_model, particle_material_model};
pub use model::{
    load_object_model, load_object_model_with, ObjectBatch, ObjectMaterial, ObjectModel,
    ObjectPart, ObjectPose, ObjectVertex, PoseRig,
};
pub use pass::{
    blend, layer_drawn, layer_pass, layer_pass_forced, texture_matrices, LayerPass, ProgramFlags,
    RenderSettings, StagePass, VertexProgram,
};
pub use pixel_shader::{
    parse_pixel_shader, PixelProgram, PsInstruction, PsOp, PsRegister, PsSource, PsSourceModifier,
};
pub use point_lights::{
    object_point_lights, static_point_light, PointLight, SceneLight, MAX_POINT_LIGHTS,
};
pub use skeleton::{blend_vertex, GroupPose, ObjectSkeleton, SkeletonGroup};
pub use water::load_water_model;
