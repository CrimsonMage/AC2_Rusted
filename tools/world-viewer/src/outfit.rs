//! The player's clothes in the viewer. The client never builds an outfit itself: the server
//! sends the avatar's `VisualDesc` (the appearance tables its worn gear and its physique pick),
//! and the character creation screens ask the `CharacterGenSystem` script for the same thing
//! (`CallIFun_SetCGRDefaults`, then `gmCharGenResult::InqAppliedAppearanceVDesc`). The viewer
//! takes that second path, through the client's own `ScriptCharGen`, so `--outfit starter`
//! dresses the player as a new character of that race and sex is dressed (the racial starter
//! shirt, trousers and boots). The viewer's player never has a server to tell it.

use std::sync::Arc;

use ac2_client::ScriptCharGen;
use ac2_core::DataId;
use ac2_dat::DatDataset;
use ac2_db::ResourceStore;
use ac2_net::protocol::CharGenResult;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_ui::pregame::CharGenSystem;
use ac2_weenie::Appearance;

use crate::races::PlayerModel;

/// What `--outfit` asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outfit {
    /// The undressed base body (the default).
    None,
    /// A new character's defaults: `SetCGRDefaults` and the appearance they apply.
    Starter,
}

impl Outfit {
    pub fn parse(word: &str) -> Result<Self, String> {
        match word {
            "none" => Ok(Self::None),
            "starter" => Ok(Self::Starter),
            other => Err(format!("--outfit: none or starter, not {other}")),
        }
    }
}

/// The appearance the player model wears under `outfit`, parented to the model's own visual (the
/// viewer's body is the race table's, which the server's `VisualDesc` would name as its parent).
pub fn appearance(
    outfit: &Outfit,
    dataset: &Arc<DatDataset>,
    store: &ResourceStore,
    model: &PlayerModel,
) -> Result<Appearance, String> {
    let base = Appearance {
        parent: model.visual.raw(),
        ..Appearance::default()
    };
    if *outfit == Outfit::None {
        return Ok(base);
    }
    let wlib = ac2_script::load_wlib(store, DataId::new(ac2_script::LEGIONS_WLIB_DID))
        .map_err(|e| e.to_string())?
        .ok_or("the DATs hold no script library")?;
    let data = DatScriptData::new(Arc::clone(dataset), store.clone());
    data.preload_tables();
    data.preload_states();
    data.set_cache_only(true);
    let host = ScriptHost::new(Arc::new(Program::from_stream(&wlib.stream)), Box::new(data));
    let mut system = ScriptCharGen::new(host);
    let mut result = CharGenResult {
        race: model.race,
        sex: model.sex,
        entity_did: system.avatar_entity_did(false),
        ..CharGenResult::default()
    };
    if !system.set_defaults(&mut result) {
        return Err("CharacterGenSystem: SetCGRDefaults failed".into());
    }
    let applied = system
        .applied_appearance(&result)
        .ok_or("CharacterGenSystem: InqAppliedAppearanceVDesc failed")?;
    Ok(Appearance {
        parent: base.parent,
        ..applied
    })
}
