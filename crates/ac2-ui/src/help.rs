//! Help pages for `KeyStone::OpenHelp(token, table)` (Legions 0x004B26D0): the help window's
//! page for a help id.
//!
//! Retail: a token of 0 opens the help plugin's default page. Any other token goes through
//! `StringInfo::SetStringIDandTableEnum(token, 0x4100004E)` and `GetString`: the table entry
//! named by the token is the page, a `.ksml` file name the plugin renders. The token is
//! `Tokenize("ID_Help_<topic>")`, the same hash the callers keep in their `ID_Help_*` globals.
//! The shipped table has one entry per topic below. The window itself is not drawn here yet.

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
