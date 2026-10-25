//! The craft registry and the records it holds, as a server packs them for the registry messages
//! (`WM_Craft` 0x19, 0x27, 0x34, 0x1A, 0x37). They are per-player heap objects, not the DAT WStates:
//! a `CraftSkillRecord` (package 0x345F, six slots) names its `CraftSkill` WState by DID, and a
//! `RecipeRecord` (package 0x1D9, twelve slots) names its `Recipe` WState by DID. The registry keys
//! each by that DID (`CraftSkillRecord` 0x4 reads slot 2, `RecipeRecord` 0x15 reads slot 7).
//!
//! The other members are packed as the records' constructors leave them (`CraftSkillRecord` 0x5,
//! `RecipeRecord` 0x11); what the server put there for a player's progress is not traced.

use super::Args;
use crate::protocol::native::{HashKey, NativeObject, ScriptHash, ScriptList};
use crate::protocol::pack::{entry_flags, HeapObject, PackValue, PackedEntry, PackedReference};
use crate::protocol::ProtocolError;

/// `CraftSkillRecord`'s package id.
const CRAFT_SKILL_RECORD: u32 = 0x345F;
/// `RecipeRecord`'s package id.
const RECIPE_RECORD: u32 = 0x1D9;
/// The slot each record keeps its WState DID in.
const SKILL_DID_SLOT: usize = 2;
const RECIPE_DID_SLOT: usize = 7;
/// `R_LIST`: the native type of a list of records (`gRList`).
const R_LIST: u32 = 0x65;
/// The reference id of a message's root value.
const ROOT_ID: u32 = 1;
/// A heap entry as the server packs it.
const HEAP_FLAGS: u32 = entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE;

/// The VM's slot tags (`tag::REF`, `tag::INT`, `tag::FLOAT`, `tag::DOUBLE`, `tag::LONG_INT`,
/// `tag::TIME`). A 64-bit store tags its first slot only; the second keeps the uninitialised
/// fill (`tag::UNINIT`), as the constructors leave it.
const REF: u8 = 0x01;
const INT: u8 = 0x02;
const FLOAT: u8 = 0x03;
const DOUBLE: u8 = 0x83;
const LONG_INT: u8 = 0x82;
const TIME: u8 = 0xC3;
const UNINIT: u8 = 0xCD;
/// `CraftRegistry`'s package id.
const CRAFT_REGISTRY: u32 = 0x1D2;
/// `ARHash`: the native type of the registry's hashes (`gARHash`), and its default bucket index
/// (`NEWNAT` makes the default table).
const AR_HASH: u32 = 0x11;
const DEFAULT_BUCKET_INDEX: u8 = 2;

/// The kind of record a registry message carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Record {
    CraftSkill,
    Recipe,
}

impl Record {
    /// The record for WState `did`, as its constructor leaves it with the DID set.
    fn object(self, did: u32) -> HeapObject {
        let (pkg_id, slots, tags) = match self {
            // `CraftSkillRecord` 0x5: slots 2 and 5 zero, the longs at 0 and 3 zero.
            Self::CraftSkill => (
                CRAFT_SKILL_RECORD,
                vec![0, 0, did, 0, 0, 0],
                vec![LONG_INT, UNINIT, INT, LONG_INT, UNINIT, INT],
            ),
            // `RecipeRecord` 0x11: ints 0, 3, 6, 10 zero, 11 -1, the times at 1, 4 and 8 -1.0.
            Self::Recipe => {
                let (lo, hi) = split((-1.0f64).to_bits());
                (
                    RECIPE_RECORD,
                    vec![0, lo, hi, 0, lo, hi, 0, did, lo, hi, 0, u32::MAX],
                    vec![
                        INT, TIME, UNINIT, INT, TIME, UNINIT, INT, INT, TIME, UNINIT, INT, INT,
                    ],
                )
            }
        };
        HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native: None,
            slots,
            tags,
        }
    }

    fn package(self) -> u32 {
        match self {
            Self::CraftSkill => CRAFT_SKILL_RECORD,
            Self::Recipe => RECIPE_RECORD,
        }
    }

    fn did_slot(self) -> usize {
        match self {
            Self::CraftSkill => SKILL_DID_SLOT,
            Self::Recipe => RECIPE_DID_SLOT,
        }
    }

    /// One record as a message's reference: the record is the root.
    pub(super) fn single(self, did: u32) -> PackValue {
        PackValue::Reference(PackedReference {
            ids: vec![ROOT_ID],
            entries: vec![PackedEntry::Heap {
                flags: HEAP_FLAGS,
                object: self.object(did),
            }],
        })
    }

    /// Records as a message's reference: the root is an `RList` whose items are the records'
    /// reference ids, the records after it, as `StoreView`'s sales are laid out.
    pub(super) fn list(self, dids: &[u32]) -> PackValue {
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
        ids.extend(&item_ids);
        let mut entries = vec![PackedEntry::Heap {
            flags: HEAP_FLAGS,
            object: list,
        }];
        entries.extend(dids.iter().map(|did| PackedEntry::Heap {
            flags: HEAP_FLAGS,
            object: self.object(*did),
        }));
        PackValue::Reference(PackedReference { ids, entries })
    }

    /// The DID of the one record an answer's value is.
    pub(super) fn read_single(self, args: &mut Args) -> Result<u32, ProtocolError> {
        let reference = args.reference()?;
        match reference.entries.as_slice() {
            [PackedEntry::Heap { object, .. }] => self.did_of(object),
            _ => Err(ProtocolError::Invalid("craft record")),
        }
    }

    /// The DIDs of the records in an answer's `RList`.
    pub(super) fn read_list(self, args: &mut Args) -> Result<Vec<u32>, ProtocolError> {
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
                    .position(|id| u64::from(*id) == *item)
                    .ok_or_else(invalid)?;
                match reference.entries.get(index) {
                    Some(PackedEntry::Heap { object, .. }) => self.did_of(object),
                    _ => Err(invalid()),
                }
            })
            .collect()
    }

    fn did_of(self, object: &HeapObject) -> Result<u32, ProtocolError> {
        if object.pkg_id != self.package() {
            return Err(ProtocolError::Invalid("craft record package"));
        }
        object
            .slots
            .get(self.did_slot())
            .copied()
            .ok_or(ProtocolError::Invalid("craft record"))
    }
}

/// The registry a server hands the player (`WM_Craft` 0x19): a `CraftRegistry` as its constructor
/// (`CraftRegistry` 0x11) leaves it, with the skill hash (slot 5) and the recipe hash (slot 6)
/// holding `skills` and `recipes` keyed by DID, each value a record after it.
pub(super) fn registry(skills: &[u32], recipes: &[u32]) -> PackValue {
    const SKILLS_ID: u32 = ROOT_ID + 1;
    const RECIPES_ID: u32 = ROOT_ID + 2;
    let first_record = RECIPES_ID + 1;
    let skill_ids: Vec<u32> = (0..skills.len() as u32).map(|i| first_record + i).collect();
    let recipe_ids: Vec<u32> = (0..recipes.len() as u32)
        .map(|i| first_record + skills.len() as u32 + i)
        .collect();
    let root = HeapObject {
        pkg_state_id: 0,
        pkg_id: CRAFT_REGISTRY,
        native: None,
        slots: vec![u32::MAX, 0, 0, 0, 0, SKILLS_ID, RECIPES_ID],
        tags: vec![REF, FLOAT, INT, DOUBLE, UNINIT, REF, REF],
    };
    let hash = |dids: &[u32], ids: &[u32]| HeapObject {
        pkg_state_id: 0,
        pkg_id: u32::MAX,
        native: Some(NativeObject::Hash(ScriptHash {
            native_type: AR_HASH,
            bucket_index: DEFAULT_BUCKET_INDEX,
            keys: dids
                .iter()
                .map(|did| HashKey::Value(u64::from(*did)))
                .collect(),
            values: ids.iter().map(|id| u64::from(*id)).collect(),
        })),
        slots: Vec::new(),
        tags: Vec::new(),
    };
    let heap = |object| PackedEntry::Heap {
        flags: HEAP_FLAGS,
        object,
    };
    let mut ids = vec![ROOT_ID, SKILLS_ID, RECIPES_ID];
    ids.extend(&skill_ids);
    ids.extend(&recipe_ids);
    let mut entries = vec![
        heap(root),
        heap(hash(skills, &skill_ids)),
        heap(hash(recipes, &recipe_ids)),
    ];
    entries.extend(
        skills
            .iter()
            .map(|did| heap(Record::CraftSkill.object(*did))),
    );
    entries.extend(recipes.iter().map(|did| heap(Record::Recipe.object(*did))));
    PackValue::Reference(PackedReference { ids, entries })
}

/// The skill and recipe DIDs of a registry a server sent (the inverse of [`registry`]).
pub(super) fn read_registry(args: &mut Args) -> Result<(Vec<u32>, Vec<u32>), ProtocolError> {
    let reference = args.reference()?;
    let invalid = || ProtocolError::Invalid("craft registry");
    let entry = |id: u32| {
        let index = reference.ids.iter().position(|i| *i == id)?;
        match reference.entries.get(index)? {
            PackedEntry::Heap { object, .. } => Some(object),
            _ => None,
        }
    };
    let root = entry(ROOT_ID).ok_or_else(invalid)?;
    if root.pkg_id != CRAFT_REGISTRY || root.slots.len() != 7 {
        return Err(invalid());
    }
    let dids = |slot: usize, record: Record| -> Result<Vec<u32>, ProtocolError> {
        let Some(NativeObject::Hash(hash)) =
            entry(root.slots[slot]).and_then(|o| o.native.as_ref())
        else {
            return Err(invalid());
        };
        hash.values
            .iter()
            .map(|id| record.did_of(entry(*id as u32).ok_or_else(invalid)?))
            .collect()
    };
    Ok((dids(5, Record::CraftSkill)?, dids(6, Record::Recipe)?))
}

fn split(bits: u64) -> (u32, u32) {
    (bits as u32, (bits >> 32) as u32)
}
