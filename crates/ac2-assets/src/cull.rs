#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecodedSurfaceCullMode {
    None,
    Clockwise,
    CounterClockwise,
}
