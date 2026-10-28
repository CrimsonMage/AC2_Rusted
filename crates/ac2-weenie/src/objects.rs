//! The objects the server has created and not destroyed, as plain data for the client.

use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_net::protocol::{
    CreateObject, ObjectAction, Position, Qualities, QualityUpdate, ServerPosition,
};

use crate::qualities::WeenieQualities;
use ac2_physics::Appearance;

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
    /// `gmWeenieDesc::m_packageID` as created: the script package `CWDistributor::Construct`
    /// starts the object's process from (a later player description may drop the description).
    pub script_package: Option<u32>,
    /// The display name, when the weenie description carries a literal.
    pub name: Option<Arc<str>>,
    /// Cell position and orientation; absent for contained or held objects.
    pub position: Option<Position>,
    /// Heading in degrees, from the latest position update.
    pub heading_degrees: Option<f32>,
    /// The motion interpreter's do-motion values from the latest position update.
    pub motion: [f32; 3],
    pub mode: Option<u32>,
    /// `PhysicsDesc::m_behaviors`: the behaviours the object starts with.
    pub behaviors: Vec<ac2_net::protocol::BehaviorParams>,
    /// `PhysicsDesc::m_sliders`: slider, desired value and velocity (`SliderData`).
    pub sliders: Vec<(u32, f32, f32)>,
    /// Holder and location when the object is parented (a held item).
    pub parent: Option<(u64, u32)>,
    /// `PhysicsDesc::orientation_id` / `RecvEvt_Parent`'s orientation: the placement frame the
    /// held object takes at the holding location (`CPhysicsObj::SetPlacementFrame`).
    pub orientation: Option<u32>,
    /// `CPhysicsObj::RecvEvt_Contain`: the object went into the player's inventory (out of the
    /// world, not to be destroyed).
    pub contained: bool,
    /// `CPhysicsObj::m_local_vdesc`: what the server says the object looks like.
    pub appearance: Arc<Appearance>,
    /// `CPhysicsObj::m_update_times`, by `StampID`: position, teleport, force position,
    /// move-to. `CPhysicsObj::set_description` copies them from the `PhysicsDesc`.
    pub update_times: [u16; 4],
    /// `CWRepository::m_qualities`.
    pub qualities: WeenieQualities,
}

/// What an accepted position update did to the stamps (`CPhysicsObj::RecvEvt_Position`'s
/// player branch acts on these).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionApplied {
    /// The force-position stamp was newer.
    pub forced: bool,
    /// The teleport stamp was newer.
    pub teleported: bool,
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

/// The weenie type quality (`CWInterface`'s int 0x14).
const WEENIE_TYPE: u32 = 0x14;
/// `CWInterface::IsPlayer`: the weenie type's low five bits are all set.
const PLAYER_TYPE: i32 = 0x1F;

impl RemoteObject {
    /// `CWInterface::IsPlayer`: the object's weenie type (int quality 0x14) has all of the
    /// player bits (`& 0x1F`), as the retail check reads it.
    pub fn is_player(&self) -> bool {
        self.qualities
            .ints
            .get(&WEENIE_TYPE)
            .is_some_and(|weenie_type| weenie_type & PLAYER_TYPE == PLAYER_TYPE)
    }

    pub(crate) fn from_create(object: &CreateObject) -> Self {
        Self {
            id: object.id,
            instance: object.physics.instance_stamp,
            visual: object.visual.parent,
            setup: object.visual.root_setup(),
            entity: object.entity_did(),
            script_package: object.weenie.package_id,
            name: object
                .weenie
                .name
                .as_ref()
                .and_then(|n| n.literal.clone())
                .map(Arc::from),
            position: object.physics.position,
            heading_degrees: None,
            motion: [0.0; 3],
            mode: object.physics.mode,
            behaviors: object.physics.behaviors.clone(),
            sliders: object.physics.sliders.clone(),
            parent: object
                .physics
                .parent
                .map(|(holder, location, _)| (holder, location)),
            orientation: object.physics.orientation_id,
            contained: false,
            appearance: Arc::new(Appearance::from_delta(&object.visual)),
            update_times: object.physics.timestamps,
            qualities: WeenieQualities::from_weenie_desc(object.weenie.clone()),
        }
    }
}

/// Every live object, by id.
#[derive(Debug, Default)]
pub struct ObjectTable {
    objects: BTreeMap<u64, RemoteObject>,
    /// `CPhysicsObj::m_pParent` and `m_pChildren` of every object (the physics child lists).
    links: ac2_physics::ChildGraph,
}

impl ObjectTable {
    pub(crate) fn insert(&mut self, object: RemoteObject) {
        let id = object.id;
        self.links.unset_parent(id);
        if let Some((holder, location)) = object.parent {
            self.links.set_parent(id, holder, location, true);
        }
        self.objects.insert(id, object);
    }

    /// `CPhysicsObj::~CPhysicsObj`'s unparenting: the object leaves its parent and releases its
    /// children.
    pub(crate) fn remove(&mut self, id: u64) {
        self.links.unparent_children(id);
        self.links.unset_parent(id);
        self.objects.remove(&id);
    }

    /// The physics child lists (`CPhysicsObj::m_pParent`, `m_pChildren`).
    pub fn links(&self) -> &ac2_physics::ChildGraph {
        &self.links
    }

    /// `CPhysicsObj::RecvEvt_Position`'s bookkeeping: only a newer position stamp with a cell
    /// moves the object (its orientation stays; the pack carries a heading). A newer
    /// force-position stamp is kept, then the position and teleport stamps.
    pub(crate) fn apply_position(&mut self, update: &ServerPosition) -> Option<PositionApplied> {
        let object = self.objects.get_mut(&update.sender.id)?;
        let times = &mut object.update_times;
        if !is_newer(times[STAMP_POSITION], update.position_stamp) || update.position.cell == 0 {
            return None;
        }
        let forced = is_newer(times[STAMP_FORCE_POSITION], update.force_position_stamp);
        if forced {
            times[STAMP_FORCE_POSITION] = update.force_position_stamp;
        }
        let teleported = is_newer(times[STAMP_TELEPORT], update.teleport_stamp);
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
        object.contained = false;
        Some(PositionApplied { forced, teleported })
    }

    /// `CPhysicsObj::is_newer_then_update`: a newer stamp is taken and reported.
    fn newer_then_update(object: &mut RemoteObject, stamp: usize, value: u16) -> bool {
        if !is_newer(object.update_times[stamp], value) {
            return false;
        }
        object.update_times[stamp] = value;
        true
    }

    /// The bookkeeping of the per-object physics events (`CPhysicsObj::RecvEvt_*`). Returns the
    /// object that left the world, if one did (`RecvEvt_LeaveWorld`, `RecvEvt_DeParent`).
    pub(crate) fn apply_action(
        &mut self,
        id: u64,
        action: &ObjectAction,
        is_player: bool,
    ) -> Option<u64> {
        // `CPhysicsObj::RecvEvt_DeParent`: the child named (at its instance), on a newer
        // position stamp, is unparented and leaves the world.
        if let ObjectAction::DeParent(child) = action {
            let object = self.objects.get_mut(&child.id)?;
            if object.instance != child.instance
                || !Self::newer_then_update(object, STAMP_POSITION, child.other)
            {
                return None;
            }
            object.parent = None;
            object.position = None;
            self.links.unset_parent(child.id);
            return Some(child.id);
        }
        let object = self.objects.get_mut(&id)?;
        match action {
            ObjectAction::SetMode(mode) => object.mode = Some(*mode),
            // `CPhysicsObj::RecvEvt_Parent`: on a newer position stamp.
            ObjectAction::Parent {
                parent,
                location,
                orientation,
            } => {
                if Self::newer_then_update(object, STAMP_POSITION, parent.other) {
                    object.parent = Some((parent.id, *location));
                    object.orientation = Some(*orientation);
                    object.position = None;
                    object.contained = false;
                    // `CPhysicsObj::set_parent`: the holder's child list takes the object (the
                    // holder's Setup is not asked: its body may still be loading).
                    self.links.set_parent(id, parent.id, *location, true);
                }
            }
            // `CWPhysicsObject::RecvEvt_UpdateVisualDesc`: `SetLocalVisualDesc(delta)` replaces
            // the local description.
            ObjectAction::UpdateVisualDesc(delta) => {
                object.appearance = Arc::new(Appearance::from_delta(delta));
                object.visual = (object.appearance.parent != 0).then_some(object.appearance.parent);
                object.setup = delta.root_setup();
            }
            // `CPhysicsObj::RecvEvt_LeaveWorld`: an object other than the player, on a newer
            // position stamp, is unparented and leaves the world.
            ObjectAction::LeaveWorld(stamp) => {
                if !is_player && Self::newer_then_update(object, STAMP_POSITION, *stamp) {
                    object.parent = None;
                    object.position = None;
                    self.links.unset_parent(id);
                    return Some(id);
                }
            }
            // `CPhysicsObj::RecvEvt_MoveTo` acts on the player only.
            ObjectAction::MoveTo { stamp, .. } if is_player => {
                object.update_times[STAMP_MOVE_TO] = *stamp;
            }
            _ => {}
        }
        None
    }

    /// `CPhysicsObj::RecvEvt_Contain` on the player: the child (at its instance), on a newer
    /// position stamp, is unparented and leaves the world into the player's inventory.
    pub(crate) fn apply_contain(&mut self, child: &ac2_net::protocol::InstanceIdWithStamp) -> bool {
        let Some(object) = self.objects.get_mut(&child.id) else {
            return false;
        };
        if object.instance != child.instance
            || !Self::newer_then_update(object, STAMP_POSITION, child.other)
        {
            return false;
        }
        object.parent = None;
        object.position = None;
        object.contained = true;
        self.links.unset_parent(child.id);
        true
    }

    /// The objects held by `id` (`CPhysicsObj::m_children`).
    pub(crate) fn children(&self, id: u64) -> Vec<u64> {
        self.links.children_of(id).map(|c| c.id).collect()
    }

    /// `CWRepository::SetQualities`: the player description replaces the object's qualities.
    pub(crate) fn set_qualities(&mut self, id: u64, qualities: &Qualities) -> bool {
        let Some(object) = self.objects.get_mut(&id) else {
            return false;
        };
        object.qualities = WeenieQualities::from_unpacked(qualities);
        true
    }

    /// `CWRepository::RecvEvt_Update*_Private` on object `id`.
    pub(crate) fn apply_quality(&mut self, id: u64, update: &QualityUpdate) -> bool {
        let Some(object) = self.objects.get_mut(&id) else {
            return false;
        };
        object.qualities.set(update.quality, &update.value);
        true
    }

    pub fn get(&self, id: u64) -> Option<&RemoteObject> {
        self.objects.get(&id)
    }

    /// `CPhysicsObj::inc_update_time(stamp)`: the object's `ushort` stamp moves on by one.
    pub(crate) fn inc_update_time(&mut self, id: u64, stamp: usize) {
        if let Some(object) = self.objects.get_mut(&id) {
            object.update_times[stamp] = object.update_times[stamp].wrapping_add(1);
        }
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
        self.links = Default::default();
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
            script_package: None,
            name: None,
            position: None,
            heading_degrees: None,
            motion: [0.0; 3],
            mode: None,
            behaviors: Vec::new(),
            sliders: Vec::new(),
            parent: None,
            orientation: None,
            contained: false,
            appearance: Default::default(),
            update_times: [10, 0, 0, 0],
            qualities: Default::default(),
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
    fn a_player_is_a_weenie_type_with_all_the_player_bits() {
        let typed = |weenie_type: Option<i32>| {
            let mut object = object();
            if let Some(value) = weenie_type {
                object.qualities.ints.insert(0x14, value);
            }
            object.is_player()
        };
        assert!(typed(Some(0x1F)));
        assert!(typed(Some(0x3F)));
        // A creature (0x4F) is not a player; neither is an object without a weenie type.
        assert!(!typed(Some(0x4F)));
        assert!(!typed(Some(0x0F)));
        assert!(!typed(None));
    }

    #[test]
    fn a_parented_object_is_in_its_holders_child_list_until_it_leaves() {
        use ac2_net::protocol::ObjectAction;
        let mut table = ObjectTable::default();
        table.insert(object());
        let parent = InstanceIdWithStamp {
            id: 5,
            instance: 1,
            other: 11,
        };
        let action = ObjectAction::Parent {
            parent,
            location: 3,
            orientation: 0,
        };
        table.apply_action(9, &action, false);
        assert_eq!(table.links().parent_of(9), Some(5));
        assert_eq!(table.children(5), vec![9]);
        assert_eq!(table.links().info_of(9).map(|c| c.location), Some(3));
        // The same stamp is not newer: nothing changes, and a leave-world unparents.
        table.apply_action(9, &ObjectAction::LeaveWorld(12), false);
        assert!(!table.links().has_parent(9) && table.children(5).is_empty());
        table.apply_action(9, &action, false);
        table.remove(5);
        assert!(
            !table.links().has_parent(9),
            "a removed holder releases its children"
        );
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
        assert!(table
            .apply_position(&update(10, 1, 1, 0x0299_0109))
            .is_none());
        assert!(table.apply_position(&update(11, 1, 1, 0)).is_none());
        assert_eq!(
            table.apply_position(&update(11, 4, 2, 0x0299_0109)),
            Some(super::PositionApplied {
                forced: true,
                teleported: true
            })
        );
        assert_eq!(table.get(9).unwrap().update_times, [11, 2, 4, 0]);
        // An older force stamp is kept out; position and teleport always follow the update.
        assert!(table
            .apply_position(&update(12, 3, 1, 0x0299_0109))
            .is_some());
        assert_eq!(table.get(9).unwrap().update_times, [12, 1, 4, 0]);
    }
}
