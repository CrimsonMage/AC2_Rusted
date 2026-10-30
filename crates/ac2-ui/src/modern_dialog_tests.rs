//! The modern dialogs' header (`modern_dialog.rs`) on the real gameplay layout (`AC2_DAT_DIR`):
//! each dialog has a band and its gold title above its body text, and the classic style has none.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::token::tokenize;
use crate::{
    build_styled_hud, HudStyle, DIALOG_CONFIRMATION, DIALOG_MESSAGE, DIALOG_RESURRECT, DIALOG_WAIT,
};

const FIELDS: [(u32, &str, &str, &str); 4] = [
    (
        DIALOG_CONFIRMATION,
        "ConfirmationField",
        "QuestionText",
        "CONFIRM",
    ),
    (DIALOG_MESSAGE, "MessageField", "TextField", "MESSAGE"),
    (DIALOG_WAIT, "WaitField", "TextField", "PLEASE WAIT"),
    (
        DIALOG_RESURRECT,
        "ResurrectField",
        "TextField",
        "RESURRECTION",
    ),
];

#[test]
fn each_modern_dialog_has_a_header_and_gold_title_above_its_text_and_classic_has_none() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    for style in [HudStyle::Modern, HudStyle::Classic] {
        for (kind, field, text, title) in FIELDS {
            let mut hud = build_styled_hud(&store, 1365, 768, style).unwrap();
            hud.dialogs
                .make_dialog(&mut hud.tree, "a question", kind, false);
            let tree = &hud.tree;
            let root = tree
                .roots()
                .iter()
                .copied()
                .find(|&r| tree.el(r).id() == tokenize(field))
                .unwrap_or_else(|| panic!("{field}"));
            let head = tree.child_recursive(root, tokenize("ModernDialogHead"));
            let label = tree.child_recursive(root, tokenize("ModernDialogTitle"));
            if style == HudStyle::Classic {
                assert!(head.is_none() && label.is_none(), "{field} classic");
                continue;
            }
            let (head, label) = (head.expect("band"), label.expect("title"));
            let body = tree.child_recursive(root, tokenize(text)).unwrap();
            assert!(
                tree.screen_box(head).y1 <= tree.screen_box(body).y0 + 1,
                "{field}: the band is above the text"
            );
            let words = tree
                .children(label)
                .iter()
                .find_map(|&c| tree.el(c).text.as_ref())
                .map(|t| {
                    assert!((t.color[0] - 217.0 / 255.0).abs() < 1e-3, "gold");
                    t.glyphs
                        .iter()
                        .map(|g| char::from_u32(g.ch as u32).unwrap())
                        .collect::<String>()
                })
                .unwrap();
            // Set in small caps (`text_title.rs`): each word's first letter capital, the rest the
            // small-caps face's lower case.
            assert_eq!(words.to_uppercase(), title);
            assert!(
                words.chars().nth(1).is_some_and(char::is_lowercase),
                "{words}"
            );
        }
    }
}
