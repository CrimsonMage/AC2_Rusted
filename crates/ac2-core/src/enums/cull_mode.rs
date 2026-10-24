/// Authored face culling of a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CullMode(pub i32);

#[allow(non_upper_case_globals)]
impl CullMode {
    pub const Landblock: Self = Self(0);
    pub const None: Self = Self(1);
    pub const Clockwise: Self = Self(2);
    pub const CounterClockwise: Self = Self(3);

    pub const LANDBLOCK: Self = Self::Landblock;
    pub const NONE: Self = Self::None;
    pub const CLOCKWISE: Self = Self::Clockwise;
    pub const COUNTER_CLOCKWISE: Self = Self::CounterClockwise;
}
