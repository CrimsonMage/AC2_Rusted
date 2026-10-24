#[derive(Debug, Clone, PartialEq)]
pub struct DecodedMeshVertex {
    pub position: ac2_math::Vec3,
    pub normal: ac2_math::Vec3,
    pub uv: ac2_math::Vec2,
}
