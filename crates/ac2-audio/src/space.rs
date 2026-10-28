//! The spatial questions the sound asks of the world. Positions belong to physics (`Position`,
//! its landblock offsets and frames) and the world owns the env cells, and this crate sits before
//! both in `ARCHITECTURE.md`: so the client implements [`SoundSpace`] over those types and the
//! sound code here names only plain vectors and the place type the implementor chose.

use crate::playing::Emitter;

/// What the ambient sound weighs of an env cell: its place, the cells it sees
/// (`CObjCell::m_stabList`) and whether it sees the outside (`CEnvCell::m_seenOutside`).
#[derive(Debug, Clone, PartialEq)]
pub struct EnvCellPlace<P> {
    pub cell_id: u32,
    pub place: P,
    pub stab_list: Vec<u32>,
    pub seen_outside: bool,
}

/// The world's answers for sound: a place is whatever the implementor uses for a position.
pub trait SoundSpace {
    type Place: Copy;

    /// `PositionOffset::GetOffset`: the vector from `from` to `to` in global axes.
    fn offset(&self, from: Self::Place, to: Self::Place) -> [f32; 3];

    /// `Position::distance_squared`: `(x*x + y*y) + z*z` of the offset, in f64.
    fn distance_squared(&self, from: Self::Place, to: Self::Place) -> f64 {
        let [x, y, z] = self.offset(from, to).map(f64::from);
        (x * x + y * y) + z * z
    }

    /// `Position::globaltolocalvec` in `listener`'s frame: a global-axes vector as the listener
    /// hears it.
    fn listener_local(&self, listener: Self::Place, global: [f32; 3]) -> [f32; 3];

    /// The landblock cell id the place stands in.
    fn cell_id(&self, place: Self::Place) -> u32;

    /// A place at `origin` in cell `cell_id` with no rotation (a landblock's ambient point).
    fn point(&self, cell_id: u32, origin: [f32; 3]) -> Self::Place;

    /// The env cell with this id, if the world has it loaded.
    fn env_cell(&self, cell_id: u32) -> Option<EnvCellPlace<Self::Place>>;
}

/// `SoundManager`'s emitter for an object: its offset and velocity from the listener, muted when
/// it is not in the world (`DirectXPlayingSound::UpdatePosition`, `CPhysicsObj::ShouldMuteSound`'s
/// last case).
pub fn emitter<S: SoundSpace>(
    space: &S,
    listener: S::Place,
    position: S::Place,
    velocity: [f32; 3],
    in_world: bool,
) -> Emitter {
    Emitter {
        offset: space.listener_local(listener, space.offset(listener, position)),
        velocity: space.listener_local(listener, velocity),
        muted: !in_world,
    }
}
