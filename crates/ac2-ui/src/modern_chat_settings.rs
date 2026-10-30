//! The modern chat windows' settings (AGENTS.md: the HUD's two styles): what the design lets a
//! player choose per window that retail had no setting for, and the popover the window's options
//! button (`=`) opens to choose it. A window's name (its tab's label, else `ID_Option_Chat<n>`), a
//! font size step over the chat options' per-type sizes (small, medium, large), whether its glass
//! and the log's backing show and the backing's tint, and whether its place is locked (no move,
//! no resize). The popover also hides the window (`ChatWindow::Show(0)`, as its window key does)
//! and opens the chat options page. The choices are kept with the HUD's layout settings
//! (`hud_layout`, `chat.<n>.*`).

use std::sync::Arc;

use crate::element::{UiBox, UiTree};
use crate::hud_layout::HudLayout;
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_drawer_bar::{count_label, place_label};
use crate::modern_inventory::{keep, make_part};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE};
use crate::window_parts::{entry_field, place_entry};

/// One window's choices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChatLook {
    /// The tab's label; empty: the window's own name.
    pub name: String,
    /// The font size step: -1 small, 0 medium (the options' sizes), 1 large.
    pub font: i32,
    pub background: bool,
    /// An index into `TINT_NAMES`.
    pub tint: usize,
    pub locked: bool,
}

impl Default for ChatLook {
    fn default() -> Self {
        Self {
            name: String::new(),
            font: 0,
            background: true,
            tint: 0,
            locked: false,
        }
    }
}

impl ChatLook {
    /// Window `n`'s choices as `layout` keeps them.
    pub(crate) fn read(layout: &HudLayout, n: usize) -> Self {
        let key = |k: &str| format!("chat.{n}.{k}");
        Self {
            name: layout.get(&key("name")).unwrap_or_default().to_string(),
            font: layout.number(&key("font"), 0).clamp(-1, 1) as i32,
            background: layout.flag(&key("background"), true),
            tint: (layout.number(&key("tint"), 0).max(0) as usize).min(art::TINT_NAMES.len() - 1),
            locked: layout.flag(&key("locked"), false),
        }
    }

    /// Window `n`'s choices written into `layout`; whether one changed.
    pub(crate) fn write(&self, layout: &mut HudLayout, n: usize) -> bool {
        let key = |k: &str| format!("chat.{n}.{k}");
        let mut changed = layout.set(&key("name"), &self.name);
        changed |= layout.set(&key("font"), self.font);
        changed |= layout.set(&key("background"), self.background);
        changed |= layout.set(&key("tint"), self.tint);
        changed |= layout.set(&key("locked"), self.locked);
        changed
    }
}

/// What a press in the popover asks of the chat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsAction {
    /// The window's choices changed.
    Changed,
    /// `ChatWindow::Show(0)`.
    Hide,
    /// The options panel's chat page.
    OpenOptions,
}

/// A chip: its art and its label.
#[derive(Debug, Clone, Copy)]
struct Chip {
    part: Option<usize>,
    label: Option<usize>,
}

/// The popover: its own root over every window, made once and shown for one window at a time.
#[derive(Debug, Clone)]
pub(crate) struct Popover {
    pub root: usize,
    title: Option<usize>,
    labels: Vec<(Option<usize>, (i32, i32, i32))>,
    name: Option<(usize, Option<usize>)>,
    sizes: [Chip; 3],
    background: Chip,
    tint: Chip,
    lock: Chip,
    hide: Chip,
    options: Chip,
}

/// The popover's width and height, its padding and row pitch.
const SIZE: (i32, i32) = (276, 212);
const PAD: i32 = 10;
const ROW: i32 = 22;

fn chip(
    tree: &mut UiTree,
    root: usize,
    template: Option<usize>,
    layouts: &[&Arc<LayoutDesc>],
    id: &str,
    text: &str,
) -> Chip {
    let part = make_part(
        tree,
        root,
        template,
        id,
        art::CHIP,
        Some(art::CHIP_BORDER),
        1,
    );
    if let Some(part) = part {
        keep(tree, part);
        tree.set_bool(part, MOUSE_INVISIBLE, false);
    }
    let label = part.and_then(|p| {
        let label = count_label(
            tree,
            layouts,
            p,
            &format!("{id}Text"),
            Justification::Center,
        )?;
        keep(tree, label);
        tree.set_text(label, text);
        Some(label)
    });
    Chip { part, label }
}

/// `chip` put at `(x, y)` as wide as its label; its right edge.
fn place_chip(tree: &mut UiTree, chip: Chip, (x, y): (i32, i32)) -> i32 {
    let (Some(part), Some(label)) = (chip.part, chip.label) else {
        return x;
    };
    let text = tree.el(label).text().unwrap_or_default();
    let w = tree.text_line_width(label, &text).unwrap_or(24) + 14;
    tree.resize_to(part, w, 16);
    tree.move_to(part, x, y);
    place_label(tree, Some(label), UiBox::new(0, 0, w - 1, 15));
    x + w
}

fn light(tree: &mut UiTree, chip: Chip, on: bool) {
    if let Some(part) = chip.part {
        let image = if on { art::CHIP_ON } else { art::CHIP };
        if tree.el(part).image().map(|i| i.file) != Some(image) {
            tree.set_image(part, image, crate::compose::DRAW_OVER);
        }
    }
}

impl Popover {
    /// The popover made as its own root from `layout` (copying `template`, a plain field), hidden.
    pub(crate) fn make(
        tree: &mut UiTree,
        window: usize,
        template: usize,
        gameplay: Option<&Arc<LayoutDesc>>,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Option<Self> {
        let layout = Arc::clone(&tree.el(window).layout);
        let mut desc = (*tree.el(template).reference).clone();
        desc.id = tokenize("ModernChatSettings");
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
        tree.resize_to(root, SIZE.0, SIZE.1);
        let label = |tree: &mut UiTree, id: &str, text: &str, rgb: u32| {
            let label = count_label(tree, layouts, root, id, Justification::Left)?;
            keep(tree, label);
            for (channel, shift) in [("red", 16), ("green", 8), ("blue", 0)] {
                let _ = tree.set_float(
                    label,
                    tokenize(&format!("font_color1_{channel}")),
                    ((rgb >> shift) & 0xFF) as f32 / 255.0,
                );
            }
            tree.set_text_with_font(label, text, 0, 1);
            Some(label)
        };
        let gold = 0xD9C27A;
        let cream = 0xEFE8CC;
        let title = label(tree, "ModernChatSettingsTitle", "", gold);
        let w = SIZE.0 - 2 * PAD;
        let labels = vec![
            (
                label(tree, "ModernChatSettingsName", "Name", cream),
                (PAD, PAD + ROW, 50),
            ),
            (
                label(tree, "ModernChatSettingsLook", "LOOK", gold),
                (PAD, PAD + 2 * ROW + 4, w),
            ),
            (
                label(tree, "ModernChatSettingsFont", "Font size", cream),
                (PAD, PAD + 3 * ROW + 4, 70),
            ),
            (
                label(tree, "ModernChatSettingsLayout", "LAYOUT", gold),
                (PAD, PAD + 5 * ROW + 8, w),
            ),
        ];
        let template = Some(template);
        let make =
            |tree: &mut UiTree, id: &str, text: &str| chip(tree, root, template, layouts, id, text);
        let sizes = [
            make(tree, "ModernChatFontS", "S"),
            make(tree, "ModernChatFontM", "M"),
            make(tree, "ModernChatFontL", "L"),
        ];
        let background = make(tree, "ModernChatBackground", "Background");
        let tint = make(tree, "ModernChatTint", art::TINT_NAMES[0]);
        let lock = make(tree, "ModernChatLock", "Lock position");
        let hide = make(tree, "ModernChatHide", "Hide window");
        let options = make(tree, "ModernChatOptions", "Chat options...");
        let name = gameplay.and_then(|g| {
            entry_field(
                tree,
                g,
                layouts,
                root,
                ("ModernChatNameEntry", "ModernChatNameBox"),
                (PAD + 50, PAD + ROW - 2, w - 50, 20),
            )
        });
        if let Some((_, Some(pill))) = name {
            tree.set_image(pill, art::SEARCH_PILL, crate::compose::DRAW_OVER);
            tree.set_int(pill, crate::compose::NINE_SLICE, art::SEARCH_PILL_BORDER);
        }
        tree.set_visible(root, false);
        let popover = Self {
            root,
            title,
            labels,
            name,
            sizes,
            background,
            tint,
            lock,
            hide,
            options,
        };
        popover.arrange(tree);
        Some(popover)
    }

    fn arrange(&self, tree: &mut UiTree) {
        let w = SIZE.0 - 2 * PAD;
        place_label(
            tree,
            self.title,
            UiBox::new(PAD, PAD - 2, PAD + w, PAD + 13),
        );
        for &(label, (x, y, lw)) in &self.labels {
            place_label(tree, label, UiBox::new(x, y + 2, x + lw, y + 15));
        }
        if let Some((field, pill)) = self.name {
            place_entry(tree, field, pill, (PAD + 50, PAD + ROW - 2, w - 50, 20));
            tree.move_to(field, PAD + 58, PAD + ROW - 1);
            tree.resize_to(field, w - 66, 18);
        }
        let mut x = PAD + 72;
        for chip in self.sizes {
            x = place_chip(tree, chip, (x, PAD + 3 * ROW + 4)) + 4;
        }
        let x = place_chip(tree, self.background, (PAD, PAD + 4 * ROW + 4)) + 6;
        place_chip(tree, self.tint, (x, PAD + 4 * ROW + 4));
        place_chip(tree, self.lock, (PAD, PAD + 6 * ROW + 8));
        let x = place_chip(tree, self.hide, (PAD, PAD + 7 * ROW + 8)) + 6;
        place_chip(tree, self.options, (x, PAD + 7 * ROW + 8));
    }

    /// Whether the popover shows.
    pub(crate) fn shown(&self, tree: &UiTree) -> bool {
        tree.el(self.root).visible
    }

    /// The popover shown for window `n` (its title and choices `look`), over the window's
    /// options button `button` (screen box), right-aligned with it, or under the header when
    /// there is no room above.
    pub(crate) fn show(
        &self,
        tree: &mut UiTree,
        n: usize,
        look: &ChatLook,
        button: UiBox,
        header: i32,
    ) {
        if let Some(title) = self.title {
            tree.set_text_with_font(title, &format!("WINDOW {n} SETTINGS"), 0, 1);
        }
        if let Some((field, _)) = self.name {
            tree.set_text(field, &look.name);
        }
        self.set(tree, look);
        let x = (button.x1 + 1 - SIZE.0).max(0);
        let above = button.y0 - 4 - SIZE.1;
        let y = if above >= 0 { above } else { header + 4 };
        tree.set_object_position(self.root, x, y);
        tree.move_to(self.root, x, y);
        tree.set_visible(self.root, true);
    }

    pub(crate) fn hide(&self, tree: &mut UiTree) {
        tree.set_visible(self.root, false);
    }

    /// The chips lit for `look`.
    fn set(&self, tree: &mut UiTree, look: &ChatLook) {
        for (i, chip) in self.sizes.iter().enumerate() {
            light(tree, *chip, look.font == i as i32 - 1);
        }
        light(tree, self.background, look.background);
        light(tree, self.tint, look.background);
        if let Some(label) = self.tint.label {
            tree.set_text(label, art::TINT_NAMES[look.tint]);
        }
        light(tree, self.lock, look.locked);
        self.arrange(tree);
    }

    /// A press in the popover changes `look` or asks for an action.
    pub(crate) fn press(
        &self,
        tree: &mut UiTree,
        element: usize,
        look: &mut ChatLook,
    ) -> Option<SettingsAction> {
        let hit = |chip: Chip| chip.part == Some(element);
        let action = if let Some(i) = self.sizes.iter().position(|c| hit(*c)) {
            look.font = i as i32 - 1;
            SettingsAction::Changed
        } else if hit(self.background) {
            look.background = !look.background;
            SettingsAction::Changed
        } else if hit(self.tint) {
            look.tint = (look.tint + 1) % art::TINT_NAMES.len();
            SettingsAction::Changed
        } else if hit(self.lock) {
            look.locked = !look.locked;
            SettingsAction::Changed
        } else if hit(self.hide) {
            SettingsAction::Hide
        } else if hit(self.options) {
            SettingsAction::OpenOptions
        } else {
            return None;
        };
        self.set(tree, look);
        Some(action)
    }

    /// The name the entry holds, trimmed to 24 characters.
    pub(crate) fn typed_name(&self, tree: &UiTree) -> Option<String> {
        let (field, _) = self.name?;
        let text = tree.get(field)?.text()?;
        Some(text.trim().chars().take(24).collect())
    }

    /// Whether `element` is the popover or inside it.
    pub(crate) fn contains(&self, tree: &UiTree, element: usize) -> bool {
        tree.root_of(element) == self.root
    }
}
