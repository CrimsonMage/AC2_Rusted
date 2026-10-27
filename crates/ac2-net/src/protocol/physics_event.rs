//! Per-object physics events (`ECM_Physics::RecvEvt_*`): each starts with the sender's
//! `InstanceIDWithStamp`, then its arguments, every one followed by `STREAMPACK_ALIGN`.
//! Argument names follow the `CPhysicsObj::RecvEvt_*` signatures they are passed to.

use crate::protocol::movement::{MovementParameters, PhysicsStory, TargetPosition};
use crate::protocol::opcode::Opcode;
use crate::protocol::physics::BehaviorParams;
use crate::protocol::reader::Reader;
use crate::protocol::types::{InstanceIdWithStamp, Position};
use crate::protocol::visual::VisualDesc;
use crate::protocol::ProtocolError;

/// What a per-object physics event asks of its object.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectAction {
    /// `0x10082`, `0x100A9`, `0x10095`, `0x10080`, `0x10094`, `0x100A7`, `0x100BE`, `0x100A1`:
    /// one 32-bit id (mode, layer, behavior, motion-interp DID, modifier, script, FX).
    DoMode(u32),
    SetMode(u32),
    StopLayer(u32),
    StopBehavior(u32),
    SetMotionInterp(u32),
    SetGlobalModifier(u32),
    DoFxScript(u32),
    StopFx(u32),
    /// `0x10092`, `0x10093`, `0x100BA`, `0x10091`: one float.
    SetVelocityScale(f32),
    SetAccelerationScale(f32),
    SetJumpScale(f32),
    SetHoverHeight(f32),
    /// `0x100A0`: FX id and scalar.
    DoFx {
        fx: u32,
        scalar: f32,
    },
    /// `0x100A2`: FX id, scalar and target.
    DoTargettedFx {
        fx: u32,
        scalar: f32,
        target: u64,
    },
    /// `0x10081`: slider id, new value, time.
    DoSlider {
        slider: u32,
        value: f32,
        time: f32,
    },
    /// `0x10096`: the target object (`INVALID_IID` clears).
    LookAt(u64),
    /// `0x100C9`: head direction.
    LookAtDir {
        x: f32,
        y: f32,
    },
    /// `0x100BF`: the position stamp.
    LeaveWorld(u16),
    /// `0x10087`: the parent with the child's position stamp, the location and orientation.
    Parent {
        parent: InstanceIdWithStamp,
        location: u32,
        orientation: u32,
    },
    /// `0x100C3`: the child with its position stamp.
    DeParent(InstanceIdWithStamp),
    /// `0x10098`: direction and contact.
    AddImpulse {
        direction: [f32; 3],
        contact: u32,
    },
    /// `0x10099`: target, magnitude, falloff and vertical impulse.
    AddTargetedImpulse {
        target: u64,
        magnitude: f32,
        falloff: u32,
        vertical: f32,
    },
    /// `0x100A5`: terrain type and modifier.
    AddTerrainModifier {
        terrain: u32,
        modifier: u32,
    },
    /// `0x1007F`.
    DoBehavior(Box<BehaviorParams>),
    /// `0x1009B`: the behavior id and its parameters.
    ReleaseBehavior {
        behavior: u32,
        params: Box<BehaviorParams>,
    },
    /// `0x10083`: the movement and its move-to stamp.
    MoveTo {
        params: Box<MovementParameters>,
        stamp: u16,
    },
    /// `0x100A8`.
    DoStory(Box<PhysicsStory>),
    /// `0x100B5`: launch position, velocity and target.
    InitializeMissile {
        position: Position,
        velocity: [f32; 3],
        target: Box<TargetPosition>,
    },
    /// `0x1009F`: a delta `VisualDesc`.
    UpdateVisualDesc(Box<VisualDesc>),
    /// `0x100C5`, `0x100CF`, `0x100A6`: no arguments.
    StopAllWeenieBehaviors,
    StopMoveToCancelsBehaviors,
    ClearTerrainModifiers,
}

/// A per-object physics event.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectEvent {
    pub sender: InstanceIdWithStamp,
    pub action: ObjectAction,
}

/// The opcodes [`ObjectEvent::read`] decodes.
const HANDLED: [u32; 32] = [
    Opcode::DO_MODE,
    Opcode::SET_MODE,
    Opcode::STOP_LAYER,
    Opcode::STOP_BEHAVIOR,
    Opcode::SET_MIDESC,
    Opcode::SET_GLOBAL_MODIFIER,
    Opcode::DO_FX_SCRIPT,
    Opcode::STOP_FX,
    Opcode::SET_VELOCITY_SCALE,
    Opcode::SET_ACCELERATION_SCALE,
    Opcode::SET_JUMP_SCALE,
    Opcode::SET_HOVER_HEIGHT,
    Opcode::DO_FX,
    Opcode::DO_TARGETTED_FX,
    Opcode::DO_SLIDER,
    Opcode::LOOK_AT,
    Opcode::LOOK,
    Opcode::LEAVE_WORLD,
    Opcode::PARENT,
    Opcode::DE_PARENT,
    Opcode::ADD_IMPULSE,
    Opcode::DO_BEHAVIOR,
    Opcode::UPDATE_VISUAL_DESC,
    Opcode::STOP_ALL_WEENIE_BEHAVIORS,
    Opcode::STOP_MOVE_TO_CANCELS_BEHAVIORS,
    Opcode::ADD_TARGETED_IMPULSE,
    Opcode::ADD_TERRAIN_MODIFIER,
    Opcode::RELEASE_BEHAVIOR,
    Opcode::MOVE_TO,
    Opcode::DO_STORY,
    Opcode::INITIALIZE_MISSILE,
    Opcode::CLEAR_TERRAIN_MODIFIERS,
];

fn word(r: &mut Reader<'_>) -> Result<u32, ProtocolError> {
    let v = r.u32()?;
    r.align()?;
    Ok(v)
}

fn float(r: &mut Reader<'_>) -> Result<f32, ProtocolError> {
    Ok(f32::from_bits(word(r)?))
}

fn id(r: &mut Reader<'_>) -> Result<u64, ProtocolError> {
    let v = r.u64()?;
    r.align()?;
    Ok(v)
}

impl ObjectEvent {
    /// Decodes `opcode`'s body, or `None` when the opcode is not a per-object event handled here.
    pub(crate) fn read(opcode: u32, r: &mut Reader<'_>) -> Option<Result<Self, ProtocolError>> {
        if !HANDLED.contains(&opcode) {
            return None;
        }
        let decode = |r: &mut Reader<'_>| -> Result<Self, ProtocolError> {
            let sender = InstanceIdWithStamp::read(r)?;
            let action = match opcode {
                Opcode::DO_MODE => ObjectAction::DoMode(word(r)?),
                Opcode::SET_MODE => ObjectAction::SetMode(word(r)?),
                Opcode::STOP_LAYER => ObjectAction::StopLayer(word(r)?),
                Opcode::STOP_BEHAVIOR => ObjectAction::StopBehavior(word(r)?),
                Opcode::SET_MIDESC => ObjectAction::SetMotionInterp(word(r)?),
                Opcode::SET_GLOBAL_MODIFIER => ObjectAction::SetGlobalModifier(word(r)?),
                Opcode::DO_FX_SCRIPT => ObjectAction::DoFxScript(word(r)?),
                Opcode::STOP_FX => ObjectAction::StopFx(word(r)?),
                Opcode::SET_VELOCITY_SCALE => ObjectAction::SetVelocityScale(float(r)?),
                Opcode::SET_ACCELERATION_SCALE => ObjectAction::SetAccelerationScale(float(r)?),
                Opcode::SET_JUMP_SCALE => ObjectAction::SetJumpScale(float(r)?),
                Opcode::SET_HOVER_HEIGHT => ObjectAction::SetHoverHeight(float(r)?),
                Opcode::DO_FX => ObjectAction::DoFx {
                    fx: word(r)?,
                    scalar: float(r)?,
                },
                Opcode::DO_TARGETTED_FX => ObjectAction::DoTargettedFx {
                    fx: word(r)?,
                    scalar: float(r)?,
                    target: id(r)?,
                },
                Opcode::DO_SLIDER => ObjectAction::DoSlider {
                    slider: word(r)?,
                    value: float(r)?,
                    time: float(r)?,
                },
                Opcode::LOOK_AT => ObjectAction::LookAt(id(r)?),
                Opcode::LOOK => ObjectAction::LookAtDir {
                    x: float(r)?,
                    y: float(r)?,
                },
                Opcode::LEAVE_WORLD => {
                    let stamp = r.u16()?;
                    r.align()?;
                    ObjectAction::LeaveWorld(stamp)
                }
                Opcode::PARENT => ObjectAction::Parent {
                    parent: InstanceIdWithStamp::read(r)?,
                    location: word(r)?,
                    orientation: word(r)?,
                },
                Opcode::DE_PARENT => ObjectAction::DeParent(InstanceIdWithStamp::read(r)?),
                Opcode::ADD_IMPULSE => ObjectAction::AddImpulse {
                    direction: [float(r)?, float(r)?, float(r)?],
                    contact: word(r)?,
                },
                Opcode::ADD_TARGETED_IMPULSE => ObjectAction::AddTargetedImpulse {
                    target: id(r)?,
                    magnitude: float(r)?,
                    falloff: word(r)?,
                    vertical: float(r)?,
                },
                Opcode::ADD_TERRAIN_MODIFIER => ObjectAction::AddTerrainModifier {
                    terrain: word(r)?,
                    modifier: word(r)?,
                },
                Opcode::DO_BEHAVIOR => ObjectAction::DoBehavior(Box::new(BehaviorParams::read(r)?)),
                Opcode::RELEASE_BEHAVIOR => ObjectAction::ReleaseBehavior {
                    behavior: word(r)?,
                    params: Box::new(BehaviorParams::read(r)?),
                },
                Opcode::MOVE_TO => {
                    let params = Box::new(MovementParameters::read(r)?);
                    let stamp = r.u16()?;
                    r.align()?;
                    ObjectAction::MoveTo { params, stamp }
                }
                Opcode::DO_STORY => ObjectAction::DoStory(Box::new(PhysicsStory::read(r)?)),
                Opcode::INITIALIZE_MISSILE => ObjectAction::InitializeMissile {
                    position: Position::read(r)?,
                    velocity: r.vec3()?,
                    target: Box::new(TargetPosition::read(r)?),
                },
                Opcode::UPDATE_VISUAL_DESC => {
                    ObjectAction::UpdateVisualDesc(Box::new(VisualDesc::read(r)?))
                }
                Opcode::STOP_ALL_WEENIE_BEHAVIORS => ObjectAction::StopAllWeenieBehaviors,
                Opcode::STOP_MOVE_TO_CANCELS_BEHAVIORS => ObjectAction::StopMoveToCancelsBehaviors,
                // `Opcode::CLEAR_TERRAIN_MODIFIERS`, the last handled opcode.
                _ => ObjectAction::ClearTerrainModifiers,
            };
            Ok(Self { sender, action })
        };
        Some(decode(r))
    }
}

#[cfg(test)]
mod tests {
    use super::{ObjectAction, ObjectEvent};
    use crate::protocol::opcode::Opcode;
    use crate::protocol::reader::Reader;

    fn sender() -> Vec<u8> {
        let mut b = 0x55u64.to_le_bytes().to_vec();
        b.extend([2, 0, 3, 0]);
        b
    }

    #[test]
    fn single_argument_events_decode_after_the_sender() {
        let mut b = sender();
        b.extend(0x4100_0003u32.to_le_bytes());
        let mut r = Reader::new(&b);
        let event = ObjectEvent::read(Opcode::SET_MODE, &mut r)
            .unwrap()
            .unwrap();
        r.finish().unwrap();
        assert_eq!((event.sender.id, event.sender.instance), (0x55, 2));
        assert_eq!(event.action, ObjectAction::SetMode(0x4100_0003));
        let mut b = sender();
        b.extend([7, 0, 0, 0]);
        let mut r = Reader::new(&b);
        let event = ObjectEvent::read(Opcode::LEAVE_WORLD, &mut r)
            .unwrap()
            .unwrap();
        r.finish().unwrap();
        assert_eq!(event.action, ObjectAction::LeaveWorld(7));
        assert!(ObjectEvent::read(0x1234, &mut Reader::new(&sender())).is_none());
    }

    /// `ECM_Physics::RecvEvt_SetVelocityScale` (0x10092), `SetAccelerationScale` (0x10093),
    /// `SetHoverHeight` (0x10091) and `SetJumpScale` (0x100BA): the sender, then one float.
    #[test]
    fn the_scale_and_hover_events_carry_one_float_after_the_sender() {
        for (opcode, expected) in [
            (Opcode::SET_HOVER_HEIGHT, ObjectAction::SetHoverHeight(1.5)),
            (
                Opcode::SET_VELOCITY_SCALE,
                ObjectAction::SetVelocityScale(1.5),
            ),
            (
                Opcode::SET_ACCELERATION_SCALE,
                ObjectAction::SetAccelerationScale(1.5),
            ),
            (Opcode::SET_JUMP_SCALE, ObjectAction::SetJumpScale(1.5)),
        ] {
            let mut b = sender();
            b.extend(1.5f32.to_le_bytes());
            let mut r = Reader::new(&b);
            let event = ObjectEvent::read(opcode, &mut r).unwrap().unwrap();
            r.finish().unwrap();
            assert_eq!((event.sender.id, event.sender.instance), (0x55, 2));
            assert_eq!(event.action, expected, "opcode {opcode:#x}");
        }
    }

    #[test]
    fn move_to_ends_with_its_aligned_stamp_and_impulses_align_each_argument() {
        let mut b = sender();
        b.extend(0x22u32.to_le_bytes()); // object target, no distances
        b.extend(9u64.to_le_bytes());
        b.extend([0u32, 5].iter().flat_map(|v| v.to_le_bytes())); // no waypoints, context
        b.extend([4, 0, 0, 0]);
        let mut r = Reader::new(&b);
        let event = ObjectEvent::read(Opcode::MOVE_TO, &mut r).unwrap().unwrap();
        r.finish().unwrap();
        let ObjectAction::MoveTo { params, stamp } = event.action else {
            panic!("move to");
        };
        assert_eq!((params.context_id, stamp), (5, 4));
        let mut b = sender();
        b.extend(7u64.to_le_bytes());
        b.extend(2.0f32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(0.5f32.to_le_bytes());
        let mut r = Reader::new(&b);
        let event = ObjectEvent::read(Opcode::ADD_TARGETED_IMPULSE, &mut r)
            .unwrap()
            .unwrap();
        r.finish().unwrap();
        assert_eq!(
            event.action,
            ObjectAction::AddTargetedImpulse {
                target: 7,
                magnitude: 2.0,
                falloff: 1,
                vertical: 0.5
            }
        );
    }
}
