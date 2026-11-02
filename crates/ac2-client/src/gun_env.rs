//! What the client gives GUN where retail used Win32 and COM: the `GunDll.dll` it "loads" (its
//! own [`ac2_gun::gundll::CGun`] over DirectPlay 8, `-gunclientdll` ignored), the registry values
//! `CGun` keeps under `HKCU\Software\Microsoft\Microsoft Games` (files in the AC2 configuration
//! directory), and `GetUserDefaultLCID` for platforms without Windows.

use std::path::PathBuf;
use std::rc::Rc;

use ac2_gun::dp8_bridge::Dx9Dp8Factory;
use ac2_gun::gundll::{CGun, GunEnv, GunRegistry};
use ac2_gun::turbine_gun::{CLSID_GUN, IID_IGUN_CONNECTION};
use ac2_gun::turbine_gun_utils::{ClassFactory, GunLoader, GunModule};
use ac2_gun::{Guid, GunObject, Hresult};

/// `E_NOINTERFACE`.
const E_NOINTERFACE: Hresult = Hresult(0x8000_4002);
/// `CLASS_E_CLASSNOTAVAILABLE`.
const CLASS_E_CLASSNOTAVAILABLE: Hresult = Hresult(0x8004_0111);

/// `HKCU\Software\Microsoft\Microsoft Games` as a directory of value files: a value `name` is the
/// file of that name, its bytes the value's data (a `REG_DWORD` is its 4 little-endian bytes).
pub struct FileRegistry {
    dir: PathBuf,
}

impl FileRegistry {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The default place: `<config>/AC2/MicrosoftGames`.
    pub fn user() -> Self {
        Self::new(ac2_prefs::paths::config_dir().join("MicrosoftGames"))
    }

    fn path(&self, name: &str) -> Option<PathBuf> {
        // Value names are fixed identifiers; refuse anything that is not a plain file name.
        (!name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != "..")
            .then(|| self.dir.join(name))
    }
}

impl GunRegistry for FileRegistry {
    fn query_value(&self, name: &str, max_len: usize) -> Option<Vec<u8>> {
        let data = std::fs::read(self.path(name)?).ok()?;
        (data.len() <= max_len).then_some(data)
    }

    fn set_binary(&self, name: &str, data: &[u8]) -> bool {
        let Some(path) = self.path(name) else {
            return false;
        };
        std::fs::create_dir_all(&self.dir).is_ok() && std::fs::write(path, data).is_ok()
    }

    fn set_dword(&self, name: &str, value: u32) -> bool {
        self.set_binary(name, &value.to_le_bytes())
    }

    fn delete_value(&self, name: &str) -> bool {
        self.path(name)
            .is_some_and(|path| std::fs::remove_file(path).is_ok())
    }
}

struct Factory(Option<Box<dyn GunObject>>);

impl ClassFactory for Factory {
    fn create_instance(&mut self, riid: &Guid) -> (Hresult, Option<Box<dyn GunObject>>) {
        if *riid != IID_IGUN_CONNECTION {
            return (E_NOINTERFACE, None);
        }
        (Hresult::S_OK, self.0.take())
    }
}

struct Module {
    gun: Option<Box<dyn GunObject>>,
}

impl GunModule for Module {
    fn dll_get_class_object(
        &mut self,
        rclsid: &Guid,
        _riid: &Guid,
    ) -> Option<(Hresult, Option<Box<dyn ClassFactory>>)> {
        if *rclsid != CLSID_GUN {
            return Some((CLASS_E_CLASSNOTAVAILABLE, None));
        }
        Some((
            Hresult::S_OK,
            Some(Box::new(Factory(self.gun.take())) as Box<dyn ClassFactory>),
        ))
    }

    fn get_last_error(&self) -> u32 {
        0
    }
}

/// `LoadLibraryA`: the module is always the client's own `CGun`, whatever the path.
pub struct OwnGunLoader {
    registry: Rc<dyn GunRegistry>,
}

impl OwnGunLoader {
    pub fn new(registry: Rc<dyn GunRegistry>) -> Self {
        Self { registry }
    }
}

impl GunLoader for OwnGunLoader {
    fn load_library(&mut self, _path: &str) -> Result<Box<dyn GunModule>, u32> {
        let env = GunEnv::new(Rc::new(Dx9Dp8Factory::system()), Rc::clone(&self.registry));
        Ok(Box::new(Module {
            gun: Some(Box::new(CGun::new(env))),
        }))
    }
}

/// Language tags of Microsoft's `[MS-LCID]` section 2.2 (Windows Language Code Identifier
/// Reference, version 16.0, 2024-04-23, <https://learn.microsoft.com/en-us/openspecs/
/// windows_protocols/ms-lcid/63d3d639-7fd2-4afb-abbe-0d5b5551eef8>), the rows this maps.
const LCIDS: &[(&str, u32)] = &[
    ("ar-SA", 0x0401),
    ("bg-BG", 0x0402),
    ("ca-ES", 0x0403),
    ("zh-TW", 0x0404),
    ("cs-CZ", 0x0405),
    ("da-DK", 0x0406),
    ("de-DE", 0x0407),
    ("el-GR", 0x0408),
    ("en-US", 0x0409),
    ("fi-FI", 0x040B),
    ("fr-FR", 0x040C),
    ("he-IL", 0x040D),
    ("hu-HU", 0x040E),
    ("is-IS", 0x040F),
    ("it-IT", 0x0410),
    ("ja-JP", 0x0411),
    ("ko-KR", 0x0412),
    ("nl-NL", 0x0413),
    ("nb-NO", 0x0414),
    ("pl-PL", 0x0415),
    ("pt-BR", 0x0416),
    ("ro-RO", 0x0418),
    ("ru-RU", 0x0419),
    ("hr-HR", 0x041A),
    ("sk-SK", 0x041B),
    ("sv-SE", 0x041D),
    ("th-TH", 0x041E),
    ("tr-TR", 0x041F),
    ("uk-UA", 0x0422),
    ("sl-SI", 0x0424),
    ("et-EE", 0x0425),
    ("lv-LV", 0x0426),
    ("lt-LT", 0x0427),
    ("vi-VN", 0x042A),
    ("zh-CN", 0x0804),
    ("de-CH", 0x0807),
    ("en-GB", 0x0809),
    ("es-MX", 0x080A),
    ("fr-BE", 0x080C),
    ("it-CH", 0x0810),
    ("nl-BE", 0x0813),
    ("nn-NO", 0x0814),
    ("pt-PT", 0x0816),
    ("zh-HK", 0x0C04),
    ("de-AT", 0x0C07),
    ("en-AU", 0x0C09),
    ("es-ES", 0x0C0A),
    ("fr-CA", 0x0C0C),
    ("en-CA", 0x1009),
    ("fr-CH", 0x100C),
    ("en-NZ", 0x1409),
    ("en-IE", 0x1809),
    ("en-ZA", 0x1C09),
    ("en-IN", 0x4009),
    // The neutral language rows (Language ID table).
    ("ar", 0x0001),
    ("bg", 0x0002),
    ("ca", 0x0003),
    ("cs", 0x0005),
    ("da", 0x0006),
    ("de", 0x0007),
    ("el", 0x0008),
    ("en", 0x0009),
    ("es", 0x000A),
    ("fi", 0x000B),
    ("fr", 0x000C),
    ("he", 0x000D),
    ("hu", 0x000E),
    ("is", 0x000F),
    ("it", 0x0010),
    ("ja", 0x0011),
    ("ko", 0x0012),
    ("nl", 0x0013),
    ("pl", 0x0015),
    ("pt", 0x0016),
    ("ro", 0x0018),
    ("ru", 0x0019),
    ("hr", 0x001A),
    ("sk", 0x001B),
    ("sv", 0x001D),
    ("th", 0x001E),
    ("tr", 0x001F),
    ("uk", 0x0022),
];

/// The LCID of a POSIX locale name (`de_DE.UTF-8`, `fr_CA@euro`, `en`): the name reduced to a
/// language tag (`de-DE`) and looked up in [`LCIDS`]; a tag the table lacks falls back to its
/// language alone (`de`). `None` for `C`, `POSIX`, an empty name or a language the table rows
/// above do not carry.
pub fn lcid_for_locale(locale: &str) -> Option<u32> {
    let base = locale.split(['.', '@']).next()?;
    if base.is_empty() || base == "C" || base == "POSIX" {
        return None;
    }
    let tag = base.replace('_', "-");
    let find = |t: &str| {
        LCIDS
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(t))
            .map(|e| e.1)
    };
    find(&tag).or_else(|| find(tag.split('-').next()?))
}

/// `GetUserDefaultLCID()`: the user's locale names (`sys-locale`: on Windows
/// `GetUserDefaultLocaleName`, which names the same locale `GetUserDefaultLCID` numbers, one to
/// one; elsewhere `LC_ALL`/`LC_MESSAGES`/`LANG`) through [`lcid_for_locale`]; the first one the
/// table maps. Without one the answer is 0x0409 (en-US), the language the retail client and its
/// `local_English.dat` default to.
// open: off Windows the POSIX UI-language chain is only the nearest analogue of the user's
// regional-format locale, and an unmapped locale is reported as en-US.
pub fn default_lcid() -> u32 {
    sys_locale::get_locales()
        .find_map(|locale| lcid_for_locale(&locale))
        .unwrap_or(0x0409)
}

/// `StringInfo::IsOnDisk` for the GUN strings of the table enum 0x4100000E and 0x41000018
/// (`ID_GunReAuthenticationFailed` / `ID_GunReAuthenticationSuccess` and the data patch
/// errors), read once from the DATs: whether `(string name, table enum)` has its text.
pub fn string_on_disk(store: &ac2_db::ResourceStore) -> Box<dyn Fn(&str, u32) -> bool> {
    use ac2_db::RenderInfo;
    let mut found = std::collections::HashSet::new();
    if let Ok(strings) = ac2_ui::pregame::UiStrings::load(store) {
        for table_enum in [0x4100_000Eu32, 0x4100_0018] {
            let did = ac2_assets::ac2_enum_did(store, ac2_ui::STRING_TABLE_GROUP, table_enum)
                .ok()
                .flatten()
                .map_or(0, |did| did.raw());
            for name in [
                "ID_GunReAuthenticationFailed",
                "ID_GunReAuthenticationSuccess",
                "ID_DataPatch_GUNFailed",
                "ID_DataPatch_GUNAccountBanned",
            ] {
                let info = RenderInfo {
                    string_id: ac2_net::tokenize(name.as_bytes()),
                    table_id: did,
                    ..RenderInfo::default()
                };
                if did != 0 && strings.is_on_disk(&info) {
                    found.insert((name, table_enum));
                }
            }
        }
    }
    Box::new(move |name, table| found.contains(&(name, table)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_locales_map_through_the_ms_lcid_table() {
        assert_eq!(lcid_for_locale("en_US.UTF-8"), Some(0x0409));
        assert_eq!(lcid_for_locale("de_DE.UTF-8"), Some(0x0407));
        assert_eq!(lcid_for_locale("fr_CA@euro"), Some(0x0C0C));
        assert_eq!(lcid_for_locale("pt_PT"), Some(0x0816));
        // A country the table has no row for falls back to the language.
        assert_eq!(lcid_for_locale("de_LU.UTF-8"), Some(0x0007));
        assert_eq!(lcid_for_locale("ja"), Some(0x0011));
        assert_eq!(lcid_for_locale("C"), None);
        assert_eq!(lcid_for_locale("POSIX"), None);
        assert_eq!(lcid_for_locale(""), None);
        assert_eq!(lcid_for_locale("xx_YY"), None);
    }

    #[test]
    fn the_registry_keeps_values_as_files() {
        let dir = std::env::temp_dir().join(format!("ac2-gun-reg-{}", std::process::id()));
        let reg = FileRegistry::new(dir.clone());
        assert_eq!(reg.query_value("LaunchTicketSize", 4), None);
        assert!(reg.set_dword("LaunchTicketSize", 0x0102_0304));
        assert_eq!(
            reg.query_value("LaunchTicketSize", 4),
            Some(vec![4, 3, 2, 1])
        );
        // The data must fit the buffer the caller offers.
        assert_eq!(reg.query_value("LaunchTicketSize", 3), None);
        assert!(reg.set_binary("LaunchTicketData", &[9, 9]));
        assert!(reg.delete_value("LaunchTicketData"));
        assert!(!reg.delete_value("LaunchTicketData"));
        assert!(!reg.set_binary("../x", &[1]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_loader_hands_out_the_clients_own_object_for_the_gun_clsid() {
        let mut loader = OwnGunLoader::new(Rc::new(FileRegistry::new(std::env::temp_dir())));
        let mut module = loader.load_library("whatever.dll").unwrap();
        let wrong = Guid {
            data1: 1,
            data2: 2,
            data3: 3,
            data4: [0; 8],
        };
        let (hr, factory) = module
            .dll_get_class_object(&wrong, &ac2_gun::turbine_gun_utils::IID_ICLASS_FACTORY)
            .unwrap();
        assert!(!hr.succeeded() && factory.is_none());
        let (hr, factory) = module
            .dll_get_class_object(&CLSID_GUN, &ac2_gun::turbine_gun_utils::IID_ICLASS_FACTORY)
            .unwrap();
        assert!(hr.succeeded());
        let mut factory = factory.unwrap();
        assert_eq!(factory.create_instance(&wrong).0, E_NOINTERFACE);
        let (hr, gun) = factory.create_instance(&IID_IGUN_CONNECTION);
        assert!(hr.succeeded() && gun.is_some());
    }
}
