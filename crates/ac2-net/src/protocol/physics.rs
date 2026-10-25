//! `PhysicsDesc::StreamPack` (identical in admin 7148 and Legions 7160: Legions 0x005F4D30) and
//! the `BehaviorParams` it carries.

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::types::{float_table, Position};
use crate::protocol::ProtocolError;

/// `BehaviorParams::StreamPack`: a pack bitfield, then the fields it names, in stream order.
/// Boolean fields travel as 32-bit integers; the fields not packed keep their
/// `BehaviorParams::SetDefaults` values ([`Default`]).
#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorParams {
    pub bitfield: u32,
    pub behavior: u32,
    pub time_scale: f32,
    pub mode: u32,
    pub hold_cycles: u32,
    pub fx_script: u32,
    pub fx: u32,
    pub target: u64,
    pub camera_target: u64,
    pub camera_behavior: u32,
    pub vdesc_to_clone: u32,
    pub cloned_apr: u32,
    pub cloned_app_apr: Vec<(u32, f32)>,
    /// The `g_bucketSizes` index in `m_clonedAppAprHash`'s `PHashTable` header.
    pub cloned_app_apr_buckets: u8,
    pub impulse: [f32; 3],
    pub early_callback: u32,
    pub moveto_cancels: u32,
    pub camera_parent: u32,
    pub camera_target_flag: u32,
    pub camera_hold: u32,
    pub camera_restore: u32,
    pub fade_children: u32,
    pub propagate: u32,
    pub lock_actions: u32,
    pub send_event: u32,
    pub leave_idle_alone: u32,
    pub destroy_on_completion: u32,
    pub weenie_emote: u32,
    pub context: u32,
}

impl Default for BehaviorParams {
    /// `BehaviorParams::SetDefaults`: time scale 1, one hold cycle, send event set, the rest
    /// zero (`INVALID_IID` targets).
    fn default() -> Self {
        Self {
            bitfield: 0,
            behavior: 0,
            time_scale: 1.0,
            mode: 0,
            hold_cycles: 1,
            fx_script: 0,
            fx: 0,
            target: 0,
            camera_target: 0,
            camera_behavior: 0,
            vdesc_to_clone: 0,
            cloned_apr: 0,
            cloned_app_apr: Vec::new(),
            cloned_app_apr_buckets: 0,
            impulse: [0.0; 3],
            early_callback: 0,
            moveto_cancels: 0,
            camera_parent: 0,
            camera_target_flag: 0,
            camera_hold: 0,
            camera_restore: 0,
            fade_children: 0,
            propagate: 0,
            lock_actions: 0,
            send_event: 1,
            leave_idle_alone: 0,
            destroy_on_completion: 0,
            weenie_emote: 0,
            context: 0,
        }
    }
}

impl BehaviorParams {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bits = reader.u32()?;
        let mut p = Self {
            bitfield: bits,
            ..Self::default()
        };
        let has = |bit: u32| bits & bit != 0;
        if has(0x1) {
            p.behavior = reader.u32()?;
        }
        if has(0x4) {
            p.time_scale = reader.f32()?;
        }
        if has(0x2) {
            p.mode = reader.u32()?;
        }
        if has(0x10) {
            p.hold_cycles = reader.u32()?;
        }
        if has(0x1000) {
            p.fx_script = reader.u32()?;
        }
        if has(0x10_0000) {
            p.fx = reader.u32()?;
        }
        if has(0x2000) {
            p.target = reader.u64()?;
        }
        if has(0x20_0000) {
            p.camera_target = reader.u64()?;
            p.camera_behavior = reader.u32()?;
        }
        if has(0x1_0000) {
            p.vdesc_to_clone = reader.u32()?;
            p.cloned_apr = reader.u32()?;
            let (buckets, table) = float_table(reader)?;
            p.cloned_app_apr = table;
            p.cloned_app_apr_buckets = buckets;
        }
        if has(0x4000) {
            p.impulse = reader.vec3()?;
        }
        let flags: [(u32, &mut u32); 13] = [
            (0x8, &mut p.early_callback),
            (0x20, &mut p.moveto_cancels),
            (0x80, &mut p.camera_parent),
            (0x40, &mut p.camera_target_flag),
            (0x4_0000, &mut p.camera_hold),
            (0x8_0000, &mut p.camera_restore),
            (0x100, &mut p.fade_children),
            (0x400, &mut p.propagate),
            (0x800, &mut p.lock_actions),
            (0x200, &mut p.send_event),
            (0x2_0000, &mut p.leave_idle_alone),
            (0x8000, &mut p.destroy_on_completion),
            (0x80_0000, &mut p.weenie_emote),
        ];
        for (bit, field) in flags {
            if bits & bit != 0 {
                *field = reader.u32()?;
            }
        }
        if has(0x40_0000) {
            p.context = reader.u32()?;
        }
        Ok(p)
    }

    /// `BehaviorParams::StreamPack` (pack): the fields the bitfield names, in stream order.
    pub(crate) fn write(&self, w: &mut Writer) {
        let bits = self.bitfield;
        let has = |bit: u32| bits & bit != 0;
        w.u32(bits);
        for (bit, value) in [
            (0x1, self.behavior),
            (0x4, self.time_scale.to_bits()),
            (0x2, self.mode),
            (0x10, self.hold_cycles),
            (0x1000, self.fx_script),
            (0x10_0000, self.fx),
        ] {
            if has(bit) {
                w.u32(value);
            }
        }
        if has(0x2000) {
            w.u64(self.target);
        }
        if has(0x20_0000) {
            w.u64(self.camera_target).u32(self.camera_behavior);
        }
        if has(0x1_0000) {
            w.u32(self.vdesc_to_clone).u32(self.cloned_apr).u32(
                self.cloned_app_apr.len() as u32 & 0x00FF_FFFF
                    | u32::from(self.cloned_app_apr_buckets) << 24,
            );
            for (k, v) in &self.cloned_app_apr {
                w.u32(*k).f32(*v);
            }
        }
        if has(0x4000) {
            for v in self.impulse {
                w.f32(v);
            }
        }
        for (bit, value) in [
            (0x8, self.early_callback),
            (0x20, self.moveto_cancels),
            (0x80, self.camera_parent),
            (0x40, self.camera_target_flag),
            (0x4_0000, self.camera_hold),
            (0x8_0000, self.camera_restore),
            (0x100, self.fade_children),
            (0x400, self.propagate),
            (0x800, self.lock_actions),
            (0x200, self.send_event),
            (0x2_0000, self.leave_idle_alone),
            (0x8000, self.destroy_on_completion),
            (0x80_0000, self.weenie_emote),
            (0x40_0000, self.context),
        ] {
            if has(bit) {
                w.u32(value);
            }
        }
    }
}

/// `PhysicsDesc::StreamPack`, in stream order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhysicsDesc {
    pub bitfield: u32,
    pub instance_stamp: u16,
    pub visual_ordering_stamp: u16,
    pub mode: Option<u32>,
    pub behaviors: Vec<BehaviorParams>,
    /// `STREAMPACK_STL(map<unsigned long, SliderData>)`: key, value, velocity.
    pub sliders: Vec<(u32, f32, f32)>,
    pub anim_frame: Option<u32>,
    pub position: Option<Position>,
    /// Parent object, location and the parent's instance stamp.
    pub parent: Option<(u64, u32, u16)>,
    pub orientation_id: Option<u32>,
    pub velocity: Option<[f32; 3]>,
    pub external_acceleration: Option<[f32; 3]>,
    pub velocity_scale: Option<f32>,
    pub jump_scale: Option<f32>,
    pub acceleration: Option<[f32; 3]>,
    pub omega: Option<[f32; 3]>,
    pub look_at: Option<u64>,
    pub head_x: Option<f32>,
    pub head_z: Option<f32>,
    pub target: Option<u64>,
    pub target_position: Option<Position>,
    pub target_offset: Option<[f32; 3]>,
    /// `STREAMPACK_STL(map<unsigned long, FXScalarAndTarget>)`: FX id, scalar, target.
    pub fx: Vec<(u32, f32, u64)>,
    /// `use_target_scale` (bit `0x4000`), `m_missile_is_moving` (`0x80000`) and
    /// `m_missile_is_activated` (`0x100000`) travel as bits only.
    pub timestamps: [u16; 4],
}

const MAX_SLIDERS: usize = 256;

impl PhysicsDesc {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bits = reader.u32()?;
        let mut d = Self {
            bitfield: bits,
            instance_stamp: reader.u16()?,
            visual_ordering_stamp: reader.u16()?,
            ..Self::default()
        };
        let has = |bit: u32| bits & bit != 0;
        if has(0x80) {
            d.mode = Some(reader.u32()?);
        }
        if has(0x1) {
            for _ in 0..reader.count()? {
                d.behaviors.push(BehaviorParams::read(reader)?);
            }
        }
        if has(0x2) {
            let count = reader.count()?;
            if count > MAX_SLIDERS {
                return Err(ProtocolError::Invalid("slider count"));
            }
            for _ in 0..count {
                d.sliders
                    .push((reader.u32()?, reader.f32()?, reader.f32()?));
            }
        }
        if has(0x2_0000) {
            d.anim_frame = Some(reader.u32()?);
        }
        if has(0x8000) {
            d.position = Some(Position::read(reader)?);
        }
        if has(0x20) {
            let parent = reader.u64()?;
            let location = reader.u32()?;
            let stamp = reader.u16()?;
            reader.align()?;
            d.parent = Some((parent, location, stamp));
        }
        if has(0x100) {
            d.orientation_id = Some(reader.u32()?);
        }
        if has(0x4) {
            d.velocity = Some(reader.vec3()?);
        }
        if has(0x40_0000) {
            d.external_acceleration = Some(reader.vec3()?);
        }
        if has(0x80_0000) {
            d.velocity_scale = Some(reader.f32()?);
        }
        if has(0x100_0000) {
            d.jump_scale = Some(reader.f32()?);
        }
        if has(0x8) {
            d.acceleration = Some(reader.vec3()?);
        }
        if has(0x10) {
            d.omega = Some(reader.vec3()?);
        }
        if has(0x200_0000) {
            d.look_at = Some(reader.u64()?);
        }
        if has(0x400_0000) {
            d.head_x = Some(reader.f32()?);
        }
        if has(0x800_0000) {
            d.head_z = Some(reader.f32()?);
        }
        if has(0x400) {
            d.target = Some(reader.u64()?);
        }
        if has(0x800) {
            d.target_position = Some(Position::read(reader)?);
        }
        if has(0x1000) {
            d.target_offset = Some(reader.vec3()?);
        }
        if has(0x200) {
            for _ in 0..reader.count()? {
                d.fx.push((reader.u32()?, reader.f32()?, reader.u64()?));
            }
        }
        for stamp in &mut d.timestamps {
            *stamp = reader.u16()?;
        }
        reader.align()?;
        Ok(d)
    }
}

#[cfg(test)]
mod tests {
    use super::PhysicsDesc;
    use crate::protocol::reader::Reader;

    #[test]
    fn behaviors_and_fx_sit_where_streampack_puts_them() {
        let mut b = Vec::new();
        b.extend((0x0000_0283u32 | 0x8000).to_le_bytes()); // mode, behaviors, sliders, fx, pos
        b.extend([5u16, 6].iter().flat_map(|v| v.to_le_bytes()));
        b.extend(0x4000_0001u32.to_le_bytes());
        b.extend(1u32.to_le_bytes()); // one behavior: id and time scale
        b.extend(0x5u32.to_le_bytes());
        b.extend(0x1000_0007u32.to_le_bytes());
        b.extend(1.5f32.to_le_bytes());
        b.extend(1u32.to_le_bytes()); // one slider
        b.extend(0x4000_000Au32.to_le_bytes());
        b.extend(1.0f32.to_le_bytes());
        b.extend(0.0f32.to_le_bytes());
        b.extend(0x0299_0109u32.to_le_bytes());
        for v in [59.0f32, 240.0, -44.5, 1.0, 0.0, 0.0, 0.0] {
            b.extend(v.to_le_bytes());
        }
        b.extend(1u32.to_le_bytes()); // one fx
        b.extend(0x22u32.to_le_bytes());
        b.extend(0.5f32.to_le_bytes());
        b.extend(9u64.to_le_bytes());
        b.extend([1u16, 2, 3, 4].iter().flat_map(|v| v.to_le_bytes()));
        let mut reader = Reader::new(&b);
        let d = PhysicsDesc::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!((d.instance_stamp, d.mode), (5, Some(0x4000_0001)));
        assert_eq!(d.behaviors[0].behavior, 0x1000_0007);
        assert_eq!(d.behaviors[0].time_scale, 1.5);
        assert_eq!(d.sliders, vec![(0x4000_000A, 1.0, 0.0)]);
        assert_eq!(d.position.unwrap().cell, 0x0299_0109);
        assert_eq!(d.fx, vec![(0x22, 0.5, 9)]);
        assert_eq!(d.timestamps, [1, 2, 3, 4]);
    }
}
