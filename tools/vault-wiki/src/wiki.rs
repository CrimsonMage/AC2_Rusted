//! The guide window alone: the client's gameplay HUD built in one look
//! (`ac2_ui::build_styled_hud`, so the window's frame, tabs, buttons and scrollbars are the ones
//! the client shows in that look), every root but the guide's hidden, the guide window filling
//! the display and pinned there. Input is the client's own: the UI actions and typing the
//! `Controls` give go into the tree as `HudView::input` / `text_input` feed them, and the tree's
//! messages go to `GuideWindow::message`.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_guide::MapCoord;
use ac2_ui::guide_window::{GuideRequest, GuideWindow};
use ac2_ui::{HudStyle, Modifiers, UiMessage, UiPixels, UiSurface, UiTree};

/// What the guide asked for that the application carries out.
pub enum Outcome {
    /// The page's chat line (`Title [wiki:slug]`), for the clipboard.
    Share(String),
    /// A map coordinate pressed on a page.
    PointAt(MapCoord),
}

pub struct Wiki {
    pub style: HudStyle,
    tree: UiTree,
    images: HashMap<u32, UiPixels>,
    guide: GuideWindow,
}

impl Wiki {
    /// The guide window in `style` on a `width` x `height` display, opened (at `slug` if given).
    pub fn build(
        store: &Arc<ResourceStore>,
        style: HudStyle,
        width: u32,
        height: u32,
        slug: Option<&str>,
    ) -> Result<Self, String> {
        let (w, h) = (width as i32, height as i32);
        let hud = ac2_ui::build_styled_hud(store, w, h, style).map_err(|e| e.to_string())?;
        let mut tree = hud.tree;
        let mut guide = hud
            .guide
            .ok_or("the DATs' gameplay layout has no Book for the guide window")?;
        tree.set_text_clipboard(ac2_client::shared_clipboard());
        let mut images = ac2_ui::load_ui_images(store, &tree).map_err(|e| e.to_string())?;
        // The modern style's drawn art (`modern_art`), as the client's HUD loader bakes it.
        if style == HudStyle::Modern {
            images.extend(ac2_ui::ac1_buttons::shipped_icons());
            let icons = ac2_ui::load_images(store, &ac2_ui::modern_art::SOURCE_ICONS)
                .map_err(|e| e.to_string())?;
            ac2_ui::modern_art::bake(&mut images, &icons);
        }
        let root = guide.root();
        guide.unbound_size(&mut tree);
        for other in tree.roots().to_vec() {
            if other != root {
                tree.set_visible(other, false);
            }
        }
        match slug {
            Some(slug) => guide.open_slug(&mut tree, slug),
            None => guide.open(&mut tree),
        }
        let mut wiki = Self {
            style,
            tree,
            images,
            guide,
        };
        wiki.resize(width, height);
        Ok(wiki)
    }

    /// The display and the window follow the application's size.
    pub fn resize(&mut self, width: u32, height: u32) {
        let (w, h) = (width as i32, height as i32);
        let root = self.guide.root();
        self.tree.set_display(w, h);
        self.tree.set_locked(root, false);
        self.tree.move_to(root, 0, 0);
        self.tree.resize_to(root, w, h);
        // Pinned: the window is the application, so it neither moves nor resizes by itself.
        self.tree.set_locked(root, true);
    }

    /// The shown page's slug, to open the same page after a rebuild.
    pub fn slug(&self) -> Option<String> {
        self.guide.slug()
    }

    /// Whether the window's close button closed it.
    pub fn closed(&self) -> bool {
        !self.guide.is_shown()
    }

    /// Whether a text field (the search) takes the typing.
    pub fn text_mode(&self) -> bool {
        self.tree.text_mode()
    }

    pub fn text_copy_focus(&self) -> bool {
        self.tree.text_copy_focus()
    }

    /// One frame of input and time: the UI actions, then the typing, then the tree's loop; the
    /// messages go to the guide window. Returns the messages (for the controls) and what the
    /// guide asked for.
    pub fn frame(
        &mut self,
        actions: &[(u32, f32, bool)],
        text_actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        modifiers: Modifiers,
        now: f64,
    ) -> (Vec<UiMessage>, Vec<Outcome>) {
        let tree = &mut self.tree;
        tree.set_shift(modifiers.shift);
        tree.set_alt(modifiers.alt);
        for &(action, extent, start) in actions {
            if start {
                tree.start_input(action, extent);
            } else {
                tree.stop_input(action);
            }
        }
        for &(action, extent) in text_actions {
            tree.text_action(action, extent, modifiers);
        }
        for &(ch, key) in characters {
            tree.character(ch, key, modifiers);
        }
        tree.global_loop(now);
        let mut all = Vec::new();
        let mut outcomes = Vec::new();
        loop {
            let messages = self.tree.take_messages();
            if messages.is_empty() {
                break;
            }
            for message in &messages {
                match self.guide.message(&mut self.tree, message).1 {
                    Some(GuideRequest::Share(text)) => outcomes.push(Outcome::Share(text)),
                    Some(GuideRequest::PointAt(at)) => outcomes.push(Outcome::PointAt(at)),
                    None => {}
                }
            }
            all.extend(messages);
        }
        self.guide.frame(&mut self.tree, &mut self.images);
        (all, outcomes)
    }

    /// The surfaces to draw, when anything changed since the last call.
    pub fn compose(&mut self) -> Option<Vec<UiSurface>> {
        if !self.tree.take_changed() {
            return None;
        }
        Some(self.tree.compose(&self.images))
    }
}

/// A coordinate as players write it (`12.3N, 45.6E`).
pub fn coord_text(at: MapCoord) -> String {
    let ns = if at.north < 0.0 { 'S' } else { 'N' };
    let ew = if at.east < 0.0 { 'W' } else { 'E' };
    format!("{:.1}{ns}, {:.1}{ew}", at.north.abs(), at.east.abs())
}

#[cfg(test)]
impl Wiki {
    /// The tab the window shows.
    pub fn tab(&self) -> ac2_guide::browse::Tab {
        self.guide.view().0
    }

    /// The page or item the window shows.
    pub fn view(&self) -> Option<ac2_ui::guide_window::View> {
        self.guide.view().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_guide::browse::Tab;
    use ac2_guide::Kind;
    use ac2_ui::{Controls, InputEvent};

    /// The application's input path on the real layouts (`AC2_DAT_DIR`, a baked guide): OS
    /// pointer and button events through the client's `Controls` press the Places tab and focus
    /// the search field, in both looks.
    #[test]
    fn presses_reach_the_guide_through_the_controls() {
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        if ac2_guide::baked().pages.is_empty() {
            return;
        }
        let store = Arc::new(ResourceStore::new(dataset));
        for style in [HudStyle::Modern, HudStyle::Classic] {
            let mut wiki = Wiki::build(&store, style, 1100, 760, None).unwrap();
            let mut controls = Controls::load_with(&store, None).unwrap().unwrap();
            controls.ui_attached = true;
            let mut now = 0.0;
            let mut step = |wiki: &mut Wiki, controls: &mut Controls, events: &[InputEvent]| {
                for &event in events {
                    now += 0.02;
                    controls.event(event, (now * 1000.0) as u32, now);
                }
                for _ in 0..3 {
                    now += 0.02;
                    let modifiers = controls.modifiers();
                    let actions = controls.take_ui_input();
                    let (texts, chars, _) = controls.take_text_input();
                    let (messages, _) = wiki.frame(&actions, &texts, &chars, modifiers, now);
                    controls.set_text_mode(wiki.text_mode(), (now * 1000.0) as u32, now);
                    controls.ui_messages(&messages, now);
                    controls.frame();
                }
            };
            let click = |x, y| {
                [
                    InputEvent::Cursor { x, y },
                    InputEvent::Button {
                        button: 0,
                        pressed: true,
                    },
                    InputEvent::Button {
                        button: 0,
                        pressed: false,
                    },
                ]
            };
            // Loaded (the guide decodes on a worker).
            for _ in 0..200 {
                step(&mut wiki, &mut controls, &[]);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert_eq!(wiki.tab(), Tab::Pages(Kind::Quest), "{style:?}");
            // The Places tab and the search field, where both looks put them.
            for event in click(145, 47) {
                step(&mut wiki, &mut controls, &[event]);
            }
            assert_eq!(wiki.tab(), Tab::Pages(Kind::Npc), "{style:?}");
            assert!(!wiki.text_mode());
            for event in click(150, 74) {
                step(&mut wiki, &mut controls, &[event]);
            }
            assert!(wiki.text_mode(), "{style:?}: the search takes the typing");
            // The wheel over the list scrolls it and opens nothing (each notch reaches the
            // tree as a press of its own; only the left button opens a row).
            let shown = wiki.view();
            step(
                &mut wiki,
                &mut controls,
                &[InputEvent::Cursor { x: 120, y: 300 }],
            );
            for _ in 0..6 {
                step(
                    &mut wiki,
                    &mut controls,
                    &[InputEvent::Wheel { delta: -120 }],
                );
            }
            assert_eq!(wiki.view(), shown, "{style:?}: the wheel opened a row");
            // A left click on a row opens it (a level band heading opens nothing, so a few
            // rows are tried).
            for y in [300, 318, 336, 354] {
                for event in click(120, y) {
                    step(&mut wiki, &mut controls, &[event]);
                }
                if wiki.view() != shown {
                    break;
                }
            }
            assert_ne!(
                wiki.view(),
                shown,
                "{style:?}: a click opens the row under it"
            );
        }
    }
}
