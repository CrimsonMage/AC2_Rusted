//! The double's objects on the radar: two creatures (`MonsterTemplate`, package 0x1169) and an
//! NPC (`NPCTemplate`, package 0x13A4) a few metres from where the player enters. Their scripts
//! answer `gmCWInterface::InqRadarLook` for the radar (`gmRadarUI::AddObject`). The bases
//! (`CMonster` 0x1BDC, `NPC` 0xBB) answer 0, no blip: the look is the templates' override.

use ac2_net::protocol::{InstanceIdWithStamp, QualityUpdate, QualityValue};

use crate::objects::{weenie_object_bytes, ObjectSpawn};
use crate::{Spawn, State};

/// `MonsterTemplate` (derived from `CMonster`), the creature's script package.
pub const MONSTER_PACKAGE: u32 = 0x1169;
/// `NPCTemplate` (derived from `NPC`), the non-player character's script package.
pub const NPC_PACKAGE: u32 = 0x13A4;
/// The creatures' and the NPC's instance ids (type byte 0x80: dynamic objects).
pub const CREATURE_ONE: u64 = 0x8000_0000_0000_0A01;
pub const CREATURE_TWO: u64 = 0x8000_0000_0000_0A02;
pub const NPC: u64 = 0x8000_0000_0000_0A03;
/// `IsSelectable` (bool 624): the templates' `InqRadarLook` gives a blip only to a selectable
/// object.
const IS_SELECTABLE: u32 = 0x270;
/// `Level` (int 302), which the templates' looks read.
const LEVEL: u32 = 0x12E;
/// The level the double gives its radar objects.
pub const RADAR_OBJECT_LEVEL: i32 = 5;

/// `mn_drudge.vd`: a plain drudge's `VisualDesc`, for tests that draw the radar creatures.
pub const DRUDGE_VISUAL: u32 = 0x1F00_0036;

/// The radar objects and their script packages, placed from the player's spawn.
pub fn radar_objects(spawn: &Spawn) -> Vec<(ObjectSpawn, u32)> {
    let at = |dx: f32, dy: f32| [spawn.origin[0] + dx, spawn.origin[1] + dy, spawn.origin[2]];
    let object = |id, origin, name| ObjectSpawn {
        id,
        instance: 1,
        visual: 0,
        cell: spawn.cell,
        origin,
        rotation: [1.0, 0.0, 0.0, 0.0],
        stamps: [1, 0, 0, 0],
        name,
    };
    vec![
        (
            object(CREATURE_ONE, at(8.0, 0.0), "Creature"),
            MONSTER_PACKAGE,
        ),
        (
            object(CREATURE_TWO, at(0.0, 12.0), "Creature"),
            MONSTER_PACKAGE,
        ),
        (object(NPC, at(-6.0, -6.0), "Scout"), NPC_PACKAGE),
    ]
}

impl State {
    /// `CreateObject` for each radar object (`CObjectMaint::RecvNotice_CreateObject`), then
    /// its selectability and level.
    pub fn spawn_radar_objects(&mut self) {
        let spawn = self.spawn;
        for (mut object, package) in radar_objects(&spawn) {
            if package == MONSTER_PACKAGE {
                object.visual = self.creature_visual;
            }
            self.send_event(&weenie_object_bytes(&object, package));
            let stamped = Some(InstanceIdWithStamp {
                id: object.id,
                instance: object.instance,
                other: 0,
            });
            for (quality, value) in [
                (IS_SELECTABLE, QualityValue::Bool(1)),
                (LEVEL, QualityValue::Int(RADAR_OBJECT_LEVEL)),
            ] {
                self.send_quality(&QualityUpdate {
                    object: stamped,
                    quality,
                    value,
                });
            }
        }
    }
}
