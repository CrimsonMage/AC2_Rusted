//! The double's answers to the fellowship actions and the attack buttons, as a server's
//! `WM_Fellowship` / `WM_Combat` handlers would send them: the `Fellowship` object (package
//! 0x1F0) the client's `FellowshipSystem` keeps, its fellows (`Fellow` 0x1EE with their
//! `FellowVitals` 0x1EF), the clears, and the attack state.

use ac2_net::protocol::{
    entry_flags, ntype, HashKey, HeapObject, NativeObject, PackValue, PackedEntry, PackedReference,
    ScriptCall, ScriptHash, StringInfo,
};

use crate::State;

/// The second member the double's fellowships get. Its id shares the player's bucket in the
/// fellows table `FellowshipAdaptor` keeps (47 buckets) and comes after it in the
/// `Fellowship`'s own (11 buckets: 6 after 3), so it is added later, goes to the chain's head
/// and the UI lists it before the leader: the panel's leader-first order shows.
pub const FELLOW: u64 = 0x2000_0000_0000_0030;
/// The fellowship's chat room (`Fellowship` slot 3).
pub const FELLOWSHIP_ROOM: u32 = 0x0F00_0042;

/// One member as `FellowshipSystem` keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fellow {
    pub id: u64,
    pub name: String,
    pub level: u32,
    pub health: (u32, u32),
    pub vigor: (u32, u32),
    pub cell: u32,
}

/// The fellowship the double made for the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fellowship {
    pub name: String,
    /// `Fellowship` slot 2: bit 1 set for a non-social fellowship, the looting method's bit
    /// (`1 << method`, `Fellowship` 0x2E).
    pub flags: u32,
    pub leader: u64,
    /// In the hash's order (the panel puts the leader first itself).
    pub fellows: Vec<Fellow>,
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

fn split(id: u64) -> [u32; 2] {
    [id as u32, (id >> 32) as u32]
}

/// A `Fellow` and its vitals and name, as entries `at`, `at + 1`, `at + 2`.
fn fellow_entries(f: &Fellow, at: u32) -> Vec<PackedEntry> {
    let name = StringInfo {
        literal: Some(f.name.clone()),
        ..StringInfo::default()
    };
    vec![
        heap(
            0x1EE,
            None,
            vec![0, 0, f.level, at + 1, at + 2],
            vec![0xC3, 0xCD, 2, 1, 1],
        ),
        // `FellowVitals`: max health, cell, health, PK damage, max vigor, vigor, PK vigor loss.
        heap(
            0x1EF,
            None,
            vec![f.health.1, f.cell, f.health.0, 0, f.vigor.1, f.vigor.0, 0],
            vec![2; 7],
        ),
        heap(
            u32::MAX,
            Some(NativeObject::StringInfo(name)),
            Vec::new(),
            Vec::new(),
        ),
    ]
}

impl Fellowship {
    /// The `Fellowship` graph `WM_Fellowship` 0x2B carries.
    pub fn graph(&self) -> PackedReference {
        let firsts: Vec<u32> = (0..self.fellows.len() as u32).map(|i| 4 + 3 * i).collect();
        let [leader_lo, leader_hi] = split(self.leader);
        let mut entries = vec![
            heap(
                0x1F0,
                None,
                vec![
                    0,
                    0,
                    self.flags,
                    FELLOWSHIP_ROOM,
                    2,
                    leader_lo,
                    leader_hi,
                    3,
                ],
                vec![0x82, 0xCD, 2, 2, 1, 0x82, 0xCD, 1],
            ),
            heap(
                u32::MAX,
                Some(NativeObject::Hash(ScriptHash {
                    native_type: ntype::LR_HASH,
                    bucket_index: 0,
                    keys: self.fellows.iter().map(|f| HashKey::Value(f.id)).collect(),
                    values: firsts.iter().map(|i| u64::from(*i)).collect(),
                })),
                Vec::new(),
                Vec::new(),
            ),
            heap(
                u32::MAX,
                Some(NativeObject::WString(self.name.clone())),
                Vec::new(),
                Vec::new(),
            ),
        ];
        for (f, at) in self.fellows.iter().zip(&firsts) {
            entries.extend(fellow_entries(f, *at));
        }
        let mut ids = vec![1, 2, 3];
        for at in &firsts {
            ids.extend([*at, at + 1, at + 2]);
        }
        PackedReference { ids, entries }
    }
}

/// One `Fellow` graph (`WM_Fellowship` 0x27's first argument).
pub fn fellow_graph(f: &Fellow) -> PackedReference {
    PackedReference {
        ids: vec![1, 2, 3],
        entries: fellow_entries(f, 1),
    }
}

impl State {
    fn member(&self, id: u64, name: &str, level: u32) -> Fellow {
        Fellow {
            id,
            name: name.into(),
            level,
            health: (40, 50),
            vigor: (20, 30),
            cell: self.spawn.cell,
        }
    }

    /// A client call the double answers: the fellowship actions (`WM_Fellowship` 0x22 create,
    /// 0x2A recruit, 0x23 dismiss, 0x29 leave, 0x2C disband) and the attack buttons
    /// (`WM_Combat` 0x16 / 0x14 -> 0x1B, `CPlayer` 0x2's attack state).
    pub(crate) fn answer_call(&mut self, call: &ScriptCall) {
        match (call.function, &call.arguments[..]) {
            (0x820C_001B, [PackValue::Int(looting), PackValue::Int(social), name]) => {
                let name = match name.as_native() {
                    Some(NativeObject::WString(name)) => name.clone(),
                    _ => String::new(),
                };
                let fellowship = Fellowship {
                    name,
                    flags: u32::from(*social == 0) | (1 << (looting & 0xF)),
                    leader: self.player(),
                    fellows: vec![
                        self.member(FELLOW, "Birch", 4),
                        self.member(self.player(), "Explorer", 1),
                    ],
                };
                let graph = fellowship.graph();
                self.fellowship = Some(fellowship);
                self.send_script_call(0x820C_002B, &[PackValue::Reference(graph)]);
            }
            (0x820C_0015, [PackValue::LongInt(target)]) => {
                let Some(fellowship) = self.fellowship.as_ref() else {
                    return;
                };
                if fellowship.fellows.iter().any(|f| f.id == *target) {
                    return;
                }
                let fellow = self.member(*target, "Recruit", 3);
                let graph = fellow_graph(&fellow);
                if let Some(f) = self.fellowship.as_mut() {
                    f.fellows.push(fellow);
                }
                self.send_script_call(
                    0x820C_0027,
                    &[PackValue::Reference(graph), PackValue::LongInt(*target)],
                );
            }
            (0x820C_000E, [PackValue::LongInt(target)]) => {
                if let Some(f) = self.fellowship.as_mut() {
                    f.fellows.retain(|f| f.id != *target);
                    self.send_script_call(0x820C_0026, &[PackValue::LongInt(*target)]);
                }
            }
            (0x820C_001C | 0x820C_0003, []) => {
                if self.fellowship.take().is_some() {
                    self.send_script_call(0x820C_0028, &[]);
                }
            }
            (0x8204_0008, [_]) => self.send_script_call(0x8204_001B, &[PackValue::Int(1)]),
            (0x8204_0010, []) => self.send_script_call(0x8204_001B, &[PackValue::Int(0)]),
            (0x8214_0001 | 0x8214_0002 | 0x8214_0007 | 0x8214_000C, args) => {
                self.answer_skill(call.function, args)
            }
            (0x820F_0001 | 0x820F_0002, args) => self.answer_money(call.function, args),
            (function, args) => self.answer_allegiance(function, args),
        }
    }
}
