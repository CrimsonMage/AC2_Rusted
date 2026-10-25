//! `gm3DTextUI`'s targeting reticle: four `TargettingReticle` roots framing the selected object
//! on the screen, its name (`SelectionName`) above them and its meters (`SelectionStuff`) under
//! the name, or `OffScreenIndicator` at the screen's edge when the object is behind the camera.
//!
//! `UpdateReticleInternal`: a selection farther than `MaxSelectionDistance` (planar) is cleared.
//! The object's box, averaged over the last `AverageBBoxFrames` frames (`AverageBoundingBox`),
//! is projected corner by corner (`ConvertObjectToWindow`) and its screen extent converted to
//! pixels (`SceneTool::TransformClipPointToPixel`). The reticle corners go outside that extent
//! (top-left, bottom-left, top-right, bottom-right, rotated 0, 90, 180 and 270 degrees), kept on
//! the screen below what the name and meters need and above what the bottom precludes; the name
//! is centred above the top and the meters between the name and the top. Without a name, the
//! reticle hides (`UpdateReticle`).
//!
//! Not modelled yet: the off-screen indicator (hidden), the radar-look colour of the name, the
//! meters (`DoStuffMeter`), damage text and the name plates over other objects.

use std::collections::VecDeque;
use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::token::tokenize;

/// The selected object for one frame.
pub struct ReticleTarget<'a> {
    pub id: u64,
    /// The object's bounding box in its own space (min, max).
    pub bounds: ([f32; 3], [f32; 3]),
    /// `SceneTool::TransformVector`: an object-space point to normalised device coordinates
    /// (x right, y up) and whether it is in front of the camera.
    pub project: &'a dyn Fn([f32; 3]) -> ([f32; 2], bool),
    /// Its planar distance from the player (`Position::localtolocal`), when it has a cell.
    pub planar_distance: Option<f32>,
}

/// What a frame's reticle asks of the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReticleOutcome {
    /// `gmCWInterface::SetSelection(INVALID_IID)`: the selection went out of range.
    pub deselect: bool,
}

/// `gm3DTextUI`'s reticle state.
#[derive(Debug, Clone)]
pub struct Reticle {
    corners: [usize; 4],
    off_screen: Option<usize>,
    name: usize,
    stuff: usize,
    average_frames: usize,
    max_selection_distance: i32,
    boxes: VecDeque<[f32; 6]>,
    current: Option<u64>,
    has_name: bool,
    has_info: bool,
    /// `m_precludedFromTop` / `m_precludedFromBottom` (`CM_UI::SendNotice_PrecludeFrom*`).
    pub precluded_top: i32,
    pub precluded_bottom: i32,
}

fn int_attribute(tree: &UiTree, index: usize, name: &str) -> Option<i32> {
    match tree.get(index)?.attribute(tokenize(name)) {
        Some(AttributeValue::Int(value)) => Some(*value),
        _ => None,
    }
}

/// `SceneTool::TransformClipPointToPixel`.
fn clip_to_pixel(clip: [f32; 2], width: i32, height: i32) -> (i32, i32) {
    let (w, h) = (f64::from(width), f64::from(height));
    let x = ((1.0 / w + f64::from(clip[0]) + 1.0) * w * 0.5) as i32;
    let y = ((1.0 - (1.0 / h + f64::from(clip[1]))) * h * 0.5) as i32;
    (x, y)
}

impl Reticle {
    /// `gm3DTextUI::InitReticle`: four reticle corners, the off-screen indicator, the name and
    /// the meters from the gameplay layout, all hidden.
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> Option<Self> {
        let mut root = |name: &str| tree.create_root(Arc::clone(layout), tokenize(name));
        let corners = [
            root("TargettingReticle")?,
            root("TargettingReticle")?,
            root("TargettingReticle")?,
            root("TargettingReticle")?,
        ];
        let off_screen = root("OffScreenIndicator");
        let name = root("SelectionName")?;
        let stuff = root("SelectionStuff")?;
        let reticle = Self {
            average_frames: int_attribute(tree, corners[0], "AverageBBoxFrames")
                .unwrap_or(1)
                .max(1) as usize,
            max_selection_distance: int_attribute(tree, corners[0], "MaxSelectionDistance")
                .unwrap_or(i32::MAX / 2),
            corners,
            off_screen,
            name,
            stuff,
            boxes: VecDeque::new(),
            current: None,
            has_name: false,
            has_info: false,
            precluded_top: 0,
            precluded_bottom: 0,
        };
        for root in reticle.objects() {
            tree.show_object(root, false);
        }
        Some(reticle)
    }

    fn objects(&self) -> impl Iterator<Item = usize> + '_ {
        self.corners
            .iter()
            .copied()
            .chain(self.off_screen)
            .chain([self.name, self.stuff])
    }

    /// `gm3DTextUI::RecvNotice_SelectionChanged` / `CheckName`: the new selection's name in the
    /// name element, sized to it; the averaged boxes restart.
    pub fn selection_changed(&mut self, tree: &mut UiTree, id: Option<u64>, name: Option<&str>) {
        self.current = id;
        self.has_info = false;
        self.has_name = false;
        self.boxes.clear();
        if let (Some(_), Some(name)) = (id, name) {
            tree.set_text(self.name, name);
            if let Some((width, height)) = tree.text_size_with_margins(self.name, name) {
                tree.resize_to(self.name, width, height);
            }
            self.has_name = true;
        }
    }

    pub fn current(&self) -> Option<u64> {
        self.current
    }

    /// `gm3DTextUI::AverageBoundingBox`.
    fn average(&mut self, bounds: ([f32; 3], [f32; 3])) -> ([f32; 3], [f32; 3]) {
        while self.boxes.len() >= self.average_frames {
            self.boxes.pop_front();
        }
        let (lo, hi) = bounds;
        self.boxes
            .push_back([lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]);
        let mut sum = [0.0_f64; 6];
        for b in &self.boxes {
            for (s, v) in sum.iter_mut().zip(b) {
                *s += f64::from(*v);
            }
        }
        let inv = 1.0 / self.boxes.len() as f64;
        let v = sum.map(|s| (s * inv) as f32);
        ([v[0], v[1], v[2]], [v[3], v[4], v[5]])
    }

    /// `gm3DTextUI::UpdateReticle` for this frame (`hidden` in portal space).
    pub fn update(
        &mut self,
        tree: &mut UiTree,
        target: Option<ReticleTarget<'_>>,
        hidden: bool,
    ) -> ReticleOutcome {
        let mut outcome = ReticleOutcome::default();
        let shown = if hidden {
            None
        } else {
            self.update_internal(tree, target, &mut outcome)
        };
        if shown.is_none() {
            for root in self.objects().collect::<Vec<_>>() {
                tree.show_object(root, false);
            }
        }
        outcome
    }

    /// `gm3DTextUI::UpdateReticleInternal`; `Some` when the reticle shows.
    fn update_internal(
        &mut self,
        tree: &mut UiTree,
        target: Option<ReticleTarget<'_>>,
        outcome: &mut ReticleOutcome,
    ) -> Option<()> {
        let target = target.filter(|t| Some(t.id) == self.current)?;
        if let Some(distance) = target.planar_distance {
            let max = f64::from(self.max_selection_distance);
            if max * max < f64::from(distance) * f64::from(distance) {
                outcome.deselect = true;
                return None;
            }
        }
        // `ConvertObjectToWindow`.
        let (lo, hi) = self.average(target.bounds);
        let mut in_front = false;
        let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
        for i in 0..8 {
            let corner = [
                if i & 1 == 0 { lo[0] } else { hi[0] },
                if i & 2 == 0 { lo[1] } else { hi[1] },
                if i & 4 == 0 { lo[2] } else { hi[2] },
            ];
            let (clip, front) = (target.project)(corner);
            in_front |= front;
            for axis in 0..2 {
                min[axis] = min[axis].min(clip[axis]);
                max[axis] = max[axis].max(clip[axis]);
            }
        }
        if !in_front {
            // The off-screen indicator is not modelled; nothing shows.
            return None;
        }
        let (display_w, display_h) = tree.display;
        let (x0, bottom) = clip_to_pixel(min, display_w, display_h);
        let (x1, top) = clip_to_pixel(max, display_w, display_h);
        let size = |tree: &UiTree, index: usize| {
            tree.get(index).map(|e| (e.rect.width(), e.rect.height()))
        };
        let (w, h) = size(tree, self.corners[0])?;
        let (name_w, name_h) = size(tree, self.name)?;
        let (stuff_w, stuff_h) = size(tree, self.stuff)?;
        let max_x = display_w - w;
        let bottom_limit = display_h - self.precluded_bottom - h;
        let top_limit = self.precluded_top
            + if self.has_name { name_h } else { 0 }
            + if self.has_info { stuff_h } else { 0 };
        let mut xs = [x0 - w, x0 - w, x1, x1];
        let mut ys = [top - h, bottom, top - h, bottom];
        for i in 0..4 {
            xs[i] = xs[i].clamp(0, max_x.max(0));
            ys[i] = if ys[i] < top_limit || bottom_limit < ys[i] {
                if ys[i] < top_limit {
                    top_limit
                } else {
                    bottom_limit
                }
            } else {
                ys[i]
            };
            tree.show_object(self.corners[i], true);
            tree.set_object_rotation(self.corners[i], 90 * i as i32);
            tree.set_object_position(self.corners[i], xs[i], ys[i]);
        }
        if !self.has_name {
            return None;
        }
        // The name centred above the corners' top, the meters under the name.
        let half = (xs[2] - xs[0] + w) / 2;
        let name_x = xs[0] - name_w / 2 + half;
        let mut name_y = ys[0] - name_h;
        if self.has_info {
            name_y -= stuff_h;
            let stuff_x = xs[0] - stuff_w / 2 + half;
            tree.set_object_position(self.stuff, stuff_x, name_y + name_h);
            tree.show_object(self.stuff, true);
        } else {
            tree.show_object(self.stuff, false);
        }
        tree.set_object_position(self.name, name_x, name_y);
        tree.show_object(self.name, true);
        if let Some(off_screen) = self.off_screen {
            tree.show_object(off_screen, false);
        }
        Some(())
    }
}
