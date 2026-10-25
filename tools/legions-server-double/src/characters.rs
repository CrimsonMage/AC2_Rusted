//! The double's characters: the ones a session creates (`Proto_UI::SendCharGenResult`), kept in
//! a file so a later session lists and enters them (test tooling).
//!
//! The server runs the same `CharacterGenSystem` script the client does, so [`CharGenRules`] is
//! its `IsValid` and `InqAppliedAppearanceVDesc`. A created character is kept as its creation
//! result, its id and the visual description the server made for it (the `VisualDesc` bytes it
//! sends in the login's character set and at entry): the appearance is stored with the character,
//! and a later login sends it as stored.

use std::io;
use std::path::{Path, PathBuf};

use ac2_net::protocol::{CharGenResult, VisualDesc};
use ac2_weenie::Appearance;

use crate::appearance::{appearance_visual, visual_desc_bytes};
use crate::objects::{weenie_object_with, ObjectSpawn};
use crate::{State, CREATED, PLAYER, PLAYER_PACKAGE};

/// The first word of a store file: its magic and format version.
const MAGIC: &[u8; 8] = b"AC2CHRS2";
/// The parent visual the account's first character (`Explorer`) wears.
pub const EXPLORER_VISUAL_PARENT: u32 = 0x1F00_0023;
/// `CharGenVerification`'s weenie result for an accepted character: `CallIFun_IsValid`'s answer
/// for a valid result, the code `gmCharGenResult::SanityCheck` returns for one (its errors are
/// the weenie-error table's; the client reads this value only for a refused creation, where 0
/// means corrupt).
pub const CREATED_WEENIE_RESULT: u32 = 0;

/// The server's character generation rules (the `CharacterGenSystem` script on the server).
pub trait CharGenRules {
    /// `gCharacterGenSystem::CallIFun_IsValid`: the weenie's error for a result the server
    /// refuses; 0 for a valid one.
    fn validate(&mut self, result: &CharGenResult) -> u32;
    /// `gmCharGenResult::InqAppliedAppearanceVDesc(0xFFFFFFFF)`: the appearance the result makes.
    fn appearance(&mut self, result: &CharGenResult) -> Option<Appearance>;
}

/// A created character as the store keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredCharacter {
    pub id: u64,
    pub result: CharGenResult,
    /// The character's `VisualDesc`, as the server sends it.
    pub visual: Vec<u8>,
}

/// The created characters; a store with a path writes itself on every addition.
#[derive(Debug, Default)]
pub struct CharacterStore {
    path: Option<PathBuf>,
    pub records: Vec<StoredCharacter>,
}

impl CharacterStore {
    /// The characters in `path`'s file (none when there is no file yet).
    pub fn open(path: &Path) -> io::Result<Self> {
        let records = match std::fs::read(path) {
            Ok(bytes) => decode(&bytes)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error),
        };
        Ok(Self {
            path: Some(path.to_owned()),
            records,
        })
    }

    /// Whether a created character has this name (names are not case sensitive).
    pub fn taken(&self, name: &str) -> bool {
        self.records
            .iter()
            .any(|r| r.result.name.eq_ignore_ascii_case(name))
    }

    /// The id a new character gets: `CREATED` and then one more for each stored character.
    pub fn next_id(&self) -> u64 {
        CREATED + self.records.len() as u64
    }

    /// Keeps a new character with its visual and writes the store; its id.
    pub fn add(&mut self, result: CharGenResult, visual: Vec<u8>) -> io::Result<u64> {
        let id = self.next_id();
        self.records.push(StoredCharacter { id, result, visual });
        self.save()?;
        Ok(id)
    }

    fn save(&self) -> io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temp = path.with_extension("tmp");
        std::fs::write(&temp, encode(&self.records))?;
        std::fs::rename(temp, path)
    }
}

fn encode(records: &[StoredCharacter]) -> Vec<u8> {
    let mut b = MAGIC.to_vec();
    put_u32(&mut b, records.len() as u32);
    for record in records {
        b.extend(record.id.to_le_bytes());
        let name = record.result.name.as_bytes();
        put_u32(&mut b, name.len() as u32);
        b.extend(name);
        let r = &record.result;
        for word in [
            r.entity_did,
            r.start_area,
            r.race,
            r.sex,
            r.class,
            r.implement,
        ] {
            put_u32(&mut b, word);
        }
        put_u32(&mut b, r.physical.len() as u32);
        for &(key, value) in &r.physical {
            put_u32(&mut b, key);
            put_u32(&mut b, value.to_bits());
        }
        put_u32(&mut b, record.visual.len() as u32);
        b.extend(&record.visual);
    }
    b
}

fn put_u32(b: &mut Vec<u8>, value: u32) {
    b.extend(value.to_le_bytes());
}

/// A cursor over a store file's bytes.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> io::Result<&[u8]> {
        let slice = self
            .bytes
            .get(self.at..self.at + n)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "character store"))?;
        self.at += n;
        Ok(slice)
    }

    fn u32(&mut self) -> io::Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

fn decode(bytes: &[u8]) -> io::Result<Vec<StoredCharacter>> {
    let bad = || io::Error::new(io::ErrorKind::InvalidData, "character store");
    let mut r = Reader { bytes, at: 0 };
    if r.take(MAGIC.len())? != MAGIC {
        return Err(bad());
    }
    let mut records = Vec::new();
    for _ in 0..r.u32()? {
        let id = u64::from(r.u32()?) | u64::from(r.u32()?) << 32;
        let name_len = r.u32()? as usize;
        let name = String::from_utf8(r.take(name_len)?.to_vec()).map_err(|_| bad())?;
        let mut words = [0u32; 6];
        for word in &mut words {
            *word = r.u32()?;
        }
        let [entity_did, start_area, race, sex, class, implement] = words;
        let mut physical = Vec::new();
        for _ in 0..r.u32()? {
            let key = r.u32()?;
            physical.push((key, f32::from_bits(r.u32()?)));
        }
        let visual_len = r.u32()? as usize;
        let visual = r.take(visual_len)?.to_vec();
        records.push(StoredCharacter {
            id,
            result: CharGenResult {
                name,
                entity_did,
                start_area,
                race,
                sex,
                class,
                implement,
                physical,
            },
            visual,
        });
    }
    Ok(records)
}

impl State {
    /// The player's object id: the entered character's, or `PLAYER` before one entered.
    pub fn player(&self) -> u64 {
        self.player_id.unwrap_or(PLAYER)
    }

    /// The visual description the server makes for a character: the rules' appearance with the
    /// entity as its parent (`UpdateByMask` sets the parent the same way); without an answer, the
    /// parent alone. As `VisualDesc` bytes.
    fn visual_bytes_of(&mut self, result: &CharGenResult) -> Vec<u8> {
        let visual = match self
            .rules
            .as_mut()
            .and_then(|rules| rules.appearance(result))
        {
            Some(mut appearance) => {
                appearance.parent = result.entity_did;
                appearance_visual(&appearance)
            }
            None => VisualDesc {
                parent: Some(result.entity_did),
                ..VisualDesc::default()
            },
        };
        visual_desc_bytes(&visual)
    }

    /// The account's characters as the login lists them: `Explorer`, then the created ones with
    /// their stored visuals.
    pub(crate) fn listed_characters(&self) -> Vec<(u64, String, Vec<u8>)> {
        let explorer = VisualDesc {
            parent: Some(EXPLORER_VISUAL_PARENT),
            ..VisualDesc::default()
        };
        let mut listed = vec![(PLAYER, "Explorer".to_owned(), visual_desc_bytes(&explorer))];
        for record in &self.characters.records {
            listed.push((record.id, record.result.name.clone(), record.visual.clone()));
        }
        listed
    }

    /// `Proto_UI::SendCharGenResult`: the server's answer. A result the rules refuse is response
    /// 5 with the weenie's error; a name in use (or `Explorer`'s) is response 3; otherwise the
    /// character is kept with its visual and answered with response 1 and its identity, and the
    /// account's characters are sent again.
    pub(crate) fn create_character(&mut self, account: String, result: CharGenResult) {
        let refused = self
            .rules
            .as_mut()
            .map_or(0, |rules| rules.validate(&result));
        let (response, id, visual, weenie) = if refused != 0 {
            (5, 0, Vec::new(), refused)
        } else if self.characters.taken(&result.name)
            || result.name.eq_ignore_ascii_case("Explorer")
        {
            (3, 0, Vec::new(), 0)
        } else {
            let visual = self.visual_bytes_of(&result);
            let id = self
                .characters
                .add(result.clone(), visual.clone())
                .expect("the character store");
            (1, id, visual, CREATED_WEENIE_RESULT)
        };
        let default_visual = visual_desc_bytes(&VisualDesc::default());
        let visual = if visual.is_empty() {
            default_visual
        } else {
            visual
        };
        let mut b = Vec::new();
        b.extend(0x0001_0057u32.to_le_bytes());
        put_u32(&mut b, response);
        b.extend(id.to_le_bytes());
        ac2_net::pack_wide_string(&mut b, 0, &result.name);
        put_u32(&mut b, 0);
        b.extend(visual);
        put_u32(&mut b, weenie);
        self.reliable_blob(0x2300_0007_0000_0006, 1, &b);
        self.char_gen_results.push((account, result));
        if response == 1 {
            self.next_object_blob += 1;
            let blob = 0x2300_0008_0000_0000 | u64::from(self.next_object_blob);
            self.send_character_set(blob);
        }
    }

    /// `ECM_Login::RecvEvt_CharacterSet` with every listed character, as blob `id`.
    pub(crate) fn send_character_set(&mut self, id: u64) {
        let listed = self.listed_characters();
        let refs: Vec<(u64, &str, &[u8])> = listed
            .iter()
            .map(|(id, name, visual)| (*id, name.as_str(), visual.as_slice()))
            .collect();
        self.reliable_blob_fragmented(id, 1, &crate::character_set_of(&refs));
    }

    /// The player object and qualities the double sends when `character` enters. The player
    /// becomes `character` (its object id; the world's items and contents move to it). A stored
    /// character's object takes its stored visual and name and its race; `Explorer` is the player
    /// the double has always sent.
    pub(crate) fn player_entry(&mut self, character: u64) -> (Vec<u8>, Vec<u8>) {
        self.move_player(character);
        let stored = self
            .characters
            .records
            .iter()
            .find(|r| r.id == character)
            .map(|r| (r.result.clone(), r.visual.clone()));
        let Some((result, visual)) = stored else {
            return (
                crate::create_object(&self.spawn),
                crate::player_description(),
            );
        };
        let spawn = ObjectSpawn {
            id: character,
            instance: 9,
            visual: 0,
            cell: self.spawn.cell,
            origin: self.spawn.origin,
            rotation: self.spawn.rotation,
            stamps: [4, 5, 6, 7],
            name: "",
        };
        let object = weenie_object_with(&spawn, &visual, PLAYER_PACKAGE, &result.name);
        (object, crate::player_description_of(result.race))
    }

    /// The player's object id becomes `to`: the starting items and the containers keyed by the
    /// old id move with it.
    fn move_player(&mut self, to: u64) {
        let from = self.player();
        if from == to {
            return;
        }
        let world = &mut self.script_world;
        if let Some(contents) = world.contents.remove(&from) {
            world.contents.insert(to, contents);
        }
        if let Some(capacity) = world.capacities.remove(&from) {
            world.capacities.insert(to, capacity);
        }
        self.player_id = Some(to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(name: &str, physical: Vec<(u32, f32)>) -> CharGenResult {
        CharGenResult {
            name: name.into(),
            entity_did: 0x1F00_0023,
            start_area: 2,
            race: 1,
            sex: 4096,
            class: 0,
            implement: 3,
            physical,
        }
    }

    #[test]
    fn the_store_round_trips_its_records_and_names_are_unique() {
        let dir = std::env::temp_dir().join(format!("legions-double-{}", std::process::id()));
        let path = dir.join("characters.bin");
        let _ = std::fs::remove_file(&path);
        let mut store = CharacterStore::open(&path).unwrap();
        assert!(store.records.is_empty());
        let zeda = result("Zeda", vec![(2, 1.02), (1, -0.5)]);
        let visual = vec![2, 0, 0, 0, 0x23, 0, 0x1F, 0];
        assert_eq!(store.add(zeda.clone(), visual.clone()).unwrap(), CREATED);
        assert_eq!(
            store.add(result("Ada", vec![]), Vec::new()).unwrap(),
            CREATED + 1
        );
        assert!(store.taken("zeda"));
        assert!(!store.taken("Zedb"));
        let reopened = CharacterStore::open(&path).unwrap();
        assert_eq!(reopened.records, store.records);
        assert_eq!(reopened.records[0].result, zeda);
        assert_eq!(
            reopened.records[0].visual, visual,
            "the stored appearance comes back"
        );
        assert_eq!(reopened.next_id(), CREATED + 2);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_corrupt_store_is_refused() {
        assert!(decode(b"nope").is_err());
        let mut bytes = encode(&[StoredCharacter {
            id: 1,
            result: result("A", vec![]),
            visual: vec![1, 2, 3],
        }]);
        bytes.truncate(bytes.len() - 1);
        assert!(decode(&bytes).is_err());
    }
}
