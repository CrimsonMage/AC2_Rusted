//! The UI fitted to a window of any size: a deliberate deviation from retail.
//!
//! Retail laid its UI out pixel for pixel on the window `Turbine::Client::InitUI` made
//! (`Client+0x60` / `Client+100`, 1024 x 768 unless `width` / `height` said otherwise) and only
//! changed size through a resolution change. Its layouts are authored at 1024 x 768: the pregame
//! screens are one fixed 1024 x 768 panel and the HUD's roots are anchored to that display. This
//! client's window resizes freely, so the UI is laid out on a logical display and drawn scaled:
//! every UI pixel is `scale` window pixels, with `scale = min(width / 1024, height / 768)`.
//!
//! - [`UiScale::fill`] (the HUD): the logical display covers the window, so one axis is
//!   1024 or 768 and the other is longer; the layouts' anchors spread the roots over it.
//! - [`UiScale::fit`] (the pregame screens): the logical display is exactly 1024 x 768,
//!   centred in the window with the rest left black.

/// The display retail's layouts are authored for (`Client+0x60` / `Client+100` defaults).
pub const BASE_DISPLAY: (u32, u32) = (1024, 768);

/// How the UI's logical display maps onto the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiScale {
    /// Window pixels per UI pixel.
    pub scale: f64,
    /// Where the UI's (0, 0) lies in the window, in window pixels.
    pub origin: (f64, f64),
    /// The logical display the UI lays out on (`UiTree::set_display`).
    pub display: (u32, u32),
    /// The window, in pixels.
    pub window: (u32, u32),
}

impl UiScale {
    fn factor(window: (u32, u32)) -> f64 {
        let (w, h) = (f64::from(window.0.max(1)), f64::from(window.1.max(1)));
        (w / f64::from(BASE_DISPLAY.0)).min(h / f64::from(BASE_DISPLAY.1))
    }

    /// A logical display covering the whole window (the HUD).
    pub fn fill(window: (u32, u32)) -> Self {
        let window = (window.0.max(1), window.1.max(1));
        let scale = Self::factor(window);
        let display = (
            ((f64::from(window.0) / scale).round() as u32).max(BASE_DISPLAY.0),
            ((f64::from(window.1) / scale).round() as u32).max(BASE_DISPLAY.1),
        );
        Self {
            scale,
            origin: (0.0, 0.0),
            display,
            window,
        }
    }

    /// The fixed [`BASE_DISPLAY`] centred in the window (the pregame screens).
    pub fn fit(window: (u32, u32)) -> Self {
        let window = (window.0.max(1), window.1.max(1));
        let scale = Self::factor(window);
        let used = |side: u32, base: u32| (f64::from(side) - f64::from(base) * scale) / 2.0;
        Self {
            scale,
            origin: (
                used(window.0, BASE_DISPLAY.0).floor(),
                used(window.1, BASE_DISPLAY.1).floor(),
            ),
            display: BASE_DISPLAY,
            window,
        }
    }

    /// Whether the UI is drawn pixel for pixel.
    pub fn is_identity(&self) -> bool {
        self.scale == 1.0 && self.origin == (0.0, 0.0)
    }

    /// A window point in UI pixels.
    pub fn to_ui(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.origin.0) / self.scale,
            (y - self.origin.1) / self.scale,
        )
    }

    /// A UI point in window pixels.
    pub fn to_window(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.origin.0 + x * self.scale,
            self.origin.1 + y * self.scale,
        )
    }

    /// The UI actions with the pointer extents (`UI_POINTER_X` / `UI_POINTER_Y`) moved from
    /// window pixels to UI pixels.
    pub fn actions_to_ui(&self, actions: &[(u32, f32, bool)]) -> Vec<(u32, f32, bool)> {
        actions
            .iter()
            .map(|&(action, extent, start)| {
                let extent = match action {
                    ac2_ui::UI_POINTER_X => self.to_ui(f64::from(extent), 0.0).0.floor() as f32,
                    ac2_ui::UI_POINTER_Y => self.to_ui(0.0, f64::from(extent)).1.floor() as f32,
                    _ => extent,
                };
                (action, extent, start)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{UiScale, BASE_DISPLAY};

    #[test]
    fn the_authored_window_is_drawn_pixel_for_pixel() {
        for scale in [UiScale::fill((1024, 768)), UiScale::fit((1024, 768))] {
            assert!(scale.is_identity());
            assert_eq!(scale.display, BASE_DISPLAY);
        }
    }

    #[test]
    fn a_wide_window_scales_by_height() {
        let fill = UiScale::fill((1920, 1080));
        assert_eq!(fill.scale, 1080.0 / 768.0);
        assert_eq!(fill.display, (1365, 768));
        let fit = UiScale::fit((1920, 1080));
        assert_eq!(fit.display, BASE_DISPLAY);
        // 1024 UI pixels are 1440 window pixels: 240 on each side.
        assert_eq!(fit.origin, (240.0, 0.0));
        assert_eq!(fit.to_ui(240.0 + 1440.0 / 2.0, 540.0), (512.0, 384.0));
    }

    #[test]
    fn a_small_window_shrinks_the_ui_instead_of_cutting_it_off() {
        let fit = UiScale::fit((800, 600));
        assert_eq!(fit.scale, 800.0 / 1024.0);
        assert_eq!(fit.origin, (0.0, 0.0));
        let (x, y) = fit.to_window(1024.0, 768.0);
        assert_eq!((x, y), (800.0, 600.0));
        let fill = UiScale::fill((800, 700));
        assert_eq!(fill.display.0, 1024);
        assert!(fill.display.1 > 768);
    }

    #[test]
    fn pointer_actions_move_into_ui_pixels() {
        let fit = UiScale::fit((2048, 1536));
        let moved = fit.actions_to_ui(&[
            (ac2_ui::UI_POINTER_X, 1000.0, true),
            (ac2_ui::UI_POINTER_Y, 700.0, true),
            (0x4000_0004, 1.0, true),
        ]);
        assert_eq!(
            moved,
            [
                (ac2_ui::UI_POINTER_X, 500.0, true),
                (ac2_ui::UI_POINTER_Y, 350.0, true),
                (0x4000_0004, 1.0, true),
            ]
        );
    }
}
