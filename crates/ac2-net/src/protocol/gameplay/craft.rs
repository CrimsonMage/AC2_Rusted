//! The crafting messages (`WM_Craft`, package 0x206): the server's answers to a craft and the
//! client's requests. The answers are forwarders the server reaches (`WM_Craft` 0x16 ->
//! `CPlayer` 0x30 -> `CM_Craft::SendNotice_AddIngredient_Done`; 0x1C -> `CPlayer` 0xF7 ->
//! `CM_Craft::SendNotice_ExecuteCraft_Done`), so they list their values last parameter first, as the
//! store and trade messages do. The requests are the stubs the IFuns call
//! (`gmCWInterface::RequestAddIngredient` -> 0x06, `RequestExecuteCraft` -> 0x12,
//! `RequestRaiseCraftSkill` -> 0x2E); their values are in the stub's pack order, which
//! [`ScriptRequest`](super::ScriptRequest) keeps.
//!
//! The registry messages (`WM_Craft` 0x27, 0x34, 0x1A, 0x37) fill the player's `CraftRegistry`
//! (`CPlayer` 0x23 returns it from slot 0x21): the skill hash (slot 5) and the recipe hash (slot 6),
//! keyed by each record's DID. A record is the DAT WState itself, packed as a singleton
//! ([`PackedEntry::Singleton`]); a list of records is an `RList` (native 0x65) whose items name the
//! singleton entries by reference id, laid out as `StoreView`'s sales are.

use super::{function, Args};
use crate::protocol::native::{NativeObject, ScriptHash, ScriptList};
use crate::protocol::pack::{entry_flags, HeapObject, PackValue, PackedEntry, PackedReference};
use crate::protocol::ProtocolError;

/// A server answer to a craft (`WM_Craft` forwarders).
#[derive(Debug, Clone, PartialEq)]
pub enum CraftMessage {
    /// [`function::CRAFT_ADD_INGREDIENT_DONE`]: an ingredient was added (or refused) for slot
    /// `ordinal` of `recipe`, with the spinner's value (`CM_Craft::SendNotice_AddIngredient_Done`).
    AddIngredientDone {
        recipe: u32,
        ordinal: u32,
        ingredient: u64,
        spinner: u32,
        error: u32,
    },
    /// [`function::CRAFT_EXECUTE_DONE`]: the craft of `recipe` finished with `error` (0: crafted).
    /// `notify` is the flag `CPlayer` 0xF7 tests before it raises the notice: a zero flag raises
    /// nothing (its meaning is not traced; the double sends 1).
    ExecuteCraftDone {
        notify: u32,
        recipe: u32,
        error: u32,
    },
    /// One of the panel forwarders ([`function::CRAFT_UPDATE_PANEL_19`] and the rest): raises
    /// `CM_Craft::SendNotice_UpdateCraftPanel`. `forwarder` is the function the answer is sent to;
    /// `value` is the DID of the object its one reference argument names (the notice does not carry
    /// it; the Legions wlib's type for it is not traced).
    UpdateCraftPanel { forwarder: u32, value: u32 },
    /// [`function::CRAFT_ADD_SKILL`]: craft skill `skill` (its DID) is added to the registry's skill
    /// hash; the forwarder raises `UpdateCraftSkill` and `UpdateCraftPanel`.
    AddCraftSkill { skill: u32 },
    /// [`function::CRAFT_ADD_SKILLS`]: each craft skill of `skills` is added as
    /// [`CraftMessage::AddCraftSkill`] does, from an `RList` of records.
    AddCraftSkills { skills: Vec<u32> },
    /// [`function::CRAFT_ADD_RECIPE`]: recipe `recipe` (its DID) is added to the registry's recipe
    /// hash. A recipe already known changes nothing (the forwarder returns 0x9C51).
    AddRecipe { recipe: u32 },
    /// [`function::CRAFT_ADD_RECIPES`]: each recipe of `recipes` is added as
    /// [`CraftMessage::AddRecipe`] does, from an `RList` of records.
    AddRecipes { recipes: Vec<u32> },
}

impl CraftMessage {
    /// The function id the message is sent to.
    pub fn function(&self) -> u32 {
        match self {
            Self::AddIngredientDone { .. } => function::CRAFT_ADD_INGREDIENT_DONE,
            Self::ExecuteCraftDone { .. } => function::CRAFT_EXECUTE_DONE,
            Self::UpdateCraftPanel { forwarder, .. } => *forwarder,
            Self::AddCraftSkill { .. } => function::CRAFT_ADD_SKILL,
            Self::AddCraftSkills { .. } => function::CRAFT_ADD_SKILLS,
            Self::AddRecipe { .. } => function::CRAFT_ADD_RECIPE,
            Self::AddRecipes { .. } => function::CRAFT_ADD_RECIPES,
        }
    }

    /// The packed arguments in wire order: the answer's values last parameter first, except the
    /// execute answer, whose forwarder packs the flag, the error, then the recipe (traced by a
    /// probe: a zero flag raises nothing, the notice's recipe is the last value).
    pub fn arguments(&self) -> Vec<PackValue> {
        match self {
            Self::AddIngredientDone {
                recipe,
                ordinal,
                ingredient,
                spinner,
                error,
            } => vec![
                PackValue::Int(*error),
                PackValue::Int(*spinner),
                PackValue::LongInt(*ingredient),
                PackValue::Int(*ordinal),
                PackValue::Int(*recipe),
            ],
            Self::ExecuteCraftDone {
                notify,
                recipe,
                error,
            } => vec![
                PackValue::Int(*notify),
                PackValue::Int(*error),
                PackValue::Int(*recipe),
            ],
            Self::UpdateCraftPanel { value, .. } | Self::AddCraftSkill { skill: value } => {
                vec![singleton_ref(*value)]
            }
            Self::AddRecipe { recipe } => vec![singleton_ref(*recipe)],
            Self::AddCraftSkills { skills } => vec![list_ref(skills)],
            Self::AddRecipes { recipes } => vec![list_ref(recipes)],
        }
    }

    /// The server's answer from its function id and packed arguments, or `None` for another id.
    pub fn decode(function_id: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        Some(match function_id {
            function::CRAFT_ADD_INGREDIENT_DONE => decode_with(arguments, |a| {
                let error = a.int()?;
                let spinner = a.int()?;
                let ingredient = a.long()?;
                let ordinal = a.int()?;
                Ok(Self::AddIngredientDone {
                    recipe: a.int()?,
                    ordinal,
                    ingredient,
                    spinner,
                    error,
                })
            }),
            function::CRAFT_EXECUTE_DONE => decode_with(arguments, |a| {
                let notify = a.int()?;
                let error = a.int()?;
                Ok(Self::ExecuteCraftDone {
                    notify,
                    error,
                    recipe: a.int()?,
                })
            }),
            function::CRAFT_UPDATE_PANEL_18
            | function::CRAFT_UPDATE_PANEL_19
            | function::CRAFT_UPDATE_PANEL_1B => decode_with(arguments, |a| {
                Ok(Self::UpdateCraftPanel {
                    forwarder: function_id,
                    value: singleton_did(a)?,
                })
            }),
            function::CRAFT_ADD_SKILL => decode_with(arguments, |a| {
                Ok(Self::AddCraftSkill {
                    skill: singleton_did(a)?,
                })
            }),
            function::CRAFT_ADD_SKILLS => decode_with(arguments, |a| {
                Ok(Self::AddCraftSkills {
                    skills: list_dids(a)?,
                })
            }),
            function::CRAFT_ADD_RECIPE => decode_with(arguments, |a| {
                Ok(Self::AddRecipe {
                    recipe: singleton_did(a)?,
                })
            }),
            function::CRAFT_ADD_RECIPES => decode_with(arguments, |a| {
                Ok(Self::AddRecipes {
                    recipes: list_dids(a)?,
                })
            }),
            _ => return None,
        })
    }
}

/// Reads one craft answer's values with `read`, refusing any value left over.
fn decode_with(
    arguments: &[u8],
    read: impl FnOnce(&mut Args) -> Result<CraftMessage, ProtocolError>,
) -> Result<CraftMessage, ProtocolError> {
    let mut args = Args::new(arguments)?;
    let message = read(&mut args)?;
    args.finish()?;
    Ok(message)
}

/// A DAT record packed as the reference a server sends: one singleton entry, the WState DID
/// (`PackedEntry::Singleton`, resolved by the VM through `GetWStateReference`).
fn singleton_ref(did: u32) -> PackValue {
    PackValue::Reference(PackedReference {
        ids: vec![ROOT_ID],
        entries: vec![singleton_entry(did)],
    })
}

/// A list of DAT records packed as the reference a server sends: the root is an `RList` (native
/// 0x65) whose items are the reference ids of the singleton entries after it.
fn list_ref(dids: &[u32]) -> PackValue {
    let item_ids: Vec<u32> = (0..dids.len() as u32).map(|i| ROOT_ID + 1 + i).collect();
    let list = HeapObject {
        pkg_state_id: 0,
        pkg_id: u32::MAX,
        native: Some(NativeObject::List(ScriptList {
            native_type: R_LIST,
            items: item_ids.iter().map(|id| u64::from(*id)).collect(),
        })),
        slots: Vec::new(),
        tags: Vec::new(),
    };
    let mut ids = vec![ROOT_ID];
    ids.extend(item_ids);
    let mut entries = vec![PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: list,
    }];
    entries.extend(dids.iter().map(|did| singleton_entry(*did)));
    PackValue::Reference(PackedReference { ids, entries })
}

/// The singleton entry of WState `did`.
fn singleton_entry(did: u32) -> PackedEntry {
    PackedEntry::Singleton {
        flags: entry_flags::KIND_HEAP | entry_flags::SINGLETON,
        did,
    }
}

/// The DID of the singleton reference an answer's one value is.
fn singleton_did(args: &mut Args) -> Result<u32, ProtocolError> {
    let reference = args.reference()?;
    match reference.entries.as_slice() {
        [PackedEntry::Singleton { did, .. }] => Ok(*did),
        _ => Err(ProtocolError::Invalid("craft record")),
    }
}

/// The DIDs of the records in an answer's `RList` (each item names a singleton entry by its
/// reference id).
fn list_dids(args: &mut Args) -> Result<Vec<u32>, ProtocolError> {
    let reference = args.reference()?;
    let invalid = || ProtocolError::Invalid("craft record list");
    let Some(PackedEntry::Heap { object, .. }) = reference.entries.first() else {
        return Err(invalid());
    };
    let Some(NativeObject::List(list)) = object.native.as_ref() else {
        return Err(invalid());
    };
    list.items
        .iter()
        .map(|item| {
            let index = reference
                .ids
                .iter()
                .position(|id| u64::from(*id) == *item)?;
            match reference.entries.get(index)? {
                PackedEntry::Singleton { did, .. } => Some(*did),
                _ => None,
            }
        })
        .collect::<Option<Vec<u32>>>()
        .ok_or_else(invalid)
}

/// The ingredients a craft is executed from, as the `ALHash` the stub packs (native type 8, a
/// reference whose one heap entry holds the table): the slot ordinal and the object's IID.
pub(super) fn ingredient_table(entries: &[(u32, u64)]) -> PackValue {
    let keys = entries
        .iter()
        .map(|(ordinal, _)| crate::protocol::native::HashKey::Value(u64::from(*ordinal)))
        .collect();
    let values = entries.iter().map(|(_, object)| *object).collect();
    let table = HeapObject {
        pkg_state_id: 0,
        pkg_id: u32::MAX,
        native: Some(NativeObject::Hash(ScriptHash {
            native_type: ALHASH,
            bucket_index: DEFAULT_BUCKET_INDEX,
            keys,
            values,
        })),
        slots: Vec::new(),
        tags: Vec::new(),
    };
    PackValue::Reference(PackedReference {
        ids: vec![TABLE_ID],
        entries: vec![PackedEntry::Heap {
            flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
            object: table,
        }],
    })
}

/// The ingredient entries of an `ALHash` a request packed (ordinal, IID), or `None`.
pub(super) fn ingredient_entries(reference: &PackedReference) -> Option<Vec<(u32, u64)>> {
    let object = match reference.entries.first()? {
        PackedEntry::Heap { object, .. } => object,
        _ => return None,
    };
    match object.native.as_ref()? {
        NativeObject::Hash(hash) => hash
            .keys
            .iter()
            .zip(&hash.values)
            .map(|(key, value)| match key {
                crate::protocol::native::HashKey::Value(k) => Some((*k as u32, *value)),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

/// `R_LIST`: the native type of a list of records (`gRList`).
const R_LIST: u32 = 0x65;
/// The reference id of a message's root value.
const ROOT_ID: u32 = 1;
/// `ALHash`, the native type of the ingredient table.
const ALHASH: u32 = 0x08;
/// The bucket index of the default table (32 buckets), as the stub's table writes it.
const DEFAULT_BUCKET_INDEX: u8 = 2;
/// The reference id the table is packed under. The packing side's table index: the decoder
/// ignores it, so any id packs the same table.
const TABLE_ID: u32 = 1;

#[cfg(test)]
mod tests {
    use super::super::ScriptRequest;
    use super::*;
    use crate::protocol::client_event::ClientEvent;
    use crate::protocol::pack::pack_values;
    use crate::protocol::unpack_values;

    const RECIPE: u32 = 0x6F00_1F0C;
    const SKILL: u32 = 0x6F00_28D7;
    const INGREDIENT: u64 = 0x8000_0002_0000_0021;

    /// Each answer's packed values, last parameter first, unpacked as the VM reads them.
    #[test]
    fn add_ingredient_done_packs_its_last_parameter_first() {
        let message = CraftMessage::AddIngredientDone {
            recipe: RECIPE,
            ordinal: 2,
            ingredient: INGREDIENT,
            spinner: 7,
            error: 0,
        };
        assert_eq!(message.function(), function::CRAFT_ADD_INGREDIENT_DONE);
        let bytes = pack_values(&message.arguments());
        assert_eq!(
            unpack_values(&bytes).unwrap(),
            vec![
                PackValue::Int(0),
                PackValue::Int(7),
                PackValue::LongInt(INGREDIENT),
                PackValue::Int(2),
                PackValue::Int(RECIPE),
            ]
        );
        let decoded = CraftMessage::decode(function::CRAFT_ADD_INGREDIENT_DONE, &bytes)
            .expect("a craft answer")
            .unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn execute_craft_done_packs_the_flag_first() {
        let message = CraftMessage::ExecuteCraftDone {
            notify: 1,
            recipe: RECIPE,
            error: 0,
        };
        assert_eq!(message.function(), function::CRAFT_EXECUTE_DONE);
        let bytes = pack_values(&message.arguments());
        assert_eq!(
            unpack_values(&bytes).unwrap(),
            vec![PackValue::Int(1), PackValue::Int(0), PackValue::Int(RECIPE)]
        );
        let decoded = CraftMessage::decode(function::CRAFT_EXECUTE_DONE, &bytes)
            .expect("a craft answer")
            .unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn a_craft_answer_with_an_extra_value_is_refused() {
        let mut values = CraftMessage::ExecuteCraftDone {
            notify: 1,
            recipe: RECIPE,
            error: 0,
        }
        .arguments();
        values.push(PackValue::Int(9));
        let bytes = pack_values(&values);
        assert!(CraftMessage::decode(function::CRAFT_EXECUTE_DONE, &bytes)
            .unwrap()
            .is_err());
        assert!(CraftMessage::decode(0x8206_0001, &bytes).is_none());
    }

    /// `RequestAddIngredient`'s stub packs the spinner, the object, the ordinal and the recipe.
    #[test]
    fn add_ingredient_request_is_in_stub_order() {
        let request = ScriptRequest::CraftAddIngredient {
            recipe: RECIPE,
            ordinal: 2,
            object: INGREDIENT,
            spinner: 7,
        };
        let ClientEvent::Interp {
            function,
            arguments,
            ..
        } = request.to_event()
        else {
            panic!("an interp event");
        };
        assert_eq!(function, function::CRAFT_ADD_INGREDIENT);
        assert_eq!(
            unpack_values(&arguments).unwrap(),
            vec![
                PackValue::Int(7),
                PackValue::LongInt(INGREDIENT),
                PackValue::Int(2),
                PackValue::Int(RECIPE),
            ]
        );
        assert_eq!(
            ScriptRequest::decode(function, &arguments)
                .unwrap()
                .unwrap(),
            request
        );
    }

    /// `RequestExecuteCraft`'s stub packs a long int, the ingredient table, the spinner and the
    /// recipe; the table's entries are the (ordinal, IID) pairs.
    #[test]
    fn execute_craft_request_carries_the_ingredient_table() {
        let request = ScriptRequest::CraftExecute {
            helper: 0,
            recipe: RECIPE,
            spinner: 9,
            ingredients: vec![(0, 0x8000_0002_0000_0021), (1, 0x8000_0002_0000_0022)],
        };
        let ClientEvent::Interp {
            function,
            arguments,
            ..
        } = request.to_event()
        else {
            panic!("an interp event");
        };
        assert_eq!(function, function::CRAFT_EXECUTE);
        let values = unpack_values(&arguments).unwrap();
        assert_eq!(values[0], PackValue::LongInt(0));
        assert_eq!(values[2], PackValue::Int(9));
        assert_eq!(values[3], PackValue::Int(RECIPE));
        assert_eq!(
            ScriptRequest::decode(function, &arguments)
                .unwrap()
                .unwrap(),
            request
        );
    }

    #[test]
    fn raise_craft_skill_request_is_one_int() {
        let request = ScriptRequest::CraftRaiseSkill(SKILL);
        let ClientEvent::Interp {
            function,
            arguments,
            ..
        } = request.to_event()
        else {
            panic!("an interp event");
        };
        assert_eq!(function, function::CRAFT_RAISE_SKILL);
        assert_eq!(
            unpack_values(&arguments).unwrap(),
            vec![PackValue::Int(SKILL)]
        );
    }

    /// A panel forwarder takes one singleton reference (the object's WState DID).
    #[test]
    fn update_panel_answers_pack_one_singleton_for_their_forwarder() {
        for forwarder in [
            function::CRAFT_UPDATE_PANEL_18,
            function::CRAFT_UPDATE_PANEL_19,
            function::CRAFT_UPDATE_PANEL_1B,
        ] {
            let message = CraftMessage::UpdateCraftPanel {
                forwarder,
                value: RECIPE,
            };
            assert_eq!(message.function(), forwarder);
            let bytes = pack_values(&message.arguments());
            let values = unpack_values(&bytes).unwrap();
            assert_eq!(values.len(), 1);
            assert_eq!(
                CraftMessage::decode(forwarder, &bytes)
                    .expect("a craft answer")
                    .unwrap(),
                message
            );
        }
    }

    /// The single forms: one singleton reference, the record's DID.
    #[test]
    fn add_skill_and_recipe_pack_one_singleton() {
        for (message, function) in [
            (
                CraftMessage::AddCraftSkill { skill: SKILL },
                function::CRAFT_ADD_SKILL,
            ),
            (
                CraftMessage::AddRecipe { recipe: RECIPE },
                function::CRAFT_ADD_RECIPE,
            ),
        ] {
            assert_eq!(message.function(), function);
            let bytes = pack_values(&message.arguments());
            let decoded = CraftMessage::decode(function, &bytes)
                .expect("a craft answer")
                .unwrap();
            assert_eq!(decoded, message);
            let mut values = message.arguments();
            values.push(PackValue::Int(1));
            assert!(CraftMessage::decode(function, &pack_values(&values))
                .unwrap()
                .is_err());
        }
    }

    /// The list forms: an `RList` root whose items name the singleton entries after it, in
    /// order; the decoder reads them back in that order.
    #[test]
    fn add_skills_and_recipes_pack_an_rlist_of_singletons() {
        let skills = vec![SKILL, SKILL + 1];
        for (message, function) in [
            (
                CraftMessage::AddCraftSkills {
                    skills: skills.clone(),
                },
                function::CRAFT_ADD_SKILLS,
            ),
            (
                CraftMessage::AddRecipes {
                    recipes: vec![RECIPE, RECIPE + 1],
                },
                function::CRAFT_ADD_RECIPES,
            ),
        ] {
            assert_eq!(message.function(), function);
            let bytes = pack_values(&message.arguments());
            let decoded = CraftMessage::decode(function, &bytes)
                .expect("a craft answer")
                .unwrap();
            assert_eq!(decoded, message);
        }
        // The empty list is a root with no items.
        let empty = CraftMessage::AddRecipes {
            recipes: Vec::new(),
        };
        let bytes = pack_values(&empty.arguments());
        assert_eq!(
            CraftMessage::decode(function::CRAFT_ADD_RECIPES, &bytes)
                .unwrap()
                .unwrap(),
            empty
        );
    }
}
