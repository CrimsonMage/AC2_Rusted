//! `gmChatUI`: the toolbar's chat, four [`ChatWindow`](crate::chat_window)s of which the first
//! starts shown and focused. `ListenToMessage` takes the global key presses: the window keys
//! (`0x41000041`..`44`) show and hide a window, the rest go to the target window
//! (`DispatchKeypressToWindow` / `DetermineTargetWindow`): Enter opens or sends its entry line,
//! the command and alias keys start a line with their prefix, the reply key starts a reply, and
//! the history, paging and Escape keys go to `ChatWindow::KeyPress`.
//!
//! A sent line leaves as [`ChatRequest::Line`] for `gmCWInterface::ProcessText`; the history keys
//! leave as [`ChatRequest::History`] for the history the interface keeps, which answers through
//! [`ChatUi::set_entry_text`] or [`ChatUi::assisted_tell`].
//!
//! The spew box (`chat_spew`) shows the error-type lines over the world for their `decay_time`.
//! Window opacity is kept per window (`ChatSettings::opacities`, `chat_window`).
//!
//! Not modelled yet: `AltKeyDown` (Alt is not a meta key of the input manager).

use std::collections::HashMap;
use std::sync::Arc;

use crate::chat_spew::SpewBox;
use crate::chat_window::{
    ChatMessageInfo, ChatWindow, EntrySelection, CHAT_TEXT_TYPES, MSG_WIDGET_PRESS,
    MSG_WIDGET_RELEASE, TEXT_TYPE_ERROR,
};
use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS, MSG_MOUSE_RELEASE};

/// `StringInfo::SetTableEnum`'s group and the chat's string tables.
pub const STRING_TABLE_GROUP: u32 = 0x4000_0004;
pub const CHAT_STRING_TABLE: u32 = 0x4100_0018;
pub const REPLY_STRING_TABLE: u32 = 0x4100_0007;

const ENTER: u32 = 0x4000_0018;
const ESCAPE: u32 = 0x4000_001E;
const UP: u32 = 0x4000_0015;
const DOWN: u32 = 0x4000_0016;
const PAGE_UP: u32 = 0x4000_0019;
const PAGE_DOWN: u32 = 0x4000_001A;
const HOME: u32 = 0x4000_001B;
const END: u32 = 0x4000_001C;
const COMMAND_KEY: u32 = 0x4100_0010;
const ALIAS_KEY: u32 = 0x4100_0011;
const REPLY_KEY: u32 = 0x4100_0049;
const WINDOW_KEYS: [u32; 4] = [0x4100_0041, 0x4100_0042, 0x4100_0043, 0x4100_0044];

/// The chat part of `GameplayOptionsProfile`: per-window filters (keyed 1 to 4) and the font
/// size, font colour and popup flag of each text type. The default is `Clear`'s.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatSettings {
    pub filters: HashMap<u32, u32>,
    pub font_sizes: HashMap<u32, u32>,
    pub font_colors: HashMap<u32, u32>,
    pub popups: HashMap<u32, bool>,
    /// The text type each window shows (Legions' version-6 table, `FUN_005bfd50`).
    pub window_text_types: HashMap<u32, u32>,
    /// `GameplayOptionsProfile::InqWindowOpacity`: each window's saved opacity, by window id.
    pub opacities: HashMap<u32, f32>,
}

impl Default for ChatSettings {
    /// `GameplayOptionsProfile::Clear`: no filters, `FlushChatFontOptions`' sizes and colours,
    /// broadcasts pop up.
    fn default() -> Self {
        Self {
            filters: HashMap::new(),
            font_sizes: CHAT_TEXT_TYPES.iter().map(|&(t, _)| (t, 0)).collect(),
            font_colors: CHAT_TEXT_TYPES.iter().copied().collect(),
            popups: HashMap::from([(crate::chat_window::TEXT_TYPE_BROADCAST, true)]),
            window_text_types: HashMap::new(),
            opacities: HashMap::new(),
        }
    }
}

impl ChatSettings {
    /// `GameplayOptionsProfile::GetFilter` (0 when unset).
    pub fn filter(&self, window: u32) -> u32 {
        self.filters.get(&window).copied().unwrap_or(0)
    }

    /// `FUN_005bfd50`: the text type window `window` shows (0x10 when unset).
    pub fn window_text_type(&self, window: u32) -> u32 {
        self.window_text_types
            .get(&window)
            .copied()
            .unwrap_or(crate::chat_options_table::DEFAULT_WINDOW_TEXT_TYPE)
    }

    pub(crate) fn message_info(&self, text_type: u32) -> ChatMessageInfo {
        ChatMessageInfo {
            size: self.font_sizes.get(&text_type).copied().unwrap_or(0),
            color: self.font_colors.get(&text_type).copied().unwrap_or(0),
            popup: self.popups.get(&text_type).copied().unwrap_or(false),
        }
    }
}

/// The chat's strings, read with the HUD (`ID_Chat_*` of the chat table, `ID_Reply`).
#[derive(Debug, Clone, Default)]
pub struct ChatStrings {
    pub command_prefix: Option<Vec<u16>>,
    pub alias_prefix: Option<Vec<u16>>,
    pub unknown_command: Option<Vec<u16>>,
    pub usage_message: Option<Vec<u16>>,
    pub chat_entry: Option<Vec<u16>>,
    pub reply: Option<Vec<u16>>,
    /// `ID_Chat_AssistedTell`, with its `prefix` and `target` variables.
    pub assisted_tell: Option<ac2_db::StringTableString>,
}

/// Which history a history key steps through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatHistory {
    /// `InqPrevCmdHistoryItem` / `InqNextCmdHistoryItem`: the lines sent.
    Command,
    /// `InqPrevRetellHistoryItem`: the players told.
    Retell,
    /// `InqPrevReplyHistoryItem`: the players who told.
    Reply,
}

/// What the chat asks of the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRequest {
    /// A line entered (`gmCWInterface::ProcessText`).
    Line(String),
    /// A history key: the previous (`prev`) or next item, for [`ChatUi::set_entry_text`]
    /// (commands) or [`ChatUi::assisted_tell`] (retell and reply).
    History { history: ChatHistory, prev: bool },
}

/// The modifier keys as `ActionInterpreter` reports them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

/// `gmChatUI`'s state.
#[derive(Debug, Clone)]
pub struct ChatUi {
    windows: Vec<ChatWindow>,
    last_focus: Option<usize>,
    strings: ChatStrings,
    requests: Vec<ChatRequest>,
    /// The windows whose opacity the player set on release: window id and opacity.
    opacity_requests: Vec<(u32, f32)>,
    /// `gmChatUI::_spewBoxPending` / `_spewBoxItems`; `None` when the layout has no spew box.
    spew: Option<SpewBox>,
    /// The modern style's windows (`modern_chat`); `None` in the classic style.
    modern: Option<crate::modern_chat::ModernChat>,
    /// The windows' names (`ID_Option_Chat1`..`4`), once the UI table renders them.
    names: Vec<String>,
    /// A modern window's popover asked for the options panel's chat page.
    open_options: bool,
    /// The modern windows' choices changed since the layout was last written.
    looks_changed: bool,
}

impl ChatUi {
    /// `gmChatUI::gmChatUI` / `Init`: the four windows from the toolbar layout; the first is
    /// shown and active. The spew box is made from the spew layout (`messages`).
    pub fn create(
        tree: &mut UiTree,
        toolbar: &Arc<LayoutDesc>,
        messages: &Arc<LayoutDesc>,
        strings: ChatStrings,
    ) -> Self {
        let windows: Vec<_> = (1..=4)
            .filter_map(|n| ChatWindow::create(tree, toolbar, n))
            .collect();
        let spew = SpewBox::create(tree, messages);
        let mut chat = Self {
            windows,
            last_focus: None,
            strings,
            requests: Vec::new(),
            opacity_requests: Vec::new(),
            spew,
            modern: None,
            names: Vec::new(),
            open_options: false,
            looks_changed: false,
        };
        if !chat.windows.is_empty() {
            chat.windows[0].show(tree, true);
            chat.set_active_window(tree, 0);
        }
        chat
    }

    /// The modern style's windows laid over the four (`modern_chat`); `layouts` give its small
    /// font.
    pub fn set_modern(
        &mut self,
        tree: &mut UiTree,
        gameplay: Option<&Arc<LayoutDesc>>,
        layouts: &[&Arc<LayoutDesc>],
    ) {
        let roots: Vec<usize> = self.windows.iter().map(|w| w.root).collect();
        self.modern = Some(crate::modern_chat::ModernChat::adopt(
            tree, &roots, gameplay, layouts,
        ));
    }

    /// Window `i`'s modern choices applied: its font step and time stamps (new lines), its line
    /// fade and limit, and its lock.
    fn apply_look(&mut self, tree: &mut UiTree, i: usize) {
        let Some(look) = self.modern.as_ref().map(|m| m.look_of(i)) else {
            return;
        };
        if let Some(window) = self.windows.get_mut(i) {
            window.font_step = look.font;
            window.timestamps = look.timestamps;
            window.fade = look.fade;
            window.max_lines = (look.lines != 0).then_some(look.lines);
            window.keep_lines(tree);
            tree.set_locked(window.root, look.locked);
        }
        self.looks_changed = true;
    }

    /// The modern windows' choices as the HUD's layout settings keep them.
    pub fn apply_layout(&mut self, tree: &mut UiTree, layout: &crate::hud_layout::HudLayout) {
        if let Some(modern) = &self.modern {
            modern.read_looks(layout);
        }
        for i in 0..self.windows.len() {
            self.apply_look(tree, i);
        }
        self.looks_changed = false;
    }

    /// The modern windows' choices written into the HUD's layout settings; whether they changed.
    pub fn export_layout(&self, layout: &mut crate::hud_layout::HudLayout) -> bool {
        self.modern.as_ref().is_some_and(|m| m.write_looks(layout))
    }

    /// Whether a modern window's popover asked for the chat options page since the last call.
    pub fn take_open_options(&mut self) -> bool {
        std::mem::take(&mut self.open_options)
    }

    /// The modern windows laid out for their sizes and states (every frame; nothing in classic);
    /// their names rendered from the UI table until it has them.
    pub fn modern_tick(&mut self, tree: &mut UiTree, strings: &dyn crate::ui_string::UiStrings) {
        let Some(modern) = &self.modern else {
            return;
        };
        if self.names.iter().all(|n| n.is_empty()) {
            self.names = crate::chat_options_table::WINDOW_NAMES
                .iter()
                .map(|name| {
                    strings
                        .render_string(&crate::ui_string::UiString::ui(tokenize(name), Vec::new()))
                })
                .collect();
        }
        if modern.follow(tree, &self.names) {
            self.looks_changed = true;
        }
        // Old lines fade while a window is not in use: the pointer away and its entry idle.
        let (x, y) = tree.pointer();
        for window in &mut self.windows {
            if !window.fade && !window.faded {
                continue;
            }
            let at = tree.screen_box(window.root);
            let over = x >= at.x0 && x <= at.x1 && y >= at.y0 && y <= at.y1;
            let typing = window.entry.is_some() && tree.active == window.entry;
            window.fade_lines(tree, over || typing);
        }
    }

    /// Window `i` (tests reach its log through it).
    #[cfg(test)]
    pub(crate) fn window_mut(&mut self, i: usize) -> &mut ChatWindow {
        &mut self.windows[i]
    }

    /// The windows' roots, first to fourth.
    pub fn roots(&self) -> Vec<usize> {
        self.windows.iter().map(|w| w.root).collect()
    }

    /// The window entry lines, first to fourth.
    pub fn entries(&self) -> Vec<Option<usize>> {
        self.windows.iter().map(|w| w.entry).collect()
    }

    /// The window logs, first to fourth.
    pub fn logs(&self) -> Vec<Option<usize>> {
        self.windows.iter().map(|w| w.log).collect()
    }

    /// What the chat asked since the last call.
    pub fn take_requests(&mut self) -> Vec<ChatRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The windows whose opacity the player set since the last call (`ChatWindow`'s
    /// `SetWindowOpacity`, sent with `UploadUISettings`).
    pub fn take_opacity_requests(&mut self) -> Vec<(u32, f32)> {
        std::mem::take(&mut self.opacity_requests)
    }

    /// `gmChatUI::SetActiveWindow`.
    fn set_active_window(&mut self, tree: &mut UiTree, window: usize) {
        for (i, w) in self.windows.iter().enumerate() {
            w.set_active_window(tree, i == window);
        }
        self.last_focus = Some(window);
    }

    /// `gmChatUI::UpdateZLevels`: the active window at `top`, the others behind it in order.
    fn update_z_levels(&self, tree: &mut UiTree, top: i32) {
        for (i, window) in self.windows.iter().enumerate() {
            let z = if window.is_active(tree) {
                top
            } else {
                i as i32 + 1 + top
            };
            window.set_z_level(tree, z);
        }
    }

    /// `gmChatUI::DetermineTargetWindow`: the window holding the active element, else the last
    /// focused one if shown, else the first shown one; with `activate`, the last focused one is
    /// shown when none is.
    fn determine_target_window(&mut self, tree: &mut UiTree, activate: bool) -> Option<usize> {
        if let Some(i) = self
            .windows
            .iter()
            .position(|w| w.is_active_root(tree) && w.shown(tree))
        {
            return Some(i);
        }
        let last = self.last_focus?;
        if !self.windows[last].shown(tree) {
            if let Some(i) = self.windows.iter().position(|w| w.shown(tree)) {
                return Some(i);
            }
            if !activate {
                return None;
            }
            self.windows[last].show(tree, true);
        }
        Some(last)
    }

    /// One UI message (`gmChatUI::ListenToMessage` and each `ChatWindow::ListenToMessage`).
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage, modifiers: Modifiers) {
        // The modern windows' own parts: the opacity slider (it sets the window's opacity as
        // retail's slider did, and a release saves it), the settings popover.
        let event = self.modern.as_ref().map(|m| m.message(tree, message));
        if let Some(Some(event)) = event {
            use crate::modern_chat::ChatEvent;
            match event {
                ChatEvent::Opacity {
                    window,
                    opacity,
                    released,
                } => {
                    if let Some(w) = self.windows.get(window) {
                        w.apply_opacity(tree, opacity);
                        if released {
                            self.opacity_requests.push((w.window_id, opacity));
                        }
                    }
                }
                ChatEvent::Look(i) => self.apply_look(tree, i),
                ChatEvent::Hide(i) => {
                    if let Some(w) = self.windows.get_mut(i) {
                        w.show(tree, false);
                    }
                }
                ChatEvent::OpenOptions => self.open_options = true,
            }
            return;
        }
        if message.element.is_none() {
            if message.message == GLOBAL_KEY_PRESS {
                self.key_press(tree, message.data as u32, modifiers);
            }
            return;
        }
        for i in 0..self.windows.len() {
            self.window_message(tree, i, message);
        }
    }

    /// `ChatWindow::ListenToMessage` for element messages.
    fn window_message(&mut self, tree: &mut UiTree, i: usize, message: &UiMessage) {
        let window = &self.windows[i];
        if window.is_active_root(tree) {
            self.last_focus = Some(i);
        }
        let window = &mut self.windows[i];
        // `ChatWindow::ListenToMessage`'s opacity slider: the window follows it as it moves; a
        // release saves the opacity (`SetWindowOpacity`, then `UploadUISettings`).
        if message.element.is_some() && message.element == window.opacity_slider {
            let slider = message.element.unwrap_or_default();
            let opacity = tree
                .float_attribute(slider, tokenize("y_position"))
                .unwrap_or(0.0);
            window.apply_opacity(tree, opacity);
            if message.message == MSG_WIDGET_RELEASE || message.message == MSG_MOUSE_RELEASE {
                self.opacity_requests.push((window.window_id, opacity));
            }
            return;
        }
        // `ChatWindow::ListenToMessage`'s log scroll bar: a press locks the log, a release
        // unlocks it.
        if message.element.is_some() && message.element == window.log_scrollbar {
            if message.message == MSG_WIDGET_PRESS {
                window.lock_scroll = true;
            } else if message.message == MSG_WIDGET_RELEASE {
                window.lock_scroll = false;
            }
            return;
        }
        if message.element_id == window.toggle_button && message.message == MSG_MOUSE_RELEASE {
            let shown = window.shown(tree);
            window.show(tree, !shown);
            return;
        }
        if message.element.is_some()
            && message.element == window.move_bar
            && message.message == MSG_MOUSE_PRESS
        {
            self.set_active_window(tree, i);
            return;
        }
        if message.element.is_some() && message.element == window.options_button {
            if message.message == MSG_BUTTON_RELEASE_ON {
                if let Some(field) = window.options_field {
                    let visible = tree.get(field).is_some_and(|e| e.visible);
                    tree.set_visible(field, !visible);
                }
            }
        } else if message.element.is_some()
            && message.element == window.entry
            && message.message == tokenize("activated")
        {
            window.handle_activation(tree);
            let top = window.zlevel_front;
            self.update_z_levels(tree, top);
            self.set_active_window(tree, i);
        }
    }

    /// `gmChatUI::ListenToMessage`'s key presses.
    fn key_press(&mut self, tree: &mut UiTree, key: u32, modifiers: Modifiers) {
        if let Some(n) = WINDOW_KEYS.iter().position(|&k| k == key) {
            if let Some(window) = self.windows.get_mut(n) {
                let shown = window.shown(tree);
                window.show(tree, !shown);
            }
            return;
        }
        // `DispatchKeypressToWindow`.
        let activate = matches!(key, ENTER | COMMAND_KEY | ALIAS_KEY | REPLY_KEY);
        let Some(i) = self.determine_target_window(tree, activate) else {
            return;
        };
        match key {
            COMMAND_KEY | ALIAS_KEY => {
                // `HandleCommandOrAliasKey`.
                let prefix = if key == COMMAND_KEY {
                    &self.strings.command_prefix
                } else {
                    &self.strings.alias_prefix
                };
                let prefix = prefix.clone().unwrap_or_default();
                self.windows[i].prefill_entry(tree, &prefix, EntrySelection::None);
            }
            ENTER => {
                if let Some(line) = self.windows[i].handle_enter_key(tree) {
                    self.requests.push(ChatRequest::Line(line));
                }
            }
            REPLY_KEY => self.handle_reply_key(tree, i),
            _ => self.window_key_press(tree, i, key, modifiers),
        }
    }

    /// `ChatWindow::HandleReplyKey`: the entry line starts with the reply command, `/` and the
    /// first word of `ID_Reply`.
    fn handle_reply_key(&mut self, tree: &mut UiTree, i: usize) {
        let reply = self.strings.reply.clone().unwrap_or_default();
        let word: Vec<u16> = reply
            .split(|c| matches!(c, 0x3B | 0x3A | 0x2C))
            .find(|w| !w.is_empty())
            .map(<[u16]>::to_vec)
            .unwrap_or_default();
        if word.is_empty() {
            return;
        }
        let mut line = vec![u16::from(b'/')];
        line.extend(word);
        line.push(u16::from(b' '));
        self.windows[i].prefill_entry(tree, &line, EntrySelection::Keep);
    }

    /// `ChatWindow::KeyPress`.
    fn window_key_press(&mut self, tree: &mut UiTree, i: usize, key: u32, m: Modifiers) {
        let window = &mut self.windows[i];
        let history = |history, prev| ChatRequest::History { history, prev };
        if m.ctrl && !m.alt && !m.shift {
            match key {
                UP | DOWN => {
                    let request = history(ChatHistory::Retell, key == UP);
                    self.requests.push(request);
                    return;
                }
                HOME | END => return window.scroll_log(tree, key == END),
                _ => {}
            }
        }
        if m.shift && !m.ctrl && !m.alt && matches!(key, UP | DOWN) {
            self.requests.push(history(ChatHistory::Reply, key == UP));
            return;
        }
        if !m.alt && !m.ctrl && !m.shift {
            match key {
                UP | DOWN => self.requests.push(history(ChatHistory::Command, key == UP)),
                PAGE_UP | PAGE_DOWN => window.page(tree, key == PAGE_DOWN),
                ESCAPE => window.deactivate_pending = true,
                _ => {}
            }
        }
    }

    /// `ChatWindow::DisplayCmdHistory`'s answer: the target window's active entry line shows
    /// `text`.
    pub fn set_entry_text(&mut self, tree: &mut UiTree, text: &str) {
        let Some(i) = self.determine_target_window(tree, false) else {
            return;
        };
        if !self.windows[i].is_active(tree) {
            return;
        }
        let units: Vec<u16> = text.encode_utf16().collect();
        self.windows[i].prefill_entry(tree, &units, EntrySelection::All);
    }

    /// The entry line of the window chat would send from shows `text`, ready to send or edit (the
    /// guide's Share).
    pub fn prefill(&mut self, tree: &mut UiTree, text: &str) {
        let Some(i) = self.determine_target_window(tree, false) else {
            return;
        };
        self.windows[i].prefill_entry(tree, &utf16(text), EntrySelection::Keep);
    }

    /// `gmChatUI::RecvNotice_DoAssistedTell` / `ChatWindow::DoAssistedTell`: the entry line
    /// becomes a tell to `target` (`ID_Chat_AssistedTell` with `prefix` and `target`).
    pub fn assisted_tell(&mut self, tree: &mut UiTree, target: &str) {
        let Some(i) = self.determine_target_window(tree, false) else {
            return;
        };
        let Some(template) = &self.strings.assisted_tell else {
            return;
        };
        let prefix = self.strings.command_prefix.clone().unwrap_or_default();
        let line = fill_variables(
            template,
            &[
                (tokenize("prefix"), &prefix),
                (tokenize("target"), &utf16(target)),
            ],
        );
        self.windows[i].prefill_entry(tree, &line, EntrySelection::All);
    }

    /// `RecvNotice_DisplayStringInfo` to every window; the error-type lines (`type & 1`) also
    /// queue for the spew box.
    pub fn display(&mut self, text_type: u32, text: &str) {
        let units = utf16(text);
        for window in &mut self.windows {
            window.display(text_type, &units);
        }
        if text_type & TEXT_TYPE_ERROR != 0 {
            if let Some(spew) = &mut self.spew {
                spew.queue(text);
            }
        }
    }

    /// `ID_Chat_UnknownCommand` as an error (`ChatWindow::ProcessCommand` when
    /// `ProcessText` does not know the command).
    pub fn unknown_command(&mut self) {
        if let Some(text) = self.strings.unknown_command.clone() {
            for window in &mut self.windows {
                window.display(TEXT_TYPE_ERROR, &text);
            }
        }
    }

    /// `RecvNotice_ReloadChatSettings` to every window.
    pub fn reload_chat_settings(&mut self, tree: &mut UiTree, settings: &ChatSettings) {
        for window in &mut self.windows {
            window.reload_chat_settings(tree, settings, &self.strings);
        }
    }

    /// The spew box (tests).
    #[cfg(test)]
    pub(crate) fn spew_box(&self) -> Option<&SpewBox> {
        self.spew.as_ref()
    }

    /// `UIMESSAGE_GLOBAL_LOOP` at `Timer::cur_time` `now`: each window writes its pending
    /// strings and drops a pending deactivation, and the spew box updates (`gmChatUI::Update`).
    pub fn global_loop(&mut self, tree: &mut UiTree, now: f64) {
        for window in &mut self.windows {
            window.global_loop(tree);
        }
        if let Some(spew) = &mut self.spew {
            spew.update(tree, now);
        }
    }
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// `StringInfo::GetString` with string variables: each piece followed by the value of the
/// variable after it (a variable without a value adds nothing).
fn fill_variables(string: &ac2_db::StringTableString, variables: &[(u32, &[u16])]) -> Vec<u16> {
    let mut out = Vec::new();
    for (i, piece) in string.pieces.iter().enumerate() {
        out.extend(piece.iter().flatten().copied());
        if let Some(id) = string.variables.get(i) {
            if let Some((_, value)) = variables.iter().find(|(v, _)| v == id) {
                out.extend_from_slice(value);
            }
        }
    }
    out
}
