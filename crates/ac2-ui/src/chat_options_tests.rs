//! `gmChatOptionsUI` on the real toolbar layout (`AC2_DAT_DIR`): the expanders in table order,
//! the refresh from the chat settings, the changed state, Apply, Recall and Default.

use std::collections::HashMap;

use ac2_db::ResourceStore;

use crate::chat::ChatSettings;
use crate::chat_options_table::ROWS;
use crate::dialog::DialogOutcome;
use crate::element::UiTree;
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::{
    build_gameplay_hud, ChatOptionRequest, ChatOptionSetting, ChatOptionsUi, DialogFactory,
    GameplayHud,
};

const SAY: &str = "ID_Option_ChatSayMessages";
const TELL: &str = "ID_Option_ChatTellMessages";
const LOG: &str = "ID_Option_ChatLogMessages";
const ALLEGIANCE: &str = "ID_Option_ChatAllegianceMessages";
const ENTRY: &str = "ID_Option_ChatEntryMessages";
const DEVOTED: &str = "ID_Option_DevotedChatWindows";
const FONT_SIZE: &str = "ID_Option_ChatFontSize";
const FONT_COLOR: &str = "ID_Option_ChatFontColor";
const POPUP: &str = "ID_Option_ChatPopup";
const WINDOW_1: &str = "ID_Option_Chat1";
const WINDOW_2: &str = "ID_Option_Chat2";
const TEXT_TYPE_SAY: u32 = 0x10;
const TEXT_TYPE_TELL: u32 = 0x20;
const TEXT_TYPE_LOG: u32 = 0x80;
const TEXT_TYPE_ALLEGIANCE: u32 = 0x2_0000;

/// The player's settings: window 1 shows say, tell and log, window 2 standard and broadcast,
/// window 4 fellowship; say is large and colour 6, tell has its popup, window 2 is allegiance.
fn settings() -> ChatSettings {
    let mut s = ChatSettings {
        filters: HashMap::from([
            (1, TEXT_TYPE_SAY | TEXT_TYPE_TELL | TEXT_TYPE_LOG),
            (2, 0x8 | 0x100),
            (3, 0),
            (4, 0x1_0000),
        ]),
        ..ChatSettings::default()
    };
    s.font_sizes.insert(TEXT_TYPE_SAY, 2);
    s.font_colors.insert(TEXT_TYPE_SAY, 5);
    s.popups.insert(TEXT_TYPE_TELL, true);
    s.window_text_types.insert(2, TEXT_TYPE_ALLEGIANCE);
    s
}

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

fn message(element_id: &str, element: Option<usize>, message: u32) -> UiMessage {
    UiMessage {
        element_id: tokenize(element_id),
        element,
        message,
        data: 0,
        item: None,
    }
}

/// The user presses the page's button `element_id`.
fn press(
    tree: &mut UiTree,
    page: &mut ChatOptionsUi,
    dialogs: &mut DialogFactory,
    element_id: &str,
) {
    let element = tree.child_recursive(page.page(), tokenize(element_id));
    page.message(
        tree,
        dialogs,
        &message(element_id, element, MSG_BUTTON_RELEASE_ON),
    );
}

/// The tab opens (the panel's `opened` message).
fn open(tree: &mut UiTree, page: &mut ChatOptionsUi, dialogs: &mut DialogFactory) {
    page.message(
        tree,
        dialogs,
        &UiMessage {
            element_id: tokenize("ChatTab"),
            element: None,
            message: MSG_OPENED,
            data: 0,
            item: None,
        },
    );
}

/// The user answers the shown confirmation; the outcomes go to the page.
fn answer(tree: &mut UiTree, page: &mut ChatOptionsUi, dialogs: &mut DialogFactory, yes: bool) {
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("ConfirmationField"))
        })
        .expect("the confirmation dialog");
    let name = if yes { "YesButton" } else { "NoButton" };
    let button = tree.child_recursive(root, tokenize(name));
    dialogs.message(tree, &message(name, button, MSG_BUTTON_RELEASE_ON));
    let outcomes: Vec<DialogOutcome> = dialogs.take_outcomes();
    page.dialog_outcomes(tree, dialogs, &outcomes);
}

/// The window filter request of window `window`.
fn filter_of(requests: &[ChatOptionRequest], window: u32) -> Option<u32> {
    requests.iter().find_map(|r| match r {
        ChatOptionRequest::Set(ChatOptionSetting::WindowFilter { window: w, filter })
            if *w == window =>
        {
            Some(*filter)
        }
        _ => None,
    })
}

fn expanders() -> Vec<&'static str> {
    ROWS.iter().filter(|r| !r.hidden).map(|r| r.name).collect()
}

#[test]
fn the_page_lists_the_expanders_in_table_order_with_their_options() {
    let Some(mut hud) = hud() else {
        return;
    };
    let page = hud.chat_options.take().expect("ChatPage");
    // The top level: the visible text types, the admin one hidden.
    assert_eq!(page.listed(&hud.tree), expanders());
    assert_eq!(page.listed(&hud.tree).len(), 16);
    assert!(!page
        .listed(&hud.tree)
        .contains(&"ID_Option_ChatAdminMessages"));
    // Each expander is listed and closed; its options are not.
    for title in expanders() {
        assert_eq!(page.is_open(title), Some(false), "{title}");
        assert_eq!(page.rows(&hud.tree, title), Some(0), "{title}");
    }
    // The order: an expander, then its options in `PopulateExpander`'s order (font size, colour,
    // the windows' flags, the popup flag); the entry line has only the first two.
    let order = page.order();
    let start = order
        .iter()
        .position(|&n| n == "ID_Option_ChatErrorMessages")
        .unwrap();
    assert_eq!(
        order[start..start + 9],
        [
            "ID_Option_ChatErrorMessages",
            FONT_SIZE,
            FONT_COLOR,
            WINDOW_1,
            WINDOW_2,
            "ID_Option_Chat3",
            "ID_Option_Chat4",
            POPUP,
            "ID_Option_ChatCombatMessages",
        ]
    );
    let entry = order.iter().rposition(|&n| n == ENTRY).unwrap();
    assert_eq!(order[entry..], [ENTRY, FONT_SIZE, FONT_COLOR]);
    let devoted = order.iter().position(|&n| n == DEVOTED).unwrap();
    assert_eq!(
        order[devoted + 1..devoted + 5],
        [WINDOW_1, WINDOW_2, "ID_Option_Chat3", "ID_Option_Chat4"]
    );
    assert_eq!(order.len(), 16 + 14 * 7 + 2 + 4);
    // Every option has a label from the string table.
    for title in expanders() {
        let element = page.element(title, FONT_SIZE);
        if title == DEVOTED {
            continue;
        }
        let label = hud
            .tree
            .child_recursive(element.expect(title), tokenize("OptionText"))
            .and_then(|t| hud.tree.get(t)?.text());
        assert!(label.is_some_and(|l| !l.is_empty()), "{title}: no label");
    }
}

#[test]
fn refresh_from_the_settings_sets_each_control() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.chat_options.take().expect("ChatPage");
    let tree = &mut hud.tree;
    page.refresh(tree, &settings());
    // Say: large (the third choice of four is 2, the fourth is large: choice 4), colour 6, and
    // shown in window 1 only.
    assert_eq!(page.value(SAY, FONT_SIZE), Some(4.0));
    assert_eq!(page.value(SAY, FONT_COLOR), Some(6.0));
    assert_eq!(page.value(SAY, WINDOW_1), Some(1.0));
    assert_eq!(page.value(SAY, WINDOW_2), Some(0.0));
    assert_eq!(page.value(SAY, POPUP), Some(0.0));
    // Tell: its popup, and window 1's flag.
    assert_eq!(page.value(TELL, POPUP), Some(1.0));
    assert_eq!(page.value(TELL, WINDOW_1), Some(1.0));
    assert_eq!(page.value(LOG, WINDOW_1), Some(1.0));
    // The windows' text types: window 2 is allegiance (its row's choice), the others the default
    // say (row 5).
    assert_eq!(page.value(DEVOTED, WINDOW_2), Some(10.0));
    assert_eq!(page.value(DEVOTED, WINDOW_1), Some(5.0));
    assert_eq!(page.value(ALLEGIANCE, WINDOW_2), Some(0.0));
    // Nothing differs from the settings.
    assert!(!page.changed());
    assert!(page.ghosted(tree, page.apply_button()));
    assert!(page.ghosted(tree, page.recall_button()));
}

#[test]
fn a_change_marks_changed_and_unghosts_apply_and_recall() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.chat_options.take().expect("ChatPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &settings());
    // A filter, a colour and a size each differ from the saved value.
    for (title, name, value) in [
        (SAY, WINDOW_1, 0.0),
        (SAY, FONT_COLOR, 1.0),
        (SAY, FONT_SIZE, 3.0),
    ] {
        assert!(page.set_value(tree, title, name, value), "{name}");
        assert!(page.changed(), "{name}");
        assert!(!page.ghosted(tree, page.apply_button()));
        assert!(!page.ghosted(tree, page.recall_button()));
        // Put it back: nothing differs again.
        let saved = match name {
            FONT_SIZE => 4.0,
            FONT_COLOR => 6.0,
            _ => 1.0,
        };
        assert!(page.set_value(tree, title, name, saved));
        assert!(!page.changed(), "{name} back");
    }
    // A choice outside the drop-down is refused.
    assert!(!page.set_value(tree, SAY, FONT_SIZE, 5.0));
    assert!(!page.set_value(tree, SAY, "ID_Option_NoSuchOption", 1.0));
    // The page reads its buttons after the tab opens.
    open(tree, &mut page, &mut dialogs);
    assert!(page.page_open());
}

#[test]
fn apply_asks_then_sends_every_setter_the_filters_the_reload_and_the_upload() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.chat_options.take().expect("ChatPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &settings());
    open(tree, &mut page, &mut dialogs);
    assert!(page.set_value(tree, SAY, WINDOW_1, 0.0));
    // Apply asks first: a no keeps the values and sends nothing.
    press(tree, &mut page, &mut dialogs, "ApplyButton");
    answer(tree, &mut page, &mut dialogs, false);
    assert!(page.take_requests().is_empty());
    assert!(page.changed());
    // A yes applies.
    press(tree, &mut page, &mut dialogs, "ApplyButton");
    answer(tree, &mut page, &mut dialogs, true);
    let requests = page.take_requests();
    // 15 font sizes, 15 colours, 14 popups and the 4 windows' text types, then the 4 window
    // filters and the upload.
    assert_eq!(requests.len(), 15 + 15 + 14 + 4 + 4 + 1);
    assert_eq!(
        requests[0],
        ChatOptionRequest::Set(ChatOptionSetting::FontSize {
            text_type: ROWS[0].text_type,
            size: 0,
        })
    );
    assert!(
        requests.contains(&ChatOptionRequest::Set(ChatOptionSetting::FontSize {
            text_type: TEXT_TYPE_SAY,
            size: 2,
        }))
    );
    assert!(
        requests.contains(&ChatOptionRequest::Set(ChatOptionSetting::FontColor {
            text_type: TEXT_TYPE_SAY,
            color: 5,
        }))
    );
    assert!(
        requests.contains(&ChatOptionRequest::Set(ChatOptionSetting::PopupFlag {
            text_type: TEXT_TYPE_TELL,
            flag: 1,
        }))
    );
    assert!(
        requests.contains(&ChatOptionRequest::Set(ChatOptionSetting::WindowTextType {
            window: 2,
            text_type: TEXT_TYPE_ALLEGIANCE,
        }))
    );
    // The window filters: window 1 loses say and keeps tell, log and the admin/log bits it had.
    // Each window keeps only its page's bits (and 0x84), so the entry bit goes.
    assert_eq!(
        filter_of(&requests, 1),
        Some(TEXT_TYPE_TELL | TEXT_TYPE_LOG)
    );
    assert_eq!(filter_of(&requests, 2), Some(0x8 | 0x100));
    assert_eq!(filter_of(&requests, 3), Some(0));
    assert_eq!(filter_of(&requests, 4), Some(0x1_0000));
    for (i, window) in (1..=4).enumerate() {
        assert!(matches!(
            requests[requests.len() - 5 + i],
            ChatOptionRequest::Set(ChatOptionSetting::WindowFilter { window: w, .. }) if w == window
        ));
    }
    assert_eq!(requests.last(), Some(&ChatOptionRequest::Upload));
    // The saved values follow, and the windows reload once.
    assert!(!page.changed());
    assert!(page.take_reload());
    assert!(!page.take_reload());
}

#[test]
fn recall_asks_then_restores_the_saved_values() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.chat_options.take().expect("ChatPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &settings());
    open(tree, &mut page, &mut dialogs);
    assert!(page.set_value(tree, LOG, WINDOW_1, 0.0));
    press(tree, &mut page, &mut dialogs, "RecallButton");
    answer(tree, &mut page, &mut dialogs, true);
    assert_eq!(page.value(LOG, WINDOW_1), Some(1.0));
    assert!(!page.changed());
    // Recall sends nothing: the profile keeps what it has.
    assert!(page.take_requests().is_empty());
}

#[test]
fn default_asks_then_applies_the_defaults() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.chat_options.take().expect("ChatPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &settings());
    open(tree, &mut page, &mut dialogs);
    assert!(page.set_value(tree, SAY, FONT_COLOR, 2.0));
    press(tree, &mut page, &mut dialogs, "DefaultButton");
    answer(tree, &mut page, &mut dialogs, true);
    // The defaults are the table's: the say colour is its own (choice 5), the window 1 text type
    // is say, the filters are clear, and the popup is broadcast's only.
    assert_eq!(page.value(SAY, FONT_COLOR), Some(5.0));
    assert_eq!(page.value(SAY, WINDOW_1), Some(0.0));
    assert_eq!(page.value(DEVOTED, WINDOW_2), Some(5.0));
    assert_eq!(page.value(TELL, POPUP), Some(0.0));
    assert!(!page.changed());
    let requests = page.take_requests();
    assert_eq!(requests.len(), 53);
    for window in 1..=4 {
        assert_eq!(filter_of(&requests, window), Some(0), "window {window}");
    }
    // The defaults' font size is the size as the choice's index (retail's `SetDefault` of the
    // size): the first choice, the smaller font.
    assert!(
        requests.contains(&ChatOptionRequest::Set(ChatOptionSetting::FontSize {
            text_type: TEXT_TYPE_SAY,
            size: 3,
        }))
    );
}

#[test]
fn an_unopened_page_keeps_its_buttons_as_they_were() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.chat_options.take().expect("ChatPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &settings());
    assert!(page.set_value(tree, SAY, WINDOW_1, 0.0));
    // With the tab shut the buttons do not answer.
    press(tree, &mut page, &mut dialogs, "ApplyButton");
    assert!(page.take_requests().is_empty());
    assert!(!page.page_open());
}
