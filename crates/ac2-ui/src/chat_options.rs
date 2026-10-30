//! `gmChatOptionsUI`: the chat options page (`ChatPage` under the `ChatTab` of the toolbar's
//! panel, Legions 7160). `Setup` lists an expander per visible text type of
//! [`chat_options_table::ROWS`] (`PopulateExpander`): its font size and colour choices, the four
//! windows' filter flags and the popup flag (the entry line has only the first two). The window
//! section (`ID_Option_DevotedChatWindows`) has each window's text-type choice. `Refresh` reads the
//! player's [`ChatSettings`] into them. Apply sends each option's setter to the player's script
//! (`gmCWInterface::SetChatFontSize`, `SetChatFontColor`, `SetChatPopupFlag`, the window's text
//! type), then each window's filter (the bits of the page's types, the others as the profile had
//! them), the reload of the chat windows and the upload; Recall restores the saved values, and
//! Default restores the defaults and applies them. Apply, Recall and Default ask a confirmation
//! first (`ID_Option_ApplyText` and the like), answered through [`ChatOptionsUi::dialog_outcomes`].
//!
//! Choices are numbered from 1 (the widget's convention, as the character page's): choice `n` of a
//! font size is `FONT_SIZES[n - 1]`, of a colour the colour `n - 1`, of a window's text type the
//! row `n - 1` of [`chat_options_table::ROWS`].
//!
//! Each drop-down is a `UIElement_Menu` filled with its items when the option is made; its new
//! selection is read back into the option.
//!
//! Not modelled: the tooltips (no help strings are known for these options), and the
//! notice registrations.

use std::sync::Arc;

use crate::character::describe;
use crate::character::Labels;
use crate::character_options::{
    fill_choices, is_changed_of, selected_choice, sync_option, template_name, Kind,
};
use crate::chat::ChatSettings;
use crate::chat_options_setup::{flag, read, specs, Field, Spec};
use crate::chat_options_table::{FONT_SIZES, ROWS};
use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::menu::MSG_MENU_NEW_SELECTION;
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
use crate::text::UiTextData;
use crate::token::{
    tokenize, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON, MSG_HIGHLIGHT_STATE_CHANGE,
};

/// `MSG_PANEL_PAGE_CLOSED`: the tab shut.
const MSG_CLOSED: u32 = tokenize("closed");
/// The confirmations' texts (`ID_Option_ApplyText`, `ID_Option_RecallText`, `ID_Option_DefaultText`).
const APPLY_TEXT: &str = "ID_Option_ApplyText";
const RECALL_TEXT: &str = "ID_Option_RecallText";
const DEFAULT_TEXT: &str = "ID_Option_DefaultText";
/// The profile's filter mask kept by `Apply` (`GetFilter(window) & 0x84`: admin and log).
const KEPT_FILTER_BITS: u32 = 0x84;

/// A setter the page asks the player's script for (`gmCWInterface::Set*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatOptionSetting {
    /// `SetChatFontSize(textType, size)`.
    FontSize { text_type: u32, size: u32 },
    /// `SetChatFontColor(textType, colour)`.
    FontColor { text_type: u32, color: u32 },
    /// `SetChatPopupFlag(textType, flag)`.
    PopupFlag { text_type: u32, flag: u32 },
    /// `SetChatWindowFilter(window, bitfield)`.
    WindowFilter { window: u32, filter: u32 },
    /// The window's text type (`FUN_00458e50`'s IFun 0xB40143).
    WindowTextType { window: u32, text_type: u32 },
}

/// What the page asks of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatOptionRequest {
    /// A setter on the player's script.
    Set(ChatOptionSetting),
    /// `gmCWInterface::UploadChatOptions` (the profile to the server).
    Upload,
}

/// One option of an expander (`OptionFlag` / `OptionChoice`): the value the control shows, the
/// saved one and the default, as the widget keeps them.
#[derive(Debug, Clone, PartialEq)]
struct ChatOption {
    field: Field,
    /// The text type of its expander (the window section for the windows' choices).
    text_type: u32,
    /// The label's string id name.
    name: &'static str,
    /// The expander this option is under (an index into the page's entries).
    parent: usize,
    element: usize,
    control: Option<usize>,
    kind: Kind,
    current: f64,
    saved: f64,
    default: f64,
}

/// An expander (`OptionExpander`) of a text type.
#[derive(Debug, Clone, PartialEq)]
struct Expander {
    title: &'static str,
    element: usize,
    header: Option<usize>,
    list: Option<usize>,
    children: Vec<usize>,
    open: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Entry {
    Tree(Expander),
    Option(ChatOption),
}

/// `gmChatOptionsUI`'s state.
#[derive(Debug, Clone)]
pub struct ChatOptionsUi {
    page: usize,
    list: Option<usize>,
    apply: Option<usize>,
    recall: Option<usize>,
    /// Every expander and option, in `Setup`'s order.
    entries: Vec<Entry>,
    /// The top-level entries: the expanders.
    main: Vec<usize>,
    /// The tab shows the page.
    open: bool,
    /// The filters the page last read (`GetFilter` of each window), where `Apply` keeps its
    /// [`KEPT_FILTER_BITS`].
    base_filters: [u32; 4],
    context_apply: u32,
    context_recall: u32,
    context_default: u32,
    apply_text: String,
    recall_text: String,
    default_text: String,
    requests: Vec<ChatOptionRequest>,
    reload: bool,
}

/// The toolbar layout's element `name`, made as an element (`OptionBase::Create`).
fn make_element(tree: &mut UiTree, toolbar: &Arc<LayoutDesc>, name: &str) -> Option<usize> {
    let template = toolbar.element(tokenize(name))?.clone();
    tree.create_element(Arc::clone(toolbar), template, false)
}

impl ChatOptionsUi {
    /// `gmChatOptionsUI::gmChatOptionsUI` (and its `Setup`): the page over the HUD tree's
    /// `ChatPage`, with the expanders and options made from the toolbar layout's templates. `text`
    /// reads the UI string table `table`. The player's settings come with
    /// [`ChatOptionsUi::refresh`] (the constructor's `Refresh`). `None` without the page.
    pub fn create(
        tree: &mut UiTree,
        toolbar: &Arc<LayoutDesc>,
        text: &dyn UiTextData,
        table: u32,
    ) -> Option<Self> {
        let labels = Labels { text, table };
        let page = tree
            .roots()
            .iter()
            .find_map(|&root| tree.child_recursive(root, tokenize("ChatPage")))?;
        let mut this = Self {
            page,
            list: tree.child_recursive(page, tokenize("ChatOptionsList")),
            apply: tree.child_recursive(page, tokenize("ApplyButton")),
            recall: tree.child_recursive(page, tokenize("RecallButton")),
            entries: Vec::new(),
            main: Vec::new(),
            open: false,
            base_filters: [0; 4],
            context_apply: 0,
            context_recall: 0,
            context_default: 0,
            apply_text: labels.get(APPLY_TEXT),
            recall_text: labels.get(RECALL_TEXT),
            default_text: labels.get(DEFAULT_TEXT),
            requests: Vec::new(),
            reload: false,
        };
        this.setup(tree, toolbar, labels);
        this.build_list(tree);
        Some(this)
    }

    /// `gmChatOptionsUI::Setup`: an expander per visible text type, in table order, with its
    /// options (`PopulateExpander`).
    fn setup(&mut self, tree: &mut UiTree, toolbar: &Arc<LayoutDesc>, labels: Labels) {
        let defaults = ChatSettings::default();
        for (row_index, row) in ROWS.iter().enumerate() {
            if row.hidden {
                continue;
            }
            let Some(element) = make_element(tree, toolbar, "OptionTree") else {
                continue;
            };
            let header = tree.child_recursive(element, tokenize("TreeToggle"));
            let list = tree.child_recursive(element, tokenize("TreeList"));
            describe(tree, labels, element, row.name, None);
            let parent = self.entries.len();
            self.entries.push(Entry::Tree(Expander {
                title: row.name,
                element,
                header,
                list,
                children: Vec::new(),
                open: false,
            }));
            self.main.push(parent);
            crate::option_expander::refit(tree, element, header, list);
            for spec in specs(row_index, &defaults, labels) {
                let Some(option) = self.make_option(tree, toolbar, labels, spec, parent) else {
                    continue;
                };
                let at = self.entries.len();
                self.entries.push(Entry::Option(option));
                if let Entry::Tree(expander) = &mut self.entries[parent] {
                    expander.children.push(at);
                }
            }
        }
    }

    /// One option's widget: its template, its control, its label and tooltip-less description,
    /// with its default as the value.
    fn make_option(
        &self,
        tree: &mut UiTree,
        toolbar: &Arc<LayoutDesc>,
        labels: Labels,
        spec: Spec,
        parent: usize,
    ) -> Option<ChatOption> {
        let element = make_element(tree, toolbar, template_name(spec.kind))?;
        let control = match spec.kind {
            Kind::Flag => tree.child_recursive(element, tokenize("CheckboxButton")),
            Kind::Choice { .. } => tree.child_recursive(element, tokenize("DropDownMenu")),
            Kind::Value { .. } => None,
        };
        describe(tree, labels, element, spec.name, None);
        fill_choices(tree, control, &spec.choices);
        let option = ChatOption {
            field: spec.field,
            text_type: spec.text_type,
            name: spec.name,
            parent,
            element,
            control,
            kind: spec.kind,
            current: spec.default,
            saved: spec.default,
            default: spec.default,
        };
        sync_option(tree, control, option.kind, option.current);
        Some(option)
    }

    /// The top-level expanders in the `ChatOptionsList`.
    fn build_list(&mut self, tree: &mut UiTree) {
        let Some(list) = self.list else {
            return;
        };
        for &index in &self.main {
            if let Entry::Tree(expander) = &self.entries[index] {
                tree.list_add_item(list, expander.element);
            }
        }
    }

    /// `gmChatOptionsUI::Refresh`: each option's value from the player's settings, and the value
    /// saved; the window filters the next `Apply` keeps are the ones read here.
    pub fn refresh(&mut self, tree: &mut UiTree, settings: &ChatSettings) {
        self.base_filters = [1, 2, 3, 4].map(|window| settings.filter(window));
        for index in 0..self.entries.len() {
            let Entry::Option(option) = &mut self.entries[index] else {
                continue;
            };
            let text_type = option.text_type;
            if let Some(value) = read(option.field, text_type, settings) {
                option.current = value;
            }
            option.saved = option.current;
            sync_option(tree, option.control, option.kind, option.current);
        }
        self.update_buttons(tree);
    }

    /// Whether an option differs from its saved value (`gmChatOptionsUI::Changed`).
    pub fn changed(&self) -> bool {
        self.options()
            .any(|o| is_changed_of(o.kind, o.current, o.saved))
    }

    fn options(&self) -> impl Iterator<Item = &ChatOption> {
        self.entries.iter().filter_map(|e| match e {
            Entry::Option(option) => Some(option),
            Entry::Tree(_) => None,
        })
    }

    /// `gmChatOptionsUI::UpdateButtons`: Apply and Recall ghost while nothing has changed.
    pub fn update_buttons(&mut self, tree: &mut UiTree) {
        let changed = self.changed();
        for button in [self.apply, self.recall].into_iter().flatten() {
            tree.set_bool(button, BUTTON_GHOSTED, !changed);
        }
    }

    /// `gmChatOptionsUI::Apply`: every option's setter (font size, colour, popup flag, window text
    /// type, in table order), each window's filter (the page's flags over the kept bits of the
    /// last read), the reload of the chat windows, then the upload. Retail sends every option,
    /// changed or not; the saved values follow.
    pub fn apply(&mut self, tree: &mut UiTree) {
        let mut masks = self.base_filters.map(|filter| filter & KEPT_FILTER_BITS);
        for index in 0..self.entries.len() {
            let Entry::Option(option) = &mut self.entries[index] else {
                continue;
            };
            let value = option.current;
            let choice = value.max(1.0) as usize - 1;
            let text_type = option.text_type;
            match option.field {
                Field::FontSize => {
                    let size = FONT_SIZES[choice].1;
                    self.requests
                        .push(ChatOptionRequest::Set(ChatOptionSetting::FontSize {
                            text_type,
                            size,
                        }));
                }
                Field::FontColor => {
                    self.requests
                        .push(ChatOptionRequest::Set(ChatOptionSetting::FontColor {
                            text_type,
                            color: choice as u32,
                        }));
                }
                Field::WindowFilter(window) => {
                    let mask = &mut masks[window as usize - 1];
                    if value != 0.0 {
                        *mask |= text_type;
                    } else {
                        *mask &= !text_type;
                    }
                }
                Field::Popup => {
                    self.requests
                        .push(ChatOptionRequest::Set(ChatOptionSetting::PopupFlag {
                            text_type,
                            flag: u32::from(value != 0.0),
                        }));
                }
                Field::WindowType(window) => {
                    self.requests
                        .push(ChatOptionRequest::Set(ChatOptionSetting::WindowTextType {
                            window,
                            text_type: ROWS[choice].text_type,
                        }));
                }
            }
            option.saved = option.current;
        }
        for window in 1..=4u32 {
            let filter = masks[window as usize - 1];
            self.requests
                .push(ChatOptionRequest::Set(ChatOptionSetting::WindowFilter {
                    window,
                    filter,
                }));
        }
        self.reload = true;
        self.requests.push(ChatOptionRequest::Upload);
        self.update_buttons(tree);
    }

    /// `gmChatOptionsUI::Recall`: every option back to its saved value.
    pub fn recall(&mut self, tree: &mut UiTree) {
        for index in 0..self.entries.len() {
            let Entry::Option(option) = &mut self.entries[index] else {
                continue;
            };
            option.current = option.saved;
            sync_option(tree, option.control, option.kind, option.current);
        }
        self.update_buttons(tree);
    }

    /// `gmChatOptionsUI::Default`: every option to its default, then applied.
    pub fn default(&mut self, tree: &mut UiTree) {
        for index in 0..self.entries.len() {
            let Entry::Option(option) = &mut self.entries[index] else {
                continue;
            };
            option.current = option.default;
            sync_option(tree, option.control, option.kind, option.current);
        }
        self.apply(tree);
    }

    /// `gmChatOptionsUI::ConfirmApply`: asks once while the question is open.
    pub fn confirm_apply(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_apply == 0 {
            self.context_apply =
                dialogs.make_dialog(tree, &self.apply_text, DIALOG_CONFIRMATION, false);
        }
    }

    /// `gmChatOptionsUI::ConfirmRecall`.
    pub fn confirm_recall(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_recall == 0 {
            self.context_recall =
                dialogs.make_dialog(tree, &self.recall_text, DIALOG_CONFIRMATION, false);
        }
    }

    /// `gmChatOptionsUI::ConfirmDefault`.
    pub fn confirm_default(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_default == 0 {
            self.context_default =
                dialogs.make_dialog(tree, &self.default_text, DIALOG_CONFIRMATION, false);
        }
    }

    /// `gmChatOptionsUI::RecvNotice_CloseConfirmationDialog`: a yes runs the question's action;
    /// the questions close.
    pub fn recv_close_confirmation_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        context: u32,
        yes: bool,
    ) {
        if yes && context != 0 {
            if context == self.context_apply {
                self.apply(tree);
            }
            if context == self.context_recall {
                self.recall(tree);
            }
            if context == self.context_default {
                self.default(tree);
            }
        }
        // The answered question is no longer open; the others close (`ClearConfirmations`).
        for slot in [
            &mut self.context_apply,
            &mut self.context_recall,
            &mut self.context_default,
        ] {
            if *slot == context {
                *slot = 0;
            }
        }
        for open in [
            self.context_apply,
            self.context_recall,
            self.context_default,
        ] {
            if open != 0 {
                dialogs.close_dialog(tree, open);
            }
        }
        self.context_apply = 0;
        self.context_recall = 0;
        self.context_default = 0;
    }

    /// The confirmations' answers (`DialogFactory`'s outcomes).
    pub fn dialog_outcomes(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        outcomes: &[DialogOutcome],
    ) {
        for outcome in outcomes {
            if let DialogOutcome::Confirmation { context, yes } = *outcome {
                self.recv_close_confirmation_dialog(tree, dialogs, context, yes);
            }
        }
    }

    /// `gmChatOptionsUI::ListenToMessage`: a checkbox's highlight is read into its option, an
    /// expander's header toggles it; while the tab is open, the buttons' presses ask their
    /// confirmations and the tab's `opened` and `closed` messages set it.
    pub fn message(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, message: &UiMessage) {
        if message.element_id == tokenize("ChatTab") {
            if message.message == MSG_OPENED {
                self.open = true;
                self.update_buttons(tree);
            } else if message.message == MSG_CLOSED {
                self.open = false;
            }
            return;
        }
        if message.message == MSG_HIGHLIGHT_STATE_CHANGE
            || message.message == MSG_MENU_NEW_SELECTION
        {
            if let Some(index) = self.control_at(message.element) {
                self.read_control(tree, index);
            }
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            if let Some(index) = self.header_at(message.element) {
                self.toggle(tree, index);
            }
        }
        if !self.open {
            return;
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            match message.element_id {
                e if e == tokenize("ApplyButton") => self.confirm_apply(tree, dialogs),
                e if e == tokenize("RecallButton") => self.confirm_recall(tree, dialogs),
                e if e == tokenize("DefaultButton") => self.confirm_default(tree, dialogs),
                _ => {}
            }
        }
        self.update_buttons(tree);
    }

    /// Sets an option's value as its control would when the player changes it (`0` or `1` for a
    /// flag, the choice number for a drop-down), then the buttons. False for an unknown option or
    /// a choice outside the drop-down.
    pub fn set_value(&mut self, tree: &mut UiTree, title: &str, name: &str, value: f64) -> bool {
        let Some(index) = self.option_index(title, name) else {
            return false;
        };
        let Entry::Option(option) = &mut self.entries[index] else {
            return false;
        };
        match option.kind {
            Kind::Choice { count } => {
                if !(1.0..=f64::from(count)).contains(&value) {
                    return false;
                }
                option.current = value.trunc();
            }
            _ => option.current = flag(value != 0.0),
        }
        sync_option(tree, option.control, option.kind, option.current);
        self.update_buttons(tree);
        true
    }

    /// The option `name` under the expander titled `title`: its current value.
    pub fn value(&self, title: &str, name: &str) -> Option<f64> {
        self.option_index(title, name)
            .and_then(|i| match &self.entries[i] {
                Entry::Option(option) => Some(option.current),
                Entry::Tree(_) => None,
            })
    }

    /// The option's element.
    pub fn element(&self, title: &str, name: &str) -> Option<usize> {
        self.option_index(title, name)
            .and_then(|i| match &self.entries[i] {
                Entry::Option(option) => Some(option.element),
                Entry::Tree(_) => None,
            })
    }

    /// The option's control (its checkbox or drop-down).
    pub fn control(&self, title: &str, name: &str) -> Option<usize> {
        self.option_index(title, name)
            .and_then(|i| match &self.entries[i] {
                Entry::Option(option) => option.control,
                Entry::Tree(_) => None,
            })
    }

    /// The expander titled `title`: whether it is open (its options listed).
    pub fn is_open(&self, title: &str) -> Option<bool> {
        self.entries.iter().find_map(|e| match e {
            Entry::Tree(expander) if expander.title == title => Some(expander.open),
            _ => None,
        })
    }

    /// The expander titled `title`'s header button (`TreeToggle`).
    pub fn header(&self, title: &str) -> Option<usize> {
        self.entries.iter().find_map(|e| match e {
            Entry::Tree(expander) if expander.title == title => expander.header,
            _ => None,
        })
    }

    /// How many options the expander titled `title` lists while it is open.
    pub fn rows(&self, tree: &UiTree, title: &str) -> Option<usize> {
        self.entries.iter().find_map(|e| match e {
            Entry::Tree(expander) if expander.title == title => {
                expander.list.map(|list| tree.list_items(list).len())
            }
            _ => None,
        })
    }

    /// The expanders' titles in `Setup`'s order, each followed by its options' names.
    pub fn order(&self) -> Vec<&'static str> {
        self.entries
            .iter()
            .map(|e| match e {
                Entry::Tree(expander) => expander.title,
                Entry::Option(option) => option.name,
            })
            .collect()
    }

    /// The titles of the top-level rows in `ChatOptionsList`, in the list's order.
    pub fn listed(&self, tree: &UiTree) -> Vec<&'static str> {
        let Some(list) = self.list else {
            return Vec::new();
        };
        tree.list_items(list)
            .iter()
            .filter_map(|&element| {
                self.main.iter().find_map(|&i| match &self.entries[i] {
                    Entry::Tree(expander) if expander.element == element => Some(expander.title),
                    _ => None,
                })
            })
            .collect()
    }

    /// The page's element (`ChatPage`).
    pub fn page(&self) -> usize {
        self.page
    }

    /// The Apply button's element.
    pub fn apply_button(&self) -> Option<usize> {
        self.apply
    }

    /// The Recall button's element.
    pub fn recall_button(&self) -> Option<usize> {
        self.recall
    }

    /// Whether a button shows ghosted.
    pub fn ghosted(&self, tree: &UiTree, button: Option<usize>) -> bool {
        button.is_some_and(|b| {
            matches!(
                tree.get(b).and_then(|e| e.attribute(BUTTON_GHOSTED)),
                Some(AttributeValue::Bool(true))
            )
        })
    }

    /// The tab's `opened` state.
    pub fn page_open(&self) -> bool {
        self.open
    }

    /// The requests since the last take.
    pub fn take_requests(&mut self) -> Vec<ChatOptionRequest> {
        std::mem::take(&mut self.requests)
    }

    /// Whether the chat windows reload since the last take (an Apply sent the reload notice).
    pub fn take_reload(&mut self) -> bool {
        std::mem::take(&mut self.reload)
    }

    fn option_index(&self, title: &str, name: &str) -> Option<usize> {
        self.entries.iter().position(|e| match e {
            Entry::Option(option) => {
                option.name == name
                    && matches!(&self.entries[option.parent],
                        Entry::Tree(expander) if expander.title == title)
            }
            Entry::Tree(_) => false,
        })
    }

    /// The expander whose header is `element`.
    fn header_at(&self, element: Option<usize>) -> Option<usize> {
        let element = element?;
        self.entries.iter().position(|e| match e {
            Entry::Tree(expander) => expander.header == Some(element),
            Entry::Option(_) => false,
        })
    }

    /// The option whose control is `element`.
    fn control_at(&self, element: Option<usize>) -> Option<usize> {
        let element = element?;
        self.entries.iter().position(|e| match e {
            Entry::Option(option) => option.control == Some(element),
            Entry::Tree(_) => false,
        })
    }

    /// Reads a checkbox's highlight, or a drop-down's selected choice, into its option.
    fn read_control(&mut self, tree: &UiTree, index: usize) {
        let Entry::Option(option) = &mut self.entries[index] else {
            return;
        };
        let Some(control) = option.control else {
            return;
        };
        match option.kind {
            Kind::Choice { .. } => {
                if let Some(choice) = selected_choice(tree, Some(control)) {
                    option.current = choice;
                }
            }
            _ => {
                if let Some(AttributeValue::Bool(on)) = tree
                    .get(control)
                    .and_then(|e| e.attribute(BUTTON_HIGHLIGHTED))
                {
                    option.current = flag(*on);
                }
            }
        }
    }

    /// `OptionExpander::ListenToMessage`'s toggle: open lists its options, closed takes them out.
    fn toggle(&mut self, tree: &mut UiTree, index: usize) {
        let Entry::Tree(expander) = &mut self.entries[index] else {
            return;
        };
        let Some(list) = expander.list else {
            return;
        };
        expander.open = !expander.open;
        let open = expander.open;
        let children = expander.children.clone();
        let header = expander.header;
        let element = expander.element;
        if open {
            for child in children {
                if let Entry::Option(option) = &self.entries[child] {
                    tree.list_add_item(list, option.element);
                }
            }
        } else {
            while !tree.list_items(list).is_empty() {
                tree.list_remove_item(list, 0);
            }
        }
        if let Some(header) = header {
            tree.set_state(header, tokenize(if open { "highlight" } else { "normal" }));
        }
        crate::option_expander::refit(tree, element, header, Some(list));
    }
}
