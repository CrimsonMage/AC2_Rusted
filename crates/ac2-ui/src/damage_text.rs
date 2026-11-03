//! `gm3DTextUI`'s floating damage numbers. `RecvNotice_Do3DDamage` makes a `DamageTextTemplate`
//! text in the blob's colour and records it with the object it is over and the time it started
//! (`Timer::cur_time`). The text is measured (`InqSizewMargins`), the element and its surface
//! take that size (`UIElement_Text::ResizeTo`, `UISurfaceObject::ResizeSurface`), and the surface
//! is shown at the size times the blob's scale (`SetScreenPosition(x, y, w * scale, h * scale)`,
//! filtered: `SetMaterialFiltering(1)`), so the lettering grows with the scale. Each frame
//! (`UpdateDamage`) a text's progress is the time since it started over its duration: past the
//! end it is destroyed, otherwise it rises by its shown height times `FloatyTextTravel` times the
//! progress (above the selection's reticle and name when the object is selected), fades to
//! `1 - progress` and is placed over the object's averaged box like a name plate
//! (`CalcScreenPosition`), clear of the damage texts placed before it. The placement uses the
//! shown size (`GetObjectViewWidth` / `GetObjectViewHeight`). Hidden while the player is in
//! portal space. `CleanUp` destroys them all.
//!
//! The template (`gameplay.uil`) is a centred `impact_24` text with an outline. Retail stretched
//! its small bitmap; here the root is composed at the scale instead
//! (`UiTree::set_root_render_scale`, the glyphs grown from the font's coverage as the scaled
//! pregame screens are), and the client's damage text size preference
//! (`damage_text_pref`, AGENTS.md) multiplies the blob's scale.
//!
//! Legions 7160 has it too: its `gm3DTextUI` keeps `UpdateDamage` and the damage array
//! (`SmartArray<DamageTextInfo>::Reset`, `DamageTextInfo::operator=`), and the DVD's script
//! library calls `CM_Physics::SendNotice_Do3DDamage` (efun 0x008D0001) in two places. The notice
//! handler (Legions 0x0042AF10, unmatched in the index) is admin 7148's 0x0042B150 instruction
//! for instruction.

use std::collections::VecDeque;
use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::placement::{place_box, Box2D};
use crate::reticle::{average_box, clip_to_pixel, object_to_window};
use crate::token::tokenize;
use ac2_weenie::DamageText;

/// `DamageTextTemplate`'s font colour attributes, red, green, blue.
const CHANNELS: [&str; 3] = ["red", "green", "blue"];

/// An object a damage text may be over, for this frame.
pub struct DamagePlace<'a> {
    pub id: u64,
    /// The object's box in its own space (min, max).
    pub bounds: ([f32; 3], [f32; 3]),
    /// Object space to clip space (x right, y up) and whether in front of the camera.
    pub project: &'a dyn Fn([f32; 3]) -> ([f32; 2], bool),
}

/// `gm3DTextUI::DamageTextInfo`.
#[derive(Debug, Clone)]
struct Damage {
    root: usize,
    object: u64,
    /// `startTime` (`Timer::cur_time` when the notice came).
    start: f64,
    /// `duration`, the text's life in seconds.
    duration: f32,
    /// `deque<BBox>`: the object's boxes, averaged over the last frames.
    boxes: VecDeque<[f32; 6]>,
    /// The surface's shown size (`m_view_width`, `m_view_height`): the measured size times the
    /// scale, truncated.
    view: (i32, i32),
}

/// `gm3DTextUI`'s damage texts, in `m_damageTextArray` order.
#[derive(Debug, Clone)]
pub struct DamageTexts {
    layout: Arc<LayoutDesc>,
    entries: Vec<Damage>,
    /// `m_averageBBoxFrames`, `m_maxNamePlacementAttempts`, `m_floatyTextTravel`.
    average_frames: usize,
    max_attempts: i32,
    travel: i32,
    /// The damage text size preference, times the blob's scale (1 is retail).
    size: f64,
}

impl DamageTexts {
    /// `gm3DTextUI::Init`'s part: none yet, with the reticle's tuning.
    pub fn create(
        layout: &Arc<LayoutDesc>,
        average_frames: usize,
        max_attempts: i32,
        travel: i32,
    ) -> Self {
        Self {
            layout: Arc::clone(layout),
            entries: Vec::new(),
            average_frames: average_frames.max(1),
            max_attempts,
            travel,
            size: 1.0,
        }
    }

    /// The damage text size preference (`damage_text_pref`): new texts are drawn at the blob's
    /// scale times `size`. 1 is retail's size.
    pub fn set_size(&mut self, size: f64) {
        self.size = if size.is_finite() && size > 0.0 {
            size
        } else {
            1.0
        };
    }

    /// The damage text size preference in use.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// A damage text's shown size (`GetObjectViewWidth`, `GetObjectViewHeight`).
    pub fn view_size(&self, object: u64) -> Option<(i32, i32)> {
        self.entries
            .iter()
            .find(|e| e.object == object)
            .map(|e| e.view)
    }

    /// The objects with a damage text in the air, in array order.
    pub fn objects(&self) -> Vec<u64> {
        self.entries.iter().map(|e| e.object).collect()
    }

    /// The number of damage texts in the air.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no damage text is in the air.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `gm3DTextUI::RecvNotice_Do3DDamage`: a text over `object`, started at `now`. It stays
    /// hidden until `update` places it.
    pub fn add(&mut self, tree: &mut UiTree, object: u64, text: &DamageText, now: f64) -> bool {
        let Some(root) = tree.create_root(Arc::clone(&self.layout), tokenize("DamageTextTemplate"))
        else {
            return false;
        };
        // A damage number is the world's overlay, not a window.
        tree.set_overlay_root(root);
        for (channel, value) in CHANNELS.iter().zip(text.color) {
            tree.set_float(root, tokenize(&format!("font_color_{channel}")), value);
        }
        // `InqSizewMargins`; the element and its surface take that size (`ResizeTo`,
        // `ResizeSurface`), and the surface is shown at it times the scale (`_ftol2`).
        let (width, height) = tree
            .text_size_with_margins(root, &text.text)
            .unwrap_or_else(|| {
                let rect = tree.get(root).map(|e| e.rect()).unwrap_or_default();
                (rect.width(), rect.height())
            });
        tree.resize_to(root, width, height);
        tree.set_text(root, &text.text);
        let scale = f64::from(text.scale) * self.size;
        let view = (
            (f64::from(width) * scale) as i32,
            (f64::from(height) * scale) as i32,
        );
        // Composed at the scale rather than stretched (see the module docs).
        tree.set_root_render_scale(root, Some(scale));
        tree.show_object(root, false);
        self.entries.push(Damage {
            root,
            object,
            start: now,
            duration: text.duration,
            boxes: VecDeque::new(),
            view,
        });
        true
    }

    /// `gm3DTextUI::CleanUp`: every damage text destroyed.
    pub fn clear(&mut self, tree: &mut UiTree) {
        for entry in self.entries.drain(..) {
            tree.remove_root(entry.root);
        }
    }

    /// `gm3DTextUI::UpdateDamage` for this frame at `now`. `selection` is the selected object
    /// with `selection_offset` the reticle's rise for it (see [`Reticle::selection_offset`]),
    /// `precluded` the space kept at the top and bottom of the screen. Hidden in portal space.
    ///
    /// [`Reticle::selection_offset`]: crate::Reticle::selection_offset
    #[allow(clippy::too_many_arguments)] // the frame's inputs, as `UpdateDamage` reads them
    pub fn update(
        &mut self,
        tree: &mut UiTree,
        places: &[DamagePlace<'_>],
        selection: (u64, i32),
        precluded: (i32, i32),
        now: f64,
        hidden: bool,
    ) {
        let (display_w, display_h) = tree.display;
        let screen = Box2D {
            x0: 0,
            y0: 0,
            x1: display_w - 1,
            y1: display_h - 1,
        };
        let mut placed: Vec<Box2D> = Vec::new();
        let mut index = 0;
        while index < self.entries.len() {
            let root = self.entries[index].root;
            tree.show_object(root, false);
            if hidden {
                index += 1;
                continue;
            }
            let entry = &mut self.entries[index];
            // Progress: over the duration it is spent (`fVar15`), then as a float (`per`).
            let ratio = (now - entry.start) / f64::from(entry.duration);
            if ratio > 1.0 {
                self.destroy(tree, index);
                continue;
            }
            let per = (ratio as f32).max(0.0);
            if per >= 1.0 {
                self.destroy(tree, index);
                continue;
            }
            let Some(place) = places.iter().find(|p| p.id == entry.object) else {
                index += 1;
                continue;
            };
            // `GetObjectViewWidth`, `GetObjectViewHeight`: the shown size. A blob without a
            // scale is shown at no size: nothing is drawn.
            let (width, height) = entry.view;
            if width <= 0 || height <= 0 {
                index += 1;
                continue;
            }
            // The rise: height * FloatyTextTravel * progress, truncated (`_ftol2`), plus the
            // selection's reticle when this is the selected object.
            let mut offset = (f64::from(height) * f64::from(self.travel) * f64::from(per)) as i32;
            if entry.object == selection.0 {
                offset += selection.1;
            }
            // `CalcScreenPosition`.
            let (lo, hi) = average_box(&mut entry.boxes, self.average_frames, place.bounds);
            let Some((min, max)) = object_to_window(place.project, lo, hi) else {
                index += 1;
                continue;
            };
            let (wx0, _) = clip_to_pixel(min, display_w, display_h);
            let (wx1, top) = clip_to_pixel(max, display_w, display_h);
            let x = (wx0 + (wx1 - wx0) / 2).clamp(0, display_w);
            let y = (top - (offset + height)).clamp(precluded.0, display_h - precluded.1);
            tree.set_object_opacity(root, 1.0 - per);
            let x0 = (f64::from(x) - f64::from(width as u32) * 0.5) as i32;
            if let Some(here) = place_box(
                &screen,
                Box2D::new(x0, y, width, height),
                &placed,
                self.max_attempts,
            ) {
                placed.push(here);
                tree.set_object_position(root, here.x0, here.y0);
                tree.show_object(root, true);
            }
            index += 1;
        }
    }

    /// `SmartArray::RemoveOrderedByIndex` with the element destroyed.
    fn destroy(&mut self, tree: &mut UiTree, index: usize) {
        let entry = self.entries.remove(index);
        tree.remove_root(entry.root);
    }
}
