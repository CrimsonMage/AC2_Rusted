/// Decode family a cached resource is keyed under, carried as its `QualifiedDataId` type id.
///
/// AC2 records are addressed by their full data id, so AC2 decoders key under `Unknown` (type id
/// zero); raw landblock height records key under `LandBlock`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u32)]
pub enum ResourceTypeId {
    #[default]
    Unknown = 0,
    LandBlock = 1,
    /// Distinct families for the store's own tests.
    #[cfg(test)]
    FixtureA = 0x7F00_0001,
    #[cfg(test)]
    FixtureB = 0x7F00_0002,
    #[cfg(test)]
    FixtureC = 0x7F00_0003,
    #[cfg(test)]
    FixtureD = 0x7F00_0004,
}

impl ResourceTypeId {
    pub const fn from_type_id(type_id: u32) -> Option<Self> {
        match type_id {
            0 => Some(Self::Unknown),
            1 => Some(Self::LandBlock),
            #[cfg(test)]
            0x7F00_0001 => Some(Self::FixtureA),
            #[cfg(test)]
            0x7F00_0002 => Some(Self::FixtureB),
            #[cfg(test)]
            0x7F00_0003 => Some(Self::FixtureC),
            #[cfg(test)]
            0x7F00_0004 => Some(Self::FixtureD),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceTypeId;

    #[test]
    fn type_ids_round_trip() {
        for family in [
            ResourceTypeId::Unknown,
            ResourceTypeId::LandBlock,
            ResourceTypeId::FixtureA,
            ResourceTypeId::FixtureD,
        ] {
            assert_eq!(ResourceTypeId::from_type_id(family as u32), Some(family));
        }
        assert_eq!(ResourceTypeId::from_type_id(0x1FFF_FFFF), None);
    }
}
