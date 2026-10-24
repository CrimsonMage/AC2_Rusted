use crate::Plane;
use ac2_core::Sidedness;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipPlane {
    pub plane: Option<Plane>,
    pub side: Sidedness,
}

impl Default for ClipPlane {
    fn default() -> Self {
        Self::empty()
    }
}

impl ClipPlane {
    pub const fn empty() -> Self {
        Self {
            plane: None,
            side: Sidedness::POSITIVE,
        }
    }

    pub const fn new(plane: Plane, side: Sidedness) -> Self {
        Self {
            plane: Some(plane),
            side,
        }
    }

    pub fn is_empty(self) -> bool {
        self.plane.is_none()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClipPlaneList {
    planes: Vec<ClipPlane>,
    pub leaf_contains_obj: bool,
}

impl ClipPlaneList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.planes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.planes.is_empty()
    }

    pub fn push(&mut self, plane: ClipPlane) {
        self.planes.push(plane);
    }

    pub fn clear(&mut self) {
        self.planes.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ClipPlane> {
        self.planes.iter()
    }
}

impl<'a> IntoIterator for &'a ClipPlaneList {
    type Item = &'a ClipPlane;
    type IntoIter = std::slice::Iter<'a, ClipPlane>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use ac2_core::Sidedness;
    use glam::Vec3;

    use super::{ClipPlane, ClipPlaneList};
    use crate::Plane;

    #[test]
    fn empty_clip_plane_is_none_backed() {
        let plane = ClipPlane::default();
        assert!(plane.is_empty());
    }

    #[test]
    fn clip_plane_list_push_and_clear() {
        let mut list = ClipPlaneList::new();
        list.push(ClipPlane::new(
            Plane::new(Vec3::Z, 0.0),
            Sidedness::POSITIVE,
        ));

        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());

        list.clear();
        assert!(list.is_empty());
    }
}
