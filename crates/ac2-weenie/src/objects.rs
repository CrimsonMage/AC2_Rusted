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
    /// `CPhysicsObj::m_update_times`, by `StampID`: position, teleport, force position,
    /// move-to. `CPhysicsObj::set_description` copies them from the `PhysicsDesc`.
    pub update_times: [u16; 4],
}

/// `StampID` indices into [`RemoteObject::update_times`].
pub const STAMP_POSITION: usize = 0;
pub const STAMP_TELEPORT: usize = 1;
pub const STAMP_FORCE_POSITION: usize = 2;
pub const STAMP_MOVE_TO: usize = 3;

/// `CPhysicsObj::is_newer`: a larger stamp is newer unless it leads by more than `0x7FFF`; a
/// smaller one is newer (wrapped) when it trails by more than `0x7FFF`.
pub(crate) fn is_newer(old: u16, new: u16) -> bool {
    if new > old {
        new - old <= 0x7FFF
    } else {
        old - new > 0x7FFF
    }
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
            update_times: object.physics.timestamps,
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

    /// `CPhysicsObj::RecvEvt_Position`'s bookkeeping: only a newer position stamp with a cell
    /// moves the object (its orientation stays; the pack carries a heading). A newer
    /// force-position stamp is kept, then the position and teleport stamps.
    pub(crate) fn apply_position(&mut self, update: &ServerPosition) -> bool {
        let Some(object) = self.objects.get_mut(&update.sender.id) else {
            return false;
        };
        let times = &mut object.update_times;
        if !is_newer(times[STAMP_POSITION], update.position_stamp) || update.position.cell == 0 {
            return false;
        }
        if is_newer(times[STAMP_FORCE_POSITION], update.force_position_stamp) {
            times[STAMP_FORCE_POSITION] = update.force_position_stamp;
        }
        times[STAMP_POSITION] = update.position_stamp;
        times[STAMP_TELEPORT] = update.teleport_stamp;
        let rotation = object.position.map_or([1.0, 0.0, 0.0, 0.0], |p| p.rotation);
        object.position = Some(Position {
            cell: update.position.cell,
            origin: update.position.origin,
            rotation,
        });
        object.heading_degrees = Some(update.heading_degrees);
        object.motion = update.motion;
        true
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
            // `CPhysicsObj::RecvEvt_MoveTo`.
            ObjectAction::MoveTo { stamp, .. } => object.update_times[STAMP_MOVE_TO] = *stamp,
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

#[cfg(test)]
mod tests {
    use super::{is_newer, ObjectTable, RemoteObject};
    use ac2_net::protocol::{CellPosition, InstanceIdWithStamp, ServerPosition};

    fn object() -> RemoteObject {
        RemoteObject {
            id: 9,
            instance: 1,
            visual: None,
            setup: None,
            entity: None,
            name: None,
            position: None,
            heading_degrees: None,
            motion: [0.0; 3],
            mode: None,
            parent: None,
            update_times: [10, 0, 0, 0],
        }
    }

    fn update(position_stamp: u16, force: u16, teleport: u16, cell: u32) -> ServerPosition {
        ServerPosition {
            sender: InstanceIdWithStamp {
                id: 9,
                instance: 1,
                other: 0,
            },
            time: 0.0,
            position: CellPosition {
                cell,
                origin: [1.0, 2.0, 3.0],
            },
            heading_degrees: 0.0,
            motion: [0.0; 3],
            contact: false,
            position_stamp,
            force_position_stamp: force,
            teleport_stamp: teleport,
            jump: false,
            impulse: false,
            velocity: None,
        }
    }

    #[test]
    fn stamps_compare_by_half_range_wrap() {
        assert!(is_newer(10, 11) && is_newer(10, 10 + 0x7FFF));
        assert!(!is_newer(10, 10) && !is_newer(10, 9) && !is_newer(0, 0x8000));
        assert!(is_newer(0x8000, 0) && is_newer(0xFFFF, 3));
    }

    #[test]
    fn only_newer_positions_with_a_cell_move_the_object_and_set_its_stamps() {
        let mut table = ObjectTable::default();
        table.insert(object());
        assert!(!table.apply_position(&update(10, 1, 1, 0x0299_0109)));
        assert!(!table.apply_position(&update(11, 1, 1, 0)));
        assert!(table.apply_position(&update(11, 4, 2, 0x0299_0109)));
        assert_eq!(table.get(9).unwrap().update_times, [11, 2, 4, 0]);
        // An older force stamp is kept out; position and teleport always follow the update.
        assert!(table.apply_position(&update(12, 3, 1, 0x0299_0109)));
        assert_eq!(table.get(9).unwrap().update_times, [12, 1, 4, 0]);
    }
}
