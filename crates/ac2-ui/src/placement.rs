//! Screen-box placement shared by `gm3DTextUI`'s name plates and floating damage texts: a box
//! is kept on the screen and off the boxes placed before it by moving it off whatever it hits
//! (`gm3DTextUI::PlaceHere`, `MakeNewData`), up to `MaxNamePlacementAttempts` times.

/// `Box2D` (inclusive corners).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Box2D {
    pub(crate) x0: i32,
    pub(crate) y0: i32,
    pub(crate) x1: i32,
    pub(crate) y1: i32,
}

impl Box2D {
    pub(crate) fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x0: x,
            y0: y,
            x1: x - 1 + width,
            y1: y - 1 + height,
        }
    }

    /// `Box2D::Collide`: both at least a pixel each way, and overlapping.
    pub(crate) fn collide(&self, other: &Box2D) -> bool {
        let real = |b: &Box2D| b.x1 - b.x0 >= 0 && b.y1 - b.y0 >= 0;
        real(self)
            && real(other)
            && self.x0 <= other.x1
            && other.x0 <= self.x1
            && self.y0 <= other.y1
            && other.y0 <= self.y1
    }
}

/// `gm3DTextUI::MakeNewData`: `moveme` moved off `collide` the shortest allowed way (left, up,
/// right, down by bit), that way's opposite then barred.
#[allow(clippy::needless_range_loop)] // `i` is also the bit of `bits` tested
pub(crate) fn make_new_data(moveme: &mut Box2D, collide: &Box2D, bits: &mut u32) {
    let dist = [
        (moveme.x1 - collide.x0 - 1).abs(),
        (moveme.y1 - collide.y0 - 1).abs(),
        (collide.x1 - moveme.x0 + 1).abs(),
        (collide.y1 - moveme.y0 + 1).abs(),
    ];
    let first = (0..4).find(|&i| *bits & (1 << i) != 0).unwrap_or(4);
    let mut best = first;
    let mut least = dist.get(first).copied().unwrap_or(0);
    for i in first + 1..4 {
        if *bits & (1 << i) != 0 && dist[i] < least {
            least = dist[i];
            best = i;
        }
    }
    let (dx, dy) = match best {
        0 => (-least, 0),
        2 => (least, 0),
        1 => (0, -least),
        3 => (0, least),
        _ => (0, 0),
    };
    moveme.x0 += dx;
    moveme.x1 += dx;
    moveme.y0 += dy;
    moveme.y1 += dy;
    *bits &= !(1 << ((best as u32).wrapping_sub(2) & 3));
}

/// The placement loop of `gm3DTextUI::UpdateNames` and `UpdateDamage`: `here` is moved off the
/// boxes in `placed` up to `attempts` times, and `None` when it leaves `screen` on the way (or
/// is not on it to start, once any attempt is made). With no attempts it is placed as it is.
pub(crate) fn place_box(
    screen: &Box2D,
    mut here: Box2D,
    placed: &[Box2D],
    attempts: i32,
) -> Option<Box2D> {
    let mut bits = 0xF;
    for _ in 0..attempts {
        if !screen.collide(&here) {
            return None;
        }
        // `PlaceHere`.
        let Some(hit) = placed.iter().position(|b| b.collide(&here)) else {
            break;
        };
        make_new_data(&mut here, &placed[hit], &mut bits);
        if bits == 0 {
            bits = 0xF;
        }
    }
    Some(here)
}

#[cfg(test)]
mod tests {
    use super::{make_new_data, place_box, Box2D};

    #[test]
    fn a_colliding_box_moves_the_shortest_allowed_way() {
        let placed = Box2D::new(100, 100, 40, 10);
        let mut moving = Box2D::new(134, 102, 20, 10);
        let mut bits = 0xF;
        make_new_data(&mut moving, &placed, &mut bits);
        // Retail's distances: left |153 - 100 - 1| = 52, up |111 - 100 - 1| = 10, right
        // |139 - 134 + 1| = 6, down |109 - 102 + 1| = 8: it moves right by 6.
        assert_eq!(moving, Box2D::new(140, 102, 20, 10));
        // Moved right: left is barred.
        assert_eq!(bits, 0xF & !1);
        assert!(!moving.collide(&placed));
        assert!(Box2D::new(0, 0, 1, 1).collide(&Box2D::new(0, 0, 5, 5)));
        assert!(!Box2D::new(0, 0, 0, 5).collide(&Box2D::new(0, 0, 5, 5)));
    }

    #[test]
    fn place_box_moves_off_placed_boxes_and_leaves_the_screen_out() {
        let screen = Box2D::new(0, 0, 200, 200);
        let placed = [Box2D::new(100, 100, 40, 10)];
        let here = place_box(&screen, Box2D::new(110, 102, 20, 10), &placed, 4).unwrap();
        assert!(!here.collide(&placed[0]));
        // Pushed past the screen's edge: not placed.
        let edge = Box2D::new(-30, 0, 20, 10);
        assert_eq!(place_box(&screen, edge, &[], 4), None);
        // No attempts: placed where it is, on or off the screen.
        assert_eq!(place_box(&screen, edge, &[], 0), Some(edge));
    }
}
