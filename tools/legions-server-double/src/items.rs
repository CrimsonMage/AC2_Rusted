//! The double's starter inventory: a pack (`CContainer`, package 0x1AA6) in the player's
//! contents holding a stack of potions (`CItem` 0x1AA9, stackable) and a sword (`CWeapon`
//! 0x1AAB, with durability), each a server-made (dynamic) object whose `WeenieDesc` carries
//! what the inventory panels read and whose other qualities follow as updates.

use ac2_net::protocol::{InstanceIdWithStamp, QualityUpdate, QualityValue, StringInfo, WeenieDesc};

use crate::{words, State, PLAYER};

/// The pack, the potions, the sword.
pub const PACK: u64 = 0x8000_0000_0000_0101;
pub const POTIONS: u64 = 0x8000_0000_0000_0102;
pub const SWORD: u64 = 0x8000_0000_0000_0103;

/// One item as the double makes it.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemSpawn {
    pub id: u64,
    pub package: u32,
    pub name: &'static str,
    pub plural: Option<&'static str>,
    /// The icon (`DataID` 257).
    pub icon: u32,
    /// Int 262; `None` for a single item.
    pub quantity: Option<i32>,
    pub value: i32,
    pub container: u64,
    /// Durability current and max (ints 619 / 618).
    pub durability: Option<(i32, i32)>,
    /// Further qualities: stackable (bool 700), maximum stack (int 261), capacity (int 280).
    pub stackable: bool,
    pub max_stack: Option<i32>,
    pub capacity: Option<i32>,
}

/// `CreateObject` for a contained item: no `VisualDesc` parts, a `PhysicsDesc` without a
/// position (it is not in the world), the four timestamps, then its `WeenieDesc`.
pub fn item_object_bytes(item: &ItemSpawn) -> Vec<u8> {
    let mut b = 0x0001_0058u32.to_le_bytes().to_vec();
    b.extend(item.id.to_le_bytes());
    words(&mut b, &[0, 0]);
    b.extend(1u16.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    for stamp in [1u16, 0, 0, 0] {
        b.extend(stamp.to_le_bytes());
    }
    let desc = WeenieDesc {
        package_id: Some(item.package),
        name: Some(StringInfo::literal(item.name)),
        plural_name: item.plural.map(StringInfo::literal),
        icon: Some(item.icon),
        container: Some(item.container),
        quantity: item.quantity,
        value: Some(item.value),
        durability_current: item.durability.map(|d| d.0),
        durability_max: item.durability.map(|d| d.1),
        ..WeenieDesc::default()
    };
    b.extend(desc.to_bytes());
    b
}

/// The double's starter items.
pub fn starter_items() -> Vec<ItemSpawn> {
    vec![
        ItemSpawn {
            id: PACK,
            package: 0x1AA6,
            name: "Pack",
            plural: None,
            icon: 0x0600_1000,
            quantity: None,
            value: 10,
            container: PLAYER,
            durability: None,
            stackable: false,
            max_stack: None,
            capacity: Some(24),
        },
        ItemSpawn {
            id: POTIONS,
            package: 0x1AA9,
            name: "Healing Potion",
            plural: Some("Healing Potions"),
            icon: 0x0600_1001,
            quantity: Some(5),
            value: 20,
            container: PACK,
            durability: None,
            stackable: true,
            max_stack: Some(10),
            capacity: None,
        },
        ItemSpawn {
            id: SWORD,
            package: 0x1AAB,
            name: "Practice Sword",
            plural: None,
            icon: 0x0600_1002,
            quantity: None,
            value: 50,
            container: PACK,
            durability: Some((40, 50)),
            stackable: false,
            max_stack: None,
            capacity: None,
        },
    ]
}

impl State {
    /// Gives the player [`starter_items`]: each object, its further qualities (visual
    /// updates), and the contents the login message and the pack's opening list.
    pub fn give_starter_items(&mut self) {
        let items = starter_items();
        for item in &items {
            self.send_event(&item_object_bytes(item));
            let object = Some(InstanceIdWithStamp {
                id: item.id,
                instance: 1,
                other: 0,
            });
            let mut qualities = Vec::new();
            if item.stackable {
                qualities.push((700, QualityValue::Bool(1)));
            }
            if let Some(max) = item.max_stack {
                qualities.push((261, QualityValue::Int(max)));
            }
            if let Some(capacity) = item.capacity {
                qualities.push((280, QualityValue::Int(capacity)));
            }
            for (quality, value) in qualities {
                self.send_quality(&QualityUpdate {
                    object,
                    quality,
                    value,
                });
            }
        }
        let world = &mut self.script_world;
        world.contents.insert(PLAYER, (vec![PACK], Vec::new()));
        world
            .contents
            .insert(PACK, (Vec::new(), vec![POTIONS, SWORD]));
    }
}
