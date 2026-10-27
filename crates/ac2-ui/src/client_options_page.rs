//! `gmClientUI`'s widgets: the client options page (`ClientPage` under the `ClientTab` of the
//! option panel). `Construct` makes one expander per section of [`SECTION_ORDER`] (`OptionTree`,
//! listed in `ClientList`); each expander lists the preferences the bank holds in its section,
//! as `OptionExpander::AddOption` does with `OptionFlag`, `OptionValue` and `OptionChoice`
//! widgets. The [`PreferenceBank`] is the one store of the values: the widgets show them and the
//! player's changes are written into it. Apply runs at once; Recall and Default ask first
//! ([`ClientPage`] keeps the questions). What the answers do to the client (the file, the live
//! sound, the scene, the window) is the caller's: the page emits [`ClientRequest`]s.
//!
//! A preference's label is the name the caller gives it (`label_of`: the registration's
//! `Tokenize`d name, with `_Help` for its tooltip), read from the preferences' label table; where
//! that table has no string, the row shows the preference's INI key name. A preference with no
//! name shows no row. A choice's label is its own string ID. An expander's header is its section's name from the DAT's
//! enum tables through a [`NameTable`], as text (the section string IDs are not identified).
//! Preferences that are text show no row.
//!
//! Not modelled: the drop-down's menu (`UIElement_Menu`: the choice changes through
//! [`ClientOptionsUi::set_value`] only, its label shows on `DropDownMenu`) and the expanders'
//! `Refit` sizes.

use std::sync::Arc;

use ac2_prefs::{NameTable, Pref, PreferenceBank, Value};

use crate::character::{describe, Labels};
use crate::character_options::{sync_control_of, template_name, Kind};
use crate::client_page::{ClientPage, ClientPageText, PageAction, SECTION_ORDER};
use crate::dialog::{DialogFactory, DialogOutcome};
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
use crate::scrollbar::MSG_NEW_POSITION;
use crate::text::UiTextData;
use crate::token::{
    tokenize, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON,
    MSG_HIGHLIGHT_STATE_CHANGE, MSG_MOUSE_PRESS,
};

/// `MSG_PANEL_PAGE_CLOSED`: the tab shut.
const MSG_CLOSED: u32 = tokenize("closed");
/// The string table enum the preferences' labels are in (`Preference::Describe` sets
/// `mStringTable = 0x40000008`), under the string table group `STRING_TABLE_GROUP`.
pub const PREF_LABEL_TABLE: u32 = 0x4000_0008;
/// The confirmations' texts (`ID_Option_RecallText`, `ID_Option_DefaultText`).
const RECALL_TEXT: &str = "ID_Option_RecallText";
const DEFAULT_TEXT: &str = "ID_Option_DefaultText";
/// The render verification's question (`VerifyRenderOptions`); its string ID is not identified,
/// so the question is this port's own text when the table has none under the name below.
const RENDER_TEXT: &str = "ID_Option_VerifyRenderText";
const RENDER_FALLBACK: &str = "Keep these display settings?";

/// What the page asks the client to do with the bank.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientRequest {
    /// `gmClientUI::Apply`: every section's changes are applied (the display section is verified).
    Apply,
    /// A question's answer (`gmClientUI::RecvNotice_CloseConfirmationDialog`), as the page's
    /// actions (`ClientPage::recv_close_confirmation`).
    Answer(Vec<PageAction>),
}

/// An option's widgets: the checkbox, slider or drop-down, and its slider arrows.
#[derive(Debug, Clone, PartialEq)]
struct Row {
    key: u32,
    /// The label name (`ID_<section>_<key>`), which the row is found by.
    name: String,
    element: usize,
    kind: Kind,
    control: Option<usize>,
    slider_buttons: [Option<usize>; 2],
    /// The drop-down's labels, for choices `1..=count` (the bank's choice order).
    labels: Vec<String>,
    /// The bank's values of the choices, in the same order as `labels`.
    choice_values: Vec<i32>,
}

/// An expander (`OptionExpander`): its header, its list while open, and its rows.
#[derive(Debug, Clone, PartialEq)]
struct Expander {
    section: u32,
    element: usize,
    header: Option<usize>,
    list: Option<usize>,
    rows: Vec<usize>,
    open: bool,
}

/// `gmClientUI`'s widgets over the page element.
#[derive(Debug, Clone)]
pub struct ClientOptionsUi {
    page: ClientPage,
    page_element: usize,
    list: Option<usize>,
    apply: Option<usize>,
    recall: Option<usize>,
    expanders: Vec<Expander>,
    rows: Vec<Row>,
    /// The tab shows the page (`PanelPageMsg_opened`).
    open: bool,
    requests: Vec<ClientRequest>,
}

/// The layout's `ClientPage` with its expanders made from the bank, or `None` without the page.
impl ClientOptionsUi {
    /// `gmClientUI::gmClientUI` and `Construct`: the page over the tree's `ClientPage`, with the
    /// expanders and rows made from the toolbar layout's templates and the bank's preferences.
    /// `tables` are the DIDs of the UI string table (the page's questions) and of the preferences'
    /// label table ([`PREF_LABEL_TABLE`]); `label_of` names a preference by its section and key.
    /// `None` without the page.
    pub fn create(
        tree: &mut UiTree,
        toolbar: &Arc<LayoutDesc>,
        text: &dyn UiTextData,
        tables: (u32, u32),
        bank: &PreferenceBank,
        names: &dyn NameTable,
        label_of: fn(u32, u32) -> Option<&'static str>,
    ) -> Option<Self> {
        let (table, pref_table) = tables;
        let page_element = tree
            .roots()
            .iter()
            .find_map(|&root| tree.child_recursive(root, tokenize("ClientPage")))?;
        let labels = Labels { text, table };
        let pref_labels = Labels {
            text,
            table: pref_table,
        };
        let page_text = ClientPageText {
            recall: labels.get(RECALL_TEXT),
            default: labels.get(DEFAULT_TEXT),
            render: match labels.get(RENDER_TEXT) {
                text if text.is_empty() => RENDER_FALLBACK.to_owned(),
                text => text,
            },
        };
        let mut this = Self {
            page: ClientPage::new(page_text),
            page_element,
            list: tree.child_recursive(page_element, tokenize("ClientList")),
            apply: tree.child_recursive(page_element, tokenize("ApplyButton")),
            recall: tree.child_recursive(page_element, tokenize("RecallButton")),
            expanders: Vec::new(),
            rows: Vec::new(),
            open: false,
            requests: Vec::new(),
        };
        this.setup(tree, toolbar, pref_labels, bank, names, label_of);
        this.build_list(tree);
        this.update_buttons(tree, bank);
        Some(this)
    }

    /// `gmClientUI::Construct`: an expander per section of [`SECTION_ORDER`], each with the rows of
    /// the bank's preferences in its section.
    fn setup(
        &mut self,
        tree: &mut UiTree,
        toolbar: &Arc<LayoutDesc>,
        labels: Labels,
        bank: &PreferenceBank,
        names: &dyn NameTable,
        label_of: fn(u32, u32) -> Option<&'static str>,
    ) {
        for section in SECTION_ORDER {
            let Some(element) = self.make(tree, toolbar, "OptionTree") else {
                continue;
            };
            let title = names.section_name(section).unwrap_or_default();
            if let Some(text) = tree.child_recursive(element, tokenize("OptionText")) {
                tree.set_text(text, &title);
            }
            let header = tree.child_recursive(element, tokenize("TreeToggle"));
            let list = tree.child_recursive(element, tokenize("TreeList"));
            let index = self.expanders.len();
            self.expanders.push(Expander {
                section,
                element,
                header,
                list,
                rows: Vec::new(),
                open: false,
            });
            for key in bank.list(section) {
                let Some(pref) = bank.find(key) else {
                    continue;
                };
                if let Some(row) = self.make_row(tree, toolbar, labels, pref, names, label_of) {
                    self.expanders[index].rows.push(self.rows.len());
                    self.rows.push(row);
                }
            }
        }
    }

    /// `OptionBase::Create`: the template `name` of the toolbar layout, made as an element.
    fn make(&self, tree: &mut UiTree, toolbar: &Arc<LayoutDesc>, name: &str) -> Option<usize> {
        let template = toolbar.element(tokenize(name))?.clone();
        tree.create_element(Arc::clone(toolbar), template, false)
    }

    /// One preference's widget, labelled and showing its current value.
    fn make_row(
        &self,
        tree: &mut UiTree,
        toolbar: &Arc<LayoutDesc>,
        labels: Labels,
        pref: &Pref,
        names: &dyn NameTable,
        label_of: fn(u32, u32) -> Option<&'static str>,
    ) -> Option<Row> {
        let kind = match &pref.value {
            Value::Bool(_) => Kind::Flag,
            Value::Float(_) => Kind::Value {
                lower: f64::from(pref.min),
                upper: f64::from(pref.max),
            },
            Value::Int(_) if !pref.choices.is_empty() => Kind::Choice {
                count: pref.choices.len() as u32,
            },
            Value::Int(_) | Value::Text(_) => return None,
        };
        let name = label_of(pref.section, pref.key)?.to_owned();
        let element = self.make(tree, toolbar, template_name(kind))?;
        let mut row = Row {
            key: pref.key,
            name: name.clone(),
            element,
            kind,
            control: None,
            slider_buttons: [None, None],
            labels: Vec::new(),
            choice_values: Vec::new(),
        };
        match kind {
            Kind::Flag => row.control = tree.child_recursive(element, tokenize("CheckboxButton")),
            Kind::Value { .. } => {
                row.control = tree.child_recursive(element, tokenize("SliderBar"));
                row.slider_buttons = [
                    tree.child_recursive(element, tokenize("SliderLeft")),
                    tree.child_recursive(element, tokenize("SliderRight")),
                ];
            }
            Kind::Choice { .. } => {
                row.control = tree.child_recursive(element, tokenize("DropDownMenu"));
                row.labels = pref
                    .choices
                    .iter()
                    .map(|c| labels.get_id(c.label))
                    .collect();
                row.choice_values = pref.choices.iter().map(|c| c.value).collect();
            }
        }
        describe(tree, labels, element, &name, Some(&format!("{name}_Help")));
        // The label is not in the preferences' table: the row shows the preference's INI key name.
        if labels.get(&name).is_empty() {
            if let (Some(text), Some(key)) = (
                tree.child_recursive(element, tokenize("OptionText")),
                names.key_name(pref.key),
            ) {
                tree.set_text(text, &key);
            }
        }
        sync_row(tree, &row, pref);
        Some(row)
    }

    /// `gmClientUI`'s `OptionExpander::Construct`: the top-level expanders in `ClientList`.
    fn build_list(&mut self, tree: &mut UiTree) {
        let Some(list) = self.list else {
            return;
        };
        for expander in &self.expanders {
            tree.list_add_item(list, expander.element);
        }
    }

    /// The bank's preference of `row`.
    fn pref<'a>(bank: &'a PreferenceBank, row: &Row) -> Option<&'a Pref> {
        bank.find(row.key)
    }

    /// `gmClientUI::Changed`: some section's preference differs from its applied value.
    pub fn changed(&self, bank: &PreferenceBank) -> bool {
        SECTION_ORDER
            .iter()
            .any(|&section| !bank.submit(section).is_empty())
    }

    /// `gmClientUI::UpdateButtons`: Apply and Recall (and Default) follow whether something changed.
    pub fn update_buttons(&mut self, tree: &mut UiTree, bank: &PreferenceBank) {
        let Some(states) = self.page.buttons(self.changed(bank)) else {
            return;
        };
        for (button, enabled) in [(self.apply, states.apply), (self.recall, states.recall)] {
            if let Some(button) = button {
                tree.set_bool(button, BUTTON_GHOSTED, !enabled);
            }
        }
    }

    /// Shows every row's value from the bank (after a refresh, an apply or a recall), then the
    /// buttons.
    pub fn refresh(&mut self, tree: &mut UiTree, bank: &PreferenceBank) {
        for row in &self.rows {
            if let Some(pref) = Self::pref(bank, row) {
                sync_row(tree, row, pref);
            }
        }
        self.update_buttons(tree, bank);
    }

    /// The page's expanders are open (the rows listed) while its header is toggled.
    fn toggle(&mut self, tree: &mut UiTree, bank: &PreferenceBank, index: usize) {
        let expander = &mut self.expanders[index];
        let Some(list) = expander.list else {
            return;
        };
        expander.open = !expander.open;
        let open = expander.open;
        let rows = expander.rows.clone();
        let header = expander.header;
        if open {
            for row in rows {
                tree.list_add_item(list, self.rows[row].element);
            }
        } else {
            while !tree.list_items(list).is_empty() {
                tree.list_remove_item(list, 0);
            }
        }
        if let Some(header) = header {
            tree.set_bool(header, BUTTON_HIGHLIGHTED, open);
        }
        for row in self.rows.clone() {
            if let Some(pref) = Self::pref(bank, &row) {
                sync_row(tree, &row, pref);
            }
        }
    }

    /// The row whose control is `element`.
    fn control_at(&self, element: Option<usize>) -> Option<usize> {
        let element = element?;
        self.rows.iter().position(|r| r.control == Some(element))
    }

    /// The row and the step of the slider arrow `element` is (`OptionValue`'s left and right).
    fn arrow_at(&self, element: Option<usize>) -> Option<(usize, f64)> {
        let element = element?;
        self.rows.iter().enumerate().find_map(|(i, r)| {
            if r.slider_buttons[0] == Some(element) {
                Some((i, -1.0))
            } else if r.slider_buttons[1] == Some(element) {
                Some((i, 1.0))
            } else {
                None
            }
        })
    }

    /// The expander whose header is `element`.
    fn header_at(&self, element: Option<usize>) -> Option<usize> {
        let element = element?;
        self.expanders
            .iter()
            .position(|e| e.header == Some(element))
    }

    /// `OptionValue::ListenToMessage`'s arrows: a tenth of the range.
    fn step_value(&self, tree: &mut UiTree, bank: &mut PreferenceBank, index: usize, step: f64) {
        let row = &self.rows[index];
        let Kind::Value { lower, upper } = row.kind else {
            return;
        };
        let Some(pref) = bank.find_mut(row.key) else {
            return;
        };
        let current = f64::from(float_of(pref));
        let value = (current + step * (upper - lower) * 0.1).clamp(lower, upper);
        pref.set_float(value as f32);
        sync_row(tree, row, pref);
    }

    /// Reads a control's own state into its preference (a checkbox's highlight, a slider's
    /// position); a drop-down changes only through [`ClientOptionsUi::set_value`].
    fn read_control(&self, tree: &UiTree, bank: &mut PreferenceBank, index: usize) {
        let row = &self.rows[index];
        let Some(control) = row.control else {
            return;
        };
        let Some(pref) = bank.find_mut(row.key) else {
            return;
        };
        match row.kind {
            Kind::Flag => {
                if let Some(AttributeValue::Bool(on)) = tree
                    .get(control)
                    .and_then(|e| e.attribute(BUTTON_HIGHLIGHTED))
                {
                    pref.set_bool(*on);
                }
            }
            Kind::Value { lower, upper } => {
                let position = f64::from(tree.scrollbar_position(control));
                pref.set_float((position * (upper - lower) + lower) as f32);
            }
            Kind::Choice { .. } => {}
        }
    }

    /// Sets a preference as its widget would when the player changes it: `0` or `1` for a
    /// checkbox, the range's value for a slider, the choice's number (`1..=count`) for a
    /// drop-down. Returns false for an unknown option or a choice outside the drop-down.
    pub fn set_value(
        &mut self,
        tree: &mut UiTree,
        bank: &mut PreferenceBank,
        name: &str,
        value: f64,
    ) -> bool {
        let Some(index) = self.row_index(name) else {
            return false;
        };
        let row = self.rows[index].clone();
        let Some(pref) = bank.find_mut(row.key) else {
            return false;
        };
        match row.kind {
            Kind::Flag => pref.set_bool(value != 0.0),
            Kind::Value { lower, upper } => pref.set_float(value.clamp(lower, upper) as f32),
            Kind::Choice { count } => {
                if !(1.0..=f64::from(count)).contains(&value) {
                    return false;
                }
                let choice = row.choice_values[value.trunc() as usize - 1];
                if !pref.set_choice(choice) {
                    return false;
                }
            }
        }
        sync_row(tree, &row, pref);
        self.update_buttons(tree, bank);
        true
    }

    /// The row of the preference named `name` (`ID_<section>_<key>`).
    fn row_index(&self, name: &str) -> Option<usize> {
        self.rows.iter().position(|row| row.name == name)
    }

    /// The value of the preference named `name`, as its widget shows it (see [`ClientOptionsUi::set_value`]).
    pub fn value(&self, bank: &PreferenceBank, name: &str) -> Option<f64> {
        let row = &self.rows[self.row_index(name)?];
        let pref = Self::pref(bank, row)?;
        Some(shown_value(pref, row))
    }

    /// `gmClientUI::Apply`'s button: the request waits for [`ClientOptionsUi::take_requests`].
    fn apply_pressed(&mut self) {
        self.requests.push(ClientRequest::Apply);
    }

    /// `gmClientUI::ConfirmRecall`: asks once while the question is open.
    pub fn confirm_recall(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        self.page.confirm_recall(tree, dialogs);
    }

    /// `gmClientUI::ConfirmDefault`: asks once while the question is open.
    pub fn confirm_default(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        self.page.confirm_default(tree, dialogs);
    }

    /// `gmClientUI::VerifyRenderOptions` through [`ClientPage::after_apply`]: an Apply that changed
    /// the display asks for its verification (a timed dialog). Returns whether it was asked.
    pub fn after_apply(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        bank: &PreferenceBank,
        verify_render: bool,
    ) -> bool {
        let asked = self.page.after_apply(tree, dialogs, verify_render);
        self.update_buttons(tree, bank);
        asked
    }

    /// The confirmations' answers (`DialogFactory`'s outcomes). Only a question this page opened
    /// is its own: its answer becomes a [`ClientRequest::Answer`] when it does something.
    pub fn dialog_outcomes(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        outcomes: &[DialogOutcome],
    ) {
        for outcome in outcomes {
            let DialogOutcome::Confirmation { context, yes } = *outcome else {
                continue;
            };
            if !self.page.open_contexts().contains(&context) {
                continue;
            }
            let actions = self
                .page
                .recv_close_confirmation(tree, dialogs, context, yes);
            if !actions.is_empty() {
                self.requests.push(ClientRequest::Answer(actions));
            }
        }
    }

    /// `gmClientUI::ListenToMessage`: the widgets' own listeners run first (checkbox, slider and
    /// arrows, expander headers), then, while the tab is open, the buttons' presses. The tab's
    /// `opened` and `closed` messages come first of all.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        bank: &mut PreferenceBank,
        message: &UiMessage,
    ) {
        if message.element_id == tokenize("ClientTab") {
            if message.message == MSG_OPENED {
                self.open = true;
                self.update_buttons(tree, bank);
            } else if message.message == MSG_CLOSED {
                self.open = false;
            }
            return;
        }
        if message.message == MSG_MOUSE_PRESS {
            if let Some((index, step)) = self.arrow_at(message.element) {
                self.step_value(tree, bank, index, step);
            }
        } else if message.message == MSG_HIGHLIGHT_STATE_CHANGE
            || message.message == MSG_NEW_POSITION
        {
            if let Some(index) = self.control_at(message.element) {
                self.read_control(tree, bank, index);
            }
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            if let Some(index) = self.header_at(message.element) {
                self.toggle(tree, bank, index);
            }
        }
        if !self.open {
            return;
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            match message.element_id {
                e if e == tokenize("ApplyButton") => self.apply_pressed(),
                e if e == tokenize("RecallButton") => self.confirm_recall(tree, dialogs),
                e if e == tokenize("DefaultButton") => self.confirm_default(tree, dialogs),
                _ => {}
            }
        }
        self.update_buttons(tree, bank);
    }

    /// The requests since the last call (the caller answers them with the client's preferences).
    pub fn take_requests(&mut self) -> Vec<ClientRequest> {
        std::mem::take(&mut self.requests)
    }

    /// Whether the tab shows the page.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// The label a row shows for the preference named `name` (`ID_<section>_<key>`).
    pub fn row_label(&self, tree: &UiTree, name: &str) -> Option<String> {
        let row = &self.rows[self.row_index(name)?];
        let label = tree.child_recursive(row.element, tokenize("OptionText"))?;
        tree.get(label)?.text()
    }

    /// The page element (`ClientPage`).
    pub fn page_element(&self) -> usize {
        self.page_element
    }

    /// The section ids of the expanders, in order.
    pub fn sections(&self) -> Vec<u32> {
        self.expanders.iter().map(|e| e.section).collect()
    }

    /// The `ClientPage`'s dialog model (its open questions).
    pub fn page(&self) -> &ClientPage {
        &self.page
    }
}

impl Labels<'_> {
    /// The string with ID `id` of the table, as text (empty when the table has none).
    pub(crate) fn get_id(&self, id: u32) -> String {
        self.text
            .string(self.table, id)
            .map(|units| String::from_utf16_lossy(&units))
            .unwrap_or_default()
    }
}

/// A float preference's value (zero for any other type).
fn float_of(pref: &Pref) -> f32 {
    match pref.value {
        Value::Float(v) => v,
        _ => 0.0,
    }
}

/// The value a row shows: a checkbox's 0 or 1, a slider's value, a drop-down's choice number.
fn shown_value(pref: &Pref, row: &Row) -> f64 {
    match (&pref.value, row.kind) {
        (Value::Bool(on), _) => f64::from(u8::from(*on)),
        (Value::Float(v), _) => f64::from(*v),
        (Value::Int(choice), _) => row
            .choice_values
            .iter()
            .position(|&v| v == *choice)
            .map_or(0.0, |n| (n + 1) as f64),
        (Value::Text(_), _) => 0.0,
    }
}

/// Shows a preference's value on its row's control.
fn sync_row(tree: &mut UiTree, row: &Row, pref: &Pref) {
    let current = shown_value(pref, row);
    sync_control_of(tree, row.control, row.kind, current, &row.labels);
}
