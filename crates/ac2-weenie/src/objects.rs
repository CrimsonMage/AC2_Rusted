//! The objects the server has created and not destroyed, as plain data for the client.

use std::collections::BTreeMap;

use ac2_net::protocol::{CreateObject, ObjectAction, Position, ServerPosition};

/// One object the server has created and not destroyed.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteObject {
    pub id: u64,
    pub instance: u16,
    /// `VisualDesc` parent DID: the object's appearance.
    pub visual: Option<u32>,
    /// The root part group's Setup.
    pub setup: Option<u32>,
    pub entity: Option<u32>,
    /// The display name, when the weenie description carries a literal.
    pub name: Option<String>,
    /// Cell position and orientation; absent for contained or held objects.
    pub position: Option<Position>,
    /// Heading in degrees, from the latest position update.
    pub heading_degrees: Option<f32>,
    /// The motion interpreter's do-motion values from the latest position update.
    pub motion: [f32; 3],
    pub mode: Option<u32>,
    /// Holder and location when the object is parented (a held item).
    pub parent: Option<(u64, u32)>,
    /// `PositionPack` stamps of the latest applied update: position, forced, teleport.
    pub stamps: [u16; 3],
}

impl RemoteObject {
    pub(crate) fn from_create(object: &CreateObject) -> Self {
        Self {
            id: object.id,
            instance: object.physics.instance_stamp,
            visual: object.visual.parent,
            setup: object.visual.root_setup(),
            entity: object.entity_did(),
            name: object.weenie.name.as_ref().and_then(|n| n.literal.clone()),
            position: object.physics.position,
            heading_degrees: None,
            motion: [0.0; 3],
            mode: object.physics.mode,
            parent: object
                .physics
                .parent
                .map(|(holder, location, _)| (holder, location)),
            stamps: [0; 3],
        }
    }
}

/// Every live object, by id.
#[derive(Debug, Default)]
pub struct ObjectTable {
    objects: BTreeMap<u64, RemoteObject>,
}

impl ObjectTable {
    pub(crate) fn insert(&mut self, object: RemoteObject) {
        self.objects.insert(object.id, object);
    }

    pub(crate) fn remove(&mut self, id: u64) {
        self.objects.remove(&id);
    }

    /// A position update moves the object; its orientation stays (the pack carries a heading).
    pub(crate) fn apply_position(&mut self, update: &ServerPosition) {
        let Some(object) = self.objects.get_mut(&update.sender.id) else {
            return;
        };
        let rotation = object.position.map_or([1.0, 0.0, 0.0, 0.0], |p| p.rotation);
        object.position = Some(Position {
            cell: update.position.cell,
            origin: update.position.origin,
            rotation,
        });
        object.heading_degrees = Some(update.heading_degrees);
        object.motion = update.motion;
        object.stamps = [
            update.position_stamp,
            update.force_position_stamp,
            update.teleport_stamp,
        ];
    }

    pub(crate) fn apply_action(&mut self, id: u64, action: &ObjectAction) {
        let Some(object) = self.objects.get_mut(&id) else {
            return;
        };
        match action {
            ObjectAction::SetMode(mode) => object.mode = Some(*mode),
            ObjectAction::Parent {
                parent, location, ..
            } => {
                object.parent = Some((parent.id, *location));
                object.position = None;
            }
            ObjectAction::DeParent(_) => object.parent = None,
            ObjectAction::LeaveWorld(_) => object.position = None,
            _ => {}
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
