//! The automatic tools' settings window (a modern addition, `ac2_macro::automation`), opened by
//! the Tracker window's Tools button.
//!
//! It is built as the Tracker's is: another `Book` of the gameplay layout made as its own root,
//! its frame and title bar drawn in nine pieces. The text area holds both tools' sections; each
//! choice is a bracketed word a press changes (`[On]`, `[-]`, `[+]`), and the effects the player
//! has now are listed with `[+]` to keep each up, by the first skill on the shortcut bar
//! (`[<]` / `[>]` pick another). A press becomes an [`ac2_macro::automation::Change`] for the
//! client to apply and save.

use std::sync::Arc;

use ac2_macro::automation::{Change, Flag, Settings, Value};

use crate::book::BookUi;
use crate::book_rows::{line_height, margin_left, position_to_pixels, text_width};
use crate::compose::NINE_SLICE;
use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::{UiMessage, GLOBAL_MOUSE_DOWN, UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::scrollbar::MSG_NEW_POSITION;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};
use crate::window_parts::owned_by;

const WINDOW: (i32, i32) = (400, 470);
const FRAME_BORDER: i32 = 32;

/// What the window shows: the settings, the player's effects and the skills on the bar.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AutomationPresentation {
    pub settings: Settings,
    /// The effects the strip lists now (names).
    pub effects: Vec<String>,
    /// The skills on the shortcut bar: enum and name.
    pub skills: Vec<(u32, String)>,
}

#[derive(Debug, Clone)]
struct Piece {
    text: String,
    action: Option<Change>,
}

/// The settings window.
#[derive(Debug, Clone, Default)]
pub struct AutomationUi {
    book: Option<BookUi>,
    rows: Vec<Vec<Piece>>,
    scroll: i32,
}

fn plain(text: &str) -> Piece {
    Piece {
        text: text.to_owned(),
        action: None,
    }
}
fn press(text: &str, change: Change) -> Piece {
    Piece {
        text: text.to_owned(),
        action: Some(change),
    }
}
fn toggle(label: &str, on: bool, flag: Flag) -> Vec<Piece> {
    vec![
        plain(label),
        press(if on { "[On]" } else { "[Off]" }, Change::Toggle(flag)),
    ]
}
fn stepper(label: &str, shown: String, value: Value) -> Vec<Piece> {
    vec![
        plain(label),
        press("[-]", Change::Step(value, false)),
        plain(&format!(" {shown} ")),
        press("[+]", Change::Step(value, true)),
    ]
}

/// The window's rows for `p`.
fn rows(p: &AutomationPresentation) -> Vec<Vec<Piece>> {
    let b = &p.settings.butcher;
    let r = &p.settings.rebuff;
    let mut rows = vec![
        toggle("Auto-butcher  ", b.enabled, Flag::Butcher),
        vec![plain(
            "  Uses your butchering knife on each new corpse in reach.",
        )],
        toggle(
            "  Pause in combat  ",
            b.pause_in_combat,
            Flag::ButcherPauseInCombat,
        ),
        toggle(
            "  Only my kills  ",
            b.only_my_kills,
            Flag::ButcherOnlyMyKills,
        ),
        stepper(
            "  Reach  ",
            format!("{:.0} m", b.max_distance),
            Value::MaxDistance,
        ),
        stepper(
            "  Between uses  ",
            format!("{:.0} s", b.cooldown),
            Value::Cooldown,
        ),
        Vec::new(),
        toggle("Auto-rebuff  ", r.enabled, Flag::Rebuff),
        vec![plain(
            "  Presses a skill's shortcut slot to keep its effect up.",
        )],
        stepper(
            "  Recast before the end  ",
            format!("{:.0} s", r.lead),
            Value::Lead,
        ),
        toggle(
            "  Only out of combat  ",
            r.out_of_combat_only,
            Flag::OutOfCombat,
        ),
        stepper(
            "  Least vigor  ",
            format!("{:.0}%", r.min_vigor * 100.0),
            Value::MinVigor,
        ),
        toggle(
            "  Wait for the current action  ",
            r.wait_for_action,
            Flag::WaitForAction,
        ),
        Vec::new(),
        vec![plain("Kept up")],
    ];
    if r.entries.is_empty() {
        rows.push(vec![plain("  Nothing yet: press [+] by an effect below.")]);
    }
    let skill_name = |skill: u32| {
        p.skills.iter().find(|(s, _)| *s == skill).map_or_else(
            || format!("skill {skill:#x} (not on the bar)"),
            |(_, n)| n.clone(),
        )
    };
    let cycle = |skill: u32, by: isize| -> Option<u32> {
        if p.skills.is_empty() {
            return None;
        }
        let at = p.skills.iter().position(|(s, _)| *s == skill);
        let n = p.skills.len() as isize;
        let next = at.map_or(0, |i| (i as isize + by).rem_euclid(n));
        Some(p.skills[next as usize].0)
    };
    for (index, entry) in r.entries.iter().enumerate() {
        let mut row = vec![
            plain("  "),
            press("[x]", Change::Forget(index)),
            plain(" "),
            press(
                if entry.enabled { "[On]" } else { "[Off]" },
                Change::ToggleEntry(index),
            ),
            plain(&format!(" {} by ", entry.effect)),
        ];
        if let Some(skill) = cycle(entry.skill, -1) {
            row.push(press("[<]", Change::SetSkill { index, skill }));
        }
        row.push(plain(&format!(" {} ", skill_name(entry.skill))));
        if let Some(skill) = cycle(entry.skill, 1) {
            row.push(press("[>]", Change::SetSkill { index, skill }));
        }
        rows.push(row);
    }
    rows.push(Vec::new());
    rows.push(vec![plain("Your effects")]);
    let kept = |name: &str| r.entries.iter().any(|e| e.effect == name);
    let mut any = false;
    for effect in p.effects.iter().filter(|e| !kept(e)) {
        any = true;
        let mut row = vec![plain("  ")];
        match p.skills.first() {
            Some((skill, _)) => row.push(press(
                "[+]",
                Change::Maintain {
                    effect: effect.clone(),
                    skill: *skill,
                },
            )),
            None => row.push(plain("   ")),
        }
        row.push(plain(&format!(" {effect}")));
        rows.push(row);
    }
    if !any {
        rows.push(vec![plain("  None to add: cast a buff first.")]);
    }
    if p.skills.is_empty() {
        rows.push(vec![plain("  Put the skills to cast on the shortcut bar.")]);
    }
    rows
}

impl AutomationUi {
    /// The hidden window over the gameplay layout's `Book`.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>) -> Self {
        let mut ui = Self::default();
        let Some(book) = BookUi::create_window(tree, gameplay, "AutomationBook") else {
            return ui;
        };
        let root = book.field();
        let find = |tree: &UiTree, name: &str| tree.child_recursive(root, tokenize(name));
        tree.set_custom_wheel(root, true);
        tree.resize_to(root, WINDOW.0, WINDOW.1);
        let children: Vec<usize> = tree.children(root).to_vec();
        for child in children {
            let Some(element) = tree.get(child) else {
                continue;
            };
            if element.image().is_some() {
                let height = element.rect().height();
                tree.resize_to(child, WINDOW.0, height.min(WINDOW.1));
                if height > 100 {
                    tree.resize_to(child, WINDOW.0, WINDOW.1);
                }
                tree.set_int(child, NINE_SLICE, FRAME_BORDER);
            }
        }
        for name in [
            "BookForeground",
            "BookClientAreaTextHeader",
            "BookHelpButton",
            "BookCopyButton",
        ] {
            if let Some(e) = find(tree, name) {
                tree.set_visible(e, false);
            }
        }
        if let Some(close) = find(tree, "BookCloseButton") {
            tree.move_to(close, WINDOW.0 - 34, 7);
        }
        if let Some(title) = find(tree, "Panel_Title_Book") {
            tree.resize_to(title, WINDOW.0, 29);
            tree.set_text(title, "Automation");
        }
        if let Some(text) = find(tree, "BookClientAreaText") {
            tree.move_to(text, 22, 40);
            tree.resize_to(text, WINDOW.0 - 44, WINDOW.1 - 40 - 40);
        }
        if let Some(bottom) = find(tree, "BookBottom") {
            tree.move_to(bottom, 0, WINDOW.1 - 32);
            tree.resize_to(bottom, WINDOW.0, 30);
            tree.set_int(bottom, NINE_SLICE, FRAME_BORDER);
        }
        ui.book = Some(book);
        ui
    }

    pub fn window(&self) -> Option<usize> {
        self.book.as_ref().map(BookUi::field)
    }

    pub fn is_shown(&self) -> bool {
        self.book.as_ref().is_some_and(BookUi::is_shown)
    }

    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        if let Some(book) = &mut self.book {
            book.show(tree, shown);
        }
    }

    /// Lays the settings out in the text area.
    pub fn present(&mut self, tree: &mut UiTree, presentation: &AutomationPresentation) {
        self.rows = rows(presentation);
        if let Some(text) = self.text(tree) {
            let lines: Vec<String> = self
                .rows
                .iter()
                .map(|row| row.iter().map(|p| p.text.as_str()).collect())
                .collect();
            tree.set_text(text, &lines.join("\n"));
        }
        self.sync_scroll(tree);
    }

    /// The window's messages: the close button hides it, a press on a bracketed word changes a
    /// setting, the wheel and the slider scroll. Returns whether the message was the window's.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) -> (bool, Option<Change>) {
        if !self.is_shown() {
            return (false, None);
        }
        if self.wheel(tree, message) {
            return (true, None);
        }
        let Some(root) = self.window() else {
            return (false, None);
        };
        if !owned_by(tree, message.element, root) {
            return (false, None);
        }
        if message.element_id == tokenize("BookCloseButton")
            && message.message == MSG_BUTTON_RELEASE_ON
        {
            self.show(tree, false);
            return (true, None);
        }
        if message.element_id == tokenize("BookSliderBar") && message.message == MSG_NEW_POSITION {
            if let Some(slider) = tree.child_recursive(root, tokenize("BookSliderBar")) {
                let position = tree.scrollbar_position(slider);
                self.scroll = position_to_pixels(position, self.max_scroll(tree));
                self.sync_scroll(tree);
            }
            return (true, None);
        }
        if message.element_id == tokenize("BookClientAreaText")
            && message.message == MSG_MOUSE_PRESS_TOP
        {
            return (true, self.action_at(tree, tree.pointer()));
        }
        (false, None)
    }

    fn text(&self, tree: &UiTree) -> Option<usize> {
        tree.child_recursive(self.window()?, tokenize("BookClientAreaText"))
    }

    fn action_at(&self, tree: &UiTree, (x, y): (i32, i32)) -> Option<Change> {
        let text = self.text(tree)?;
        let rect = tree.screen_box(text);
        if x < rect.x0 || x >= rect.x1 || y < rect.y0 || y >= rect.y1 {
            return None;
        }
        let row = ((y - rect.y0 + self.scroll) / line_height(tree, text)) as usize;
        let mut left = x - rect.x0 - margin_left(tree, text);
        for piece in self.rows.get(row)? {
            let width = text_width(tree, text, &piece.text);
            if left < width {
                return piece.action.clone();
            }
            left -= width;
        }
        None
    }

    fn wheel(&mut self, tree: &mut UiTree, message: &UiMessage) -> bool {
        let action = message.data as u32;
        if message.element.is_some()
            || message.message != GLOBAL_MOUSE_DOWN
            || !matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT)
        {
            return false;
        }
        let Some(text) = self.text(tree) else {
            return false;
        };
        let rect = tree.screen_box(text);
        let (x, y) = tree.pointer();
        if x < rect.x0 || x >= rect.x1 || y < rect.y0 || y >= rect.y1 {
            return false;
        }
        let line = line_height(tree, text);
        self.scroll += if action == UI_WHEEL_IN { -line } else { line };
        self.sync_scroll(tree);
        true
    }

    fn max_scroll(&self, tree: &UiTree) -> i32 {
        let Some(text) = self.text(tree) else {
            return 0;
        };
        let content = self.rows.len() as i32 * line_height(tree, text);
        let view = tree.get(text).map_or(0, |e| e.rect().height());
        (content - view).max(0)
    }

    fn sync_scroll(&mut self, tree: &mut UiTree) {
        let Some(root) = self.window() else {
            return;
        };
        let max = self.max_scroll(tree);
        self.scroll = self.scroll.clamp(0, max);
        if let Some(text) = self.text(tree) {
            tree.set_text_view_y(text, self.scroll);
        }
        if let Some(bottom) = tree.child_recursive(root, tokenize("BookBottom")) {
            tree.set_visible(bottom, max > 0);
        }
        if let Some(slider) = tree.child_recursive(root, tokenize("BookSliderBar")) {
            let position = if max > 0 {
                self.scroll as f32 / max as f32
            } else {
                0.0
            };
            tree.scrollbar_set_position(slider, position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_macro::automation::RebuffEntry;

    fn actions(rows: &[Vec<Piece>]) -> Vec<Change> {
        rows.iter()
            .flatten()
            .filter_map(|p| p.action.clone())
            .collect()
    }

    #[test]
    fn the_rows_offer_every_choice_and_the_effects_to_keep_up() {
        let mut p = AutomationPresentation {
            effects: vec!["Defensive Stance".into(), "Haste".into()],
            skills: vec![(0xA8, "Defensive Stance".into()), (0xB0, "Haste".into())],
            ..AutomationPresentation::default()
        };
        p.settings.rebuff.entries.push(RebuffEntry {
            effect: "Haste".into(),
            skill: 0xA8,
            enabled: true,
        });
        let all = actions(&rows(&p));
        for flag in [
            Flag::Butcher,
            Flag::ButcherPauseInCombat,
            Flag::ButcherOnlyMyKills,
            Flag::Rebuff,
            Flag::OutOfCombat,
            Flag::WaitForAction,
        ] {
            assert!(all.contains(&Change::Toggle(flag)), "{flag:?}");
        }
        for value in [
            Value::MaxDistance,
            Value::Cooldown,
            Value::Lead,
            Value::MinVigor,
        ] {
            assert!(
                all.contains(&Change::Step(value, true))
                    && all.contains(&Change::Step(value, false))
            );
        }
        // The kept entry cycles its skill both ways; only the effect not yet kept can be added.
        assert!(all.contains(&Change::SetSkill {
            index: 0,
            skill: 0xB0
        }));
        assert!(all.contains(&Change::Forget(0)) && all.contains(&Change::ToggleEntry(0)));
        let added: Vec<_> = all
            .iter()
            .filter(|c| matches!(c, Change::Maintain { .. }))
            .collect();
        assert_eq!(
            added,
            vec![&Change::Maintain {
                effect: "Defensive Stance".into(),
                skill: 0xA8,
            }]
        );
        // With no skill on the bar nothing can be added, and the window says why.
        p.skills.clear();
        let none = rows(&p);
        assert!(!actions(&none)
            .iter()
            .any(|c| matches!(c, Change::Maintain { .. })));
        assert!(none
            .iter()
            .flatten()
            .any(|piece| piece.text.contains("shortcut bar")));
    }
}
