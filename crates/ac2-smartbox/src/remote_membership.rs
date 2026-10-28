//! A remote's entry in its cell's object list.

use crate::membership::{CellMembership, CellObject};
use crate::remotes::Remote;

/// `CPhysicsObj::add_to_cell` / `remove_from_cell` and `CObjCell::AddObject` / `RemoveObject`:
/// an object in the world with a body is listed in the cell of its position; a move to another
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
