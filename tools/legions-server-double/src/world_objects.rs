//! The double's world objects a player uses: a door (`DoorTemplate`'s package), a lever
//! (`LeverTemplate`'s package) and a ground item (`CItem`), placed a few metres from where the
//! player enters. A use of the door or the lever is answered as a server answers any use: the
//! `WM_Usage` desc comes back with the outcome the test configured (success by default). The
//! door's and the lever's own state and animation are not answered here (see the catalog: their
//! behaviour ids are not traced). A use of the ground item takes it: the server moves it off the
//! landblock (`CPhysicsObj::RecvEvt_Contain` with a newer position stamp) into the player's main
//! pack, lists the pack (`WM_Inventory` 0x53) and answers the use.

use ac2_net::protocol::{Opcode, ScriptMessage, UsageDesc};

use crate::objects::{weenie_object_bytes, ObjectSpawn};
use crate::State;

/// The door, the lever and the ground item (type byte 0x80: dynamic objects, as the chest's are).
pub const DOOR: u64 = 0x8000_0000_0000_0A05;
pub const LEVER: u64 = 0x8000_0000_0000_0A06;
pub const GROUND_ITEM: u64 = 0x8000_0000_0000_0108;

/// The door's, the lever's and the item's script packages (`DoorTemplate`, `LeverTemplate`,
/// `CItem`).
pub const DOOR_PACKAGE: u32 = 0x0EB3;
pub const LEVER_PACKAGE: u32 = 0x37AE;
pub const GROUND_ITEM_PACKAGE: u32 = 0x1AA9;

/// The position stamp the ground item was created with (`CreateObject`'s first stamp).
const CREATED_STAMP: u16 = 1;

impl State {
    /// Places the door, the lever and the ground item (`CreateObject` with a position each).
    pub fn spawn_world_objects(&mut self) {
        let spawn = self.spawn;
        let at = |dx: f32, dy: f32| [spawn.origin[0] + dx, spawn.origin[1] + dy, spawn.origin[2]];
        let objects = [
            (DOOR, DOOR_PACKAGE, "Door", at(-2.0, 6.0)),
            (LEVER, LEVER_PACKAGE, "Lever", at(5.0, 2.0)),
            (GROUND_ITEM, GROUND_ITEM_PACKAGE, "Sigil", at(-3.0, 2.0)),
        ];
        for (id, package, name, origin) in objects {
            let object = ObjectSpawn {
                id,
                instance: 1,
                visual: 0,
                cell: spawn.cell,
                origin,
                rotation: [1.0, 0.0, 0.0, 0.0],
                stamps: [CREATED_STAMP, 0, 0, 0],
                name,
            };
            self.send_event(&weenie_object_bytes(&object, package));
        }
    }

    /// `WM_Usage` on the ground item: the item leaves the world for the player's main pack, the
    /// pack lists it, and the use is answered with success.
    pub(crate) fn answer_pickup(&mut self, desc: &UsageDesc) {
        let item = desc.item;
        let player = self.player();
        // `CPhysicsObj::RecvEvt_Contain`: the child's id, its instance and a position stamp one
        // newer than the one it was created with.
        let mut contain = Opcode::CONTAIN.to_le_bytes().to_vec();
        contain.extend(item.to_le_bytes());
        contain.extend(1u16.to_le_bytes());
        contain.extend((CREATED_STAMP + 1).to_le_bytes());
        self.send_event(&contain);
        let world = &mut self.script_world;
        for (_, items) in world.contents.values_mut() {
            items.retain(|i| *i != item);
        }
        world.contents.entry(player).or_default().1.push(item);
        self.send_contents(player);
        let mut done = desc.clone();
        done.error = 0;
        self.send_script(&ScriptMessage::UsageDone(done));
    }
}
