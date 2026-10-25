//! `ConstraintManager` (`CMotionInterp::mConstraintManager`): after a forced position the body's
//! own movement slows to a stop between two distances from it (`CMotionInterp::
//! ConvertDoMotionToLocalDesired`'s constraint factor).

use crate::fpu::{ld, st};
use crate::position::Position;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConstraintManager {
    /// `m_is_constrained`: set by `ConstrainTo`, cleared by `UnConstrain`.
    pub constrained: bool,
    /// `m_constraint_enabled`: an interpolating body enables it only at the end of its path.
    pub enabled: bool,
    position: Position,
    start: f32,
    max: f32,
}

impl ConstraintManager {
    /// `CMotionInterp::MakeConstraintManager`: the player slows from 19 m and stops at 20 m,
    /// other bodies from 2 m to 4 m.
    pub(crate) fn new(player: bool) -> Self {
        let (start, max) = if player { (19.0, 20.0) } else { (2.0, 4.0) };
        Self {
            constrained: false,
            enabled: true,
            position: Position::default(),
            start,
            max,
        }
    }

    /// `ConstraintManager::ConstrainTo`.
    pub(crate) fn constrain_to(&mut self, position: Position) {
        self.constrained = true;
        self.position = position;
    }

    /// `ConstraintManager::GetConstraintFactor`: 1 unconstrained, disabled or within the start
    /// distance of the constraint position; past it falling linearly to 0 at the maximum.
    pub(crate) fn factor(&self, position: &Position) -> f32 {
        if !self.constrained || !self.enabled {
            return 1.0;
        }
        let distance = position.distance(&self.position);
        if distance < ld(self.start) || distance == ld(self.start) {
            return 1.0;
        }
        let stored = st(distance);
        if !(stored < self.max) {
            return 0.0;
        }
        st((ld(self.max) - ld(stored)) / (ld(self.max) - ld(self.start)))
    }
}
