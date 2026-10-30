//! `gmStoryUI`'s drawn part over the gameplay layout (`0x41000009`): the `BackGround` root and the
//! two story field roots `StoryField1` and `StoryField2`, each with its `ImageField` and
//! `TextField`. `gmStoryUI::gmStoryUI` makes the roots and `gmStoryUI::Play` draws each keyframe
//! into the field `_lastState` names (`Frame1` is field 1, `Frame2` field 2).
//!
//! Retail makes the roots when the story is made and removes them when it is destroyed. Here they
//! are made once with the HUD and hidden between stories (as [`crate::BookUi`] keeps its panel),
//! so [`StoryView::show`] and [`StoryView::hide`] are the only lifetime there is.
//!
//! The cross-fade: a keyframe whose image differs from the last one flips the active field. The
//! new field's state becomes its `Frame` state and the other field goes back to `stateless`; the
//! layout's transitions between those states are the fade (`story_layout_tests.rs`). The
//! image-less first keyframe still starts field 1 in `Frame1`, as `gmStoryUI::ListenToMessage`
//! does when the background's fade is done.

use std::sync::Arc;

use ac2_net::protocol::StringInfo;

use crate::element::UiTree;
use crate::gameplay_story::StoryEvent;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::story::{StoryField, StoryShow};
use crate::token::{tokenize, STATELESS};
use crate::ui_text::UiStringTables;

/// The background root (`gmStoryUI::_bgField`, the `BackgroundField` element of the gameplay layout).
const BACKGROUND: &str = "BackgroundField";
/// The two field roots, in `StoryField` order.
const FIELD_NAMES: [&str; 2] = ["StoryField1", "StoryField2"];
/// `gmStoryUI::Play`'s element names inside a field.
const IMAGE_FIELD: &str = "ImageField";
const TEXT_FIELD: &str = "TextField";
/// The state a field takes for each `StoryField` (`Frame1` and `Frame2`).
const FRAME_STATES: [&str; 2] = ["Frame1", "Frame2"];
/// `gmStoryUI::Play`'s `AddImageToState(.., 0, ..)` draw mode for the keyframe image.
const IMAGE_DRAW_MODE: u32 = 0;

/// One story field's elements.
#[derive(Debug, Clone, Copy)]
struct FieldElements {
    root: usize,
    image: Option<usize>,
    text: Option<usize>,
    /// The state `Frame1` or `Frame2`.
    frame: u32,
}

/// `gmStoryUI`'s drawn elements.
#[derive(Debug, Clone)]
pub struct StoryView {
    background: usize,
    /// Field 1 and field 2, indexed by `StoryField as usize`.
    fields: [FieldElements; 2],
    /// Whether a story is on the screen (`UIFramework::IsShown`).
    shown: bool,
    /// Whether the active field has taken its first `Frame` state in this story.
    started: bool,
    /// The message the background's media sends when its fade is over (`gmStoryUI::ListenToMessage`
    /// compares the background's message with its `token_play`; the layout carries that id as the
    /// background's `MD_Data_Message`). `None` for a background with no such item.
    fade_done: Option<u32>,
}

/// The index of `field` in [`StoryView::fields`].
fn index(field: StoryField) -> usize {
    match field {
        StoryField::First => 0,
        StoryField::Second => 1,
    }
}

impl StoryView {
    /// `gmStoryUI::gmStoryUI`'s roots, from the gameplay layout, hidden. `None` when the layout
    /// lacks the background or a field.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>) -> Option<Self> {
        let background = tree.create_root(Arc::clone(gameplay), tokenize(BACKGROUND))?;
        // The story's backdrop covers the display: it is not a window.
        tree.set_overlay_root(background);
        tree.set_visible(background, false);
        let mut fields = Vec::with_capacity(2);
        for (name, frame) in FIELD_NAMES.iter().zip(FRAME_STATES) {
            let root = tree.create_root(Arc::clone(gameplay), tokenize(name))?;
            tree.set_visible(root, false);
            fields.push(FieldElements {
                root,
                image: tree.child_recursive(root, tokenize(IMAGE_FIELD)),
                text: tree.child_recursive(root, tokenize(TEXT_FIELD)),
                frame: tokenize(frame),
            });
        }
        let [first, second]: [FieldElements; 2] = fields.try_into().ok()?;
        let fade_done = tree.media_messages(background).first().copied();
        Some(Self {
            background,
            fields: [first, second],
            shown: false,
            started: false,
            fade_done,
        })
    }

    /// Whether a story is on the screen.
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// `gmStoryUI::gmStoryUI` on the screen: the background and the two fields show and the
    /// background's media (its fade) starts from the top.
    pub fn begin(&mut self, tree: &mut UiTree) {
        self.show_roots(tree);
        tree.restart_media(self.background);
    }

    /// Whether the background fades by its media, so `token_play` reports the fade's end. A
    /// background without media has no fade to wait for.
    pub fn fades_by_media(&self) -> bool {
        self.fade_done.is_some()
    }

    /// Whether `message` is the background's fade-done message.
    pub fn is_fade_done(&self, message: &UiMessage) -> bool {
        message.element == Some(self.background) && self.fade_done == Some(message.message)
    }

    /// The background and the story fields show, once.
    fn show_roots(&mut self, tree: &mut UiTree) {
        if !self.shown {
            self.shown = true;
            tree.set_visible(self.background, true);
            for field in &self.fields {
                tree.set_visible(field.root, true);
            }
        }
    }

    /// `gmStoryUI::Play` for one keyframe: the background and the story fields show, the image
    /// goes into the active field (the flip and its fade on a new image), the subtitle into its
    /// text, rendered by `render` (its string-table entry, or its literal). A keyframe with
    /// neither leaves the fields as they are.
    pub fn show(
        &mut self,
        tree: &mut UiTree,
        show: &StoryShow,
        render: &dyn Fn(&StringInfo) -> String,
    ) {
        self.show_roots(tree);
        let active = index(show.field);
        let other = 1 - active;
        if show.image_changed || !self.started {
            self.started = true;
            tree.set_state(self.fields[active].root, self.fields[active].frame);
            tree.set_state(self.fields[other].root, STATELESS);
        }
        let field = self.fields[active];
        if let (Some(image), Some(element)) = (show.image, field.image) {
            tree.add_image_to_state(element, image, IMAGE_DRAW_MODE, crate::token::STATELESS);
            tree.set_visible(element, true);
        }
        if let (Some(subtitle), Some(element)) = (&show.subtitle, field.text) {
            tree.set_text(element, &render(subtitle));
        }
    }

    /// `gmStoryUI::~gmStoryUI`'s part on the screen: the roots hide, the images and subtitles
    /// clear, and both fields return to `stateless` for the next story.
    /// `gmStoryUI`'s frame events in order: a keyframe shows, the slot's begin and hide. Returns
    /// one flag per begin (`false`, the gameplay panels leave) and hide (`true`, they return), in
    /// order; the other events are the caller's.
    pub fn frame_events(
        &mut self,
        tree: &mut UiTree,
        events: Vec<StoryEvent>,
        strings: Option<&UiStringTables>,
    ) -> Vec<bool> {
        let mut gameplay = Vec::new();
        for event in events {
            match event {
                StoryEvent::Begin => {
                    self.begin(tree);
                    gameplay.push(false);
                }
                StoryEvent::Show(show) => {
                    self.show(tree, &show, &|info| subtitle_text(strings, info));
                }
                StoryEvent::Hide => {
                    self.hide(tree);
                    gameplay.push(true);
                }
                StoryEvent::StopAudio
                | StoryEvent::MouseLook(_)
                | StoryEvent::PlayScenesDone(_) => {}
            }
        }
        gameplay
    }

    pub fn hide(&mut self, tree: &mut UiTree) {
        self.shown = false;
        self.started = false;
        tree.set_visible(self.background, false);
        for field in self.fields {
            if let Some(image) = field.image {
                tree.clear_image(image);
            }
            if let Some(text) = field.text {
                tree.clear_text(text);
            }
            tree.set_state(field.root, STATELESS);
            tree.set_visible(field.root, false);
        }
    }
}

/// A subtitle's text: its `StringInfo` rendered on the loaded string tables. Before the tables
/// load, a literal is its own text and a table entry has none.
pub fn subtitle_text(strings: Option<&UiStringTables>, info: &StringInfo) -> String {
    match strings {
        Some(strings) => {
            let info = crate::chat_text::render_info(info);
            String::from_utf16_lossy(&ac2_db::get_string(&strings.tables, &info, false))
        }
        None => info.literal.clone().unwrap_or_default(),
    }
}
