use std::collections::BTreeMap;

use ac2_net::protocol::{CellPosition, CreateObject, ServerPosition};

/// One object the server has created and not destroyed.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteObject {
    pub id: u64,
    pub instance: u16,
    /// Parent `VisualDesc` DID: the object's appearance.
    pub visual: Option<u32>,
    pub entity: Option<u32>,
    /// Cell position and orientation (w, x, y, z); absent for contained or held objects.
    pub position: Option<(CellPosition, [f32; 4])>,
    /// Heading in degrees clockwise from north, from the latest movement update.
    pub heading_degrees: Option<f32>,
    /// Body-local intended motion from the latest movement update.
    pub motion: [f32; 3],
    /// Holder and location when the object is parented (a held item).
    pub parent: Option<(u64, u32)>,
}

/// Every live object, by id.
#[derive(Debug, Default)]
pub struct ObjectTable {
    objects: BTreeMap<u64, RemoteObject>,
}

impl ObjectTable {
    /// Adds or replaces an object from its `CreateObject`.
    pub fn create(&mut self, object: &CreateObject) {
        self.objects.insert(
            object.id,
            RemoteObject {
                id: object.id,
                instance: object.physics.instance_stamp,
                visual: object.visual.parent,
                entity: object.entity_did(),
                position: object.physics.position,
                heading_degrees: None,
                motion: [0.0; 3],
                parent: object
                    .physics
                    .parent
                    .map(|(holder, location, _)| (holder, location)),
            },
        );
    }

    /// Applies a movement update; `false` when the object is unknown or the update is stale.
    pub fn update(&mut self, update: &ServerPosition) -> bool {
        let Some(object) = self.objects.get_mut(&update.id) else {
            return false;
        };
        if update.instance != object.instance {
            return false;
        }
        let orientation = object.position.map_or([1.0, 0.0, 0.0, 0.0], |(_, q)| q);
        object.position = Some((update.position, orientation));
        object.heading_degrees = Some(update.heading_degrees);
        object.motion = update.motion;
        true
    }

    /// Removes an object; `false` when it was unknown or the instance does not match.
    pub fn destroy(&mut self, id: u64, instance: u16) -> bool {
        match self.objects.get(&id) {
            Some(object) if object.instance == instance => {
                self.objects.remove(&id);
                true
            }
            _ => false,
        }
    }

    pub fn get(&self, id: u64) -> Option<&RemoteObject> {
        self.objects.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RemoteObject> {
        self.objects.values()
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}
