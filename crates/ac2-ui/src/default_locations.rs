//! Proportional default places for windows nobody has placed (a modern addition, AGENTS.md: the
//! window resizes freely). Retail applied a location only where the player's profile held one
//! (`UIMainFramework::ApplyLocations`, from `gmGamePlayUI::ListenToMessage` `0xffff0007` after
//! `UIElementManager::RefreshEvent`, and `RecvNotice_ReloadOptions`): a window that saves its
//! location (`save_loc`) and had none stayed where `UIElement::UpdateSizeAndPosition` left it, and
//! for the layouts' fixed-edge windows (the radar, the chat windows, most panels) that is their
//! authored pixel place, so on a bigger display they sat near the top left. Retail did record each
//! such root's clip-space box when it was made (`UIElementManager::CreateRootElementByDataID`,
//! `UIElement::SetDefaultLocation` into `m_defaultLocation`) but never read it back.
//!
//! Here that box is the default: a `save_loc` root with fixed edges and no saved place of the
//! player's or of the server's is put where its authored box lies in clip space
//! (`SceneTool::TransformPixelToClipPoint` over the layout's display), on the current display
//! (`ClipToWindow`), as `UIElement::ApplyLocation` places a saved one: it keeps its pixel size
//! unless it allows resizing. A window anything else has moved since (the modern style's own
//! placement, a drag) is left where it is.

use crate::element::UiTree;
use crate::token::ALLOW_RESIZE;
use crate::window_locations::{clip_to_pixel, pixel_to_clip};

impl UiTree {
    /// The windows nothing else has moved: at their authored place or where the last default put
    /// them. Taken before the display's change places the roots again.
    pub(crate) fn default_candidates(&self) -> Vec<usize> {
        self.roots
            .iter()
            .copied()
            .filter(|&index| {
                if !self.is_window_root(index) || !self.saves_location(index) {
                    return false;
                }
                let el = self.el(index);
                let (authored, _) = Self::position_from_desc(&el.reference, el.state);
                let now = self.screen_box(index);
                let ours = self
                    .default_places
                    .iter()
                    .find(|(i, _)| *i == index)
                    .map(|&(_, at)| at);
                (now.x0, now.y0) == (authored.x0, authored.y0) || Some((now.x0, now.y0)) == ours
            })
            .collect()
    }

    /// Places the candidate windows that have no saved place by their authored clip box.
    pub(crate) fn apply_default_locations(&mut self, candidates: &[usize]) {
        for &root in candidates {
            self.apply_default_location(root);
        }
    }

    fn apply_default_location(&mut self, index: usize) {
        let el = self.el(index);
        if el.reference.edges != [0; 4] {
            return;
        }
        let key = (el.layout.did, el.id());
        if self
            .saved_positions
            .iter()
            .any(|p| (p.layout, p.element) == key)
            || self
                .server_locations
                .iter()
                .any(|s| (s.layout, s.element) == key)
        {
            return;
        }
        let (authored, _) = Self::position_from_desc(&el.reference, el.state);
        let layout = (i32::from(el.layout.width), i32::from(el.layout.height));
        let (cx0, cy0) = pixel_to_clip(authored.x0, authored.y0, layout.0, layout.1);
        let (cx1, cy1) = pixel_to_clip(authored.x1, authored.y1, layout.0, layout.1);
        let (width, height) = self.display;
        let (x0, y0) = clip_to_pixel(cx0, cy0, width, height);
        let (x1, y1) = clip_to_pixel(cx1, cy1, width, height);
        if self.bool_attribute(index, ALLOW_RESIZE) {
            self.resize_to(index, x1 - x0 + 1, y1 - y0 + 1);
        }
        self.move_to(index, x0, y0);
        let placed = self.screen_box(index);
        self.default_places.retain(|(i, _)| *i != index);
        self.default_places.push((index, (placed.x0, placed.y0)));
        self.moved = true;
        self.changed = true;
    }
}
