#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Box2D {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Box2D {
    pub const fn from_origin_size(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x0: x,
            y0: y,
            x1: x + width - 1,
            y1: y + height - 1,
        }
    }

    pub const fn from_corners_inclusive(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub const fn width(self) -> i32 {
        self.x1 - self.x0 + 1
    }

    pub const fn height(self) -> i32 {
        self.y1 - self.y0 + 1
    }

    pub const fn is_valid(self) -> bool {
        self.width() > 0 && self.height() > 0
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let intersection = Self {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        };

        intersection.is_valid().then_some(intersection)
    }

    pub fn includes(self, other: Self) -> bool {
        self.is_valid()
            && other.is_valid()
            && other.x0 >= self.x0
            && other.x1 <= self.x1
            && other.y0 >= self.y0
            && other.y1 <= self.y1
    }

    pub fn collides(self, other: Self) -> bool {
        self.is_valid()
            && other.is_valid()
            && self.x0 <= other.x1
            && self.x1 >= other.x0
            && self.y0 <= other.y1
            && self.y1 >= other.y0
    }

    pub fn subtract(self, other: Self) -> Box2DFragments {
        let mut fragments = Box2DFragments::default();
        let Some(overlap) = self.intersection(other) else {
            return fragments;
        };

        fragments.push(Self::from_corners_inclusive(
            self.x0,
            self.y0,
            overlap.x0 - 1,
            self.y1,
        ));
        fragments.push(Self::from_corners_inclusive(
            overlap.x1 + 1,
            self.y0,
            self.x1,
            self.y1,
        ));
        fragments.push(Self::from_corners_inclusive(
            overlap.x0,
            self.y0,
            overlap.x1,
            overlap.y0 - 1,
        ));
        fragments.push(Self::from_corners_inclusive(
            overlap.x0,
            overlap.y1 + 1,
            overlap.x1,
            self.y1,
        ));
        fragments
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Box2DFragments {
    boxes: [Box2D; 4],
    len: usize,
}

impl Box2DFragments {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Box2D> {
        self.boxes[..self.len].iter()
    }

    fn push(&mut self, candidate: Box2D) {
        if candidate.is_valid() && self.len < self.boxes.len() {
            self.boxes[self.len] = candidate;
            self.len += 1;
        }
    }
}

impl<'a> IntoIterator for &'a Box2DFragments {
    type Item = &'a Box2D;
    type IntoIter = std::slice::Iter<'a, Box2D>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Box2D;

    #[test]
    fn inclusive_sizes_and_intersection_match_legacy_shape() {
        let left = Box2D::from_origin_size(10, 20, 4, 3);
        let right = Box2D::from_origin_size(12, 21, 4, 3);

        assert_eq!(left.width(), 4);
        assert_eq!(left.height(), 3);
        assert_eq!(
            left.intersection(right),
            Some(Box2D::from_corners_inclusive(12, 21, 13, 22))
        );
    }

    #[test]
    fn includes_and_collides_behave_as_expected() {
        let outer = Box2D::from_origin_size(0, 0, 10, 10);
        let inner = Box2D::from_origin_size(2, 2, 3, 3);
        let far = Box2D::from_origin_size(20, 20, 2, 2);

        assert!(outer.includes(inner));
        assert!(outer.collides(inner));
        assert!(!outer.collides(far));
    }

    #[test]
    fn subtract_splits_remaining_regions() {
        let source = Box2D::from_origin_size(0, 0, 5, 5);
        let remove = Box2D::from_origin_size(1, 1, 3, 3);

        let fragments = source.subtract(remove);
        let pieces: Vec<_> = fragments.iter().copied().collect();

        assert_eq!(fragments.len(), 4);
        assert_eq!(pieces[0], Box2D::from_corners_inclusive(0, 0, 0, 4));
        assert_eq!(pieces[1], Box2D::from_corners_inclusive(4, 0, 4, 4));
        assert_eq!(pieces[2], Box2D::from_corners_inclusive(1, 0, 3, 0));
        assert_eq!(pieces[3], Box2D::from_corners_inclusive(1, 4, 3, 4));
    }
}
