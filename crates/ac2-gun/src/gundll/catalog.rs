//! The GUN message catalogue: for every message class of GunDll its id, XML element and the attributes
//! `RenderUnicode` (vtable slot 0) prints and `InitFromPacket` (slot 2) reads, as
//! `decompiled-pdb/gundll/spec/0a_receive_messages.md` Part 2 records them (rows verified char for
//! char by `verify_0a_receive_messages.md`). The rows themselves are in the generated `catalog_*.rs`.
//!
//! Template shape (all rows): `<Name ` then, by [`Header`], `Cx="0x%x" ` (request) or
//! `Cx="0x%x" HR="0x%x" ` (response; `GunXml::WriteOpenTagWithId` 0x10012d10, `FUN_10012e80`
//! 0x10012e80), then the attribute fields in order as `Attr="value" `, then `/>` when the class has
//! no child elements or `>` and the children (blob elements `<Elem Val="..."/>`, list rows
//! `<Row A="..." />`) and `</Name> `.

use super::catalog_chat::CHAT;
use super::catalog_conn_ticket::CONN_TICKET;
use super::catalog_friends::FRIENDS;
use super::catalog_names::ELEMENT_TABLE;
use super::catalog_other::OTHER;

/// Which header attributes a class carries (`FUN_10012ce0` request: `Cx`; `FUN_10012e30` result:
/// `Cx` then `HR`; events: none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Header {
    Request,
    Response,
    Event,
}

/// One attribute of a list row: its name and whether it is a decimal (`%d`) or a string (`%ls`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RA(pub &'static str, pub bool);

/// A body field in render order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// `Name="%ls"` (escaped wide string; read through `IXmlPacket` slot 0x10, missing tolerated).
    S(&'static str),
    /// `Name="%d"` (read through slot 0x14, missing tolerated).
    D(&'static str),
    /// `Cnt="%d"`: the count of the class's list rows; written, never read back.
    C(&'static str),
    /// A blob child `<Name Val="..."/>` (6-bit text; the child is mandatory when parsing).
    B(&'static str),
    /// List rows `<Elem A="..." ... />`; every row attribute is required when parsing.
    R(&'static str, &'static [RA]),
    /// Generic list rows `<Elem A="v" A="v" />`: every attribute of the child by index, all printed
    /// under the literal name `A`; a row without attributes counts in `Cnt` but prints nothing
    /// (`FUN_10037190`, `FUN_10037340`).
    X(&'static str),
    /// One generic child `<Name A="v" ... />` (the `Hdrs` of `HdrRowRes`): a missing child is
    /// tolerated, one without attributes prints nothing.
    H(&'static str),
    /// A GUID child `<Name Val="{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}"/>` (`FUN_100375b0`,
    /// `FUN_10037530`); `CLSIDFromString` failure on parse.
    G(&'static str),
    /// The `HostData` block (`FUN_10038770` / `FUN_10038180`), kept by `HostData` (+0x20) and
    /// `MHostData` (+0x24, asm 0x10025c52).
    HD,
}

/// One catalogued message class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgDef {
    /// `Gun::MessageId`.
    pub id: u32,
    /// The XML element name (the 216-entry table at GunDll 0x10068128).
    pub element: &'static str,
    pub header: Header,
    pub fields: &'static [Field],
    /// GunDll address of the class's `RenderUnicode` (0 when the spec row has none recorded).
    pub render: u32,
    /// GunDll address of the class's `InitFromPacket`.
    pub init: u32,
    pub class: &'static str,
    /// True for the 134 rows `verify_0a_receive_messages.md` checked; false for the extractor-only
    /// rows (account, news, persona, FR*, Ds*, M* classes), which the AC2 client barely uses.
    pub verified: bool,
}

/// Const constructor used by the generated rows.
#[allow(clippy::too_many_arguments)]
pub const fn m(
    id: u32,
    element: &'static str,
    header: Header,
    fields: &'static [Field],
    render: u32,
    init: u32,
    class: &'static str,
    verified: bool,
) -> MsgDef {
    MsgDef {
        id,
        element,
        header,
        fields,
        render,
        init,
        class,
        verified,
    }
}

/// `ConnectFailedEv` (0x1302): GunDll has a class (`FUN_10013f20`, ctor id 0x1302, vtable 0x10059db0,
/// `FUN_10013fc0` renders the element `ConnectFailedEv`) but no entry in the 216-entry table, so it
/// is raised locally (`FUN_10009e10` kind 1) and never parsed. 0x1c bytes: an event without fields.
pub static CONNECT_FAILED_EV: MsgDef = m(
    0x1302,
    "ConnectFailedEv",
    Header::Event,
    &[],
    0x10013fc0,
    0x10013f60,
    "(ConnectFailedEv, FUN_10013f20)",
    true,
);

/// `LoginRes` (0x2200): a class exists (ctors 0x10017ad0/0x10017af0, vtable 0x1005a078,
/// `<LoginRes Cx HR Status="%d" />`) that nothing constructs, and no table entry
/// (`0a_receive_messages.md` 1.5).
pub static LOGIN_RES: MsgDef = m(
    0x2200,
    "LoginRes",
    Header::Response,
    &[Field::D("Status")],
    0x10017be0,
    0x10017b60,
    "Gun::CLoginResult",
    true,
);

/// Every catalogued class, in family order.
pub fn all() -> impl Iterator<Item = &'static MsgDef> {
    [CONN_TICKET, FRIENDS, CHAT, OTHER]
        .into_iter()
        .flat_map(|t| t.iter())
        .chain([&CONNECT_FAILED_EV, &LOGIN_RES])
}

/// The class for a message id, if catalogued.
pub fn by_id(id: u32) -> Option<&'static MsgDef> {
    all().find(|d| d.id == id)
}

/// The class for an element name, `_stricmp` as `FUN_10033850`'s table is searched.
pub fn by_element(name: &str) -> Option<&'static MsgDef> {
    let id = element_id(name)?;
    by_id(id)
}

/// `FUN_10033850` (0x10033850): a linear search of the 216-entry `{id, element}` table at 0x10068128
/// with `_stricmp` (case-insensitive); 0 when absent.
pub fn element_id(name: &str) -> Option<u32> {
    ELEMENT_TABLE
        .iter()
        .find(|(_, element)| stricmp(element, name))
        .map(|(id, _)| *id)
}

/// The ids whose classes exist in GunDll but have no case in the message factory `FUN_10004d80`
/// (ConnectEv, DisconnectEv, InitRes, GetTicketRes, GetNewsRes): they are only ever raised locally, and
/// a message with one of these roots read from the wire is `0x8007000E` (final messages audit N3).
pub const LOCAL_ONLY: [u32; 5] = [0x1300, 0x1301, 0x1204, 0xf203, 0x3200];

/// CRT `_stricmp(a, b) == 0` over bytes (`tolower` of the "C" locale).
pub fn stricmp(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .all(|(x, y)| x.eq_ignore_ascii_case(&y))
}

/// Ids of the table without a catalogued class: GetNews 0x3100, NewsEv 0x3301, GetIdFromName(Res)
/// 0x4106/0x4206 and GetNameFromId(Res) 0x4107/0x4207 have no class in GunDll at all, so both the DLL
/// and the port answer `0x8007000E` for them.
pub fn is_open(id: u32) -> bool {
    element_id_known(id) && by_id(id).is_none()
}

fn element_id_known(id: u32) -> bool {
    ELEMENT_TABLE.iter().any(|(i, _)| *i == id)
}
