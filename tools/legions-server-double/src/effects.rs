//! The double's effects on the player: the `EffectRegistry` (package 0x14F) `WM_Player` 0x27
//! carries as argument 6 (`CPlayer` 0x65 -> `CEffectSystem` 0x2 -> `gmEntity` 0xFA, the
//! entity's slot 11), which `CEffectSystem` 0x7 walks for `gmCWInterface::InqEffectList`.
//!
//! A cast adds an effect with `WM_Effect` 0x17 and its end removes it with 0x16 (`add_effect`,
//! `remove_effect`); the shipped wlib's server-side effect functions are stubs, so which skill
//! applies which effect, for how long and under which id are the double's own choices.
//!
//! The registry: slot 4 an `ARHash` effect id -> `EffectRecord`, slot 11 an `AAHash` order ->
//! effect id (`EffectRegistry` 0x4A visits the orders from the highest down), slot 2 the
//! parent registry (null). An `EffectRecord` (package 0x14E): slots 2-3 the time it was cast
//! (a double, `EffectRecord` 0x2), slot 6 the duration (a float, 0x23), slot 14 the `Effect`
//! (0x1D; a package singleton by `WState` DID, whose name, description and icon the strip
//! shows), slot 20 the flags (0x2 / 0x4 timed, 0x200 toggled, 0x400 pulsed, 0x800 hidden).

use std::collections::BTreeMap;

use ac2_net::protocol::{
    entry_flags, ntype, HashKey, HeapObject, NativeObject, PackValue, PackedEntry, PackedReference,
    ScriptHash,
};

/// `WM_Effect` 0x17 -> `CPlayer` 0x6B -> `CEffectSystem` 0x6 (`EffectRegistry` 0x3A): an effect
/// is added to the player's registry. Its values pack the `EffectRecord` first, then the effect
/// id (the forwarder's deepest argument is the first packed value).
pub const ADD_EFFECT: u32 = 0x8208_0017;
/// `WM_Effect` 0x16 -> `CPlayer` 0x133 -> `CEffectSystem` 0x3 (`EffectRegistry` 0x39): the effect
/// with this id (one int) leaves the registry. Both raise `SendNotice_UpdateEffect` for the
/// player, which re-lists the strip (`gmEffectsUI::Init`).
pub const REMOVE_EFFECT: u32 = 0x8208_0016;

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

/// The `EffectRecord` `WM_Effect` 0x17 carries for `effect`: the record, then the `Effect` it
/// names (a package singleton), the record's slot 14 referring to it.
pub fn record_graph(effect: &EffectGrant) -> PackedReference {
    let entries = record_entries(effect, 1);
    PackedReference {
        ids: (1..=entries.len() as u32).collect(),
        entries,
    }
}

/// The record's heap entry and the `Effect`'s singleton, the record being entry `record`
/// (1-based) of its graph.
fn record_entries(e: &EffectGrant, record: u32) -> Vec<PackedEntry> {
    let time = e.time_cast.to_bits();
    // `PackageInfo::size` of `EffectRecord`: 26 slots (slots 24 and 25 are status words that
    // `EffectRegistry` 0x3A ORs into when it adds the record).
    let mut slots = vec![0; 26];
    let mut tags = vec![2; 26];
    (slots[2], slots[3]) = (time as u32, (time >> 32) as u32);
    (tags[2], tags[3]) = (0xC3, 0xCD);
    (slots[6], tags[6]) = (e.duration.to_bits(), 3);
    (slots[14], tags[14]) = (record + 1, 1);
    slots[20] = e.flags;
    vec![
        heap(RECORD_PACKAGE, None, slots, tags),
        PackedEntry::Singleton {
            flags: entry_flags::KIND_HEAP | entry_flags::SINGLETON,
            did: e.effect,
        },
    ]
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
        entries.extend(record_entries(e, record_at(i)));
    }
    PackedReference {
        ids: (1..=entries.len() as u32).collect(),
        entries,
    }
}

/// A skill that puts an effect on the player: the `Effect`'s `WState` DID and how long it lasts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkillEffect {
    pub effect: u32,
    pub duration: f32,
}

/// The effects the double puts on the player and takes off again.
#[derive(Debug, Clone, Default)]
pub struct Effects {
    /// What each self-buff skill applies (skill id -> effect); the double's table.
    pub skill_effects: BTreeMap<u32, SkillEffect>,
    /// The id the next effect gets.
    pub next_eid: u32,
    /// The effects that ran out, in order.
    pub expired: Vec<u32>,
}

impl crate::State {
    /// The effects the next `WM_Player` 0x27 carries.
    pub fn give_effects(&mut self, effects: Vec<EffectGrant>) {
        self.script_world.effects = effects;
    }

    /// The server's clock as the client's `Timer::cur_time` reads it: the connect's server time
    /// (5000) plus the time since the connect.
    pub fn server_time(&self) -> f64 {
        crate::SERVER_TIME + self.elapsed - self.connected_at
    }

    /// Sets what `skill` applies to the player when it is used.
    pub fn set_skill_effect(&mut self, skill: u32, effect: u32, duration: f32) {
        self.effects
            .skill_effects
            .insert(skill, SkillEffect { effect, duration });
    }

    /// Puts `effect` on the player: it joins the registry later logins carry and goes out as
    /// `WM_Effect` 0x17 (the record, then the effect id).
    pub fn add_effect(&mut self, effect: EffectGrant) {
        let message = [
            PackValue::Reference(record_graph(&effect)),
            PackValue::Int(effect.eid),
        ];
        self.script_world.effects.push(effect);
        self.send_script_call(ADD_EFFECT, &message);
    }

    /// Takes effect `eid` off the player (`WM_Effect` 0x16).
    pub fn remove_effect(&mut self, eid: u32) {
        self.script_world.effects.retain(|e| e.eid != eid);
        self.send_script_call(REMOVE_EFFECT, &[PackValue::Int(eid)]);
    }

    /// The player used `skill`: when it applies an effect, the effect goes on (timed, cast now)
    /// and its id is returned.
    pub(crate) fn apply_skill_effect(&mut self, skill: u32) -> Option<u32> {
        let SkillEffect { effect, duration } = self.effects.skill_effects.get(&skill).copied()?;
        self.effects.next_eid += 1;
        let eid = 0x100 + self.effects.next_eid;
        let time_cast = self.server_time();
        self.add_effect(EffectGrant {
            eid,
            effect,
            time_cast,
            duration,
            flags: EFFECT_TIMED,
        });
        Some(eid)
    }

    /// Timed effects whose time ran out leave the player.
    pub(crate) fn effects_tick(&mut self) {
        let now = self.server_time();
        let ended: Vec<u32> = self
            .script_world
            .effects
            .iter()
            .filter(|e| e.flags & EFFECT_TIMED != 0 && now >= e.time_cast + f64::from(e.duration))
            .map(|e| e.eid)
            .collect();
        for eid in ended {
            self.effects.expired.push(eid);
            self.remove_effect(eid);
        }
    }
}
