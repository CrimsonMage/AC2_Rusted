//! `PropertyCollection` and `PropertyGroup` (`property/propertycollection.cpp`,
//! `property/propertygroup.cpp`): the editable property sets a script builds and queries through
//! `gPropertyCollection` / `gPropertyGroup`. A collection holds groups in insertion order; a
//! group holds a required and an optional list of [`BaseProperty`]. Every add, update and inquiry
//! returns retail's int result as a `bool`.

use crate::base_property::BaseProperty;
use crate::property::ac2_property_desc;

/// `PropertyGroup`: a group word and its required and optional properties in insertion order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropertyGroup {
    /// `m_groupName`.
    pub name: u32,
    /// `m_properties`.
    pub properties: Vec<BaseProperty>,
    /// `m_optionalProperties`.
    pub optional: Vec<BaseProperty>,
}

impl PropertyGroup {
    /// `PropertyGroup::GetProperty`: the required property of that name.
    pub fn get_property(&self, name: u32) -> Option<&BaseProperty> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// `PropertyGroup::GetOptionalProperty`.
    pub fn get_optional_property(&self, name: u32) -> Option<&BaseProperty> {
        self.optional.iter().find(|p| p.name == name)
    }

    /// `PropertyGroup::ContainsProperty`.
    pub fn contains_property(&self, name: u32) -> bool {
        self.get_property(name).is_some()
    }

    /// `PropertyGroup::ContainsOptionalProperty`.
    pub fn contains_optional_property(&self, name: u32) -> bool {
        self.get_optional_property(name).is_some()
    }

    /// `PropertyGroup::AddProperty`: appends unless a required property has that name (0).
    pub fn add_property(&mut self, property: &BaseProperty) -> bool {
        if self.contains_property(property.name) {
            return false;
        }
        self.properties.push(property.clone());
        true
    }

    /// `PropertyGroup::AddOptionalProperty`: appends unless an optional property has that name.
    pub fn add_optional_property(&mut self, property: &BaseProperty) -> bool {
        if self.contains_optional_property(property.name) {
            return false;
        }
        self.optional.push(property.clone());
        true
    }

    /// `PropertyGroup::RemoveProperty`: removes the required property of that name, keeping the
    /// order of the rest.
    pub fn remove_property(&mut self, name: u32) -> bool {
        match self.properties.iter().position(|p| p.name == name) {
            Some(index) => {
                self.properties.remove(index);
                true
            }
            None => false,
        }
    }

    /// `PropertyGroup::UpdateProperty`: a required property of the name takes the new value
    /// (`BasePropertyValue::Set`, which copies only a value of its own type; a property without a
    /// value changes nothing) and the result is 1; otherwise the property is appended.
    pub fn update_property(&mut self, property: &BaseProperty) -> bool {
        if let Some(existing) = self.properties.iter_mut().find(|p| p.name == property.name) {
            if let (Some(slot), Some(source)) = (existing.value.as_mut(), property.value.as_ref()) {
                slot.set_from(source);
            }
            return true;
        }
        self.properties.push(property.clone());
        true
    }

    /// `PropertyGroup::UpdateGroup`: updates every required property of `group`; always 1.
    pub fn update_group(&mut self, group: &PropertyGroup) -> bool {
        for property in &group.properties {
            self.update_property(property);
        }
        true
    }

    /// `PropertyGroup::SetOptionalToRequired`: an empty required list takes the optional list
    /// (which empties); a group with required properties answers 0.
    pub fn set_optional_to_required(&mut self) -> bool {
        if !self.properties.is_empty() {
            return false;
        }
        self.properties = std::mem::take(&mut self.optional);
        true
    }

    /// `PropertyGroup::InqPropertyAt`.
    pub fn property_at(&self, index: u32) -> Option<&BaseProperty> {
        self.properties.get(index as usize)
    }

    /// `PropertyGroup::InqOptionalPropertyAt`.
    pub fn optional_property_at(&self, index: u32) -> Option<&BaseProperty> {
        self.optional.get(index as usize)
    }
}

/// `PropertyCollection`: groups in insertion order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropertyCollection {
    pub groups: Vec<PropertyGroup>,
}

/// The group `MasterProperty` files the property name under (`FUN_0064e730`, the group inquiry
/// `PropertyCollection::GetProperty` makes); `None` when the name has no desc, or its group is 0.
fn master_group(name: u32) -> Option<u32> {
    ac2_property_desc(name)
        .map(|desc| desc.group)
        .filter(|&group| group != 0)
}

impl PropertyCollection {
    /// `PropertyCollection::GetGroup`.
    pub fn get_group(&self, name: u32) -> Option<&PropertyGroup> {
        self.groups.iter().find(|g| g.name == name)
    }

    fn group_mut(&mut self, name: u32) -> Option<&mut PropertyGroup> {
        self.groups.iter_mut().find(|g| g.name == name)
    }

    /// `PropertyCollection::GetProperty`: the group the name's `MasterProperty` desc files it
    /// under, then that group's required property; a name with no desc finds nothing.
    pub fn get_property(&self, name: u32) -> Option<&BaseProperty> {
        self.get_group(master_group(name)?)?.get_property(name)
    }

    /// `PropertyCollection::GetOptionalProperty`.
    pub fn get_optional_property(&self, name: u32) -> Option<&BaseProperty> {
        self.get_group(master_group(name)?)?
            .get_optional_property(name)
    }

    /// `PropertyCollection::ContainsGroup`.
    pub fn contains_group(&self, name: u32) -> bool {
        self.get_group(name).is_some()
    }

    /// `PropertyCollection::ContainsProperty`.
    pub fn contains_property(&self, name: u32) -> bool {
        self.get_property(name).is_some()
    }

    /// `PropertyCollection::AddGroup`: a group already present is left alone (1).
    pub fn add_group(&mut self, group: &PropertyGroup) -> bool {
        if !self.contains_group(group.name) {
            self.groups.push(group.clone());
        }
        true
    }

    /// `PropertyCollection::UpdateGroup`: merges into the group of the name
    /// (`PropertyGroup::UpdateGroup`) or adds it.
    pub fn update_group(&mut self, group: &PropertyGroup) -> bool {
        match self.group_mut(group.name) {
            Some(existing) => existing.update_group(group),
            None => self.add_group(group),
        }
    }

    /// `PropertyCollection::UpdateCollection`: every group of `other` updates in.
    pub fn update_collection(&mut self, other: &PropertyCollection) -> bool {
        other
            .groups
            .iter()
            .fold(true, |ok, group| self.update_group(group) & ok)
    }

    /// `PropertyCollection::SetOptionalToRequired`: stops at the first group that refuses.
    pub fn set_optional_to_required(&mut self) -> bool {
        self.groups
            .iter_mut()
            .all(PropertyGroup::set_optional_to_required)
    }

    /// `PropertyCollection::AddProperty`: refused when the name is already present; otherwise
    /// added to the group the property itself names, which is created when absent.
    pub fn add_property(&mut self, property: &BaseProperty) -> bool {
        self.add_with(property, PropertyGroup::add_property, Self::get_property)
    }

    /// `PropertyCollection::AddOptionalProperty`.
    pub fn add_optional_property(&mut self, property: &BaseProperty) -> bool {
        self.add_with(
            property,
            PropertyGroup::add_optional_property,
            Self::get_optional_property,
        )
    }

    fn add_with(
        &mut self,
        property: &BaseProperty,
        add: fn(&mut PropertyGroup, &BaseProperty) -> bool,
        present: fn(&Self, u32) -> Option<&BaseProperty>,
    ) -> bool {
        if present(self, property.name).is_some() {
            return false;
        }
        if let Some(group) = self.group_mut(property.group) {
            return add(group, property);
        }
        let mut group = PropertyGroup {
            name: property.group,
            ..PropertyGroup::default()
        };
        add(&mut group, property) && self.add_group(&group)
    }

    /// `PropertyCollection::UpdateProperty`: updates inside the property's own group, or adds the
    /// property when that group is absent.
    pub fn update_property(&mut self, property: &BaseProperty) -> bool {
        match self.group_mut(property.group) {
            Some(group) => group.update_property(property),
            None => self.add_property(property),
        }
    }

    /// `PropertyCollection::InqGroupAt`.
    pub fn group_at(&self, index: u32) -> Option<&PropertyGroup> {
        self.groups.get(index as usize)
    }
}
