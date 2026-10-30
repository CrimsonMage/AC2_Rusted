//! The modern chat windows' tabs (AGENTS.md: the HUD's two styles): windows docked together share
//! one frame, the window shown carrying a tab for each of them (its window key and name, an
//! unread dot while a hidden one takes lines) and a `+` that docks another window in. A press on
//! a tab shows that window in the frame's place (`ChatWindow::Show`, the others of the frame
//! hidden); the window keys still show and hide each window, a docked window shown by its key
//! taking the frame. Retail's windows were each their own; docking is the player's choice.

use std::sync::Arc;

use crate::element::{UiBox, UiTree};
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_chat::ChatEvent;
use crate::modern_drawer_bar::{count_label, place_label};
use crate::modern_inventory::{keep, make_part};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE};

/// The most windows (and so tabs).
pub(crate) const WINDOWS: usize = 4;
const TAB_HEIGHT: i32 = 21;
const KEY_WIDTH: i32 = 18;
const NAME_MAX: i32 = 96;
const ADD: i32 = 19;

/// One tab of a window's header.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TabSlot {
    pub tab: Option<usize>,
    key: Option<usize>,
    key_label: Option<usize>,
    name: Option<usize>,
    dot: Option<usize>,
}

/// A window's tab strip: a tab per docked window and the `+`.
#[derive(Debug, Clone)]
pub(crate) struct TabStrip {
    pub slots: Vec<TabSlot>,
    pub add: Option<usize>,
}

fn color(tree: &mut UiTree, label: usize, rgb: u32) {
    for (channel, shift) in [("red", 16), ("green", 8), ("blue", 0)] {
        let _ = tree.set_float(
            label,
            tokenize(&format!("font_color1_{channel}")),
            ((rgb >> shift) & 0xFF) as f32 / 255.0,
        );
    }
}

impl TabStrip {
    /// The strip's parts made under the window's root (`template`: a plain field of it), hidden.
    pub(crate) fn make(
        tree: &mut UiTree,
        root: usize,
        template: Option<usize>,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Self {
        let part =
            |tree: &mut UiTree, parent: usize, id: &str, image: u32, nine: Option<i32>, z: i32| {
                let part = make_part(tree, parent, template, id, image, nine, z)?;
                keep(tree, part);
                tree.set_visible(part, false);
                Some(part)
            };
        let slots = (0..WINDOWS)
            .map(|i| {
                let tab = part(
                    tree,
                    root,
                    &format!("ModernChatTabSlot{i}"),
                    art::CHAT_TAB_OFF,
                    Some(art::TAB_BORDER),
                    3,
                );
                if let Some(tab) = tab {
                    tree.set_bool(tab, MOUSE_INVISIBLE, false);
                }
                let key = tab.and_then(|t| {
                    let key = part(
                        tree,
                        t,
                        &format!("ModernChatTabKey{i}"),
                        art::CHAT_KEY,
                        Some(2),
                        1,
                    )?;
                    tree.set_bool(key, MOUSE_INVISIBLE, true);
                    tree.set_visible(key, true);
                    Some(key)
                });
                let label = |tree: &mut UiTree, parent: Option<usize>, id: String, j, rgb| {
                    let label = count_label(tree, layouts, parent?, &id, j)?;
                    keep(tree, label);
                    color(tree, label, rgb);
                    Some(label)
                };
                let key_label = label(
                    tree,
                    key,
                    format!("ModernChatTabKeyText{i}"),
                    Justification::Center,
                    0xA88F48,
                );
                let name = label(
                    tree,
                    tab,
                    format!("ModernChatTabName{i}"),
                    Justification::Left,
                    0xA9B3A2,
                );
                let dot = tab.and_then(|t| {
                    let dot = part(
                        tree,
                        t,
                        &format!("ModernChatTabDot{i}"),
                        art::CHAT_UNREAD,
                        None,
                        0,
                    )?;
                    tree.set_bool(dot, MOUSE_INVISIBLE, true);
                    Some(dot)
                });
                TabSlot {
                    tab,
                    key,
                    key_label,
                    name,
                    dot,
                }
            })
            .collect();
        let add = part(tree, root, "ModernChatAdd", art::CHAT_ADD, None, 3);
        if let Some(add) = add {
            tree.set_bool(add, MOUSE_INVISIBLE, false);
        }
        Self { slots, add }
    }

    /// The tabs for `tabs` (each: window key, name, shown, unread) from `x` along the header's
    /// bottom (`header` high), then the `+` when `can_add`; the strip's right edge.
    pub(crate) fn lay_out(
        &self,
        tree: &mut UiTree,
        x: i32,
        header: i32,
        tabs: &[(usize, String, bool, bool)],
        can_add: bool,
    ) -> i32 {
        let mut x = x;
        for (i, slot) in self.slots.iter().enumerate() {
            let Some(tab) = slot.tab else {
                continue;
            };
            let Some((key, name, on, unread)) = tabs.get(i) else {
                if tree.el(tab).visible {
                    tree.set_visible(tab, false);
                }
                continue;
            };
            if let Some(label) = slot.key_label {
                tree.set_text_with_font(label, &format!("F{key}"), 0, 1);
            }
            if let Some(label) = slot.name {
                color(tree, label, if *on { 0xFFF4CF } else { 0xA9B3A2 });
                tree.set_text_with_font(label, name, 0, 1);
            }
            let name_w = slot
                .name
                .and_then(|l| tree.text_line_width(l, name))
                .unwrap_or(60)
                .min(NAME_MAX);
            let dot_w = if *unread { 9 } else { 0 };
            let w = 6 + KEY_WIDTH + 4 + name_w + dot_w + 6;
            tree.resize_to(tab, w, TAB_HEIGHT);
            tree.move_to(tab, x, header - TAB_HEIGHT);
            tree.set_image(
                tab,
                if *on {
                    art::CHAT_TAB
                } else {
                    art::CHAT_TAB_OFF
                },
                crate::compose::DRAW_OVER,
            );
            tree.set_int(tab, crate::compose::NINE_SLICE, art::TAB_BORDER);
            if let Some(k) = slot.key {
                tree.resize_to(k, KEY_WIDTH, 12);
                tree.move_to(k, 6, 5);
            }
            place_label(tree, slot.key_label, UiBox::new(0, 0, KEY_WIDTH - 1, 11));
            let name_x = 6 + KEY_WIDTH + 4;
            place_label(
                tree,
                slot.name,
                UiBox::new(name_x, 3, name_x + name_w, TAB_HEIGHT - 4),
            );
            if let Some(dot) = slot.dot {
                tree.resize_to(dot, 10, 10);
                tree.move_to(dot, name_x + name_w + 1, 6);
                if tree.el(dot).visible != *unread {
                    tree.set_visible(dot, *unread);
                }
            }
            if !tree.el(tab).visible {
                tree.set_visible(tab, true);
            }
            x += w + 2;
        }
        if let Some(add) = self.add {
            if can_add {
                tree.resize_to(add, ADD, ADD);
                tree.move_to(add, x + 2, header - TAB_HEIGHT + 1);
                x += ADD + 4;
            }
            if tree.el(add).visible != can_add {
                tree.set_visible(add, can_add);
            }
        }
        x
    }

    /// The tab (its index in the strip) `element` is.
    pub(crate) fn tab_of(&self, element: usize) -> Option<usize> {
        self.slots.iter().position(|s| s.tab == Some(element))
    }
}

/// The `+`'s menu: a chip per window that can dock in, its own root over the windows.
#[derive(Debug, Clone)]
pub(crate) struct DockMenu {
    pub root: usize,
    chips: Vec<(Option<usize>, Option<usize>)>,
}

impl DockMenu {
    pub(crate) fn make(
        tree: &mut UiTree,
        window: usize,
        template: usize,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Option<Self> {
        let layout = Arc::clone(&tree.el(window).layout);
        let mut desc = (*tree.el(template).reference).clone();
        desc.id = tokenize("ModernChatDockMenu");
        desc.states.clear();
        desc.edges = [0; 4];
        desc.z_level = 1;
        let root = tree.create_element(layout, Arc::new(desc), true)?;
        tree.el_mut(root)
            .attributes
            .remove(&crate::token::ALPHA_VALUE);
        tree.set_z_level(root, 1);
        tree.add_root(root);
        tree.set_overlay_root(root);
        tree.set_image(root, art::CHAT_POPOVER, crate::compose::DRAW_OVER);
        tree.set_int(root, crate::compose::NINE_SLICE, art::CHAT_BORDER);
        let chips = (0..WINDOWS)
            .map(|i| {
                let chip = make_part(
                    tree,
                    root,
                    Some(template),
                    &format!("ModernChatDockChip{i}"),
                    art::CHIP,
                    Some(art::CHIP_BORDER),
                    1,
                );
                if let Some(chip) = chip {
                    keep(tree, chip);
                    tree.set_bool(chip, MOUSE_INVISIBLE, false);
                }
                let label = chip.and_then(|c| {
                    let label = count_label(
                        tree,
                        layouts,
                        c,
                        &format!("ModernChatDockText{i}"),
                        Justification::Left,
                    )?;
                    keep(tree, label);
                    Some(label)
                });
                (chip, label)
            })
            .collect();
        tree.set_visible(root, false);
        Some(Self { root, chips })
    }

    /// The menu shown under `under` (screen box) with a chip for each of `windows` (window key
    /// and name).
    pub(crate) fn show(&self, tree: &mut UiTree, under: UiBox, windows: &[(usize, String)]) {
        let mut y = 6;
        let mut width = 120;
        for (i, (chip, label)) in self.chips.iter().enumerate() {
            let (Some(chip), Some(label)) = (*chip, *label) else {
                continue;
            };
            let Some((key, name)) = windows.get(i) else {
                tree.set_visible(chip, false);
                continue;
            };
            let text = format!("F{key}  {name}");
            tree.set_text(label, &text);
            let w = tree.text_line_width(label, &text).unwrap_or(100) + 16;
            width = width.max(w + 12);
            tree.resize_to(chip, w, 16);
            tree.move_to(chip, 6, y);
            place_label(tree, Some(label), UiBox::new(8, 0, w - 1, 15));
            tree.set_visible(chip, true);
            y += 20;
        }
        tree.resize_to(self.root, width, y + 4);
        tree.set_object_position(self.root, under.x0, under.y1 + 4);
        tree.move_to(self.root, under.x0, under.y1 + 4);
        tree.set_visible(self.root, true);
    }

    pub(crate) fn hide(&self, tree: &mut UiTree) {
        tree.set_visible(self.root, false);
    }

    pub(crate) fn shown(&self, tree: &UiTree) -> bool {
        tree.el(self.root).visible
    }

    /// The chip (its index) `element` is.
    pub(crate) fn chip_of(&self, element: usize) -> Option<usize> {
        self.chips.iter().position(|(c, _)| *c == Some(element))
    }
}

/// The docked windows' frame: which windows share it, which one shows, and the presses on its
/// tabs, its `+` and the dock menu.
impl crate::modern_chat::ModernChat {
    /// The windows docked together, in tab order (none when each stands alone).
    #[cfg(test)]
    pub(crate) fn group(&self) -> Vec<usize> {
        self.group.borrow().clone()
    }

    /// The tab name of window `i`: the player's name for it, else its own (`names`).
    pub(crate) fn name_of(&self, i: usize, names: &[String]) -> String {
        let looks = self.looks.borrow();
        match looks.get(i) {
            Some(look) if !look.name.is_empty() => look.name.clone(),
            _ => names.get(i).cloned().unwrap_or_default(),
        }
    }

    /// The docked windows kept in one frame (run each frame before the layout): a docked window
    /// shown by its key takes the frame from the one shown; the shown one's lines are seen.
    pub(crate) fn keep_group(&self, tree: &UiTree, events: &mut Vec<ChatEvent>) {
        let group = self.group.borrow().clone();
        if group.len() < 2 {
            return;
        }
        let shown: Vec<usize> = group
            .iter()
            .copied()
            .filter(|&m| tree.el(self.windows[m].root).visible)
            .collect();
        let active = self.active.get().filter(|a| group.contains(a));
        match (active, shown.as_slice()) {
            (_, []) => {}
            (Some(a), [only]) if *only == a => {}
            (Some(a), _) if shown.iter().any(|&m| m != a) => {
                // Another tab shown by its key: it takes the frame.
                let new = *shown.iter().rev().find(|&&m| m != a).unwrap_or(&a);
                let at = tree.screen_box(self.windows[a].root);
                events.push(ChatEvent::Activate {
                    show: new,
                    hide: Some(a),
                    at,
                });
                self.active.set(Some(new));
            }
            _ => {
                let first = shown[0];
                self.active.set(Some(first));
                for &m in &shown[1..] {
                    events.push(ChatEvent::Hide(m));
                }
            }
        }
        if let Some(a) = self.active.get() {
            let counts = self.counts.borrow();
            self.seen.borrow_mut()[a] = counts[a];
        }
    }

    /// A press on a tab, the `+` or a dock menu chip: the event it asks for (a tab's window shown
    /// in the frame, a window docked in).
    pub(crate) fn tab_press(&self, tree: &mut UiTree, element: usize) -> Option<ChatEvent> {
        // The dock menu's chips.
        if let (Some(menu), Some(i)) = (&self.menu, self.menu_for.get()) {
            if let Some(c) = menu.chip_of(element) {
                let k = *self.menu_offers.borrow().get(c)?;
                menu.hide(tree);
                self.menu_for.set(None);
                return self.dock(tree, k, i);
            }
        }
        for (i, w) in self.windows.iter().enumerate() {
            if w.strip.add == Some(element) {
                return self.toggle_menu(tree, i);
            }
            let Some(slot) = w.strip.tab_of(element) else {
                continue;
            };
            let group = self.group.borrow().clone();
            let m = *group.get(slot)?;
            if m == i || !group.contains(&i) {
                return None;
            }
            self.active.set(Some(m));
            return Some(ChatEvent::Activate {
                show: m,
                hide: Some(i),
                at: tree.screen_box(w.root),
            });
        }
        None
    }

    /// The dock menu opened under window `i`'s `+` with the windows that are not in its frame, or
    /// closed if open for it.
    pub(crate) fn toggle_menu(&self, tree: &mut UiTree, i: usize) -> Option<ChatEvent> {
        let menu = self.menu.as_ref()?;
        if self.menu_for.get() == Some(i) && menu.shown(tree) {
            menu.hide(tree);
            self.menu_for.set(None);
            return None;
        }
        let group = self.group.borrow().clone();
        let offers: Vec<usize> = (0..self.windows.len())
            .filter(|&k| k != i && !group.contains(&k))
            .collect();
        let names: Vec<String> = Vec::new();
        let entries: Vec<(usize, String)> = offers
            .iter()
            .map(|&k| (k + 1, self.tab_name(k, &names)))
            .collect();
        let at = tree.screen_box(self.windows[i].strip.add?);
        menu.show(tree, at, &entries);
        *self.menu_offers.borrow_mut() = offers;
        self.menu_for.set(Some(i));
        None
    }

    /// Window `k` docked into window `i`'s frame (a frame made of `i` when it stands alone): `k`
    /// hides behind `i`'s tab.
    pub(crate) fn dock(&self, _tree: &mut UiTree, k: usize, i: usize) -> Option<ChatEvent> {
        {
            let mut group = self.group.borrow_mut();
            if !group.contains(&i) {
                group.clear();
                group.push(i);
            }
            if !group.contains(&k) {
                group.push(k);
            }
        }
        self.active.set(Some(i));
        for w in &self.windows {
            *w.laid_out.borrow_mut() = None;
        }
        Some(ChatEvent::Hide(k))
    }

    /// The settings popover's pop out or dock for window `i`: out of its frame (shown beside it),
    /// or into the frame of the windows docked together, else of the first other window shown.
    pub(crate) fn tear(&self, tree: &mut UiTree, i: usize) -> Option<ChatEvent> {
        let in_group = self.group.borrow().contains(&i);
        for w in &self.windows {
            *w.laid_out.borrow_mut() = None;
        }
        if in_group {
            let at = tree.screen_box(self.windows[i].root);
            let mut group = self.group.borrow_mut();
            group.retain(|&m| m != i);
            let rest = group.first().copied();
            if group.len() < 2 {
                group.clear();
            }
            drop(group);
            if self.active.get() == Some(i) {
                self.active
                    .set(rest.filter(|_| !self.group.borrow().is_empty()));
            }
            let shifted = at.offset(24, -at.height() - 8);
            // The window shown in the frame pops out beside it; the frame's next window takes
            // the frame's place.
            if let Some(r) = rest.filter(|&r| !tree.el(self.windows[r].root).visible) {
                let root = self.windows[i].root;
                tree.move_to(root, shifted.x0, shifted.y0);
                return Some(ChatEvent::Activate {
                    show: r,
                    hide: None,
                    at,
                });
            }
            return Some(ChatEvent::PopOut {
                window: i,
                at: shifted,
            });
        }
        let group = self.group.borrow().clone();
        let host = match group.first() {
            Some(&h) => h,
            None => (0..self.windows.len())
                .find(|&k| k != i && tree.el(self.windows[k].root).visible)?,
        };
        let shown = self
            .active
            .get()
            .filter(|a| group.contains(a))
            .unwrap_or(host);
        self.dock(tree, i, shown)
    }

    /// Window `i`'s tab name without the HUD's names (the dock menu): the player's name for it,
    /// else `Chat <n>`.
    pub(crate) fn tab_name(&self, i: usize, names: &[String]) -> String {
        let name = self.name_of(i, names);
        if name.is_empty() {
            format!("Chat {}", i + 1)
        } else {
            name
        }
    }
}
