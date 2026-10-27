//! `gmAssistanceUI` on the option toolbar: the assistance page (`AssistancePage`) where a player
//! calls for a GM. The page has an abuser name entry (`AbuserNameEntryText`), a description entry
//! (`AssistanceDescriptionEntryText`), the call-type drop-down (`CallTypeMenu`, a `UIElement_Menu`
//! whose selection shows in `CallTypeSelection`) and the send button (`AssistanceSendButton`).
//!
//! The call types are the GUN `SUPPORT_CALLTYPE` values. Each menu item carries its call type as
//! the `GUNCallType` attribute, which the send reads from the selected item
//! (`gmAssistanceUI::HandleSendButton`). A call type with bit 0x4000 needs an abuser name and one
//! with bit 0x8000 needs a description; the first missing one answers with its message box. The
//! complete call goes to the GUN service (`gmCWInterface::RequestGUN_CallCreate`): the panel
//! returns it as an [`AssistanceRequest`], the client answers with whether it was queued, and
//! [`AssistancePanel::send_answered`] shows the "can't connect" box, or resets the page.
//!
//! The toolbar's `AssistanceButton` closes an open message box (`ListenToMessage`). Not modelled
//! yet: the world name the call carries (`Turbine::Client`'s world, not kept by the client), and the
//! order of the eight call types the decompile does not trace (see [`CALL_TYPES`]).

use crate::dialog::{DialogFactory, DIALOG_MESSAGE};
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::menu::MSG_MENU_NEW_SELECTION;
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};
use std::sync::Arc;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `CALLTYPE_*`: the call types the drop-down holds, in the order `InitCallTypeMenu` inserts them.
/// Each item is inserted at index 0 (retail's `InsertTextItem(menu, …, 0)`), so the list shows
/// them in reverse. The first is `ID_Assistance_CallType_Other`, as the admin decompile names it;
/// the rest follow the GUN enum's declaration order (the decompile drops the setup of those
/// strings, so their order is not traced).
pub const CALL_TYPES: [(&str, u32); 9] = [
    ("ID_Assistance_CallType_Other", 0x8008),
    ("ID_Assistance_CallType_Lag", 0x0004),
    ("ID_Assistance_CallType_Macro", 0x4006),
    ("ID_Assistance_CallType_Spam", 0x6002),
    ("ID_Assistance_CallType_Stuck", 0x8003),
    ("ID_Assistance_CallType_Bug", 0x8005),
    ("ID_Assistance_CallType_Abuse", 0xE000),
    ("ID_Assistance_CallType_Harassment", 0xE001),
    ("ID_Assistance_CallType_OtherCOC", 0xE007),
];

/// The call type needs an abuser name (`gmAssistanceUI::HandleSendButton`'s 0x4000 test).
const NEEDS_ABUSER: u32 = 0x4000;
/// The call type needs a description (its sign bit, `(char)(calltype >> 8) < 0`).
const NEEDS_DESCRIPTION: u32 = 0x8000;

/// What the page asks of the GUN service (`gmCWInterface::RequestGUN_CallCreate`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistanceRequest {
    CallCreate {
        call_type: u32,
        world: String,
        target: String,
        description: String,
    },
}

/// The assistance page's state: its elements, and the message box it has open.
#[derive(Debug, Clone)]
pub struct AssistancePanel {
    /// The toolbar layout's string table (the UI strings the page shows).
    table: u32,
    /// `_assistancePage`.
    page: Option<usize>,
    /// `_descriptionText`, `_abuserText`, `_calltypeMenu`, `_sendButton`.
    description: Option<usize>,
    abuser: Option<usize>,
    menu: Option<usize>,
    send: Option<usize>,
    /// `_dialogContext`: the message box the page opened (0 for none).
    dialog: u32,
}

/// `GUNCallType`: the item attribute that holds the call type.
fn call_type_attribute() -> u32 {
    id("GUNCallType")
}

impl AssistancePanel {
    /// `gmAssistanceUI::gmAssistanceUI`: the page and its elements found, the call-type menu
    /// filled, then `Reset`.
    pub fn create(tree: &mut UiTree, toolbar: &Arc<LayoutDesc>) -> Self {
        let page = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("AssistancePage")));
        let mut panel = Self {
            table: toolbar.string_table,
            page,
            description: None,
            abuser: None,
            menu: None,
            send: None,
            dialog: 0,
        };
        if let Some(page) = page {
            panel.description = tree.child_recursive(page, id("AssistanceDescriptionEntryText"));
            panel.abuser = tree.child_recursive(page, id("AbuserNameEntryText"));
            panel.menu = tree.child_recursive(page, id("CallTypeMenu"));
            panel.send = tree.child_recursive(page, id("AssistanceSendButton"));
        }
        panel.init_call_type_menu(tree);
        panel.reset(tree);
        panel
    }

    /// Whether the page was found in the layout.
    pub fn has_page(&self) -> bool {
        self.page.is_some()
    }

    /// The call-type drop-down, for the page's messages and tests.
    pub fn menu(&self) -> Option<usize> {
        self.menu
    }

    /// `gmAssistanceUI::InitCallTypeMenu`: each call type inserted at the top of the menu, its
    /// `GUNCallType` set.
    fn init_call_type_menu(&self, tree: &mut UiTree) {
        let Some(menu) = self.menu else {
            return;
        };
        for (name, call_type) in CALL_TYPES {
            let Some(item) = tree.menu_insert_string(menu, self.table, id(name), 0) else {
                return;
            };
            tree.set_int(item, call_type_attribute(), call_type as i32);
        }
    }

    /// `gmAssistanceUI::Reset`: no call type selected, both entries cleared, the send button
    /// ghosted, and the selection shows `ID_Assistance_CallType_Default`.
    pub fn reset(&mut self, tree: &mut UiTree) {
        let (Some(menu), Some(abuser), Some(description), Some(send)) =
            (self.menu, self.abuser, self.description, self.send)
        else {
            return;
        };
        tree.menu_set_selected_item(menu, None);
        // Retail's selection listener runs inside the call above, before the send button is
        // ghosted below; the queued message would otherwise unghost it.
        tree.menu_drop_selection_messages(menu);
        tree.set_text(abuser, "");
        tree.set_text(description, "");
        tree.set_bool(send, BUTTON_GHOSTED, true);
        if let Some(selection) = self
            .page
            .and_then(|page| tree.child_recursive(page, id("CallTypeSelection")))
        {
            let text = ui_text(tree, self.table, id("ID_Assistance_CallType_Default"));
            tree.set_text(selection, &text);
        }
    }

    /// `gmAssistanceUI::RecvNotice_CloseMessageDialog`: the message box is no longer open.
    pub fn close_message(&mut self, context: u32) {
        if context == self.dialog {
            self.dialog = 0;
        }
    }

    /// The page's messages. A new call-type selection unghosts the send button, and a release on
    /// the toolbar's `AssistanceButton` closes an open message box (`gmAssistanceUI::ListenToMessage`;
    /// the button's registered message is taken as the release). A release on the send button runs
    /// `HandleSendButton`, whose complete call is returned (see [`AssistancePanel::send_answered`]).
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        message: &UiMessage,
        world: &str,
    ) -> Option<AssistanceRequest> {
        if message.message == MSG_BUTTON_RELEASE_ON
            && message.element_id == id("AssistanceButton")
            && self.dialog != 0
        {
            dialogs.close_dialog(tree, self.dialog);
        }
        if message.message == MSG_MENU_NEW_SELECTION
            && message.element.is_some()
            && message.element == self.menu
        {
            if let Some(send) = self.send {
                tree.set_bool(send, BUTTON_GHOSTED, false);
            }
            return None;
        }
        if message.message == MSG_BUTTON_RELEASE_ON
            && message.element.is_some()
            && message.element == self.send
        {
            return self.handle_send(tree, dialogs, world);
        }
        None
    }

    /// `gmAssistanceUI::HandleSendButton`: the selected call type's checks, in retail's order.
    /// A missing abuser name or description opens its message box and sends nothing. A call type
    /// of 0 (the placeholder) sends nothing and opens nothing.
    fn handle_send(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        world: &str,
    ) -> Option<AssistanceRequest> {
        let menu = self.menu?;
        let item = tree.menu_selected_item(menu)?;
        let call_type = match tree.get(item)?.attribute(call_type_attribute()) {
            Some(AttributeValue::Int(value)) => *value as u32,
            _ => return None,
        };
        if call_type == 0 {
            return None;
        }
        if self.dialog != 0 {
            dialogs.close_dialog(tree, self.dialog);
        }
        let abuser = self.entry_text(tree, self.abuser);
        let description = self.entry_text(tree, self.description);
        if call_type & NEEDS_ABUSER != 0 && abuser.is_empty() {
            self.open_message(tree, dialogs, id("ID_Assistance_Err_NoAbuserName"));
            return None;
        }
        if call_type & NEEDS_DESCRIPTION != 0 && description.is_empty() {
            self.open_message(tree, dialogs, id("ID_Assistance_Err_NoAbuseDesc"));
            return None;
        }
        Some(AssistanceRequest::CallCreate {
            call_type,
            world: world.to_owned(),
            target: abuser,
            description,
        })
    }

    /// The answer to a sent call: queued (`true`) resets the page; not queued opens the "can't
    /// connect" box (`HandleSendButton`'s 0 result from `RequestGUN_CallCreate`).
    pub fn send_answered(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, queued: bool) {
        if queued {
            self.reset(tree);
        } else {
            self.open_message(tree, dialogs, id("ID_Assistance_Err_CantConnect"));
        }
    }

    fn open_message(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, string: u32) {
        let text = ui_text(tree, self.table, string);
        self.dialog = dialogs.make_dialog(tree, &text, DIALOG_MESSAGE, false);
    }

    fn entry_text(&self, tree: &UiTree, entry: Option<usize>) -> String {
        entry
            .and_then(|entry| tree.get(entry))
            .and_then(|entry| entry.text())
            .unwrap_or_default()
    }
}

/// A UI string as text (`StringInfo` of the UI table, no variables).
fn ui_text(tree: &UiTree, table: u32, string: u32) -> String {
    tree.text_data
        .as_ref()
        .and_then(|data| data.string(table, string))
        .map(|units| String::from_utf16_lossy(&units))
        .unwrap_or_default()
}
