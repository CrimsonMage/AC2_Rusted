//! The crafting messages (`WM_Craft`, package 0x206): the server's answers to a craft and the
//! client's requests. The answers are forwarders the server reaches (`WM_Craft` 0x16 ->
//! `CPlayer` 0x30 -> `CM_Craft::SendNotice_AddIngredient_Done`; 0x1C -> `CPlayer` 0xF7 ->
//! `CM_Craft::SendNotice_ExecuteCraft_Done`), so they list their values last parameter first, as the
//! store and trade messages do. The requests are the stubs the IFuns call
//! (`gmCWInterface::RequestAddIngredient` -> 0x06, `RequestExecuteCraft` -> 0x12,
//! `RequestRaiseCraftSkill` -> 0x2E); their values are in the stub's pack order, which
//! [`ScriptRequest`](super::ScriptRequest) keeps.

use super::{function, Args};
use crate::protocol::native::{NativeObject, ScriptHash};
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
}

impl CraftMessage {
    /// The function id the message is sent to.
    pub fn function(&self) -> u32 {
        match self {
            Self::AddIngredientDone { .. } => function::CRAFT_ADD_INGREDIENT_DONE,
            Self::ExecuteCraftDone { .. } => function::CRAFT_EXECUTE_DONE,
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
        }
    }

    /// The server's answer from its function id and packed arguments, or `None` for another id.
    pub fn decode(function_id: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        let decode = |read: fn(&mut Args) -> Result<Self, ProtocolError>| {
            let mut args = Args::new(arguments)?;
            let message = read(&mut args)?;
            args.finish()?;
            Ok(message)
        };
        Some(match function_id {
            function::CRAFT_ADD_INGREDIENT_DONE => decode(|a| {
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
            function::CRAFT_EXECUTE_DONE => decode(|a| {
                let notify = a.int()?;
                let error = a.int()?;
                Ok(Self::ExecuteCraftDone {
                    notify,
                    error,
                    recipe: a.int()?,
                })
            }),
            _ => return None,
        })
    }
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
            vec![PackValue::Int(1), PackValue::Int(0), PackValue::Int(RECIPE),]
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
        let event = request.to_event();
        let ClientEvent::Interp {
            function,
            arguments,
            ..
        } = event
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
        let request = ScriptRequest::CraftRaiseSkill(0x6F00_23A8);
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
            vec![PackValue::Int(0x6F00_23A8)]
        );
    }
}
