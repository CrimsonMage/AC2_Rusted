//! The client's [`SoundSpace`]: physics' `Position` frames and offsets, and the terrain's env
//! cells, for the sound code in `ac2-audio`.

use ac2_audio::{EnvCellPlace, SoundSpace};
use ac2_physics::{CellSource, Frame, Position, Quaternion, Terrain, Vector};

/// The sound's view of the world: the loaded terrain (its env cells) and the physics places.
pub(crate) struct PhysicsSpace<'a> {
    pub(crate) terrain: &'a Terrain,
}

impl SoundSpace for PhysicsSpace<'_> {
    type Place = Position;

    fn offset(&self, from: Position, to: Position) -> [f32; 3] {
        let o = from.offset_to(&to);
        [o.x, o.y, o.z]
    }

    fn listener_local(&self, listener: Position, global: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = global;
        let v = listener.frame.global_to_local_vec(Vector::new(x, y, z));
        [v.x, v.y, v.z]
    }

    fn cell_id(&self, place: Position) -> u32 {
        place.cell_id
    }

    fn point(&self, cell_id: u32, origin: [f32; 3]) -> Position {
        let [x, y, z] = origin;
        Position::new(
            cell_id,
            Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY),
        )
    }

    fn env_cell(&self, cell_id: u32) -> Option<EnvCellPlace<Position>> {
        self.terrain.env_cell(cell_id).map(|env| EnvCellPlace {
            cell_id: env.cell_id,
            place: env.position,
            stab_list: env.stab_list.clone(),
            seen_outside: env.seen_outside,
        })
    }
}
