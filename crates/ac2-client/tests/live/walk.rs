//! Walking and finding objects for the live tests: the held movement action of the rig, the
//! player's placement from the frame, and an object of the world by its entity DID.

use ac2_weenie::{ActiveAction, ACTION_CLASS_MOVEMENT};

use super::LiveRig;

/// The planar distance between two placements.
pub fn planar(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

impl LiveRig {
    /// Walks the way the keys allow until within `within` metres of `target` (planar). The body
    /// keeps the heading it entered with (west): forward goes west, a strafe north or south, so
    /// a target to the east cannot be reached. The distance after.
    pub fn walk_to(&mut self, target: [f32; 3], within: f32, seconds: f64) -> f32 {
        use ac2_weenie::{ACTION_FORWARD, ACTION_STRAFE_NEGATIVE, ACTION_STRAFE_POSITIVE};
        let mut run = 0.0;
        while run < seconds && planar(self.origin(), target) > within {
            let here = self.origin();
            let mut keys = Vec::new();
            if target[0] < here[0] - 0.3 {
                keys.push(ACTION_FORWARD);
            }
            if target[1] > here[1] + 0.3 {
                keys.push(ACTION_STRAFE_POSITIVE);
            } else if target[1] < here[1] - 0.3 {
                keys.push(ACTION_STRAFE_NEGATIVE);
            }
            if keys.is_empty() {
                break;
            }
            self.held = keys
                .into_iter()
                .map(|enum_value| ActiveAction {
                    class: ACTION_CLASS_MOVEMENT,
                    enum_value,
                    map_extent: 1.0,
                    extent: 1.0,
                })
                .collect();
            self.run(0.1);
            run += 0.1;
        }
        self.held.clear();
        self.run(0.5);
        planar(self.origin(), target)
    }

    /// Where the player's body stands.
    pub fn origin(&self) -> [f32; 3] {
        self.client
            .frame()
            .player
            .as_ref()
            .expect("the player's body")
            .origin
    }

    /// Where object `id` stands once it is in the world.
    pub fn place_of(&self, id: u64) -> Option<[f32; 3]> {
        self.client
            .frame()
            .objects
            .iter()
            .find(|o| o.id == id && o.in_world)
            .map(|o| o.origin)
    }

    /// The id of the session's object created from the entity DID `entity`.
    pub fn object_of(&self, entity: u32) -> Option<u64> {
        self.client
            .session()
            .objects()
            .iter()
            .find(|o| o.entity == Some(entity))
            .map(|o| o.id)
    }

    /// Holds the movement `actions` together until the body is within `within` metres of `target` (or `seconds` run
    /// out), then lets go and lets it settle. The planar distance after.
    pub fn walk_towards(
        &mut self,
        actions: &[u32],
        target: [f32; 3],
        within: f32,
        seconds: f64,
    ) -> f32 {
        self.held = actions
            .iter()
            .map(|&enum_value| ActiveAction {
                class: ACTION_CLASS_MOVEMENT,
                enum_value,
                map_extent: 1.0,
                extent: 1.0,
            })
            .collect();
        let mut run = 0.0;
        while run < seconds && planar(self.origin(), target) > within {
            self.run(0.05);
            run += 0.05;
        }
        self.held.clear();
        self.run(0.5);
        planar(self.origin(), target)
    }
}
