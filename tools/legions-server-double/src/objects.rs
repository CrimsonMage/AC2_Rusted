//! The object messages the double builds: `CreateObject` (`0x10058`) for a world object, a
//! server position (`0x10086`) and the per-object events.

use crate::words;

/// An object the server creates (`CreateObject`, `0x10058`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectSpawn {
    pub id: u64,
    pub instance: u16,
    /// The `VisualDesc` parent DID.
    pub visual: u32,
    pub cell: u32,
    pub origin: [f32; 3],
    /// (w, x, y, z).
    pub rotation: [f32; 4],
    /// `PhysicsDesc` timestamps: position, teleport, force position, move-to.
    pub stamps: [u16; 4],
    /// The weenie's name (`gmWeenieDesc` bit 4, a literal `StringInfo`), as retail servers send
    /// one for every weenie; empty for none.
    pub name: &'static str,
}

/// `CreateObject`: the id; `VisualDesc` with its parent (bit 2); `PhysicsDesc` with a position
/// (bit `0x8000`), the instance stamp, visual stamp 0, the cell and frame, the four timestamps;
/// a `WeenieDesc` with the name, if any.
pub fn create_object_bytes(object: &ObjectSpawn) -> Vec<u8> {
    weenie_object_bytes(object, 0)
}

/// [`create_object_bytes`] with the weenie's script package (`gmWeenieDesc` bit 1, the
/// `CWDistributor::Construct` package) when `package` is not 0.
pub fn weenie_object_bytes(object: &ObjectSpawn, package: u32) -> Vec<u8> {
    let mut visual = Vec::new();
    words(&mut visual, &[2, object.visual]);
    weenie_object_with(object, &visual, package, object.name)
}

/// [`weenie_object_bytes`] with the `VisualDesc` bytes given and the weenie's name (the spawn's
/// `visual` and `name` fields are not used).
pub fn weenie_object_with(
    object: &ObjectSpawn,
    visual: &[u8],
    package: u32,
    name: &str,
) -> Vec<u8> {
    let mut b = 0x0001_0058u32.to_le_bytes().to_vec();
    b.extend(object.id.to_le_bytes());
    b.extend(visual);
    words(&mut b, &[0x8000]);
    b.extend(object.instance.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    words(&mut b, &[object.cell]);
    for v in object.origin.iter().chain(object.rotation.iter()) {
        b.extend(v.to_le_bytes());
    }
    for stamp in object.stamps {
        b.extend(stamp.to_le_bytes());
    }
    let mask = u32::from(package != 0) | if name.is_empty() { 0 } else { 4 };
    words(&mut b, &[mask]);
    if package != 0 {
        words(&mut b, &[package]);
    }
    if !name.is_empty() {
        b.extend(ac2_net::protocol::StringInfo::literal(name).to_bytes());
    }
    b
}

/// A server position (`ECM_Physics::RecvEvt_Position`, `0x10086`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectPosition {
    pub id: u64,
    pub instance: u16,
    pub time: f64,
    pub cell: u32,
    pub origin: [f32; 3],
    /// `Heading::ToUInt8 << 24 | Vector::UnitVectorToUInt24` of the do-motion values.
    pub heading_and_motion: u32,
    /// `PositionPack` flags: 2 jump, 4 contact, 8 impulse.
    pub flags: u32,
    /// Position, teleport and force-position stamps.
    pub stamps: [u16; 3],
    /// The jump or impulse velocity (flag 2 or 8).
    pub velocity: Option<[f32; 3]>,
}

/// `PositionPack::StreamPack` after the sender: time, cell and origin, the heading/motion
/// word, the flags, the position, force-position and teleport stamps, the velocity, aligned.
pub fn position_bytes(p: &ObjectPosition) -> Vec<u8> {
    let mut b = 0x0001_0086u32.to_le_bytes().to_vec();
    b.extend(p.id.to_le_bytes());
    b.extend(p.instance.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    b.extend(p.time.to_le_bytes());
    words(&mut b, &[p.cell]);
    for v in p.origin {
        b.extend(v.to_le_bytes());
    }
    words(&mut b, &[p.heading_and_motion, p.flags]);
    for stamp in [p.stamps[0], p.stamps[2], p.stamps[1]] {
        b.extend(stamp.to_le_bytes());
    }
    if let Some(velocity) = p.velocity {
        for v in velocity {
            b.extend(v.to_le_bytes());
        }
    }
    while !b.len().is_multiple_of(4) {
        b.push(0);
    }
    b
}

/// A per-object event: opcode, the sender (id, instance, visual stamp 0), its words.
pub fn object_event_bytes(opcode: u32, id: u64, instance: u16, args: &[u32]) -> Vec<u8> {
    let mut b = opcode.to_le_bytes().to_vec();
    b.extend(id.to_le_bytes());
    b.extend(instance.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    words(&mut b, args);
    b
}
