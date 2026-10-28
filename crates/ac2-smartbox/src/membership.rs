//! Which objects stand in which cell, and who watches them: `CObjCell::m_objectList`, its
//! voyeur table (`DetectionVoyeur`, keyed by the watcher's `InstanceID`) and the landblock's list
//! of dynamic objects (`CLandBlock::m_dynamic_objects`).
//!
//! A voyeur is a physics object whose detection volume covers the cell
//! (`CObjCell::AddVoyeur`); every object that enters, leaves, hides or shows in the cell tells
//! each voyeur (`CObjCell::UpdateAllVoyeur` -> `SendObjectDetection`, which ends in the
//! watcher's `CPhysicsObj::ReceiveDetectionUpdate`). The detection manager that consumes the
//! reports is not ported, so the reports queue as [`DetectionNotice`]s for it to drain.

use std::collections::HashMap;

/// `DetectionStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionStatus {
    /// `EnteredDetection`.
    Entered,
    /// `LeftDetection`.
    Left,
}

/// What `CObjCell` reads about an object through `ICPhysicsObj`: its id, `is_undetectable`,
/// whether it is static, and the report types it answers to (`CPhysicsObj`'s detection match).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellObject {
    pub id: u64,
    pub undetectable: bool,
    pub is_static: bool,
    /// `DetectionReportType::m_wtype_mask` bits the object is reported under.
    pub report_mask: u64,
}

impl CellObject {
    /// `CObjCell::UpdateAllVoyeur` and `ObjectWatchedBy` skip undetectable and static objects.
    fn watchable(&self) -> bool {
        !self.undetectable && !self.is_static
    }
}

/// `DetectionVoyeur`: the report type a watcher asked for and how many of its volumes cover
/// the cell (`ref_count`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voyeur {
    pub report_type: u64,
    pub object_iid: u64,
    pub ref_count: i32,
}

/// One report to a watcher (`DetectionInfo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectionNotice {
    pub voyeur: u64,
    pub object: u64,
    pub report_type: u64,
    pub status: DetectionStatus,
}

/// `CLandBlock::DynamicObjectInfo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DynamicObjectInfo {
    pub object: u64,
    /// `m_nVisibleTimestamp`: the scene timestamp the object last passed the view cull.
    pub visible_timestamp: u32,
}

#[derive(Debug, Default)]
struct CellMembers {
    /// `m_objectList`, in arrival order; removal moves the last entry into the hole.
    objects: Vec<u64>,
    /// `m_voyeurTable`, created by the first `AddVoyeur`.
    voyeurs: Option<HashMap<u64, Voyeur>>,
}

/// The cells' object lists and voyeur tables and the landblocks' dynamic lists.
#[derive(Debug, Default)]
pub struct CellMembership {
    cells: HashMap<u32, CellMembers>,
    /// `ICPhysicsObj::GetObjectA`: the objects the lists can name.
    objects: HashMap<u64, CellObject>,
    dynamic: HashMap<u32, Vec<DynamicObjectInfo>>,
    notices: Vec<DetectionNotice>,
}

fn block_of(cell_id: u32) -> u32 {
    cell_id & 0xFFFF_0000
}

impl CellMembership {
    pub fn new() -> Self {
        Self::default()
    }

    /// An object the world knows (`ICPhysicsObj::GetObjectA` finds it) without it standing in a
    /// cell yet; a watcher is one of these.
    pub fn know_object(&mut self, object: CellObject) {
        self.objects.insert(object.id, object);
    }

    /// An object the world forgets; lists that still hold it keep the entry.
    pub fn forget_object(&mut self, id: u64) {
        self.objects.remove(&id);
    }

    /// `CObjCell::SendObjectDetection`: tells `voyeur_id` about `object`, unless the voyeur is the
    /// object itself, the object is undetectable or static, or the voyeur is not a known object.
    fn send_object_detection(
        &mut self,
        voyeur_id: u64,
        voyeur: &Voyeur,
        object: &CellObject,
        status: DetectionStatus,
    ) {
        if voyeur_id == 0 || object.id == voyeur_id || !object.watchable() {
            return;
        }
        if !self.objects.contains_key(&voyeur_id) {
            return;
        }
        self.notices.push(DetectionNotice {
            voyeur: voyeur_id,
            object: object.id,
            report_type: voyeur.report_type,
            status,
        });
    }

    /// `CObjCell::UpdateAllVoyeur`: reports `object` to every watcher of the cell.
    pub fn update_all_voyeur(
        &mut self,
        cell_id: u32,
        object: &CellObject,
        status: DetectionStatus,
    ) {
        if !object.watchable() {
            return;
        }
        let Some(table) = self
            .cells
            .get(&cell_id)
            .and_then(|members| members.voyeurs.as_ref())
        else {
            return;
        };
        let voyeurs: Vec<(u64, Voyeur)> = table.iter().map(|(id, v)| (*id, *v)).collect();
        for (id, voyeur) in voyeurs {
            self.send_object_detection(id, &voyeur, object, status);
        }
    }

    /// `CObjCell::HideObject`: the watchers see it leave.
    pub fn hide_object(&mut self, cell_id: u32, object: &CellObject) {
        self.update_all_voyeur(cell_id, object, DetectionStatus::Left);
    }

    /// `CObjCell::UnhideObject`: the watchers see it enter.
    pub fn unhide_object(&mut self, cell_id: u32, object: &CellObject) {
        self.update_all_voyeur(cell_id, object, DetectionStatus::Entered);
    }

    /// `CObjCell::AddObject`: lists the object in the cell, tells the watchers it entered, and
    /// (when it is not static) adds it to the landblock's dynamic list
    /// (`CLandBlock::AddDynamicObject`).
    pub fn add_object(&mut self, cell_id: u32, object: CellObject) {
        self.objects.insert(object.id, object);
        self.cells
            .entry(cell_id)
            .or_default()
            .objects
            .push(object.id);
        self.update_all_voyeur(cell_id, &object, DetectionStatus::Entered);
        if !object.is_static {
            self.add_dynamic_object(block_of(cell_id), object.id);
        }
    }

    /// `CObjCell::RemoveObject`: takes the object out of the cell's list (the last entry fills
    /// its place), tells the watchers it left, and drops it from the landblock's dynamic list.
    /// Does nothing when the cell does not list it.
    pub fn remove_object(&mut self, cell_id: u32, id: u64) {
        let Some(members) = self.cells.get_mut(&cell_id) else {
            return;
        };
        let Some(index) = members.objects.iter().position(|&other| other == id) else {
            return;
        };
        members.objects.swap_remove(index);
        let Some(object) = self.objects.get(&id).copied() else {
            return;
        };
        self.update_all_voyeur(cell_id, &object, DetectionStatus::Left);
        if !object.is_static {
            self.remove_dynamic_object(block_of(cell_id), id);
        }
    }

    /// `CObjCell::AddVoyeur`: `voyeur_id` watches the cell for `report_type`. A watcher already
    /// there gains a reference; a new one is told about every object in the cell. False for the
    /// null id.
    pub fn add_voyeur(&mut self, cell_id: u32, voyeur_id: u64, report_type: u64) -> bool {
        if voyeur_id == 0 {
            return false;
        }
        let members = self.cells.entry(cell_id).or_default();
        let table = members.voyeurs.get_or_insert_with(HashMap::new);
        if let Some(existing) = table.get_mut(&voyeur_id) {
            existing.ref_count += 1;
            return true;
        }
        let voyeur = Voyeur {
            report_type,
            object_iid: voyeur_id,
            ref_count: 1,
        };
        table.insert(voyeur_id, voyeur);
        let listed: Vec<u64> = members.objects.clone();
        for id in listed {
            if let Some(object) = self.objects.get(&id).copied() {
                self.send_object_detection(voyeur_id, &voyeur, &object, DetectionStatus::Entered);
            }
        }
        true
    }

    /// `CObjCell::RemoveVoyeur`: drops one reference; the last one tells the watcher every object
    /// left and removes it. False when the cell has no such watcher.
    pub fn remove_voyeur(&mut self, cell_id: u32, voyeur_id: u64) -> bool {
        let Some(table) = self
            .cells
            .get_mut(&cell_id)
            .and_then(|members| members.voyeurs.as_mut())
        else {
            return false;
        };
        let Some(voyeur) = table.get_mut(&voyeur_id) else {
            return false;
        };
        voyeur.ref_count -= 1;
        if voyeur.ref_count < 1 {
            let voyeur = *voyeur;
            let listed: Vec<u64> = self.cells[&cell_id].objects.clone();
            for id in listed {
                if let Some(object) = self.objects.get(&id).copied() {
                    self.send_object_detection(voyeur_id, &voyeur, &object, DetectionStatus::Left);
                }
            }
            if let Some(table) = self
                .cells
                .get_mut(&cell_id)
                .and_then(|members| members.voyeurs.as_mut())
            {
                table.remove(&voyeur_id);
            }
        }
        true
    }

    /// `CObjCell::ObjectWatchedBy`: `voyeur_id` watches the cell and the object is detectable,
    /// not static, not the watcher itself, and answers to the watcher's report type.
    pub fn object_watched_by(&self, cell_id: u32, object: &CellObject, voyeur_id: u64) -> bool {
        if !object.watchable() || object.id == voyeur_id {
            return false;
        }
        let Some(table) = self
            .cells
            .get(&cell_id)
            .and_then(|members| members.voyeurs.as_ref())
        else {
            return false;
        };
        table
            .get(&voyeur_id)
            .is_some_and(|voyeur| object.report_mask & voyeur.report_type != 0)
    }

    /// `CObjCell::InqVoyeurs`: the watchers of the cell that watch `object`. The result is
    /// whether the list is not empty.
    pub fn inq_voyeurs(&self, cell_id: u32, object: &CellObject) -> (bool, Vec<u64>) {
        let mut watchers: Vec<u64> = self
            .cells
            .get(&cell_id)
            .and_then(|members| members.voyeurs.as_ref())
            .map(|table| {
                table
                    .values()
                    .filter(|voyeur| self.object_watched_by(cell_id, object, voyeur.object_iid))
                    .map(|voyeur| voyeur.object_iid)
                    .collect()
            })
            .unwrap_or_default();
        watchers.sort_unstable();
        (!watchers.is_empty(), watchers)
    }

    /// `CObjCell::InqObjects`: the cell's objects (only the non-static ones when `dynamic_only`).
    pub fn inq_objects(&self, cell_id: u32, dynamic_only: bool) -> Vec<u64> {
        self.cells
            .get(&cell_id)
            .map(|members| {
                members
                    .objects
                    .iter()
                    .copied()
                    .filter(|id| {
                        !dynamic_only || self.objects.get(id).is_some_and(|o| !o.is_static)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `CLandBlock::InqObjects`: every cell of the block, in cell id order.
    pub fn inq_block_objects(&self, block: u32, dynamic_only: bool) -> Vec<u64> {
        let mut cells: Vec<u32> = self
            .cells
            .keys()
            .copied()
            .filter(|cell| block_of(*cell) == block_of(block))
            .collect();
        cells.sort_unstable();
        cells
            .into_iter()
            .flat_map(|cell| self.inq_objects(cell, dynamic_only))
            .collect()
    }

    /// `CLandBlock::AddDynamicObject`.
    pub fn add_dynamic_object(&mut self, block: u32, object: u64) {
        self.dynamic
            .entry(block_of(block))
            .or_default()
            .push(DynamicObjectInfo {
                object,
                visible_timestamp: 0,
            });
    }

    /// `CLandBlock::RemoveDynamicObject`: the last entry fills the hole.
    pub fn remove_dynamic_object(&mut self, block: u32, object: u64) {
        let Some(list) = self.dynamic.get_mut(&block_of(block)) else {
            return;
        };
        if let Some(index) = list.iter().position(|info| info.object == object) {
            list.swap_remove(index);
        }
    }

    /// The block's dynamic objects (`CLandBlock::m_dynamic_objects`).
    pub fn dynamic_objects(&self, block: u32) -> &[DynamicObjectInfo] {
        self.dynamic
            .get(&block_of(block))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// A landblock leaves (`CLandBlock::ReleaseDynamicObjects`): its cells' lists and watchers
    /// and its dynamic list go.
    pub fn drop_block(&mut self, block: u32) {
        let block = block_of(block);
        self.cells.retain(|cell, _| block_of(*cell) != block);
        self.dynamic.remove(&block);
    }

    /// The reports queued since the last call, oldest first.
    pub fn take_notices(&mut self) -> Vec<DetectionNotice> {
        std::mem::take(&mut self.notices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CELL: u32 = 0x1234_0101;
    const OTHER: u32 = 0x1234_0102;

    fn mobile(id: u64) -> CellObject {
        CellObject {
            id,
            undetectable: false,
            is_static: false,
            report_mask: 0b11,
        }
    }

    fn fixed(id: u64) -> CellObject {
        CellObject {
            is_static: true,
            ..mobile(id)
        }
    }

    #[test]
    fn a_new_voyeur_hears_the_objects_already_there() {
        let mut m = CellMembership::new();
        m.know_object(mobile(9));
        m.add_object(CELL, mobile(1));
        m.add_object(CELL, fixed(2));
        assert!(m.take_notices().is_empty());
        assert!(m.add_voyeur(CELL, 9, 0b01));
        let notices = m.take_notices();
        assert_eq!(notices.len(), 1, "the static object is not reported");
        assert_eq!(notices[0].object, 1);
        assert_eq!(notices[0].voyeur, 9);
        assert_eq!(notices[0].status, DetectionStatus::Entered);
        assert!(!m.add_voyeur(CELL, 0, 1), "the null id never watches");
    }

    #[test]
    fn objects_entering_and_leaving_are_reported_to_every_watcher() {
        let mut m = CellMembership::new();
        m.know_object(mobile(8));
        m.know_object(mobile(9));
        m.add_voyeur(CELL, 8, 1);
        m.add_voyeur(CELL, 9, 1);
        m.add_object(CELL, mobile(1));
        let mut seen: Vec<_> = m
            .take_notices()
            .iter()
            .map(|n| (n.voyeur, n.status))
            .collect();
        seen.sort_by_key(|(voyeur, _)| *voyeur);
        assert_eq!(
            seen,
            vec![(8, DetectionStatus::Entered), (9, DetectionStatus::Entered)]
        );
        m.remove_object(CELL, 1);
        assert_eq!(m.take_notices().len(), 2);
        m.remove_object(CELL, 1);
        assert!(
            m.take_notices().is_empty(),
            "an unlisted object reports nothing"
        );
    }

    #[test]
    fn a_watcher_never_hears_about_itself_and_unknown_watchers_hear_nothing() {
        let mut m = CellMembership::new();
        m.know_object(mobile(8));
        m.add_voyeur(CELL, 8, 1);
        m.add_voyeur(CELL, 77, 1);
        m.add_object(CELL, mobile(8));
        assert!(m.take_notices().is_empty());
        m.add_object(CELL, mobile(5));
        let notices = m.take_notices();
        assert_eq!(notices.len(), 1, "77 is not a known object");
        assert_eq!(notices[0].voyeur, 8);
    }

    #[test]
    fn hiding_and_showing_report_leaving_and_entering() {
        let mut m = CellMembership::new();
        m.know_object(mobile(9));
        m.add_voyeur(CELL, 9, 1);
        let object = mobile(1);
        m.add_object(CELL, object);
        m.take_notices();
        m.hide_object(CELL, &object);
        m.unhide_object(CELL, &object);
        let status: Vec<_> = m.take_notices().iter().map(|n| n.status).collect();
        assert_eq!(
            status,
            vec![DetectionStatus::Left, DetectionStatus::Entered]
        );
        let ghost = CellObject {
            undetectable: true,
            ..object
        };
        m.hide_object(CELL, &ghost);
        assert!(m.take_notices().is_empty());
    }

    #[test]
    fn a_voyeur_is_counted_and_leaves_with_its_last_reference() {
        let mut m = CellMembership::new();
        m.know_object(mobile(9));
        m.add_object(CELL, mobile(1));
        m.add_voyeur(CELL, 9, 1);
        m.add_voyeur(CELL, 9, 1);
        m.take_notices();
        assert!(m.remove_voyeur(CELL, 9));
        assert!(m.take_notices().is_empty(), "one reference is left");
        assert!(m.remove_voyeur(CELL, 9));
        let notices = m.take_notices();
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].status, DetectionStatus::Left);
        assert!(!m.remove_voyeur(CELL, 9), "the watcher is gone");
        assert!(!m.remove_voyeur(OTHER, 9));
    }

    #[test]
    fn object_watched_by_needs_a_matching_report_type() {
        let mut m = CellMembership::new();
        m.know_object(mobile(9));
        m.add_voyeur(CELL, 9, 0b100);
        let a = mobile(1);
        assert!(!m.object_watched_by(CELL, &a, 9), "mask 0b11 has no bit 2");
        let b = CellObject {
            report_mask: 0b100,
            ..mobile(2)
        };
        assert!(m.object_watched_by(CELL, &b, 9));
        assert!(!m.object_watched_by(CELL, &b, 8));
        assert_eq!(m.inq_voyeurs(CELL, &b), (true, vec![9]));
        assert_eq!(m.inq_voyeurs(CELL, &a), (false, Vec::new()));
        let own = CellObject { id: 9, ..b };
        assert!(!m.object_watched_by(CELL, &own, 9));
    }

    #[test]
    fn the_dynamic_list_follows_non_static_objects_and_swap_removes() {
        let mut m = CellMembership::new();
        m.add_object(CELL, mobile(1));
        m.add_object(CELL, fixed(2));
        m.add_object(OTHER, mobile(3));
        m.add_object(OTHER, mobile(4));
        let list = |m: &CellMembership| -> Vec<u64> {
            m.dynamic_objects(CELL).iter().map(|d| d.object).collect()
        };
        assert_eq!(list(&m), vec![1, 3, 4]);
        m.remove_object(CELL, 1);
        assert_eq!(list(&m), vec![4, 3], "the last entry fills the hole");
        assert_eq!(m.inq_objects(CELL, false), vec![2]);
        assert_eq!(m.inq_objects(OTHER, true), vec![3, 4]);
        assert_eq!(m.inq_block_objects(CELL, false), vec![2, 3, 4]);
        assert_eq!(m.inq_block_objects(CELL, true), vec![3, 4]);
        m.drop_block(CELL);
        assert!(m.dynamic_objects(CELL).is_empty());
        assert!(m.inq_block_objects(CELL, false).is_empty());
    }

    #[test]
    fn removal_fills_the_hole_in_the_cell_list_too() {
        let mut m = CellMembership::new();
        for id in 1..=4 {
            m.add_object(CELL, mobile(id));
        }
        m.remove_object(CELL, 1);
        assert_eq!(m.inq_objects(CELL, false), vec![4, 2, 3]);
    }
}
