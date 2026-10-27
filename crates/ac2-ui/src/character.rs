//! `gmCharacterUI`: the character options page (`CharacterPage` under the `CharacterTab` of the
//! toolbar's panel). `Setup` builds its tree of expanders and options ([`crate::character_options`]),
//! `BuildList` lists the top-level ones in `CharacterList`, and `Refresh` reads the player's
//! options profile into them. Apply sends each changed option's setter to the player's script
//! (`ApplyFlag`, `ApplyValue`, `ApplyChoice`) and then uploads the profile; Recall restores the
//! saved values, and Default (after a confirmation) restores the defaults and applies them. Recall
//! and Default ask a confirmation (`ID_Option_RecallText` / `ID_Option_DefaultText`) through the
//! [`DialogFactory`]; their answers come back as [`CharacterUi::dialog_outcomes`].
//!
//! The panel's buttons are `ApplyButton`, `RecallButton` and `DefaultButton`; they ghost while
//! nothing differs from its saved value (`UpdateButtons`). The presses and the widgets' own
//! messages come through [`CharacterUi::message`]; what the page asks of the player is queued for
//! [`CharacterUi::take_requests`].
//!
//! A drop-down (`OptionChoice`) is a `UIElement_Menu` on `DropDownMenu`, filled with its labels
//! when the option is made; the menu's selection is the choice, read back on its new selection.
//!
//! Not modelled: the native `CM_Character::SendNotice_ReloadGameplayOptions` to the other option
//! listeners (`gmNetStatusUI`, `gmPetUI`, `FellowshipHelper`), the preference bank's handlers
//! (`PreferenceBank::Change`) and the expanders' `Refit` sizes.

use std::sync::Arc;

use crate::character_options::{
    choice_name, fill_choices, is_changed_of, read, selected_choice, string_id, sync_option,
    template_name, write, Kind, OptionDesc, OptionSetting, OptionsSettings, Setter, SETUP,
};
use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::menu::MSG_MENU_NEW_SELECTION;
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
use crate::scrollbar::MSG_NEW_POSITION;
use crate::text::UiTextData;
use crate::token::{
    tokenize, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON,
    MSG_HIGHLIGHT_STATE_CHANGE, MSG_MOUSE_PRESS,
};

/// `ID_Option_RecallText` and `ID_Option_DefaultText` (the confirmations).
const RECALL_TEXT: &str = "ID_Option_RecallText";
const DEFAULT_TEXT: &str = "ID_Option_DefaultText";
/// `MSG_PANEL_PAGE_CLOSED`: the tab shut.
const MSG_CLOSED: u32 = tokenize("closed");

/// What the page asks of the player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharacterRequest {
    /// A setter on the player's script (`gmCWInterface::Set*`, then the next option's).
    Set(OptionSetting),
    /// `gmCWInterface::UploadGameOptions` (the profile to the server).
    Upload,
}

/// An option's widget state (`OptionFlag` / `OptionValue` / `OptionChoice`): the value the
/// control shows, the saved one, and the widget's elements.
#[derive(Debug, Clone, PartialEq)]
struct OptionEntry {
    desc: OptionDesc,
    /// The expander this option is under (`OptionExpander::AddOption`).
    parent: Option<usize>,
    /// `m_Element`: the option's own element, from the template.
    element: usize,
    /// The checkbox (`CheckboxButton`), slider bar (`SliderBar`) or drop-down (`DropDownMenu`).
    control: Option<usize>,
    /// `SliderLeft` and `SliderRight`.
    slider_buttons: [Option<usize>; 2],
    /// `m_Preference`'s current value and its saved one (`mPrevious`).
    current: f64,
    saved: f64,
}

/// An expander (`OptionExpander`): its header, the list of its options while open, and its
/// options (indices into the page's entries).
#[derive(Debug, Clone, PartialEq)]
struct TreeEntry {
    title: &'static str,
    element: usize,
    header: Option<usize>,
    list: Option<usize>,
    children: Vec<usize>,
    open: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Entry {
    Tree(TreeEntry),
    Option(OptionEntry),
}

/// `gmCharacterUI`'s state (`_characterPage`, `_optionsMain`, `_options`, the confirmation
/// contexts, the radar mask the page works from).
#[derive(Debug, Clone)]
pub struct CharacterUi {
    page: usize,
    list: Option<usize>,
    apply: Option<usize>,
    recall: Option<usize>,
    /// `_options`: every expander and option, in `Setup`'s order.
    entries: Vec<Entry>,
    /// `_optionsMain`: the top-level entries (the expanders and the options outside them).
    main: Vec<usize>,
    /// The tab shows the page (`PanelPageMsg_opened`).
    open: bool,
    /// The radar mask the page's flags edit (`GetRadarMask` as `ApplyFlag` reads it).
    radar_mask: u32,
    /// `_contextRecall` and `_contextDefault`: the open confirmation, 0 for none.
    context_recall: u32,
    context_default: u32,
    recall_text: String,
    default_text: String,
    requests: Vec<CharacterRequest>,
}

/// The UI string table's strings, as text: its data and the table's DID (`StoreTextData` keys
/// its tables by DID).
#[derive(Clone, Copy)]
pub(crate) struct Labels<'a> {
    pub(crate) text: &'a dyn UiTextData,
    pub(crate) table: u32,
}

impl Labels<'_> {
    /// The string `name` of the table, as text (empty when the table has no such string).
    pub(crate) fn get(&self, name: &str) -> String {
        self.text
            .string(self.table, string_id(name))
            .map(|units| String::from_utf16_lossy(&units))
            .unwrap_or_default()
    }
}

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `OptionBase::Describe`: the label (`OptionText`) and, with a help string, the tooltip.
pub(crate) fn describe(
    tree: &mut UiTree,
    labels: Labels,
    element: usize,
    name: &str,
    help: Option<&str>,
) {
    if let Some(label) = tree.child_recursive(element, id("OptionText")) {
        tree.set_text(label, &labels.get(name));
    }
    if let Some(help) = help {
        tree.set_tooltip(element, Some(labels.get(help)));
    }
    tree.set_bool(element, id("tooltips_on"), help.is_some());
}

impl CharacterUi {
    /// `gmCharacterUI::gmCharacterUI` (and its `Setup` and `BuildList`): the page over the HUD
    /// tree's `CharacterPage`, with the expanders and options made from the toolbar layout's
    /// templates. `text` reads the UI string table `table` (its DID). The player's values come
    /// with [`CharacterUi::refresh`]. `None` without the page.
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
            .find_map(|&root| tree.child_recursive(root, id("CharacterPage")))?;
        let list = tree.child_recursive(page, id("CharacterList"));
        let apply = tree.child_recursive(page, id("ApplyButton"));
        let recall = tree.child_recursive(page, id("RecallButton"));
        let mut this = Self {
            page,
            list,
            apply,
            recall,
            entries: Vec::new(),
            main: Vec::new(),
            open: false,
            radar_mask: 0,
            context_recall: 0,
            context_default: 0,
            recall_text: labels.get(RECALL_TEXT),
            default_text: labels.get(DEFAULT_TEXT),
            requests: Vec::new(),
        };
        this.setup(tree, toolbar, labels);
        this.build_list(tree);
        Some(this)
    }

    /// `gmCharacterUI::Setup`: each expander (`OptionTree`, dismantled) and its options, in
    /// `SETUP`'s order; the expanders are top-level, the options go under them.
    fn setup(&mut self, tree: &mut UiTree, toolbar: &Arc<LayoutDesc>, labels: Labels) {
        for tree_desc in SETUP.iter() {
            let Some(element) = self.make(tree, toolbar, "OptionTree") else {
                continue;
            };
            let header = tree.child_recursive(element, id("TreeToggle"));
            let list = tree.child_recursive(element, id("TreeList"));
            describe(tree, labels, element, tree_desc.title, None);
            let index = self.entries.len();
            self.entries.push(Entry::Tree(TreeEntry {
                title: tree_desc.title,
                element,
                header,
                list,
                children: Vec::new(),
                open: false,
            }));
            self.main.push(index);
            for option in tree_desc.options {
                let entry = self.make_option(tree, toolbar, labels, option, index);
                if let Some(entry) = entry {
                    let at = self.entries.len();
                    self.entries.push(Entry::Option(entry));
                    if let Entry::Tree(tree_entry) = &mut self.entries[index] {
                        tree_entry.children.push(at);
                    }
                }
            }
        }
    }

    /// `OptionBase::Create`: the template `name` of the toolbar layout, made as an element.
    fn make(&self, tree: &mut UiTree, toolbar: &Arc<LayoutDesc>, name: &str) -> Option<usize> {
        let template = toolbar.element(id(name))?.clone();
        tree.create_element(Arc::clone(toolbar), template, false)
    }

    /// `OptionFlag::OptionFlag` / `OptionValue::OptionValue` / `OptionChoice::OptionChoice` with
    /// `Setup`'s defaults and labels.
    fn make_option(
        &self,
        tree: &mut UiTree,
        toolbar: &Arc<LayoutDesc>,
        labels: Labels,
        desc: &OptionDesc,
        parent: usize,
    ) -> Option<OptionEntry> {
        let element = self.make(tree, toolbar, template_name(desc.kind))?;
        let mut entry = OptionEntry {
            desc: *desc,
            parent: Some(parent),
            element,
            control: None,
            slider_buttons: [None, None],
            current: 0.0,
            saved: 0.0,
        };
        match desc.kind {
            Kind::Flag => {
                entry.control = tree.child_recursive(element, id("CheckboxButton"));
            }
            Kind::Value { .. } => {
                entry.control = tree.child_recursive(element, id("SliderBar"));
                entry.slider_buttons = [
                    tree.child_recursive(element, id("SliderLeft")),
                    tree.child_recursive(element, id("SliderRight")),
                ];
            }
            Kind::Choice { count } => {
                entry.control = tree.child_recursive(element, id("DropDownMenu"));
                let choices: Vec<(u32, String)> = (1..=count)
                    .map(|n| (n, labels.get(&choice_name(desc.name, n))))
                    .collect();
                fill_choices(tree, entry.control, &choices);
            }
        }
        // `SetDefault`: the default and the current value, as the widget shows them.
        entry.current = desc.default;
        describe(
            tree,
            labels,
            element,
            desc.name,
            Some(&format!("{}_Help", desc.name)),
        );
        sync_control(tree, &entry);
        Some(entry)
    }

    /// `gmCharacterUI::BuildList`: the top-level elements in `CharacterList`.
    fn build_list(&mut self, tree: &mut UiTree) {
        let Some(list) = self.list else {
            return;
        };
        for &index in &self.main {
            let element = match &self.entries[index] {
                Entry::Tree(tree_entry) => tree_entry.element,
                Entry::Option(entry) => entry.element,
            };
            tree.list_add_item(list, element);
        }
    }

    /// `gmCharacterUI::Refresh`: each option's value from the player's profile, and the value
    /// saved (`OptionBase::Refresh`'s commit), then the buttons.
    pub fn refresh(&mut self, tree: &mut UiTree, settings: &OptionsSettings) {
        self.radar_mask = settings.radar_mask;
        for index in 0..self.entries.len() {
            let Entry::Option(entry) = &mut self.entries[index] else {
                continue;
            };
            entry.current = read(entry.desc.target, settings);
            entry.saved = entry.current;
            sync_control(tree, entry);
        }
        self.update_buttons(tree);
    }

    /// `gmCharacterUI::Changed`: an option's value differs from its saved one.
    pub fn changed(&self) -> bool {
        self.options().any(is_changed)
    }

    fn options(&self) -> impl Iterator<Item = &OptionEntry> {
        self.entries.iter().filter_map(|e| match e {
            Entry::Option(option) => Some(option),
            Entry::Tree(_) => None,
        })
    }

    /// `gmCharacterUI::UpdateButtons`: Apply and Recall ghost while nothing has changed.
    pub fn update_buttons(&mut self, tree: &mut UiTree) {
        let changed = self.changed();
        for button in [self.apply, self.recall].into_iter().flatten() {
            tree.set_bool(button, BUTTON_GHOSTED, !changed);
        }
    }

    /// `gmCharacterUI::Apply`: each changed option's setter (`ApplyFlag` also sets the radar
    /// mask after every changed flag), its value saved, then the profile uploaded.
    pub fn apply(&mut self, tree: &mut UiTree) {
        for index in 0..self.entries.len() {
            let Entry::Option(entry) = &mut self.entries[index] else {
                continue;
            };
            if !is_changed(entry) {
                continue;
            }
            let desc = entry.desc;
            match desc.target {
                crate::character_options::Target::Radar(bit) => {
                    // `ApplyFlag`: the mask with this bit set or cleared; no setter of its own.
                    if entry.current != 0.0 {
                        self.radar_mask |= bit;
                    } else {
                        self.radar_mask &= !bit;
                    }
                }
                _ => {
                    self.requests.push(CharacterRequest::Set(write(
                        desc.setter,
                        desc.target,
                        entry.current,
                    )));
                    // `gmShortcutUI_DOCB` reads `GetShortcutLocked` as the drag starts.
                    if desc.setter == Setter::ShortcutLocked {
                        tree.set_shortcut_locked(entry.current != 0.0);
                    }
                }
            }
            if let Kind::Flag = desc.kind {
                self.requests
                    .push(CharacterRequest::Set(OptionSetting::Int {
                        setter: Setter::RadarMask,
                        value: self.radar_mask,
                    }));
            }
            entry.saved = entry.current;
        }
        self.requests.push(CharacterRequest::Upload);
        self.update_buttons(tree);
    }

    /// `gmCharacterUI::Recall`: every option back to its saved value.
    pub fn recall(&mut self, tree: &mut UiTree) {
        for index in 0..self.entries.len() {
            let Entry::Option(entry) = &mut self.entries[index] else {
                continue;
            };
            entry.current = entry.saved;
            sync_control(tree, entry);
        }
        self.update_buttons(tree);
    }

    /// `gmCharacterUI::Default`: every option to its default, then applied.
    pub fn default(&mut self, tree: &mut UiTree) {
        for index in 0..self.entries.len() {
            let Entry::Option(entry) = &mut self.entries[index] else {
                continue;
            };
            entry.current = entry.desc.default;
            sync_control(tree, entry);
        }
        self.apply(tree);
    }

    /// `gmCharacterUI::ConfirmRecall`: asks once while the question is open.
    pub fn confirm_recall(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_recall == 0 {
            self.context_recall =
                dialogs.make_dialog(tree, &self.recall_text, DIALOG_CONFIRMATION, false);
        }
    }

    /// `gmCharacterUI::ConfirmDefault`.
    pub fn confirm_default(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_default == 0 {
            self.context_default =
                dialogs.make_dialog(tree, &self.default_text, DIALOG_CONFIRMATION, false);
        }
    }

    /// `gmCharacterUI::ClearConfirmations`: the open questions close.
    fn clear_confirmations(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        for context in [self.context_recall, self.context_default] {
            if context != 0 {
                dialogs.close_dialog(tree, context);
            }
        }
        self.context_recall = 0;
        self.context_default = 0;
    }

    /// `gmCharacterUI::RecvNotice_CloseConfirmationDialog`: a yes runs the question's action.
    pub fn recv_close_confirmation_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        context: u32,
        yes: bool,
    ) {
        if yes {
            if context == self.context_recall {
                self.recall(tree);
            }
            if context == self.context_default {
                self.default(tree);
            }
        }
        if context == self.context_recall {
            self.context_recall = 0;
        }
        if context == self.context_default {
            self.context_default = 0;
        }
        self.clear_confirmations(tree, dialogs);
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

    /// `gmCharacterUI::RecvNotice_SetShortcutHeight`: the shortcut bar's height changed; the
    /// player's height, the drop-down and the upload follow.
    pub fn recv_shortcut_height(&mut self, tree: &mut UiTree, height: u32) {
        let Some(index) = self.option_index("ID_Option_ShortcutHeight") else {
            return;
        };
        self.requests
            .push(CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::ShortcutHeight,
                value: height,
            }));
        if let Entry::Option(entry) = &mut self.entries[index] {
            entry.current = f64::from(height);
            entry.saved = entry.current;
            sync_control(tree, entry);
        }
        self.requests.push(CharacterRequest::Upload);
    }

    /// `gmCharacterUI::ListenToMessage`. The widgets' own listeners run first and whether or not
    /// the tab shows: a checkbox's highlight or a slider's position is read into its option, an
    /// arrow steps a slider, an expander's header toggles it. Then, while the tab is open, the
    /// buttons' presses (`ApplyButton`, `RecallButton`, `DefaultButton`) and the tab's own
    /// `opened` / `closed` messages.
    pub fn message(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, message: &UiMessage) {
        if message.element_id == id("CharacterTab") {
            if message.message == MSG_OPENED {
                self.open = true;
                self.update_buttons(tree);
            } else if message.message == MSG_CLOSED {
                self.open = false;
            }
            return;
        }
        if message.message == MSG_MOUSE_PRESS {
            if let Some((index, step)) = self.arrow_at(message.element) {
                self.step_value(tree, index, step);
            }
        } else if message.message == MSG_HIGHLIGHT_STATE_CHANGE
            || message.message == MSG_NEW_POSITION
            || message.message == MSG_MENU_NEW_SELECTION
        {
            if let Some(index) = self.control_at(message.element) {
                self.read_control(tree, index);
            }
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            if let Some(index) = self.tree_at_header(message.element) {
                self.toggle(tree, index);
            }
        }
        if !self.open {
            return;
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            match message.element_id {
                e if e == id("ApplyButton") => self.apply(tree),
                e if e == id("RecallButton") => self.confirm_recall(tree, dialogs),
                e if e == id("DefaultButton") => self.confirm_default(tree, dialogs),
                _ => {}
            }
        }
        self.update_buttons(tree);
    }

    /// Sets an option's value as its control would when the player changes it (`0` or `1` for a
    /// checkbox, the range's value for a slider, the choice for a drop-down), then the buttons.
    /// Returns false for an unknown option or a choice outside the drop-down.
    pub fn set_value(&mut self, tree: &mut UiTree, name: &str, value: f64) -> bool {
        let Some(index) = self.option_index(name) else {
            return false;
        };
        let Entry::Option(entry) = &mut self.entries[index] else {
            return false;
        };
        match entry.desc.kind {
            Kind::Flag => entry.current = if value != 0.0 { 1.0 } else { 0.0 },
            Kind::Value { lower, upper } => entry.current = value.clamp(lower, upper),
            Kind::Choice { count } => {
                if !(1.0..=f64::from(count)).contains(&value) {
                    return false;
                }
                entry.current = value.trunc();
            }
        }
        sync_control(tree, entry);
        self.update_buttons(tree);
        true
    }

    /// The option `name`'s current value, as the page shows it.
    pub fn value(&self, name: &str) -> Option<f64> {
        self.option_index(name)
            .and_then(|i| match &self.entries[i] {
                Entry::Option(entry) => Some(entry.current),
                Entry::Tree(_) => None,
            })
    }

    /// The option `name`'s element (its control's element is [`CharacterUi::control`]).
    pub fn element(&self, name: &str) -> Option<usize> {
        self.option_index(name)
            .and_then(|i| match &self.entries[i] {
                Entry::Option(entry) => Some(entry.element),
                Entry::Tree(_) => None,
            })
    }

    /// The option `name`'s control: its checkbox (`CheckboxButton`), slider bar or drop-down.
    pub fn control(&self, name: &str) -> Option<usize> {
        self.option_index(name)
            .and_then(|i| match &self.entries[i] {
                Entry::Option(entry) => entry.control,
                Entry::Tree(_) => None,
            })
    }

    /// The expander titled `title`: whether it is open (its options listed).
    pub fn is_open(&self, title: &str) -> Option<bool> {
        self.entries.iter().find_map(|e| match e {
            Entry::Tree(tree_entry) if tree_entry.title == title => Some(tree_entry.open),
            _ => None,
        })
    }

    /// The options in `Setup`'s order: the expanders' titles and the options' names.
    pub fn order(&self) -> Vec<&'static str> {
        self.entries
            .iter()
            .map(|e| match e {
                Entry::Tree(tree_entry) => tree_entry.title,
                Entry::Option(entry) => entry.desc.name,
            })
            .collect()
    }

    /// The names of the top-level rows in `CharacterList`, in the list's order.
    pub fn listed(&self, tree: &UiTree) -> Vec<&'static str> {
        let Some(list) = self.list else {
            return Vec::new();
        };
        tree.list_items(list)
            .iter()
            .filter_map(|&element| {
                self.main.iter().find_map(|&i| match &self.entries[i] {
                    Entry::Tree(t) if t.element == element => Some(t.title),
                    Entry::Option(o) if o.element == element => Some(o.desc.name),
                    _ => None,
                })
            })
            .collect()
    }

    /// The expander titled `title`'s header button (`TreeToggle`).
    pub fn header(&self, title: &str) -> Option<usize> {
        self.entries.iter().find_map(|e| match e {
            Entry::Tree(t) if t.title == title => t.header,
            _ => None,
        })
    }

    /// How many option rows the expander titled `title` lists while it is open.
    pub fn rows(&self, tree: &UiTree, title: &str) -> Option<usize> {
        self.entries.iter().find_map(|e| match e {
            Entry::Tree(t) if t.title == title => t.list.map(|l| tree.list_items(l).len()),
            _ => None,
        })
    }

    /// The tab's `opened` state (`PanelPageMsg_opened`).
    pub fn page_open(&self) -> bool {
        self.open
    }

    /// The page's element (`CharacterPage`).
    pub fn page(&self) -> usize {
        self.page
    }

    /// The requests since the last take.
    pub fn take_requests(&mut self) -> Vec<CharacterRequest> {
        std::mem::take(&mut self.requests)
    }

    fn option_index(&self, name: &str) -> Option<usize> {
        let string = string_id(name);
        self.entries.iter().position(|e| match e {
            Entry::Option(entry) => string_id(entry.desc.name) == string,
            Entry::Tree(_) => false,
        })
    }

    /// The expander whose header is `element`.
    fn tree_at_header(&self, element: Option<usize>) -> Option<usize> {
        let element = element?;
        self.entries.iter().position(|e| match e {
            Entry::Tree(tree_entry) => tree_entry.header == Some(element),
            Entry::Option(_) => false,
        })
    }

    /// The option whose control is `element`.
    fn control_at(&self, element: Option<usize>) -> Option<usize> {
        let element = element?;
        self.entries.iter().position(|e| match e {
            Entry::Option(entry) => entry.control == Some(element),
            Entry::Tree(_) => false,
        })
    }

    /// The option and the step of the slider arrow `element` is (`OptionValue`'s left and right).
    fn arrow_at(&self, element: Option<usize>) -> Option<(usize, f64)> {
        let element = element?;
        self.entries.iter().enumerate().find_map(|(i, e)| match e {
            Entry::Option(entry) if entry.slider_buttons[0] == Some(element) => Some((i, -1.0)),
            Entry::Option(entry) if entry.slider_buttons[1] == Some(element) => Some((i, 1.0)),
            _ => None,
        })
    }

    /// `OptionValue::ListenToMessage`'s arrows: a tenth of the range.
    fn step_value(&mut self, tree: &mut UiTree, index: usize, step: f64) {
        let Entry::Option(entry) = &mut self.entries[index] else {
            return;
        };
        let Kind::Value { lower, upper } = entry.desc.kind else {
            return;
        };
        entry.current = (entry.current + step * (upper - lower) * 0.1).clamp(lower, upper);
        sync_control(tree, entry);
    }

    /// Reads a control's own state into the option (a checkbox's highlight, a slider's position).
    fn read_control(&mut self, tree: &UiTree, index: usize) {
        let Entry::Option(entry) = &mut self.entries[index] else {
            return;
        };
        let Some(control) = entry.control else {
            return;
        };
        match entry.desc.kind {
            Kind::Flag => {
                if let Some(AttributeValue::Bool(on)) = tree
                    .get(control)
                    .and_then(|e| e.attribute(BUTTON_HIGHLIGHTED))
                {
                    entry.current = if *on { 1.0 } else { 0.0 };
                }
            }
            Kind::Value { lower, upper } => {
                let position = f64::from(tree.scrollbar_position(control));
                entry.current = position * (upper - lower) + lower;
            }
            Kind::Choice { .. } => {
                if let Some(choice) = selected_choice(tree, entry.control) {
                    entry.current = choice;
                }
            }
        }
    }

    /// `OptionExpander::ListenToMessage`'s toggle: open lists its options
    /// (`Construct`), closed takes them out (`Dismantle`); the header's highlight follows.
    fn toggle(&mut self, tree: &mut UiTree, index: usize) {
        let Entry::Tree(tree_entry) = &mut self.entries[index] else {
            return;
        };
        let Some(list) = tree_entry.list else {
            return;
        };
        tree_entry.open = !tree_entry.open;
        let open = tree_entry.open;
        let children = tree_entry.children.clone();
        let header = tree_entry.header;
        if open {
            for child in children {
                if let Entry::Option(entry) = &self.entries[child] {
                    tree.list_add_item(list, entry.element);
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
    }
}

fn is_changed(entry: &OptionEntry) -> bool {
    is_changed_of(entry.desc.kind, entry.current, entry.saved)
}

/// Shows an option's value on its control: the checkbox's highlight, the slider's position, the
/// drop-down's selected item.
fn sync_control(tree: &mut UiTree, entry: &OptionEntry) {
    sync_option(tree, entry.control, entry.desc.kind, entry.current);
}
