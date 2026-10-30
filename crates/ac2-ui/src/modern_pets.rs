//! The modern style's pet panel (AGENTS.md: the HUD's two styles; the design's "Pets", expanded
//! and compact, the user's choice). Retail's `PetPanel` put its title and the four command
//! buttons (`AttackButton`, `ReturnButton`, `HealButton`, `BuffButton`) in a framed block over the
//! framed list of pets, at one size. Here the title and a fold chevron head the panel, the pets'
//! rows follow in their well, and the four buttons stand under them; the chevron folds the well to
//! the rows the panel lists (compact) or opens it to retail's height (expanded). The elements are
//! retail's, so the rows, their mode arrows, the buttons and their requests stay retail's; the
//! choice is kept in the HUD layout (`pets.compact`).

use std::cell::Cell;

use crate::element::{UiBox, UiTree};
use crate::modern_art as art;
use crate::modern_inventory::{keep, make_part};
use crate::mouse::UiMessage;
use crate::token::{MOUSE_INVISIBLE, MSG_MOUSE_PRESS_TOP};

/// `PetPanel`'s unnamed parts: the command block and its frame, the list's frame, the list, the
/// help text, the title and the backing (a plain field, the parts' template).
const COMMANDS: u32 = 0x9738_95B0;
const COMMANDS_FRAME: u32 = 0x5C5A_B253;
const LIST_FRAME: u32 = 0x6E26_8A53;
const LIST: u32 = 0xC1BA_01CE;
const HELP: u32 = 0x9638_E79B;
const TITLE: u32 = 0x7B91_89D8;
const BACKING: u32 = 0xE930_EB61;

/// The header's height, the commands' row height, a row's height in the list, the gaps, and the
/// panel's retail height (the expanded panel's).
const HEAD: i32 = 24;
const COMMANDS_HEIGHT: i32 = 44;
const ROW: i32 = 20;
const GAP: i32 = 4;
const EXPANDED: i32 = 262;

/// The pet panel's design parts and its mode.
#[derive(Debug, Clone)]
pub(crate) struct ModernPets {
    panel: usize,
    fold: Option<usize>,
    compact: Cell<bool>,
    /// The mode and row count last laid out.
    laid_out: Cell<Option<(bool, usize)>>,
}

fn put(tree: &mut UiTree, element: Option<usize>, (x, y, w, h): (i32, i32, i32, i32)) {
    let Some(element) = element else {
        return;
    };
    let at = UiBox::new(x, y, x + w - 1, y + h - 1);
    if tree.el(element).rect != at {
        tree.resize_to(element, w, h);
        tree.move_to(element, x, y);
    }
}

impl ModernPets {
    /// The panel's fold chevron made; expanded. Its other parts are laid out by [`Self::tick`].
    pub(crate) fn adopt(tree: &mut UiTree, panel: usize) -> Self {
        let find = |tree: &UiTree, id: u32| tree.child_recursive(panel, id);
        let template = find(tree, BACKING);
        let fold = make_part(
            tree,
            panel,
            template,
            "ModernPetsFold",
            art::WIN_ARROW[4],
            Some(0),
            0,
        );
        if let Some(fold) = fold {
            keep(tree, fold);
            tree.set_bool(fold, MOUSE_INVISIBLE, false);
        }
        Self {
            panel,
            fold,
            compact: Cell::new(false),
            laid_out: Cell::new(None),
        }
    }

    pub(crate) fn is_compact(&self) -> bool {
        self.compact.get()
    }

    pub(crate) fn set_compact(&self, compact: bool) {
        self.compact.set(compact);
    }

    /// The panel laid out for its mode and `rows` pets (when either changed; a pet's coming or
    /// going is the panel's state change).
    pub(crate) fn tick(&self, tree: &mut UiTree, rows: usize) {
        let compact = self.compact.get();
        if self.laid_out.get() == Some((compact, rows)) {
            return;
        }
        self.laid_out.set(Some((compact, rows)));
        let find = |tree: &UiTree, id: u32| tree.child_recursive(self.panel, id);
        // The panel's state change (`nopets`, `pets`) makes its parts anew: each is kept where
        // it is put again, the title moved up to the header and the commands' frame hidden.
        for id in [COMMANDS, COMMANDS_FRAME, LIST_FRAME, LIST, HELP, TITLE] {
            if let Some(element) = find(tree, id) {
                keep(tree, element);
            }
        }
        if let Some(frame) = find(tree, COMMANDS_FRAME) {
            tree.set_visible(frame, false);
        }
        if let Some(title) = find(tree, TITLE) {
            if tree.el(title).parent != Some(self.panel) {
                tree.set_parent(title, Some(self.panel));
            }
        }
        let w = tree.el(self.panel).rect.width();
        let well = if compact {
            rows.max(1) as i32 * ROW + 2 * GAP
        } else {
            EXPANDED - HEAD - 2 * GAP - COMMANDS_HEIGHT
        };
        put(tree, find(tree, TITLE), (8, 5, w - 40, 16));
        put(tree, self.fold, (w - 24, 4, 16, 16));
        if let Some(fold) = self.fold {
            // Up folds the panel to its rows; down opens it.
            let image = art::WIN_ARROW[if compact { 6 } else { 4 }];
            tree.set_image(fold, image, crate::compose::DRAW_OVER);
        }
        put(tree, find(tree, LIST_FRAME), (2, HEAD, w - 4, well));
        put(
            tree,
            find(tree, LIST),
            (6, HEAD + GAP, w - 12, well - 2 * GAP),
        );
        put(
            tree,
            find(tree, HELP),
            (10, HEAD + GAP, w - 20, well - 2 * GAP),
        );
        let commands_y = HEAD + well + GAP;
        if let Some(commands) = find(tree, COMMANDS) {
            put(tree, Some(commands), (0, commands_y, w, COMMANDS_HEIGHT));
            for button in tree.children(commands).to_vec() {
                let r = tree.el(button).rect;
                if r.width() == 32 && r.y0 != 6 {
                    tree.move_to(button, r.x0, 6);
                }
            }
        }
        tree.resize_to(self.panel, w, commands_y + COMMANDS_HEIGHT + GAP);
    }

    /// A press on the fold chevron changes the mode. Whether it took the press.
    pub(crate) fn message(&self, message: &UiMessage) -> bool {
        if message.message != MSG_MOUSE_PRESS_TOP
            || message.element != self.fold
            || self.fold.is_none()
        {
            return false;
        }
        self.compact.set(!self.compact.get());
        true
    }
}
