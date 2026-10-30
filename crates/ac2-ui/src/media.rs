//! An element's media timeline: the state's media items run in order (`UIElement::UpdateMedia`)
//! until one waits, and the global loop resumes it. Each item is `MD_Data_*::Update`: an image or
//! alpha mask is set, a sound played, the cursor set, and the run goes on; an animation shows its
//! frames over its duration and a pause waits a rolled time; a jump or message happens with its
//! probability; a state item switches the element's state, which starts that state's media.
//!
//! Retail keeps a timed item's clock in the shared description (`MD_Data_Anim::m_StartTime`,
//! `MD_Data_Pause::m_endTime`), so every element made from one description shares it, and
//! `ResetMedia` resets it for all of them; the clocks here are kept per description item too.

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::{ElementDesc, MediaDesc};
use crate::token::STATELESS;

/// A timed media item's clock (`MD_Data_Anim`'s start and shown frame, `MD_Data_Pause`'s end).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MediaClock {
    start: f64,
    shown_frame: i32,
    end: f64,
}

impl Default for MediaClock {
    /// `MD_Data_Anim::Reset`, `MD_Data_Pause::Reset`: -1, unset.
    fn default() -> Self {
        Self {
            start: -1.0,
            shown_frame: -1,
            end: -1.0,
        }
    }
}

/// The clock key of media item `item` of state `state` of `desc`.
fn clock_key(desc: &ElementDesc, state: usize, item: usize) -> usize {
    std::ptr::from_ref(&desc.states[state].media[item]) as usize
}

impl UiTree {
    /// `UIElement::ResetMedia`: the media of the current state, else the stateless state's, else
    /// (for a reference) the reference's current then stateless state's; every item reset, then
    /// run from the first.
    pub(crate) fn reset_media(&mut self, index: usize) {
        let el = self.el(index);
        // A state of the element's own description also has media when `AddImageToState`
        // appended an image to it.
        let added = |desc: &Arc<ElementDesc>, id: u32| {
            Arc::ptr_eq(desc, &el.desc) && el.state_images.iter().any(|(s, _)| *s == id)
        };
        let pick = |desc: &Arc<ElementDesc>, id: u32| {
            desc.states
                .iter()
                .position(|s| s.id == id && (!s.media.is_empty() || added(desc, id)))
                .map(|i| (Arc::clone(desc), i))
        };
        let state = el.state;
        let current = (state != STATELESS)
            .then(|| pick(&el.desc, state))
            .flatten();
        let media = current.or_else(|| pick(&el.desc, STATELESS)).or_else(|| {
            if Arc::ptr_eq(&el.reference, &el.desc) {
                return None;
            }
            (state != STATELESS)
                .then(|| pick(&el.reference, state))
                .flatten()
                .or_else(|| pick(&el.reference, STATELESS))
        });
        if let Some((desc, state)) = &media {
            for item in 0..desc.states[*state].media.len() {
                self.media_clocks
                    .insert(clock_key(desc, *state, item), MediaClock::default());
            }
        }
        let el = self.el_mut(index);
        el.media_state = media;
        el.media_index = 0;
        self.update_media(index);
    }

    /// `UIElement::ResetMedia` on its own: the element's media runs again from its first item (a
    /// story's background, whose fade is its media, starts each story from the top).
    pub fn restart_media(&mut self, index: usize) {
        self.reset_media(index);
    }

    /// The messages the element's description sends from its media (`MD_Data_Message`), in layout
    /// order.
    pub fn media_messages(&self, index: usize) -> Vec<u32> {
        self.el(index)
            .desc
            .states
            .iter()
            .flat_map(|state| state.media.iter())
            .filter_map(|media| match media {
                MediaDesc::Message { message, .. } => Some(*message),
                _ => None,
            })
            .collect()
    }

    /// `UIElement::UpdateMedia`: the items from the current one until one waits.
    pub(crate) fn update_media(&mut self, index: usize) {
        loop {
            let Some(el) = self.get(index) else {
                return;
            };
            let Some((desc, state)) = el.media_state.clone() else {
                return;
            };
            let Ok(item) = usize::try_from(el.media_index) else {
                return;
            };
            match desc.states[state].media.get(item) {
                Some(media) => {
                    if !self.run_media(index, &desc, state, item, media) {
                        return;
                    }
                }
                None => {
                    // The appended images (`MD_Data_Image`, which never waits) follow the
                    // description's items.
                    let id = desc.states[state].id;
                    let extra = item - desc.states[state].media.len();
                    let image = Arc::ptr_eq(&desc, &el.desc)
                        .then(|| el.state_images.iter().filter(|(s, _)| *s == id).nth(extra))
                        .flatten()
                        .map(|(_, image)| *image);
                    let Some(image) = image else {
                        return;
                    };
                    self.set_image(index, image.file, image.draw_mode);
                }
            }
            let Some(el) = self.slots[index].as_mut() else {
                return;
            };
            el.media_index += 1;
        }
    }

    /// `MD_Data_*::Update`: whether the run goes on to the next item.
    fn run_media(
        &mut self,
        index: usize,
        desc: &ElementDesc,
        state: usize,
        item: usize,
        media: &MediaDesc,
    ) -> bool {
        let key = clock_key(desc, state, item);
        let now = self.time;
        match media {
            MediaDesc::Image {
                file, draw_mode, ..
            } => self.set_image(index, *file, *draw_mode),
            MediaDesc::Alpha { file, .. } => {
                // `UIElement::SetAlphaImage`: the alpha image replaces the old one.
                self.el_mut(index).alpha_image = Some(*file);
                self.changed = true;
            }
            MediaDesc::Sound { sound } => self.sounds.push(*sound),
            MediaDesc::Cursor { file, hotspot } => {
                self.set_element_cursor(index, *file, hotspot.0, hotspot.1);
            }
            MediaDesc::Anim {
                duration,
                draw_mode,
                frames,
                ..
            } => {
                let clock = self.media_clocks.entry(key).or_default();
                if clock.start == -1.0 {
                    clock.start = now;
                    clock.shown_frame = -1;
                }
                let count = frames.len() as i32;
                if count != 0 {
                    let per_frame = f64::from(*duration) / f64::from(count);
                    let frame = (((now - clock.start) / per_frame) as i32).min(count - 1);
                    let shown = frame != clock.shown_frame;
                    if shown {
                        clock.shown_frame = frame;
                    }
                    let waiting = clock.shown_frame + 1 < count;
                    if shown {
                        self.set_image(index, frames[frame as usize], *draw_mode);
                    }
                    if waiting {
                        return false;
                    }
                }
                self.media_clocks.insert(key, MediaClock::default());
            }
            MediaDesc::Pause { min, max } => {
                if self.media_clocks.get(&key).is_none_or(|c| c.end == -1.0) {
                    // The roll is added to the time in double (objdump; the decompile shows a
                    // float add).
                    let roll = self.random.roll_dice_f32(*min, *max);
                    self.media_clocks.entry(key).or_default().end = f64::from(roll) + now;
                }
                if now < self.media_clocks[&key].end {
                    return false;
                }
                self.media_clocks.insert(key, MediaClock::default());
            }
            MediaDesc::Jump {
                index: to,
                probability,
            } => {
                if self.random.roll_dice_f32(0.0, 1.0) <= *probability {
                    // `SetMediaIndex(jump - 1)`: the run's step lands on the jump's item.
                    self.el_mut(index).media_index = *to as i32 - 1;
                }
            }
            MediaDesc::Message {
                message,
                probability,
            } => {
                if self.random.roll_dice_f32(0.0, 1.0) <= *probability {
                    self.broadcast(index, *message, 0);
                }
            }
            MediaDesc::State { state } => {
                self.set_state(index, *state);
                return false;
            }
            // `MD_Data_AVI::Update` plays a movie; no layout on the DVD has one.
            MediaDesc::Avi { .. } => {}
        }
        true
    }

    /// `UIElement::SetCursor`: the element's cursor, shown at once while the pointer is on it.
    pub(crate) fn set_element_cursor(&mut self, index: usize, file: u32, x: i32, y: i32) {
        let el = self.el_mut(index);
        el.cursor = (file, x, y);
        if el.mouse_over_top && file != 0 {
            self.set_pointer_cursor(file, x, y, false);
        }
    }

    /// `UIElementManager::SetCursor`: a cursor (0 keeps the current one); `default` also makes it
    /// the one [`UiTree::use_default_pointer_cursor`] goes back to.
    pub fn set_pointer_cursor(&mut self, file: u32, x: i32, y: i32, default: bool) {
        if default {
            self.default_cursor = Some((file, x, y));
        }
        if file != 0 {
            self.cursor = Some((file, x, y));
        }
    }

    /// `UIElementManager::UseDefaultCursor`.
    pub fn use_default_pointer_cursor(&mut self) {
        if let Some((file, x, y)) = self.default_cursor {
            self.set_pointer_cursor(file, x, y, true);
        }
    }

    /// The pointer's cursor image and hotspot, once one was set.
    pub fn pointer_cursor(&self) -> Option<(u32, i32, i32)> {
        self.cursor
    }

    /// The sounds (`MD_Data_Sound`'s DIDs) the media played since the last call.
    pub fn take_sounds(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.sounds)
    }

    /// Whether the element still has media to run.
    pub(crate) fn media_running(&self, index: usize) -> bool {
        let el = self.el(index);
        el.media_state.as_ref().is_some_and(|(desc, state)| {
            usize::try_from(el.media_index).is_ok_and(|i| i < desc.states[*state].media.len())
        })
    }

    /// The state description `SetState`'s transitions come from: the element's `state`, else
    /// the stateless one.
    pub(crate) fn transition_state_desc(
        &self,
        index: usize,
        state: u32,
    ) -> Option<(Arc<ElementDesc>, usize)> {
        let desc = Arc::clone(&self.el(index).desc);
        let position = |id: u32| desc.states.iter().position(|s| s.id == id);
        let found = position(state).or_else(|| position(STATELESS));
        found.map(|i| (desc, i))
    }
}
