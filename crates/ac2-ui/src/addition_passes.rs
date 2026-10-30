//! The HUD frame's passes for the modern additions (no retail counterpart): the quest guide and
//! its In-Game Wiki button, presses on read-only text (map coordinates, shared guide pages), the
//! world map and the Tracker. They ask nothing of the player, so [`Hud::frame`] runs them with the
//! rest in the world and [`Hud::offline_frame`] runs them alone where there is no player (the
//! world viewer).

use crate::element::UiTree;
use crate::guide_window::GuideRequest;
use crate::{Hud, HudRequest, UiMessage};

impl Hud {
    /// The additions' frame without a player: the messages since the last frame go to the guide,
    /// the world map and the Tracker. Returns their requests for the client to map.
    pub fn offline_frame(
        &mut self,
        tree: Option<&mut UiTree>,
        dirty: &mut bool,
    ) -> Vec<HudRequest> {
        let mut messages = std::mem::take(&mut self.player_messages);
        let Some(tree) = tree else {
            return Vec::new();
        };
        let mut requests = Vec::new();
        self.tracker_enter(tree, dirty, &mut messages, &mut requests);
        self.addition_pass(tree, dirty, &messages, &mut requests);
        if tree.take_changed() {
            *dirty = true;
        }
        requests
    }

    /// Opens the quest guide, or closes it when it is open (the In-Game Wiki button's press).
    pub fn toggle_guide(&mut self, tree: &mut UiTree) {
        if let Some(guide) = self.guide.as_mut() {
            if guide.is_shown() {
                guide.close(tree);
            } else {
                guide.open(tree);
            }
        }
    }

    /// Opens the Tracker's window, or closes it when it is open (its radar eyeball's press).
    pub fn toggle_tracker(&mut self, tree: &mut UiTree) {
        if let Some(tracker) = self.tracker.as_mut() {
            let shown = tracker.is_shown();
            tracker.show(tree, !shown);
        }
    }

    /// The Tracker's pattern field takes Enter while it has the focus (it adds the pattern),
    /// before the chat would open its own entry line with it.
    pub(crate) fn tracker_enter(
        &mut self,
        tree: &mut UiTree,
        dirty: &mut bool,
        messages: &mut Vec<UiMessage>,
        requests: &mut Vec<HudRequest>,
    ) {
        if let Some(tracker) = self.tracker.as_mut() {
            if let Some(request) = tracker.take_enter(tree, messages) {
                requests.push(HudRequest::Tracker(request));
                *dirty = true;
            }
        }
    }

    /// The additions' messages, after the retail panels have had theirs.
    pub(crate) fn addition_pass(
        &mut self,
        tree: &mut UiTree,
        dirty: &mut bool,
        messages: &[UiMessage],
        requests: &mut Vec<HudRequest>,
    ) {
        // A press on read-only text: a coordinate under it points the floating arrow there, and
        // the line goes to the client (a Tracker line's name selects what it found).
        if let Some(press) = tree.take_text_press() {
            if let Some((north, east)) = press.coordinate {
                requests.push(HudRequest::PointAt { north, east });
            }
            // A shared guide page (`[wiki:slug]`) opens the guide there.
            if let (Some(slug), Some(guide)) = (
                ac2_guide::browse::token_at(&press.line, press.column),
                self.guide.as_mut(),
            ) {
                guide.open_slug(tree, slug);
                *dirty = true;
            }
            requests.push(HudRequest::TextPress(press));
        }
        // The quest guide: the option panel's In-Game Wiki button opens and closes it, and its
        // own messages (Share fills the chat line; a coordinate points the arrow).
        if self
            .option_panel
            .as_mut()
            .is_some_and(|panel| panel.take_wiki())
        {
            self.toggle_guide(tree);
            *dirty = true;
        }
        // The modern Edit HUD layout button: the mode turns on (or off), the option panel closes.
        if self
            .option_panel
            .as_mut()
            .is_some_and(|panel| panel.take_edit())
        {
            if let Some(edit) = self.modern_edit.as_ref() {
                edit.set_on(tree, !edit.is_on());
            }
            if let Some(bar) = self.toolbar.as_ref() {
                bar.show(tree, crate::ToolbarChild::Option, false);
            }
            *dirty = true;
        }
        if let Some(guide) = self.guide.as_mut() {
            for message in messages {
                let (taken, request) = guide.message(tree, message);
                *dirty |= taken;
                match request {
                    Some(GuideRequest::Share(text)) => {
                        if let Some(chat) = self.chat.as_mut() {
                            chat.prefill(tree, &text);
                        }
                    }
                    Some(GuideRequest::PointAt(at)) => requests.push(HudRequest::PointAt {
                        north: at.north,
                        east: at.east,
                    }),
                    None => {}
                }
            }
        }
        // The world map's wheel, drag, layers button and layers list.
        if let (Some(map), Some(nav)) = (self.world_map.as_mut(), self.navigation.as_ref()) {
            for message in messages {
                *dirty |= map.message(tree, nav, message);
            }
        }
        // The Tracker's eyeball and window.
        if let Some(tracker) = self.tracker.as_mut() {
            for message in messages {
                let (taken, request) = tracker.message(tree, message);
                *dirty |= taken;
                requests.extend(request.map(HudRequest::Tracker));
            }
        }
    }
}
