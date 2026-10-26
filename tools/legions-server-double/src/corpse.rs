//! The double's corpses: a monster's corpse (`Corpse`, package 0x182) with its loot, in the corpse's
//! container. The player's corpse holds a trophy skin (`TrophyMimbuSkin`, the trophy's package
//! 0x1019) and an item; a corpse of another character (Birch's kill) holds one item.
//!
//! A corpse opens for its owner only. Another character's corpse is refused in the open answer
//! with `NOT_MINE` (1077, `gmWErrorType::NotMine`): the wlib's open handler (`CInventorySystem`,
//! reached from `CPlayer` 0x8C) takes its error branch and lists nothing. Its take-all moves
//! nothing either: the wlib's Loot All counts the corpse's lists (`Container` 0x2B, slots 15 and
//! 19) and sends no take-all when they are empty.
//!
//! The corpse's usage permission (`CorpseUsagePermission`, package 0x26A, a singleton) is read by
//! the corpse's function 0x1C; its fields are not traced, so the double keeps only each corpse's
//! owner. The permission blob (`CorpsePermissionBlob`, 0x1BDF) is not traced to the message that
//! carries it, so none is sent.

use ac2_net::protocol::{InstanceIdWithStamp, QualityUpdate, QualityValue};

use crate::items::{item_object_bytes, ItemSpawn};
use crate::objects::{weenie_object_bytes, ObjectSpawn};
use crate::{State, FELLOW};

/// The player's corpse (a monster the player killed): its instance id (type byte 0x80).
pub const CORPSE: u64 = 0x8000_0000_0000_0A05;
/// The player's corpse's loot: the trophy skin, then an item.
pub const CORPSE_SKIN: u64 = 0x8000_0000_0000_0107;
pub const CORPSE_ITEM: u64 = 0x8000_0000_0000_0108;
/// Another character's corpse (Birch's kill) and its item.
pub const FOREIGN_CORPSE: u64 = 0x8000_0000_0000_0A06;
pub const FOREIGN_ITEM: u64 = 0x8000_0000_0000_0109;
/// The open answer's error for another character's corpse: `gmWErrorType::NotMine` (1077), the
/// value the wlib's open handler's error branch switches on.
pub const NOT_MINE: u32 = 1077;
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
/// `CItem`, the package of an item that is not a trophy.
const ITEM_PACKAGE: u32 = 0x1AA9;
/// The corpse's slots (int 280).
pub const CORPSE_CAPACITY: u32 = 12;

impl State {
    /// The player's corpse, three metres west and four south of where the player enters, with its
    /// skin and item.
    pub fn spawn_corpse(&mut self) {
        let spawn = self.spawn;
        let origin = [
            spawn.origin[0] - 3.0,
            spawn.origin[1] - 4.0,
            spawn.origin[2],
        ];
        self.spawn_corpse_at(spawn.cell, origin);
    }

    /// Places the player's corpse at `origin` in `cell` (a creature's corpse is where the creature
    /// died), with its skin and item. The corpse's id is one id: a second corpse replaces nothing
    /// and is not placed (see the combat's `kill`).
    pub fn spawn_corpse_at(&mut self, cell: u32, origin: [f32; 3]) {
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
        let owner = self.player();
        self.spawn_corpse_of(CORPSE, owner, cell, origin, &[skin, item]);
    }

    /// Birch's corpse, on the other side of the player, with one item; the player may not loot it.
    pub fn spawn_foreign_corpse(&mut self) {
        let item = ItemSpawn {
            icon: 0x0600_1004,
            value: 10,
            ..ItemSpawn::new(FOREIGN_ITEM, ITEM_PACKAGE, "Bone", FOREIGN_CORPSE)
        };
        let spawn = self.spawn;
        let origin = [
            spawn.origin[0] + 4.0,
            spawn.origin[1] - 4.0,
            spawn.origin[2],
        ];
        self.spawn_corpse_of(FOREIGN_CORPSE, FELLOW, spawn.cell, origin, &[item]);
    }

    /// A corpse `id` owned by `owner` at `origin` in `cell`, with `loot`: its `CreateObject`, its
    /// weenie type, its items and the contents its open answer lists.
    fn spawn_corpse_of(
        &mut self,
        id: u64,
        owner: u64,
        cell: u32,
        origin: [f32; 3],
        loot: &[ItemSpawn],
    ) {
        let corpse = ObjectSpawn {
            id,
            instance: 1,
            visual: 0,
            cell,
            origin,
            rotation: [1.0, 0.0, 0.0, 0.0],
            stamps: [1, 0, 0, 0],
            name: "Corpse",
        };
        self.send_event(&weenie_object_bytes(&corpse, CORPSE_PACKAGE));
        self.send_quality(&QualityUpdate {
            object: Some(InstanceIdWithStamp {
                id,
                instance: 1,
                other: 0,
            }),
            quality: WEENIE_TYPE,
            value: QualityValue::Int(CORPSE_WEENIE_TYPE),
        });
        for item in loot {
            self.send_event(&item_object_bytes(item));
        }
        let world = &mut self.script_world;
        world.capacities.insert(id, CORPSE_CAPACITY);
        world.corpse_owners.insert(id, owner);
        world
            .contents
            .insert(id, (Vec::new(), loot.iter().map(|item| item.id).collect()));
    }

    /// Whether the player may loot container `container`: any container that is not a corpse, and
    /// the player's own corpses.
    pub(crate) fn may_loot(&self, container: u64) -> bool {
        self.script_world
            .corpse_owners
            .get(&container)
            .is_none_or(|owner| *owner == self.player())
    }
}
