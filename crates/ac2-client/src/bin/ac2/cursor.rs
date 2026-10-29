//! The game's own mouse cursor (`RenderDeviceD3D::SetCursorProperties`): the image and hot spot
//! the HUD set (`UIElementManager::SetCursor`) as the window's cursor, made once per image and
//! hot spot. Until the HUD sets one, the system's arrow shows.

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ac2_ui::UiPixels;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CustomCursor, CustomCursorSource, Window};

/// A cursor's image file and hot spot.
type CursorKey = (u32, i32, i32);

#[derive(Default)]
pub struct GameCursor {
    shown: Option<CursorKey>,
    made: HashMap<CursorKey, CustomCursor>,
}

impl GameCursor {
    /// Shows `cursor` (its key and pixels) on `window` when it changed; an image the system
    /// cannot take leaves the cursor as it is.
    pub fn show(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: &Window,
        cursor: Option<(CursorKey, &UiPixels)>,
    ) {
        let Some((key, pixels)) = cursor else {
            return;
        };
        if self.shown == Some(key) {
            return;
        }
        let made = match self.made.entry(key) {
            Entry::Occupied(made) => made.into_mut(),
            Entry::Vacant(slot) => {
                let Some(source) = cursor_source(key, pixels) else {
                    return;
                };
                slot.insert(event_loop.create_custom_cursor(source))
            }
        };
        window.set_cursor(made.clone());
        self.shown = Some(key);
    }
}

/// The cursor image with its hot spot kept on it, `None` for an image the system cannot take.
fn cursor_source(key: CursorKey, pixels: &UiPixels) -> Option<CustomCursorSource> {
    let width = u16::try_from(pixels.width).ok().filter(|w| *w > 0)?;
    let height = u16::try_from(pixels.height).ok().filter(|h| *h > 0)?;
    let hot_x = key.1.clamp(0, i32::from(width) - 1) as u16;
    let hot_y = key.2.clamp(0, i32::from(height) - 1) as u16;
    CustomCursor::from_rgba(pixels.rgba.to_vec(), width, height, hot_x, hot_y).ok()
}
