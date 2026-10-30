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
        let (cx, cy) = (
            (t.slot.x0 + t.slot.x1) as f32 / 2.0,
            (t.slot.y0 + t.slot.y1) as f32 / 2.0,
        );
        let half = art::RING_ART / 2;
        let actions = t.actions();
        self.shown.set(actions);
        for (i, &button) in self.buttons.iter().enumerate() {
            let Some(button) = button else {
                continue;
            };
            let angle = (t.towards - SPREAD + i as f32 * 2.0 * SPREAD / 4.0).to_radians();
            let x = (cx + RADIUS * angle.cos()).round() as i32;
            let y = (cy + RADIUS * angle.sin()).round() as i32;
            let x = x.clamp(
                t.bounds.x0 + half,
                (t.bounds.x1 - half).max(t.bounds.x0 + half),
            );
            let y = y.clamp(
                t.bounds.y0 + half,
                (t.bounds.y1 - half).max(t.bounds.y0 + half),
            );
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
