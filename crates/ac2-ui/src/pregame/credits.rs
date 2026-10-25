//! `gmCreditsUI`: the credits from character selection's credits button. `TextField`'s text
//! area gets every `ID_Credits<n>` string of the field's `stringtable` from 1 up
//! (`Initialize`), sized to the laid-out text, and scrolls from below the field to above it
//! over a duration made from the field's `duration` (`UpdateText`); `PictureField` shows its
//! `ID_Picture<n>` images in turn, each at the next `x_<n>`, `y_<n>` and faded in and out by its
//! `fading` state (`UpdatePicture`). The end of the scroll or a key press returns to character
//! selection behind a please-wait dialog (`Close`).
//!
//! Not modelled yet: the alternate fields (Ctrl+Alt held when the screen opens) and the music
//! (`ID_Sound<n>`, `UpdateSound`).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};

use crate::compose::UiPixels;
use crate::dialog::DIALOG_WAIT;
use crate::layout::{AttributeValue, ElementDesc, LayoutDesc};
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::pregame::flow::MODE_CHARACTER_SELECTION;
use crate::pregame::{Ctx, LAYOUT_CREDITS};
use crate::text_data::StoreTextData;
use crate::token::{tokenize, STATELESS};
use crate::UiTextData;

/// `UIElement::GetAttribute_*` of an element description's stateless attributes.
fn attribute(desc: &ElementDesc, name: &str) -> Option<AttributeValue> {
    let id = tokenize(name);
    desc.states
        .iter()
        .filter(|s| s.id == STATELESS)
        .flat_map(|s| &s.attributes)
        .find(|a| a.id == id)
        .map(|a| a.value.clone())
}

/// The credits' strings and pictures, read with the pregame data on its worker thread.
pub(crate) fn preload_credits(
    store: &ResourceStore,
    layout: &LayoutDesc,
    text: &Arc<StoreTextData>,
    images: &mut HashMap<u32, UiPixels>,
) -> ResourceResult<()> {
    for desc in &layout.elements {
        if let Some(AttributeValue::DataId(table)) = attribute(desc, "stringtable") {
            for n in 1.. {
                if text
                    .string(table, tokenize(&format!("ID_Credits{n}")))
                    .is_none()
                {
                    break;
                }
            }
        }
        for n in 0.. {
            let Some(AttributeValue::DataId(file)) = attribute(desc, &format!("ID_Picture{n}"))
            else {
                break;
            };
            if images.contains_key(&file) {
                continue;
            }
            if let Some(surface) = ac2_assets::load_ac2_ui_surface(store, DataId::new(file))? {
                let pixels = UiPixels {
                    width: surface.width,
                    height: surface.height,
                    rgba: surface.rgba,
                };
                images.insert(file, pixels);
            }
        }
    }
    Ok(())
}

/// `gmCreditsUI`'s state.
#[derive(Debug, Clone, Default)]
pub(crate) struct CreditsUi {
    picture: Option<usize>,
    text_field: Option<usize>,
    text_area: Option<usize>,
    /// `m_pictureArray`, `m_xArray`, `m_yArray` and the last index of each.
    pictures: Vec<u32>,
    xs: Vec<i32>,
    ys: Vec<i32>,
    last: [usize; 3],
    /// `m_duration`, and `m_endTime` once the loop runs.
    duration: f32,
    end: Option<f64>,
    wait_context: u32,
    /// `Close` unregistered the key press and the loop.
    closed: bool,
}

impl CreditsUi {
    /// `gmCreditsUI::gmCreditsUI`: the fields, then `Initialize`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let mut ui = Self::default();
        if let Some(layout) = ctx.assets.layout(LAYOUT_CREDITS) {
            ui.picture = ctx
                .tree
                .create_root(layout.clone(), tokenize("PictureField"));
            ui.text_field = ctx.tree.create_root(layout.clone(), tokenize("TextField"));
        }
        if let Some(field) = ui.text_field {
            if let Some(AttributeValue::Id(area)) = ctx
                .tree
                .get(field)
                .and_then(|e| e.attribute(tokenize("textarea")).cloned())
            {
                ui.text_area = ctx.tree.child_recursive(field, area);
            }
        }
        ui.initialize(ctx);
        ui
    }

    /// The framework's destructor: its roots removed, its dialog closed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        for root in [self.picture, self.text_field].into_iter().flatten() {
            ctx.tree.remove_root(root);
        }
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
    }

    fn int(ctx: &Ctx<'_>, index: usize, name: &str) -> Option<AttributeValue> {
        ctx.tree.get(index)?.attribute(tokenize(name)).cloned()
    }

    /// `gmCreditsUI::Initialize`.
    fn initialize(&mut self, ctx: &mut Ctx<'_>) {
        let (Some(picture), Some(field), Some(area)) =
            (self.picture, self.text_field, self.text_area)
        else {
            return;
        };
        let table = match Self::int(ctx, field, "stringtable") {
            Some(AttributeValue::DataId(table)) => table,
            _ => 0,
        };
        let data = ctx.tree.text_data.clone();
        let mut next = 1_u32;
        while let Some(units) = data
            .as_ref()
            .and_then(|d| d.string(table, tokenize(&format!("ID_Credits{next}"))))
        {
            if let Some(text) = ctx.tree.el_mut(area).text.as_mut() {
                text.add_text(&units);
            }
            next += 1;
        }
        ctx.tree.changed = true;
        let field_height = ctx.tree.screen_box(field).height();
        let field_width = ctx.tree.screen_box(field).width();
        ctx.tree.resize_to(area, field_width, 0);
        let paper = ctx.tree.text_paper_height(area).unwrap_or(0);
        ctx.tree.resize_to(area, field_width, paper);
        self.update_picture(ctx);
        let duration = match Self::int(ctx, field, "duration") {
            Some(AttributeValue::Float(d)) => d,
            _ => 1.0,
        };
        let (f, a) = (f64::from(field_height), f64::from(paper));
        let duration = (a + f) / ((a / f64::from(next) + f) / f64::from(duration));
        self.duration = duration as f32;
        let read = |prefix: &str| -> Vec<AttributeValue> {
            (0..)
                .map_while(|n| Self::int(ctx, picture, &format!("{prefix}{n}")))
                .collect()
        };
        self.pictures = read("ID_Picture")
            .into_iter()
            .filter_map(|v| match v {
                AttributeValue::DataId(d) => Some(d),
                _ => None,
            })
            .collect();
        let ints = |values: Vec<AttributeValue>| -> Vec<i32> {
            values
                .into_iter()
                .filter_map(|v| match v {
                    AttributeValue::Int(i) => Some(i),
                    _ => None,
                })
                .collect()
        };
        self.xs = ints(read("x_"));
        self.ys = ints(read("y_"));
    }

    /// `GetX` / `GetY` / the picture: the entry after the last one shown, wrapping.
    fn next(last: &mut usize, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        *last = (*last + 1) % len;
        Some(*last)
    }

    /// `gmCreditsUI::UpdatePicture`: once a picture's `fading` state is over, the next picture
    /// at the next place, sized to its image, fading.
    fn update_picture(&mut self, ctx: &mut Ctx<'_>) {
        let Some(picture) = self.picture else {
            return;
        };
        let fading = tokenize("fading");
        if ctx.tree.get(picture).is_none_or(|e| e.state() == fading) {
            return;
        }
        let Some(index) = Self::next(&mut self.last[0], self.pictures.len()) else {
            return;
        };
        let file = self.pictures[index];
        if file == 0 {
            return;
        }
        let y = Self::next(&mut self.last[2], self.ys.len()).map_or(0, |i| self.ys[i]);
        let x = Self::next(&mut self.last[1], self.xs.len()).map_or(0, |i| self.xs[i]);
        ctx.tree.move_to(picture, x, y);
        ctx.tree.moved = true;
        ctx.tree.set_image(picture, file, 0);
        if let Some(image) = ctx.assets.images.get(&file) {
            ctx.tree
                .resize_to(picture, image.width as i32, image.height as i32);
        }
        ctx.tree.set_state(picture, fading);
        ctx.tree.changed = true;
    }

    /// `gmCreditsUI::UpdateText`: the text area at its place along the scroll; at the end,
    /// `Close`.
    fn update_text(&mut self, ctx: &mut Ctx<'_>, now: f64) {
        let (Some(field), Some(area), Some(end)) = (self.text_field, self.text_area, self.end)
        else {
            return;
        };
        let mut percentage = 1.0_f32;
        if self.duration != 0.0 {
            percentage = (1.0 - (end - now) / f64::from(self.duration)) as f32;
        }
        let percentage = percentage.clamp(0.0, 1.0);
        let field_height = ctx.tree.screen_box(field).height();
        let span = (field_height + ctx.tree.screen_box(area).height()) as f32;
        let travelled = (f64::from(percentage) * f64::from(span)) as i32;
        let x = ctx.tree.el(area).rect.x0;
        ctx.tree.move_to(area, x, field_height - travelled);
        ctx.tree.changed = true;
        if percentage == 1.0 {
            self.close(ctx);
        }
    }

    /// `gmCreditsUI::Close`: back to character selection behind a please-wait dialog.
    fn close(&mut self, ctx: &mut Ctx<'_>) {
        if self.closed {
            return;
        }
        self.closed = true;
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
        let text = ctx.strings.get("ID_Wait_PleaseWait");
        self.wait_context = ctx.dialogs.make_dialog(ctx.tree, &text, DIALOG_WAIT, false);
        ctx.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
    }

    /// `ListenToMessage(UIMESSAGE_GLOBAL_KEYPRESS)`.
    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        if !self.closed && message.element.is_none() && message.message == GLOBAL_KEY_PRESS {
            self.close(ctx);
        }
    }

    /// `ListenToMessage(UIMESSAGE_GLOBAL_LOOP)` at `now` seconds (`Update`): the scroll's end
    /// time is set on the first loop (`Initialize`'s `Timer` read).
    pub(crate) fn global_loop(&mut self, ctx: &mut Ctx<'_>, now: f64) {
        if self.closed {
            return;
        }
        let duration = f64::from(self.duration);
        self.end.get_or_insert(now + duration);
        self.update_picture(ctx);
        self.update_text(ctx, now);
    }
}
