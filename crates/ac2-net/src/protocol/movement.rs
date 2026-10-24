//! Movement, missile and story arguments of the per-object physics events:
//! `MovementParameters::StreamPack` (admin 0x0069C550, Legions 0x0063AFE0),
//! `TargetPosition::StreamPack` (admin 0x00496C10, Legions 0x0048F3E0) and
//! `PhysicsStory::StreamPack` with its `StoryHookData` (Legions 0x006477E0, 0x00646C70); each
//! Legions copy is instruction-identical to admin.

use crate::protocol::physics::BehaviorParams;
use crate::protocol::reader::Reader;
use crate::protocol::types::Position;
use crate::protocol::ProtocolError;

/// Where a move-to goes: an object (flag `0x20`) or a position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveTarget {
    Object(u64),
    Position(Position),
}

/// `MovementParameters::StreamPack`. Flag bits: 1 move towards, 2 turn, 4 move away,
/// 8 persistent, `0x10` persistent fail exits, `0x20` object target, `0x40` use cylinders,
/// `0x100` offset vector, `0x200` offset (callback) id.
#[derive(Debug, Clone, PartialEq)]
pub struct MovementParameters {
    pub flags: u32,
    /// Flags 1 or 4: desired velocity, fail distance, desired distance.
    pub desired_velocity: Option<f32>,
    pub fail_distance: Option<f32>,
    pub desired_distance: Option<f32>,
    /// Flag 1.
    pub max_success_distance: Option<f32>,
    /// Flag 4.
    pub min_success_distance: Option<f32>,
    pub target: MoveTarget,
    /// Zero when flag `0x100` is clear.
    pub offset: [f32; 3],
    /// Zero when flag `0x200` is clear.
    pub offset_id: u32,
    /// `PList<Position>::UnPack`: a u32 count, then the positions.
    pub positions: Vec<Position>,
    pub context_id: u32,
}

impl MovementParameters {
    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let flags = r.u32()?;
        let (mut desired_velocity, mut fail_distance, mut desired_distance) = (None, None, None);
        if flags & 5 != 0 {
            desired_velocity = Some(r.f32()?);
            fail_distance = Some(r.f32()?);
            desired_distance = Some(r.f32()?);
        }
        let max_success_distance = (flags & 1 != 0).then(|| r.f32()).transpose()?;
        let min_success_distance = (flags & 4 != 0).then(|| r.f32()).transpose()?;
        let target = if flags & 0x20 != 0 {
            MoveTarget::Object(r.u64()?)
        } else {
            MoveTarget::Position(Position::read(r)?)
        };
        let offset = if flags & 0x100 != 0 {
            r.vec3()?
        } else {
            [0.0; 3]
        };
        let offset_id = if flags & 0x200 != 0 { r.u32()? } else { 0 };
        let positions = (0..r.count()?)
            .map(|_| Position::read(r))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            flags,
            desired_velocity,
            fail_distance,
            desired_distance,
            max_success_distance,
            min_success_distance,
            target,
            offset,
            offset_id,
            positions,
            context_id: r.u32()?,
        })
    }
}

/// `TargetPosition::StreamPack`: target, offset position, target velocity and whether the
/// target was initialized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetPosition {
    pub target: u64,
    pub offset: Position,
    pub velocity: [f32; 3],
    pub initialized: u32,
}

impl TargetPosition {
    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            target: r.u64()?,
            offset: Position::read(r)?,
            velocity: r.vec3()?,
            initialized: r.u32()?,
        })
    }
}

/// `StoryHookData::StreamPack`: a pack bitfield whose low four bits are the hook type (1
/// damage, 2 create missiles, 3 fire missile), then the fields it names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StoryHook {
    pub bitfield: u32,
    pub target: Option<u64>,
    pub weapon: Option<u64>,
    pub attack_result: Option<u32>,
    pub anim_hook: Option<u32>,
    pub attack_hook: Option<u32>,
    pub target_health_change: Option<i32>,
    pub attacker_health_change: Option<i32>,
    pub target_pk_damage_change: Option<i32>,
    pub attacker_pk_damage_change: Option<i32>,
}

impl StoryHook {
    pub fn kind(&self) -> u32 {
        self.bitfield & 0xF
    }

    fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bits = r.u32()?;
        let has = |bit: u32| bits & bit != 0;
        let mut hook = Self {
            bitfield: bits,
            ..Self::default()
        };
        if has(0x100) {
            hook.target = Some(r.u64()?);
        }
        if has(0x20) {
            hook.weapon = Some(r.u64()?);
        }
        let mut word = |bit: u32| has(bit).then(|| r.u32()).transpose();
        hook.attack_result = word(0x80)?;
        hook.anim_hook = word(0x200)?;
        hook.attack_hook = word(0x400)?;
        hook.target_health_change = word(0x800)?.map(|v| v as i32);
        hook.attacker_health_change = word(0x1000)?.map(|v| v as i32);
        hook.target_pk_damage_change = word(0x2000)?.map(|v| v as i32);
        hook.attacker_pk_damage_change = word(0x4000)?.map(|v| v as i32);
        Ok(hook)
    }
}

/// `PhysicsStory::StreamPack`: a pack bitfield, the fields it names, then the hooks
/// (`STREAMPACK_STL_SET`: a u32 count, then each `StoryHookData`). Bit `0x20` is
/// `m_basicAttack` itself.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhysicsStory {
    pub bitfield: u32,
    pub attacker: Option<u64>,
    pub attacker_behavior: Option<BehaviorParams>,
    pub client_attack_context: Option<u32>,
    pub skill: Option<u32>,
    /// The hooks in stream order (retail keeps them in a `multiset`, sorted).
    pub hooks: Vec<StoryHook>,
}

impl PhysicsStory {
    pub fn basic_attack(&self) -> bool {
        self.bitfield & 0x20 != 0
    }

    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bits = r.u32()?;
        let attacker = (bits & 2 != 0).then(|| r.u64()).transpose()?;
        let attacker_behavior = (bits & 4 != 0)
            .then(|| BehaviorParams::read(r))
            .transpose()?;
        let client_attack_context = (bits & 8 != 0).then(|| r.u32()).transpose()?;
        let skill = (bits & 0x10 != 0).then(|| r.u32()).transpose()?;
        let hooks = (0..r.count()?)
            .map(|_| StoryHook::read(r))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            bitfield: bits,
            attacker,
            attacker_behavior,
            client_attack_context,
            skill,
            hooks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MoveTarget, MovementParameters, PhysicsStory};
    use crate::protocol::reader::Reader;

    fn put(out: &mut Vec<u8>, words: &[u32]) {
        out.extend(words.iter().flat_map(|w| w.to_le_bytes()));
    }

    fn position(out: &mut Vec<u8>, cell: u32) {
        put(out, &[cell]);
        for v in [1.0f32, 2.0, 3.0, 1.0, 0.0, 0.0, 0.0] {
            out.extend(v.to_le_bytes());
        }
    }

    #[test]
    fn move_to_an_object_with_offset_and_waypoints() {
        let mut b = Vec::new();
        put(&mut b, &[0x1 | 0x20 | 0x100 | 0x200]);
        for v in [2.5f32, 0.5, 1.0, 3.0] {
            b.extend(v.to_le_bytes()); // velocity, fail, desired, max success
        }
        b.extend(0x0200_0000_0000_0011u64.to_le_bytes());
        for v in [0.0f32, 1.0, 0.0] {
            b.extend(v.to_le_bytes());
        }
        put(&mut b, &[9, 1]); // offset id, one waypoint
        position(&mut b, 0x0299_0109);
        put(&mut b, &[77]);
        let mut r = Reader::new(&b);
        let p = MovementParameters::read(&mut r).unwrap();
        r.finish().unwrap();
        assert_eq!(p.desired_velocity, Some(2.5));
        assert_eq!(
            (p.max_success_distance, p.min_success_distance),
            (Some(3.0), None)
        );
        assert_eq!(p.target, MoveTarget::Object(0x0200_0000_0000_0011));
        assert_eq!((p.offset, p.offset_id), ([0.0, 1.0, 0.0], 9));
        assert_eq!(p.positions[0].cell, 0x0299_0109);
        assert_eq!(p.context_id, 77);
        for end in 0..b.len() {
            assert!(MovementParameters::read(&mut Reader::new(&b[..end])).is_err());
        }
    }

    #[test]
    fn turn_only_moves_to_a_position_with_no_distances() {
        let mut b = Vec::new();
        put(&mut b, &[0x2]);
        position(&mut b, 0x0299_010A);
        put(&mut b, &[0, 0]);
        let p = MovementParameters::read(&mut Reader::new(&b)).unwrap();
        assert_eq!(p.desired_velocity, None);
        assert!(matches!(p.target, MoveTarget::Position(pos) if pos.cell == 0x0299_010A));
    }

    #[test]
    fn story_hooks_follow_their_bitfields() {
        let mut b = Vec::new();
        put(&mut b, &[0x2 | 0x10 | 0x20]);
        b.extend(5u64.to_le_bytes());
        put(&mut b, &[0x4000_0001, 2]); // skill, two hooks
        put(&mut b, &[1 | 0x100 | 0x800]);
        b.extend(6u64.to_le_bytes());
        put(&mut b, &[(-12i32) as u32]);
        put(&mut b, &[3 | 0x80 | 0x400, 2, 4]);
        let mut r = Reader::new(&b);
        let s = PhysicsStory::read(&mut r).unwrap();
        r.finish().unwrap();
        assert!(s.basic_attack());
        assert_eq!((s.attacker, s.skill), (Some(5), Some(0x4000_0001)));
        assert_eq!(s.hooks[0].kind(), 1);
        assert_eq!(s.hooks[0].target_health_change, Some(-12));
        assert_eq!(
            (s.hooks[1].attack_result, s.hooks[1].attack_hook),
            (Some(2), Some(4))
        );
    }
}
