//! `gm3DTextUI`'s name plates: while the player shows object names (`ShowObjectNames`, toggled
//! with `ToggleNames`), every object with a radar look gets a `NameTextTemplate` root in its
//! look's colour, sized to its name (`AddName`). Each frame (`UpdateNames`) a plate shows over its
//! object when the object is not the selection, lies within the radar's range on the ground
//! plane and projects in front of the camera: centred over the object's averaged box
//! (`CalcScreenPosition`), kept clear of the selection's reticle and of the plates placed before
//! it by moving it off whatever it hits (`PlaceHere`, `MakeNewData`), up to
//! `MaxNamePlacementAttempts` times, and dropped when it leaves the screen. Pressing a plate
//! selects its object (`ListenToMessage`).

use std::collections::VecDeque;
use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::placement::{place_box, Box2D};
use crate::reticle::{average_box, clip_to_pixel, object_to_window};
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};

/// The input action that shows or hides the names (`ToggleNames`).
const TOGGLE_NAMES_ACTION: u32 = 0x4100_000F;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// An object's look as `AddName` takes it (`gmCWInterface::InqRadarLook` and its name).
#[derive(Debug, Clone, PartialEq)]
pub struct NameLook {
    pub id: u64,
    pub name: String,
    pub color: [f32; 4],
}

/// Where an object is this frame.
pub struct NamePlace<'a> {
    pub id: u64,
    /// The object's box in its own space (min, max).
    pub bounds: ([f32; 3], [f32; 3]),
    /// Object space to clip space (x right, y up) and whether in front of the camera.
    pub project: &'a dyn Fn([f32; 3]) -> ([f32; 2], bool),
    /// `Position::xy_distance_squared` from the player.
    pub xy_distance_sq: f32,
}

/// `gm3DTextUI::NameTextInfo`.
#[derive(Debug, Clone)]
struct Plate {
    root: usize,
    look: NameLook,
    boxes: VecDeque<[f32; 6]>,
}

/// `gm3DTextUI`'s name plates.
#[derive(Debug, Clone)]
pub struct NamePlates {
    layout: Arc<LayoutDesc>,
    /// `m_nameTextArray`.
    plates: Vec<Plate>,
    /// `m_showingNames`.
    showing: bool,
    /// `m_averageBBoxFrames`, `m_maxNamePlacementAttempts`.
    average_frames: usize,
    max_attempts: i32,
}

impl NamePlates {
    /// `gm3DTextUI::gm3DTextUI`'s part, with the reticle's tuning (`InitReticle` reads
    /// `AverageBBoxFrames` and `MaxNamePlacementAttempts` off the first `TargettingReticle`):
    /// names off until the options say otherwise.
    pub fn create(layout: &Arc<LayoutDesc>, average_frames: usize, max_attempts: i32) -> Self {
        Self {
            layout: Arc::clone(layout),
            plates: Vec::new(),
            showing: false,
            average_frames: average_frames.max(1),
            max_attempts,
        }
    }

    /// Whether names show.
    pub fn showing(&self) -> bool {
        self.showing
    }

    /// The objects with a plate, in `m_nameTextArray` order.
    pub fn named(&self) -> Vec<u64> {
        self.plates.iter().map(|p| p.look.id).collect()
    }

    /// The plate root of `object`.
    pub fn plate(&self, object: u64) -> Option<usize> {
        self.plates
            .iter()
            .find(|p| p.look.id == object)
            .map(|p| p.root)
    }

    /// `gm3DTextUI::ClearNames`: every plate gone (the last moved into the first's place each
    /// time, as retail empties its array).
    fn clear_names(&mut self, tree: &mut UiTree) {
        while !self.plates.is_empty() {
            let plate = self.plates.swap_remove(0);
            tree.remove_root(plate.root);
        }
    }

    /// `gm3DTextUI::RemoveName` (`SmartArray::RemoveUnOrdered`).
    fn remove_name(&mut self, tree: &mut UiTree, object: u64) {
        if let Some(at) = self.plates.iter().position(|p| p.look.id == object) {
            let plate = self.plates.swap_remove(at);
            tree.remove_root(plate.root);
        }
    }

    /// `gm3DTextUI::AddName`: a plate in the look's colour sized to the name, unless the object
    /// has one.
    fn add_name(&mut self, tree: &mut UiTree, look: &NameLook) {
        if self.plates.iter().any(|p| p.look.id == look.id) {
            return;
        }
        let Some(root) = tree.create_root(Arc::clone(&self.layout), id("NameTextTemplate")) else {
            return;
        };
        // A name is the world's overlay, not a window.
        tree.set_overlay_root(root);
        tree.el_mut(root)
            .attributes
            .insert(id("ObjectID"), AttributeValue::InstanceId(look.id));
        for (channel, value) in ["red", "green", "blue"].iter().zip(look.color) {
            tree.set_float(root, id(&format!("font_color_{channel}")), value);
        }
        tree.set_font_num_and_color(root, 0, 0);
        if let Some((w, h)) = tree.text_size_with_margins(root, &look.name) {
            tree.resize_to(root, w, h);
        }
        tree.set_text(root, &look.name);
        tree.show_object(root, false);
        self.plates.push(Plate {
            root,
            look: look.clone(),
            boxes: VecDeque::new(),
        });
    }

    /// `gm3DTextUI::RecvNotice_ReloadOptions` / `ToggleNames`: names on (`InitNames`, every
    /// look anew) or off (`ClearNames`).
    pub fn set_showing(&mut self, tree: &mut UiTree, showing: bool, looks: &[NameLook]) {
        self.showing = showing;
        self.clear_names(tree);
        if showing {
            for look in looks {
                self.add_name(tree, look);
            }
        }
    }

    /// The looks as they stand: `RecvNotice_CreateObject` adds a plate, `RecvNotice_BeingDeleted`
    /// removes one, and a look that changed (`RecvNotice_ChangeRadarLook`) is removed and added
    /// again.
    pub fn sync(&mut self, tree: &mut UiTree, looks: &[NameLook]) {
        if !self.showing {
            return;
        }
        let gone: Vec<u64> = self
            .plates
            .iter()
            .filter(|p| !looks.contains(&p.look))
            .map(|p| p.look.id)
            .collect();
        for object in gone {
            self.remove_name(tree, object);
        }
        for look in looks {
            self.add_name(tree, look);
        }
    }

    /// `gm3DTextUI::UpdateNames` (`hidden` in portal space): `range_sq` the radar range squared
    /// where the player stands, `precluded` the space kept at the top and bottom of the screen.
    pub fn update(
        &mut self,
        tree: &mut UiTree,
        places: &[NamePlace<'_>],
        selection: u64,
        selection_rect: Option<[i32; 4]>,
        range_sq: f32,
        precluded: (i32, i32),
        hidden: bool,
    ) {
        if !self.showing {
            return;
        }
        let (display_w, display_h) = tree.display;
        let screen = Box2D {
            x0: 0,
            y0: 0,
            x1: display_w - 1,
            y1: display_h - 1,
        };
        let mut placed: Vec<Box2D> = Vec::new();
        if selection != 0 {
            if let Some([x0, y0, x1, y1]) = selection_rect {
                placed.push(Box2D { x0, y0, x1, y1 });
            }
        }
        for plate in &mut self.plates {
            tree.show_object(plate.root, false);
            if hidden || plate.look.id == selection {
                continue;
            }
            let Some(place) = places.iter().find(|p| p.id == plate.look.id) else {
                continue;
            };
            if range_sq < place.xy_distance_sq {
                continue;
            }
            let (width, height) = tree
                .get(plate.root)
                .map_or((0, 0), |e| (e.rect.width(), e.rect.height()));
            // `CalcScreenPosition`.
            let (lo, hi) = average_box(&mut plate.boxes, self.average_frames, place.bounds);
            let Some((min, max)) = object_to_window(place.project, lo, hi) else {
                continue;
            };
            let (wx0, _) = clip_to_pixel(min, display_w, display_h);
            let (wx1, top) = clip_to_pixel(max, display_w, display_h);
            let x = (wx0 + (wx1 - wx0) / 2).clamp(0, display_w);
            let mut y = top - height;
            let (top_limit, bottom_limit) = (precluded.0, display_h - precluded.1);
            if y < top_limit {
                y = top_limit;
            } else if bottom_limit < y {
                y = bottom_limit;
            }
            let x0 = (f64::from(x) - f64::from(width as u32) * 0.5) as i32;
            let Some(here) = place_box(
                &screen,
                Box2D::new(x0, y, width, height),
                &placed,
                self.max_attempts,
            ) else {
                continue;
            };
            placed.push(here);
            tree.set_object_position(plate.root, here.x0, here.y0);
            tree.show_object(plate.root, true);
        }
    }

    /// `gm3DTextUI::ListenToMessage` (the global one): the names key (input action
    /// `0x4100000F`) asks `ToggleNames`.
    pub fn toggle_requested(message: &UiMessage) -> bool {
        message.message == GLOBAL_KEY_PRESS && message.data as u32 == TOGGLE_NAMES_ACTION
    }

    /// `gm3DTextUI::ListenToMessage`: a press on a plate selects its object.
    pub fn message(&self, tree: &UiTree, message: &UiMessage) -> Option<u64> {
        if message.element_id != id("NameTextTemplate") || message.message != MSG_MOUSE_PRESS_TOP {
            return None;
        }
        match tree.get(message.element?)?.attribute(id("ObjectID")) {
            Some(AttributeValue::InstanceId(object)) if *object != 0 => Some(*object),
            _ => None,
        }
    }
}
