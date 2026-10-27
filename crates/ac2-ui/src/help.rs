//! Help pages for `KeyStone::OpenHelp(token, table)` (Legions 0x004B26D0): the help window's
//! page for a help id.
//!
//! Retail: a token of 0 opens the help plugin's default page. Any other token goes through
//! `StringInfo::SetStringIDandTableEnum(token, 0x4100004E)` and `GetString`: the table entry
//! named by the token is the page, a `.ksml` file name the plugin renders. The token is
//! `Tokenize("ID_Help_<topic>")`, the same hash the callers keep in their `ID_Help_*` globals.
//! The shipped table has one entry per topic below. The window is [`crate::help_window`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ac2_assets::ac2_enum_did;
use ac2_db::{load_string_table, ResourceResult, ResourceStore, StringTable};
use ac2_db::{CachedResource, STRING_TABLE_FIRST, STRING_TABLE_LAST};

use crate::ksml::{parse_ksml, KsmlDocument, KsmlError};
use crate::token::tokenize;

/// The string table enum `KeyStone::OpenHelp` passes (`0x4100004E`).
pub const HELP_TABLE_ENUM: u32 = 0x4100_004E;

/// `FileArray::GetDIDFromEnum`'s group for string tables (`0x40000004`).
const STRING_TABLE_GROUP: u32 = 0x4000_0004;

/// The page token 0 opens. `AC2HelpPlugin.dll` carries this name as its own default page.
pub const DEFAULT_HELP_PAGE: &str = "content/ac2-help.ksml";

/// A help topic: the `ID_Help_*` name a window passes to `KeyStone::OpenHelp`, and the retail
/// windows that pass it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpTopic {
    pub id: &'static str,
    pub callers: &'static str,
}

/// Every topic a retail window opens. `gmToolbarUI` opens the default page (token 0) instead.
pub const HELP_TOPICS: &[HelpTopic] = &[
    HelpTopic {
        id: "ID_Help_Shop",
        callers: "gmShopUI",
    },
    HelpTopic {
        id: "ID_Help_ShopCatalog",
        callers: "gmShopCatalogUI",
    },
    HelpTopic {
        id: "ID_Help_ShopConsignment",
        callers: "gmShopConsignmentUI, gmShopNewSaleUI",
    },
    HelpTopic {
        id: "ID_Help_Book",
        callers: "gmBookUI",
    },
    HelpTopic {
        id: "ID_Help_Skill",
        callers: "gmSkillUI",
    },
    HelpTopic {
        id: "ID_Help_Navigation",
        callers: "gmNavigationUI",
    },
    HelpTopic {
        id: "ID_Help_Option",
        callers: "gmOptionUI",
    },
    HelpTopic {
        id: "ID_Help_OverviewMap",
        callers: "gmOverviewMapUI",
    },
    HelpTopic {
        id: "ID_Help_Inventory",
        callers: "gmInventoryUI",
    },
    HelpTopic {
        id: "ID_Help_Quest",
        callers: "gmQuestsUI",
    },
    HelpTopic {
        id: "ID_Help_Social",
        callers: "gmSocialUI",
    },
    HelpTopic {
        id: "ID_Help_Examine",
        callers: "gmExaminationUI",
    },
];

/// The token for a topic's `ID_Help_*` name (`Tokenize`).
pub const fn help_token(topic: &HelpTopic) -> u32 {
    tokenize(topic.id)
}

/// Why a help page did not open.
#[derive(Debug)]
pub enum HelpError {
    /// The token names no entry of the help table.
    UnknownToken(u32),
    /// The page file is missing or unreadable.
    Read {
        page: String,
        source: std::io::Error,
    },
    /// The page is not valid KSML.
    Page { page: String, source: KsmlError },
}

impl std::fmt::Display for HelpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownToken(token) => write!(f, "no help page for token {token:08x}"),
            Self::Read { page, source } => write!(f, "help page {page}: {source}"),
            Self::Page { page, source } => write!(f, "help page {page}: {source}"),
        }
    }
}

impl std::error::Error for HelpError {}

/// The page a token names: token 0 is [`DEFAULT_HELP_PAGE`], any other token is the table's
/// entry text (a plain page name, no meta letters). `None` when the table has no such entry.
pub fn help_page_name(table: &StringTable, token: u32) -> Option<String> {
    if token == 0 {
        return Some(DEFAULT_HELP_PAGE.to_string());
    }
    let text = table.get(token)?.text();
    (!text.is_empty()).then_some(text)
}

/// The file a page name opens under the content directory (`Content`, which the page names
/// reach as `content/<page>`; the directory itself is the one the plugin ships beside).
pub fn help_page_path(content_dir: &Path, page: &str) -> PathBuf {
    let stripped = page
        .get(..8)
        .filter(|prefix| {
            prefix.eq_ignore_ascii_case("content/") || prefix.eq_ignore_ascii_case("content\\")
        })
        .map_or(page, |_| &page[8..]);
    content_dir.join(stripped)
}

/// Reads and parses the page a help token names. Reads a file: call it off the frame thread.
pub fn load_help_page(
    content_dir: &Path,
    table: &StringTable,
    token: u32,
) -> Result<KsmlDocument, HelpError> {
    let page = help_page_name(table, token).ok_or(HelpError::UnknownToken(token))?;
    let path = help_page_path(content_dir, &page);
    let bytes = std::fs::read(&path).map_err(|source| HelpError::Read {
        page: page.clone(),
        source,
    })?;
    parse_ksml(&bytes).map_err(|source| HelpError::Page { page, source })
}

/// The help string table: the table enum [`HELP_TABLE_ENUM`] resolves to in the local DAT.
/// `None` when the DAT has no such enum. Reads files: call it on a worker.
pub fn load_help_table(
    store: &ResourceStore,
) -> ResourceResult<Option<CachedResource<StringTable>>> {
    let Some(did) = ac2_enum_did(store, STRING_TABLE_GROUP, HELP_TABLE_ENUM)? else {
        return Ok(None);
    };
    if !(STRING_TABLE_FIRST..=STRING_TABLE_LAST).contains(&did.raw()) {
        return Ok(None);
    }
    load_string_table(store, did)
}

/// `gmToolbarUI::ListenToMessage`'s key stone key (`0x400000BB`): `ShowKeyStone(!shown)`, the
/// help window's default page when it opens, the window closed when it is shown.
pub const KEYSTONE_PANEL_KEY: u32 = 0x4000_00BB;

/// The help buttons whose panel's `ListenToMessage` opens a topic (`elementID == ...HelpButton`,
/// then `KeyStone::OpenHelp`), as (button element name, topic id). `gmBookUI` and `gmSocialUI`
/// return their help request from their own panel instead. `ShopHelpButton` is both `gmShopUI`'s
/// and `gmShopNewSaleUI`'s; the HUD routes the new-sale one while its dialog shows.
pub const HELP_BUTTONS: &[(&str, &str)] = &[
    ("ShopHelpButton", "ID_Help_Shop"),
    ("CatalogHelpButton", "ID_Help_ShopCatalog"),
    ("HelpButton", "ID_Help_ShopConsignment"),
    ("SkillHelpButton", "ID_Help_Skill"),
    ("NavigationHelpButton", "ID_Help_Navigation"),
    ("OptionHelpButton", "ID_Help_Option"),
    ("OverviewMapHelpButton", "ID_Help_OverviewMap"),
    ("InventoryHelpButton", "ID_Help_Inventory"),
    ("QuestHelpButton", "ID_Help_Quest"),
    ("ExaminationHelpButton", "ID_Help_Examine"),
];

/// The topic a help button's element name opens, from [`HELP_BUTTONS`].
pub fn help_button_topic(element_id: u32) -> Option<&'static HelpTopic> {
    let (_, id) = HELP_BUTTONS
        .iter()
        .find(|(name, _)| tokenize(name) == element_id)?;
    HELP_TOPICS.iter().find(|topic| topic.id == *id)
}

/// The key a page name is stored under: its file name, the `content/` prefix dropped.
pub fn page_key(page: &str) -> &str {
    page.get(..8)
        .filter(|prefix| {
            prefix.eq_ignore_ascii_case("content/") || prefix.eq_ignore_ascii_case("content\\")
        })
        .map_or(page, |_| &page[8..])
}

/// The help pages a client keeps in memory: every `.ksml` file of the content directory parsed
/// once (on a worker), and the page each topic token names. The window reads these on the frame
/// thread, so it never touches a file.
#[derive(Debug, Default, Clone)]
pub struct HelpPages {
    pages: HashMap<String, KsmlDocument>,
    topics: HashMap<u32, String>,
}

impl HelpPages {
    /// The parsed page a name (`content/<file>` or `<file>`) names.
    pub fn page(&self, name: &str) -> Option<&KsmlDocument> {
        self.pages.get(page_key(name))
    }

    /// The page a token opens: token 0 is [`DEFAULT_HELP_PAGE`], a topic token its table entry.
    pub fn page_for_token(&self, token: u32) -> Option<&str> {
        if token == 0 {
            return Some(DEFAULT_HELP_PAGE);
        }
        self.topics.get(&token).map(String::as_str)
    }

    /// The number of parsed pages.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Whether no page parsed.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

/// Loads the help pages from `content_dir` and the topic table from the DAT. `None` when the
/// DAT has no help table. A file that does not parse is skipped. Reads files: call it on a worker.
pub fn load_help_pages(
    store: &ResourceStore,
    content_dir: &Path,
) -> ResourceResult<Option<HelpPages>> {
    let Some(table) = load_help_table(store)? else {
        return Ok(None);
    };
    let table = &*table;
    let mut topics = HashMap::new();
    for topic in HELP_TOPICS {
        if let Some(page) = help_page_name(table, help_token(topic)) {
            topics.insert(help_token(topic), page);
        }
    }
    let mut pages = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(content_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_ksml = path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("ksml"));
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !is_ksml {
                continue;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(document) = parse_ksml(&bytes) {
                    pages.insert(name.to_string(), document);
                }
            }
        }
    }
    Ok(Some(HelpPages { pages, topics }))
}
