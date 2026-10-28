//! `gmExaminationUI`'s filter tabs (Legions 7160; admin 7148 has none). The panel keeps the
//! request's filter word (`+0xB8`, 0x203 at construction) and four tab elements, each owning
//! the mask of the profile sections it shows: `GeneralTab` 0x203, `DetailsTab` 0x18C,
//! `EffectsTab` 0x10 and `UsageTab` 0x60 (Legions 0x004316A0 registers them for `mouse_press`).
//!
//! A tab's state follows the filter and the profile (Legions 0x00431170): `active` when its mask
//! is in the filter, else `inactive` when the profile has content for it (the word the profile
//! packs after its nodes), else `disabled`. A press on a tab that is not disabled sets the
//! filter (a Ctrl press toggles the tab's bits instead, Legions 0x00431550) and asks the
//! player to start the examination again with the new word (`gmCWInterface::StartExamination`).
//! The selection keys `SELECTION_NEXT_EXAM_TAB_KEY` (0x4100006A) and `SELECTION_PREV_EXAM_TAB_KEY`
//! (0x4100006B) step to the next or previous tab that is not disabled (Legions 0x00431350 and
//! 0x00431430) and do the same.

use crate::element::UiTree;
use crate::token::tokenize;

/// The filter word at construction (`gmExaminationUI` `+0xB8`): the General tab's mask.
pub const DEFAULT_FILTER: u32 = 0x203;
/// `SELECTION_NEXT_EXAM_TAB_KEY`.
pub const NEXT_TAB_KEY: u32 = 0x4100_006A;
/// `SELECTION_PREV_EXAM_TAB_KEY`.
pub const PREV_TAB_KEY: u32 = 0x4100_006B;

const ACTIVE: u32 = tokenize("active");
const INACTIVE: u32 = tokenize("inactive");
const DISABLED: u32 = tokenize("disabled");

/// The tabs in the order Legions registers them, with their masks.
const TABS: [(&str, u32); 4] = [
    ("GeneralTab", 0x203),
    ("DetailsTab", 0x18C),
    ("EffectsTab", 0x10),
    ("UsageTab", 0x60),
];

#[derive(Debug, Clone)]
struct Tab {
    id: u32,
    element: usize,
    mask: u32,
}

/// The tabs and the filter word.
#[derive(Debug, Clone)]
pub(crate) struct ExaminationTabs {
    tabs: Vec<Tab>,
    /// `+0xA0`: the id of the tab last chosen (the last one whose mask meets the filter).
    current: u32,
    /// `+0xB8`.
    filter: u32,
}

impl ExaminationTabs {
    /// The tabs found under the panel's field (Legions 0x004316A0), in the construction state.
    pub fn new(tree: &UiTree, field: Option<usize>) -> Self {
        let tabs = field
            .map(|field| {
                TABS.iter()
                    .filter_map(|&(name, mask)| {
                        let id = tokenize(name);
                        let element = tree.child_recursive(field, id)?;
                        Some(Tab { id, element, mask })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            tabs,
            current: tokenize(TABS[0].0),
            filter: DEFAULT_FILTER,
        }
    }

    pub fn filter(&self) -> u32 {
        self.filter
    }

    pub fn set_filter(&mut self, filter: u32) {
        self.filter = filter;
    }

    /// Legions 0x00431170: each tab is `active` when `selected` has its mask, else `inactive`
    /// when `available` has it, else `disabled`.
    pub fn set_states(&self, tree: &mut UiTree, selected: u32, available: u32) {
        for tab in &self.tabs {
            let state = if selected & tab.mask != 0 {
                ACTIVE
            } else if available & tab.mask != 0 {
                INACTIVE
            } else {
                DISABLED
            };
            if tree.get(tab.element).map(|e| e.state()) != Some(state) {
                tree.set_state(tab.element, state);
            }
        }
    }

    /// Legions 0x00431550: the tab's mask becomes the filter, or with `ctrl` toggles its bits in
    /// it; the current tab is the last whose mask meets `mask`.
    fn apply(&mut self, mask: u32, ctrl: bool) {
        if !ctrl {
            self.filter = mask;
        } else if mask & self.filter == 0 {
            self.filter |= mask;
        } else {
            self.filter &= !mask;
        }
        self.current = tokenize(TABS[0].0);
        for tab in &self.tabs {
            if tab.mask & mask != 0 {
                self.current = tab.id;
            }
        }
    }

    fn disabled(&self, tree: &UiTree, tab: &Tab) -> bool {
        tree.get(tab.element).map(|e| e.state()) == Some(DISABLED)
    }

    /// A press on `element_id` (Legions 0x00431990): when it is a tab that is not disabled,
    /// the filter it chooses.
    pub fn press(&mut self, tree: &UiTree, element_id: u32, ctrl: bool) -> Option<u32> {
        let tab = self.tabs.iter().find(|t| t.id == element_id)?;
        if self.disabled(tree, tab) {
            return None;
        }
        let mask = tab.mask;
        self.apply(mask, ctrl);
        Some(self.filter)
    }

    /// The selection keys (Legions 0x00431900): the next or previous tab that is not disabled,
    /// cycling; the current tab again when no other is enabled.
    pub fn step(&mut self, tree: &UiTree, forward: bool, ctrl: bool) -> Option<u32> {
        let count = self.tabs.len();
        let start = self
            .tabs
            .iter()
            .position(|t| t.id == self.current)
            .unwrap_or(0);
        let pick = (1..=count)
            .map(|n| {
                if forward {
                    (start + n) % count
                } else {
                    (start + count - n % count) % count
                }
            })
            .find(|&i| i == start || !self.disabled(tree, &self.tabs[i]))?;
        let mask = self.tabs[pick].mask;
        self.apply(mask, ctrl);
        Some(self.filter)
    }
}
