//! `gmAvatarModificationUI`: character generation's appearance screen. The current result's
//! avatar stands in its race's viewport (`<race>Viewport`, `Init`); one slider per physique type
//! the race and sex offer steps through the script's valid values (`GetDataTable`; a type with
//! none loses its field), labelled by the script (`InitElements`) and set from the result
//! (`InitUIFromData`). Moving a slider sets that value (`Temp`) and remakes the avatar's
//! appearance (`UpdateByMask`). The head, body and clothing buttons switch which sliders show;
//! randomize changes the shown part (or everything); zoom and the rotation slider (or dragging
//! over `MouseRotationOverField`) move the camera; back returns to avatar selection; next opens
//! the naming dialog (`gmGenerationConfirmationUI`), which cancel closes.
//!
//! The scene is the client's: it stands [`AvatarModificationUi::phys_obj`] in
//! [`AvatarModificationUi::viewport`], renders the current result (`PersistentData::current`)
//! again whenever [`AvatarModificationUi::appearance`] changes, and frames it by
//! [`AvatarModificationUi::camera`] (`InitCamera` / `UpdateCamera`).
//!
//! Not modelled yet: the tooltips and the music.

use ac2_weenie::Appearance;

use crate::dialog::DIALOG_WAIT;
use crate::mouse::UiMessage;
use crate::pregame::chargen::{phys_type as pt, phys_type_value, set_phys_type_value, RACE_NAMES};
use crate::pregame::confirmation::ConfirmationUi;
use crate::pregame::flow::MODE_AVATAR_SELECTION;
use crate::pregame::{screen_music, Ctx, PregameRequest, LAYOUT_AVATAR_MODIFICATION};
use crate::scrollbar::MSG_NEW_POSITION;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_MOVE_TOP};

/// `InitUITables`: each physique type's element names (`<Name>Slider`, `LeftButton`,
/// `RightButton`, `Label`, `Field`).
const PHYS_TYPES: [(u32, &str); 12] = [
    (pt::HEIGHT, "Height"),
    (pt::BUILD, "Build"),
    (pt::SKIN_TONE, "SkinTone"),
    (pt::SKIN_DETAIL, "SkinDetail"),
    (pt::FACE_DETAIL, "FaceDetail"),
    (pt::HEAD_DETAIL, "HeadDetail"),
    (pt::HEAD_FRILL, "HeadFrill"),
    (pt::FRILL_COLOR, "FrillColor"),
    (pt::SPECIAL, "Special"),
    (pt::SHIRT_CLOTHING_COLOR, "ShirtClothingColor"),
    (pt::PANTS_CLOTHING_COLOR, "PantsClothingColor"),
    (pt::BOOTS_CLOTHING_COLOR, "BootsClothingColor"),
];

/// The camera the client frames the avatar with (`_zoomed`, the rotation slider).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ModificationCamera {
    pub zoomed: bool,
    /// `RotationSlider`'s position, [0, 1].
    pub rotation: f32,
}

/// One offered physique type: its values and elements.
#[derive(Debug, Clone, PartialEq)]
struct Slider {
    phys_type: u32,
    name: &'static str,
    values: Vec<f32>,
}

/// `gmAvatarModificationUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct AvatarModificationUi {
    field: Option<usize>,
    torch: Option<usize>,
    viewport: Option<usize>,
    /// `_headField`, `_bodyField`, `_clothingField` and their buttons.
    parts: [Option<usize>; 3],
    part_buttons: [Option<usize>; 3],
    /// `_dataTable` / `_dataToUITable`: the offered types.
    sliders: Vec<Slider>,
    camera: ModificationCamera,
    /// `_lastX`, `_lastY`.
    last: (i32, i32),
    /// Bumped by `UpdateByMask`: the avatar's appearance changed.
    appearance: u32,
    /// `gmCharGenResult::GetPhysObj`: the avatar's physics object (`Init`).
    phys_obj: u32,
    /// The appearance `UpdateByMask` last applied (`InqAppliedAppearanceVDesc`), boxed to keep
    /// the screen's state small.
    applied: Option<Box<Appearance>>,
    dialog: Option<ConfirmationUi>,
    wait_context: u32,
}

impl AvatarModificationUi {
    fn find(&self, ctx: &Ctx<'_>, name: &str) -> Option<usize> {
        ctx.tree.child_recursive(self.field?, tokenize(name))
    }

    /// `gmAvatarModificationUI::gmAvatarModificationUI`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let mut ui = Self::default();
        if let Some(layout) = ctx.assets.layout(LAYOUT_AVATAR_MODIFICATION) {
            ui.field = ctx
                .tree
                .create_root(layout.clone(), tokenize("AvatarModificationField"));
            ui.torch = ctx
                .tree
                .create_root(layout.clone(), tokenize("TorchViewport"));
        }
        let parts = ["HeadField", "BodyField", "ClothingField"].map(|n| ui.find(ctx, n));
        let buttons = ["HeadButton", "BodyButton", "ClothingButton"].map(|n| ui.find(ctx, n));
        ui.parts = parts;
        ui.part_buttons = buttons;
        if buttons[0].is_some() && parts.iter().chain(&buttons).all(Option::is_some) {
            ui.show_part(ctx, 1);
        }
        ui.init(ctx);
        ui.get_data_table(ctx);
        ui.init_elements(ctx);
        ui.init_ui_from_data(ctx);
        // `InitCamera`, whose `UpdateCamera` reads the rotation slider.
        ui.update_camera(ctx);
        ui
    }

    /// The framework's destructor: roots and the naming dialog gone, its wait dialog closed.
    pub(crate) fn destroy(mut self, ctx: &mut Ctx<'_>) {
        for root in [self.viewport, self.torch, self.field]
            .into_iter()
            .flatten()
        {
            ctx.tree.remove_root(root);
        }
        if let Some(dialog) = self.dialog.take() {
            dialog.destroy(ctx);
        }
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
    }

    /// The head (0), body (1) or clothing (2) sliders shown, their button ghosted.
    fn show_part(&self, ctx: &mut Ctx<'_>, part: usize) {
        for i in 0..3 {
            if let Some(button) = self.part_buttons[i] {
                ctx.tree.set_bool(button, BUTTON_GHOSTED, i == part);
            }
            if let Some(field) = self.parts[i] {
                ctx.tree.set_visible(field, i == part);
            }
        }
    }

    /// `gmAvatarModificationUI::Init`: the avatar's physics object and the race's viewport for
    /// it.
    fn init(&mut self, ctx: &mut Ctx<'_>) {
        self.last = ctx.tree.pointer();
        let Some(race) = ctx.data.current().map(|c| c.race) else {
            return;
        };
        let sex = ctx.data.current().map_or(0, |c| c.sex);
        self.phys_obj = ctx.system.phys_obj(race, sex);
        self.applied = ctx
            .data
            .current()
            .cloned()
            .and_then(|cgr| ctx.system.applied_appearance(&cgr))
            .map(Box::new);
        let name = ctx.system.enum_string(RACE_NAMES, race).unwrap_or_default();
        let viewport = format!("{}Viewport", name.to_lowercase());
        if let Some(layout) = ctx.assets.layout(LAYOUT_AVATAR_MODIFICATION) {
            self.viewport = ctx.tree.create_root(layout.clone(), tokenize(&viewport));
        }
        // The race viewport's music (`gmAvatarModificationUI::gmAvatarModificationUI`).
        if let Some(did) = screen_music(ctx.tree, self.viewport) {
            ctx.requests.push(PregameRequest::Music(did));
        }
    }

    /// `gmAvatarModificationUI::GetDataTable`: each type's valid values; a type without any
    /// loses its field.
    fn get_data_table(&mut self, ctx: &mut Ctx<'_>) {
        let Some((race, sex)) = ctx.data.current().map(|c| (c.race, c.sex)) else {
            return;
        };
        self.sliders.clear();
        for (phys_type, name) in PHYS_TYPES {
            match ctx.system.valid_phys_type_values(race, sex, phys_type) {
                Some(values) => self.sliders.push(Slider {
                    phys_type,
                    name,
                    values,
                }),
                None => {
                    if let Some(field) = self.find(ctx, &format!("{name}Field")) {
                        ctx.tree.set_visible(field, false);
                        ctx.tree.delete_element(field);
                    }
                }
            }
        }
    }

    /// `gmAvatarModificationUI::InitElements`: each slider's label from the script.
    fn init_elements(&mut self, ctx: &mut Ctx<'_>) {
        let Some((race, sex)) = ctx.data.current().map(|c| (c.race, c.sex)) else {
            return;
        };
        for slider in &self.sliders {
            let Some(label) = self.find(ctx, &format!("{}Label", slider.name)) else {
                continue;
            };
            if let Some(info) = ctx.system.phys_type_label(race, sex, slider.phys_type) {
                let text = ctx.strings.render_info(&info);
                ctx.tree.set_text(label, &text);
            }
        }
    }

    /// `gmAvatarModificationUI::InitUIFromData`: each slider at its value's stop.
    fn init_ui_from_data(&mut self, ctx: &mut Ctx<'_>) {
        let Some(cgr) = ctx.data.current().cloned() else {
            return;
        };
        if self.field.is_none() {
            return;
        }
        for slider in &self.sliders {
            let value = phys_type_value(&cgr, slider.phys_type);
            let index = slider
                .values
                .iter()
                .position(|v| *v == value)
                .map_or(-1, |i| i as i32);
            let Some(element) = self.find(ctx, &format!("{}Slider", slider.name)) else {
                continue;
            };
            let stops = slider.values.len() as i32;
            ctx.tree
                .scrollbar_set_position(element, index as f32 / (stops - 1) as f32);
            ctx.tree.scrollbar_set_stops(element, stops);
        }
    }

    /// `gmAvatarModificationUI::MakeCurrentMask`: the shown part's types.
    fn current_mask(&self, ctx: &Ctx<'_>) -> u32 {
        let visible = |i: usize| {
            self.parts[i]
                .and_then(|f| ctx.tree.get(f))
                .is_some_and(|f| f.visible())
        };
        if self.parts.iter().any(Option::is_none) {
            return u32::MAX;
        }
        if visible(0) {
            0x870
        } else if visible(1) {
            0x8F
        } else if visible(2) {
            pt::CLOTHING_MASK
        } else {
            u32::MAX
        }
    }

    /// `gmAvatarModificationUI::UpdateByMask`: the avatar's appearance remade from the current
    /// result (`InqAppliedAppearanceVDesc`), and counted.
    fn update_by_mask(&mut self, ctx: &mut Ctx<'_>) {
        self.applied = ctx
            .data
            .current()
            .cloned()
            .and_then(|cgr| ctx.system.applied_appearance(&cgr))
            .map(Box::new);
        self.appearance = self.appearance.wrapping_add(1);
    }

    /// `gmAvatarModificationUI::MakePleaseWaitDialog`.
    fn make_please_wait_dialog(&mut self, ctx: &mut Ctx<'_>) {
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
        let text = ctx.strings.get("ID_Wait_PleaseWait");
        self.wait_context = ctx.dialogs.make_dialog(ctx.tree, &text, DIALOG_WAIT, false);
    }

    /// `gmAvatarModificationUI::ListenToMessage`, then the naming dialog's.
    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage, left_down: bool) {
        if let Some(dialog) = &mut self.dialog {
            dialog.message(ctx, message);
        }
        if ctx.data.current().is_none() || self.field.is_none() {
            return;
        }
        let element = message.element_id;
        let name = |n: &str| element == tokenize(n);
        if name("RotationSlider") && message.message == MSG_NEW_POSITION {
            self.update_camera(ctx);
            return;
        }
        if name("MouseRotationOverField") && message.message == MSG_MOUSE_MOVE_TOP {
            self.drag_rotation(ctx, message, left_down);
            return;
        }
        if message.message == MSG_NEW_POSITION {
            let Some(slider) = self
                .sliders
                .iter()
                .position(|s| name(&format!("{}Slider", s.name)))
            else {
                return;
            };
            self.temp(ctx, slider, message.element);
            self.update_by_mask(ctx);
            return;
        }
        if message.message != MSG_BUTTON_RELEASE_ON {
            return;
        }
        if name("BackButton") {
            self.make_please_wait_dialog(ctx);
            ctx.flow.queue_mode(MODE_AVATAR_SELECTION, "");
        } else if name("CancelButton") {
            if let Some(dialog) = self.dialog.take() {
                dialog.destroy(ctx);
            }
        } else if name("RandomizeCurrentButton") || name("RandomizeAllButton") {
            let mask = if name("RandomizeAllButton") {
                u32::MAX
            } else {
                self.current_mask(ctx)
            };
            let system = &mut *ctx.system;
            if let Some(cgr) = ctx.data.current_mut() {
                system.randomize(mask, cgr);
            }
            self.init_ui_from_data(ctx);
            self.update_by_mask(ctx);
        } else if name("ZoomButton") {
            self.camera.zoomed = !self.camera.zoomed;
        } else if name("NextButton") {
            if self.dialog.is_none() {
                self.dialog = Some(ConfirmationUi::create(ctx));
            }
        } else if let Some(part) = ["HeadButton", "BodyButton", "ClothingButton"]
            .iter()
            .position(|n| name(n))
        {
            if self
                .parts
                .iter()
                .chain(&self.part_buttons)
                .all(Option::is_some)
            {
                self.show_part(ctx, part);
                self.camera.zoomed = part == 0;
            }
        }
    }

    /// `gmAvatarModificationUI::Temp`: the slider's stop picks the type's value.
    fn temp(&mut self, ctx: &mut Ctx<'_>, slider: usize, element: Option<usize>) {
        let Some(element) = element else {
            return;
        };
        let slider = &self.sliders[slider];
        let position = f64::from(ctx.tree.scrollbar_position(element));
        let count = slider.values.len() as f64;
        let index = ((count - 1.0) * position + 0.5) as i64;
        let value = usize::try_from(index)
            .ok()
            .and_then(|i| slider.values.get(i))
            .copied();
        if let (Some(value), Some(cgr)) = (value, ctx.data.current_mut()) {
            set_phys_type_value(cgr, slider.phys_type, value);
        }
    }

    /// `UpdateCamera`: the rotation slider's position for the client's camera.
    fn update_camera(&mut self, ctx: &Ctx<'_>) {
        if let Some(slider) = self.find(ctx, "RotationSlider") {
            self.camera.rotation = ctx.tree.scrollbar_position(slider);
        }
    }

    /// `ListenToMessage(MouseRotationOverField)`: dragging with a left button held turns the
    /// avatar (the rotation slider by the pointer's travel over the field's width, wrapping).
    fn drag_rotation(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage, left_down: bool) {
        let pointer = ctx.tree.pointer();
        let over = message
            .element
            .and_then(|e| ctx.tree.get(e))
            .is_some_and(|e| e.mouse_over());
        if over && left_down {
            let field = message.element.map(|e| ctx.tree.screen_box(e));
            if let (Some(field), Some(slider)) = (field, self.find(ctx, "RotationSlider")) {
                let delta = self.last.0 - pointer.0;
                let mut position =
                    ctx.tree.scrollbar_position(slider) - delta as f32 / field.width() as f32;
                if position < 0.0 {
                    position = 1.0;
                } else if position > 1.0 {
                    position = 0.0;
                }
                ctx.tree.scrollbar_set_position(slider, position);
                self.update_camera(ctx);
            }
        }
        self.last = pointer;
    }

    /// How many times the avatar's appearance changed (`UpdateByMask`).
    pub fn appearance(&self) -> u32 {
        self.appearance
    }

    /// The applied appearance the avatar shows (`None` before the first `UpdateByMask`).
    pub fn applied(&self) -> Option<&Appearance> {
        self.applied.as_deref()
    }

    pub fn camera(&self) -> ModificationCamera {
        self.camera
    }

    /// The avatar's physics object (`GetPhysObj`), 0 without a current result.
    pub fn phys_obj(&self) -> u32 {
        self.phys_obj
    }

    /// The race's viewport root the avatar stands in (`Init`).
    pub fn viewport(&self) -> Option<usize> {
        self.viewport
    }

    /// The offered physique types, in `InitUITables` order.
    pub fn offered(&self) -> Vec<u32> {
        self.sliders.iter().map(|s| s.phys_type).collect()
    }

    /// The naming dialog, while open.
    pub fn dialog(&self) -> Option<&ConfirmationUi> {
        self.dialog.as_ref()
    }

    pub(crate) fn dialog_mut(&mut self) -> Option<&mut ConfirmationUi> {
        self.dialog.as_mut()
    }
}
