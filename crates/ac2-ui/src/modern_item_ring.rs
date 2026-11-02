//! The modern inventory drawer's item ring (`modern_inventory.rs`; AGENTS.md: the HUD's two
//! styles): round buttons fanned out around the selected item's slot, shortcuts to the gestures
//! that item already takes. Use (`UseSelection`, retail's double-click), Examine (the right press,
//! `OpenExaminationPanel`), Split (the tab drop's stack question and move, `gmStackUI`), Melt (the
//! drop on the money bag, `gmMoneyBagUI::HandleMoneyBagDrop`, its question and confirmation), and
//! Equip (the drop on the doll, `EquipItem` at the item's preferred location) or, for a worn item,
//! Unequip (`UnEquipItem` into the first page with room). Each sends what its gesture sends; an
//! action that does not apply is dimmed and takes no press. The ring opens up from a slot of the
//! grid (down when there is no room above) and sideways from a worn item's slot, towards the doll,
//! and hides while something is dragged.

use std::cell::Cell;

use crate::element::{UiBox, UiTree};
use crate::modern_art as art;
use crate::modern_inventory::{keep, make_part};
use crate::token::MOUSE_INVISIBLE;

/// The ring's actions, in the art's order (`modern_art::ring_button`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingAction {
    Use,
    Examine,
    Split,
    Melt,
    Equip,
    Unequip,
}

const ACTIONS: [RingAction; art::RING_ACTIONS] = [
    RingAction::Use,
    RingAction::Examine,
    RingAction::Split,
    RingAction::Melt,
    RingAction::Equip,
    RingAction::Unequip,
];

/// The ring's radius from the slot's centre and the spread of its buttons either side of the
/// way it opens, degrees.
const RADIUS: f32 = 40.0;
const SPREAD: f32 = 78.0;

/// Where the ring opens from and what applies, as the drawer found it this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RingTarget {
    pub item: u64,
    /// The slot's box on the screen.
    pub slot: UiBox,
    /// The box the buttons stay inside (the grid, or the drawer for a worn item).
    pub bounds: UiBox,
    /// Opening direction, degrees (0 right, 90 down).
    pub towards: f32,
    pub worn: bool,
    /// Split, Melt and Equip apply (Use and Examine always do).
    pub split: bool,
    pub melt: bool,
    pub equip: bool,
}

impl RingTarget {
    /// The actions shown, in order, each with whether it applies.
    fn actions(&self) -> [(RingAction, bool); 5] {
        let wear = if self.worn {
            (RingAction::Unequip, true)
        } else {
            (RingAction::Equip, self.equip)
        };
        [
            (RingAction::Use, true),
            (RingAction::Examine, true),
            (RingAction::Split, self.split && !self.worn),
            (RingAction::Melt, self.melt && !self.worn),
            wear,
        ]
    }
}

/// Where button `i` of the ring for `t` is centred, on the screen. The buttons fan out at
/// [`RADIUS`] and stay inside the target's bounds, but never over the slot the ring opened from:
/// where the bounds are so close that the clamp would put a button on the slot, a press on the
/// slot (the double click that uses the item) would hit the button instead and no use would be
/// sent. Such a button goes outside the bounds, inside `drawer`, and farther out while it still
/// touches the slot.
pub(crate) fn button_center(t: &RingTarget, i: usize, drawer: UiBox) -> (i32, i32) {
    let (cx, cy) = (
        (t.slot.x0 + t.slot.x1) as f32 / 2.0,
        (t.slot.y0 + t.slot.y1) as f32 / 2.0,
    );
    let half = art::RING_ART / 2;
    // The button's place in the fan, as an offset from the way the ring opens.
    let offset = -SPREAD + i as f32 * 2.0 * SPREAD / 4.0;
    let clamp_to = |bounds: UiBox, offset: f32, r: f32| {
        let angle = (t.towards + offset).to_radians();
        let x = (cx + r * angle.cos()).round() as i32;
        let y = (cy + r * angle.sin()).round() as i32;
        (
            x.clamp(bounds.x0 + half, (bounds.x1 - half).max(bounds.x0 + half)),
            y.clamp(bounds.y0 + half, (bounds.y1 - half).max(bounds.y0 + half)),
        )
    };
    let covers = |(x, y): (i32, i32)| {
        x - half < t.slot.x1 && x + half > t.slot.x0 && y - half < t.slot.y1 && y + half > t.slot.y0
    };
    let first = clamp_to(t.bounds, offset, RADIUS);
    if !covers(first) {
        return first;
    }
    // Farther out, then (a narrower fan) nearer the way the ring opens, first inside the bounds
    // and then anywhere in the drawer.
    for bounds in [t.bounds, drawer] {
        for narrow in 0..4 {
            let offset = offset * (1.0 - narrow as f32 * 0.25);
            for step in 0..4 {
                let candidate = clamp_to(bounds, offset, RADIUS + 6.0 * step as f32);
                if !covers(candidate) {
                    return candidate;
                }
            }
        }
    }
    first
}

/// The ring's buttons and what they were laid out for.
#[derive(Debug, Clone)]
pub(crate) struct ItemRing {
    /// The field the buttons are made under (the inventory's).
    parent: usize,
    /// One button per shown place (five), each showing any action's art.
    buttons: Vec<Option<usize>>,
    target: Cell<Option<RingTarget>>,
    /// Each button's action and whether it applies, as last shown.
    shown: Cell<[(RingAction, bool); 5]>,
    lit: Cell<Option<usize>>,
}

impl ItemRing {
    pub(crate) fn make(tree: &mut UiTree, parent: usize, template: Option<usize>) -> Self {
        let buttons = (0..5)
            .map(|i| {
                let button = make_part(
                    tree,
                    parent,
                    template,
                    &format!("ModernItemRing{i}"),
                    art::ring_button(0, 0),
                    None,
                    // In front of the drawer's other parts (a lower z draws in front).
                    0,
                )?;
                keep(tree, button);
                tree.set_bool(button, MOUSE_INVISIBLE, false);
                tree.resize_to(button, art::RING_ART, art::RING_ART);
                tree.set_visible(button, false);
                Some(button)
            })
            .collect();
        Self {
            parent,
            buttons,
            target: Cell::new(None),
            shown: Cell::new([(RingAction::Use, false); 5]),
            lit: Cell::new(None),
        }
    }

    /// The ring shown around `target` (hidden for `None`), each button lit while the pointer is
    /// over it.
    pub(crate) fn show(&self, tree: &mut UiTree, target: Option<RingTarget>) {
        if target != self.target.get() {
            self.target.set(target);
            self.lit.set(None);
            match target {
                Some(t) => self.lay_out(tree, &t),
                None => {
                    for &button in self.buttons.iter().flatten() {
                        if tree.el(button).visible {
                            tree.set_visible(button, false);
                        }
                    }
                }
            }
        }
        if target.is_none() {
            return;
        }
        // The button under the pointer lit, if its action applies.
        let pointer = tree.pointer();
        let lit = self.buttons.iter().position(|b| {
            b.is_some_and(|b| {
                let at = tree.screen_box(b);
                pointer.0 >= at.x0 && pointer.0 <= at.x1 && pointer.1 >= at.y0 && pointer.1 <= at.y1
            })
        });
        if lit != self.lit.get() {
            self.lit.set(lit);
            self.paint(tree);
        }
    }

    fn lay_out(&self, tree: &mut UiTree, t: &RingTarget) {
        let origin = tree.screen_box(self.parent);
        let half = art::RING_ART / 2;
        let actions = t.actions();
        self.shown.set(actions);
        for (i, &button) in self.buttons.iter().enumerate() {
            let Some(button) = button else {
                continue;
            };
            let (x, y) = button_center(t, i, origin);
            tree.move_to(button, x - half - origin.x0, y - half - origin.y0);
            tree.set_visible(button, true);
        }
        self.paint(tree);
    }

    /// Each button's art: its action at rest, lit, or dimmed when it does not apply.
    fn paint(&self, tree: &mut UiTree) {
        let shown = self.shown.get();
        for (i, &button) in self.buttons.iter().enumerate() {
            let (Some(button), Some(&(action, applies))) = (button, shown.get(i)) else {
                continue;
            };
            let index = ACTIONS.iter().position(|&a| a == action).unwrap_or(0);
            let look = if !applies {
                2
            } else if self.lit.get() == Some(i) {
                1
            } else {
                0
            };
            tree.set_image(
                button,
                art::ring_button(index, look),
                crate::compose::DRAW_OVER,
            );
        }
    }

    /// A press on a button whose action applies: that action and the item.
    pub(crate) fn press(&self, element: usize) -> Option<(RingAction, u64)> {
        let target = self.target.get()?;
        let i = self.buttons.iter().position(|b| *b == Some(element))?;
        let (action, applies) = *self.shown.get().get(i)?;
        applies.then_some((action, target.item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(slot: UiBox, bounds: UiBox, towards: f32) -> RingTarget {
        RingTarget {
            item: 1,
            slot,
            bounds,
            towards,
            worn: false,
            split: false,
            melt: false,
            equip: false,
        }
    }

    /// A slot in a grid just one row tall: every button the bounds clamp would sit on the slot
    /// goes clear of it, so a press on the slot reaches the item.
    #[test]
    fn no_button_covers_the_slot_it_opened_from() {
        let half = art::RING_ART / 2;
        let drawer = UiBox {
            x0: 700,
            y0: 170,
            x1: 1020,
            y1: 700,
        };
        let grid = UiBox {
            x0: 710,
            y0: 520,
            x1: 1010,
            y1: 566,
        };
        for (x0, towards) in [(716, -90.0), (800, -90.0), (990, 90.0), (712, 90.0)] {
            let slot = UiBox {
                x0,
                y0: 524,
                x1: x0 + 36,
                y1: 560,
            };
            let t = target(slot, grid, towards);
            for i in 0..5 {
                let (x, y) = button_center(&t, i, drawer);
                let overlap = x - half < slot.x1
                    && x + half > slot.x0
                    && y - half < slot.y1
                    && y + half > slot.y0;
                assert!(
                    !overlap,
                    "button {i} at ({x}, {y}) covers {slot:?} (x0 {x0})"
                );
            }
        }
    }
}
