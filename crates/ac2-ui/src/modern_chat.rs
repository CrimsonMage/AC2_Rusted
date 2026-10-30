//! The modern style's chat windows (AGENTS.md: the HUD's two styles): each of `gmChatUI`'s four
//! `ChatWindow`s laid out as the design's. Retail framed a window in stone (`0x41005979`..
//! `0x41005988`) with a move bar along its top, the other windows' toggles at its top right, an
//! options tab at its left whose field held a vertical opacity slider, the log and its scroll bar,
//! the entry line under a rule (`0x41005995`), and a grip (`0x410023DC`). Here the window is teal
//! glass in a bronze line (gold while it is the active window) under a bronze header holding its
//! tab (its window key and its name, `ID_Option_Chat1`..`4`), the other windows' toggles, an
//! opacity slider (`op`) and the options button (`=`); the log sits on darker glass with the thin
//! scroll bar; the entry line has a channel pill (SAY, or the command it starts with) before its
//! box; and the grip is two light lines. The slider is the client's: it sets the window's opacity
//! as retail's did (`ChatWindow::SetOpacity`) and saves it on release (`SetWindowOpacity`). Every
//! other part is retail's, so the window's messages, keys, filters and resizing stay retail's.
//!
//! The art is restyled in the chat's copy of the toolbar layout ([`restyle`]); the places follow
//! the window's size each frame ([`ModernChat::follow`]).

use std::cell::Cell;
use std::sync::Arc;

use crate::compose::{DRAW_OVER, DRAW_OVER_FADE, NINE_SLICE};
use crate::element::{UiBox, UiTree};
use crate::layout::{ElementDesc, LayoutDesc, MediaDesc};
use crate::modern_art as art;
use crate::modern_chat_settings::{ChatLook, Popover, SettingsAction};
use crate::modern_drawer_bar::{count_label, place_label};
use crate::modern_inventory::{keep, make_part};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE};

/// The retail art the design's stands for.
const ART: [(u32, u32); 26] = [
    (0x4100_5979, art::CHAT_CLEAR),
    (0x4100_597A, art::CHAT_CLEAR),
    (0x4100_597B, art::CHAT_CLEAR),
    (0x4100_597C, art::CHAT_CLEAR),
    (0x4100_597D, art::CHAT_CLEAR),
    (0x4100_597E, art::CHAT_CLEAR),
    (0x4100_597F, art::CHAT_CLEAR),
    (0x4100_5980, art::CHAT_CLEAR),
    (0x4100_5981, art::CHAT_CLEAR),
    (0x4100_5982, art::CHAT_CLEAR),
    (0x4100_5983, art::CHAT_CLEAR),
    (0x4100_5984, art::CHAT_CLEAR),
    (0x4100_5985, art::CHAT_CLEAR),
    (0x4100_5986, art::CHAT_CLEAR),
    (0x4100_5987, art::CHAT_CLEAR),
    (0x4100_5988, art::CHAT_CLEAR),
    (0x4100_5995, art::CHAT_CLEAR),
    (0x4100_598B, art::SCROLL_UP),
    (0x4100_598C, art::SCROLL_UP_LIT),
    (0x4100_5993, art::SCROLL_DOWN),
    (0x4100_5994, art::SCROLL_DOWN_LIT),
    (0x4100_598D, art::THUMB_TOP),
    (0x4100_598F, art::THUMB_MID),
    (0x4100_5991, art::THUMB_BOTTOM),
    (0x4100_23DC, art::CHAT_GRIP),
    (0x4100_6F42, art::CHAT_CLEAR),
];
/// The options button's art (normal, then its other states) and the design's `=`.
const MENU_ART: [(u32, u32); 3] = [
    (0x4100_6F43, art::CHAT_MENU),
    (0x4100_6F44, art::CHAT_MENU_LIT),
    (0x4100_6F45, art::CHAT_MENU_LIT),
];

/// `ChatWindow`'s unnamed children: its frame, the toggles' field, the options tab, the log's
/// field, the entry's field and its rule, the grip.
const FRAME: u32 = 0xE930_EB61;
const TOGGLES: u32 = 0x52EF_E72C;
const OPTIONS_TAB: u32 = 0x0339_4037;
const LOG_FIELD: u32 = 0x8B36_899A;
const ENTRY_FIELD: u32 = 0x2757_2022;
const ENTRY_RULE: u32 = 0x797D_94EF;
const GRIP: u32 = 0xA372_7775;
/// The log's scroll bar's arrows.
const LOG_UP: u32 = 0xE0FF_9433;
const LOG_DOWN: u32 = 0xF94B_392B;

/// The header's height, the entry's, the padding, the tab's height, the slider's width.
const HEADER: i32 = 26;
const ENTRY: i32 = 28;
const PAD: i32 = 6;
const TAB_HEIGHT: i32 = 21;
const SLIDER: i32 = 54;
const MENU: (i32, i32) = (20, 19);
/// The least opacity the slider sets.
const MIN_OPACITY: f32 = 0.1;

/// The chat's copy of the toolbar layout with the chat's art the design's (`ART`, `MENU_ART`),
/// drawn over. Every element keeps its id and states.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    fn walk(d: &ElementDesc) -> ElementDesc {
        let mut d = d.clone();
        for state in &mut d.states {
            for media in &mut state.media {
                if let MediaDesc::Image {
                    file, draw_mode, ..
                } = media
                {
                    if let Some(&(_, to)) = ART
                        .iter()
                        .chain(MENU_ART.iter())
                        .find(|(from, _)| from == file)
                    {
                        *file = to;
                        // Retail's mode 3 art faded with the window's opacity; so does its own.
                        *draw_mode = if *draw_mode == 3 {
                            DRAW_OVER_FADE
                        } else {
                            DRAW_OVER
                        };
                    }
                }
            }
            for child in &mut state.children {
                *child = Arc::new(walk(child));
            }
        }
        d
    }
    let mut layout = layout.clone();
    // Every element of the chat's copy (a window's parts may be references to templates
    // elsewhere in the layout; the copy serves only the chat).
    for element in &mut layout.elements {
        *element = Arc::new(walk(element));
    }
    layout
}

/// One window's parts beside retail's.
#[derive(Debug, Clone)]
struct WindowParts {
    root: usize,
    glass: Option<usize>,
    header: Option<usize>,
    tab: Option<usize>,
    key: Option<usize>,
    key_label: Option<usize>,
    name: Option<usize>,
    op_label: Option<usize>,
    track: Option<usize>,
    fill: Option<usize>,
    knob: Option<usize>,
    log_backing: Option<usize>,
    entry_bar: Option<usize>,
    input: Option<usize>,
    pill: Option<usize>,
    pill_label: Option<usize>,
    /// What the window was laid out for: its size, its active look, its name, its pill and its
    /// opacity.
    laid_out: std::cell::RefCell<Option<(i32, i32, bool, String, String, i32)>>,
}

/// The chat windows' parts.
#[derive(Debug, Clone)]
pub struct ModernChat {
    windows: Vec<WindowParts>,
    /// The window whose slider is being dragged.
    dragging: Cell<Option<usize>>,
    /// Each window's choices (`modern_chat_settings`).
    looks: std::cell::RefCell<Vec<ChatLook>>,
    /// The settings popover and the window it is open for.
    popover: Option<Popover>,
    open_for: Cell<Option<usize>>,
}

/// What a message to the modern windows asks of the chat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatEvent {
    /// Window `i`'s opacity set by its slider (saved when `released`).
    Opacity {
        window: usize,
        opacity: f32,
        released: bool,
    },
    /// Window `i`'s choices changed (its font step and lock are the chat's to apply).
    Look(usize),
    /// Hide window `i` (`ChatWindow::Show(0)`).
    Hide(usize),
    /// Open the options panel's chat page.
    OpenOptions,
}

fn place(tree: &mut UiTree, element: Option<usize>, (x, y, w, h): (i32, i32, i32, i32)) {
    let Some(element) = element else {
        return;
    };
    let at = UiBox::new(x, y, x + w - 1, y + h - 1);
    if tree.el(element).rect != at {
        keep(tree, element);
        tree.resize_to(element, w.max(0), h.max(0));
        tree.move_to(element, x, y);
    }
}

fn set_color(tree: &mut UiTree, label: usize, rgb: u32) {
    for (channel, shift) in [("red", 16), ("green", 8), ("blue", 0)] {
        let _ = tree.set_float(
            label,
            tokenize(&format!("font_color1_{channel}")),
            ((rgb >> shift) & 0xFF) as f32 / 255.0,
        );
    }
}

impl ModernChat {
    /// The four windows' parts made (`roots`: each window's root).
    pub fn adopt(
        tree: &mut UiTree,
        roots: &[usize],
        gameplay: Option<&Arc<LayoutDesc>>,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Self {
        let windows: Vec<WindowParts> = roots
            .iter()
            .map(|&root| {
                // A plain field of the window to copy the parts from (the frame is a reference).
                let template = [ENTRY_FIELD, LOG_FIELD, TOGGLES]
                    .into_iter()
                    .filter_map(|id| tree.child_recursive(root, id))
                    .find(|&e| tree.el(e).reference.kind == tree.el(e).desc.kind);
                let part = |tree: &mut UiTree, id: &str, image: u32, nine: Option<i32>, z: i32| {
                    let part = make_part(tree, root, template, id, image, nine, z)?;
                    keep(tree, part);
                    tree.set_bool(part, MOUSE_INVISIBLE, true);
                    // The window's opacity fades its glass, as it faded retail's frame.
                    tree.set_image(part, image, DRAW_OVER_FADE);
                    Some(part)
                };
                let label = |tree: &mut UiTree, parent: Option<usize>, id: &str, j, rgb| {
                    let label = count_label(tree, layouts, parent?, id, j)?;
                    keep(tree, label);
                    set_color(tree, label, rgb);
                    Some(label)
                };
                let glass = part(
                    tree,
                    "ModernChatGlass",
                    art::CHAT_GLASS,
                    Some(art::CHAT_BORDER),
                    950,
                );
                let header = part(
                    tree,
                    "ModernChatHeader",
                    art::CHAT_HEADER,
                    Some(art::CHAT_BORDER),
                    940,
                );
                let log_backing = part(tree, "ModernChatLog", art::CHAT_LOG, Some(1), 930);
                let entry_bar = part(
                    tree,
                    "ModernChatEntryBar",
                    art::CHAT_ENTRY_BAR,
                    Some(art::CHAT_BORDER),
                    930,
                );
                let input = part(tree, "ModernChatInput", art::CHAT_INPUT, Some(4), 920);
                let tab = part(
                    tree,
                    "ModernChatTab",
                    art::CHAT_TAB,
                    Some(art::TAB_BORDER),
                    3,
                );
                let key = tab.and_then(|t| {
                    let key = make_part(
                        tree,
                        t,
                        template,
                        "ModernChatKey",
                        art::CHAT_KEY,
                        Some(2),
                        1,
                    )?;
                    keep(tree, key);
                    tree.set_bool(key, MOUSE_INVISIBLE, true);
                    Some(key)
                });
                let key_label = label(
                    tree,
                    key,
                    "ModernChatKeyText",
                    Justification::Center,
                    0xA88F48,
                );
                let name = label(tree, tab, "ModernChatName", Justification::Left, 0xFFF4CF);
                let pill = part(
                    tree,
                    "ModernChatPill",
                    art::CHAT_PILL,
                    Some(art::PILL_BORDER_CHAT),
                    3,
                );
                let pill_label = label(
                    tree,
                    pill,
                    "ModernChatPillText",
                    Justification::Center,
                    0xEFE8CC,
                );
                let op_label = label(
                    tree,
                    Some(root),
                    "ModernChatOp",
                    Justification::Right,
                    0xA88F48,
                );
                if let Some(l) = op_label {
                    tree.set_text_with_font(l, "op", 0, 1);
                }
                let track = part(
                    tree,
                    "ModernChatOpTrack",
                    art::SLIDER_TRACK,
                    Some(art::SLIDER_BORDER),
                    3,
                );
                let fill = part(
                    tree,
                    "ModernChatOpFill",
                    art::SLIDER_FILL,
                    Some(art::SLIDER_BORDER),
                    2,
                );
                let knob = part(tree, "ModernChatOpKnob", art::SLIDER_KNOB, None, 1);
                for p in [track, knob].into_iter().flatten() {
                    tree.set_bool(p, MOUSE_INVISIBLE, false);
                }
                // Retail's frame, options field (its vertical slider) and entry rule give way.
                for id in [FRAME, ENTRY_RULE] {
                    if let Some(e) = tree.child_recursive(root, id) {
                        tree.set_visible(e, false);
                    }
                }
                WindowParts {
                    root,
                    glass,
                    header,
                    tab,
                    key,
                    key_label,
                    name,
                    op_label,
                    track,
                    fill,
                    knob,
                    log_backing,
                    entry_bar,
                    input,
                    pill,
                    pill_label,
                    laid_out: Default::default(),
                }
            })
            .collect();
        // The settings popover, from the first window's layout and plain field.
        let popover = roots.first().and_then(|&root| {
            let template = [ENTRY_FIELD, LOG_FIELD, TOGGLES]
                .into_iter()
                .filter_map(|id| tree.child_recursive(root, id))
                .find(|&e| tree.el(e).reference.kind == tree.el(e).desc.kind)?;
            Popover::make(tree, root, template, gameplay, layouts)
        });
        Self {
            looks: std::cell::RefCell::new(vec![ChatLook::default(); windows.len()]),
            windows,
            dragging: Cell::new(None),
            popover,
            open_for: Cell::new(None),
        }
    }

    /// Each window's choices as `layout` keeps them.
    pub fn read_looks(&self, layout: &crate::hud_layout::HudLayout) {
        let mut looks = self.looks.borrow_mut();
        for (i, look) in looks.iter_mut().enumerate() {
            *look = ChatLook::read(layout, i + 1);
        }
        for w in &self.windows {
            *w.laid_out.borrow_mut() = None;
        }
    }

    /// Each window's choices written into `layout`; whether one changed.
    pub fn write_looks(&self, layout: &mut crate::hud_layout::HudLayout) -> bool {
        let looks = self.looks.borrow();
        let mut changed = false;
        for (i, look) in looks.iter().enumerate() {
            changed |= look.write(layout, i + 1);
        }
        changed
    }

    /// Window `i`'s choices.
    pub(crate) fn look_of(&self, i: usize) -> ChatLook {
        self.looks.borrow().get(i).cloned().unwrap_or_default()
    }

    /// Each shown window's parts laid out for its size, its active look (`active`: the window
    /// whose move bar shows `active`), its name (`names`, by window), the pill for its entry's
    /// text, and its opacity; run every frame.
    pub fn follow(&self, tree: &mut UiTree, names: &[String]) -> bool {
        // The name typed in the open popover is the window's.
        let mut renamed = false;
        if let (Some(popover), Some(i)) = (&self.popover, self.open_for.get()) {
            if let Some(name) = popover.typed_name(tree) {
                let mut looks = self.looks.borrow_mut();
                if let Some(look) = looks.get_mut(i) {
                    if look.name != name {
                        look.name = name;
                        renamed = true;
                    }
                }
            }
        }
        for (i, w) in self.windows.iter().enumerate() {
            if !tree.get(w.root).is_some_and(|e| e.visible) {
                continue;
            }
            let find = |tree: &UiTree, id: u32| tree.child_recursive(w.root, id);
            let r = tree.el(w.root).rect;
            let (width, height) = (r.width(), r.height());
            let active = find(tree, tokenize("MoveBarField"))
                .is_some_and(|f| tree.el(f).state == tokenize("active"));
            let look = self.looks.borrow().get(i).cloned().unwrap_or_default();
            let name = if look.name.is_empty() {
                names.get(i).cloned().unwrap_or_default()
            } else {
                look.name.clone()
            };
            let entry_text = find(tree, tokenize("ChatEntry"))
                .and_then(|e| tree.el(e).text())
                .unwrap_or_default();
            let pill = channel(&entry_text);
            let opacity = opacity(tree, w.root);
            let key = (
                width,
                height,
                active,
                name.clone(),
                pill.clone(),
                (opacity * 100.0) as i32,
            );
            if w.laid_out.borrow().as_ref() == Some(&key) {
                continue;
            }
            *w.laid_out.borrow_mut() = Some(key);
            if let Some(glass) = w.glass {
                let image = if active {
                    art::CHAT_GLASS_ACTIVE
                } else {
                    art::CHAT_GLASS
                };
                tree.set_image(glass, image, DRAW_OVER_FADE);
                tree.set_int(glass, NINE_SLICE, art::CHAT_BORDER);
            }
            place(tree, w.glass, (0, 0, width, height));
            // The background choice: the glass and the log's backing, in the chosen tint.
            for part in [w.glass, w.log_backing].into_iter().flatten() {
                if tree.el(part).visible != look.background {
                    tree.set_visible(part, look.background);
                }
            }
            if let Some(backing) = w.log_backing {
                let tint = art::CHAT_TINTS[look.tint.min(art::CHAT_TINTS.len() - 1)];
                tree.set_image(backing, tint, DRAW_OVER_FADE);
                tree.set_int(backing, NINE_SLICE, 1);
            }
            // A locked window shows no grip.
            if let Some(grip) = find(tree, GRIP) {
                if tree.el(grip).visible == look.locked {
                    tree.set_visible(grip, !look.locked);
                }
            }
            place(tree, w.header, (0, 0, width, HEADER));
            // The move bar over the whole header (its grip mark centred on top).
            let bar = find(tree, tokenize("MoveBar"));
            place(tree, bar, (0, 0, width, HEADER));
            if let Some(bar) = bar {
                for part in tree.children(bar).to_vec() {
                    place(tree, Some(part), (width / 2 - 79, 0, 158, 7));
                }
            }
            // The tab: the window key's chip, then the name.
            let key_text = format!("F{}", i + 1);
            let key_w = 18;
            let name_w = tree
                .text_line_width(w.name.unwrap_or(w.root), &name)
                .unwrap_or(60)
                .min(96);
            let tab_w = 6 + key_w + 4 + name_w + 6;
            place(tree, w.tab, (4, HEADER - TAB_HEIGHT, tab_w, TAB_HEIGHT));
            place(tree, w.key, (6, 5, key_w, 12));
            if let Some(label) = w.key_label {
                tree.set_text_with_font(label, &key_text, 0, 1);
            }
            place_label(tree, w.key_label, UiBox::new(0, 0, key_w - 1, 11));
            if let Some(label) = w.name {
                tree.set_text_with_font(label, &name, 0, 1);
            }
            place_label(
                tree,
                w.name,
                UiBox::new(6 + key_w + 4, 3, 6 + key_w + 4 + name_w, TAB_HEIGHT - 4),
            );
            // The other windows' toggles after the tab.
            let toggles = find(tree, TOGGLES);
            place(tree, toggles, (4 + tab_w + 4, 9, 60, 12));
            // The options button (`=`) at the header's right, the opacity slider before it.
            let menu_x = width - PAD - MENU.0;
            let tab_holder = find(tree, OPTIONS_TAB);
            place(
                tree,
                tab_holder,
                (menu_x, (HEADER - MENU.1) / 2, MENU.0, MENU.1),
            );
            if let Some(button) = find(tree, tokenize("OptionsButton")) {
                crate::modern_inventory::remap_art(tree, button, &MENU_ART, false);
                place(tree, Some(button), (0, 0, MENU.0, MENU.1));
            }
            if let Some(field) = find(tree, tokenize("OptionsField")) {
                if tree.el(field).visible {
                    tree.set_visible(field, false);
                }
            }
            let track_x = menu_x - 8 - SLIDER;
            let track_y = HEADER / 2 - 3;
            place(tree, w.track, (track_x, track_y, SLIDER, 6));
            let knob_x = track_x + (opacity * SLIDER as f32).round() as i32;
            place(
                tree,
                w.fill,
                (track_x, track_y, (knob_x - track_x).max(6), 6),
            );
            place(
                tree,
                w.knob,
                (
                    knob_x - art::KNOB / 2,
                    HEADER / 2 - art::KNOB / 2,
                    art::KNOB,
                    art::KNOB,
                ),
            );
            place_label(
                tree,
                w.op_label,
                UiBox::new(track_x - 22, track_y - 4, track_x - 4, track_y + 9),
            );
            // The log on its darker glass, the thin bar at its right.
            let log_h = height - HEADER - ENTRY - 2;
            place(tree, w.log_backing, (1, HEADER, width - 2, log_h + 2));
            let log_field = find(tree, LOG_FIELD);
            place(
                tree,
                log_field,
                (PAD, HEADER + 2, width - 2 * PAD, log_h - 2),
            );
            let lw = width - 2 * PAD;
            let lh = log_h - 2;
            let bar_w = art::SCROLL_WIDTH;
            place(
                tree,
                find(tree, tokenize("ChatLog")),
                (0, 0, lw - bar_w - 2, lh),
            );
            place(tree, find(tree, LOG_UP), (lw - bar_w, 0, bar_w, bar_w));
            place(
                tree,
                find(tree, LOG_DOWN),
                (lw - bar_w, lh - bar_w, bar_w, bar_w),
            );
            let scrollbar = find(tree, tokenize("ChatLogScrollbar"));
            place(tree, scrollbar, (lw - bar_w, bar_w, bar_w, lh - 2 * bar_w));
            if let Some(scrollbar) = scrollbar {
                let mut under = tree.children(scrollbar).to_vec();
                while let Some(part) = under.pop() {
                    under.extend(tree.children(part).iter().copied());
                    let r = tree.el(part).rect;
                    if r.width() != bar_w {
                        tree.resize_to(part, bar_w, r.height());
                    }
                }
            }
            // The entry: its bar, the channel pill, the box and the line.
            place(tree, w.entry_bar, (1, height - ENTRY, width - 2, ENTRY - 1));
            let pill_w = w
                .pill_label
                .and_then(|l| tree.text_line_width(l, &pill))
                .unwrap_or(30)
                + 14;
            place(tree, w.pill, (PAD, height - ENTRY + 6, pill_w, 16));
            if let Some(label) = w.pill_label {
                tree.set_text_with_font(label, &pill, 0, 1);
            }
            place_label(tree, w.pill_label, UiBox::new(0, 1, pill_w - 1, 14));
            let input_x = PAD + pill_w + 6;
            let input_w = width - input_x - PAD - 14;
            place(tree, w.input, (input_x, height - ENTRY + 4, input_w, 20));
            let entry_field = find(tree, ENTRY_FIELD);
            place(
                tree,
                entry_field,
                (input_x, height - ENTRY + 4, input_w, 20),
            );
            place(
                tree,
                find(tree, tokenize("ChatEntry")),
                (4, 1, input_w - 8, 18),
            );
            place(tree, find(tree, GRIP), (width - 12, height - 12, 12, 12));
        }
        renamed
    }

    /// A message to the modern windows: the opacity slider's drag (a press on a window's track
    /// or knob starts it, the pointer moving sets the opacity, a release ends and saves it), the
    /// options button opening and closing the settings popover, the popover's chips, and a press
    /// elsewhere closing it.
    pub fn message(
        &self,
        tree: &mut UiTree,
        message: &crate::mouse::UiMessage,
    ) -> Option<ChatEvent> {
        use crate::mouse::{GLOBAL_MOUSE_DOWN, GLOBAL_MOUSE_MOVE, GLOBAL_MOUSE_UP};
        use crate::token::{MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};
        // The options button (`=`) opens the window's popover, or closes it.
        if message.message == MSG_BUTTON_RELEASE_ON {
            if let Some(i) = self.windows.iter().position(|w| {
                message.element.is_some()
                    && tree.child_recursive(w.root, tokenize("OptionsButton")) == message.element
            }) {
                self.toggle_popover(tree, i);
                return None;
            }
        }
        if let (Some(popover), Some(i)) = (&self.popover, self.open_for.get()) {
            if message.message == MSG_MOUSE_PRESS_TOP {
                if let Some(element) = message.element.filter(|&e| popover.contains(tree, e)) {
                    let mut looks = self.looks.borrow_mut();
                    let look = looks.get_mut(i)?;
                    return match popover.press(tree, element, look)? {
                        SettingsAction::Changed => {
                            *self.windows[i].laid_out.borrow_mut() = None;
                            Some(ChatEvent::Look(i))
                        }
                        SettingsAction::Hide => {
                            popover.hide(tree);
                            self.open_for.set(None);
                            Some(ChatEvent::Hide(i))
                        }
                        SettingsAction::OpenOptions => {
                            popover.hide(tree);
                            self.open_for.set(None);
                            Some(ChatEvent::OpenOptions)
                        }
                    };
                }
            }
            // A press away from the popover and the window's button closes it.
            if message.message == GLOBAL_MOUSE_DOWN && message.element.is_none() {
                let (x, y) = tree.pointer();
                let inside = |e: usize| {
                    let b = tree.screen_box(e);
                    x >= b.x0 && x <= b.x1 && y >= b.y0 && y <= b.y1
                };
                let button = tree.child_recursive(self.windows[i].root, tokenize("OptionsButton"));
                if !inside(popover.root) && !button.is_some_and(inside) {
                    popover.hide(tree);
                    self.open_for.set(None);
                }
            }
        }
        let pressed = (message.message == MSG_MOUSE_PRESS_TOP)
            .then(|| {
                self.windows.iter().position(|w| {
                    message.element.is_some()
                        && (message.element == w.track || message.element == w.knob)
                })
            })
            .flatten();
        if let Some(i) = pressed {
            self.dragging.set(Some(i));
        }
        let i = self.dragging.get()?;
        let released = message.message == GLOBAL_MOUSE_UP;
        if pressed.is_none() && message.message != GLOBAL_MOUSE_MOVE && !released {
            return None;
        }
        if released {
            self.dragging.set(None);
        }
        let track = tree.screen_box(self.windows.get(i)?.track?);
        let share = (tree.pointer().0 - track.x0) as f32 / track.width().max(1) as f32;
        Some(ChatEvent::Opacity {
            window: i,
            opacity: share.clamp(MIN_OPACITY, 1.0),
            released,
        })
    }

    /// Window `i`'s popover opened over its options button, or closed if open for it.
    fn toggle_popover(&self, tree: &mut UiTree, i: usize) {
        let Some(popover) = &self.popover else {
            return;
        };
        if self.open_for.get() == Some(i) && popover.shown(tree) {
            popover.hide(tree);
            self.open_for.set(None);
            return;
        }
        let w = &self.windows[i];
        let Some(button) = tree.child_recursive(w.root, tokenize("OptionsButton")) else {
            return;
        };
        let at = tree.screen_box(button);
        let header = tree.screen_box(w.root).y0 + HEADER;
        let look = self.looks.borrow().get(i).cloned().unwrap_or_default();
        popover.show(tree, i + 1, &look, at, header);
        self.open_for.set(Some(i));
    }
}

/// A window's opacity: its root's alpha value (whole until set).
fn opacity(tree: &UiTree, root: usize) -> f32 {
    let a = tree.el(root).alpha_value;
    if (0.0..=1.0).contains(&a) {
        a
    } else {
        1.0
    }
}

/// The channel pill's text for the entry line `text`: the command it starts with (`/tell` gives
/// TELL), else SAY.
fn channel(text: &str) -> String {
    text.strip_prefix('/')
        .and_then(|rest| rest.split_whitespace().next())
        .map(|word| word.to_uppercase())
        .filter(|word| !word.is_empty())
        .unwrap_or_else(|| "SAY".to_string())
}
