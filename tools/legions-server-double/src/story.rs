//! The double's story state: the `ActRegistry` (package 0x1A4) `WM_Player` 0x27 carries as
//! argument 2 (`CPlayer` 0x65 keeps it as the player's registry).

use ac2_net::protocol::{
    entry_flags, ntype, HashKey, HeapObject, NativeObject, PackedEntry, PackedReference, ScriptHash,
};

/// `WM_Quest` 0x23 -> `CPlayer` 0x2A -> `CQuestSystem::UpdateStoryQuest_Done`: `(errorCode,
/// sceneId, bAdd)`.
pub const UPDATE_STORY_QUEST: u32 = 0x8213_0023;

const REGISTRY_PACKAGE: u32 = 0x1A4;

/// An empty registry, the shape of the shipped singleton `700001a4`: slot 0 an int, slot 1 the
/// act hash.
pub fn act_registry() -> PackedReference {
    let registry = PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id: REGISTRY_PACKAGE,
            native: None,
            slots: vec![0, 2],
            tags: vec![2, 1],
        },
    };
    let acts = PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id: u32::MAX,
            native: Some(NativeObject::Hash(ScriptHash {
                native_type: ntype::AR_HASH,
                bucket_index: 0,
                keys: Vec::<HashKey>::new(),
                values: Vec::new(),
            })),
            slots: Vec::new(),
            tags: Vec::new(),
        },
    };
    PackedReference {
        ids: vec![1, 2],
        entries: vec![registry, acts],
    }
}
