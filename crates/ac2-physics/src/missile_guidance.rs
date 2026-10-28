//! `CMissileManager`'s guidance: the velocity a missile wants toward its target
//! (`GetDesiredVelocity`, `CollideVelocity`, `MaintDistVelocity`, `PerpAdjust`,
//! `InterceptVelocity`). The x87 spills follow the retail code.

use crate::fpu::{ld, st, F_EPSILON};
use crate::missile::{chase_velocity, targeting_bits, MissileManager};
use crate::position::Position;
use crate::vector::Vector;

fn negate(v: &mut Vector) {
    *v = Vector::new(st(-ld(v.x)), st(-ld(v.y)), st(-ld(v.z)));
}

impl MissileManager {
    /// `CMissileManager::GetDesiredVelocity`: the speed to fly at. Catching up flies at the
    /// initial speed when one was set, else the top speed. Conserving fuel flies at the target's
    /// closing speed plus the catch-up margin (and the speed that arrives before the lifespan
    /// ends), between the minimum and the top speed. `now` is `Timer::cur_time`.
    pub fn get_desired_velocity(
        &self,
        diff: Vector,
        away_speed: Vector,
        catch_up: bool,
        now: f64,
    ) -> f32 {
        let (min, max) = (self.info.min_velocity, self.info.max_velocity);
        if catch_up {
            return if self
                .target_info
                .test_bit_mask(targeting_bits::INITIAL_SPEED)
            {
                self.target_info.initial_velocity
            } else {
                max
            };
        }
        if !self.info.conserve_fuel {
            return max;
        }
        let d = st(
            ((ld(diff.x) * ld(diff.x) + ld(diff.y) * ld(diff.y)) + ld(diff.z) * ld(diff.z)).sqrt(),
        );
        let inverse = 1.0 / ld(d);
        let z_term = inverse * ld(diff.z) * ld(away_speed.z);
        let y_term = inverse * ld(diff.y) * ld(away_speed.y);
        let x_term = inverse * ld(diff.x) * ld(away_speed.x);
        let rate = st((z_term + y_term) + x_term);
        let catch_up_speed = ld(self.info.min_catchup_speed) + ld(rate);
        let mut speed = if catch_up_speed > ld(min) {
            st(catch_up_speed)
        } else {
            min
        };
        if self.info.consider_remaining_time {
            let remaining = self.missile_expire_time - now - 1.0;
            if remaining > ld(F_EPSILON) {
                let arrive = ld(d) / remaining + ld(rate);
                if arrive > ld(speed) {
                    speed = st(arrive);
                }
            }
        }
        if ld(speed) <= ld(max) {
            speed
        } else {
            max
        }
    }

    /// `CMissileManager::CollideVelocity`: the velocity that meets a target moving at
    /// `target_velocity`, `diff` away (the missile's position minus the target's), at the desired
    /// speed: the earliest non-negative root of the intercept quadratic. When there is none (the
    /// root is not above `F_EPSILON`) retail calls `ChaseVelocity` into a scratch vector and
    /// leaves `out` as it was. Then the climb (not for the first velocity) and the speed limits
    /// apply.
    pub fn collide_velocity(
        &self,
        diff: Vector,
        target_velocity: Vector,
        catch_up: bool,
        is_init: bool,
        now: f64,
        out: &mut Vector,
    ) {
        let (d, w) = (diff, target_velocity);
        let speed = self.get_desired_velocity(d, w, catch_up, now);
        let a =
            ld(speed) * ld(speed) - ((ld(w.x) * ld(w.x) + ld(w.y) * ld(w.y)) + ld(w.z) * ld(w.z));
        let dot = (ld(w.z) * ld(d.z) + ld(w.y) * ld(d.y)) + ld(d.x) * ld(w.x);
        let b = dot + dot;
        let d2 = (ld(d.x) * ld(d.x) + ld(d.y) * ld(d.y)) + ld(d.z) * ld(d.z);
        let discriminant = b * b - ((-d2) * a * 4.0);
        let (first, second) = if discriminant < 0.0 {
            (-1.0, -1.0)
        } else {
            let root = discriminant.sqrt();
            let inverse = 1.0 / (a + a);
            ((root - b) * inverse, (-b - root) * inverse)
        };
        let mut time = st(first);
        if second >= 0.0 && (first < 0.0 || second < first) {
            time = st(second);
        }
        if ld(time) > ld(F_EPSILON) {
            let inverse = 1.0 / ld(time);
            *out = Vector::new(
                st(ld(w.x) - inverse * ld(d.x)),
                st(ld(w.y) - ld(st(inverse * ld(d.y)))),
                st(ld(w.z) - ld(st(inverse * ld(d.z)))),
            );
        } else {
            let _scratch = chase_velocity(d, w, speed);
        }
        if !is_init {
            self.adjust_height(out);
        }
        self.adjust_speed(out);
    }

    /// `CMissileManager::PerpAdjust`: which way round the perpendicular turn goes. Method 0
    /// flips it, 1 keeps it; otherwise a missile with no minimum speed (method 2) goes straight,
    /// the first turn is flipped by a coin (`Random::Check(0.5)`), and later turns flip when
    /// they would turn against the current velocity.
    pub fn perp_adjust(&self, perp: &mut Vector, initial: bool, mut coin: impl FnMut(f32) -> bool) {
        match self.info.tight_turn_method {
            0 => negate(perp),
            1 => {}
            method => {
                if method == 2 && self.info.min_velocity == 0.0 {
                    *perp = Vector::new(
                        st(ld(perp.x) * 0.0),
                        st(ld(perp.y) * 0.0),
                        st(ld(perp.z) * 0.0),
                    );
                } else if initial {
                    if coin(0.5) {
                        negate(perp);
                    }
                } else {
                    let v = self.current_velocity;
                    let dot = (ld(v.x) * ld(perp.x) + ld(v.z) * ld(perp.z)) + ld(v.y) * ld(perp.y);
                    if dot < 0.0 {
                        negate(perp);
                    }
                }
            }
        }
    }

    /// `CMissileManager::MaintDistVelocity`: fly around the target at the maintain distance:
    /// the horizontal offset's perpendicular (turned by `perp_adjust`) scaled to the distance
    /// left to the ring, plus the target's velocity.
    pub fn maint_dist_velocity(
        &self,
        diff: Vector,
        target_velocity: Vector,
        initial: bool,
        is_init: bool,
        coin: impl FnMut(f32) -> bool,
        out: &mut Vector,
    ) {
        let maint = self.info.maintain_distance;
        let horizontal = (ld(diff.y) * ld(diff.y) + ld(diff.x) * ld(diff.x)).sqrt();
        let dxy = st(horizontal);
        let (x0, y0);
        if horizontal < ld(F_EPSILON) {
            let m = target_velocity;
            let length2 = (ld(m.x) * ld(m.x) + ld(m.y) * ld(m.y)) + ld(m.z) * ld(m.z);
            if length2 <= ld(F_EPSILON) {
                (x0, y0) = (1.0, 0.0);
            } else {
                (x0, y0) = (-ld(m.y), ld(m.x));
            }
        } else {
            let inverse = 1.0 / ld(dxy);
            let n_x = st(ld(diff.x) * inverse);
            let n_y = st(ld(diff.y) * inverse);
            let mut perp = Vector::new(-n_y, n_x, st(inverse * 0.0));
            self.perp_adjust(&mut perp, initial, coin);
            let m = if ld(dxy) > ld(maint) { maint } else { dxy };
            let tmp = st(ld(perp.y) * ld(m));
            let ring = ld(maint) - ld(dxy);
            let u1 = st(ld(n_x) * ring);
            let u2 = st(ring * ld(n_y));
            x0 = ld(perp.x) * ld(m) + ld(u1);
            y0 = ld(u2) + ld(tmp);
        }
        let mut v = Vector::new(
            st(x0 + ld(target_velocity.x)),
            st(y0 + ld(target_velocity.y)),
            st(ld(diff.z) + ld(target_velocity.z)),
        );
        if !is_init {
            self.adjust_height(&mut v);
        }
        self.adjust_speed(&mut v);
        *out = v;
    }

    /// `CMissileManager::InterceptVelocity`: the guidance toward `m_target_pos` from
    /// `missile_pos`; around the target at a distance when `use_maint_dist`, else the intercept.
    pub fn intercept_velocity(
        &self,
        missile_pos: &Position,
        initial: bool,
        is_init: bool,
        now: f64,
        coin: impl FnMut(f32) -> bool,
        out: &mut Vector,
    ) {
        let diff = self.target_pos.offset_pos.offset_to(missile_pos);
        let target_velocity = self.target_pos.target_velocity;
        if self.info.use_maint_dist {
            self.maint_dist_velocity(diff, target_velocity, initial, is_init, coin, out);
        } else {
            self.collide_velocity(diff, target_velocity, initial, is_init, now, out);
        }
    }
}

#[cfg(test)]
#[path = "missile_guidance_tests.rs"]
mod tests;
