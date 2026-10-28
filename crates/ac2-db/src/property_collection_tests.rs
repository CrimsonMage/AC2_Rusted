//! `PropertyGroup` and `PropertyCollection` semantics that need no `MasterProperty` table: the
//! group-level add/update/remove rules and the collection's group routing (the name-to-group
//! lookups need the table and are covered against real data in `ac2-script`).

use crate::base_property::{BaseProperty, BasePropertyValue};
use crate::property::TYPE_INTEGER;
use crate::property_collection::{PropertyCollection, PropertyGroup};

fn int_property(name: u32, group: u32, value: i32) -> BaseProperty {
    BaseProperty {
        name,
        value_type: TYPE_INTEGER,
        group,
        value: Some(BasePropertyValue::Integer(value)),
    }
}

#[test]
fn group_add_refuses_a_second_property_of_the_name() {
    let mut group = PropertyGroup::default();
    assert!(group.add_property(&int_property(1, 7, 10)));
    assert!(!group.add_property(&int_property(1, 7, 11)));
    assert_eq!(group.properties.len(), 1);
    assert!(group.add_optional_property(&int_property(1, 7, 12)));
    assert!(!group.add_optional_property(&int_property(1, 7, 13)));
    assert_eq!(group.optional.len(), 1);
}

#[test]
fn group_update_replaces_the_value_or_appends() {
    let mut group = PropertyGroup::default();
    group.add_property(&int_property(1, 7, 10));
    assert!(group.update_property(&int_property(1, 7, 20)));
    assert_eq!(
        group.get_property(1).unwrap().value,
        Some(BasePropertyValue::Integer(20))
    );
    assert!(group.update_property(&int_property(2, 7, 30)));
    assert_eq!(group.properties.len(), 2);
    // A value of another type is not copied in (`BasePropertyValue::Set`), yet the result is 1.
    let mut other = int_property(1, 7, 0);
    other.value = Some(BasePropertyValue::Bool(true));
    assert!(group.update_property(&other));
    assert_eq!(
        group.get_property(1).unwrap().value,
        Some(BasePropertyValue::Integer(20))
    );
}

#[test]
fn group_remove_keeps_the_order_of_the_rest() {
    let mut group = PropertyGroup::default();
    for name in 1..=3 {
        group.add_property(&int_property(name, 7, 0));
    }
    assert!(group.remove_property(2));
    assert!(!group.remove_property(2));
    let names: Vec<u32> = group.properties.iter().map(|p| p.name).collect();
    assert_eq!(names, [1, 3]);
    assert_eq!(group.property_at(1).unwrap().name, 3);
    assert!(group.property_at(2).is_none());
}

#[test]
fn optional_to_required_needs_an_empty_required_list() {
    let mut group = PropertyGroup::default();
    group.add_optional_property(&int_property(1, 7, 0));
    assert!(group.set_optional_to_required());
    assert_eq!(group.properties.len(), 1);
    assert!(group.optional.is_empty());
    group.add_optional_property(&int_property(2, 7, 0));
    assert!(!group.set_optional_to_required());
}

#[test]
fn collection_adds_into_the_propertys_own_group() {
    let mut collection = PropertyCollection::default();
    assert!(collection.add_property(&int_property(1, 7, 0)));
    assert!(collection.add_property(&int_property(2, 7, 0)));
    assert!(collection.add_optional_property(&int_property(3, 8, 0)));
    assert_eq!(collection.groups.len(), 2);
    assert_eq!(collection.group_at(0).unwrap().properties.len(), 2);
    assert_eq!(collection.group_at(1).unwrap().optional.len(), 1);
    assert!(collection.contains_group(8));
    assert!(!collection.contains_group(9));
    // Same name in the same group is refused by the group.
    assert!(!collection.add_property(&int_property(1, 7, 5)));
}

#[test]
fn collection_update_merges_groups_and_adds_missing_ones() {
    let mut base = PropertyCollection::default();
    base.add_property(&int_property(1, 7, 10));
    let mut update = PropertyCollection::default();
    update.add_property(&int_property(1, 7, 99));
    update.add_property(&int_property(5, 9, 1));
    assert!(base.update_collection(&update));
    assert_eq!(base.groups.len(), 2);
    assert_eq!(
        base.group_at(0).unwrap().get_property(1).unwrap().value,
        Some(BasePropertyValue::Integer(99))
    );
    assert!(base.update_property(&int_property(6, 11, 1)));
    assert!(base.contains_group(11));
}
