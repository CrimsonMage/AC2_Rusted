//! Butchering a corpse: the player uses a butchery tool on a corpse (`WM_Usage` 0x22 with a
//! `UsageDesc` naming the tool as the item and the corpse as the target) and receives its skin.
//!
//! What the client does is traced: the tool needs a target (`Usage_NumberOfEntitiesUsedWith`, int
//! 720, at least 1; `Usage_InventoryRequired`, bool 305; `Usage_ValidTargetTypes`, int 722, masked
//! with the target's `Usage_TargetType`, int 721, in `CUsageSystem` 0xB: a monster's corpse is 0x40), so `CUsageSystem` 0x9 shows the targeting
//! pointer and keeps the tool in the usage list until the corpse is used (`CUsageSystem` 0x3),
//! which sends the desc. A failure comes back as `WM_Usage` 0x23 (`CUsageSystem` 0x2), whose
//! switch has a string for each `Usage_Butchery_*` error (`gmWErrorType` 10061-10065).
//!
//! The server is the double's, not retail's (the butchery rules run on the retail server and are
//! not in the client): a corpse butchers once; it must hold a skin; the pack needs a free slot.
//! A success marks the corpse `Death_IsBeingButchered` (bool 293, 0x125), makes the skin in the
//! player's pack and answers the use. The tool's damage (`Usage_Butchery_TooDamaged`) is not
//! modelled.

use ac2_net::protocol::{
    InstanceIdWithStamp, QualityUpdate, QualityValue, ScriptMessage, UsageDesc,
};

use crate::corpse::{CORPSE, MONSTER_CORPSE_TARGET};
use crate::items::{item_object_bytes, ItemSpawn};
use crate::{State, PLAYER};

/// The butchery tool in the player's pack, and the skin a butchered corpse gives.
pub const BUTCHER_KNIFE: u64 = 0x8000_0000_0000_0110;
pub const BUTCHER_SKIN: u64 = 0x8000_0000_0000_0111;
/// `wsl_butcherytool.entity`: the tool's entity description (its `ButcheryToolUsageAction`, package
/// 0x3539, and usage qualities).
pub const BUTCHERY_TOOL_ENTITY: u32 = 0x4700_1113;
/// `Death_IsBeingButchered` (`gmboolstats.emp`, 293).
pub const IS_BEING_BUTCHERED: u32 = 0x125;
/// `Usage_NumberOfEntitiesUsedWith` (int) and `Usage_InventoryRequired` (bool): what makes a use
/// wait for a target.
const ENTITIES_USED_WITH: u32 = 720;
const INVENTORY_REQUIRED: u32 = 305;
/// `Usage_ValidTargetTypes` (int 722): the `SkillValidTargets` the tool may be used on.
const VALID_TARGET_TYPES: u32 = 722;
/// `gmWErrorType`: `Usage_Butchery_AlreadyButchered`, `_NotButcherable`, `_Empty`, `_NoRoom`.
pub const ALREADY_BUTCHERED: u32 = 10061;
pub const NOT_BUTCHERABLE: u32 = 10062;
pub const EMPTY: u32 = 10064;
pub const NO_ROOM: u32 = 10065;
/// `CItem`, the package of the tool and the skin.
const ITEM_PACKAGE: u32 = 0x1AA9;

impl State {
    /// Puts the butchery tool in the player's pack (its object, its two usage qualities and the
    /// pack's listing).
    pub fn give_butcher_tool(&mut self) {
        let knife = ItemSpawn {
            icon: 0x0600_1005,
            value: 15,
            entity: Some(BUTCHERY_TOOL_ENTITY),
            ..ItemSpawn::new(BUTCHER_KNIFE, ITEM_PACKAGE, "Butchering Knife", PLAYER)
        };
        self.send_event(&item_object_bytes(&knife));
        for (quality, value) in [
            (ENTITIES_USED_WITH, QualityValue::Int(1)),
            (INVENTORY_REQUIRED, QualityValue::Bool(1)),
            (VALID_TARGET_TYPES, QualityValue::Int(MONSTER_CORPSE_TARGET)),
        ] {
            self.send_quality(&QualityUpdate {
                object: Some(InstanceIdWithStamp {
                    id: BUTCHER_KNIFE,
                    instance: 1,
                    other: 0,
                }),
                quality,
                value,
            });
        }
        let player = self.player();
        self.script_world
            .contents
            .entry(player)
            .or_default()
            .1
            .push(BUTCHER_KNIFE);
        self.send_contents(player);
    }

    /// Makes the corpse a creature without a skin to take (the butchery profile is empty).
    pub fn set_corpse_skinless(&mut self, skinless: bool) {
        self.script_world.corpse_skinless = skinless;
    }

    /// A use of the butchery tool: the failure the double's rules give, else the skin.
    pub(crate) fn answer_butcher(&mut self, desc: &UsageDesc) {
        let target = desc.target;
        let player = self.player();
        let pack = self.script_world.contents.get(&player);
        let slots = self.script_world.capacities.get(&player).copied();
        let full = pack
            .zip(slots)
            .is_some_and(|(p, slots)| p.0.len() + p.1.len() >= slots as usize);
        let world = &self.script_world;
        let error = if target != CORPSE || !world.contents.contains_key(&CORPSE) {
            Some(NOT_BUTCHERABLE)
        } else if world.butchered.contains(&target) {
            Some(ALREADY_BUTCHERED)
        } else if world.corpse_skinless {
            Some(EMPTY)
        } else if full {
            Some(NO_ROOM)
        } else {
            None
        };
        if let Some(error) = error {
            self.send_script(&ScriptMessage::UsageError {
                error,
                target,
                item: desc.item,
            });
            return;
        }
        self.script_world.butchered.insert(target);
        self.send_quality(&QualityUpdate {
            object: Some(InstanceIdWithStamp {
                id: target,
                instance: 1,
                other: 0,
            }),
            quality: IS_BEING_BUTCHERED,
            value: QualityValue::Bool(1),
        });
        let skin = ItemSpawn {
            icon: 0x0600_1019,
            value: 25,
            ..ItemSpawn::new(BUTCHER_SKIN, ITEM_PACKAGE, "Mimbu Skin", player)
        };
        self.send_event(&item_object_bytes(&skin));
        self.script_world
            .contents
            .entry(player)
            .or_default()
            .1
            .push(BUTCHER_SKIN);
        self.send_contents(player);
        self.send_script(&ScriptMessage::UsageDone(desc.clone()));
    }
}
