//! The playable race and sex player models. `CharacterGenSystem` (a script) keeps the table;
//! the viewer never links the script or network crates, so `race-table` writes it to a file
//! (`race<TAB>sex<TAB>phys_obj`, hex) and the viewer reads that, running the tool once when the
//! file is missing. Each `phys_obj` is an `EntityDesc` whose data DID is the player's visual.

use std::path::{Path, PathBuf};
use std::process::Command;

use ac2_assets::Ac2EntityDesc;
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};

/// One race and sex and the visual its player wears.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerModel {
    pub race: u32,
    pub sex: u32,
    pub phys_obj: u32,
    pub visual: DataId,
}

impl PlayerModel {
    /// `race 0x1, male`: the race and sex ids as the script names them.
    pub fn label(&self) -> String {
        let sex = match self.sex {
            0x1000 => "male".to_string(),
            0x2000 => "female".to_string(),
            other => format!("sex {other:#x}"),
        };
        format!("race {:#x}, {sex}", self.race)
    }
}

/// Where the table lives: `AC2_RACE_TABLE`, else the user's cache directory.
pub fn table_path() -> PathBuf {
    if let Some(path) = std::env::var_os("AC2_RACE_TABLE") {
        return path.into();
    }
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_default();
    cache.join("ac2-world-viewer").join("race_table.tsv")
}

/// Runs the `race-table` tool next to the viewer's executable, writing `path`.
fn generate(path: &Path, dat_dir: &Path) -> Result<(), String> {
    let tool = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .with_file_name("race-table");
    if !tool.exists() {
        return Err(format!(
            "{} is missing: build it with `cargo build -p race-table`, or write the table with \
             `race-table --out {}`",
            tool.display(),
            path.display()
        ));
    }
    let status = Command::new(&tool)
        .arg("--dat-dir")
        .arg(dat_dir)
        .arg("--out")
        .arg(path)
        .status()
        .map_err(|e| format!("{}: {e}", tool.display()))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{} failed ({status})", tool.display()))
}

/// The player models, generating the table file first when it is missing.
pub fn load(store: &ResourceStore, dat_dir: &Path) -> Result<Vec<PlayerModel>, String> {
    let path = table_path();
    if !path.exists() {
        generate(&path, dat_dir)?;
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut models = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line
            .split('\t')
            .map(|field| u32::from_str_radix(field.trim(), 16));
        let (Some(Ok(race)), Some(Ok(sex)), Some(Ok(phys_obj))) =
            (fields.next(), fields.next(), fields.next())
        else {
            return Err(format!("{}: bad line {line:?}", path.display()));
        };
        let entity = store
            .load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(phys_obj),
                0,
            ))
            .map_err(|e| e.to_string())?;
        let Some(visual) = entity.and_then(|entity| entity.data) else {
            continue;
        };
        models.push(PlayerModel {
            race,
            sex,
            phys_obj,
            visual,
        });
    }
    if models.is_empty() {
        return Err(format!("{} lists no player models", path.display()));
    }
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_name_the_race_and_sex_ids() {
        let model = |sex| PlayerModel {
            race: 0x10,
            sex,
            phys_obj: 0x4700_0001,
            visual: DataId::new(0x1F00_0001),
        };
        assert_eq!(model(0x1000).label(), "race 0x10, male");
        assert_eq!(model(0x2000).label(), "race 0x10, female");
        assert_eq!(model(0x3000).label(), "race 0x10, sex 0x3000");
    }
}
