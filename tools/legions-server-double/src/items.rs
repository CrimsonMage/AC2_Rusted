//! The double's starter inventory and money: the player's pack holds a stack of potions
//! (`CItem` 0x1AA9, stackable) and a sword (`CWeapon` 0x1AAB, with durability, wielded in a
//! hand) beside a side pack (`CContainer` 0x1AA6) with rations in it; each a server-made
//! (dynamic) object whose `WeenieDesc` carries what the inventory panels read and whose other
//! qualities follow as updates. The money bag's requests (`WM_Money` 0x7 / 0x8) move money.

use ac2_net::protocol::{
    entry_flags, ntype, HeapObject, InstanceIdWithStamp, NativeObject, PackValue, PackedEntry,
    PackedReference, QualityUpdate, QualityValue, ScriptList, StringInfo, WeenieDesc,
};

use crate::{words, State, PLAYER};

/// The side pack, the potions, the sword, the rations.
pub const PACK: u64 = 0x8000_0000_0000_0101;
pub const POTIONS: u64 = 0x8000_0000_0000_0102;
pub const SWORD: u64 = 0x8000_0000_0000_0103;
pub const RATIONS: u64 = 0x8000_0000_0000_0104;
/// The worn starter shield ([`State::give_starter_gear`]).
pub const SHIELD: u64 = 0x8000_0000_0000_0105;

/// The starter items' icons: real RenderSurfaces of the Legions portal DAT, so the inventory
/// draws them (`icon_it_uncoverpast_satchel`, `icon_it_potion`, `icon_it_swordbasicggw`,
/// `icon_it_flatbread_siege_stores`, `icon_it_pyrealblue`, `icon_it_hum_shield_basicroundshield_pri`).
pub const ICON_PACK: u32 = 0x4100_7F6F;
pub const ICON_POTION: u32 = 0x4100_6F03;
pub const ICON_SWORD: u32 = 0x4100_829F;
pub const ICON_RATION: u32 = 0x4100_A146;
pub const ICON_COIN: u32 = 0x4100_A21E;
pub const ICON_SHIELD: u32 = 0x4100_441B;
/// The starter quests' icon (`icon_it_scroll_generals_quest`).
pub const ICON_QUEST: u32 = 0x4100_A21A;
/// The starter shield's visual (`it_hum_shield_basicroundshield_craft.vd`).
pub const SHIELD_VISUAL: u32 = 0x1F00_1514;
/// The holding locations the starter gear hangs from: the right and the left weapon holders
/// (`gmbonetype` `rweaponholder` / `lweaponholder` on the human Setup).
pub const RIGHT_HAND_HOLD: u32 = 1;
pub const LEFT_HAND_HOLD: u32 = 2;
/// The first starter quest's id ([`State::give_starter_quests`]).
pub const STARTER_QUEST: u32 = 0x100;
/// `gmQuestInfo::_questStatus`: underway and completed.
pub const QUEST_UNDERWAY: u32 = 0x1000_0000;
pub const QUEST_COMPLETED: u32 = 0x3000_0000;
/// The player's own pack's slots (int 280 on the player).
pub const MAIN_PACK_CAPACITY: i32 = 24;
/// The money the starter inventory comes with.
pub const STARTER_MONEY: i32 = 250;
/// The first id of a coin stack the double makes (`make_coins` counts up from it).
const COINS_BASE: u64 = 0x8000_0000_0000_0200;
/// The most coins a stack holds, as the double's coin rule (`GetCoinMaxStackSize`).
const COIN_MAX_STACK: i32 = 1000;
/// `PrimaryHand_InvLoc` / `SecondaryHand_InvLoc`.
pub const PRIMARY_HAND: u32 = 0x40;
pub const SECONDARY_HAND: u32 = 0x80;

/// `CContainer`, the script package of a container (the side pack, the world chest).
pub const CONTAINER_PACKAGE: u32 = 0x1AA6;
/// `ContainerSegmentDescriptor`: slot 0 the segment's slots, slot 1 the items in it.
const SEGMENT_PACKAGE: u32 = 0x24A7;

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
    /// Further qualities: stackable (bool 700), maximum stack (int 261), capacity (int 280),
    /// valid and preferred equip locations (ints 275, 282).
    pub stackable: bool,
    pub max_stack: Option<i32>,
    pub capacity: Option<i32>,
    pub locations: Option<(u32, u32)>,
    /// The entity description (`WeenieDesc` bit 0x0200_0000, an `EntityDesc` DID): the DAT record
    /// that gives the object its usage action and usage qualities.
    pub entity: Option<u32>,
    /// The item's `VisualDesc` (a DAT visual), for an item that is drawn when wielded; `None` for
    /// one with no parts.
    pub visual: Option<u32>,
}

impl ItemSpawn {
    pub(crate) fn new(id: u64, package: u32, name: &'static str, container: u64) -> Self {
        Self {
            id,
            package,
            name,
            plural: None,
            icon: 0,
            quantity: None,
            value: 0,
            container,
            durability: None,
            stackable: false,
            max_stack: None,
            capacity: None,
            locations: None,
            entity: None,
            visual: None,
        }
    }
}

/// `CreateObject` for a contained item: its `VisualDesc` (none, or its DAT visual), a
/// `PhysicsDesc` without a position (it is not in the world), the four timestamps, then its
/// `WeenieDesc`.
pub fn item_object_bytes(item: &ItemSpawn) -> Vec<u8> {
    let mut b = 0x0001_0058u32.to_le_bytes().to_vec();
    b.extend(item.id.to_le_bytes());
    match item.visual {
        Some(visual) => words(&mut b, &[2, visual]),
        None => words(&mut b, &[0]),
    }
    words(&mut b, &[0]);
    b.extend(1u16.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    for stamp in [1u16, 0, 0, 0] {
        b.extend(stamp.to_le_bytes());
    }
    let desc = WeenieDesc {
        package_id: Some(item.package),
        entity_did: item.entity,
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
            icon: ICON_PACK,
            value: 10,
            capacity: Some(24),
            ..ItemSpawn::new(PACK, CONTAINER_PACKAGE, "Pack", PLAYER)
        },
        ItemSpawn {
            plural: Some("Healing Potions"),
            icon: ICON_POTION,
            quantity: Some(5),
            value: 20,
            stackable: true,
            max_stack: Some(10),
            ..ItemSpawn::new(POTIONS, 0x1AA9, "Healing Potion", PLAYER)
        },
        ItemSpawn {
            icon: ICON_SWORD,
            value: 50,
            durability: Some((40, 50)),
            locations: Some((PRIMARY_HAND | SECONDARY_HAND, PRIMARY_HAND)),
            ..ItemSpawn::new(SWORD, 0x1AAB, "Practice Sword", PLAYER)
        },
        ItemSpawn {
            plural: Some("Rations"),
            icon: ICON_RATION,
            quantity: Some(3),
            value: 2,
            stackable: true,
            max_stack: Some(20),
            ..ItemSpawn::new(RATIONS, 0x1AA9, "Ration", PACK)
        },
    ]
}

/// The segments `WM_Inventory` 0x53 (and `WM_Player` 0x27's argument 10) carry for a
/// container (`Container` slot 18, read by `Container` 0x3F `GetContainerSegmentInfo`): an
/// `RList` of `ContainerSegmentDescriptor`s. The double gives each container one segment, all
/// its slots and items.
pub fn segments(capacity: u32, items: usize) -> PackedReference {
    let heap = |pkg_id, native, slots: Vec<u32>, tags| PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native,
            slots,
            tags,
        },
    };
    let list = NativeObject::List(ScriptList {
        native_type: ntype::R_LIST,
        items: vec![2],
    });
    PackedReference {
        ids: vec![1, 2],
        entries: vec![
            heap(u32::MAX, Some(list), Vec::new(), Vec::new()),
            heap(
                SEGMENT_PACKAGE,
                None,
                vec![capacity, items as u32],
                vec![2, 2],
            ),
        ],
    }
}

impl State {
    /// Gives the player [`starter_items`] and [`STARTER_MONEY`]: each object, its further
    /// qualities (visual updates), the player's pack size, and the contents the login message
    /// and the side pack's opening list carry.
    pub fn give_starter_items(&mut self) {
        let items = starter_items();
        for item in &items {
            self.send_item(item);
        }
        self.send_private(280, QualityValue::Int(MAIN_PACK_CAPACITY));
        let player = self.player();
        let world = &mut self.script_world;
        for item in &items {
            if let Some(capacity) = item.capacity {
                world.capacities.insert(item.id, capacity as u32);
            }
        }
        world.capacities.insert(player, MAIN_PACK_CAPACITY as u32);
        world
            .contents
            .insert(player, (vec![PACK], vec![POTIONS, SWORD]));
        world.contents.insert(PACK, (Vec::new(), vec![RATIONS]));
        self.give_money(STARTER_MONEY);
    }

    /// An item's object and its stackable, stack, capacity and equip-location qualities.
    fn send_item(&mut self, item: &ItemSpawn) {
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
        if let Some((valid, preferred)) = item.locations {
            qualities.push((275, QualityValue::Int(valid as i32)));
            qualities.push((282, QualityValue::Int(preferred as i32)));
        }
        for (quality, value) in qualities {
            self.send_quality(&QualityUpdate {
                object,
                quality,
                value,
            });
        }
    }

    /// The starter gear, after [`State::give_starter_items`] and before the player's login state
    /// ([`State::send_init_player`]): a round shield worn in the off hand and the practice sword
    /// wielded, so the paper doll's slots show worn items. Both are drawn in the character's
    /// hands: each gets its visual and hangs from its holding point (`RecvEvt_Parent`), as a
    /// server wields what the player equips (the world viewer's session).
    pub fn give_starter_gear(&mut self) {
        let player = self.player();
        let sword = starter_items()
            .into_iter()
            .find(|item| item.id == SWORD)
            .map(|item| ItemSpawn {
                visual: Some(crate::SWORD_VISUAL),
                ..item
            })
            .expect("the starter sword");
        self.send_event(&item_object_bytes(&sword));
        let shield = ItemSpawn {
            icon: ICON_SHIELD,
            value: 30,
            durability: Some((60, 60)),
            locations: Some((SECONDARY_HAND, SECONDARY_HAND)),
            visual: Some(SHIELD_VISUAL),
            ..ItemSpawn::new(SHIELD, 0x1AAB, "Round Shield", player)
        };
        self.send_item(&shield);
        self.inventory_move(SWORD, 0, -1);
        let world = &mut self.script_world;
        world.equipped.push((PRIMARY_HAND, SWORD));
        world.equipped.push((SECONDARY_HAND, SHIELD));
        self.wield(SWORD, player, RIGHT_HAND_HOLD, 0);
        self.wield(SHIELD, player, LEFT_HAND_HOLD, 0);
    }

    /// Starter quests for the login message ([`State::send_init_player`]): one underway and one
    /// completed, so the quest journal lists both pages (the world viewer's session).
    pub fn give_starter_quests(&mut self) {
        let quest = |id, name: &str, journal: &str, status| ac2_net::protocol::GmQuestInfo {
            id,
            name: literal(name),
            description: literal(journal),
            icon: ICON_QUEST,
            status,
            phase: 1,
            journal: literal(journal),
            challenge_level: 1,
            ..ac2_net::protocol::GmQuestInfo::default()
        };
        self.script_world.quests = vec![
            quest(
                STARTER_QUEST,
                "Ruins of the Old Tower",
                "Explore the old tower and report what you find.",
                QUEST_UNDERWAY,
            ),
            quest(
                STARTER_QUEST + 1,
                "First Steps",
                "You learned to find your way around.",
                QUEST_COMPLETED,
            ),
        ];
    }

    /// The coin stack of `amount` coins the double makes for `DragFromMoneyBag`: its object and
    /// stackable and maximum-stack qualities, then the container listed with it at `slot` (-1:
    /// the end). Returns its id.
    pub fn make_coins(&mut self, container: u64, slot: i32, amount: i32) -> u64 {
        let c = &mut self.script_world.character;
        c.coins_made += 1;
        let coin = ItemSpawn {
            plural: Some("Coins"),
            icon: ICON_COIN,
            quantity: Some(amount),
            stackable: true,
            max_stack: Some(COIN_MAX_STACK),
            ..ItemSpawn::new(
                COINS_BASE + u64::from(c.coins_made),
                0x1AA9,
                "Coin",
                container,
            )
        };
        self.send_event(&item_object_bytes(&coin));
        let object = Some(InstanceIdWithStamp {
            id: coin.id,
            instance: 1,
            other: 0,
        });
        for (quality, value) in [
            (700, QualityValue::Bool(1)),
            (261, QualityValue::Int(COIN_MAX_STACK)),
        ] {
            self.send_quality(&QualityUpdate {
                object,
                quality,
                value,
            });
        }
        let items = &mut self.script_world.contents.entry(container).or_default().1;
        match usize::try_from(slot) {
            Ok(at) if at < items.len() => items.insert(at, coin.id),
            _ => items.push(coin.id),
        }
        self.send_contents(container);
        coin.id
    }

    /// The server adds `amount` to the player's money (int 3100, `CPlayer` 0x85 -> 0x71:
    /// `SendNotice_UpdateMoney`).
    pub fn give_money(&mut self, amount: i32) {
        let c = &mut self.script_world.character;
        c.money += amount;
        let money = c.money;
        self.send_private(3100, QualityValue::Int(money));
    }

    /// `WM_Money` 0x8 (`DragFromMoneyBag`: container, slot, quantity) takes the money and
    /// answers `WM_Money` 0x9 with the coin stack made of it, in the container at the slot; 0x7
    /// (`DragToMoneyBag`: container, slot, quantity, item) adds the quantity, takes the item
    /// out of its container, answers `WM_Money` 0x6 and lists the container again.
    pub(crate) fn answer_money(&mut self, function: u32, args: &[PackValue]) {
        match (function, args) {
            (
                0x820F_0001,
                [PackValue::LongInt(container), PackValue::Int(slot), PackValue::Int(quantity)],
            ) => {
                self.give_money(-(*quantity as i32));
                let coin = self.make_coins(*container, *slot as i32, *quantity as i32);
                self.send_script_call(
                    0x820F_0009,
                    &[
                        PackValue::LongInt(*container),
                        PackValue::Int(*slot),
                        PackValue::LongInt(coin),
                    ],
                );
            }
            (
                0x820F_0002,
                [PackValue::LongInt(container), _, PackValue::Int(quantity), PackValue::LongInt(item)],
            ) => {
                for (_, items) in self.script_world.contents.values_mut() {
                    items.retain(|i| i != item);
                }
                self.give_money(*quantity as i32);
                self.send_script_call(0x820F_0006, &[PackValue::LongInt(*item)]);
                self.send_contents(*container);
            }
            _ => {}
        }
    }
}

fn literal(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.into()),
        ..StringInfo::default()
    }
}
