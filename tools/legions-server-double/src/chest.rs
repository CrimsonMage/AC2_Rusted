//! The double's world chest: a `CContainer` (the side pack's package) on the landblock five metres
//! from where the player enters. It is not in the player's containers; its two items are in it,
//! and it opens and gives items out as the side pack does (`WM_Inventory` 0x5D / 0x53 and the
//! move answers), with the same script package path. The client's external container panel
//! (`gmExternalContainerUI::RecvNotice_CreateObject`, admin 0x00420520) registers its notice
//! handlers on a world object when it is created, so the chest needs no inventory membership.

use ac2_net::protocol::{InstanceIdWithStamp, QualityUpdate, QualityValue};

use crate::items::{item_object_bytes, ItemSpawn, CONTAINER_PACKAGE, ICON_POTION, ICON_QUEST};
use crate::objects::{weenie_object_bytes, ObjectSpawn};
use crate::State;

/// The chest's instance id (type byte 0x80: a dynamic object, as the radar's objects are).
pub const CHEST: u64 = 0x8000_0000_0000_0A04;
/// The chest's two items, in slot order.
pub const CHEST_GEM: u64 = 0x8000_0000_0000_0105;
pub const CHEST_SCROLL: u64 = 0x8000_0000_0000_0106;
/// The chest's slots (int 280).
pub const CHEST_CAPACITY: i32 = 12;

/// `CItem`, the package of an item that is not a weapon or container.
const ITEM_PACKAGE: u32 = 0x1AA9;

impl State {
    /// Places the chest and its two items: each object's `CreateObject` (the items carry no
    /// position, being in the chest), the chest's capacity, and the contents the chest's open
    /// answer lists. Nothing is sent for the contents themselves; opening the chest lists them.
    pub fn spawn_chest(&mut self) {
        let spawn = self.spawn;
        let chest = ObjectSpawn {
            id: CHEST,
            instance: 1,
            visual: 0,
            cell: spawn.cell,
            origin: [
                spawn.origin[0] + 3.0,
                spawn.origin[1] - 4.0,
                spawn.origin[2],
            ],
            rotation: [1.0, 0.0, 0.0, 0.0],
            stamps: [1, 0, 0, 0],
            name: "Chest",
        };
        self.send_event(&weenie_object_bytes(&chest, CONTAINER_PACKAGE));
        let items = [
            ItemSpawn {
                icon: ICON_POTION,
                value: 40,
                ..ItemSpawn::new(CHEST_GEM, ITEM_PACKAGE, "Gem", CHEST)
            },
            ItemSpawn {
                icon: ICON_QUEST,
                value: 15,
                ..ItemSpawn::new(CHEST_SCROLL, ITEM_PACKAGE, "Scroll", CHEST)
            },
        ];
        for item in &items {
            self.send_event(&item_object_bytes(item));
        }
        self.send_quality(&QualityUpdate {
            object: Some(InstanceIdWithStamp {
                id: CHEST,
                instance: 1,
                other: 0,
            }),
            quality: 280,
            value: QualityValue::Int(CHEST_CAPACITY),
        });
        let world = &mut self.script_world;
        world.capacities.insert(CHEST, CHEST_CAPACITY as u32);
        world
            .contents
            .insert(CHEST, (Vec::new(), vec![CHEST_GEM, CHEST_SCROLL]));
    }
}
