//! `CObjectMaint`'s temporary ids (`AllocateTemporaryID`, `DeallocateTemporaryID`): the ids of the
//! objects the client makes itself (`CreateTemporaryObject`: an FX's objects, a story's
//! missiles). `IIDUtils::CreateTemporary(0)` is the first; each allocation takes the lowest id
//! from there that `m_used_temporary_ids` does not hold and no object has, and an object's
//! destruction gives its id back.

use std::collections::BTreeSet;

/// `IIDUtils::CreateTemporary(0)`: the temporary id type (`0x13`) in the top byte, index 0.
pub const TEMPORARY_BASE: u64 = 0x1300_0000_0000_0000;

/// `IIDUtils::VerifyTemporary`.
pub fn is_temporary(id: u64) -> bool {
    ac2_net::iid::id_type(id) == ac2_net::iid::IdType::Temporary
}

/// `CObjectMaint::m_used_temporary_ids`.
#[derive(Debug, Default, Clone)]
pub struct TemporaryIds {
    used: BTreeSet<u64>,
}

impl TemporaryIds {
    /// `CObjectMaint::AllocateTemporaryID`: the lowest unused id from [`TEMPORARY_BASE`]; `None`
    /// past the temporary range (`VerifyTemporary` fails) or when an object already has it
    /// (`exists`, the object table's `GetObjectA`).
    pub fn allocate(&mut self, exists: impl Fn(u64) -> bool) -> Option<u64> {
        let mut id = TEMPORARY_BASE;
        while self.used.contains(&id) {
            id = id.wrapping_add(1);
        }
        if !is_temporary(id) || exists(id) {
            return None;
        }
        self.used.insert(id);
        Some(id)
    }

    /// `CObjectMaint::DeallocateTemporaryID`.
    pub fn deallocate(&mut self, id: u64) {
        self.used.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_count_up_from_the_base_and_a_freed_one_is_taken_again() {
        let mut ids = TemporaryIds::default();
        let none = |_| false;
        assert_eq!(ids.allocate(none), Some(TEMPORARY_BASE));
        assert_eq!(ids.allocate(none), Some(TEMPORARY_BASE + 1));
        assert_eq!(ids.allocate(none), Some(TEMPORARY_BASE + 2));
        ids.deallocate(TEMPORARY_BASE + 1);
        assert_eq!(
            ids.allocate(none),
            Some(TEMPORARY_BASE + 1),
            "the lowest free id"
        );
        assert_eq!(ids.allocate(none), Some(TEMPORARY_BASE + 3));
        // An object already holding the id fails the allocation and reserves nothing.
        assert_eq!(ids.allocate(|id| id == TEMPORARY_BASE + 4), None);
        assert_eq!(ids.allocate(none), Some(TEMPORARY_BASE + 4));
        assert!(is_temporary(TEMPORARY_BASE + 4));
        assert!(!is_temporary(0x8000_0000_0000_0001));
    }
}
