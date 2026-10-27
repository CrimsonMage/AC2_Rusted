//! The `.keymap` files a player's controls screen saves (`gmKeyboardUI::SaveKeyboardList`) and the
//! client reads back at start (`gmClient::InitKeymap`, `CInputManager_WIN32::PushKeyMap`).
//!
//! A keymap file is a `PFileParser` text tree: `name [ child child ... ]`, a node's children in
//! brackets, names bare or in double quotes (`PFileParser::PutQuotedString` quotes a name with
//! white space or one of `#()[]{},"` and writes an empty name as `""`), and `#` comments to the end
//! of the line. `CKeyMap::ToFileNode` writes the tree this way, and `CKeyMap::FromFileNode` reads it:
//!
//! ```text
//! VersionHeader [ 1 ]
//! "" [ <map name> ]
//! "" [ <Data1> <Data2> <Data3> <Data4 0> ... <Data4 7> ]    the map's GUID, 11 parts
//! "" [ <action name> <semantic name> ]                       one per binding
//! ```
//!
//! The semantic name is the key's name (`CKeyMap::GetNameBySemantic`); this port writes its own
//! key names (`key_names`), so a file from a retail client loads only the bindings whose names
//! this port knows (the others are dropped, as `FromFileNode` drops an unknown semantic name).
//! `PFileParser`'s include files, escapes and version upgrade table (`UpdateVersion`) are not
//! read.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::maps::{ActionMapping, KeyMap};
use crate::pregame::key_names::{key_name, semantic_from_name};
use crate::pregame::KeyMapSet;

/// The name of the version header node (`CKeyMap::ToFileNode`'s `VersionHeader`).
const VERSION_HEADER: &str = "VersionHeader";
/// The version `ToFileNode` writes.
const VERSION: u32 = 1;
/// A GUID's parts: `Data1`, `Data2`, `Data3` and eight `Data4` bytes.
const GUID_PARTS: usize = 11;

/// One node of a `PFileParser` tree: a name and its children (none for a leaf).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    pub name: String,
    pub children: Vec<FileNode>,
}

impl FileNode {
    fn leaf(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
        }
    }

    fn record(children: Vec<FileNode>) -> Self {
        Self {
            name: String::new(),
            children,
        }
    }
}

/// A file that does not hold a keymap this reader understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapFormatError(pub &'static str);

impl std::fmt::Display for KeymapFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "keymap file: {}", self.0)
    }
}

impl std::error::Error for KeymapFormatError {}

// ---------------------------------------------------------------------------------------------
// The text tree
// ---------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Name(String),
    Open,
    Close,
}

fn tokenize(text: &str) -> Result<Vec<Token>, KeymapFormatError> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '#' => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '[' => {
                chars.next();
                tokens.push(Token::Open);
            }
            ']' => {
                chars.next();
                tokens.push(Token::Close);
            }
            '"' => {
                chars.next();
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some(c) => name.push(c),
                        None => return Err(KeymapFormatError("unclosed quoted name")),
                    }
                }
                tokens.push(Token::Name(name));
            }
            _ => {
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || matches!(c, '[' | ']' | '"' | '#') {
                        break;
                    }
                    name.push(c);
                    chars.next();
                }
                tokens.push(Token::Name(name));
            }
        }
    }
    Ok(tokens)
}

/// Parses a keymap file's text into its root node (the file itself, unnamed).
pub fn parse_tree(text: &str) -> Result<FileNode, KeymapFormatError> {
    let tokens = tokenize(text)?;
    let mut pos = 0;
    let children = parse_nodes(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err(KeymapFormatError("unbalanced brackets"));
    }
    Ok(FileNode::record(children))
}

fn parse_nodes(tokens: &[Token], pos: &mut usize) -> Result<Vec<FileNode>, KeymapFormatError> {
    let mut nodes = Vec::new();
    while let Some(token) = tokens.get(*pos) {
        let Token::Name(name) = token else {
            break;
        };
        *pos += 1;
        let mut node = FileNode::leaf(name.clone());
        if let Some(Token::Open) = tokens.get(*pos) {
            *pos += 1;
            node.children = parse_nodes(tokens, pos)?;
            if tokens.get(*pos) != Some(&Token::Close) {
                return Err(KeymapFormatError("missing ]"));
            }
            *pos += 1;
        }
        nodes.push(node);
    }
    Ok(nodes)
}

/// Formats a tree as keymap text: one node per line, children indented (the layout
/// `PFileParser::FormatNode` writes, without its line-wrap rules).
pub fn format_tree(root: &FileNode) -> String {
    let mut out = String::new();
    write_nodes(&mut out, &root.children, 0);
    out
}

fn write_nodes(out: &mut String, nodes: &[FileNode], depth: usize) {
    for node in nodes {
        for _ in 0..depth {
            out.push('\t');
        }
        write_name(out, &node.name);
        if !node.children.is_empty() {
            out.push_str(" [\n");
            write_nodes(out, &node.children, depth + 1);
            for _ in 0..depth {
                out.push('\t');
            }
            out.push(']');
        }
        out.push('\n');
    }
}

/// `PFileParser::PutQuotedString`: a name is quoted when it is empty or holds white space or a
/// delimiter.
fn write_name(out: &mut String, name: &str) {
    let quote = name.is_empty()
        || name.chars().any(|c| {
            c.is_whitespace() || matches!(c, '#' | '(' | ')' | ',' | '[' | ']' | '{' | '}')
        });
    if quote {
        out.push('"');
        out.push_str(name);
        out.push('"');
    } else {
        out.push_str(name);
    }
}

// ---------------------------------------------------------------------------------------------
// The keymap node layout (CKeyMap::ToFileNode / FromFileNode)
// ---------------------------------------------------------------------------------------------

/// `CKeyMap::ToFileNode`: the tree a map is saved as.
pub fn key_map_tree(map: &KeyMap) -> FileNode {
    let mut root = Vec::new();
    root.push(FileNode {
        name: VERSION_HEADER.to_owned(),
        children: vec![FileNode::leaf(VERSION.to_string())],
    });
    root.push(FileNode::record(vec![FileNode::leaf(map.name.clone())]));
    let guid = &map.guid;
    let mut parts = vec![
        FileNode::leaf(u32::from_le_bytes([guid[0], guid[1], guid[2], guid[3]]).to_string()),
        FileNode::leaf(u16::from_le_bytes([guid[4], guid[5]]).to_string()),
        FileNode::leaf(u16::from_le_bytes([guid[6], guid[7]]).to_string()),
    ];
    parts.extend(guid[8..].iter().map(|b| FileNode::leaf(b.to_string())));
    root.push(FileNode::record(parts));
    for mapping in &map.mappings {
        root.push(FileNode::record(vec![
            FileNode::leaf(mapping.name.clone()),
            FileNode::leaf(key_name(mapping.semantic)),
        ]));
    }
    FileNode::record(root)
}

/// `CKeyMap::FromFileNode`: a map read from its tree. `actions` maps an action's name to its
/// enum value; a name it does not know drops that binding, as `GetEnum` returning 0 does.
pub fn key_map_from_tree(
    root: &FileNode,
    actions: &HashMap<String, u32>,
) -> Result<KeyMap, KeymapFormatError> {
    let mut records = root.children.iter().peekable();
    if let Some(header) = records.peek() {
        if header.name == VERSION_HEADER {
            header
                .children
                .first()
                .and_then(|v| parse_uint(&v.name))
                .ok_or(KeymapFormatError("invalid version info"))?;
            records.next();
        }
    }
    let name = record(records.next(), 1, "missing map name")?
        .first()
        .map(|n| n.name.clone())
        .unwrap_or_default();
    let guid_parts = record(records.next(), GUID_PARTS, "GUID must have 11 parts")?;
    let mut guid = [0_u8; 16];
    let number = |node: &FileNode| -> Result<u64, KeymapFormatError> {
        parse_uint(&node.name).ok_or(KeymapFormatError("invalid GUID part"))
    };
    guid[..4].copy_from_slice(&(number(&guid_parts[0])? as u32).to_le_bytes());
    guid[4..6].copy_from_slice(&(number(&guid_parts[1])? as u16).to_le_bytes());
    guid[6..8].copy_from_slice(&(number(&guid_parts[2])? as u16).to_le_bytes());
    for (byte, part) in guid[8..].iter_mut().zip(&guid_parts[3..]) {
        *byte = number(part)? as u8;
    }
    let mut mappings = Vec::new();
    for node in records {
        if !node.name.is_empty() || node.children.len() != 2 {
            return Err(KeymapFormatError("a binding is an action and a key"));
        }
        let (action, key) = (&node.children[0].name, &node.children[1].name);
        let (Some(action), Some(semantic)) = (action_of(actions, action), semantic_from_name(key))
        else {
            continue;
        };
        mappings.push(ActionMapping {
            action,
            semantic,
            name: node.children[0].name.clone(),
        });
    }
    Ok(KeyMap {
        name,
        guid,
        mappings,
    })
}

/// The action a name stands for: its name in the table, or the enum value a name the table does
/// not hold is written as (`0x........`, the port's form for an action no map names).
fn action_of(actions: &HashMap<String, u32>, name: &str) -> Option<u32> {
    actions.get(name).copied().or_else(|| {
        let hex = name.strip_prefix("0x")?;
        u32::from_str_radix(hex, 16).ok()
    })
}

fn record<'a>(
    node: Option<&'a FileNode>,
    parts: usize,
    error: &'static str,
) -> Result<&'a [FileNode], KeymapFormatError> {
    match node {
        Some(node) if node.name.is_empty() && node.children.len() == parts => Ok(&node.children),
        _ => Err(KeymapFormatError(error)),
    }
}

/// `PSUtils::is_uint32` with `strtoul(.., 0)`: decimal, or hexadecimal with `0x`.
fn parse_uint(text: &str) -> Option<u64> {
    let value = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16).ok()?,
        None => text.parse::<u64>().ok()?,
    };
    (value <= u64::from(u32::MAX)).then_some(value)
}

/// The action table a file's action names resolve through: each name the maps give to an action
/// (`EnumMapper::GetEnum(0x4000001d, name)` in retail).
pub fn action_table<'a>(maps: impl IntoIterator<Item = &'a KeyMap>) -> HashMap<String, u32> {
    let mut table = HashMap::new();
    for map in maps {
        for mapping in &map.mappings {
            if !mapping.name.is_empty() {
                table.entry(mapping.name.clone()).or_insert(mapping.action);
            }
        }
    }
    table
}

/// The text of a key map file.
pub fn format_key_map(map: &KeyMap) -> String {
    format_tree(&key_map_tree(map))
}

/// A key map from its file's text.
pub fn parse_key_map(
    text: &str,
    actions: &HashMap<String, u32>,
) -> Result<KeyMap, KeymapFormatError> {
    key_map_from_tree(&parse_tree(text)?, actions)
}

// ---------------------------------------------------------------------------------------------
// The user's files
// ---------------------------------------------------------------------------------------------

/// The three keymap files (`Main.keymap`, `Shift.keymap`, `Control.keymap`, the preference bank's
/// entries `0x40000030` to `0x40000032`), in the user's configuration directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapFiles {
    pub main: PathBuf,
    pub shift: PathBuf,
    pub control: PathBuf,
}

impl KeymapFiles {
    /// The files under the user's AC2 configuration directory (`ac2_prefs::paths::resolve`).
    pub fn user() -> Self {
        Self {
            main: ac2_prefs::paths::resolve("Main.keymap"),
            shift: ac2_prefs::paths::resolve("Shift.keymap"),
            control: ac2_prefs::paths::resolve("Control.keymap"),
        }
    }

    /// The file of map `index`: 0 main, 1 shift, 2 control (`gmClient::InitKeymap`'s order).
    pub fn path(&self, index: usize) -> &Path {
        match index {
            0 => &self.main,
            1 => &self.shift,
            _ => &self.control,
        }
    }

    /// The three files in `dir`, under their retail names.
    pub fn in_dir(dir: &Path) -> Self {
        Self {
            main: dir.join("Main.keymap"),
            shift: dir.join("Shift.keymap"),
            control: dir.join("Control.keymap"),
        }
    }
}

/// Reads one keymap file: `Ok(None)` when it does not exist (the DAT's map is used then).
pub fn read_key_map(
    path: &Path,
    actions: &HashMap<String, u32>,
) -> io::Result<Option<Result<KeyMap, KeymapFormatError>>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(parse_key_map(&text, actions))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Writes a keymap set's three files (`SaveKeyboardList`'s `ToFileNode` and `SaveToFile`).
pub fn save_set(files: &KeymapFiles, set: &KeyMapSet) -> io::Result<()> {
    ac2_prefs::paths::write_atomic(&files.main, format_key_map(&set.main).as_bytes())?;
    ac2_prefs::paths::write_atomic(&files.shift, format_key_map(&set.shift).as_bytes())?;
    ac2_prefs::paths::write_atomic(&files.control, format_key_map(&set.control).as_bytes())
}
