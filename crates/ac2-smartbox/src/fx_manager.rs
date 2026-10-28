//! The `FXManager` effects that act on a body or decide what the appearance does: the slider
//! effect (`DoSliderFX`) and the appearance effect's apply-or-remove choice (`DoAppearanceFX`).

use crate::remotes::Remotes;

/// What `FXManager::DoAppearanceFX` asks `PHYSICSOBJREP` to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppearanceFxStep {
    /// No appearance id: nothing happens.
    None,
    /// `PHYSICSOBJREP::DoAppearanceFX(apr_id, key)` with the key's scalar times the FX scalar.
    Apply { scalar: f32 },
    /// `PHYSICSOBJREP::RemoveAppearanceFX(apr_id, key)`: the scaled scalar is not positive.
    Remove,
}

/// `FXManager::DoAppearanceFX`: the key's scalar is multiplied by the FX scalar (stored as
/// `float`); with an appearance id, a positive result applies the appearance and any other
/// removes it.
pub fn appearance_fx_step(apr_id: u32, key_scalar: f32, scalar: f32) -> AppearanceFxStep {
    let scaled = (f64::from(key_scalar) * f64::from(scalar)) as f32;
    if apr_id == 0 {
        AppearanceFxStep::None
    } else if 0.0 < scaled {
        AppearanceFxStep::Apply { scalar: scaled }
    } else {
        AppearanceFxStep::Remove
    }
}

impl<B> Remotes<B> {
    /// `FXManager::DoSliderFX`: the emitter's `PHYSICSOBJREP::DoSlider(0, slider, scalar, ramp)`;
    /// with `propagate`, `PHYSICSOBJREP::DoSliderOnChild` then does the same on each of the
    /// emitter's direct children (`CPhysicsObj::DoSliderOnChild`), `children` in list order.
    /// Returns how many bodies moved the slider.
    pub fn do_slider_fx(
        &mut self,
        emitter: u64,
        propagate: bool,
        slider: u32,
        scalar: f32,
        ramp_time: f32,
        children: &[u64],
    ) -> usize {
        let mut moved = 0;
        let Some(body) = self.body_mut(emitter) else {
            return 0;
        };
        body.do_slider(0, slider, scalar, ramp_time);
        moved += 1;
        if propagate {
            for &child in children {
                if let Some(body) = self.body_mut(child) {
                    body.do_slider(0, slider, scalar, ramp_time);
                    moved += 1;
                }
            }
        }
        moved
    }
}

#[cfg(test)]
mod tests {
    use ac2_physics::{BodyShape, PhysicsBody};

    use super::*;
    use crate::remotes::Remote;

    fn remote_with_body() -> Remote {
        let shape = BodyShape {
            spheres: Vec::new(),
            height: 1.0,
            step_up_height: 0.1,
            step_down_height: 0.1,
            scale: 1.0,
            allow_free_heading: false,
        };
        Remote {
            visual: None,
            body: Some(PhysicsBody::new(Default::default(), shape, None)),
            failed: false,
            stored: None,
            in_world: true,
            lost_cell: None,
            active: true,
            player_distance: 0.0,
            object: None,
            shadow: None,
            ethereal: false,
            alpha_moves: None,
            fx_follow: None,
            listed_cell: None,
        }
    }

    #[test]
    fn a_slider_effect_moves_the_emitter_and_with_propagation_its_children() {
        const SLIDER: u32 = 0x4000_0015;
        let mut remotes = Remotes::<()>::default();
        for id in [1, 2, 3] {
            remotes.insert(id, remote_with_body());
        }
        // Not propagated: only the emitter's slider moves.
        assert_eq!(
            remotes.do_slider_fx(1, false, SLIDER, 0.25, 0.0, &[2, 3]),
            1
        );
        let desired = |remotes: &Remotes<()>, id| {
            remotes
                .get(id)
                .unwrap()
                .body()
                .unwrap()
                .slider_desired_value(SLIDER)
        };
        assert_eq!(desired(&remotes, 1), 0.25);
        assert_ne!(desired(&remotes, 2), 0.25);
        // Propagated: each listed child (here one is not tracked) follows.
        assert_eq!(
            remotes.do_slider_fx(1, true, SLIDER, 0.5, 0.0, &[2, 99, 3]),
            3
        );
        assert_eq!((desired(&remotes, 2), desired(&remotes, 3)), (0.5, 0.5));
        // An emitter without a body does nothing.
        assert_eq!(remotes.do_slider_fx(99, true, SLIDER, 0.5, 0.0, &[2]), 0);
    }

    #[test]
    fn the_appearance_applies_while_the_scaled_scalar_is_positive() {
        assert_eq!(appearance_fx_step(0, 1.0, 1.0), AppearanceFxStep::None);
        assert_eq!(
            appearance_fx_step(7, 0.5, 4.0),
            AppearanceFxStep::Apply { scalar: 2.0 }
        );
        assert_eq!(appearance_fx_step(7, 0.5, 0.0), AppearanceFxStep::Remove);
        assert_eq!(appearance_fx_step(7, -1.0, 1.0), AppearanceFxStep::Remove);
    }
}
