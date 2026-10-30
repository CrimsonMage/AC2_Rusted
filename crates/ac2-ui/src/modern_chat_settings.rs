//! The modern chat windows' settings (AGENTS.md: the HUD's two styles): what the design lets a
//! player choose per window that retail had no setting for, and the popover the window's options
//! button (`=`) opens to choose it. A window's name (its tab's label, else `ID_Option_Chat<n>`), a
//! font size step over the chat options' per-type sizes (small, medium, large), whether its glass
//! and the log's backing show and the backing's tint, a local time before each new line, old lines
//! faded while the window is not in use, the most lines the log keeps, and whether its place is
//! locked (no move, no resize). It also holds the window's retail choices from the chat options
//! page: the text types it shows (its filter) and the type its entry line sends (its devoted type,
//! `ID_Option_DevotedChatWindows`); a press sets that option on the page and runs the page's
//! Apply, so what goes to the server is the page's. The popover also hides the window (`ChatWindow::Show(0)`, as its window key does)
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
    pub timestamps: bool,
    pub fade: bool,
    /// The most lines the log keeps; 0: all (retail's log kept every line).
    pub lines: usize,
}

/// The line limits the chip steps through (0: all).
const LINE_LIMITS: [usize; 5] = [0, 100, 200, 500, 1000];

impl Default for ChatLook {
    fn default() -> Self {
        Self {
            name: String::new(),
            font: 0,
            background: true,
            tint: 0,
            locked: false,
            timestamps: false,
            fade: false,
            lines: 0,
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
            timestamps: layout.flag(&key("timestamps"), false),
            fade: layout.flag(&key("fade"), false),
            lines: layout.number(&key("lines"), 0).max(0) as usize,
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
        changed |= layout.set(&key("timestamps"), self.timestamps);
        changed |= layout.set(&key("fade"), self.fade);
        changed |= layout.set(&key("lines"), self.lines);
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
    /// Pop the window out of its tabs, or dock it into them.
    Tear,
    /// The options panel's chat page.
    OpenOptions,
    /// The window's filter: show or hide this text type (the chat options page's flag).
    Filter(u32),
    /// The window's devoted type: the next type a window can send.
    Devoted,
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
    timestamps: Chip,
    fade: Chip,
    lines: Chip,
    lock: Chip,
    tear: Chip,
    hide: Chip,
    options: Chip,
    /// The filter's chips (one per text type the page filters) and the devoted type's.
    types: Vec<(u32, Chip)>,
    devoted: Chip,
    /// The filter block's heading.
    shows: Option<usize>,
    /// The filter block's height (its chips wrap), laid out by `arrange`.
    types_height: std::cell::Cell<i32>,
}

/// The text types a window's filter takes (the page's rows with filter flags), with short names.
pub(crate) fn filter_types() -> Vec<(u32, &'static str)> {
    crate::chat_options_table::ROWS
        .iter()
        .filter(|r| {
            !r.hidden
                && r.text_type != crate::chat_options_table::WINDOW_SECTION
                && r.text_type != crate::chat_options_table::ENTRY_TEXT_TYPE
        })
        .map(|r| (r.text_type, short_name(r.name)))
        .collect()
}

/// A text type's row name shortened (`ID_Option_ChatGeneralChannelMessages`: `General`).
pub(crate) fn short_name(name: &'static str) -> &'static str {
    let name = name.strip_prefix("ID_Option_Chat").unwrap_or(name);
    let name = name.strip_suffix("Messages").unwrap_or(name);
    name.strip_suffix("Channel").unwrap_or(name)
}

/// The popover's width and its height without the filter block, its padding and row pitch.
const SIZE: (i32, i32) = (276, 252);
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
                (PAD, PAD + 7 * ROW + 6, w),
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
        let timestamps = make(tree, "ModernChatTimestamps", "Timestamps");
        let fade = make(tree, "ModernChatFade", "Fade old lines");
        let lines = make(tree, "ModernChatLines", "Max lines: all");
        let lock = make(tree, "ModernChatLock", "Lock position");
        let tear = make(tree, "ModernChatTear", "Dock into tabs");
        let hide = make(tree, "ModernChatHide", "Hide window");
        let options = make(tree, "ModernChatOptions", "Chat options...");
        let types = filter_types()
            .into_iter()
            .map(|(text_type, name)| {
                let id = format!("ModernChatShows{text_type:x}");
                (text_type, make(tree, &id, name))
            })
            .collect();
        let devoted = make(tree, "ModernChatDevoted", "Sends: Say");
        let shows = label(tree, "ModernChatSettingsShows", "SHOWS", gold);
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
        if let Some((field, pill)) = name {
            keep(tree, field);
            if let Some(pill) = pill {
                keep(tree, pill);
            }
        }
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
            timestamps,
            fade,
            lines,
            lock,
            tear,
            hide,
            options,
            types,
            devoted,
            shows,
            types_height: std::cell::Cell::new(0),
        };
        popover.arrange(tree);
        Some(popover)
    }

    fn arrange(&self, tree: &mut UiTree) {
        let w = SIZE.0 - 2 * PAD;
        // The filter block under the name: its chips wrapped in the popover's width, then the
        // devoted type; everything after it moves down by its height.
        let top = PAD + 3 * ROW + 2;
        let (mut x, mut y) = (PAD, top);
        for &(_, chip) in &self.types {
            let width = chip
                .label
                .and_then(|l| {
                    let text = tree.el(l).text().unwrap_or_default();
                    tree.text_line_width(l, &text)
                })
                .unwrap_or(24)
                + 14;
            if x + width > PAD + w {
                x = PAD;
                y += 20;
            }
            x = place_chip(tree, chip, (x, y)) + 4;
        }
        y += 22;
        place_chip(tree, self.devoted, (PAD, y));
        let extra = y + 22 - (PAD + 2 * ROW + 4);
        self.types_height.set(extra);
        // The popover's height first: a resize lays its children out again.
        let height = SIZE.1 + extra;
        if tree.el(self.root).rect.height() != height {
            tree.resize_to(self.root, SIZE.0, height);
        }
        let at = |row: i32, gap: i32| PAD + row * ROW + gap + extra;
        place_label(
            tree,
            self.title,
            UiBox::new(PAD, PAD - 2, PAD + w, PAD + 13),
        );
        place_label(
            tree,
            self.shows,
            UiBox::new(PAD, PAD + 2 * ROW + 6, PAD + w, PAD + 2 * ROW + 19),
        );
        for (i, &(label, (x, y, lw))) in self.labels.iter().enumerate() {
            // The name stays; the sections after the filter block move down.
            let y = if i == 0 { y } else { y + extra };
            place_label(tree, label, UiBox::new(x, y + 2, x + lw, y + 15));
        }
        if let Some((field, pill)) = self.name {
            place_entry(tree, field, pill, (PAD + 50, PAD + ROW - 2, w - 50, 20));
            tree.move_to(field, PAD + 58, PAD + ROW - 1);
            tree.resize_to(field, w - 66, 18);
        }
        let mut x = PAD + 72;
        for chip in self.sizes {
            x = place_chip(tree, chip, (x, at(3, 4))) + 4;
        }
        let x = place_chip(tree, self.background, (PAD, at(4, 4))) + 6;
        place_chip(tree, self.tint, (x, at(4, 4)));
        let x = place_chip(tree, self.timestamps, (PAD, at(5, 4))) + 6;
        place_chip(tree, self.fade, (x, at(5, 4)));
        place_chip(tree, self.lines, (PAD, at(6, 4)));
        let x = place_chip(tree, self.lock, (PAD, at(8, 8))) + 6;
        place_chip(tree, self.tear, (x, at(8, 8)));
        let x = place_chip(tree, self.hide, (PAD, at(9, 8))) + 6;
        place_chip(tree, self.options, (x, at(9, 8)));
    }

    /// The filter's chips lit for the types the window shows (`filter`, the profile's bits) and
    /// the devoted chip naming the type it sends.
    pub(crate) fn set_types(&self, tree: &mut UiTree, filter: u32, sends: u32) {
        for &(text_type, chip) in &self.types {
            light(tree, chip, filter & text_type != 0);
        }
        if let Some(label) = self.devoted.label {
            let name = crate::chat_options_table::ROWS
                .iter()
                .find(|r| r.text_type == sends)
                .map_or("Say", |r| short_name(r.name));
            tree.set_text(label, &format!("Sends: {name}"));
        }
        light(tree, self.devoted, true);
        self.arrange(tree);
    }

    /// Whether the popover shows.
    pub(crate) fn shown(&self, tree: &UiTree) -> bool {
        tree.el(self.root).visible
    }

    /// The popover shown for window `n` (its title and choices `look`), over the window's
    /// options button `button` (screen box), right-aligned with it, or under the header when
    /// there is no room above; its pop-out chip says what it does for a window `docked` or not.
    pub(crate) fn show(
        &self,
        tree: &mut UiTree,
        n: usize,
        look: &ChatLook,
        docked: bool,
        button: UiBox,
        header: i32,
    ) {
        if let Some(label) = self.tear.label {
            let text = if docked {
                "Pop out into a window"
            } else {
                "Dock into tabs"
            };
            tree.set_text(label, text);
        }
        if let Some(title) = self.title {
            tree.set_text_with_font(title, &format!("WINDOW {n} SETTINGS"), 0, 1);
        }
        if let Some((field, _)) = self.name {
            tree.set_text(field, &look.name);
        }
        self.set(tree, look);
        let x = (button.x1 + 1 - SIZE.0).max(0);
        let above = button.y0 - 4 - (SIZE.1 + self.types_height.get());
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
        light(tree, self.timestamps, look.timestamps);
        light(tree, self.fade, look.fade);
        light(tree, self.lines, look.lines != 0);
        if let Some(label) = self.lines.label {
            let text = match look.lines {
                0 => "Max lines: all".to_string(),
                n => format!("Max lines: {n}"),
            };
            tree.set_text(label, &text);
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
        } else if hit(self.timestamps) {
            look.timestamps = !look.timestamps;
            SettingsAction::Changed
        } else if hit(self.fade) {
            look.fade = !look.fade;
            SettingsAction::Changed
        } else if hit(self.lines) {
            let at = LINE_LIMITS
                .iter()
                .position(|&n| n == look.lines)
                .unwrap_or(0);
            look.lines = LINE_LIMITS[(at + 1) % LINE_LIMITS.len()];
            SettingsAction::Changed
        } else if hit(self.lock) {
            look.locked = !look.locked;
            SettingsAction::Changed
        } else if hit(self.tear) {
            SettingsAction::Tear
        } else if hit(self.hide) {
            SettingsAction::Hide
        } else if hit(self.options) {
            SettingsAction::OpenOptions
        } else if let Some(&(text_type, _)) = self.types.iter().find(|(_, c)| hit(*c)) {
            return Some(SettingsAction::Filter(text_type));
        } else if hit(self.devoted) {
            return Some(SettingsAction::Devoted);
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
