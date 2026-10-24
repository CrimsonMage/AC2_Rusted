use bitflags::bitflags;

bitflags! {
    /// Per-object physics state bits.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PhysicsState: u32 {
        const STATIC = 0x0000_0001;
        const ETHEREAL = 0x0000_0004;
        const REPORT_COLLISIONS = 0x0000_0008;
        const IGNORE_COLLISIONS = 0x0000_0010;
        const NO_DRAW = 0x0000_0020;
        const MISSILE = 0x0000_0040;
        const PUSHABLE = 0x0000_0080;
        const ALIGN_PATH = 0x0000_0100;
        const PATH_CLIPPED = 0x0000_0200;
        const GRAVITY = 0x0000_0400;
        const LIGHTING_ON = 0x0000_0800;
        const PARTICLE_EMITTER = 0x0000_1000;
        const HIDDEN = 0x0000_4000;
        const SCRIPTED_COLLISION = 0x0000_8000;
        const HAS_PHYSICS_BSP = 0x0001_0000;
        const INELASTIC = 0x0002_0000;
        const HAS_DEFAULT_ANIM = 0x0004_0000;
        const HAS_DEFAULT_SCRIPT = 0x0008_0000;
        const CLOAKED = 0x0010_0000;
        const REPORT_COLLISIONS_AS_ENVIRONMENT = 0x0020_0000;
        const EDGE_SLIDE = 0x0040_0000;
        const SLEDDING = 0x0080_0000;
        const FROZEN = 0x0100_0000;
    }
}
