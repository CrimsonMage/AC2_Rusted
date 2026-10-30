//! A remote's entry in its cell's object list.

use crate::membership::{CellMembership, CellObject};
use crate::remotes::Remote;

/// `CPhysicsObj::change_cell`, `enter_cell`, `leave_cell` (`add_to_cell` / `remove_from_cell`)
/// and `CObjCell::AddObject` / `RemoveObject`: an object in the world with a body is listed in the cell of its position; a move to another
/// cell takes it out of the old list and into the new one.
pub(crate) fn sync_membership(guid: u64, remote: &mut Remote, membership: &mut CellMembership) {
    let wanted = match (&remote.body, remote.in_world) {
        (Some(body), true) => Some((
            body.position().cell_id,
            CellObject {
                id: guid,
                undetectable: false,
                is_static: body.is_static(),
                report_mask: u64::MAX,
            },
        )),
        _ => None,
    };
    match (wanted, remote.listed_cell) {
        (Some((cell, object)), None) => {
            membership.add_object(cell, object);
            remote.listed_cell = Some(cell);
        }
        (Some((cell, object)), Some(old)) if cell != old => {
            membership.remove_object(old, guid);
            membership.add_object(cell, object);
            remote.listed_cell = Some(cell);
        }
        (None, Some(old)) => {
            membership.remove_object(old, guid);
            remote.listed_cell = None;
        }
        _ => {}
    }
}

impl<B> crate::remotes::Remotes<B> {
    /// `CPhysicsObj::InqCellObjects`: the objects in the cell the object stands in (only the
    /// non-static ones when `dynamic_only`); empty when it is in no cell.
    pub fn inq_cell_objects(&mut self, id: u64, dynamic_only: bool) -> Vec<u64> {
        let Some(cell) = self.get(id).and_then(|remote| remote.listed_cell) else {
            return Vec::new();
        };
        self.membership().inq_objects(cell, dynamic_only)
    }

    /// `CPhysicsObj::InqBlockObjects`: the objects in every cell of the landblock the object
    /// stands in; empty when it is in no cell.
    pub fn inq_block_objects(&mut self, id: u64, dynamic_only: bool) -> Vec<u64> {
        let Some(cell) = self.get(id).and_then(|remote| remote.listed_cell) else {
            return Vec::new();
        };
        self.membership().inq_block_objects(cell, dynamic_only)
    }
}
