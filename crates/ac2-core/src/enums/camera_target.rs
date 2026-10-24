use bitflags::bitflags;

bitflags! {
    /// How a camera frames its target.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
    pub struct CameraTarget: u32 {
        const LOOK_IN_DIRECTION = 0x0000_0001;
        const LOOK_AT_OBJECT = 0x0000_0002;
        const LOOK_AT_PIVOT = 0x0000_0004;
        const ALIGN_WITH_PIVOT = 0x0000_0008;
        const ALIGN_WITH_PLANE = 0x0000_0010;
    }
}
