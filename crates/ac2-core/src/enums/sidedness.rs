/// Side of a plane a point or sphere lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Sidedness(pub u32);

impl Sidedness {
    pub const POSITIVE: Self = Self(0);
    pub const NEGATIVE: Self = Self(1);
    pub const IN_PLANE: Self = Self(2);
    pub const CROSSING: Self = Self(3);
}
