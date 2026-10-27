//! The double's world objects a player uses: a door (`DoorTemplate`'s package), a lever
//! (`LeverTemplate`'s package) and a ground item (`CItem`), placed a few metres from where the
//! player enters. A use of the door or the lever is answered as a server answers any use: the
//! `WM_Usage` desc comes back with success, and the object's motion follows as the server sends
//! it. A use of the door is a latent `DoMode` into its other mode (shut and open): the body's
//! behaviour table plays the transition (the opening or the closing animation), and the door's
//! `Open` bool quality follows. A use of the lever is a `DoBehavior` of its flip (0x41000086).
//! A use of the ground item takes it: the server moves it off the landblock
//! (`CPhysicsObj::RecvEvt_Contain` with a newer position stamp) into the player's main pack, lists
//! the pack (`WM_Inventory` 0x53) and answers the use.

use ac2_net::protocol::{
    InstanceIdWithStamp, Opcode, QualityUpdate, QualityValue, ScriptMessage, UsageDesc,
};

use crate::objects::{object_event_bytes, weenie_object_bytes, ObjectSpawn};
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

/// The door's and the lever's visuals (`bd_doorsingle.vd`, `dn_lever.vd`): the Setups their
/// bodies, animations and behaviour tables load from.
pub const DOOR_VISUAL: u32 = 0x1F00_006C;
pub const LEVER_VISUAL: u32 = 0x1F00_1880;

/// The door's modes in its behaviour table (`bd_doorsingle`): shut (0x41000002) and open
/// (0x41000003). The shut to open transition plays the opening behaviour (0x41000006), the open to
/// shut one the closing behaviour (0x41000008), `gmentitylinktype`'s `Trigger_Close`. The door
/// starts shut (`SetMode`).
pub const DOOR_SHUT: u32 = 0x4100_0002;
pub const DOOR_OPEN: u32 = 0x4100_0003;
/// The `Open` bool quality (`gmboolstats.emp`, 702).
const OPEN_QUALITY: u32 = 702;

/// The lever's flip behaviour in its table (`dn_lever`, 0x41000086): the animation a pull plays
/// (`dn_lever_flip`). The table's off and on modes (0x41000006, 0x41000007) play no animation of
/// their own, so a pull does not change the mode.
pub const LEVER_FLIP: u32 = 0x4100_0086;

/// The position stamp the ground item was created with (`CreateObject`'s first stamp).
const CREATED_STAMP: u16 = 1;

impl State {
    /// Places the door, the lever and the ground item (`CreateObject` with a position each).
    pub fn spawn_world_objects(&mut self) {
        let spawn = self.spawn;
        let at = |dx: f32, dy: f32| [spawn.origin[0] + dx, spawn.origin[1] + dy, spawn.origin[2]];
        let objects = [
            (DOOR, DOOR_PACKAGE, DOOR_VISUAL, "Door", at(-2.0, 6.0)),
            (LEVER, LEVER_PACKAGE, LEVER_VISUAL, "Lever", at(5.0, 2.0)),
            (GROUND_ITEM, GROUND_ITEM_PACKAGE, 0, "Sigil", at(-3.0, 2.0)),
        ];
        for (id, package, visual, name, origin) in objects {
            let object = ObjectSpawn {
                id,
                instance: 1,
                visual,
                cell: spawn.cell,
                origin,
                rotation: [1.0, 0.0, 0.0, 0.0],
                stamps: [CREATED_STAMP, 0, 0, 0],
                name,
            };
            self.send_event(&weenie_object_bytes(&object, package));
        }
        // The door starts shut (`SetMode`: the mode the server creates it with) and its `Open`
        // quality is 0.
        self.send_mode(DOOR, DOOR_SHUT, Opcode::SET_MODE);
        self.send_open_quality(false);
    }

    /// A physics mode event for `object` (`CPhysicsObj::RecvEvt_SetMode` or `RecvEvt_DoMode`).
    fn send_mode(&mut self, object: u64, mode: u32, opcode: u32) {
        self.send_event(&object_event_bytes(opcode, object, 1, &[mode]));
    }

    /// The door's `Open` bool quality (`gmboolstats.emp` 702) on the door.
    fn send_open_quality(&mut self, open: bool) {
        self.send_quality(&QualityUpdate {
            object: Some(InstanceIdWithStamp {
                id: DOOR,
                instance: 1,
                other: 0,
            }),
            quality: OPEN_QUALITY,
            value: QualityValue::Bool(i32::from(open)),
        });
    }

    /// `WM_Usage` on the door: a shut door opens and an open one shuts. The server moves the
    /// door's mode with a latent `DoMode` (the body plays the transition's behaviour), sets the
    /// `Open` quality and answers the use with success.
    pub(crate) fn answer_door(&mut self, desc: &UsageDesc) {
        let open = !self.door_open;
        self.door_open = open;
        let mode = if open { DOOR_OPEN } else { DOOR_SHUT };
        self.send_mode(DOOR, mode, Opcode::DO_MODE);
        self.send_open_quality(open);
        self.answer_success(desc);
    }

    /// `WM_Usage` on the lever: the server sends the lever's flip behaviour (`DoBehavior` with
    /// the behaviour alone, so the mode stays), and the use is answered with success.
    pub(crate) fn answer_lever(&mut self, desc: &UsageDesc) {
        self.send_event(&object_event_bytes(
            Opcode::DO_BEHAVIOR,
            LEVER,
            1,
            // `BehaviorParams` bit 0x1: the behaviour only.
            &[1, LEVER_FLIP],
        ));
        self.answer_success(desc);
    }

    /// `UsageDone` with no error.
    fn answer_success(&mut self, desc: &UsageDesc) {
        let mut done = desc.clone();
        done.error = 0;
        self.send_script(&ScriptMessage::UsageDone(done));
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
        self.answer_success(desc);
    }
}
