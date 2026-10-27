//! `gmKeyboardUI`: the controls screen from character selection's options button.
//! `KeyboardField` holds the binding list (`list_box`) and its buttons. The list is made from the
//! maps the input manager holds (`Create`, `InitializeKeyboardList`): an action's header row
//! (`action_box_item`) and its two key rows (`key_box_item`), each row's text its `textid` child.
//! Selecting a key row waits for a key (`KeyHitHandler`, `StartSet`); clear, clear all, restore a
//! preset, undo, save and quit follow `ListenToMessage`, each dialog's answer following
//! `RecvNotice_CloseConfirmationDialog`. Save and restore hand the maps to the input manager
//! (`PregameRequest::KeyMaps`).
//!
//! Not modelled yet: the keymap file save (`keyboard_maps`), the `ICIDM` reserved-key check (no key
//! is reserved here), and the dialogs' retail text: the questions are worded here.

use crate::dialog::{DialogOutcome, DIALOG_CONFIRMATION, DIALOG_WAIT};
use crate::layout::AttributeValue;
use crate::list_box_select::MSG_NEW_SELECTION;
use crate::mouse::UiMessage;
use crate::pregame::flow::MODE_CHARACTER_SELECTION;
use crate::pregame::key_names::key_name;
use crate::pregame::keyboard_list::{KeyBinding, KeyboardList, SetOutcome, SlotRef, META_NONE};
use crate::pregame::keyboard_maps::{list_of, meta_key_of, saved_set, Preset};
use crate::pregame::{Ctx, PregameRequest, LAYOUT_KEYBOARD};
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};

/// The restore buttons and the preset each one picks.
const RESTORE_BUTTONS: [(&str, Preset); 3] = [
    ("restore_button", Preset::Default),
    ("restore_eq_button", Preset::Eq),
    ("restore_daoc_button", Preset::Daoc),
];

/// The open questions' contexts (`m_clearAllContext`, `m_already_mapped`, `m_restoreContext`,
/// `m_undoContext`, `m_exitContext`, `m_waitContext`); 0 is none.
#[derive(Debug, Clone, Copy, Default)]
struct Questions {
    clear_all: u32,
    already_mapped: u32,
    restore: u32,
    undo: u32,
    exit: u32,
    wait: u32,
}

impl Questions {
    /// `ListenToMessage` and `KeyHitHandler` take nothing while a question is up (the restore,
    /// undo, exit, clear-all and already-mapped ones).
    fn any_open(&self) -> bool {
        [
            self.clear_all,
            self.already_mapped,
            self.restore,
            self.undo,
            self.exit,
        ]
        .iter()
        .any(|&c| c != 0)
    }
}

/// `gmKeyboardUI`'s state.
#[derive(Debug, Clone, Default)]
pub(crate) struct KeyboardUi {
    field: Option<usize>,
    list: Option<usize>,
    /// `m_clearButton`, `m_applyButton`, `m_undoButton`.
    buttons: [Option<usize>; 3],
    model: KeyboardList,
    questions: Questions,
    /// `m_restoreMode`: the preset the restore question asks about.
    restore_mode: Option<Preset>,
}

impl KeyboardUi {
    /// `gmKeyboardUI::Create` and `gmKeyboardUI::gmKeyboardUI`: the field, its buttons and the
    /// list the input manager's maps make, then `UpdateButtons`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let field = ctx.assets.layout(LAYOUT_KEYBOARD).and_then(|layout| {
            ctx.tree
                .create_root(layout.clone(), tokenize("KeyboardField"))
        });
        let child = |ctx: &Ctx<'_>, name: &str| ctx.tree.child_recursive(field?, tokenize(name));
        let ui = Self {
            field,
            list: child(ctx, "list_box"),
            buttons: [
                child(ctx, "clear_button"),
                child(ctx, "save_button"),
                child(ctx, "undo_button"),
            ],
            model: ctx.keymaps.list(),
            ..Self::default()
        };
        ui.rebuild_rows(ctx);
        ui.update_buttons(ctx);
        ui
    }

    /// The framework's destructor: its questions closed (`CloseAllDialogs`), its root removed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        let q = self.questions;
        for context in [
            q.clear_all,
            q.already_mapped,
            q.restore,
            q.undo,
            q.exit,
            q.wait,
        ] {
            if context != 0 {
                ctx.dialogs.close_dialog(ctx.tree, context);
            }
        }
        if let Some(field) = self.field {
            ctx.tree.remove_root(field);
        }
    }

    /// `gmKeyboardUI::UpdateButtons`: clear needs a selected row; save and undo a change.
    fn update_buttons(&self, ctx: &mut Ctx<'_>) {
        let [Some(clear), Some(apply), Some(undo)] = self.buttons else {
            return;
        };
        ctx.tree
            .set_bool(clear, BUTTON_GHOSTED, self.model.selected().is_none());
        ctx.tree
            .set_bool(apply, BUTTON_GHOSTED, !self.model.modified());
        ctx.tree
            .set_bool(undo, BUTTON_GHOSTED, !self.model.modified());
    }

    /// `gmKeyboardUI::InitializeKeyboardList`'s items: an action's header, then its two keys, the
    /// texts from the names; the selected slot stays selected.
    fn rebuild_rows(&self, ctx: &mut Ctx<'_>) {
        let Some(list) = self.list else {
            return;
        };
        ctx.tree.list_flush(list);
        ctx.tree.list_multi_op_start(list);
        let Some(library) = ctx.keymaps.library.clone() else {
            ctx.tree.list_multi_op_complete(list);
            return;
        };
        let table = library.action_map.string_table;
        let layout = ctx.assets.layout(LAYOUT_KEYBOARD).cloned();
        for row in self.model.rows() {
            let action = ctx.strings.render_in(table, row.name);
            if let Some(item) = layout
                .as_ref()
                .and_then(|l| ctx.tree.create_from(l, tokenize("action_box_item")))
            {
                set_item_text(ctx, item, &action);
                ctx.tree.list_add_item(list, item);
            }
            for slot in row.slots {
                let text = slot.map(binding_text).unwrap_or_default();
                if let Some(item) = layout
                    .as_ref()
                    .and_then(|l| ctx.tree.create_from(l, tokenize("key_box_item")))
                {
                    set_item_text(ctx, item, &text);
                    ctx.tree.list_add_item(list, item);
                }
            }
        }
        ctx.tree.list_multi_op_complete(list);
        if let Some(slot) = self.model.selected() {
            ctx.tree
                .list_set_selected_index(list, Some(item_index(slot)), false);
        }
    }

    /// `gmKeyboardUI::Close`: back to character selection behind a please-wait dialog.
    fn close(&mut self, ctx: &mut Ctx<'_>) {
        let text = ctx.strings.get("ID_Wait_PleaseWait");
        self.questions.wait = ctx.dialogs.make_dialog(ctx.tree, &text, DIALOG_WAIT, false);
        ctx.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
    }

    /// `ListenToMessage`: the list's selection, and the buttons.
    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        if self.questions.any_open() {
            return;
        }
        let list_id = tokenize("list_box");
        if message.element_id == list_id && message.message == MSG_NEW_SELECTION {
            if let Some(index) = self.list.and_then(|l| ctx.tree.list_selected_index(l)) {
                self.click(ctx, index);
            }
            return;
        }
        if message.message != MSG_BUTTON_RELEASE_ON {
            return;
        }
        let id = message.element_id;
        if id == tokenize("clear_button") {
            self.clear_selected(ctx);
        } else if id == tokenize("save_button") {
            self.save(ctx);
        } else if id == tokenize("undo_button") && self.model.modified() {
            let text = "Undo every change since the last save?";
            self.questions.undo =
                ctx.dialogs
                    .make_dialog(ctx.tree, text, DIALOG_CONFIRMATION, false);
        } else if id == tokenize("quit_button") {
            if self.model.modified() {
                let text = "Leave the controls without saving your changes?";
                self.questions.exit =
                    ctx.dialogs
                        .make_dialog(ctx.tree, text, DIALOG_CONFIRMATION, false);
            } else {
                self.close(ctx);
            }
        } else if id == tokenize("clear_all_button") {
            let text = "Clear every key binding?";
            self.questions.clear_all =
                ctx.dialogs
                    .make_dialog(ctx.tree, text, DIALOG_CONFIRMATION, false);
        } else if let Some((_, preset)) = RESTORE_BUTTONS
            .iter()
            .find(|(name, _)| id == tokenize(name))
        {
            self.restore_mode = Some(*preset);
            let text = "Restore the key bindings of this preset?";
            self.questions.restore =
                ctx.dialogs
                    .make_dialog(ctx.tree, text, DIALOG_CONFIRMATION, false);
        }
        self.update_buttons(ctx);
    }

    /// A list selection: a header selects its first key (`ListenToMessage`'s index % 3 rule).
    fn click(&mut self, ctx: &mut Ctx<'_>, index: usize) {
        if let Some(slot) = self.model.click(index) {
            if let Some(list) = self.list {
                ctx.tree
                    .list_set_selected_index(list, Some(item_index(slot)), false);
            }
        }
        self.update_buttons(ctx);
    }

    /// The clear button: the selected key goes (`ListenToMessage`'s clear branch).
    fn clear_selected(&mut self, ctx: &mut Ctx<'_>) {
        if self.model.selected().is_none() {
            return;
        }
        self.model.clear_selected();
        self.rebuild_rows(ctx);
    }

    /// `SaveKeyboardList`: the three maps go to the input manager; nothing is unsaved.
    fn save(&mut self, ctx: &mut Ctx<'_>) {
        let names = ctx
            .keymaps
            .library
            .as_ref()
            .map(|library| library.action_names())
            .unwrap_or_default();
        let set = saved_set(&self.model, &names);
        self.model.set_modified(false);
        ctx.keymaps.active = Some(set.clone());
        ctx.requests.push(PregameRequest::KeyMaps(set.clone()));
        ctx.requests.push(PregameRequest::SaveKeyMaps(set));
    }

    /// The key hit handler: a key while a row is selected starts a set (`KeyHitHandler`).
    pub(crate) fn key_hit(&mut self, ctx: &mut Ctx<'_>, semantic: u32, meta_mode: u32) {
        let dialog_open = ctx.dialogs.shown().is_some();
        let meta = meta_key_of(meta_mode);
        let outcome = self.model.key_hit(semantic, meta, dialog_open, false);
        match outcome {
            Some(SetOutcome::AlreadyMapped { occupied }) => {
                let text = self.already_mapped_text(ctx, occupied, semantic, meta);
                self.questions.already_mapped =
                    ctx.dialogs
                        .make_dialog(ctx.tree, &text, DIALOG_CONFIRMATION, false);
            }
            Some(SetOutcome::Set) => {}
            None => return,
        }
        self.rebuild_rows(ctx);
        self.update_buttons(ctx);
    }

    /// The already-mapped question: the key and the action it is bound to.
    fn already_mapped_text(
        &self,
        ctx: &Ctx<'_>,
        occupied: SlotRef,
        semantic: u32,
        meta: u32,
    ) -> String {
        let action = ctx
            .keymaps
            .library
            .as_ref()
            .zip(self.model.rows().get(occupied.row))
            .map(|(library, row)| {
                ctx.strings
                    .render_in(library.action_map.string_table, row.name)
            })
            .unwrap_or_default();
        let key = binding_text(KeyBinding { semantic, meta });
        format!("{key} is already bound to {action}. Replace it?")
    }

    /// `RecvNotice_CloseConfirmationDialog`: each question's answer.
    pub(crate) fn dialog_outcome(&mut self, ctx: &mut Ctx<'_>, outcome: DialogOutcome) {
        let DialogOutcome::Confirmation { context, yes } = outcome else {
            return;
        };
        let q = &mut self.questions;
        if context == q.already_mapped && context != 0 {
            q.already_mapped = 0;
            self.model.answer_already_mapped(yes);
            self.rebuild_rows(ctx);
        } else if context == q.clear_all && context != 0 {
            q.clear_all = 0;
            if yes {
                self.model.clear_all();
                self.rebuild_rows(ctx);
            }
        } else if context == q.restore && context != 0 {
            q.restore = 0;
            if let (true, Some(preset)) = (yes, self.restore_mode) {
                self.restore(ctx, preset);
            }
        } else if context == q.undo && context != 0 {
            q.undo = 0;
            if yes {
                let modified = self.model.modified();
                self.model = ctx.keymaps.list();
                self.model.set_modified(modified);
                self.rebuild_rows(ctx);
            }
        } else if context == q.exit && context != 0 {
            q.exit = 0;
            if yes {
                self.close(ctx);
            }
        } else {
            return;
        }
        self.update_buttons(ctx);
    }

    /// The restore question's yes: the preset's maps are the input manager's, the list shows them,
    /// and the list is modified (it can be saved).
    fn restore(&mut self, ctx: &mut Ctx<'_>, preset: Preset) {
        let Some(library) = ctx.keymaps.library.clone() else {
            return;
        };
        let Some(set) = library.preset(preset) else {
            return;
        };
        self.model = list_of(&library.action_map, &set);
        self.model.set_modified(true);
        ctx.keymaps.active = Some(set.clone());
        ctx.requests.push(PregameRequest::KeyMaps(set));
        self.rebuild_rows(ctx);
    }
}

/// The listbox index of a slot: three items per action, the header first.
fn item_index(slot: SlotRef) -> usize {
    slot.row * 3 + slot.slot + 1
}

/// A key's text: the key, after its meta key (`InqKeyStringInfo`'s two parts).
fn binding_text(binding: KeyBinding) -> String {
    let key = key_name(binding.semantic);
    if binding.meta == META_NONE {
        key
    } else {
        format!("{}+{key}", key_name(binding.meta))
    }
}

/// `UIElement_Text::SetText` of an item's text child (its `textid` attribute).
fn set_item_text(ctx: &mut Ctx<'_>, item: usize, text: &str) {
    let child = match ctx.tree.el(item).attribute(tokenize("textid")) {
        Some(AttributeValue::Id(id)) => ctx.tree.child_recursive(item, *id),
        _ => None,
    };
    if let Some(child) = child {
        ctx.tree.set_text(child, text);
    }
}
