//! `gm3DTextUI`'s floating damage numbers. `RecvNotice_Do3DDamage` makes a `DamageTextTemplate`
//! text in the blob's colour and scale, and records it with the object it is over and the time
//! it started (`Timer::cur_time`). Each frame (`UpdateDamage`) a text's progress is the time
//! since it started over its duration: past the end it is destroyed, otherwise it rises by its
//! height times `FloatyTextTravel` times the progress (above the selection's reticle and name
//! when the object is selected), fades to `1 - progress` and is placed over the object's
//! averaged box like a name plate (`CalcScreenPosition`), clear of the damage texts placed
//! before it. Hidden while the player is in portal space. `CleanUp` destroys them all.
//!
//! Legions 7160 has it too: its `gm3DTextUI` keeps `UpdateDamage` and the damage array
//! (`SmartArray<DamageTextInfo>::Reset`, `DamageTextInfo::operator=`), and the DVD's script
//! library calls `CM_Physics::SendNotice_Do3DDamage` (efun 0x008D0001) in two places. The notice
//! handler itself has no Legions match in the index, so its body follows admin 7148.

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
        }
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
        // `InqSizewMargins`, then the surface scaled by the blob's scale (truncated).
        if let Some((width, height)) = tree.text_size_with_margins(root, &text.text) {
            let scale = f64::from(text.scale);
            let (w, h) = (
                (f64::from(width) * scale) as i32,
                (f64::from(height) * scale) as i32,
            );
            tree.resize_to(root, w, h);
        }
        tree.set_text(root, &text.text);
        tree.show_object(root, false);
        self.entries.push(Damage {
            root,
            object,
            start: now,
            duration: text.duration,
            boxes: VecDeque::new(),
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
            let (width, height) = tree
                .get(root)
                .map_or((0, 0), |e| (e.rect.width(), e.rect.height()));
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
