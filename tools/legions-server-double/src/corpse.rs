//! The double's corpse: a monster's corpse (`Corpse`, package 0x182) with its loot, a trophy skin
//! (`TrophyMimbuSkin`, the trophy's package 0x1019) and an item, in the corpse's container. The
//! corpse is a container as the chest is: opening it lists its contents (`WM_Inventory` 0x5D), and
//! the loot answers (`LootAll`, the per-item moves) take items out of it as the server does.
//!
//! What the double does not model: the corpse's permission. Retail's `CorpsePermissionBlob`
//! (package 0x1BDF) and `CorpseUsagePermission` (0x26A) are not traced to the message that
//! carries them, so the double sends no permission and opens the corpse for the player (its
//! killer, the only character it has).

use ac2_net::protocol::{InstanceIdWithStamp, QualityUpdate, QualityValue};

use crate::items::{item_object_bytes, ItemSpawn};
use crate::objects::{weenie_object_bytes, ObjectSpawn};
use crate::State;

/// The corpse's instance id (type byte 0x80: a dynamic object).
pub const CORPSE: u64 = 0x8000_0000_0000_0A05;
/// The corpse's loot: the trophy skin, then an item.
pub const CORPSE_SKIN: u64 = 0x8000_0000_0000_0107;
pub const CORPSE_ITEM: u64 = 0x8000_0000_0000_0108;
/// `Corpse`, the corpse's script package.
pub const CORPSE_PACKAGE: u32 = 0x182;
/// `TrophyMimbuSkin`, the trophy skin's script package.
pub const TROPHY_MIMBU_SKIN_PACKAGE: u32 = 0x1019;
/// The corpse's weenie type (int 20), `Corpse`: enum mapper `gmweenietype.emp` (DID 0x230000C5,
/// not in the test dataset; its base mapper 0x230000C6 holds `Container` 0x3, `GameplayContainer`
/// 0x103 and `Player` 0x1F, which the same `(type & 3) == 3` container test passes). The client's
/// `EngineWeenieTypeSystem` 0x5 is what the loot checks read.
pub const CORPSE_WEENIE_TYPE: i32 = 0x0002_0103;
/// The weenie type quality (int 20, `WeenieType`).
const WEENIE_TYPE: u32 = 20;
/// `CItem`, the package of the item.
const ITEM_PACKAGE: u32 = 0x1AA9;
/// The corpse's slots (int 280).
pub const CORPSE_CAPACITY: u32 = 12;

impl State {
    /// Places the corpse three metres west and four south of where the player enters, with its
    /// skin and item in it (each object's `CreateObject`, the corpse's capacity, and the contents
    /// its open answer lists).
    pub fn spawn_corpse(&mut self) {
        let spawn = self.spawn;
        let origin = [
            spawn.origin[0] - 3.0,
            spawn.origin[1] - 4.0,
            spawn.origin[2],
        ];
        self.spawn_corpse_at(spawn.cell, origin);
    }

    /// Places the corpse at `origin` in `cell` (a creature's corpse is where the creature died),
    /// with its skin and item in it. The corpse's id is one id: a second corpse replaces nothing
    /// and is not placed (see the combat's `kill`).
    pub fn spawn_corpse_at(&mut self, cell: u32, origin: [f32; 3]) {
        let corpse = ObjectSpawn {
            id: CORPSE,
            instance: 1,
            visual: 0,
            cell,
            origin,
            rotation: [1.0, 0.0, 0.0, 0.0],
            stamps: [1, 0, 0, 0],
            name: "Corpse",
        };
        self.send_event(&weenie_object_bytes(&corpse, CORPSE_PACKAGE));
        let skin = ItemSpawn {
            icon: 0x0600_1019,
            value: 25,
            ..ItemSpawn::new(CORPSE_SKIN, TROPHY_MIMBU_SKIN_PACKAGE, "Mimbu Skin", CORPSE)
        };
        let item = ItemSpawn {
            icon: 0x0600_1004,
            value: 10,
            ..ItemSpawn::new(CORPSE_ITEM, ITEM_PACKAGE, "Bone", CORPSE)
        };
        self.send_quality(&QualityUpdate {
            object: Some(InstanceIdWithStamp {
                id: CORPSE,
                instance: 1,
                other: 0,
            }),
            quality: WEENIE_TYPE,
            value: QualityValue::Int(CORPSE_WEENIE_TYPE),
        });
        self.send_event(&item_object_bytes(&skin));
        self.send_event(&item_object_bytes(&item));
        let world = &mut self.script_world;
        world.capacities.insert(CORPSE, CORPSE_CAPACITY);
        world
            .contents
            .insert(CORPSE, (Vec::new(), vec![CORPSE_SKIN, CORPSE_ITEM]));
    }
}
