//! `ChatWindow`: one of the four chat windows (`ChatWindow1`..`ChatWindow4` of the toolbar
//! layout), each a log (`ChatLog`), an entry line (`ChatEntry`), a move bar and an options
//! field. `RecvNotice_DisplayStringInfo` queues the strings whose text type passes the window's
//! filter (`m_chatBitfield`); the UI loop writes them to the log (`AllTextToLog`) in the font size
//! and colour the chat options give that type (`GetChatMessageInfo`).
//!
//! The opacity slider (`OpacitySliderBar`) sets the window's alpha as it moves and, on release,
//! asks for the window's opacity to be saved (`SetWindowOpacity`, then `UploadUISettings`); the
//! reload applies the saved opacity (`ChatWindow::RecvNotice_ReloadOptions`).
//!
//! The log's scroll bar (`ChatLogScrollbar`) locks the log while the player drags it
//! (`ChatWindow::ListenToMessage`'s `m_lockScroll`): a new line then does not scroll the log to
//! its end (`AllTextToLog`).
//!
//! The move bar's field shows the window's active look (`SetActiveWindow`: state `active` or
//! `inactive`). The entry line selects its text where retail does (`SelectAll` in
//! `HandleActivation` of a used line, `DoAssistedTell`, `DisplayCmdHistory`; `Deselect` in
//! `HandleCommandOrAliasKey`). A window's location is saved by the client's window-position code
//! (AGENTS.md: window positions), which stands in for `ChatWindow::LocationApplied` and the
//! `location_changed` listener (`SaveLocationsChanged`).
//!
//! Not modelled: the IME font (`kanjifont`, Japanese/Korean IME is not extended).

use std::collections::HashMap;

use crate::element::UiTree;
use crate::token::tokenize;

/// What the entry line does with its selection when a line is put in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntrySelection {
    /// Leave it (a reply or the guide's share).
    Keep,
    /// `UIElement_Text::SelectAll` (`DoAssistedTell`, `DisplayCmdHistory`).
    All,
    /// `UIElement_Text::Deselect` (`HandleCommandOrAliasKey`).
    None,
}

/// The chat options of one text type (`ChatWindow::ChatMessageInfo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChatMessageInfo {
    /// `GameplayOptionsProfile::GetChatFontSize`: the text element's font number.
    pub size: u32,
    /// `GetChatFontColor`: the text element's font colour number.
    pub color: u32,
    /// `GetChatPopupFlag`: a message of the type shows its window.
    pub popup: bool,
}

/// `TextType`s (`StringInfo` display types).
pub const TEXT_TYPE_ERROR: u32 = 0x1;
pub const TEXT_TYPE_COMBAT: u32 = 0x2;
pub const TEXT_TYPE_ADMIN: u32 = 0x4;
pub const TEXT_TYPE_STANDARD: u32 = 0x8;
pub const TEXT_TYPE_SAY: u32 = 0x10;
pub const TEXT_TYPE_TELL: u32 = 0x20;
pub const TEXT_TYPE_EMOTE: u32 = 0x40;
pub const TEXT_TYPE_LOG: u32 = 0x80;
pub const TEXT_TYPE_BROADCAST: u32 = 0x100;
pub const TEXT_TYPE_FELLOWSHIP: u32 = 0x1_0000;
pub const TEXT_TYPE_ALLEGIANCE: u32 = 0x2_0000;
pub const TEXT_TYPE_CHAT_ENTRY: u32 = 0x4_0000;
pub const TEXT_TYPE_GENERAL_CHANNEL: u32 = 0x8_0000;
pub const TEXT_TYPE_TRADE_CHANNEL: u32 = 0x10_0000;
pub const TEXT_TYPE_REGION_CHANNEL: u32 = 0x20_0000;
pub const TEXT_TYPE_FACTION_CHANNEL: u32 = 0x40_0000;

/// The chat text types with their default font colour, in `FlushChatFontOptions`' order (every
/// default font size is `Small_ChatFontSize`, 0).
pub const CHAT_TEXT_TYPES: [(u32, u32); 16] = [
    (TEXT_TYPE_ERROR, 0),
    (TEXT_TYPE_COMBAT, 1),
    (TEXT_TYPE_ADMIN, 2),
    (TEXT_TYPE_STANDARD, 3),
    (TEXT_TYPE_SAY, 4),
    (TEXT_TYPE_TELL, 5),
    (TEXT_TYPE_EMOTE, 6),
    (TEXT_TYPE_LOG, 4),
    (TEXT_TYPE_FELLOWSHIP, 7),
    (TEXT_TYPE_ALLEGIANCE, 8),
    (TEXT_TYPE_FACTION_CHANNEL, 4),
    (TEXT_TYPE_CHAT_ENTRY, 4),
    (TEXT_TYPE_GENERAL_CHANNEL, 4),
    (TEXT_TYPE_TRADE_CHANNEL, 4),
    (TEXT_TYPE_REGION_CHANNEL, 4),
    (TEXT_TYPE_BROADCAST, 9),
];

/// `Tokenize("ChatWindowN")`: the window's root element and its id in the profile's opacities
/// (`ChatWindow`'s `m_rootElement`).
pub fn chat_window_id(number: u32) -> u32 {
    tokenize(&format!("ChatWindow{number}"))
}

/// `Token_Scrollbar_y_position`: the slider's position, which is the opacity.
const Y_POSITION: u32 = tokenize("y_position");
/// `Token_ScrollbarMsg_widget_release`.
pub(crate) const MSG_WIDGET_RELEASE: u32 = tokenize("widget_release");
/// `Token_ScrollbarMsg_widget_press`: the bar's thumb taken (the scroll lock starts).
pub(crate) const MSG_WIDGET_PRESS: u32 = tokenize("widget_press");

/// A chat window's elements and state.
#[derive(Debug, Clone)]
pub(crate) struct ChatWindow {
    /// `m_windowNumber`, 1 to 4: the key of its filter.
    pub number: u32,
    /// `m_rootElement`: the window's id (`chat_window_id`).
    pub window_id: u32,
    /// `m_chatArea`, the window's root.
    pub root: usize,
    /// `m_opacitySlider`.
    pub opacity_slider: Option<usize>,
    /// `ChatLogScrollbar`: the log's scroll bar, whose press locks the log.
    pub log_scrollbar: Option<usize>,
    pub entry: Option<usize>,
    /// `m_textBox`: the log.
    pub log: Option<usize>,
    pub move_bar: Option<usize>,
    /// `m_moveBarField`: the move bar's active look.
    pub move_bar_field: Option<usize>,
    pub options_field: Option<usize>,
    pub options_button: Option<usize>,
    /// `m_toggleButton`: the id of the button that shows and hides the window.
    pub toggle_button: u32,
    pub zlevel_front: i32,
    pub prepopulated: bool,
    pub deactivate_pending: bool,
    pub lock_scroll: bool,
    pub initial_message_displayed: bool,
    /// `m_chatBitfield`: the text types the window shows.
    pub bitfield: u32,
    /// `m_chatLogPending`: (text type, text).
    pub pending: Vec<(u32, Vec<u16>)>,
    /// `m_messageInfoTable`.
    pub message_info: HashMap<u32, ChatMessageInfo>,
    /// The modern style's font size step over the chat options' sizes (-1, 0, 1).
    pub font_step: i32,
    /// The modern style's line choices (`modern_chat_settings`): a local time before each new
    /// line, old lines faded while the window is not in use, the most lines the log keeps.
    pub timestamps: bool,
    pub fade: bool,
    pub max_lines: Option<usize>,
    /// When each line still in the log arrived (`Instant`s, oldest first), for the fade.
    pub line_times: std::collections::VecDeque<std::time::Instant>,
    /// Whether the log's old lines are faded now.
    pub faded: bool,
    /// The lines the log has taken (the modern tabs' unread dot).
    pub lines_added: u64,
}

/// The time stamp's colour (the design's `#8fa89c`).
const STAMP_RGB: [f32; 3] = [
    0x8F as f32 / 255.0,
    0xA8 as f32 / 255.0,
    0x9C as f32 / 255.0,
];
/// How long a line stays before it fades, seconds (the design's 12).
const FADE_AFTER: u64 = 12;

impl ChatWindow {
    /// `ChatWindow::ChatWindow`: the root from the toolbar layout and its parts, hidden.
    pub(crate) fn create(
        tree: &mut UiTree,
        layout: &std::sync::Arc<crate::layout::LayoutDesc>,
        number: u32,
    ) -> Option<Self> {
        let root = tree.create_root(
            std::sync::Arc::clone(layout),
            tokenize(&format!("ChatWindow{number}")),
        )?;
        let child = |tree: &UiTree, name: &str| tree.child_recursive(root, tokenize(name));
        if let Some(options) = child(tree, "OptionsField") {
            tree.set_visible(options, false);
        }
        let zlevel_front = match tree.get(root)?.attribute(tokenize("zlevel_front")) {
            Some(crate::layout::AttributeValue::Int(z)) => *z,
            _ => 0,
        };
        let mut window = Self {
            number,
            window_id: chat_window_id(number),
            root,
            opacity_slider: child(tree, "OpacitySliderBar"),
            log_scrollbar: child(tree, "ChatLogScrollbar"),
            entry: child(tree, "ChatEntry"),
            log: child(tree, "ChatLog"),
            move_bar: child(tree, "MoveBar"),
            move_bar_field: child(tree, "MoveBarField"),
            options_field: child(tree, "OptionsField"),
            options_button: child(tree, "OptionsButton"),
            toggle_button: tokenize(&format!("ChatButtonToggle{number}")),
            zlevel_front,
            prepopulated: true,
            deactivate_pending: false,
            lock_scroll: false,
            initial_message_displayed: false,
            bitfield: 0xF000_0000,
            pending: Vec::new(),
            message_info: HashMap::new(),
            font_step: 0,
            timestamps: false,
            fade: false,
            max_lines: None,
            line_times: std::collections::VecDeque::new(),
            faded: false,
            lines_added: 0,
        };
        window.show(tree, false);
        Some(window)
    }

    /// `ChatWindow::SetOpacity(m_chatArea, opacity)`: the window's alpha and its children's.
    pub(crate) fn apply_opacity(&self, tree: &mut UiTree, opacity: f32) {
        tree.set_alpha_recursive(self.root, opacity);
    }

    /// `IsShown`.
    pub(crate) fn shown(&self, tree: &UiTree) -> bool {
        tree.get(self.root).is_some_and(|e| e.visible)
    }

    /// `ChatWindow::IsActive`: its entry line is the active element.
    pub(crate) fn is_active(&self, tree: &UiTree) -> bool {
        self.entry.is_some() && tree.active() == self.entry
    }

    /// `ChatWindow::IsActiveRoot`: the active element is in this window.
    pub(crate) fn is_active_root(&self, tree: &UiTree) -> bool {
        self.entry.is_some()
            && tree
                .active()
                .is_some_and(|active| tree.root_of(active) == self.root)
    }

    /// The toggle button (`IUIManager::GetElement(m_toggleButton)`), in any root.
    fn button(&self, tree: &UiTree) -> Option<usize> {
        tree.roots()
            .iter()
            .find_map(|&root| tree.child_recursive(root, self.toggle_button))
    }

    /// `ChatWindow::Show`: hiding takes the entry line's activation and marks the toggle button
    /// unchanged; showing drops a pending deactivation.
    pub(crate) fn show(&mut self, tree: &mut UiTree, shown: bool) {
        let was = self.shown(tree);
        tree.set_visible(self.root, shown);
        if was == shown {
            return;
        }
        if let Some(entry) = self.entry {
            tree.deactivate(entry);
        }
        if shown {
            self.deactivate_pending = false;
        } else if let Some(button) = self.button(tree) {
            tree.set_state(button, tokenize("unchanged"));
        }
    }

    /// `ChatWindow::SetActiveWindow`: the move bar's field takes the state `active` or `inactive`.
    pub(crate) fn set_active_window(&self, tree: &mut UiTree, active: bool) {
        if let Some(field) = self.move_bar_field {
            tree.set_state(field, tokenize(if active { "active" } else { "inactive" }));
        }
    }

    /// `ChatWindow::SetZLevel`.
    pub(crate) fn set_z_level(&self, tree: &mut UiTree, z: i32) {
        tree.set_z_level(self.root, z);
    }

    /// `ChatWindow::GetChatMessageInfo`: the options of the first chat text type in both `type`
    /// and the window's filter.
    pub(crate) fn chat_message_info(&self, text_type: u32) -> ChatMessageInfo {
        CHAT_TEXT_TYPES
            .iter()
            .filter(|(t, _)| text_type & t != 0 && self.bitfield & t != 0)
            .find_map(|(t, _)| self.message_info.get(t).copied())
            .unwrap_or_default()
    }

    /// `ChatWindow::RecvNotice_DisplayStringInfo`.
    pub(crate) fn display(&mut self, text_type: u32, text: &[u16]) {
        if text_type & self.bitfield != 0 {
            self.pending.push((text_type, text.to_vec()));
        }
    }

    /// The modern line limit: the oldest lines past `max_lines` leave the log.
    pub(crate) fn keep_lines(&mut self, tree: &mut UiTree) {
        let (Some(max), Some(log)) = (self.max_lines, self.log) else {
            return;
        };
        let Some(text) = tree.el_mut(log).text.as_mut() else {
            return;
        };
        let ends: Vec<usize> = text
            .glyphs
            .iter()
            .enumerate()
            .filter(|(_, g)| g.ch == 10)
            .map(|(i, _)| i)
            .collect();
        if ends.len() > max {
            let cut = ends[ends.len() - max - 1] + 1;
            text.delete_range(0, cut);
            let gone = ends.len() - max;
            for _ in 0..gone.min(self.line_times.len()) {
                self.line_times.pop_front();
            }
            tree.mark_changed();
        }
    }

    /// The modern fade: while the window is not in use (`in_use`: the pointer is over it or its
    /// entry is active), lines older than `FADE_AFTER` seconds are hidden; in use, every line
    /// shows. Only changes the log when a line crosses the age or the use changes.
    pub(crate) fn fade_lines(&mut self, tree: &mut UiTree, in_use: bool) {
        let Some(log) = self.log else {
            return;
        };
        let fading = self.fade && !in_use;
        let old = self
            .line_times
            .iter()
            .take_while(|t| t.elapsed().as_secs() >= FADE_AFTER)
            .count();
        let key = if fading { old } else { 0 };
        if !fading && !self.faded {
            return;
        }
        let Some(text) = tree.el_mut(log).text.as_mut() else {
            return;
        };
        // The lines are the newline-ended runs, oldest first, as many as the times kept; any
        // text before them (older than the times) counts as old.
        let lines = self.line_times.len();
        let ends: Vec<usize> = text
            .glyphs
            .iter()
            .enumerate()
            .filter(|(_, g)| g.ch == 10)
            .map(|(i, _)| i)
            .collect();
        let hide_before = if key == 0 {
            0
        } else {
            let first_tracked = ends.len().saturating_sub(lines);
            let last_old = first_tracked + key;
            ends.get(last_old - 1).map_or(0, |&e| e + 1)
        };
        let mut changed = false;
        for (i, glyph) in text.glyphs.iter_mut().enumerate() {
            let alpha = if i < hide_before { 0.0 } else { 1.0 };
            if glyph.color[3] != alpha {
                glyph.color[3] = alpha;
                changed = true;
            }
        }
        self.faded = key > 0;
        if changed {
            tree.mark_changed();
        }
    }

    /// `ChatWindow::AllTextToLog`: the pending strings to the end of the log; a hidden window's
    /// toggle button shows a change, a popup type shows the window.
    pub(crate) fn all_text_to_log(&mut self, tree: &mut UiTree) {
        let Some(log) = self.log.filter(|_| !self.pending.is_empty()) else {
            return;
        };
        let mut popup = false;
        for (text_type, text) in std::mem::take(&mut self.pending) {
            let info = self.chat_message_info(text_type);
            popup |= info.popup;
            // The modern font step (`modern_chat_settings`) over the options' size.
            let size = (info.size as i32 + self.font_step).clamp(0, 3) as u32;
            if self.timestamps {
                tree.set_font_num_and_color(log, size, info.color);
                let stamp = chrono::Local::now().format("%H:%M ").to_string();
                tree.append_text_rgb(log, &stamp, Some(STAMP_RGB));
            }
            tree.add_text_with_font(log, &text, size, info.color);
            self.line_times.push_back(std::time::Instant::now());
            self.lines_added += 1;
        }
        self.keep_lines(tree);
        self.faded = false;
        if !self.shown(tree) {
            if let Some(button) = self.button(tree) {
                tree.set_state(button, tokenize("changed"));
            }
        }
        if !self.lock_scroll {
            tree.scroll_to_end(log);
        }
        if popup {
            self.show(tree, true);
        }
    }

    /// The `UIMESSAGE_GLOBAL_LOOP` half of `ChatWindow::ListenToMessage`.
    pub(crate) fn global_loop(&mut self, tree: &mut UiTree) {
        self.all_text_to_log(tree);
        if std::mem::take(&mut self.deactivate_pending) {
            if let Some(entry) = self.entry {
                tree.deactivate(entry);
            }
        }
    }

    /// `ChatWindow::ProcessCommand`: the entry line's text, cleared, when there is any.
    pub(crate) fn process_command(&mut self, tree: &mut UiTree) -> Option<String> {
        let entry = self.entry?;
        let text = tree.get(entry)?.text()?;
        if text.is_empty() {
            return None;
        }
        tree.clear_text(entry);
        Some(text)
    }

    /// `ChatWindow::HandleEnterKey`: Enter in the active entry line sends it; otherwise it
    /// activates a shown window's entry line.
    pub(crate) fn handle_enter_key(&mut self, tree: &mut UiTree) -> Option<String> {
        let entry = self.entry?;
        if tree.active() == Some(entry) {
            self.deactivate_pending = false;
            tree.deactivate(entry);
            return self.process_command(tree);
        }
        if self.shown(tree) {
            tree.activate(entry);
        }
        None
    }

    /// `UIElement_Text::SetStringInfo` / `SetText` on the entry line in the chat-entry font,
    /// then `Activate`.
    pub(crate) fn prefill_entry(
        &mut self,
        tree: &mut UiTree,
        text: &[u16],
        selection: EntrySelection,
    ) {
        self.prepopulated = false;
        let Some(entry) = self.entry else {
            return;
        };
        let info = self.chat_message_info(TEXT_TYPE_CHAT_ENTRY);
        tree.clear_text(entry);
        tree.add_text_with_font(entry, text, info.size, info.color);
        match selection {
            EntrySelection::Keep => {}
            EntrySelection::All => tree.select_all(entry),
            EntrySelection::None => tree.deselect(entry),
        }
        tree.activate(entry);
    }

    /// `ChatWindow::HandleActivation`: the window comes to the front and becomes the active
    /// one; a prepopulated entry line is cleared, a used one is selected.
    pub(crate) fn handle_activation(&mut self, tree: &mut UiTree) {
        let Some(entry) = self.entry else {
            return;
        };
        if self.prepopulated {
            tree.clear_text(entry);
            self.prepopulated = false;
        } else {
            tree.select_all(entry);
        }
    }

    /// `ChatWindow::RecvNotice_ReloadChatSettings`: the options per type, the filter for this
    /// window (always with the chat entry type); a window whose filter became empty says how to
    /// set one up. The first reload writes the chat-entry greeting.
    pub(crate) fn reload_chat_settings(
        &mut self,
        tree: &mut UiTree,
        settings: &crate::chat::ChatSettings,
        strings: &crate::chat::ChatStrings,
    ) {
        self.message_info = CHAT_TEXT_TYPES
            .iter()
            .map(|&(t, _)| (t, settings.message_info(t)))
            .collect();
        let entry_info = self.message_info[&TEXT_TYPE_CHAT_ENTRY];
        if !self.initial_message_displayed {
            if let (Some(log), Some(text)) = (self.log, &strings.chat_entry) {
                tree.add_text_with_font(log, text, entry_info.size, entry_info.color);
            }
            self.initial_message_displayed = true;
        }
        let old = self.bitfield;
        self.bitfield = settings.filter(self.number) | TEXT_TYPE_CHAT_ENTRY;
        if self.bitfield == TEXT_TYPE_CHAT_ENTRY
            && old | TEXT_TYPE_CHAT_ENTRY != TEXT_TYPE_CHAT_ENTRY
        {
            if let Some(text) = &strings.usage_message {
                self.pending.push((TEXT_TYPE_STANDARD, text.clone()));
                self.bitfield |= TEXT_TYPE_STANDARD;
                self.all_text_to_log(tree);
                self.bitfield = TEXT_TYPE_CHAT_ENTRY;
            }
        }
        if let Some(entry) = self.entry {
            tree.set_font_num_and_color(entry, entry_info.size, entry_info.color);
        }

        // `ChatWindow::RecvNotice_ReloadOptions`: the saved opacity, if the profile has one.
        if let Some(&opacity) = settings.opacities.get(&self.window_id) {
            self.apply_opacity(tree, opacity);
            if let Some(slider) = self.opacity_slider {
                tree.set_float(slider, Y_POSITION, opacity);
            }
        }
    }

    /// `ChatWindow::KeyPress`'s page keys on the log.
    pub(crate) fn page(&self, tree: &mut UiTree, down: bool) {
        if let Some(log) = self.log {
            tree.page_text(log, down);
        }
    }

    /// `UIElement_Text::ScrollToHome` / `ScrollToEnd` on the log.
    pub(crate) fn scroll_log(&self, tree: &mut UiTree, end: bool) {
        if let Some(log) = self.log {
            if end {
                tree.scroll_to_end(log);
            } else {
                tree.scroll_to_home(log);
            }
        }
    }
}
