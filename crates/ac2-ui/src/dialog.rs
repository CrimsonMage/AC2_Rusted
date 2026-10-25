//! `DialogFactory`: one dialog shown at a time, the rest queued. `MakeDialog` numbers each
//! dialog with a new context and shows it at once when none is up; otherwise it waits at the
//! back of the queue, or (`topmost`) replaces the shown one, which goes back to the front of the
//! queue. Closing the shown dialog (`CloseDialog` with its context, `DialogDone`) shows the next
//! queued one; closing a queued one drops it. While a dialog is up the player cannot mouse-look
//! (`CM_UI::SendNotice_UpdateCanMouseLook`).
//!
//! The dialogs are child frameworks over the gameplay layout (`0x41000009`): a
//! `ConfirmationDialog` (`ConfirmationField`, its question in `QuestionText`) closes on `YesButton`
//! or `NoButton` and sends `CM_UI::SendNotice_CloseConfirmationDialog(context, yes)`; a
//! `WaitDialog` (`WaitField`, `TextField`) only shows its text; a `MessageDialog` (`MessageField`,
//! `TextField`) closes on `OkButton` (`SendNotice_CloseMessageDialog`); a `ResurrectDialog`
//! (`ResurrectField`, `TextField`) closes on `ResurrectButton` (`SendNotice_CloseResurrectDialog`).
//! The notices come back as [`DialogOutcome`]s.
//!
//! Not modelled yet: the timed dialog (`0x40000005`), which makes nothing here.

use std::collections::VecDeque;
use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

/// `DialogFactory::MakeDialog` types.
pub const DIALOG_CONFIRMATION: u32 = 0x4000_0001;
pub const DIALOG_WAIT: u32 = 0x4000_0002;
pub const DIALOG_RESURRECT: u32 = 0x4000_0003;
pub const DIALOG_MESSAGE: u32 = 0x4000_0004;
pub const DIALOG_TIMED: u32 = 0x4000_0005;
/// The layout the dialogs' roots come from (`UIChildFramework::CreateAndAddRootElement`).
pub const LAYOUT_DIALOGS: u32 = 0x4100_0009;

/// A dialog's notice to whoever made it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogOutcome {
    /// `CM_UI::SendNotice_CloseConfirmationDialog`.
    Confirmation { context: u32, yes: bool },
    /// `CM_UI::SendNotice_CloseMessageDialog`.
    Message { context: u32 },
    /// `CM_UI::SendNotice_CloseResurrectDialog`: the resurrect dialog's button.
    Resurrect { context: u32 },
}

/// `DialogInfo`: the type, text and context of a dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DialogInfo {
    kind: u32,
    text: String,
    context: u32,
}

/// The shown dialog: its info and root element (`_curInfo.pointer`).
#[derive(Debug, Clone)]
struct Shown {
    info: DialogInfo,
    root: Option<usize>,
}

/// `DialogFactory`'s state (`_globalContext`, `_curInfo`, `_dialogQueue`).
#[derive(Debug, Clone)]
pub struct DialogFactory {
    layout: Option<Arc<LayoutDesc>>,
    context: u32,
    shown: Option<Shown>,
    queue: VecDeque<DialogInfo>,
    outcomes: Vec<DialogOutcome>,
}

impl DialogFactory {
    /// The factory over the dialogs' layout (`None`: dialogs make nothing but still queue).
    pub fn new(layout: Option<Arc<LayoutDesc>>) -> Self {
        Self {
            layout,
            context: 0,
            shown: None,
            queue: VecDeque::new(),
            outcomes: Vec::new(),
        }
    }

    /// `DialogFactory::MakeDialog`: the new dialog's context.
    pub fn make_dialog(&mut self, tree: &mut UiTree, text: &str, kind: u32, topmost: bool) -> u32 {
        self.context += 1;
        let info = DialogInfo {
            kind,
            text: text.to_owned(),
            context: self.context,
        };
        match self.shown.take() {
            None => self.shown = Some(self.create(tree, info)),
            Some(shown) if !topmost => {
                self.shown = Some(shown);
                self.queue.push_back(info);
            }
            Some(shown) => {
                if let Some(root) = shown.root {
                    tree.remove_root(root);
                }
                self.queue.push_front(shown.info);
                self.shown = Some(self.create(tree, info));
            }
        }
        self.context
    }

    /// `DialogFactory::CloseDialog`: the shown dialog closes (the next queued one shows); a
    /// queued one is dropped. Context 0, or no dialog up, closes nothing.
    pub fn close_dialog(&mut self, tree: &mut UiTree, context: u32) {
        let Some(shown) = &self.shown else {
            return;
        };
        if context == 0 {
            return;
        }
        if shown.info.context == context {
            self.dialog_done(tree);
        } else if let Some(at) = self.queue.iter().position(|i| i.context == context) {
            self.queue.remove(at);
        }
    }

    /// `DialogFactory::DialogDone`.
    fn dialog_done(&mut self, tree: &mut UiTree) {
        let Some(shown) = self.shown.take() else {
            return;
        };
        if let Some(root) = shown.root {
            tree.remove_root(root);
        }
        if let Some(next) = self.queue.pop_front() {
            self.shown = Some(self.create(tree, next));
        }
    }

    /// `DialogFactory::CreateDialogInternal` and the dialog's constructor.
    fn create(&self, tree: &mut UiTree, info: DialogInfo) -> Shown {
        let (field, text) = match info.kind {
            DIALOG_CONFIRMATION => ("ConfirmationField", "QuestionText"),
            DIALOG_WAIT => ("WaitField", "TextField"),
            DIALOG_MESSAGE => ("MessageField", "TextField"),
            DIALOG_RESURRECT => ("ResurrectField", "TextField"),
            _ => {
                return Shown { info, root: None };
            }
        };
        let root = self
            .layout
            .as_ref()
            .and_then(|layout| tree.create_root(Arc::clone(layout), tokenize(field)));
        if let Some(root) = root {
            if let Some(text_element) = tree.child_recursive(root, tokenize(text)) {
                tree.set_text(text_element, &info.text);
            }
        }
        Shown { info, root }
    }

    /// The shown dialog's context and type.
    pub fn shown(&self) -> Option<(u32, u32)> {
        self.shown.as_ref().map(|s| (s.info.context, s.info.kind))
    }

    /// The contexts waiting behind the shown dialog.
    pub fn queued(&self) -> Vec<u32> {
        self.queue.iter().map(|i| i.context).collect()
    }

    /// `CM_UI::SendNotice_UpdateCanMouseLook`: mouse look is allowed while no dialog is up.
    pub fn can_mouse_look(&self) -> bool {
        self.shown.is_none()
    }

    /// The dialogs' `ListenToMessage`: a button released on the shown dialog closes it.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) {
        if message.message != MSG_BUTTON_RELEASE_ON {
            return;
        }
        let Some(shown) = &self.shown else {
            return;
        };
        let Some(root) = shown.root else {
            return;
        };
        if message.element.is_none_or(|e| tree.root_of(e) != root) {
            return;
        }
        let context = shown.info.context;
        let outcome = match shown.info.kind {
            DIALOG_CONFIRMATION => {
                let yes = message.element_id == tokenize("YesButton");
                Some(DialogOutcome::Confirmation { context, yes })
            }
            DIALOG_MESSAGE if message.element_id == tokenize("OkButton") => {
                Some(DialogOutcome::Message { context })
            }
            DIALOG_RESURRECT if message.element_id == tokenize("ResurrectButton") => {
                Some(DialogOutcome::Resurrect { context })
            }
            _ => None,
        };
        if let Some(outcome) = outcome {
            self.outcomes.push(outcome);
            self.close_dialog(tree, context);
        }
    }

    /// A confirmation's answer, as its yes or no button sends it (tests).
    #[cfg(test)]
    pub(crate) fn answer(&mut self, tree: &mut UiTree, context: u32, yes: bool) {
        self.close_dialog(tree, context);
        self.outcomes
            .push(DialogOutcome::Confirmation { context, yes });
    }

    /// The notices the dialogs sent since the last call.
    pub fn take_outcomes(&mut self) -> Vec<DialogOutcome> {
        std::mem::take(&mut self.outcomes)
    }
}

impl UiTree {
    /// An element taken from its parent and deleted with its descendants (`delete`).
    pub fn delete_element(&mut self, index: usize) {
        if self.get(index).is_none() {
            return;
        }
        self.set_parent(index, None);
        self.delete(index);
        self.changed = true;
    }

    /// `UIMainFramework::RemoveRootElement`: a root and its descendants deleted.
    pub fn remove_root(&mut self, root: usize) {
        if self.get(root).is_none() {
            return;
        }
        self.roots.retain(|&r| r != root);
        self.delete(root);
        self.changed = true;
        self.moved = true;
    }
}
