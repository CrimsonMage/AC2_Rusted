//! The modern HUD's toolbar (AGENTS.md: the HUD's two styles; the user's chosen design,
//! "variant D"). Retail's `ToolbarField` was one 798-pixel bar centred on the bottom edge with the
//! shortcut bar in its middle socket. Here the same elements, with the same states, messages and
//! tooltips, are laid out as the design's two parts:
//!
//! - the strip, 6 pixels over the bottom edge, centred in the room left of the inventory drawer, on
//!   a dark teal fade: the attack buttons side
//!   by side in round wells, the set chip (an arrow, the set's number, an arrow) and the shortcut
//!   bar's rows of glass wells (`modern_strip.rs`);
//! - the dock in the bottom-right corner, 10 pixels in and 8 up: a teal orb ringed in bronze with
//!   a pip per panel (gold while its panel shows); the panel buttons fan out of it in round wells
//!   on a bevelled backing while the pointer is over the dock, or always once the orb is pressed
//!   (pinned), the left panel's three, a line, then the right's four; the dismount and
//!   consignments buttons stand beside the orb on their own backings while they are shown.
//!
//! Every element keeps its place as retail's anchoring keeps an element whose edges are `0`
//! (`UIElement::UpdateSizeAndPosition`), so a state change does not move it back; [`Dock::arrange`]
//! lays everything out for the display. The art is `modern_art`'s.

use std::cell::Cell;
use std::sync::Arc;

use crate::compose::{DRAW_OVER, NINE_SLICE};
use crate::element::{UiBox, UiTree};
use crate::layout::{ElementDesc, MediaDesc, Rect};
use crate::modern_art as art;
use crate::token::{tokenize, MOUSE_INVISIBLE};

/// The field's height: the dock's orb and its room under it.
const FIELD_HEIGHT: i32 = 72;
/// The strip's padding (left, top, right, bottom), the gap between its parts, and its distance
/// from the bottom edge. The rows' last column carries the slots' 3-pixel gap, so the right
/// padding is 3 less than the design's 8.
const STRIP_PAD: (i32, i32, i32, i32) = (6, 4, 5, 4);
const STRIP_GAP: i32 = 6;
const STRIP_BOTTOM: i32 = 6;
/// The room the design keeps clear for the inventory drawer on the right when centring the strip.
const DRAWER_ROOM: i32 = 330;
/// The attack buttons' gap, and the set chip's width and its parts' heights.
const ATTACK_GAP: i32 = 4;
const CHIP_WIDTH: i32 = 34;
const CHIP_ARROW: i32 = 10;
const CHIP_NUMBER: i32 = 16;
/// The dock's distance from the right and bottom edges, its gap between parts, the orb's box,
/// the fan's padding (sides, top and bottom) and its wells' gap, and the separator's margins.
const DOCK_RIGHT: i32 = 10;
const DOCK_BOTTOM: i32 = 8;
const DOCK_GAP: i32 = 6;
const FAN_PAD: (i32, i32) = (8, 3);
const FAN_GAP: i32 = 5;
const SEP_MARGIN: i32 = 3;
/// The pips' angles (degrees clockwise from east) and their distance from the orb's centre, in
/// the order of [`crate::ToolbarChild::ALL`].
const PIP_ANGLES: [f32; 7] = [200.0, 240.0, 280.0, -70.0, -25.0, 20.0, 65.0];
const PIP_RADIUS: f32 = 25.0;
/// A status button's pill: its backing's size and the button's inset in it.
const PILL: (i32, i32) = (33, 26);
const PILL_INSET: (i32, i32) = (5, 3);
/// The z the parts take (a lower z draws in front): the buttons in front, their wells behind them,
/// the backings behind those.
const BUTTON_Z: i32 = 2;
const WELL_Z: i32 = 20;
const BACKING_Z: i32 = 60;
const FADE_Z: i32 = 100;

/// The field's children found by name; the others among its buttons are the set chip's halves.
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

/// A copy of `desc` whose edges are all `0`: once sized, the element keeps where it is put.
fn kept_in_place(desc: &ElementDesc) -> ElementDesc {
    let mut desc = desc.clone();
    desc.edges = [0; 4];
    desc
}

fn sized(x: i32, y: i32, (w, h): (i32, i32)) -> UiBox {
    UiBox::new(x, y, x + w - 1, y + h - 1)
}

/// The toolbar field's elements the design lays out, and the parts it adds.
#[derive(Debug, Clone)]
pub(crate) struct Dock {
    field: usize,
    fade: Option<usize>,
    bevel: Option<usize>,
    separator: Option<usize>,
    orb: Option<usize>,
    /// Per panel, in [`crate::ToolbarChild::ALL`]'s order: its button, the well behind it, its pip.
    buttons: Vec<Option<usize>>,
    wells: Vec<Option<usize>>,
    pips: Vec<Option<usize>>,
    /// The consignments and dismount buttons and the pills behind them.
    status: [Option<usize>; 2],
    pills: [Option<usize>; 2],
    start_attack: Option<usize>,
    stop_attack: Option<usize>,
    set_up: Option<usize>,
    set_down: Option<usize>,
    set_number: Option<usize>,
    /// The fan is open (the pointer over the dock, or pinned) and pinned (the orb pressed).
    open: Cell<bool>,
    pinned: Cell<bool>,
}

/// Where the parts go, on the field.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Plan {
    pub fade: UiBox,
    /// The shortcut bar's rows (the bar's own root) on the field.
    pub rows: (i32, i32),
    pub start_attack: UiBox,
    pub stop_attack: UiBox,
    pub set_up: UiBox,
    pub set_down: UiBox,
    pub set_number: UiBox,
    pub orb: UiBox,
    pub pips: Vec<UiBox>,
    pub fan: UiBox,
    /// Each panel's well (the button's box too), and the separator.
    pub wells: Vec<UiBox>,
    pub separator: UiBox,
    /// The consignments and dismount pills, when shown.
    pub pills: [Option<UiBox>; 2],
}

/// The design laid out on a field `width` wide ([`FIELD_HEIGHT`] tall) for a shortcut bar of
/// `rows` (width, height) and the status buttons `shown` (consignments, dismount).
pub(crate) fn plan(width: i32, rows: (i32, i32), shown: [bool; 2]) -> Plan {
    let bottom = FIELD_HEIGHT - 1;
    // The strip.
    let (pl, pt, pr, pb) = STRIP_PAD;
    let attack_width = 2 * art::ATTACK_WELL_SIZE + ATTACK_GAP;
    let strip_w = pl + attack_width + STRIP_GAP + CHIP_WIDTH + STRIP_GAP + rows.0 + pr;
    let strip_h = pt + rows.1 + pb;
    // Centred in the room left of the inventory drawer, as the design places it.
    let strip_x0 = (width - DRAWER_ROOM) / 2 - strip_w / 2;
    let strip_y1 = bottom - STRIP_BOTTOM;
    let strip_y0 = strip_y1 - strip_h + 1;
    let room = art::GLOW_ROOM;
    let attack_y = strip_y0 + (strip_h - art::ATTACK_WELL_SIZE) / 2 - room;
    let start_x = strip_x0 + pl - room;
    let stop_x = start_x + art::ATTACK_WELL_SIZE + ATTACK_GAP;
    let chip_x = strip_x0 + pl + attack_width + STRIP_GAP;
    let chip_bottom = strip_y1 - pb;
    let set_down = sized(
        chip_x,
        chip_bottom - (CHIP_NUMBER + CHIP_ARROW) + 1,
        (CHIP_WIDTH, CHIP_NUMBER + CHIP_ARROW),
    );
    let set_up = sized(chip_x, set_down.y0 - CHIP_ARROW, (CHIP_WIDTH, CHIP_ARROW));
    let rows_at = (chip_x + CHIP_WIDTH + STRIP_GAP, strip_y0 + pt);
    // The dock: the orb in the corner, the pills to its left, the fan to theirs.
    let orb = sized(
        width - DOCK_RIGHT - art::ORB_ART,
        bottom - DOCK_BOTTOM - art::ORB_ART + 1,
        (art::ORB_ART, art::ORB_ART),
    );
    let mid_y = (orb.y0 + orb.y1) / 2;
    let (cx, cy) = (orb.x0 as f32 + 29.0, orb.y0 as f32 + 29.0);
    let pips = PIP_ANGLES
        .iter()
        .map(|a| {
            let (s, c) = a.to_radians().sin_cos();
            let (x, y) = (cx + PIP_RADIUS * c, cy + PIP_RADIUS * s);
            let half = art::PIP_ART as f32 / 2.0;
            sized(
                (x - half).round() as i32,
                (y - half).round() as i32,
                (art::PIP_ART, art::PIP_ART),
            )
        })
        .collect();
    let mut x1 = orb.x0 + 10 - DOCK_GAP;
    let mut pills = [None, None];
    for (i, &on) in shown.iter().enumerate().rev() {
        if on {
            let pill = sized(x1 - PILL.0 + 1, mid_y - PILL.1 / 2, PILL);
            x1 = pill.x0 - DOCK_GAP;
            pills[i] = Some(pill);
        }
    }
    let well = art::PANEL_WELL_SIZE;
    let fan_w = 2 * FAN_PAD.0 + 7 * well + 6 * FAN_GAP + 2 * SEP_MARGIN + 1;
    let fan_h = 2 * FAN_PAD.1 + well;
    let fan = sized(x1 - fan_w + 1, mid_y - fan_h / 2, (fan_w, fan_h));
    let mut wells = Vec::with_capacity(7);
    let mut x = fan.x0 + FAN_PAD.0;
    let mut separator = UiBox::default();
    for i in 0..7 {
        if i == 3 {
            // The line between the left panel's buttons and the right's.
            separator = sized(x - FAN_GAP + SEP_MARGIN, mid_y - 13, (1, 26));
            x += 2 * SEP_MARGIN + 1;
        }
        wells.push(sized(
            x - room,
            fan.y0 + FAN_PAD.1 - room,
            (art::PANEL_ART, art::PANEL_ART),
        ));
        x += well + FAN_GAP;
    }
    Plan {
        fade: UiBox::new(strip_x0, strip_y0, strip_x0 + strip_w - 1, strip_y1),
        rows: rows_at,
        start_attack: sized(start_x, attack_y, (art::ATTACK_ART, art::ATTACK_ART)),
        stop_attack: sized(stop_x, attack_y, (art::ATTACK_ART, art::ATTACK_ART)),
        set_up,
        set_down,
        // The number's text is right-aligned in its box: the box ends past the chip's middle so
        // a one- or two-digit set sits centred over the arrow.
        set_number: sized(0, 0, (CHIP_WIDTH / 2 + 5, CHIP_NUMBER)),
        orb,
        pips,
        fan,
        wells,
        separator,
        pills,
    }
}

impl Dock {
    /// The field made over: its art cleared and its box the bottom edge's full width, each child
    /// kept where it is put, the wings hidden, the design's parts made. Then laid out.
    pub(crate) fn adopt(tree: &mut UiTree, field: usize) -> Self {
        adopt_field(tree, field);
        // The design lays the toolbar and the shortcut bar out itself, by the display's edges:
        // neither is a window whose saved place would move it.
        tree.set_overlay_root(field);
        if let Some(bar) = Self::shortcut_root(tree) {
            tree.set_overlay_root(bar);
        }
        let find = |tree: &UiTree, name: &str| tree.child_recursive(field, tokenize(name));
        for wing in ["LeftWingField", "RightWingField"] {
            if let Some(wing) = find(tree, wing) {
                tree.set_visible(wing, false);
            }
        }
        let template = find(tree, "LeftWingField");
        let part = |tree: &mut UiTree, id: &str, image: u32, nine: Option<i32>, z: i32| {
            make_part(tree, field, template, id, image, nine, z)
        };
        let fade = part(
            tree,
            "ModernStripFade",
            art::STRIP_FADE,
            Some(art::FADE_BORDER),
            FADE_Z,
        );
        let bevel = part(
            tree,
            "ModernFanBevel",
            art::BEVEL,
            Some(art::BEVEL_BORDER),
            BACKING_Z,
        );
        let separator = part(tree, "ModernFanSeparator", art::SEPARATOR, None, WELL_Z);
        let orb = part(tree, "ModernDockOrb", art::ORB, None, WELL_Z);
        let buttons: Vec<Option<usize>> = crate::ToolbarChild::ALL
            .iter()
            .map(|c| find(tree, &format!("{}Button", c.name())))
            .collect();
        let wells = (0..7)
            .map(|i| {
                part(
                    tree,
                    &format!("ModernPanelWell{i}"),
                    art::PANEL_WELL,
                    None,
                    WELL_Z,
                )
            })
            .collect();
        let pips = (0..7)
            .map(|i| part(tree, &format!("ModernDockPip{i}"), art::PIP, None, BUTTON_Z))
            .collect();
        let pills = [
            part(tree, "ModernSalePill", art::BEVEL, Some(12), WELL_Z),
            part(tree, "ModernDismountPill", art::BEVEL, Some(12), WELL_Z),
        ];
        // The set chip's halves are the field's two buttons that are not named (the up half above
        // the down half, which holds `SetNumberDisplay`).
        let button_kind = find(tree, "StartAttackButton").map(|b| tree.el(b).desc.kind);
        let mut chip: Vec<usize> = tree
            .children(field)
            .iter()
            .copied()
            .filter(|&c| Some(tree.el(c).desc.kind) == button_kind)
            .filter(|&c| NAMED.iter().all(|n| tokenize(n) != tree.el(c).id()))
            .collect();
        chip.sort_by_key(|&c| tree.el(c).rect.y0);
        let dock = Self {
            field,
            fade,
            bevel,
            separator,
            orb,
            buttons,
            wells,
            pips,
            status: [
                find(tree, "ConsignmentsButton"),
                find(tree, "DismountButton"),
            ],
            pills,
            start_attack: find(tree, "StartAttackButton"),
            stop_attack: find(tree, "StopAttackButton"),
            set_up: chip.first().copied(),
            set_down: chip.get(1).copied(),
            set_number: find(tree, "SetNumberDisplay"),
            open: Cell::new(false),
            pinned: Cell::new(false),
        };
        for &button in dock.buttons.iter().flatten() {
            set_depth(tree, button, BUTTON_Z);
        }
        // The buttons' art is the design's (`modern_art` under their own DIDs), composited over.
        let mut drawn: Vec<usize> = dock.buttons.iter().flatten().copied().collect();
        drawn.extend(
            dock.buttons
                .iter()
                .flatten()
                .flat_map(|&b| tree.children(b).to_vec()),
        );
        drawn.extend(
            [
                dock.start_attack,
                dock.stop_attack,
                dock.set_up,
                dock.set_down,
            ]
            .into_iter()
            .flatten(),
        );
        for element in drawn {
            draw_over(tree, element);
        }
        // The set's number in the design's gold.
        if let Some(number) = dock.set_number {
            for (channel, value) in ["red", "green", "blue"].iter().zip(art::GOLD_RGB) {
                tree.set_float(number, tokenize(&format!("font_color_{channel}")), value);
            }
            tree.set_font_num_and_color(number, 0, 0);
        }
        dock
    }

    /// The shortcut bar's root, if the HUD has one.
    fn shortcut_root(tree: &UiTree) -> Option<usize> {
        tree.roots()
            .iter()
            .copied()
            .find(|&r| tree.el(r).id() == tokenize("ShortcutField"))
    }

    /// The plan for the field as it is now.
    pub(crate) fn plan(&self, tree: &UiTree) -> Plan {
        let field = tree.screen_box(self.field);
        let rows = Self::shortcut_root(tree)
            .map(|r| tree.screen_box(r))
            .map_or((350, 35), |b| (b.width(), b.height()));
        let shown = |e: Option<usize>| e.and_then(|e| tree.get(e)).is_some_and(|e| e.visible);
        plan(
            field.width(),
            rows,
            [shown(self.status[0]), shown(self.status[1])],
        )
    }

    /// Every part put where [`plan`] says, the fan shown while open, the wells and pips lit for
    /// the `shown` panels (in [`crate::ToolbarChild::ALL`]'s order).
    pub(crate) fn arrange(&self, tree: &mut UiTree, shown: [bool; 7]) {
        let plan = self.plan(tree);
        let field = tree.screen_box(self.field);
        put(tree, self.fade, plan.fade);
        put(tree, self.start_attack, plan.start_attack);
        put(tree, self.stop_attack, plan.stop_attack);
        put(tree, self.set_up, plan.set_up);
        put(tree, self.set_down, plan.set_down);
        put(tree, self.set_number, plan.set_number);
        if let Some(root) = Self::shortcut_root(tree) {
            let at = (field.x0 + plan.rows.0, field.y0 + plan.rows.1);
            let now = tree.screen_box(root);
            if (now.x0, now.y0) != at {
                tree.move_to(root, at.0, at.1);
            }
        }
        put(tree, self.orb, plan.orb);
        for (i, (&pip, at)) in self.pips.iter().zip(&plan.pips).enumerate() {
            put(tree, pip, *at);
            if let Some(pip) = pip {
                tree.set_image(
                    pip,
                    if shown[i] { art::PIP_ON } else { art::PIP },
                    DRAW_OVER,
                );
            }
        }
        let open = self.open.get();
        put(tree, self.bevel, plan.fan);
        put(tree, self.separator, plan.separator);
        for part in [self.bevel, self.separator].into_iter().flatten() {
            tree.set_visible(part, open);
        }
        for (i, at) in plan.wells.iter().enumerate() {
            put(tree, self.wells[i], *at);
            put_button(tree, self.buttons[i], *at);
            if let Some(well) = self.wells[i] {
                let image = if shown[i] {
                    art::PANEL_WELL_ON
                } else {
                    art::PANEL_WELL
                };
                tree.set_image(well, image, DRAW_OVER);
                tree.set_visible(well, open);
            }
            if let Some(button) = self.buttons[i] {
                tree.set_visible(button, open);
            }
        }
        for i in 0..2 {
            let pill = plan.pills[i];
            if let Some(element) = self.pills[i] {
                tree.set_visible(element, pill.is_some());
            }
            if let Some(at) = pill {
                put(tree, self.pills[i], at);
                let inset = PILL_INSET;
                put(
                    tree,
                    self.status[i],
                    sized(at.x0 + inset.0, at.y0 + inset.1, (21, 20)),
                );
            }
        }
    }

    /// The pointer's place on the field.
    fn pointer(&self, tree: &UiTree) -> (i32, i32) {
        let field = tree.screen_box(self.field);
        (tree.pointer.0 - field.x0, tree.pointer.1 - field.y0)
    }

    /// The pointer moved (`pressed`: the left button went down): the fan opens over the dock and
    /// folds once the pointer leaves it, unless pinned; a press on the orb pins or unpins it.
    /// Whether the fan's state changed.
    pub(crate) fn pointer_at(&self, tree: &UiTree, pressed: bool) -> bool {
        let plan = self.plan(tree);
        let at = self.pointer(tree);
        let inside = |b: &UiBox| (b.x0..=b.x1).contains(&at.0) && (b.y0..=b.y1).contains(&at.1);
        let on_orb = {
            let dx = at.0 - (plan.orb.x0 + 29);
            let dy = at.1 - (plan.orb.y0 + 29);
            dx * dx + dy * dy <= 19 * 19
        };
        if pressed && on_orb {
            self.pinned.set(!self.pinned.get());
        }
        let over = on_orb || (self.open.get() && inside(&plan.fan)) || inside(&plan.orb);
        let open = self.pinned.get() || over;
        let changed = open != self.open.get();
        self.open.set(open);
        changed || (pressed && on_orb)
    }

    /// Whether the fan is open.
    pub(crate) fn is_open(&self) -> bool {
        self.open.get()
    }
}

/// `element`'s state images drawn with [`DRAW_OVER`] (its description rewritten, so every state
/// the button takes draws so), and its image now.
fn draw_over(tree: &mut UiTree, element: usize) {
    let over = |desc: &ElementDesc| {
        let mut desc = desc.clone();
        for state in &mut desc.states {
            for media in &mut state.media {
                if let MediaDesc::Image { draw_mode, .. } = media {
                    *draw_mode = DRAW_OVER;
                }
            }
        }
        Arc::new(desc)
    };
    // A reference draws its referent's states (`desc`); its own description is rewritten too.
    let (reference, desc) = {
        let el = tree.el(element);
        (Arc::clone(&el.reference), Arc::clone(&el.desc))
    };
    let new_reference = over(&reference);
    let new_desc = if Arc::ptr_eq(&reference, &desc) {
        Arc::clone(&new_reference)
    } else {
        over(&desc)
    };
    let el = tree.el_mut(element);
    el.reference = new_reference;
    el.desc = new_desc;
    if let Some(image) = tree.el(element).image {
        tree.set_image(element, image.file, DRAW_OVER);
    }
}

/// `element` at depth `z`, in its description too, so the anchoring keeps it there.
fn set_depth(tree: &mut UiTree, element: usize, z: i32) {
    let mut desc = (*tree.el(element).reference).clone();
    desc.z_level = z as u16;
    for state in &mut desc.states {
        state.z_level = z as u16;
    }
    let desc = Arc::new(desc);
    let el = tree.el_mut(element);
    if Arc::ptr_eq(&el.desc, &el.reference) {
        el.desc = Arc::clone(&desc);
    }
    el.reference = desc;
    tree.set_z_level(element, z);
}

/// `element` moved and sized to `at` (field coordinates), when it is not there.
fn put(tree: &mut UiTree, element: Option<usize>, at: UiBox) {
    let Some(element) = element else {
        return;
    };
    if tree.el(element).rect != at {
        tree.resize_to(element, at.width(), at.height());
        tree.move_to(element, at.x0, at.y0);
    }
}

/// A panel button to `at`, and the quest button's own button (it is a field over its art's
/// button) to the field's whole box.
fn put_button(tree: &mut UiTree, button: Option<usize>, at: UiBox) {
    put(tree, button, at);
    let Some(button) = button else {
        return;
    };
    if tree.el(button).image.is_none() {
        if let Some(&inner) = tree.children(button).first() {
            put(
                tree,
                Some(inner),
                UiBox::new(0, 0, at.width() - 1, at.height() - 1),
            );
        }
    }
}

/// A part of the design under `field`: a copy of `template` (a plain field of the toolbar layout)
/// with no states, named `id`, its art `image` (in nine pieces with corners `nine` when given),
/// taking no pointer, at depth `z`.
fn make_part(
    tree: &mut UiTree,
    field: usize,
    template: Option<usize>,
    id: &str,
    image: u32,
    nine: Option<i32>,
    z: i32,
) -> Option<usize> {
    let template = template?;
    let layout = Arc::clone(&tree.el(field).layout);
    let mut desc = kept_in_place(&tree.el(template).reference);
    desc.id = tokenize(id);
    desc.states.clear();
    // Its depth in its description: the anchoring (`UIElement::UpdateSizeAndPosition`, run again
    // at every display change) takes an element's depth from it.
    desc.z_level = z as u16;
    let part = tree.create_element(layout, Arc::new(desc), false)?;
    tree.set_parent(part, Some(field));
    tree.set_image(part, image, DRAW_OVER);
    if let Some(border) = nine {
        tree.set_int(part, NINE_SLICE, border);
    }
    tree.set_bool(part, MOUSE_INVISIBLE, true);
    tree.set_z_level(part, z);
    Some(part)
}

/// The field made over: each child kept where it is put (its description swapped in the field's
/// lists too, so the field's own update finds it unchanged), its art cleared, its box the bottom
/// edge's full width and [`FIELD_HEIGHT`] tall, taking no pointer itself.
fn adopt_field(tree: &mut UiTree, field: usize) {
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
        // Left kept, top, right and bottom with the display's far side: the bottom edge's full
        // width at any size.
        desc.edges = [1, 2, 1, 1];
        for state in &mut desc.states {
            if state.rect.x0 != -1 {
                state.rect = field_box;
            }
            state
                .media
                .retain(|m| !matches!(m, MediaDesc::Image { .. }));
            for listed in &mut state.children {
                if let Some((_, new)) = swapped.iter().find(|(old, _)| Arc::ptr_eq(old, listed)) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_strip_is_centred_on_the_bottom_edge_with_its_parts_in_order() {
        let p = plan(1365, (350, 35), [false; 2]);
        let middle = (p.fade.x0 + p.fade.x1) / 2;
        assert!(
            (middle - (1365 - DRAWER_ROOM) / 2).abs() <= 1,
            "{:?}",
            p.fade
        );
        assert_eq!(p.fade.y1, FIELD_HEIGHT - 1 - STRIP_BOTTOM);
        assert!(p.start_attack.x1 < p.stop_attack.x1);
        assert!(p.stop_attack.x1 - art::GLOW_ROOM < p.set_up.x0);
        assert!(p.set_up.x1 < p.rows.0);
        assert_eq!(p.set_up.y1 + 1, p.set_down.y0);
    }

    #[test]
    fn the_dock_sits_in_the_corner_and_the_fan_and_pills_go_left_of_the_orb() {
        let p = plan(1365, (350, 35), [true, true]);
        assert_eq!(p.orb.x1, 1365 - 1 - DOCK_RIGHT);
        let (sale, dismount) = (p.pills[0].unwrap(), p.pills[1].unwrap());
        assert!(dismount.x1 < p.orb.x0 + 10 && sale.x1 < dismount.x0);
        assert!(p.fan.x1 < sale.x0);
        // The wells in a row, the separator between the third and the fourth.
        assert!(p.wells.windows(2).all(|w| w[0].x0 < w[1].x0));
        assert!(p.wells[2].x1 - art::GLOW_ROOM < p.separator.x0);
        assert!(p.separator.x0 < p.wells[3].x0 + art::GLOW_ROOM);
    }
}
