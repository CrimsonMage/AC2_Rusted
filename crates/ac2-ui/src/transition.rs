//! `UIElement::SetState` as the layouts' state machine, with its transitions. Leaving a state
//! starts the old state's exit transitions (`when` 2) whose target is the new state (or
//! `all_states`); while one runs the switch waits, and it happens when the last one stops
//! (`StopTransition` sets the state it was heading to). Entering a state starts the new state's
//! enter transitions (`when` 1) from the old state (or `all_states`, or `no_state` from state
//! 0). Transitions move only an element with its own surface object; a fade
//! (`TD_Data_Fade`) takes the object's material opacity from the start alpha (or the current
//! opacity unless flag 1) to the end alpha over its duration (`UpdateTransitions`).
//!
//! Not modelled: rotations (`TD_Data_Rotate`; no layout on the DVD has one, and retail reads
//! their duration from the fade's slot), moves and resizes (retail starts none of them), and
//! deleting an element when its exit transition stops (`m_delOnTransStop`).

use crate::element::UiTree;
use crate::layout_media::{TransitionData, TransitionDesc};
use crate::token::tokenize;

const ALL_STATES: u32 = tokenize("all_states");
const NO_STATE: u32 = tokenize("no_state");

/// A running transition (`m_transitionArray`'s entry and its run-time data).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ActiveTransition {
    desc: TransitionDesc,
    start_time: f64,
    start_alpha: f32,
}

impl UiTree {
    /// `UIElement::SetState`'s transitions around [`UiTree::enter_state`].
    pub(crate) fn transition_set_state(&mut self, index: usize, state: u32) {
        let el = self.el(index);
        let old = el.state;
        if state == old {
            if el.transitions.is_empty() {
                return;
            }
        } else if el.transitions.first().is_some_and(|t| t.desc.when == 2)
            && state == el.transition_state
        {
            return;
        }
        self.stop_all_transitions(index);
        if !self.el(index).transition_close {
            if let Some((desc, at)) = self.transition_state_desc(index, old) {
                for t in &desc.states[at].transitions {
                    if t.when == 2 && (t.state == state || t.state == ALL_STATES) {
                        self.start_transition(index, *t, state);
                    }
                }
            }
        }
        if !self.el(index).transitions.is_empty() {
            return;
        }
        self.el_mut(index).transition_close = false;
        self.enter_state(index, state);
        let Some(el) = self.get(index) else {
            return;
        };
        let now_state = el.state;
        if let Some((desc, at)) = self.transition_state_desc(index, now_state) {
            for t in &desc.states[at].transitions {
                if t.when == 1
                    && (old == t.state
                        || t.state == ALL_STATES
                        || (old == 0 && t.state == NO_STATE))
                {
                    self.start_transition(index, *t, state);
                }
            }
        }
    }

    /// `UIElement::StartTransition`: only a fade on an element with its own surface object.
    fn start_transition(&mut self, index: usize, desc: TransitionDesc, state: u32) -> bool {
        let now = self.time;
        let el = self.el_mut(index);
        if el.object.is_none() {
            return false;
        }
        let TransitionData::Fade {
            start_alpha, flags, ..
        } = desc.data
        else {
            return false;
        };
        let start_alpha = if flags & 1 == 0 {
            el.opacity
        } else {
            start_alpha
        };
        el.transition_state = state;
        el.transitions.push(ActiveTransition {
            desc,
            start_time: now,
            start_alpha,
        });
        self.update_transitions(index);
        true
    }

    /// `UIElement::UpdateTransitions`: each fade's opacity at the time; finished ones stop.
    pub(crate) fn update_transitions(&mut self, index: usize) {
        let now = self.time;
        let el = self.el_mut(index);
        if el.transitions.is_empty() {
            return;
        }
        let mut finished = 0;
        let mut opacity = None;
        for t in &el.transitions {
            let TransitionData::Fade {
                end_alpha,
                duration,
                ..
            } = t.desc.data
            else {
                finished += 1;
                continue;
            };
            if el.object.is_none() {
                finished += 1;
                continue;
            }
            let ratio = (now - t.start_time) / duration;
            let progress = if 1.0 < ratio { 1.0 } else { ratio as f32 };
            let start = f64::from(t.start_alpha);
            opacity = Some(((f64::from(end_alpha) - start) * f64::from(progress) + start) as f32);
            if progress >= 1.0 {
                finished += 1;
            }
        }
        if let Some(opacity) = opacity {
            self.set_object_opacity(index, opacity);
        }
        // Retail stops the array's first entries, as many as finished.
        for at in 0..finished {
            let Some(t) = self.get(index).and_then(|e| e.transitions.get(at)).copied() else {
                continue;
            };
            self.stop_transition(index, t);
        }
    }

    /// `UIElement::StopTransition`: an exit transition that was the last one sets the state it
    /// was heading to.
    fn stop_transition(&mut self, index: usize, t: ActiveTransition) {
        let el = self.el_mut(index);
        if let Some(at) = el.transitions.iter().position(|x| *x == t) {
            el.transitions.remove(at);
        }
        if el.transitions.is_empty() && t.desc.when == 2 {
            el.transition_close = true;
            let state = el.transition_state;
            self.set_state(index, state);
        }
    }

    /// `UIElement::StopAllTransitions`: dropped where they are (the opacity stays).
    fn stop_all_transitions(&mut self, index: usize) {
        self.el_mut(index).transitions.clear();
    }

    /// `UISurfaceObject::SetMaterialOpacity` of a root's surface object.
    pub fn set_object_opacity(&mut self, root: usize, opacity: f32) {
        let el = self.el_mut(root);
        if el.opacity != opacity {
            el.opacity = opacity;
            self.moved = true;
        }
    }

    /// `UIMESSAGE_GLOBAL_LOOP` to every element (`UIElement::ListenToMessage`): its transitions,
    /// then its media; a meter's movement (`UIElement_Meter::Global_Loop`).
    pub(crate) fn element_global_loop(&mut self, index: usize) {
        if self.get(index).is_none() {
            return;
        }
        self.update_transitions(index);
        self.button_global_loop(index);
        self.scrollbar_global_loop(index);
        self.text_cursor_flash(index);
        if self.get(index).is_some() && self.media_running(index) {
            self.update_media(index);
        }
    }
}
