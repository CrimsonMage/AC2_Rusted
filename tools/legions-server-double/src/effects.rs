//! The double's effects on the player: the `EffectRegistry` (package 0x14F) `WM_Player` 0x27
//! carries as argument 6 (`CPlayer` 0x65 -> `CEffectSystem` 0x2 -> `gmEntity` 0xFA, the
//! entity's slot 11), which `CEffectSystem` 0x7 walks for `gmCWInterface::InqEffectList`.
//!
//! The registry: slot 4 an `ARHash` effect id -> `EffectRecord`, slot 11 an `AAHash` order ->
//! effect id (`EffectRegistry` 0x4A visits the orders from the highest down), slot 2 the
//! parent registry (null). An `EffectRecord` (package 0x14E): slots 2-3 the time it was cast
//! (a double, `EffectRecord` 0x2), slot 6 the duration (a float, 0x23), slot 14 the `Effect`
//! (0x1D; a package singleton by `WState` DID, whose name, description and icon the strip
//! shows), slot 20 the flags (0x2 / 0x4 timed, 0x200 toggled, 0x400 pulsed, 0x800 hidden).

use ac2_net::protocol::{
    entry_flags, ntype, HashKey, HeapObject, NativeObject, PackedEntry, PackedReference, ScriptHash,
};

/// `EffectRecord` slot 20 bits.
pub const EFFECT_TIMED: u32 = 0x2;
pub const EFFECT_PULSED: u32 = 0x400;

const REGISTRY_PACKAGE: u32 = 0x14F;
const RECORD_PACKAGE: u32 = 0x14E;
const NULL: u32 = u32::MAX;

/// One effect on the player.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectGrant {
    pub eid: u32,
    /// The `Effect`'s `WState` DID.
    pub effect: u32,
    /// When it was cast (server time) and for how long; used when `flags` has
    /// [`EFFECT_TIMED`].
    pub time_cast: f64,
    pub duration: f32,
    pub flags: u32,
}

fn heap(pkg_id: u32, native: Option<NativeObject>, slots: Vec<u32>, tags: Vec<u8>) -> PackedEntry {
    PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native,
            slots,
            tags,
        },
    }
}

fn hash(native_type: u32, keys: Vec<u64>, values: Vec<u64>) -> NativeObject {
    NativeObject::Hash(ScriptHash {
        native_type,
        bucket_index: 0,
        keys: keys.into_iter().map(HashKey::Value).collect(),
        values,
    })
}

/// The registry graph for `effects`, the first the highest in the strip's order.
pub fn effect_registry(effects: &[EffectGrant]) -> PackedReference {
    // Entries: 1 the registry, 2 its record hash, 3 its order hash, then per effect its
    // record and its `Effect` singleton.
    let record_at = |i: usize| 4 + 2 * i as u32;
    let mut entries = Vec::new();
    let mut slots = vec![NULL; 13];
    slots[4] = 2;
    slots[11] = 3;
    entries.push(heap(REGISTRY_PACKAGE, None, slots, vec![1; 13]));
    let eids: Vec<u64> = effects.iter().map(|e| u64::from(e.eid)).collect();
    let records = (0..effects.len())
        .map(|i| u64::from(record_at(i)))
        .collect();
    entries.push(heap(
        u32::MAX,
        Some(hash(ntype::AR_HASH, eids.clone(), records)),
        Vec::new(),
        Vec::new(),
    ));
    let orders = (0..effects.len())
        .map(|i| (effects.len() - i) as u64)
        .collect();
    entries.push(heap(
        u32::MAX,
        Some(hash(ntype::AA_HASH, orders, eids)),
        Vec::new(),
        Vec::new(),
    ));
    for (i, e) in effects.iter().enumerate() {
        let time = e.time_cast.to_bits();
        let mut slots = vec![0; 21];
        let mut tags = vec![2; 21];
        (slots[2], slots[3]) = (time as u32, (time >> 32) as u32);
        (tags[2], tags[3]) = (0xC3, 0xCD);
        (slots[6], tags[6]) = (e.duration.to_bits(), 3);
        (slots[14], tags[14]) = (record_at(i) + 1, 1);
        slots[20] = e.flags;
        entries.push(heap(RECORD_PACKAGE, None, slots, tags));
        entries.push(PackedEntry::Singleton {
            flags: entry_flags::KIND_HEAP | entry_flags::SINGLETON,
            did: e.effect,
        });
    }
    PackedReference {
        ids: (1..=entries.len() as u32).collect(),
        entries,
    }
}

impl crate::State {
    /// The effects the next `WM_Player` 0x27 carries.
    pub fn give_effects(&mut self, effects: Vec<EffectGrant>) {
        self.script_world.effects = effects;
    }
}
