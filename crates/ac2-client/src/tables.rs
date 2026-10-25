//! The string tables the session's player-script stand-in names, resolved from the DATs as
//! `StringInfo::SetStringIDandTableEnum` does: `FileArray::GetDIDFromEnum(enum, 0x40000004)`,
//! kept only when `MasterDBMap::DivineType` says the DID is a string table. Run it on the
//! loader side (it reads the enum maps) and hand the result to
//! `OnlineSession::set_string_tables`.

use ac2_script::data::DB_TYPE_STRING_TABLE;
use ac2_script::vm::ScriptData;
use ac2_weenie::StringTableIds;

/// `FileArray::GetDIDFromEnum`'s group for string tables.
const STRING_TABLE_GROUP: u32 = 0x4000_0004;

/// `StringTableIds::ENUMS` resolved through `data`.
pub fn string_table_ids(data: &dyn ScriptData) -> StringTableIds {
    let mut tables = StringTableIds::default();
    for table_enum in StringTableIds::ENUMS {
        let did = data
            .enum_did(table_enum, STRING_TABLE_GROUP)
            .filter(|did| data.db_type(*did) == Some(DB_TYPE_STRING_TABLE))
            .unwrap_or(0);
        tables.insert(table_enum, did);
    }
    tables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stand_ins_tables_resolve_to_string_table_dids() {
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        let store = ac2_db::ResourceStore::new(dataset.clone());
        let data = ac2_script::data::DatScriptData::new(dataset, store);
        let tables = string_table_ids(&data);
        for table_enum in StringTableIds::ENUMS {
            let did = tables.did(table_enum);
            assert_eq!(did >> 24, 0x25, "{table_enum:#x} -> {did:#x}");
        }
        // The usage table is the one `UsageDesc`'s class-default message names.
        assert_eq!(tables.did(ac2_weenie::TABLE_ENUM_USAGE), 0x2500_0014);
    }
}
