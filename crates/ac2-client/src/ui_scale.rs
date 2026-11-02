//! The UI sized for a window of any size.
//!
//! Retail laid its UI out pixel for pixel on the display `Turbine::Client::InitUI` made
//! (`Client+0x60` / `Client+100`, 1024 x 768 unless `width` / `height` said otherwise): every
//! root is anchored to `RenderDevice::GetDisplayWidth` / `GetDisplayHeight`
//! (`UIElement::UpdateSizeAndPosition`, run for each root by `UIElementManager::RefreshEvent`),
//! so a bigger display held the same pixel-sized UI. This client's window resizes freely and
//! the player picks the size (`ac2_ui::UiScaleSetting`): the UI is laid out on a logical
//! display and drawn scaled, every UI pixel being `scale` window pixels.
//!
//! - [`UiScale::hud`] (the HUD): the logical display covers the window (`window / scale`),
//!   so at 1x it is the window itself, as retail's was (its smallest display mode was
//!   800 x 600). A scale that would leave the logical display under [`MIN_DISPLAY`] is held
//!   at the one that fits it. [`UiScale::fill`] is the largest scale that keeps the authored
//!   1024 x 768 inside the window, the first modern client's behaviour.
//! - [`UiScale::fit`] (the pregame screens): the logical display is exactly 1024 x 768, as
//!   retail showed them, centred in the window with the rest left black, and composed at
//!   its scale (`UiTree::set_render_scale`) rather than stretched, so it stays sharp. The
//!   scale is 1 unless the window is smaller.

use ac2_ui::UiScaleSetting;

/// The smallest logical display a fixed scale may leave (retail's smallest display mode, 800 x 600).
pub const MIN_DISPLAY: (u32, u32) = (800, 600);

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
    /// The largest scale that keeps the 1024 x 768 display inside the window.
    fn factor(window: (u32, u32)) -> f64 {
        let (w, h) = (f64::from(window.0.max(1)), f64::from(window.1.max(1)));
        (w / f64::from(BASE_DISPLAY.0)).min(h / f64::from(BASE_DISPLAY.1))
    }

    /// A logical display covering the whole window (the HUD) at the setting's scale.
    pub fn hud(window: (u32, u32), setting: UiScaleSetting) -> Self {
        let window = (window.0.max(1), window.1.max(1));
        let (scale, floor) = match setting.factor() {
            Some(f) => {
                let (w, h) = (f64::from(window.0), f64::from(window.1));
                let fits = (w / f64::from(MIN_DISPLAY.0)).min(h / f64::from(MIN_DISPLAY.1));
                (f.min(fits), MIN_DISPLAY)
            }
            None => (Self::factor(window), BASE_DISPLAY),
        };
        let display = (
            ((f64::from(window.0) / scale).round() as u32).max(floor.0),
            ((f64::from(window.1) / scale).round() as u32).max(floor.1),
        );
        Self {
            scale,
            origin: (0.0, 0.0),
            display,
            window,
        }
    }

    /// The HUD filling the window ([`UiScaleSetting::Fill`]).
    pub fn fill(window: (u32, u32)) -> Self {
        Self::hud(window, UiScaleSetting::Fill)
    }

    /// The fixed [`BASE_DISPLAY`] scaled to fit the window and centred (the pregame screens).
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
    use super::{UiScale, UiScaleSetting, BASE_DISPLAY};

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
        // The pregame screens are scaled to fit the window and centred.
        let fit = UiScale::fit((1920, 1080));
        assert_eq!(fit.display, BASE_DISPLAY);
        assert_eq!(fit.scale, 1080.0 / 768.0);
        assert_eq!(fit.origin, (240.0, 0.0));
        assert_eq!(fit.to_ui(240.0 + 720.0, 540.0), (512.0, 384.0));
    }

    #[test]
    fn the_default_hud_is_laid_out_on_the_real_display() {
        let hud = UiScale::hud((2560, 1440), UiScaleSetting::Retail);
        assert!(hud.is_identity());
        assert_eq!(hud.display, (2560, 1440));
    }

    #[test]
    fn a_fixed_scale_lays_out_on_the_window_over_the_scale() {
        let two = UiScale::hud((2560, 1440), UiScaleSetting::Percent(200));
        assert_eq!((two.scale, two.display), (2.0, (1280, 720)));
        let quarter = UiScale::hud((2560, 1440), UiScaleSetting::Percent(125));
        assert_eq!((quarter.scale, quarter.display), (1.25, (2048, 1152)));
        // The display never drops under 800 x 600: 3x on 1080p is held at 1.8x.
        let three = UiScale::hud((1920, 1080), UiScaleSetting::Percent(300));
        assert_eq!((three.scale, three.display), (1.8, (1067, 600)));
        // Retail's smallest mode is drawn pixel for pixel at 1x.
        assert!(UiScale::hud((800, 600), UiScaleSetting::Retail).is_identity());
    }

    #[test]
    fn fill_is_the_first_modern_clients_scale() {
        let fill = UiScale::hud((2560, 1440), UiScaleSetting::Fill);
        assert_eq!(fill.scale, 1440.0 / 768.0);
        assert_eq!(fill.display, (1365, 768));
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
        let fit = UiScale::hud((2048, 1536), UiScaleSetting::Percent(200));
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
