//! The previous frame's posed objects, kept one frame back for the motion vectors
//! (`object_motion.rs`). The scene hands over each frame's objects whole, so keeping the last
//! frame's list is the double buffer: an object that was not drawn on the previous frame has no
//! record (it just appeared, or came back from beyond its visible distance) and an object's record
//! is gone one frame after it stops being drawn, so the map never outgrows one frame's objects.

use std::collections::HashMap;

use ac2_render::scene::SceneObject;

use crate::object::Previous;

#[derive(Default)]
pub(crate) struct PreviousObjects {
    objects: Vec<SceneObject>,
    by_key: HashMap<u64, usize>,
}

impl PreviousObjects {
    /// Keeps `last` (the frame just finished) as the previous frame, dropping every older
    /// record. Kept empty while `motion` is off.
    pub(crate) fn advance(&mut self, last: Vec<SceneObject>, motion: bool) {
        self.by_key.clear();
        if !motion {
            self.objects.clear();
            return;
        }
        self.objects = last;
        for (index, object) in self.objects.iter().enumerate() {
            self.by_key.insert(object.key, index);
        }
    }

    /// Records held.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.by_key.len()
    }

    /// Where `key`'s object stood on the previous frame.
    pub(crate) fn of(&self, key: u64) -> Previous<'_> {
        match self.by_key.get(&key).map(|&index| &self.objects[index]) {
            Some(object) => Previous::Moved {
                world: object.world,
                pose: object.pose.as_ref(),
            },
            None => Previous::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use ac2_math::Mat4;

    use super::*;

    fn object(key: u64) -> SceneObject {
        SceneObject {
            key,
            world: Mat4::from_translation([key as f32, 0.0, 0.0].into()),
            pose: None,
            tint: [1.0; 4],
            point_lights: Vec::new(),
            detail: ac2_render::object::DrawDetail::FULL,
            cell: 0,
            hidden_mesh_types: Vec::new(),
        }
    }

    #[test]
    fn a_record_lives_one_frame_past_its_last_draw() {
        let mut previous = PreviousObjects::default();
        previous.advance(vec![object(1), object(2), object(3)], true);
        assert_eq!(previous.len(), 3);
        assert!(matches!(previous.of(2), Previous::Moved { .. }));
        // Frame two draws only object 2: the others are dropped with the frame before.
        previous.advance(vec![object(2)], true);
        assert_eq!(previous.len(), 1);
        assert!(matches!(previous.of(1), Previous::Unknown));
        assert!(matches!(previous.of(2), Previous::Moved { .. }));
        // And one that returns later starts without a record.
        previous.advance(vec![object(1)], true);
        assert!(matches!(previous.of(2), Previous::Unknown));
    }

    #[test]
    fn the_map_stays_the_size_of_one_frame_over_many_frames() {
        let mut previous = PreviousObjects::default();
        for frame in 0..500_u64 {
            previous.advance((frame * 10..frame * 10 + 10).map(object).collect(), true);
            assert_eq!(previous.len(), 10);
        }
    }

    #[test]
    fn nothing_is_kept_while_motion_is_off() {
        let mut previous = PreviousObjects::default();
        previous.advance(vec![object(1)], false);
        assert_eq!(previous.len(), 0);
        assert!(matches!(previous.of(1), Previous::Unknown));
    }
}
