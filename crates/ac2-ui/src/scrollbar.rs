//! `UIElement_Scrollbar`: a track with a `widget` child placed along it by `x_position`
//! (`horizontal`) or `y_position` in [0, 1] (`UpdateLayout`; the widget keeps its own size unless
//! `proportional`). A press on the widget drags it (`MouseDown`, `Global_MouseMove`, `MouseUp`);
//! with `move_to_touched_position` a press on the track first jumps the widget's middle there.
//! With `stop_locations` a new position snaps down to one of `num_stop_locations` evenly spaced
//! stops (`HandleMoveTo`). The `left_button` / `right_button` (`up_button` / `down_button`) hot
//! buttons step one stop when `buttons_interact` is set (`HandleButtonHotclick`,
//! `HandleMoveSteps`). Every move broadcasts `new_position`.
//!
//! The wheel steps a stop-located bar, or moves the lists and texts it drives a line
//! (`HandleMouseWheel`). A disabled bar takes nothing. Not modelled yet: proportional widgets,
//! the fill children, paging by holding the track and smooth movement.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::{UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::token::tokenize;

pub(crate) const SCROLLBAR: u32 = tokenize("scrollbar");
/// `ScrollbarMsg_new_position`.
pub const MSG_NEW_POSITION: u32 = tokenize("new_position");
const MSG_WIDGET_PRESS: u32 = tokenize("widget_press");
const MSG_WIDGET_RELEASE: u32 = tokenize("widget_release");
const WIDGET: u32 = tokenize("widget");
const X_POSITION: u32 = tokenize("x_position");
const Y_POSITION: u32 = tokenize("y_position");
const NUM_STOP_LOCATIONS: u32 = tokenize("num_stop_locations");
const WIDGET_PRESSED: u32 = tokenize("pressed");
const WIDGET_NORMAL: u32 = tokenize("normal");
const LEFT: [u32; 2] = [0x4000_0004, 0x4000_0007];

/// A scrollbar's drag (`m_nMouseWidgetXOffset`, `m_nMouseWidgetYOffset`, bit 0x10000).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ScrollbarState {
    pub dragging: bool,
    pub offset: (i32, i32),
    /// A page press held on the track (`m_dwFlags` 0x20000 to 0x100000 and the next page time,
    /// offset 0x228).
    pub paging: Option<Paging>,
}

/// The page presses `UIElement_Scrollbar::MouseDown` flags on the track: which side of the
/// widget the press was on, per axis, and when the next page is due.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Paging {
    pub next: f64,
    /// Before and after the widget along x (0x20000, 0x40000) and y (0x80000, 0x100000).
    pub before_x: bool,
    pub after_x: bool,
    pub before_y: bool,
    pub after_y: bool,
}

impl UiTree {
    fn is_scrollbar(&self, index: usize) -> bool {
        self.el(index).desc.kind == SCROLLBAR
    }

    fn horizontal(&self, index: usize) -> bool {
        // `vertical` clears bit 1, `horizontal` sets it; unset means vertical.
        self.bool_attribute(index, tokenize("horizontal"))
            && !self.bool_attribute(index, tokenize("vertical"))
    }

    fn position_id(&self, index: usize) -> u32 {
        if self.horizontal(index) {
            X_POSITION
        } else {
            Y_POSITION
        }
    }

    /// `UIElement::GetScrollbar(attribute, bUnique)`: the scroll bar `owner` names in `attribute`.
    /// Without `unique_msg` it is `UIElementManager::GetElement` (IUIManager slot 0x44): the first
    /// scroll bar with that id in the whole UI, in any root. With `unique_msg` it is
    /// `GetUniqueMatchElement` (slot 0x48): a bar of the owner's own root only.
    pub(crate) fn named_scrollbar(&self, owner: usize, attribute: &str) -> Option<usize> {
        let AttributeValue::Id(id) = self.el(owner).attributes.get(&tokenize(attribute))? else {
            return None;
        };
        if self.bool_attribute(owner, tokenize("unique_msg")) {
            return self.child_recursive(self.root_of(owner), *id);
        }
        self.slots.iter().enumerate().find_map(|(index, slot)| {
            let el = slot.as_ref()?;
            (el.desc.kind == SCROLLBAR && el.id() == *id).then_some(index)
        })
    }

    fn widget(&self, index: usize) -> Option<usize> {
        self.el(index)
            .children
            .iter()
            .copied()
            .find(|&c| self.el(c).id() == WIDGET)
    }

    /// The scrollbar's position along its axis (`x_position` / `y_position`).
    pub fn scrollbar_position(&self, index: usize) -> f32 {
        match self.el(index).attributes.get(&self.position_id(index)) {
            Some(AttributeValue::Float(value)) => *value,
            _ => 0.0,
        }
    }

    fn int(&self, index: usize, id: u32) -> Option<i32> {
        match self.el(index).attributes.get(&id) {
            Some(AttributeValue::Int(value)) => Some(*value),
            _ => None,
        }
    }

    /// `SetAttribute_Float(x_position / y_position)`: stored, then `UpdateLayout`.
    pub fn scrollbar_set_position(&mut self, index: usize, position: f32) {
        let id = self.position_id(index);
        self.el_mut(index)
            .attributes
            .insert(id, AttributeValue::Float(position));
        self.scrollbar_update_layout(index);
    }

    /// `SetAttribute_Int(num_stop_locations)`.
    pub fn scrollbar_set_stops(&mut self, index: usize, stops: i32) {
        self.el_mut(index)
            .attributes
            .insert(NUM_STOP_LOCATIONS, AttributeValue::Int(stops));
    }

    /// `UIElement_Scrollbar::UpdateLayout`: the widget along the track by the position.
    pub(crate) fn scrollbar_update_layout(&mut self, index: usize) {
        if !self.is_scrollbar(index) {
            return;
        }
        if self.el(index).attributes.get(&tokenize("allow_updating"))
            == Some(&AttributeValue::Bool(false))
        {
            return;
        }
        let Some(widget) = self.widget(index) else {
            return;
        };
        let (track, w) = (self.el(index).rect, self.el(widget).rect);
        let position = f64::from(self.scrollbar_position(index));
        let (mut x, mut y) = (w.x0, w.y0);
        if self.horizontal(index) {
            let avail = track.width() - w.width();
            x = (f64::from(avail) * position) as i32;
        } else {
            let avail = track.height() - w.height();
            y = (f64::from(avail) * position) as i32;
        }
        if (x, y) != (w.x0, w.y0) {
            self.move_to(widget, x, y);
            self.changed = true;
        }
    }

    /// `UIElement_Scrollbar::HandleMoveTo`: snapped to a stop, kept in [0, 1], set, and
    /// `new_position` broadcast.
    fn scrollbar_move_to(&mut self, index: usize, position: f32) {
        let mut position = position;
        if self.bool_attribute(index, tokenize("stop_locations")) {
            let stops = self.int(index, NUM_STOP_LOCATIONS).unwrap_or(1);
            if stops == 0 || stops == 1 || position >= 1.0 {
                position = 1.0;
            } else {
                let step = 1.0 / (stops - 1) as f32;
                let mut stop = 0.0_f32;
                while stop < 1.0 {
                    if stop <= position && position < step + stop {
                        position = stop;
                        break;
                    }
                    stop += step;
                }
            }
        }
        let position = position.clamp(0.0, 1.0);
        self.scrollbar_set_position(index, position);
        self.lists_follow_scrollbar(index);
        self.texts_follow_scrollbar(index);
        self.trees_follow_scrollbar(index);
        let root = self.root_of(index);
        self.broadcast(index, MSG_NEW_POSITION, root as i32);
    }

    /// `UIElement_Scrollbar::HandleMoveSteps`: `steps` stops back (negative: on).
    fn scrollbar_move_steps(&mut self, index: usize, steps: i32) {
        let mut position = self.scrollbar_position(index);
        let stops = self.int(index, NUM_STOP_LOCATIONS).unwrap_or(1).max(1);
        if stops != 1 && position <= 1.0 {
            position -= 1.0 / (stops - 1) as f32 * steps as f32;
        } else {
            position = 1.0;
        }
        let position = position.clamp(0.0, 1.0);
        let changed = self.scrollbar_position(index) != position;
        self.scrollbar_set_position(index, position);
        self.lists_follow_scrollbar(index);
        self.texts_follow_scrollbar(index);
        self.trees_follow_scrollbar(index);
        if changed {
            let root = self.root_of(index);
            self.broadcast(index, MSG_NEW_POSITION, root as i32);
        }
    }

    /// `UIElement_Scrollbar::MouseDown`: a left press on the track (with
    /// `move_to_touched_position`) moves the widget's middle to the pointer, then any left press
    /// starts dragging the widget.
    pub(crate) fn scrollbar_mouse_down(&mut self, index: usize, action: u32) {
        if !self.is_scrollbar(index) || self.bool_attribute(index, tokenize("disabled")) {
            return;
        }
        if matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT) {
            self.scrollbar_mouse_wheel(index, action);
            return;
        }
        if !LEFT.contains(&action) {
            return;
        }
        let Some(widget) = self.widget(index) else {
            return;
        };
        let (px, py) = self.pointer;
        if !self.el(widget).mouse_over {
            if !self.bool_attribute(index, tokenize("move_to_touched_position")) {
                // `UIElement_Scrollbar::MouseDown`: a press on the track beside the widget
                // pages once. Repeating while the button is held (`Global_Loop`'s
                // `page_first_interval` timer) is not modelled.
                self.scrollbar_page_press(index, widget, (px, py));
                return;
            }
            let (track, w) = (self.screen_box(index), self.el(widget).rect);
            let position = if self.horizontal(index) {
                (px - w.width() / 2 - track.x0) as f32 / (track.width() - w.width()) as f32
            } else {
                (py - w.height() / 2 - track.y0) as f32 / (track.height() - w.height()) as f32
            };
            self.scrollbar_move_to(index, position);
        }
        let w = self.screen_box(widget);
        self.el_mut(index).scrollbar = Some(Box::new(ScrollbarState {
            dragging: true,
            offset: (px - w.x0, py - w.y0),
            paging: None,
        }));
        self.set_state(widget, WIDGET_PRESSED);
        let root = self.root_of(index);
        self.broadcast(index, MSG_WIDGET_PRESS, root as i32);
    }

    /// `UIElement_Scrollbar::MouseDown` on the track beside the widget: a press before the
    /// widget's left or top edge broadcasts the decrement page message, one past its right or
    /// bottom edge the increment (`horizontal_page_*`, `vertical_page_*`); the lists the bar
    /// drives page (`UIElement_ListBox::HandleScrollBarMessage`).
    fn scrollbar_page_press(&mut self, index: usize, widget: usize, at: (i32, i32)) {
        let w = self.screen_box(widget);
        let presses = [
            (false, at.0 < w.x0, at.0 > w.x1),
            (true, at.1 < w.y0, at.1 > w.y1),
        ];
        for (vertical, before, after) in presses {
            if before || after {
                self.scrollbar_page(index, vertical, after);
            }
        }
        // The press arms the repeat: the next page is `page_first_interval` away (0.3 s by
        // the constructor's default) when a vertical flag was set, as MouseDown's `0x180000`
        // test does.
        if presses[1].1 || presses[1].2 {
            let first = self
                .float_attribute(index, tokenize("page_first_interval"))
                .unwrap_or(0.3);
            let paging = Paging {
                next: f64::from(first) + self.time,
                before_x: presses[0].1,
                after_x: presses[0].2,
                before_y: presses[1].1,
                after_y: presses[1].2,
            };
            self.el_mut(index).scrollbar = Some(Box::new(ScrollbarState {
                paging: Some(paging),
                ..ScrollbarState::default()
            }));
        }
    }

    /// `UIElement_Scrollbar::Global_Loop`'s paging half: while the press is held on the track and
    /// its time has come, each flagged side whose widget edge the pointer has not reached, and
    /// whose bar can still move that way, pages again; the next page is one `page_interval`
    /// (0.1 s) after the one due. With no page due the time catches up to now.
    pub(crate) fn scrollbar_global_loop(&mut self, index: usize) {
        if !self.is_scrollbar(index) {
            return;
        }
        let Some(paging) = self
            .el(index)
            .scrollbar
            .as_ref()
            .and_then(|state| state.paging)
        else {
            return;
        };
        let now = self.time;
        if now < paging.next {
            return;
        }
        let Some(widget) = self.widget(index) else {
            return;
        };
        let w = self.screen_box(widget);
        let (px, py) = self.pointer;
        let own = self.scrollbar_position(index);
        let horizontal = self.horizontal(index);
        let axis = |tree: &Self, name: &str, mine: bool| -> Option<f32> {
            tree.float_attribute(index, tokenize(name))
                .or(mine.then_some(own))
        };
        let x_pos = axis(self, "x_position", horizontal);
        let y_pos = axis(self, "y_position", !horizontal);
        let can = |pos: Option<f32>, forward: bool| {
            pos.is_none_or(|p| if forward { p < 1.0 } else { p > 0.0 })
        };
        let mut paged = false;
        // Each side: (flag, pointer past the widget edge, axis, forward, bar can move).
        let sides = [
            (paging.before_x, px < w.x0, false, false, can(x_pos, false)),
            (paging.after_x, px > w.x1, false, true, can(x_pos, true)),
            (paging.before_y, py < w.y0, true, false, can(y_pos, false)),
            (paging.after_y, py > w.y1, true, true, can(y_pos, true)),
        ];
        for (flag, past, vertical, forward, movable) in sides {
            if flag && past && movable {
                self.scrollbar_page(index, vertical, forward);
                paged = true;
            }
        }
        let interval = self
            .float_attribute(index, tokenize("page_interval"))
            .unwrap_or(0.1);
        if let Some(state) = self.el_mut(index).scrollbar.as_mut() {
            if let Some(p) = state.paging.as_mut() {
                if paged {
                    p.next += f64::from(interval);
                } else if p.next < now {
                    p.next = now;
                }
            }
        }
    }

    /// The page message of a bar's axis (`vertical_page_increment` and its three partners),
    /// broadcast on the bar's root and heard by the lists it drives.
    pub(crate) fn scrollbar_page(&mut self, index: usize, vertical: bool, forward: bool) {
        let name = match (vertical, forward) {
            (true, true) => "vertical_page_increment",
            (true, false) => "vertical_page_decrement",
            (false, true) => "horizontal_page_increment",
            (false, false) => "horizontal_page_decrement",
        };
        let root = self.root_of(index);
        self.broadcast(index, tokenize(name), root as i32);
        for (list, list_vertical) in self.lists_bound_to(index) {
            if list_vertical == vertical {
                self.list_page_step(list, vertical, forward);
            }
        }
        if vertical {
            for tree in self.trees_bound_to(index) {
                self.tree_goal_step(tree, true, forward, true);
            }
        }
    }

    /// `UIElement_Scrollbar::HandleMouseWheel`: a wheel turn steps a stop-located bar (up or left
    /// one stop back); otherwise the lists the bar drives move a line, up or left on a turn in
    /// (the line message the bar broadcasts). A bar with no position attribute ignores it.
    pub(crate) fn scrollbar_mouse_wheel(&mut self, index: usize, action: u32) {
        if !self.is_scrollbar(index)
            || !self
                .el(index)
                .attributes
                .contains_key(&self.position_id(index))
        {
            return;
        }
        let turn_in = action == UI_WHEEL_IN;
        if self.bool_attribute(index, tokenize("stop_locations")) {
            self.scrollbar_move_steps(index, if turn_in { 1 } else { -1 });
            return;
        }
        for (list, vertical) in self.lists_bound_to(index) {
            self.list_line_step(list, vertical, if turn_in { -1 } else { 1 });
        }
        for text in self.texts_bound_to(index) {
            self.text_line_step(text, if turn_in { -1 } else { 1 });
        }
        for tree in self.trees_bound_to(index) {
            self.tree_line_step(tree, if turn_in { -1 } else { 1 });
        }
    }

    /// `UIElement_Scrollbar::MouseUp`: the drag ends.
    pub(crate) fn scrollbar_mouse_up(&mut self, index: usize) {
        if !self.is_scrollbar(index) {
            return;
        }
        let dragging = self
            .el_mut(index)
            .scrollbar
            .take()
            .is_some_and(|s| s.dragging);
        if let Some(widget) = self.widget(index) {
            if dragging {
                self.set_state(widget, WIDGET_NORMAL);
            }
            let root = self.root_of(index);
            self.broadcast(index, MSG_WIDGET_RELEASE, root as i32);
        }
    }

    /// `UIElement_Scrollbar::Global_MouseMove` for every dragged scrollbar: the widget follows
    /// the pointer (`HandleMouseMove`).
    pub(crate) fn scrollbar_mouse_move(&mut self) {
        let dragged: Vec<(usize, (i32, i32))> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(i, el)| {
                let state = el.as_ref()?.scrollbar.as_ref()?;
                state.dragging.then_some((i, state.offset))
            })
            .collect();
        for (index, offset) in dragged {
            let Some(widget) = self.widget(index) else {
                continue;
            };
            let (track, w) = (self.screen_box(index), self.el(widget).rect);
            let (px, py) = self.pointer;
            let (max, new) = if self.horizontal(index) {
                (track.width() - w.width(), px - offset.0 - track.x0)
            } else {
                (track.height() - w.height(), py - offset.1 - track.y0)
            };
            let position = if max == 0 {
                0.0
            } else {
                new.clamp(0, max.max(0)) as f32 / max as f32
            };
            self.scrollbar_move_to(index, position);
        }
    }

    /// `UIElement_Scrollbar::ListenToMessage(hot_click)`: a hot button named by a scrollbar's
    /// `left_button` / `up_button` steps it one stop back, `right_button` / `down_button` one on,
    /// when `buttons_interact` is set (`HandleButtonHotclick`).
    pub(crate) fn scrollbar_hot_click(&mut self, button: usize) {
        let button_id = self.el(button).id();
        let scrollbars: Vec<usize> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(i, el)| (el.as_ref()?.desc.kind == SCROLLBAR).then_some(i))
            .collect();
        for index in scrollbars {
            for (name, steps) in [
                ("up_button", 1),
                ("down_button", -1),
                ("left_button", 1),
                ("right_button", -1),
            ] {
                let named = matches!(
                    self.el(index).attributes.get(&tokenize(name)),
                    Some(AttributeValue::Id(id)) if *id == button_id
                );
                if named && self.bool_attribute(index, tokenize("buttons_interact")) {
                    self.scrollbar_move_steps(index, steps);
                }
            }
        }
    }
}
