//! `CollideProperty` and the property lists a collision report keeps for the landscape, the
//! water and the environment (`CollidePropertyList`'s `PSmartArray<CollideProperty>`, and
//! `CCollisionManager::UpdatePropertyArray` with `ChangePhysicsMaterial` and `RemoveEndElements`).
//!
//! A physics material's properties are merged into a list by name, marked before the merge:
//! every entry starts as an end, the material's inquirable properties mark the entries they name
//! as the same or changed (or add them as begun), and whatever is still an end is removed. The
//! report then lists each entry that is not the same and that the property table reports to the
//! weenie (`CCollisionManager::ReportSurfaceProperties` and its siblings).

use ac2_db::{
    Ac2PropertyDesc, Ac2PropertyGroup, Ac2PropertyValue, REPORT_IGNORED, REPORT_REPORTABLE,
};

/// The terrain-type property (`0x40000036`): the motion terrain a landscape or water material
/// gives (`CCollisionManager::UpdateLandTerrain` and `UpdateWaterTerrain`).
pub const TERRAIN_TYPE_PROPERTY: u32 = 0x4000_0036;

/// `CollideProperty::m_collide_type`: 0 begun, 1 ended, 2 changed, 3 the same as before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollideType {
    Begin,
    End,
    Change,
    Same,
}

impl CollideType {
    /// The retail value of `m_collide_type`.
    pub fn raw(self) -> u64 {
        match self {
            Self::Begin => 0,
            Self::End => 1,
            Self::Change => 2,
            Self::Same => 3,
        }
    }
}

/// One property a report lists or a list keeps (`CollideProperty`): its master id, its collide
/// type and the property it names (`m_property`).
#[derive(Debug, Clone, PartialEq)]
pub struct CollideProperty {
    /// `m_id`: the id the list gave the property when it began.
    pub id: u64,
    pub collide_type: CollideType,
    pub property: Ac2PropertyValue,
}

impl CollideProperty {
    /// `CollideProperty::GetPropertyName`: the property's name id.
    pub fn name(&self) -> u32 {
        self.property.name
    }
}

/// The property tables a list consults. The client reads the installed `MasterProperty`
/// (`ac2_property_desc`); a test can supply its own.
#[derive(Debug, Clone, Copy)]
pub struct PropertyDescs(pub fn(u32) -> Option<Ac2PropertyDesc>);

impl PropertyDescs {
    /// `BaseProperty::IsInquirable`: the property has a desc that is not ignored.
    pub fn inquirable(self, name: u32) -> bool {
        (self.0)(name).is_some_and(|desc| desc.report_to_weenie != REPORT_IGNORED)
    }

    /// `BaseProperty::IsReportedToWeenie`: the desc is `Reportable`.
    pub fn reported(self, name: u32) -> bool {
        (self.0)(name).is_some_and(|desc| desc.report_to_weenie == REPORT_REPORTABLE)
    }
}

impl Default for PropertyDescs {
    fn default() -> Self {
        Self(installed_desc)
    }
}

fn installed_desc(name: u32) -> Option<Ac2PropertyDesc> {
    ac2_db::ac2_property_desc(name)
}

/// `CCollisionManager::UpdatePropertyArray`: merges a material's property groups into `list`.
/// Each inquirable property the list already holds is the same, or changed (its value replaced);
/// one it does not hold begins, with the next master id (`m_master_id`). Returns whether any
/// entry changed or began.
pub fn update_property_array(
    groups: &[Ac2PropertyGroup],
    list: &mut Vec<CollideProperty>,
    next_id: &mut u64,
    descs: PropertyDescs,
) -> bool {
    let mut changed = false;
    for property in groups.iter().flat_map(|group| &group.properties) {
        if !descs.inquirable(property.name) {
            continue;
        }
        match list.iter_mut().find(|entry| entry.name() == property.name) {
            Some(entry) => {
                if entry.property == *property {
                    entry.collide_type = CollideType::Same;
                } else {
                    entry.property = property.clone();
                    entry.collide_type = CollideType::Change;
                    changed = true;
                }
            }
            None => {
                list.push(CollideProperty {
                    id: *next_id,
                    collide_type: CollideType::Begin,
                    property: property.clone(),
                });
                *next_id += 1;
                changed = true;
            }
        }
    }
    changed
}

/// `ChangePhysicsMaterial`'s first step (`StartEnteringWorld` on each entry): every entry is
/// marked as ended until a property of the new material names it again.
pub fn mark_all_end(list: &mut [CollideProperty]) {
    for entry in list {
        entry.collide_type = CollideType::End;
    }
}

/// `CCollisionManager::RemoveEndElements`: drops the ended entries. Like retail it moves the last
/// entry into each hole, so the order of the rest can change. Returns whether any was dropped.
pub fn remove_end(list: &mut Vec<CollideProperty>) -> bool {
    let mut removed = false;
    let mut index = 0;
    while index < list.len() {
        if list[index].collide_type == CollideType::End {
            list.swap_remove(index);
            removed = true;
        } else {
            index += 1;
        }
    }
    removed
}

/// The terrain-type value of a property: the integer `GetValue(ulong&)` reads, when it is one.
pub fn terrain_type_value(property: &Ac2PropertyValue) -> Option<u64> {
    (property.name == TERRAIN_TYPE_PROPERTY)
        .then(|| property.integer())
        .flatten()
}
