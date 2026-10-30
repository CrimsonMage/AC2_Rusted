//! The player's mouse-hover fade (`CPhysicsObj::Draw` with `AvatarMouseHoverTransparency`): while
//! the pointer is over the third person avatar its opacity falls toward 0.3, and it recovers to 1
//! when the pointer leaves. The opacity multiplies the alpha of the avatar and of what hangs from
//! it (the draw takes the root parent's `m_fMouseHoverOpacity`).

/// The lowest opacity the fade reaches.
const MIN_OPACITY: f32 = 0.3;
/// The fade's speed: the opacity moves by `elapsed * opacity * 10` per draw.
const RATE: f64 = 10.0;

/// `CPhysicsObj::m_fMouseHoverOpacity` and `m_dLastDrawTime`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseHoverFade {
    opacity: f32,
    last_draw: f64,
}

impl Default for MouseHoverFade {
    /// `CPhysicsObj::CPhysicsObj`: fully opaque, last drawn at time zero.
    fn default() -> Self {
        Self {
            opacity: 1.0,
            last_draw: 0.0,
        }
    }
}

impl MouseHoverFade {
    /// The opacity the avatar draws with now.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// One draw of the avatar at client time `now`. `hovered`: the pointer is over the avatar
    /// (`TraceRay` hit) with the preference on and the camera in third person. The step is
    /// `(now - last) * opacity * 10` (evaluated in extended precision, so here in f64): the
    /// opacity falls by it while hovered, down to 0.3, and rises by it otherwise, up to 1.
    pub fn step(&mut self, now: f64, hovered: bool) {
        let delta = (now - self.last_draw) * f64::from(self.opacity) * RATE;
        if hovered {
            let next = f64::from(self.opacity) - delta;
            self.opacity = (next as f32).max(MIN_OPACITY);
        } else {
            let next = delta + f64::from(self.opacity);
            self.opacity = if next > 1.0 { 1.0 } else { next as f32 };
        }
        self.last_draw = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_avatar_is_opaque_and_the_first_hovered_draw_drops_to_the_floor() {
        let mut fade = MouseHoverFade::default();
        assert_eq!(fade.opacity(), 1.0);
        // Last drawn at time zero: the first elapsed time is the whole clock.
        fade.step(5.0, true);
        assert_eq!(fade.opacity(), 0.3);
    }

    #[test]
    fn hovering_fades_in_proportion_to_the_opacity_and_leaving_recovers_to_one() {
        let mut fade = MouseHoverFade::default();
        fade.step(0.0, false);
        // 10 ms at full opacity falls by 0.1 (elapsed 0.01 * 1 * 10).
        fade.step(0.01, true);
        assert!((fade.opacity() - 0.9).abs() < 1e-6, "{}", fade.opacity());
        // Smaller opacity, smaller step: 0.01 * 0.9 * 10 = 0.09.
        fade.step(0.02, true);
        assert!((fade.opacity() - 0.81).abs() < 1e-6, "{}", fade.opacity());
        for i in 3..400 {
            fade.step(f64::from(i) * 0.01, true);
        }
        assert_eq!(fade.opacity(), 0.3, "the fade never goes below 0.3");
        for i in 400..1000 {
            fade.step(f64::from(i) * 0.01, false);
        }
        assert_eq!(fade.opacity(), 1.0, "and recovers to full opacity");
    }
}
