//! What the inventory panels show of an item, read from its qualities the way the `gmEntity`
//! getters do (`gmCWInterface::GetQuantity` and the rest call them on the item's process):
//! `gCWRepository::GetRawInt` / `GetRawBool` / `GetRawDataID` / `GetRawStringInfo` with a
//! default, over `CBaseQualities::Inq*` (the `gmWeenieDesc` first).

use ac2_net::protocol::{StringInfo, StringVariable};

use crate::qualities::WeenieQualities;
use crate::tables::{StringTableIds, TABLE_ENUM_CHAT_WINDOW};

/// The qualities the getters read.
pub mod item_quality {
    /// String infos: the name and its plural.
    pub const NAME: u32 = 256;
    pub const PLURAL_NAME: u32 = 257;
    /// Data id: the icon.
    pub const ICON: u32 = 257;
    /// Ints.
    pub const MAX_STACK_SIZE: u32 = 261;
    pub const QUANTITY: u32 = 262;
    pub const MAX_CONTAINER_SIZE: u32 = 280;
    pub const VALUE: u32 = 612;
    pub const MAX_DURABILITY: u32 = 618;
    pub const DURABILITY: u32 = 619;
    /// Bool.
    pub const STACKABLE: u32 = 700;
}

/// `gmEntity` 0xAD's literal when the name is missing.
pub const MISSING_NAME: &str = "Error! Name_StringInfoStat missing";
/// `gmEntity` 0x9F: the plural string on the name's table.
const PLURAL_STRING: u32 = 0x3565_4074;
/// `gmEntity` 0x10B: "<count> <plural>" and its two variables.
const STACK_NAME_STRING: u32 = 0xFE4D_81DA;
const STACK_COUNT_VARIABLE: u32 = 0x9D56_6800;
const STACK_PLURAL_VARIABLE: u32 = 0x8615_ECB3;
/// `gmEntity` 0xC's default capacity.
const DEFAULT_CONTAINER_SIZE: i32 = 12;

/// One item's panel stats.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemStats {
    /// `gmEntity` 0xAD: StringInfo 256, or the "name missing" literal.
    pub name: StringInfo,
    /// `gmEntity` 0x9F: StringInfo 257, else the plural string on the name's table. Retail
    /// checks each with `StringInfo::IsValid` against the DAT's string tables and ends on an
    /// empty literal; that check is the resolver's.
    pub plural: StringInfo,
    /// `gmEntity` 0x23: DataID 257.
    pub icon: u32,
    /// `gmEntity` 0x10: int 262, default 1.
    pub quantity: i32,
    /// `gmEntity` 0xB: bool 700.
    pub stackable: bool,
    /// `gmEntity` 0x2E: int 261; `None` means `InventoryGlobals`' default (its slot 5), which
    /// only the script knows.
    pub max_stack: Option<i32>,
    /// `gmEntity` 0x159 / 0x15B (current, max: ints 619, 618) when 0x158 says the item has
    /// durability (int 618 above 0).
    pub durability: Option<(i32, i32)>,
    /// Int 612, the `gmWeenieDesc` value.
    pub value: i32,
    /// `gmEntity` 0xC: int 280, default 12 (retail passes it through the effect system).
    pub capacity: i32,
}

impl ItemStats {
    pub fn of(q: &WeenieQualities) -> Self {
        use item_quality::*;
        let int = |stat, default| q.inq_int(stat).unwrap_or(default);
        let name = q.inq_string_info(NAME).unwrap_or_else(|| StringInfo {
            literal: Some(MISSING_NAME.to_owned()),
            ..StringInfo::default()
        });
        let plural = q
            .inq_string_info(PLURAL_NAME)
            .unwrap_or_else(|| StringInfo {
                string_id: PLURAL_STRING,
                table_id: name.table_id,
                ..StringInfo::default()
            });
        let max_durability = int(MAX_DURABILITY, 0);
        ItemStats {
            name,
            plural,
            icon: q.inq_data_id(ICON).unwrap_or(0),
            quantity: int(QUANTITY, 1),
            stackable: q.inq_bool(STACKABLE).unwrap_or(0) != 0,
            max_stack: q.inq_int(MAX_STACK_SIZE),
            durability: (max_durability > 0).then(|| (int(DURABILITY, 0), max_durability)),
            value: int(VALUE, 0),
            capacity: int(MAX_CONTAINER_SIZE, DEFAULT_CONTAINER_SIZE),
        }
    }

    /// `gmEntity` 0x10B (`GetPotentiallyPluralizedName`): "<quantity> <plural>" for a stack of
    /// more than one, else the name.
    pub fn display_name(&self, quantity: u32, tables: &StringTableIds) -> StringInfo {
        if !(self.stackable && quantity > 1) {
            return self.name.clone();
        }
        StringInfo {
            string_id: STACK_NAME_STRING,
            table_id: tables.did(TABLE_ENUM_CHAT_WINDOW),
            literal: None,
            variables: vec![
                (
                    STACK_COUNT_VARIABLE,
                    StringVariable::Word {
                        kind: 2,
                        value: quantity,
                    },
                ),
                (
                    STACK_PLURAL_VARIABLE,
                    StringVariable::Nested(Box::new(self.plural.clone())),
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::QualityValue;

    fn named(text: &str) -> StringInfo {
        StringInfo {
            literal: Some(text.to_owned()),
            ..StringInfo::default()
        }
    }

    #[test]
    fn defaults_follow_the_getters() {
        let stats = ItemStats::of(&WeenieQualities::default());
        assert_eq!(stats.name, named(MISSING_NAME));
        assert_eq!(
            (stats.plural.string_id, stats.plural.table_id),
            (PLURAL_STRING, 0)
        );
        assert_eq!((stats.quantity, stats.icon, stats.capacity), (1, 0, 12));
        assert_eq!(
            (stats.stackable, stats.max_stack, stats.durability),
            (false, None, None)
        );
    }

    #[test]
    fn a_stack_names_its_count_and_plural() {
        let mut q = WeenieQualities::default();
        q.set(
            256,
            &QualityValue::StringInfo(Box::new(named("Healing Potion"))),
        );
        q.set(
            257,
            &QualityValue::StringInfo(Box::new(named("Healing Potions"))),
        );
        q.set(262, &QualityValue::Int(5));
        q.set(700, &QualityValue::Bool(1));
        q.set(618, &QualityValue::Int(50));
        q.set(619, &QualityValue::Int(40));
        let stats = ItemStats::of(&q);
        assert_eq!(stats.durability, Some((40, 50)));
        let mut tables = StringTableIds::default();
        tables.insert(TABLE_ENUM_CHAT_WINDOW, 0x2300_0005);
        let name = stats.display_name(5, &tables);
        assert_eq!(
            (name.string_id, name.table_id),
            (STACK_NAME_STRING, 0x2300_0005)
        );
        assert_eq!(
            name.variables,
            vec![
                (
                    STACK_COUNT_VARIABLE,
                    StringVariable::Word { kind: 2, value: 5 }
                ),
                (
                    STACK_PLURAL_VARIABLE,
                    StringVariable::Nested(Box::new(named("Healing Potions")))
                ),
            ]
        );
        assert_eq!(stats.display_name(1, &tables), named("Healing Potion"));
    }
}
