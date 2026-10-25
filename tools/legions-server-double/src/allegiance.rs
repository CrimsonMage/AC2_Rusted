//! The double's answers to the allegiance actions and its invitations, as a server's
//! `WM_Allegiance` / `WM_Fellowship` handlers would send them: the `AllegianceProfile`
//! object (package 0x1B5) with its `AllegianceData` members (0x1B1), the swear results and
//! the requests that ask the player (to accept a vassal, to join a fellowship).

use ac2_net::protocol::{
    entry_flags, ntype, HashKey, HeapObject, NativeObject, PackValue, PackedEntry, PackedReference,
    ScriptHash, ScriptList, StringInfo,
};

use crate::social::{Fellow, Fellowship};
use crate::{State, PLAYER};

/// A null reference slot.
const NULL: u32 = u32::MAX;

/// One member of the player's allegiance, with what `AllegianceDataAdaptor` carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Liege {
    pub id: u64,
    pub name: String,
    pub level: u32,
    /// The race (`m_species`, enum mapper 0x41000009: 1 Human).
    pub species: u32,
    /// The gender (enum mapper 0x4100000B: 0x1000 Male, 0x2000 Female); with the rank and
    /// the race it keys the rank title (`AllegianceRankTable` 0x1).
    pub gender: u32,
    /// Experience passed up to the patron.
    pub xp_passed: u64,
    /// A custom title; empty for the rank's.
    pub title: String,
    pub officer: bool,
}

impl Liege {
    /// A Human male member with the rank's title, passing nothing up.
    pub fn new(id: u64, name: &str, level: u32) -> Self {
        Self {
            id,
            name: name.into(),
            level,
            species: 1,
            gender: 0x1000,
            xp_passed: 0,
            title: String::new(),
            officer: false,
        }
    }
}

/// The player's allegiance as the double keeps it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Allegiance {
    /// The patron (also the monarch: the double's allegiances are two levels deep).
    pub patron: Option<Liege>,
    pub vassals: Vec<Liege>,
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

fn literal(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.into()),
        ..StringInfo::default()
    }
}

fn native(object: NativeObject) -> PackedEntry {
    heap(u32::MAX, Some(object), Vec::new(), Vec::new())
}

/// Builds a graph entry by entry; `add` returns the new entry's id.
#[derive(Default)]
struct Graph {
    ids: Vec<u32>,
    entries: Vec<PackedEntry>,
}

impl Graph {
    fn add(&mut self, entry: PackedEntry) -> u32 {
        let id = self.ids.len() as u32 + 1;
        self.ids.push(id);
        self.entries.push(entry);
        id
    }

    /// An `AllegianceData` (`AllegianceData` 0x1: XP passed in slots 0-1, species 2, level
    /// 3, a custom title 4 (a `wpstring`; null: `AllegianceControl` 0x8's title for the rank,
    /// gender and species), gender 5, officer 8, the id 9-10, rank 11, faction 12, the name
    /// 13).
    fn member(&mut self, who: &Liege, rank: u32) -> u32 {
        let at = self.add(heap(0x1B1, None, Vec::new(), Vec::new()));
        let name = self.add(native(NativeObject::StringInfo(literal(&who.name))));
        let title = if who.title.is_empty() {
            NULL
        } else {
            self.add(native(NativeObject::WString(who.title.clone())))
        };
        let slots = vec![
            who.xp_passed as u32,
            (who.xp_passed >> 32) as u32,
            who.species,
            who.level,
            title,
            who.gender,
            0,
            0,
            u32::from(who.officer),
            who.id as u32,
            (who.id >> 32) as u32,
            rank,
            0,
            name,
        ];
        let tags = vec![0x82, 0xCD, 2, 2, 1, 2, 0x82, 0xCD, 2, 0x82, 0xCD, 2, 2, 1];
        if let PackedEntry::Heap { object, .. } = &mut self.entries[at as usize - 1] {
            (object.slots, object.tags) = (slots, tags);
        }
        at
    }
}

impl Allegiance {
    /// The `AllegianceProfile` (`AllegianceProfile` 0x2's slots: the name 0, patron 1, the
    /// player 2, monarch 3, their online flags 4 / 5 / 9, the vassals 6 and their online
    /// flags 7, the MotD 8, faction 10, total 11, officers 12) `WM_Allegiance` 0x2E carries.
    pub fn profile(&self, player: &Liege) -> PackedReference {
        let mut g = Graph::default();
        let root = g.add(heap(0x1B5, None, Vec::new(), Vec::new()));
        let name = g.add(native(NativeObject::StringInfo(literal(
            "Order of the Double",
        ))));
        let me = g.member(player, 1);
        let patron = self.patron.as_ref().map(|p| g.member(p, 2));
        let vassals: Vec<u32> = self.vassals.iter().map(|v| g.member(v, 0)).collect();
        let list = g.add(native(NativeObject::List(ScriptList {
            native_type: ntype::R_LIST,
            items: vassals.iter().map(|v| u64::from(*v)).collect(),
        })));
        let online = g.add(native(NativeObject::List(ScriptList {
            native_type: ntype::A_LIST,
            items: vec![1; vassals.len()],
        })));
        let motd = g.add(native(NativeObject::StringInfo(literal("Welcome."))));
        let officers = g.add(native(NativeObject::Hash(ScriptHash {
            native_type: ntype::LA_HASH_SET,
            bucket_index: 0,
            keys: Vec::<HashKey>::new(),
            values: Vec::new(),
        })));
        let patron = patron.unwrap_or(NULL);
        let total = 1 + u32::from(patron != NULL) + vassals.len() as u32;
        let slots = vec![
            name,
            patron,
            me,
            patron,
            1,
            u32::from(patron != NULL),
            list,
            online,
            motd,
            u32::from(patron != NULL),
            0,
            total,
            officers,
        ];
        let tags = vec![1, 1, 1, 1, 2, 2, 1, 1, 1, 2, 2, 2, 1];
        if let PackedEntry::Heap { object, .. } = &mut g.entries[root as usize - 1] {
            (object.slots, object.tags) = (slots, tags);
        }
        PackedReference {
            ids: g.ids,
            entries: g.entries,
        }
    }
}

fn info(name: &str) -> PackValue {
    PackValue::native(NativeObject::StringInfo(literal(name)))
}

impl State {
    fn player_liege(&self) -> Liege {
        Liege::new(PLAYER, "Explorer", self.script_world.character.level as u32)
    }

    fn send_profile(&mut self) {
        let profile = self.allegiance.profile(&self.player_liege());
        self.send_script_call(0x8203_002E, &[PackValue::Reference(profile)]);
    }

    /// Someone asks to swear allegiance to the player (`WM_Allegiance` 0x2F ->
    /// `AllegianceSystem` 0x4F: `SendNotice_RequestVassal`, or an automatic decline when the
    /// player does not accept requests).
    pub fn ask_to_swear(&mut self, vassal: u64, name: &str) {
        self.asking_vassal = Some(Liege {
            gender: 0x2000,
            xp_passed: 1500,
            title: "Squire".into(),
            ..Liege::new(vassal, name, 2)
        });
        self.send_script_call(0x8203_002F, &[info(name), PackValue::LongInt(vassal)]);
    }

    /// Someone invites the player into their fellowship (`WM_Fellowship` 0x2E ->
    /// `FellowshipSystem` 0x3F: `SendNotice_RequestRecruitment`, or an automatic decline).
    pub fn invite_to_fellowship(&mut self, recruiter: u64, name: &str, fellowship: &str) {
        self.recruiter = Some((recruiter, name.into(), fellowship.into()));
        self.send_script_call(
            0x820C_002E,
            &[info(fellowship), info(name), PackValue::LongInt(recruiter)],
        );
    }

    /// The allegiance actions and the invitation answers: swear (`WM_Allegiance` 0x39 ->
    /// 0x33 `Vassal_SwearDone`, then the profile 0x2E), break (0x32 -> the profile), accept a
    /// vassal (0x31 -> 0x2D `Patron_SwearDone`, then the profile), decline one (0x3B -> 0x2D
    /// with the context as the result), query (0x36 -> the profile); fellowship accept (0x25
    /// -> the recruiter's fellowship, 0x2B).
    pub(crate) fn answer_allegiance(&mut self, function: u32, args: &[PackValue]) {
        match (function, args) {
            (0x8203_000A, [PackValue::LongInt(patron)]) => {
                let patron = Liege {
                    officer: true,
                    ..Liege::new(*patron, "Birch", 7)
                };
                let call = [
                    PackValue::Int(0),
                    info(&patron.name),
                    PackValue::LongInt(patron.id),
                ];
                self.allegiance.patron = Some(patron);
                self.send_script_call(0x8203_0033, &call);
                self.send_profile();
            }
            (0x8203_0009, [PackValue::LongInt(target)]) => {
                if self
                    .allegiance
                    .patron
                    .as_ref()
                    .is_some_and(|p| p.id == *target)
                {
                    self.allegiance.patron = None;
                }
                self.allegiance.vassals.retain(|v| v.id != *target);
                self.send_profile();
            }
            (0x8203_001C, [PackValue::LongInt(vassal)]) => {
                let Some(asking) = self.asking_vassal.take().filter(|v| v.id == *vassal) else {
                    return;
                };
                let call = [
                    PackValue::Int(0),
                    info(&asking.name),
                    PackValue::LongInt(asking.id),
                ];
                self.allegiance.vassals.push(asking);
                self.send_script_call(0x8203_002D, &call);
                self.send_profile();
            }
            (0x8203_0029, [PackValue::Int(context), PackValue::LongInt(vassal)]) => {
                if let Some(asking) = self.asking_vassal.take().filter(|v| v.id == *vassal) {
                    let call = [
                        PackValue::Int(*context),
                        info(&asking.name),
                        PackValue::LongInt(asking.id),
                    ];
                    self.send_script_call(0x8203_002D, &call);
                }
            }
            (0x8203_0005, [_]) => self.send_profile(),
            (0x820C_000D, [PackValue::LongInt(recruiter)]) => {
                let Some((id, name, fellowship)) =
                    self.recruiter.take().filter(|r| r.0 == *recruiter)
                else {
                    return;
                };
                let member = |id, name: &str, level| Fellow {
                    id,
                    name: name.into(),
                    level,
                    health: (40, 50),
                    vigor: (20, 30),
                    cell: self.spawn.cell,
                };
                let fellowship = Fellowship {
                    name: fellowship,
                    flags: 1 | (1 << 1),
                    leader: id,
                    fellows: vec![member(id, &name, 5), member(PLAYER, "Explorer", 1)],
                };
                let graph = fellowship.graph();
                self.fellowship = Some(fellowship);
                self.send_script_call(0x820C_002B, &[PackValue::Reference(graph)]);
            }
            (0x820C_001A, [_, PackValue::LongInt(_)]) => self.recruiter = None,
            _ => {}
        }
    }
}
