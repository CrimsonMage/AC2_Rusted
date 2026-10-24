//! `InterpolationManager::InterpolationPath`: the queue of server positions a remote object
//! steers along, each with the path distance retail keeps beside it, and the lookups the
//! manager makes on it.

mod manager;

pub(crate) use manager::InterpolationManager;

use std::collections::VecDeque;

use crate::fpu::{ld, st, F_EPSILON};
use crate::position::Position;
use crate::vector::Vector;

/// `InterpolationManager::QueuedPosition`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct QueuedPosition {
    pub pos: Position,
    pub time_sent: f64,
    /// The sender's do-motion, each component in `[-1, 1]`.
    pub local_motion: Vector,
    pub impulse: bool,
    pub impulse_stamp: u32,
    pub impulse_is_jump: bool,
    pub has_contact: bool,
    pub velocity_at_impulse: Vector,
}

/// A node and the path distance stored with it (`pair<QueuedPosition, float>::second`).
type Node = (QueuedPosition, f32);

#[derive(Debug, Clone, Default)]
pub(crate) struct InterpolationPath {
    pub last_impulse_stamp: u32,
    pub next_impulse_stamp: u32,
    pub queue: VecDeque<Node>,
}

/// The two nodes a path distance falls between: `hi == len` is the end of the queue.
#[derive(Debug, Clone, Copy)]
struct Keyframes {
    lo: usize,
    hi: usize,
}

/// `clamp(x, lo, hi)` as `FindClosestPoint` and its kin inline it on `float`s.
fn clamp01(value: f32) -> f32 {
    let value = if 1.0 <= value { 1.0 } else { value };
    if 0.0 < value {
        value
    } else {
        0.0
    }
}

impl InterpolationPath {
    pub(crate) fn clear(&mut self) {
        self.queue.clear();
        self.last_impulse_stamp = 0;
        self.next_impulse_stamp = 0;
    }

    fn pending_impulse(&self, node: &QueuedPosition) -> bool {
        node.impulse && self.last_impulse_stamp < node.impulse_stamp
    }

    /// `InterpolationPath::Add`: queue `qp` (an impulse only if newer than the last one taken)
    /// unless older than the last node; a position on top of the last one updates it instead.
    pub(crate) fn add(&mut self, qp: &QueuedPosition) {
        if qp.impulse && !(self.last_impulse_stamp < qp.impulse_stamp) {
            return;
        }
        let mut total = 0.0_f32;
        if let Some((back, back_distance)) = self.queue.back_mut() {
            if qp.time_sent < back.time_sent {
                return;
            }
            let d = back.pos.distance(&qp.pos);
            total = st(ld(*back_distance) + d);
            if d < ld(F_EPSILON) && (!back.impulse || !qp.impulse) {
                back.pos.frame.set_heading(qp.pos.frame.heading());
                back.time_sent = qp.time_sent;
                back.local_motion = qp.local_motion;
                back.has_contact = qp.has_contact;
                if qp.impulse {
                    back.impulse = true;
                    back.impulse_stamp = qp.impulse_stamp;
                    back.velocity_at_impulse = qp.velocity_at_impulse;
                    back.impulse_is_jump = qp.impulse_is_jump;
                }
                return;
            }
        }
        self.queue.push_back((*qp, total));
    }

    /// `InterpolationPath::FindKeyframesForDistance`: the first node whose distance reaches
    /// `distance` (or that holds an impulse not yet taken) and the node before it.
    fn keyframes(&self, distance: f32) -> Option<Keyframes> {
        if !(0.0 <= distance) || self.queue.is_empty() {
            return None;
        }
        let mut hi = 0;
        while hi < self.queue.len() {
            let (node, node_distance) = &self.queue[hi];
            if distance <= *node_distance || self.pending_impulse(node) {
                break;
            }
            hi += 1;
        }
        let lo = hi.saturating_sub(1);
        Some(Keyframes { lo, hi })
    }

    /// `InterpolationPath::GetPointFromDistance`: the point `distance` along the path,
    /// extrapolated along the last node's motion past the end, held at a pending impulse.
    pub(crate) fn point_from_distance(&self, distance: f32) -> Position {
        let Some(k) = self.keyframes(distance) else {
            return Position::default();
        };
        let (lo, lo_distance) = &self.queue[k.lo];
        let rem = st(ld(distance) - ld(*lo_distance));
        if self.pending_impulse(lo) {
            return lo.pos;
        }
        if k.hi == self.queue.len() {
            let mut out = lo.pos;
            let r = ld(rem);
            let m = lo.local_motion;
            out.add_local_offset(Vector::new(
                st(r * ld(m.x)),
                st(r * ld(m.y)),
                st(r * ld(m.z)),
            ));
            return out;
        }
        let (hi, hi_distance) = &self.queue[k.hi];
        let off = lo.pos.offset_to(&hi.pos);
        let (x, y, z) = (ld(off.x), ld(off.y), ld(off.z));
        let length = ((z * z + y * y) + x * x).sqrt();
        let r = ld(rem);
        let (sx, sy, sz) = if length < ld(F_EPSILON) {
            (0.0, 0.0, 0.0)
        } else {
            let inverse = 1.0 / length;
            (
                ld(st(inverse * x)) * r,
                ld(st(y * inverse)) * r,
                ld(st(z * inverse)) * r,
            )
        };
        let segment = st(ld(*hi_distance) - ld(*lo_distance));
        let t = if ld(segment) > 0.0 {
            let travelled = ((sz * sz + sy * sy) + sx * sx).sqrt();
            clamp01(st(travelled / ld(segment)))
        } else {
            0.0
        };
        let mut out = Position::default();
        out.lerp_origin(&lo.pos, &hi.pos, t);
        out.frame.slerp_rotation(&lo.pos.frame, &hi.pos.frame, t);
        out
    }

    /// `InterpolationPath::HasContact`: the contact flag of the node before `distance`.
    pub(crate) fn has_contact(&self, distance: f32) -> bool {
        self.keyframes(distance)
            .is_some_and(|k| self.queue[k.lo].0.has_contact)
    }

    /// `InterpolationPath::ShouldImpulseHere`: the next impulse's position, velocity and jump
    /// flag when `distance` is within 0.1 of a node carrying it. Retail takes the stamp of the
    /// later node and the jump flag of the earlier one in both cases (the stamp past the end of
    /// the queue reads as 0 here).
    pub(crate) fn should_impulse_here(&self, distance: f32) -> Option<(Position, Vector, bool)> {
        let k = self.keyframes(distance)?;
        let next = self.last_impulse_stamp.wrapping_add(1);
        let near = |node_distance: f32| 0.1 > (ld(distance) - ld(node_distance)).abs();
        let (lo, lo_distance) = &self.queue[k.lo];
        let hi = self.queue.get(k.hi);
        let hi_stamp = hi.map_or(0, |(node, _)| node.impulse_stamp);
        if lo.impulse && hi_stamp == next && near(*lo_distance) {
            return Some((lo.pos, lo.velocity_at_impulse, lo.impulse_is_jump));
        }
        let (hi, hi_distance) = hi?;
        if hi.impulse && hi.impulse_stamp == next && near(*hi_distance) {
            return Some((hi.pos, hi.velocity_at_impulse, lo.impulse_is_jump));
        }
        None
    }

    /// `InterpolationPath::GetInterpolatedValuesForDistance`: the motion between the nodes
    /// around `distance` and the earlier node's heading.
    pub(crate) fn interpolated_values(
        &self,
        distance: f32,
    ) -> Option<(Vector, crate::heading::Heading)> {
        let k = self.keyframes(distance)?;
        let (lo, lo_distance) = &self.queue[k.lo];
        let heading = lo.pos.frame.heading();
        let Some((hi, _)) = self.queue.get(k.hi) else {
            return Some((lo.local_motion, heading));
        };
        let rem = st(ld(distance) - ld(*lo_distance));
        let off = lo.pos.offset_to(&hi.pos);
        let (x, y, z) = (ld(off.x), ld(off.y), ld(off.z));
        let length = ((z * z + y * y) + x * x).sqrt();
        let t = if length > 0.0 {
            clamp01(st(ld(rem) / length))
        } else {
            0.0
        };
        let t = ld(t);
        let (a, b) = (lo.local_motion, hi.local_motion);
        let lerp = |a: f32, b: f32| st((ld(b) - ld(a)) * t + ld(a));
        Some((
            Vector::new(lerp(a.x, b.x), lerp(a.y, b.y), lerp(a.z, b.z)),
            heading,
        ))
    }

    pub(crate) fn last(&self) -> Option<&QueuedPosition> {
        self.queue.back().map(|(node, _)| node)
    }

    /// `InterpolationPath::GetTotalDistance`.
    pub(crate) fn total_distance(&self) -> f32 {
        self.queue.back().map_or(0.0, |(_, distance)| *distance)
    }

    /// `InterpolationPath::ThrowAway`: drop the nodes behind `distance` (keeping the one just
    /// behind it and any impulse not yet taken), then store each kept node's distance as the
    /// length of the segment to it. Retail stores segment lengths here, not running totals,
    /// and `Add` sums onto them; that is kept. Returns how many nodes were dropped.
    pub(crate) fn throw_away(&mut self, distance: f32) -> usize {
        if !(0.0 <= distance) || self.queue.is_empty() {
            return 0;
        }
        let behind = self
            .queue
            .iter()
            .take_while(|(_, node_distance)| !(distance < *node_distance))
            .count();
        if behind == 0 {
            return 0;
        }
        let mut remaining = behind - 1;
        while remaining != 0 {
            let front = &self.queue[0].0;
            if front.impulse && !(front.impulse_stamp <= self.last_impulse_stamp) {
                break;
            }
            self.queue.pop_front();
            remaining -= 1;
        }
        let mut length = 0.0_f64;
        for index in 0..self.queue.len() {
            self.queue[index].1 = st(length);
            if index + 1 < self.queue.len() {
                length = self.queue[index]
                    .0
                    .pos
                    .distance(&self.queue[index + 1].0.pos);
            }
        }
        behind - 1 - remaining
    }

    /// `InterpolationPath::FindClosestPoint`: the point of the path nearest `current` (the
    /// segment past the last node extrapolates the last motion over the time since it was
    /// sent), its distance along the path and whether it lies on that extrapolation. A pending
    /// impulse stops the search at its node.
    pub(crate) fn closest_point(
        &self,
        current: &Position,
        now: f64,
        desired_velocity: &dyn Fn(Vector) -> Vector,
    ) -> (Position, f32, bool) {
        let mut best = f64::from(f32::MAX);
        let mut found: Option<(Position, f32, bool)> = None;
        for index in 0..self.queue.len() {
            let (node, node_distance) = &self.queue[index];
            let start = node.pos;
            if self.pending_impulse(node) {
                return (node.pos, *node_distance, false);
            }
            let next = self.queue.get(index + 1).map(|(next, _)| next);
            let end = match next {
                Some(next) => next.pos,
                None => {
                    let velocity = desired_velocity(node.local_motion);
                    let dt = ld(st(now - node.time_sent));
                    let mut end = start;
                    end.add_local_offset(Vector::new(
                        st(ld(velocity.x) * dt),
                        st(ld(velocity.y) * dt),
                        st(ld(velocity.z) * dt),
                    ));
                    end
                }
            };
            let block = crate::landdefs::get_block_offset(start.cell_id, end.cell_id);
            let (so, eo) = (start.frame.origin, end.frame.origin);
            let dx = st((ld(block.x) + ld(eo.x)) - ld(so.x));
            let dy = st((ld(block.y) + ld(eo.y)) - ld(so.y));
            let dz = st(ld(st(ld(block.z) + ld(eo.z))) - ld(so.z));
            let to_current = crate::landdefs::get_block_offset(start.cell_id, current.cell_id);
            let co = current.frame.origin;
            let qx = ld(st((ld(to_current.x) + ld(co.x)) - ld(so.x)));
            let qy = (ld(to_current.y) + ld(co.y)) - ld(so.y);
            let qz = ld(st(ld(to_current.z) + ld(co.z))) - ld(so.z);
            let (dx, dy, dz) = (ld(dx), ld(dy), ld(dz));
            let dd = (dx * dx + dz * dz) + dy * dy;
            let u = if dd > 0.0 {
                clamp01(st(((qx * dx + qz * dz) + qy * dy) / dd))
            } else {
                0.0
            };
            let u = ld(u);
            let (px, py, pz) = (st(dx * u), st(dy * u), st(dz * u));
            let closest = Vector::new(
                st(ld(px) + ld(so.x)),
                st(ld(py) + ld(so.y)),
                st(ld(pz) + ld(so.z)),
            );
            // Retail measures against the current origin without its landblock offset.
            let ex = ld(co.x) - ld(closest.x);
            let ey = ld(co.y) - ld(closest.y);
            let ez = ld(co.z) - ld(closest.z);
            let distance2 = (ex * ex + ez * ez) + ey * ey;
            if distance2 <= best {
                best = ld(st(distance2));
                let (px, py, pz) = (ld(px), ld(py), ld(pz));
                let along = st((px * px + pz * pz + py * py).sqrt());
                let segment = end.distance(&start);
                let t = if segment > 0.0 {
                    clamp01(st(ld(along) / segment))
                } else {
                    0.0
                };
                let mut point = Position::default();
                point.lerp_origin(&start, &end, t);
                point.frame.slerp_rotation(&start.frame, &end.frame, t);
                let along = st(ld(along) + ld(*node_distance));
                found = Some((point, along, next.is_none()));
            }
            if let Some(next) = next {
                if self.pending_impulse(next) {
                    break;
                }
            }
        }
        found.unwrap_or_else(|| {
            let first = self
                .queue
                .front()
                .map_or(Position::default(), |(node, _)| node.pos);
            (first, 0.0, false)
        })
    }
}

#[cfg(test)]
mod tests;
