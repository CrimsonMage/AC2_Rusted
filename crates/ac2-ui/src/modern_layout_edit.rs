//! The modern style's Edit layout mode (AGENTS.md: the HUD's two styles), opened from the option
//! panel's Edit HUD layout button. While it is on, a clear shield over the display takes every
//! press: each movable part of the HUD shows a dashed outline and its name, and a drag moves it.
//! The parts the design lays out by the display's edges (the vitals, the radar, the inventory
//! drawer, the shortcut strip, the dock) keep an offset from their place (`UiTree::hud_offset`);
//! the chat windows move as windows (their places kept as every window's are). A bar at the top
//! locks the whole HUD in place (no part or window moves or resizes; leaving the mode), resets the
//! layout, or closes the mode. The offsets and the lock are kept in the HUD's layout settings
//! (`offset.<part>`, `hud.locked`). Retail had none of this.

use std::cell::Cell;
use std::sync::Arc;

use crate::element::{UiBox, UiTree};
use crate::hud_layout::HudLayout;
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_drawer_bar::{count_label, place_label};
use crate::modern_inventory::{keep, make_part};
use crate::mouse::{UiMessage, GLOBAL_MOUSE_MOVE, GLOBAL_MOUSE_UP};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE, MSG_MOUSE_PRESS_TOP};

/// The parts laid out by the display's edges: their names (the offsets' keys), what the tag says,
/// and how the part is found (a root's id, or a part's id anywhere).
const PINNED: [(&str, &str, &str); 5] = [
    ("vitals", "Vitals", "StatusbarField"),
    ("radar", "Radar", "RadarField"),
    ("drawer", "Inventory", "RightSidePanel"),
    ("strip", "Shortcuts", "ModernStripFade"),
    ("dock", "Dock", "ModernDockOrb"),
];
/// The chat windows, moved as windows.
const WINDOWS: [&str; 4] = ["ChatWindow1", "ChatWindow2", "ChatWindow3", "ChatWindow4"];
/// The bar's size and the tag's height.
const BAR: (i32, i32) = (380, 50);
const TAG: i32 = 14;

/// A movable part's outline and tag.
#[derive(Debug, Clone, Copy)]
struct Mark {
    outline: Option<usize>,
    tag: Option<usize>,
    label: Option<usize>,
}

/// What a press on the bar asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditAction {
    /// Reset to default: the offsets cleared here; the chat's choices are the chat's to reset, and
    /// its windows go back to their places (`reset_windows`).
    Reset,
}

/// The mode's parts and state.
#[derive(Debug, Clone)]
pub struct EditLayout {
    shield: usize,
    marks: Vec<Mark>,
    bar: usize,
    lock: (Option<usize>, Option<usize>),
    reset: (Option<usize>, Option<usize>),
    done: (Option<usize>, Option<usize>),
    on: Cell<bool>,
    /// A pinned part being dragged: its index in `PINNED`, the pointer and the offset at the press.
    drag: Cell<Option<(usize, (i32, i32), (i32, i32))>>,
}

fn root_named(tree: &UiTree, name: &str) -> Option<usize> {
    let id = tokenize(name);
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.el(r).id() == id)
}

fn find(tree: &UiTree, name: &str) -> Option<usize> {
    root_named(tree, name).or_else(|| {
        let id = tokenize(name);
        tree.roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id))
    })
}

/// A root of the display's size (or `size`) made from `window`'s layout (copying `template`).
fn make_root(tree: &mut UiTree, window: usize, template: usize, id: &str, z: i32) -> Option<usize> {
    let layout = Arc::clone(&tree.el(window).layout);
    let mut desc = (*tree.el(template).reference).clone();
    desc.id = tokenize(id);
    desc.states.clear();
    desc.edges = [0; 4];
    let root = tree.create_element(layout, Arc::new(desc), true)?;
    tree.el_mut(root)
        .attributes
        .remove(&crate::token::ALPHA_VALUE);
    tree.set_z_level(root, z);
    tree.add_root(root);
    tree.set_overlay_root(root);
    tree.set_visible(root, false);
    Some(root)
}

/// Reset to default for the chat windows: each back at its layout's place, its saved place
/// forgotten.
pub(crate) fn reset_windows(tree: &mut UiTree) {
    for name in WINDOWS {
        if let Some(root) = root_named(tree, name) {
            tree.reset_window_place(root);
        }
    }
}

impl EditLayout {
    /// The mode's shield, marks and bar made from `window`'s layout (a chat window's, copying its
    /// plain field `template`); hidden.
    pub(crate) fn make(
        tree: &mut UiTree,
        window: usize,
        template: usize,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Option<Self> {
        let shield = make_root(tree, window, template, "ModernEditShield", 2)?;
        tree.set_image(shield, art::CHAT_CLEAR, crate::compose::DRAW_OVER);
        tree.set_bool(shield, crate::token::SIZE_IMAGE_TO_OWNER, true);
        let part = |tree: &mut UiTree, parent: usize, id: &str, image: u32, nine: Option<i32>| {
            let part = make_part(tree, parent, Some(template), id, image, nine, 1)?;
            keep(tree, part);
            Some(part)
        };
        let label = |tree: &mut UiTree, parent: Option<usize>, id: &str, rgb: u32| {
            let label = count_label(tree, layouts, parent?, id, Justification::Left)?;
            keep(tree, label);
            for (channel, shift) in [("red", 16), ("green", 8), ("blue", 0)] {
                let _ = tree.set_float(
                    label,
                    tokenize(&format!("font_color1_{channel}")),
                    ((rgb >> shift) & 0xFF) as f32 / 255.0,
                );
            }
            Some(label)
        };
        let count = PINNED.len() + WINDOWS.len();
        let marks = (0..count)
            .map(|i| {
                let outline = part(
                    tree,
                    shield,
                    &format!("ModernEditOutline{i}"),
                    art::EDIT_OUTLINE,
                    Some(art::OUTLINE_BORDER),
                );
                let tag = part(
                    tree,
                    shield,
                    &format!("ModernEditTag{i}"),
                    art::EDIT_TAG,
                    Some(4),
                );
                let label = label(tree, tag, &format!("ModernEditTagText{i}"), 0x161006);
                Mark {
                    outline,
                    tag,
                    label,
                }
            })
            .collect();
        let bar = make_root(tree, window, template, "ModernEditBar", 1)?;
        tree.set_image(bar, art::CHAT_POPOVER, crate::compose::DRAW_OVER);
        tree.set_int(bar, crate::compose::NINE_SLICE, art::CHAT_BORDER);
        tree.resize_to(bar, BAR.0, BAR.1);
        let title = label(tree, Some(bar), "ModernEditTitle", 0xD9C27A);
        if let Some(t) = title {
            tree.set_text_with_font(t, "EDIT LAYOUT  -  drag any outlined part", 0, 1);
            place_label(tree, Some(t), UiBox::new(10, 6, BAR.0 - 10, 19));
        }
        let chip = |tree: &mut UiTree, id: &str, text: &str, x: i32| {
            let chip = part(tree, bar, id, art::CHIP, Some(art::CHIP_BORDER));
            let label = label(tree, chip, &format!("{id}Text"), 0xEFE8CC);
            if let (Some(c), Some(l)) = (chip, label) {
                tree.set_bool(c, MOUSE_INVISIBLE, false);
                tree.set_text_with_font(l, text, 0, 1);
                let w = tree.text_line_width(l, text).unwrap_or(60) + 14;
                tree.resize_to(c, w, 16);
                tree.move_to(c, x, 26);
                place_label(tree, Some(l), UiBox::new(7, 1, w - 1, 15));
            }
            (chip, label)
        };
        let lock = chip(tree, "ModernEditLock", "Lock HUD layout", 10);
        let reset = chip(tree, "ModernEditReset", "Reset to default", 140);
        let done = chip(tree, "ModernEditDone", "Done", 300);
        Some(Self {
            shield,
            marks,
            bar,
            lock,
            reset,
            done,
            on: Cell::new(false),
            drag: Cell::new(None),
        })
    }

    pub(crate) fn is_on(&self) -> bool {
        self.on.get()
    }

    /// The mode turned on (the HUD's lock released) or off.
    pub(crate) fn set_on(&self, tree: &mut UiTree, on: bool) {
        self.on.set(on);
        self.drag.set(None);
        if on {
            tree.hud_locked = false;
        }
        for root in [self.shield, self.bar] {
            tree.set_visible(root, on);
        }
        if on {
            self.follow(tree);
        }
    }

    /// The movable parts on the screen now: their tags' text and boxes, and how each moves.
    fn movables(&self, tree: &UiTree) -> Vec<(String, Option<UiBox>)> {
        let mut out: Vec<(String, Option<UiBox>)> = PINNED
            .iter()
            .map(|(_, tag, id)| {
                let at = find(tree, id).map(|e| tree.screen_box(e));
                (tag.to_string(), at)
            })
            .collect();
        for (i, name) in WINDOWS.iter().enumerate() {
            let at = root_named(tree, name)
                .filter(|&r| tree.el(r).visible)
                .map(|r| tree.screen_box(r));
            out.push((format!("Chat {}", i + 1), at));
        }
        out
    }

    /// The shield the display's size and the marks on the parts where they are; the bar at the
    /// top's middle. Run every frame while on.
    pub(crate) fn follow(&self, tree: &mut UiTree) {
        if !self.on.get() {
            return;
        }
        let (w, h) = tree.display;
        let now = tree.screen_box(self.shield);
        if (now.x0, now.y0, now.width(), now.height()) != (0, 0, w, h) {
            tree.resize_to(self.shield, w, h);
            tree.set_object_position(self.shield, 0, 0);
            tree.move_to(self.shield, 0, 0);
        }
        let bar_at = ((w - BAR.0) / 2, 8);
        let now = tree.screen_box(self.bar);
        if (now.x0, now.y0) != bar_at {
            tree.set_object_position(self.bar, bar_at.0, bar_at.1);
            tree.move_to(self.bar, bar_at.0, bar_at.1);
        }
        let lit = |tree: &mut UiTree, chip: Option<usize>, on: bool| {
            if let Some(c) = chip {
                tree.set_image(
                    c,
                    if on { art::CHIP_ON } else { art::CHIP },
                    crate::compose::DRAW_OVER,
                );
            }
        };
        lit(tree, self.lock.0, tree.hud_locked);
        for (mark, (name, at)) in self.marks.iter().zip(self.movables(tree)) {
            let shown = at.is_some();
            for part in [mark.outline, mark.tag].into_iter().flatten() {
                if tree.el(part).visible != shown {
                    tree.set_visible(part, shown);
                }
            }
            let Some(at) = at else {
                continue;
            };
            if let Some(o) = mark.outline {
                if tree.el(o).rect != at.offset(-2, -2) {
                    tree.resize_to(o, at.width() + 4, at.height() + 4);
                    tree.move_to(o, at.x0 - 2, at.y0 - 2);
                }
            }
            if let (Some(t), Some(l)) = (mark.tag, mark.label) {
                let text = format!("{name} (drag)");
                if tree.el(l).text().as_deref() != Some(text.as_str()) {
                    tree.set_text_with_font(l, &text, 0, 1);
                }
                let tw = tree.text_line_width(l, &text).unwrap_or(60) + 12;
                let ty = (at.y0 - 2 - TAG).max(0);
                tree.resize_to(t, tw, TAG);
                tree.move_to(t, at.x0 - 2, ty);
                place_label(tree, Some(l), UiBox::new(6, 0, tw - 1, TAG - 1));
            }
        }
    }

    /// A message while the mode is on: a press on the shield picks the part under the pointer (a
    /// chat window first) and starts its move, the pointer moves a pinned part, a release ends
    /// it; the bar's chips. Whether the mode took the message, and what a chip asks.
    pub(crate) fn message(
        &self,
        tree: &mut UiTree,
        message: &UiMessage,
    ) -> (bool, Option<EditAction>) {
        if !self.on.get() {
            return (false, None);
        }
        let pointer = tree.pointer();
        if message.message == MSG_MOUSE_PRESS_TOP {
            let Some(element) = message.element else {
                return (false, None);
            };
            if Some(element) == self.done.0 {
                self.set_on(tree, false);
                return (true, None);
            }
            if Some(element) == self.lock.0 {
                tree.hud_locked = true;
                self.set_on(tree, false);
                return (true, None);
            }
            if Some(element) == self.reset.0 {
                tree.hud_offsets.clear();
                tree.mark_changed();
                return (true, Some(EditAction::Reset));
            }
            if element != self.shield {
                return (false, None);
            }
            let inside = |b: UiBox| {
                pointer.0 >= b.x0 && pointer.0 <= b.x1 && pointer.1 >= b.y0 && pointer.1 <= b.y1
            };
            // A chat window under the pointer moves as a window.
            for name in WINDOWS {
                if let Some(root) = root_named(tree, name).filter(|&r| tree.el(r).visible) {
                    if inside(tree.screen_box(root)) {
                        tree.start_movement(root);
                        return (true, None);
                    }
                }
            }
            for (i, (key, _, id)) in PINNED.iter().enumerate() {
                if find(tree, id).is_some_and(|e| inside(tree.screen_box(e))) {
                    self.drag.set(Some((i, pointer, tree.hud_offset(key))));
                    return (true, None);
                }
            }
            return (true, None);
        }
        if let Some((i, start, from)) = self.drag.get() {
            if message.message == GLOBAL_MOUSE_MOVE || message.message == GLOBAL_MOUSE_UP {
                let offset = (from.0 + pointer.0 - start.0, from.1 + pointer.1 - start.1);
                tree.hud_offsets.insert(PINNED[i].0.to_string(), offset);
                tree.mark_changed();
                if message.message == GLOBAL_MOUSE_UP {
                    self.drag.set(None);
                }
                return (true, None);
            }
        }
        (false, None)
    }
}

/// The offsets and the lock written into the HUD's layout settings; whether they changed.
pub(crate) fn export(tree: &UiTree, layout: &mut HudLayout) -> bool {
    let mut changed = layout.set("hud.locked", tree.hud_locked);
    for (key, _, _) in PINNED {
        let (x, y) = tree.hud_offset(key);
        changed |= layout.set(&format!("offset.{key}"), format!("{x},{y}"));
    }
    changed
}

/// The offsets and the lock as the HUD's layout settings keep them.
pub(crate) fn apply(tree: &mut UiTree, layout: &HudLayout) {
    tree.hud_locked = layout.flag("hud.locked", false);
    tree.hud_offsets.clear();
    for (key, _, _) in PINNED {
        let value = layout.get(&format!("offset.{key}")).unwrap_or_default();
        let mut parts = value.split(',').map(|v| v.trim().parse::<i32>());
        if let (Some(Ok(x)), Some(Ok(y))) = (parts.next(), parts.next()) {
            if (x, y) != (0, 0) {
                tree.hud_offsets.insert(key.to_string(), (x, y));
            }
        }
    }
}
