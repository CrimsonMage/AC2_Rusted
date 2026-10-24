/// Quarter-turn rotation of a placed landscape object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Rotation(pub u32);

impl Rotation {
    pub const ROT_0: Self = Self(0);
    pub const ROT_90: Self = Self(1);
    pub const ROT_180: Self = Self(2);
    pub const ROT_270: Self = Self(3);
}

/// Water coverage of a terrain cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct WaterType(pub u32);

impl WaterType {
    pub const NOT_WATER: Self = Self(0);
    pub const PARTIALLY_WATER: Self = Self(1);
    pub const ENTIRELY_WATER: Self = Self(2);
}
