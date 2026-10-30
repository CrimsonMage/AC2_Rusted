//! The tree's parent and root bookkeeping (split from `element.rs`): an element's parent, the
//! z-ordered child and root lists.

use crate::element::UiTree;

impl UiTree {
    /// `UIElement::SetParent`: out of the old parent's children, into the new one's by z
    /// (`UIRegion::SetParent_Internal`'s parent link).
    pub(crate) fn set_parent(&mut self, index: usize, parent: Option<usize>) {
        if let Some(old) = self.el(index).parent {
            self.el_mut(old).children.retain(|&c| c != index);
        }
        self.el_mut(index).parent = parent;
        if let Some(parent) = parent {
            let at = self.z_slot(&self.el(parent).children, self.el(index).z);
            self.el_mut(parent).children.insert(at, index);
        }
        // `UIElement_Meter::SetParent`.
        self.meter_update_layout(index);
    }

    /// `UIElement::AddChild`'s position for z `z` in an ascending list: last when `0xFFFF` or
    /// above the last, else before the first entry whose z is not below it.
    pub(crate) fn z_slot(&self, list: &[usize], z: i32) -> usize {
        match list.last() {
            None => 0,
            Some(&last) if z == 0xFFFF || self.el(last).z < z => list.len(),
            Some(_) => list
                .iter()
                .position(|&other| z <= self.el(other).z)
                .unwrap_or(list.len()),
        }
    }

    /// Adds a root in z order (`IUIManager` root list).
    pub(crate) fn add_root(&mut self, index: usize) {
        self.roots.retain(|&r| r != index);
        let at = self.z_slot(&self.roots, self.el(index).z);
        self.roots.insert(at, index);
    }
}
