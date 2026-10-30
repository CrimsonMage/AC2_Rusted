//! The modern chat's toast lane (AGENTS.md: the HUD's two styles): each tell the player gets
//! also shows for a few seconds as a toast in a lane under the vitals, fading at its end, so a
//! tell is seen while the chat windows are hidden or busy. The tell still goes to the windows as
//! retail's did; the lane only shows it and takes no pointer.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_inventory::keep;
use crate::token::{tokenize, MOUSE_INVISIBLE};
use crate::window_parts::{set_button_label, text_button};

/// The most toasts shown at once, how long one shows and the fade at its end (seconds).
const LANE: usize = 3;
const SHOW: f64 = 3.0;
const FADE: f64 = 0.6;
/// The lane's width, a toast's height and the gap between toasts.
const WIDTH: i32 = 290;
const HEIGHT: i32 = 24;
const GAP: i32 = 4;

/// The lane: a root per toast slot (its glass and its text), and the tells waiting or shown.
#[derive(Debug, Clone)]
pub(crate) struct Toasts {
    slots: Vec<(usize, Option<usize>)>,
    shown: VecDeque<(String, Instant)>,
    laid_out: Vec<String>,
}

impl Toasts {
    /// The lane's slots made as roots from `window`'s layout (copying `template`, a plain field)
    /// with a text from the gameplay layout's text button; hidden.
    pub(crate) fn make(
        tree: &mut UiTree,
        window: usize,
        template: usize,
        gameplay: Option<&Arc<LayoutDesc>>,
    ) -> Self {
        let slots = (0..LANE)
            .filter_map(|i| {
                let layout = Arc::clone(&tree.el(window).layout);
                let mut desc = (*tree.el(template).reference).clone();
                desc.id = tokenize(&format!("ModernChatToast{i}"));
                desc.states.clear();
                desc.edges = [0; 4];
                let root = tree.create_element(layout, Arc::new(desc), true)?;
                tree.el_mut(root)
                    .attributes
                    .remove(&crate::token::ALPHA_VALUE);
                tree.set_z_level(root, 2);
                tree.add_root(root);
                tree.set_overlay_root(root);
                tree.set_image(root, art::CHAT_POPOVER, crate::compose::DRAW_OVER);
                tree.set_int(root, crate::compose::NINE_SLICE, art::CHAT_BORDER);
                tree.set_bool(root, MOUSE_INVISIBLE, true);
                tree.resize_to(root, WIDTH, HEIGHT);
                let text = gameplay.and_then(|g| {
                    let label = text_button(
                        tree,
                        g,
                        root,
                        &format!("ModernChatToastText{i}"),
                        "",
                        (8, 2, WIDTH - 16),
                    )?;
                    keep(tree, label);
                    tree.clear_image(label);
                    tree.set_bool(label, MOUSE_INVISIBLE, true);
                    for child in tree.children(label).to_vec() {
                        tree.set_bool(child, MOUSE_INVISIBLE, true);
                        if let Some(t) = tree.el_mut(child).text.as_mut() {
                            t.justification = crate::text::Justification::Left;
                        }
                    }
                    Some(label)
                });
                tree.set_visible(root, false);
                Some((root, text))
            })
            .collect();
        Self {
            slots,
            shown: VecDeque::new(),
            laid_out: Vec::new(),
        }
    }

    /// A tell (`text`, as the chat shows it) into the lane; the oldest leaves when it is full.
    pub(crate) fn push(&mut self, text: &str) {
        let text = text.trim_end().to_string();
        if text.is_empty() {
            return;
        }
        if self.shown.len() == LANE {
            self.shown.pop_front();
        }
        self.shown.push_back((text, Instant::now()));
    }

    /// The lane laid out from `(x, y)` (screen), newest at the bottom; each toast's opacity its
    /// fade; the expired ones gone. Run every frame.
    pub(crate) fn tick(&mut self, tree: &mut UiTree, (x, y): (i32, i32)) {
        while self
            .shown
            .front()
            .is_some_and(|(_, at)| at.elapsed().as_secs_f64() >= SHOW)
        {
            self.shown.pop_front();
        }
        let texts: Vec<String> = self.shown.iter().map(|(t, _)| t.clone()).collect();
        let relay = texts != self.laid_out;
        for (i, &(root, label)) in self.slots.iter().enumerate() {
            let Some((text, at)) = self.shown.get(i) else {
                if tree.el(root).visible {
                    tree.set_visible(root, false);
                }
                continue;
            };
            if relay {
                if let Some(label) = label {
                    set_button_label(tree, label, text);
                }
                tree.set_object_position(root, x, y + i as i32 * (HEIGHT + GAP));
                tree.move_to(root, x, y + i as i32 * (HEIGHT + GAP));
                tree.set_visible(root, true);
            }
            let left = SHOW - at.elapsed().as_secs_f64();
            let opacity = (left / FADE).clamp(0.0, 1.0) as f32;
            tree.set_object_opacity(root, (opacity * 20.0).round() / 20.0);
        }
        self.laid_out = texts;
    }
}
