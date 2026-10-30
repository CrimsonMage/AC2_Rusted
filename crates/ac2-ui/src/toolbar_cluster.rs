//! The toolbar's look: a compact corner cluster in place of retail's full-width bar.
//!
//! Retail's `ToolbarField` was one 798-pixel bar centred on the bottom edge (its art
//! `0x410002E7`, the shortcut bar in its middle socket), the panel buttons in its sockets, the
//! attack buttons and the set disc beside the shortcut bar, and the dismount and consignments
//! buttons on its wings (`gmToolbarUI::SetupChildren` finds them by name). Here the same elements,
//! with the same states, messages and tooltips, are laid out apart:
//!
//! - the bar's art is gone and the field spans the bottom edge without taking the pointer
//!   (`mouse_invisible`), so the world stays open beside its parts;
//! - the shortcut bar (`gmShortcutUI`'s own root) stands alone, centred, with the attack buttons
//!   on its left and the set disc on its right;
//! - the panel buttons sit on a plate in the bottom-right corner (the book's field box,
//!   `0x41007F71`, drawn in nine pieces), one group per side panel: the left panel's children
//!   (options, navigation, examination) and the right's (inventory, skills, quests, social), in
//!   retail's order. A group rests as a stack of its buttons, the first on top, and fans out into
//!   a row while the pointer is over it or presses it; both fold again once the pointer leaves
//!   the plate. The plate keeps its right edge, so an opening group grows away from the pointer;
//! - the dismount and consignments buttons join the plate's right end only while they are shown
//!   (`RecvNotice_UpdateMountState`, `RecvNotice_ShowSaleReminder`); the wings are hidden;
//! - a side panel (`LeftSidePanel`, `RightSidePanel`, anchored to the bottom corners) that would
//!   cover the plate rests on it instead, raised just clear of it.
//!
//! Every element is placed by the client: each keeps its own place as retail's anchoring does
//! for an element whose edges are `0` (`UIElement::UpdateSizeAndPosition`), so a state change
//! does not move it back to its authored box, and [`Cluster::arrange`] lays the whole toolbar out
//! for the display, the shown status buttons and the open groups.

use std::sync::Arc;

use crate::compose::NINE_SLICE;
use crate::element::{UiBox, UiTree};
use crate::layout::{ElementDesc, MediaDesc, Rect};
use crate::token::{tokenize, MOUSE_INVISIBLE};
use crate::toolbar::ToolbarChild;

/// The field's height: the set disc beside the shortcut bar, the tallest part.
pub(crate) const FIELD_HEIGHT: i32 = 48;
/// The plate's art (the book's bottom field box: stone end caps, a dark wood inset) and the
/// corner size its nine pieces keep.
pub(crate) const PLATE_IMAGE: u32 = 0x4100_7F71;
const PLATE_BORDER: i32 = 8;
const PLATE_HEIGHT: i32 = 36;
/// The plate's distance from the display's right and bottom edges.
const MARGIN_RIGHT: i32 = 8;
const MARGIN_BOTTOM: i32 = 6;
/// The plate's inner padding at each end, and the space between the two groups.
const PAD: i32 = 10;
const GAP: i32 = 10;
/// A panel button's size (its art, `0x410009CA` and the like), its step in an open row, and the
/// part of each button a resting stack shows behind the one above it.
const ORB: (i32, i32) = (27, 28);
const PITCH: i32 = 30;
const STACK_STEP: i32 = 5;
/// A status button's size (`0x4100703D`, `0x41008C71`) and its step.
const STATUS: (i32, i32) = (21, 20);
const STATUS_PITCH: i32 = 24;
/// The space a side panel keeps over the plate when it would cover it.
const PANEL_GAP: i32 = 2;
/// The attack buttons (`0x41000CEB`, 41 pixels round), side by side with a gap between them
/// (retail's sockets overlapped them), and the set disc's halves (48 x 24 each).
const ATTACK: i32 = 41;
const ATTACK_GAP: i32 = 4;
const DISC: (i32, i32) = (48, 24);
/// The gap between the shortcut bar and the parts beside it.
const BESIDE_GAP: i32 = 3;
/// The z the buttons start from (a lower z draws in front), and the plate's, behind them.
const BUTTON_Z: i32 = 2;
const PLATE_Z: i32 = 100;
/// The client's own id for the plate, named so it never reads as a layout element's name.
const PLATE_ID: &str = "ToolbarClusterPlate";

/// The two groups, by side panel.
pub(crate) const LEFT_GROUP: [ToolbarChild; 3] = [
    ToolbarChild::Option,
    ToolbarChild::Navigation,
    ToolbarChild::Examination,
];
pub(crate) const RIGHT_GROUP: [ToolbarChild; 4] = [
    ToolbarChild::Inventory,
    ToolbarChild::Skill,
    ToolbarChild::Quest,
    ToolbarChild::Social,
];

/// Where everything goes, relative to the field, for a field `width` wide.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Plan {
    pub plate: UiBox,
    /// Each group's buttons' boxes and z, in the group's order.
    pub left: Vec<(UiBox, i32)>,
    pub right: Vec<(UiBox, i32)>,
    /// The part of the plate each group answers the pointer in.
    pub spans: [UiBox; 2],
    /// The consignments and dismount buttons, when shown.
    pub consignments: Option<UiBox>,
    pub dismount: Option<UiBox>,
    /// The stop and start attack buttons and the set disc's halves (up, down).
    pub stop_attack: UiBox,
    pub start_attack: UiBox,
    pub set_up: UiBox,
    pub set_down: UiBox,
}

fn sized(x: i32, y: i32, (w, h): (i32, i32)) -> UiBox {
    UiBox::new(x, y, x + w - 1, y + h - 1)
}

/// A group's row: open (one button every [`PITCH`]) or resting (a stack, the first on top).
fn group(count: usize, x: i32, y: i32, open: bool) -> (Vec<(UiBox, i32)>, i32) {
    let step = if open { PITCH } else { STACK_STEP };
    let boxes = (0..count as i32)
        .map(|i| {
            let z = if open { BUTTON_Z } else { BUTTON_Z + i };
            (sized(x + i * step, y, ORB), z)
        })
        .collect();
    (boxes, ORB.0 + step * (count as i32 - 1))
}

/// The toolbar laid out for a field `width` wide ([`FIELD_HEIGHT`] tall) whose shortcut bar spans
/// `shortcut` (its left and right x on the field), with the groups `open` (left, right) and the
/// status buttons `shown` (consignments, dismount).
pub(crate) fn plan(width: i32, shortcut: (i32, i32), open: [bool; 2], shown: [bool; 2]) -> Plan {
    let plate_x1 = width - 1 - MARGIN_RIGHT;
    let plate_y1 = FIELD_HEIGHT - 1 - MARGIN_BOTTOM;
    let plate_y0 = plate_y1 - PLATE_HEIGHT + 1;
    // The status buttons at the plate's right end, the dismount button outermost.
    let status_y = plate_y0 + (PLATE_HEIGHT - STATUS.1) / 2;
    let mut status_x = plate_x1 - PAD - STATUS.0 + 1;
    let mut place_status = |shown: bool| {
        shown.then(|| {
            let at = sized(status_x, status_y, STATUS);
            status_x -= STATUS_PITCH;
            at
        })
    };
    let dismount = place_status(shown[1]);
    let consignments = place_status(shown[0]);
    // The groups end where the status buttons start (a gap before them when any shows).
    let groups_x1 = if dismount.is_some() || consignments.is_some() {
        status_x + STATUS.0 - GAP / 2
    } else {
        plate_x1 - PAD + 1
    };
    let rows_y = plate_y0 + (PLATE_HEIGHT - ORB.1) / 2;
    let (_, right_width) = group(RIGHT_GROUP.len(), 0, 0, open[1]);
    let (_, left_width) = group(LEFT_GROUP.len(), 0, 0, open[0]);
    let right_x = groups_x1 - right_width;
    let left_x = right_x - GAP - left_width;
    let plate_x0 = left_x - PAD;
    let (left, _) = group(LEFT_GROUP.len(), left_x, rows_y, open[0]);
    let (right, _) = group(RIGHT_GROUP.len(), right_x, rows_y, open[1]);
    let split = right_x - GAP / 2;
    let spans = [
        UiBox::new(plate_x0, plate_y0, split - 1, plate_y1),
        UiBox::new(split, plate_y0, groups_x1 + GAP / 2 - 1, plate_y1),
    ];
    let bottom = FIELD_HEIGHT - 1;
    let start_x = shortcut.0 - BESIDE_GAP - ATTACK;
    let disc_x = shortcut.1 + BESIDE_GAP + 1;
    Plan {
        plate: UiBox::new(plate_x0, plate_y0, plate_x1, plate_y1),
        left,
        right,
        spans,
        consignments,
        dismount,
        stop_attack: sized(
            start_x - ATTACK - ATTACK_GAP,
            bottom + 1 - ATTACK,
            (ATTACK, ATTACK),
        ),
        start_attack: sized(start_x, bottom + 1 - ATTACK, (ATTACK, ATTACK)),
        set_up: sized(disc_x, bottom + 1 - 2 * DISC.1, DISC),
        set_down: sized(disc_x, bottom + 1 - DISC.1, DISC),
    }
}

/// The groups the pointer opens: the group it is over opens, an open group stays open while the
/// pointer is on the plate, and both close once it leaves (`at` on the field).
pub(crate) fn hover(plan: &Plan, open: [bool; 2], at: (i32, i32)) -> [bool; 2] {
    let inside = |b: &UiBox| (b.x0..=b.x1).contains(&at.0) && (b.y0..=b.y1).contains(&at.1);
    if !inside(&plan.plate) {
        return [false; 2];
    }
    [
        open[0] || inside(&plan.spans[0]),
        open[1] || inside(&plan.spans[1]),
    ]
}

/// The toolbar field's elements the cluster lays out.
#[derive(Debug, Clone)]
pub(crate) struct Cluster {
    field: usize,
    plate: Option<usize>,
    left: Vec<Option<usize>>,
    right: Vec<Option<usize>>,
    consignments: Option<usize>,
    dismount: Option<usize>,
    stop_attack: Option<usize>,
    start_attack: Option<usize>,
    set_up: Option<usize>,
    set_down: Option<usize>,
}

/// A copy of `desc` whose edges are all `0`: once sized, the element keeps where it is put.
fn kept_in_place(desc: &ElementDesc) -> ElementDesc {
    let mut desc = desc.clone();
    desc.edges = [0; 4];
    desc
}

impl Cluster {
    /// The field made over: its art cleared and its box the bottom edge's full width, each child
    /// kept where it is put, the wings hidden, the plate made. Then laid out.
    pub(crate) fn adopt(tree: &mut UiTree, field: usize) -> Self {
        let button = |tree: &UiTree, name: &str| tree.child_recursive(field, tokenize(name));
        // The children kept in place: their descriptions swapped in the field's lists too, so the
        // field's own update (`UIElement::UpdateChild`) finds them unchanged.
        let mut swapped: Vec<(Arc<ElementDesc>, Arc<ElementDesc>)> = Vec::new();
        for child in tree.children(field).to_vec() {
            let old = Arc::clone(&tree.el(child).reference);
            let new = Arc::new(kept_in_place(&old));
            let el = tree.el_mut(child);
            if Arc::ptr_eq(&el.desc, &old) {
                el.desc = Arc::clone(&new);
            }
            el.reference = Arc::clone(&new);
            swapped.push((old, new));
        }
        let (width, height) = {
            let layout = &tree.el(field).layout;
            (i32::from(layout.width), i32::from(layout.height))
        };
        let field_box = Rect {
            x0: 0,
            y0: (height - FIELD_HEIGHT) as i16,
            x1: (width - 1) as i16,
            y1: (height - 1) as i16,
        };
        let remake = |desc: &ElementDesc| {
            let mut desc = desc.clone();
            desc.rect = field_box;
            // Left kept, top, right and bottom with the display's far side: the bottom edge's
            // full width at any size.
            desc.edges = [1, 2, 1, 1];
            for state in &mut desc.states {
                if state.rect.x0 != -1 {
                    state.rect = field_box;
                }
                state
                    .media
                    .retain(|m| !matches!(m, MediaDesc::Image { .. }));
                for listed in &mut state.children {
                    if let Some((_, new)) = swapped.iter().find(|(old, _)| Arc::ptr_eq(old, listed))
                    {
                        *listed = Arc::clone(new);
                    }
                }
            }
            Arc::new(desc)
        };
        let (reference, desc) = {
            let el = tree.el(field);
            (Arc::clone(&el.reference), Arc::clone(&el.desc))
        };
        let new_reference = remake(&reference);
        let new_desc = if Arc::ptr_eq(&reference, &desc) {
            Arc::clone(&new_reference)
        } else {
            remake(&desc)
        };
        let el = tree.el_mut(field);
        el.reference = new_reference;
        el.desc = new_desc;
        tree.clear_image(field);
        tree.set_bool(field, MOUSE_INVISIBLE, true);
        tree.update_size_and_position(field);
        for wing in ["LeftWingField", "RightWingField"] {
            if let Some(wing) = button(tree, wing) {
                tree.set_visible(wing, false);
            }
        }
        let template = button(tree, "LeftWingField");
        let plate = Self::make_plate(tree, field, template);
        let names = |group: &[ToolbarChild]| {
            group
                .iter()
                .map(|c| button(tree, &format!("{}Button", c.name())))
                .collect()
        };
        // The set disc's halves are the field's two buttons that are not named (the up half above
        // the down half, which holds `SetNumberDisplay`).
        let button_kind = button(tree, "StartAttackButton").map(|b| tree.el(b).desc.kind);
        let mut disc: Vec<usize> = tree
            .children(field)
            .iter()
            .copied()
            .filter(|&c| Some(tree.el(c).desc.kind) == button_kind)
            .filter(|&c| NAMED.iter().all(|n| tokenize(n) != tree.el(c).id()))
            .collect();
        disc.sort_by_key(|&c| tree.el(c).rect.y0);
        let cluster = Self {
            field,
            plate,
            left: names(&LEFT_GROUP),
            right: names(&RIGHT_GROUP),
            consignments: button(tree, "ConsignmentsButton"),
            dismount: button(tree, "DismountButton"),
            stop_attack: button(tree, "StopAttackButton"),
            start_attack: button(tree, "StartAttackButton"),
            set_up: disc.first().copied(),
            set_down: disc.get(1).copied(),
        };
        cluster.arrange(tree, [false; 2]);
        cluster
    }

    /// The plate: a copy of `template` (a plain field of the toolbar layout) with no states, its
    /// art [`PLATE_IMAGE`] in nine pieces.
    fn make_plate(tree: &mut UiTree, field: usize, template: Option<usize>) -> Option<usize> {
        let template = template?;
        let layout = Arc::clone(&tree.el(field).layout);
        let mut desc = kept_in_place(&tree.el(template).reference);
        desc.id = tokenize(PLATE_ID);
        desc.states.clear();
        let plate = tree.create_element(layout, Arc::new(desc), false)?;
        tree.set_parent(plate, Some(field));
        tree.set_image(plate, PLATE_IMAGE, 2);
        tree.set_int(plate, NINE_SLICE, PLATE_BORDER);
        tree.set_z_level(plate, PLATE_Z);
        Some(plate)
    }

    /// The plan for the field as it is now: its width, the shortcut bar's span over it (a bar of
    /// retail's width centred when there is none), the status buttons shown.
    pub(crate) fn plan(&self, tree: &UiTree, open: [bool; 2]) -> Plan {
        let field = tree.screen_box(self.field);
        let shortcut = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| tree.el(r).id() == tokenize("ShortcutField"))
            .map(|r| tree.screen_box(r))
            .map_or_else(
                || {
                    let middle = field.width() / 2;
                    (middle - 164, middle + 163)
                },
                |b| (b.x0 - field.x0, b.x1 - field.x0),
            );
        let shown = |e: Option<usize>| e.and_then(|e| tree.get(e)).is_some_and(|e| e.visible);
        plan(
            field.width(),
            shortcut,
            open,
            [shown(self.consignments), shown(self.dismount)],
        )
    }

    /// Every part put where [`plan`] says.
    pub(crate) fn arrange(&self, tree: &mut UiTree, open: [bool; 2]) {
        let plan = self.plan(tree, open);
        let put = |tree: &mut UiTree, element: Option<usize>, at: UiBox, z: Option<i32>| {
            let Some(element) = element else {
                return;
            };
            if tree.el(element).rect != at {
                tree.resize_to(element, at.width(), at.height());
                tree.move_to(element, at.x0, at.y0);
            }
            if let Some(z) = z {
                tree.set_z_level(element, z);
            }
        };
        put(tree, self.plate, plan.plate, None);
        for (elements, boxes) in [(&self.left, &plan.left), (&self.right, &plan.right)] {
            for (&element, &(at, z)) in elements.iter().zip(boxes) {
                put(tree, element, at, Some(z));
            }
        }
        if let Some(at) = plan.consignments {
            put(tree, self.consignments, at, None);
        }
        if let Some(at) = plan.dismount {
            put(tree, self.dismount, at, None);
        }
        put(tree, self.stop_attack, plan.stop_attack, None);
        put(tree, self.start_attack, plan.start_attack, None);
        put(tree, self.set_up, plan.set_up, None);
        put(tree, self.set_down, plan.set_down, None);
        self.raise_panels(tree);
    }

    /// A side panel that would cover the plate at its widest (both groups open, both status
    /// buttons shown) rests on it, so the panel stays put while the groups open and fold.
    fn raise_panels(&self, tree: &mut UiTree) {
        let field = tree.screen_box(self.field);
        let plate = plan(field.width(), (0, 0), [true; 2], [true; 2])
            .plate
            .offset(field.x0, field.y0);
        for name in ["LeftSidePanel", "RightSidePanel"] {
            let Some(panel) = tree
                .roots()
                .iter()
                .copied()
                .find(|&r| tree.el(r).id() == tokenize(name))
            else {
                continue;
            };
            let at = tree.screen_box(panel);
            let covers = at.x0 <= plate.x1 && at.x1 >= plate.x0 && at.y1 >= plate.y0;
            if covers {
                tree.move_to(panel, at.x0, plate.y0 - PANEL_GAP - at.height());
            }
        }
    }

    /// The pointer's place on the field.
    pub(crate) fn pointer(&self, tree: &UiTree) -> (i32, i32) {
        let field = tree.screen_box(self.field);
        (tree.pointer.0 - field.x0, tree.pointer.1 - field.y0)
    }
}

/// A panel button drawn lit (its `normal_rollover` art) or as its state draws it. The quest
/// button is a field over its art's button, which is lit in its place.
pub(crate) fn light(tree: &mut UiTree, button: usize, lit: bool) {
    let art = if tree.el(button).image.is_some() {
        button
    } else {
        match tree.children(button).first() {
            Some(&child) => child,
            None => return,
        }
    };
    let rollover = crate::element::state_of(&tree.el(art).desc, tokenize("normal_rollover"))
        .and_then(|state| {
            state.media.iter().find_map(|m| match m {
                MediaDesc::Image {
                    file, draw_mode, ..
                } => Some((*file, *draw_mode)),
                _ => None,
            })
        });
    let Some((file, draw_mode)) = rollover else {
        return;
    };
    let showing = tree.el(art).image.map(|i| i.file);
    if lit && showing != Some(file) {
        tree.set_image(art, file, draw_mode);
    } else if !lit && showing == Some(file) && !tree.el(art).mouse_over {
        tree.reset_media(art);
    }
}

/// The field's children found by name; the others among its buttons are the set disc's halves.
const NAMED: [&str; 11] = [
    "StopAttackButton",
    "StartAttackButton",
    "QuestButton",
    "SocialButton",
    "ConsignmentsButton",
    "DismountButton",
    "OptionButton",
    "NavigationButton",
    "InventoryButton",
    "ExaminationButton",
    "SkillButton",
];
