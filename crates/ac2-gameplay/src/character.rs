//! The character set's slot rules (`ac2/shared/src/gameplay/character/characterset.cpp.c`,
//! Legions 7160). The set's active characters are an ordered list; a character is found by its
//! id. The list is `ac2_net::protocol::CharacterSet::characters`.

use ac2_net::protocol::CharacterIdentity;

/// `CharacterSet::WhichSlot`: the index of the first character with `id`, or `None` (retail -1).
pub fn which_slot(characters: &[CharacterIdentity], id: u64) -> Option<usize> {
    characters.iter().position(|c| c.id == id)
}

/// `CharacterSet::IncludesIdentity`: whether a character with `id` is listed.
pub fn includes_identity(characters: &[CharacterIdentity], id: u64) -> bool {
    which_slot(characters, id).is_some()
}

/// `SmartArray<CharacterIdentity>::InsertOrderedByIndex`: inserts at `index`, moving the
/// entries from there up by one. An index past the end refuses (retail returns 0) and nothing
/// changes; `index == len` appends.
pub fn insert_ordered_by_index(
    characters: &mut Vec<CharacterIdentity>,
    index: usize,
    identity: CharacterIdentity,
) -> bool {
    if index > characters.len() {
        return false;
    }
    characters.insert(index, identity);
    true
}

/// `CharacterSet::AddIdentity`: an identity already listed is not added (`None`). Otherwise it
/// goes in at `slot` when that is in range (`InsertOrderedByIndex`), else at the end
/// (`AddToEnd`). Returns the index it was added at.
pub fn add_identity(
    characters: &mut Vec<CharacterIdentity>,
    identity: CharacterIdentity,
    slot: Option<usize>,
) -> Option<usize> {
    if includes_identity(characters, identity.id) {
        return None;
    }
    if let Some(index) = slot {
        if insert_ordered_by_index(characters, index, identity.clone()) {
            return Some(index);
        }
    }
    characters.push(identity);
    Some(characters.len() - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::{CharacterIdentity, VisualDesc};

    fn who(id: u64) -> CharacterIdentity {
        CharacterIdentity {
            id,
            name: format!("c{id}"),
            seconds_greyed_out: 0,
            visual: VisualDesc::default(),
        }
    }

    #[test]
    fn a_character_is_found_by_id_and_absent_ids_have_no_slot() {
        let list = vec![who(10), who(20)];
        assert_eq!(which_slot(&list, 20), Some(1));
        assert_eq!(which_slot(&list, 30), None);
        assert!(includes_identity(&list, 10));
        assert!(!includes_identity(&list, 30));
    }

    #[test]
    fn an_identity_already_listed_is_not_added_again() {
        let mut list = vec![who(10)];
        assert_eq!(add_identity(&mut list, who(10), None), None);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn a_slot_inserts_in_order_and_an_out_of_range_slot_appends() {
        let mut list = vec![who(10), who(20)];
        assert_eq!(add_identity(&mut list, who(30), Some(1)), Some(1));
        assert_eq!(list.iter().map(|c| c.id).collect::<Vec<_>>(), [10, 30, 20]);
        assert_eq!(add_identity(&mut list, who(40), Some(9)), Some(3));
        assert_eq!(
            list.iter().map(|c| c.id).collect::<Vec<_>>(),
            [10, 30, 20, 40]
        );
        assert!(!insert_ordered_by_index(&mut list, 9, who(50)));
        assert_eq!(list.len(), 4);
    }
}
