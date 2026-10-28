//! `UIElement_ListBox`'s smooth movement and goals. A step sets the `goal_x_view` /
//! `goal_y_view` attribute, constrained like the view (`SetAttribute_Int`). A list without
//! `smooth_movement` takes the goal as its view at once and clears the goal; a smooth list moves
//! there over `smooth_movement_duration` seconds of the global loop (`StartAnimation`,
//! `Global_Loop`): each loop sets the view to `start + trunc((end - start) * t)`, `t` the elapsed
//! share held as a `float`, x then y, until `t` reaches one and the goals clear. A goal set while
//! the list moves restarts the movement half way (`start = 2 * view - goal`, started half the
//! duration ago), and steps taken while it moves start from the goal (`DetermineAndSetNewGoal`).
//! A view set from elsewhere after the loop has moved the list stops it (`SetAttribute_Int`,
//! `m_bitField & 0x80000`). A smooth list with no duration does not move.
//!
//! Letting go of a scroll bar's widget sets each list it drives a goal on the nearest item
//! boundary (`HandleScrollbarWidgetRelease`), unless the list `allow_largeitem_scroll`s, and at
//! the far end not when it `allow_uneven_at_end`.
//!
//! Admin 7148 addresses: `StartAnimation` 0x005F23F0, `StopAnimation` 0x005F2500, `Global_Loop`
//! 0x005F3320 (the float math from the recovery export), `SetAttribute_Int` 0x005F4170,
//! `DetermineAndSetNewGoal` 0x005F2180, `HandleScrollbarWidgetRelease` 0x005F2F60.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

const GOAL_X_VIEW: u32 = tokenize("goal_x_view");
const GOAL_Y_VIEW: u32 = tokenize("goal_y_view");
const SMOOTH_MOVEMENT_DURATION: u32 = tokenize("smooth_movement_duration");

/// The goal attribute's "no goal" value.
pub(crate) const NO_GOAL: i32 = -1_000_000;

/// A moving list's ends and times (`m_nAnimStartX` ... `m_nAnimEndTime`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ListAnim {
    pub start: (i32, i32),
    pub end: (i32, i32),
    pub start_time: f64,
    pub end_time: f64,
}

fn goal_id(vertical: bool) -> u32 {
    if vertical {
        GOAL_Y_VIEW
    } else {
        GOAL_X_VIEW
    }
}

impl UiTree {
    /// The goal attribute, `NO_GOAL` when unset.
    fn list_goal(&self, list: usize, vertical: bool) -> i32 {
        match self.el(list).attributes.get(&goal_id(vertical)) {
            Some(AttributeValue::Int(v)) => *v,
            _ => NO_GOAL,
        }
    }

    fn list_store_goal(&mut self, list: usize, vertical: bool, goal: i32) {
        self.el_mut(list)
            .attributes
            .insert(goal_id(vertical), AttributeValue::Int(goal));
    }

    /// `DetermineAndSetNewGoal`'s base: the goal while the list moves, else `view`.
    pub(crate) fn list_step_base(&self, list: usize, vertical: bool, view: i32) -> i32 {
        let moving = self
            .el(list)
            .list
            .as_ref()
            .is_some_and(|s| s.anim.is_some());
        if moving {
            self.list_goal(list, vertical)
        } else {
            view
        }
    }

    /// `SetAttribute_Int(goal_x_view / goal_y_view, goal)`.
    pub(crate) fn list_set_goal(&mut self, list: usize, vertical: bool, goal: i32) {
        let goal = self.list_constrain(list, !vertical, goal);
        self.list_store_goal(list, vertical, goal);
        if self.list_flag(list, "smooth_movement") {
            self.list_start_animation(list);
            return;
        }
        if vertical {
            self.list_set_view(list, None, Some(goal));
        } else {
            self.list_set_view(list, Some(goal), None);
        }
        self.list_store_goal(list, vertical, NO_GOAL);
    }

    /// `UIElement_ListBox::StartAnimation`.
    fn list_start_animation(&mut self, list: usize) {
        let duration = match self.el(list).attributes.get(&SMOOTH_MOVEMENT_DURATION) {
            Some(AttributeValue::Float(d)) => *d,
            _ => 0.0,
        };
        if duration == 0.0 {
            return;
        }
        let (gx, gy) = (self.list_goal(list, false), self.list_goal(list, true));
        let now = self.time;
        let state = self.list(list);
        let view = state.view;
        let end = (
            if gx == NO_GOAL { view.0 } else { gx },
            if gy == NO_GOAL { view.1 } else { gy },
        );
        let (start, start_time) = if state.anim.is_some() {
            (
                (view.0 * 2 - end.0, view.1 * 2 - end.1),
                now - f64::from(duration) * 0.5,
            )
        } else {
            state.anim_moved = false;
            (view, now)
        };
        // `m_nAnimEndTime = (double)(duration + (float)m_nAnimStartTime)`.
        let end_time = f64::from(duration) + f64::from(start_time as f32);
        state.anim = Some(ListAnim {
            start,
            end,
            start_time,
            end_time,
        });
    }

    /// `UIElement_ListBox::StopAnimation`.
    pub(crate) fn list_stop_animation(&mut self, list: usize) {
        self.list_store_goal(list, false, NO_GOAL);
        self.list_store_goal(list, true, NO_GOAL);
        self.list(list).anim = None;
    }

    /// `UIElement_ListBox::Global_Loop` at time `now`.
    pub(crate) fn list_global_loop(&mut self, list: usize, now: f64) {
        let Some(anim) = self.el(list).list.as_ref().and_then(|s| s.anim) else {
            return;
        };
        let ratio = (now - anim.start_time) / (anim.end_time - anim.start_time);
        let mut t = ratio as f32;
        if 1.0 <= ratio {
            t = 1.0;
            self.list_stop_animation(list);
        }
        // `_ftol2` truncates toward zero.
        let dy = (f64::from(anim.end.1 - anim.start.1) * f64::from(t)) as i32;
        let dx = (f64::from(anim.end.0 - anim.start.0) * f64::from(t)) as i32;
        self.list(list).anim_moved = false;
        self.list_set_view(list, Some(anim.start.0 + dx), None);
        self.list_set_view(list, None, Some(anim.start.1 + dy));
        self.list(list).anim_moved = true;
    }

    /// `UIElement_ListBox::HandleScrollbarWidgetRelease`: the view goes to the nearest item
    /// boundary.
    pub(crate) fn list_scrollbar_widget_release(&mut self, list: usize) {
        if self.list_flag(list, "allow_largeitem_scroll") {
            return;
        }
        let uneven = self.list_flag(list, "allow_uneven_at_end");
        let rect = self.el(list).rect;
        let Some(state) = self.el(list).list.as_ref() else {
            return;
        };
        let (item, view, paper) = (state.item, state.view, state.paper);
        if item.0 == 0 || item.1 == 0 {
            return;
        }
        let snap = |view: i32, size: i32| (size / 2 + view) / size * size;
        let (x, y) = (snap(view.0, item.0), snap(view.1, item.1));
        if x != view.0 && (view.0 < paper.0 - rect.width() || !uneven) {
            self.list_set_goal(list, false, x);
        }
        if y != view.1 && (view.1 < paper.1 - rect.height() || !uneven) {
            self.list_set_goal(list, true, y);
        }
    }
}
