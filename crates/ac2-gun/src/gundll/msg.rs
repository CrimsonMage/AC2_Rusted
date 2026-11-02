//! A GUN message as GunDll holds it (`Gun::CBaseMessage` and its subclasses) in one generic form
//! driven by the [`catalog`](super::catalog): `RenderUnicode` (vtable slot 0) and `InitFromPacket`
//! (slot 2) of every class, and the `Init` calls that fill a request. See
//! `decompiled-pdb/gundll/spec/0a_receive_messages.md` 1.3 (escaping, `Cx`/`HR` formats, tolerance)
//! and Part 2 (the templates).

use super::blob;
use super::catalog::{stricmp, Field, Header, MsgDef};
use super::msg_extra::{generic_row, parse_guid, HostBlock};
use super::xml_parse::XmlPacket;
use crate::interfaces::Guid;
use crate::zt::*;
use crate::Hresult;

/// One field's value, aligned with [`MsgDef::fields`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// The `Cnt` field (derived from the rows) and other fields with no stored value.
    None,
    /// A string; `None` is a NULL pointer (a missing attribute reads as NULL).
    Str(Option<String>),
    Dec(i32),
    /// A blob; `None` renders `Val=""`.
    Blob(Option<Vec<u8>>),
    /// List rows, each aligned with the row's attributes.
    Rows(Vec<Vec<Value>>),
    /// A GUID child (`Field::G`); the zero GUID until set.
    Guid(Guid),
    /// The `HostData` block (`Field::HD`).
    Host(HostBlock),
}

/// A message object: `m_msgId`, the request context / result header, and the body fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GunMsg {
    pub def: &'static MsgDef,
    /// `m_Context` (+0x1c): `Cx`. Unused for events.
    pub cx: u32,
    /// `m_hr` (+0x20): `HR`. Used for responses only.
    pub hr: u32,
    pub values: Vec<Value>,
}

impl GunMsg {
    /// A zeroed object of the class (the constructors zero the body fields).
    pub fn new(def: &'static MsgDef) -> GunMsg {
        let values = def
            .fields
            .iter()
            .map(|f| match f {
                Field::S(_) => Value::Str(None),
                Field::D(_) => Value::Dec(0),
                Field::C(_) => Value::None,
                Field::B(_) => Value::Blob(None),
                Field::R(..) | Field::X(_) | Field::H(_) => Value::Rows(Vec::new()),
                Field::G(_) => Value::Guid(Guid {
                    data1: 0,
                    data2: 0,
                    data3: 0,
                    data4: [0; 8],
                }),
                Field::HD => Value::Host(HostBlock::default()),
            })
            .collect();
        GunMsg {
            def,
            cx: 0,
            hr: 0,
            values,
        }
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.def.fields.iter().position(|f| match f {
            Field::S(n)
            | Field::D(n)
            | Field::B(n)
            | Field::R(n, _)
            | Field::X(n)
            | Field::H(n)
            | Field::G(n) => *n == name,
            Field::C(_) | Field::HD => false,
        })
    }

    /// Sets a string field by attribute name.
    pub fn set_str(&mut self, name: &str, v: Option<&str>) {
        if let Some(i) = self.index(name) {
            self.values[i] = Value::Str(v.map(str::to_owned));
        }
    }

    pub fn set_dec(&mut self, name: &str, v: i32) {
        if let Some(i) = self.index(name) {
            self.values[i] = Value::Dec(v);
        }
    }

    pub fn set_blob(&mut self, name: &str, v: Option<&[u8]>) {
        if let Some(i) = self.index(name) {
            self.values[i] = Value::Blob(v.map(<[u8]>::to_vec));
        }
    }

    pub fn set_rows(&mut self, name: &str, rows: Vec<Vec<Value>>) {
        if let Some(i) = self.index(name) {
            self.values[i] = Value::Rows(rows);
        }
    }

    /// The string field `name` (`None` when NULL).
    pub fn str_field(&self, name: &str) -> Option<&str> {
        match self.index(name).map(|i| &self.values[i]) {
            Some(Value::Str(s)) => s.as_deref(),
            _ => None,
        }
    }

    pub fn dec_field(&self, name: &str) -> i32 {
        match self.index(name).map(|i| &self.values[i]) {
            Some(Value::Dec(d)) => *d,
            _ => 0,
        }
    }

    pub fn blob_field(&self, name: &str) -> Option<&[u8]> {
        match self.index(name).map(|i| &self.values[i]) {
            Some(Value::Blob(b)) => b.as_deref(),
            _ => None,
        }
    }

    pub fn rows_field(&self, name: &str) -> &[Vec<Value>] {
        match self.index(name).map(|i| &self.values[i]) {
            Some(Value::Rows(r)) => r,
            _ => &[],
        }
    }
}

/// `FUN_10035db0` (0x10035db0): exactly four replacements per UTF-16 unit, `"` `&` `<` `>` to
/// `&quot;` `&amp;` `&lt;` `&gt;`; nothing else is escaped.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("&quot;"),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// A `%ls` argument: the escaped string, or `(null)` for a NULL pointer. [inferred: the MSVC CRT
/// prints NULL as "(null)"; `FUN_10035db0` returns NULL for NULL (`0a_send_session.md` 6.3)]
fn ls(v: &Option<String>) -> String {
    match v {
        Some(s) => escape(s),
        None => "(null)".to_owned(),
    }
}

/// `<Elem A="v" A="v" />` (`FUN_10037340`): every attribute under the literal name `A`.
fn generic_row_text(elem: &str, row: &[Value]) -> String {
    let mut out = format!("<{elem} ");
    for v in row {
        if let Value::Str(s) = v {
            out.push_str(&format!("A=\"{}\" ", ls(s)));
        }
    }
    out.push_str("/>");
    out
}

/// The render buffer is 0xFFFF wide characters (`FUN_10012a70`); longer is `ZT_E_STR_TOO_LONG`.
pub const RENDER_CAP: usize = 0xFFFF;

impl GunMsg {
    /// `RenderUnicode` of the class (`GunXml::WriteOpenTagWithId` 0x10012d10 / `FUN_10012e80` header,
    /// then the template of `catalog`), as the wide text; `FUN_10012a70` rejects more than 0xFFFF
    /// characters with `ZT_E_STR_TOO_LONG` (the printf's `ZT_E_BUFFER_TOO_SMALL` mapped).
    pub fn render(&self) -> Result<String, Hresult> {
        let def = self.def;
        let mut out = format!("<{} ", def.element);
        match def.header {
            Header::Request => out.push_str(&format!("Cx=\"0x{:x}\" ", self.cx)),
            Header::Response => {
                out.push_str(&format!("Cx=\"0x{:x}\" HR=\"0x{:x}\" ", self.cx, self.hr))
            }
            Header::Event => {}
        }
        let rows_len = def
            .fields
            .iter()
            .zip(&self.values)
            .find_map(|(f, v)| match (f, v) {
                (Field::R(..) | Field::X(_), Value::Rows(r)) => Some(r.len()),
                _ => None,
            })
            .unwrap_or(0);
        let mut children = String::new();
        let mut has_children = false;
        for (f, v) in def.fields.iter().zip(&self.values) {
            match (f, v) {
                (Field::S(n), Value::Str(s)) => out.push_str(&format!("{n}=\"{}\" ", ls(s))),
                (Field::D(n), Value::Dec(d)) => out.push_str(&format!("{n}=\"{d}\" ")),
                (Field::C(n), _) => out.push_str(&format!("{n}=\"{rows_len}\" ")),
                (Field::B(n), Value::Blob(b)) => {
                    has_children = true;
                    // FUN_100376d0: `<%hs %hs="%hs"/>` with "Val"; a NULL blob is `Val=""`
                    let enc = match b {
                        Some(bytes) => blob::encode_for_attribute(bytes)?,
                        None => String::new(),
                    };
                    children.push_str(&format!("<{n} Val=\"{enc}\"/>"));
                }
                (Field::R(elem, attrs), Value::Rows(rows)) => {
                    has_children = true;
                    for row in rows {
                        children.push_str(&format!("<{elem} "));
                        for (a, v) in attrs.iter().zip(row) {
                            match v {
                                Value::Str(s) => {
                                    children.push_str(&format!("{}=\"{}\" ", a.0, ls(s)))
                                }
                                Value::Dec(d) => children.push_str(&format!("{}=\"{d}\" ", a.0)),
                                _ => {}
                            }
                        }
                        children.push_str("/>");
                    }
                }
                (Field::X(elem), Value::Rows(rows)) => {
                    has_children = true;
                    for row in rows.iter().filter(|r| !r.is_empty()) {
                        children.push_str(&generic_row_text(elem, row));
                    }
                }
                (Field::H(elem), Value::Rows(rows)) => {
                    has_children = true;
                    for row in rows.iter().filter(|r| !r.is_empty()) {
                        children.push_str(&generic_row_text(elem, row));
                    }
                }
                (Field::G(n), Value::Guid(g)) => {
                    has_children = true;
                    children.push_str(&format!("<{n} Val=\"{}\"/>", super::cgun::guid_text(g)));
                }
                (Field::HD, Value::Host(h)) => {
                    has_children = true;
                    children.push_str(&h.render(&escape));
                }
                _ => {}
            }
        }
        if has_children {
            out.push('>');
            out.push_str(&children);
            out.push_str(&format!("</{}> ", def.element));
        } else {
            out.push_str("/>");
        }
        if out.encode_utf16().count() > RENDER_CAP {
            return Err(ZT_E_STR_TOO_LONG);
        }
        Ok(out)
    }

    /// `InitFromPacket` of the class on a parsed packet whose root is this class's element:
    /// the header (`FUN_10012ce0` / `FUN_10012e30`: `Cx`, `HR` required, lower-case hex), then the
    /// body. A missing string or decimal attribute is tolerated (`ZT_E_ATTR_NOT_FOUND` skipped; the
    /// field keeps NULL / 0), any other error aborts; blob children and list rows are strict
    /// (`FUN_10037740`, the row readers). [confirmed: `verify_0a_receive_messages.md` 1.3-miss]
    pub fn init_from_packet(def: &'static MsgDef, pkt: &XmlPacket) -> Result<GunMsg, Hresult> {
        let mut m = GunMsg::new(def);
        match def.header {
            Header::Request => m.cx = pkt.attr_hex(None, "Cx")?,
            Header::Response => {
                m.cx = pkt.attr_hex(None, "Cx")?;
                m.hr = pkt.attr_hex(None, "HR")?;
            }
            Header::Event => {}
        }
        for i in read_order(def) {
            let f = &def.fields[i];
            match f {
                Field::S(n) => {
                    m.values[i] = Value::Str(match pkt.attr_string(None, n) {
                        Ok(v) => Some(String::from_utf16_lossy(v)),
                        Err(e) if e == ZT_E_ATTR_NOT_FOUND => None,
                        Err(e) => return Err(e),
                    })
                }
                Field::D(n) => {
                    m.values[i] = Value::Dec(match pkt.attr_decimal(None, n) {
                        Ok(v) => v,
                        Err(e) if e == ZT_E_ATTR_NOT_FOUND => 0,
                        Err(e) => return Err(e),
                    })
                }
                Field::C(_) => {}
                Field::X(elem) => {
                    // FUN_10037190: every child named like the row, its attributes by index
                    let mut rows = Vec::new();
                    for c in 0..pkt.child_count(None) {
                        let child = pkt.child(None, c)?;
                        if stricmp(pkt.name(Some(child)), elem) {
                            rows.push(generic_row(pkt, child)?);
                        }
                    }
                    m.values[i] = Value::Rows(rows);
                }
                Field::H(n) => {
                    // a missing child is tolerated (nothing printed)
                    if let Ok(child) = pkt.find_child(None, n) {
                        m.values[i] = Value::Rows(vec![generic_row(pkt, child)?]);
                    }
                }
                Field::G(n) => {
                    // FUN_100375b0: the child, its attribute 0, CLSIDFromString
                    let child = pkt.find_child(None, n)?;
                    let (_, v) = pkt.attr_by_index(Some(child), 0)?;
                    let end = v.iter().position(|&c| c == 0).unwrap_or(v.len());
                    m.values[i] = Value::Guid(parse_guid(&v[..end])?);
                }
                Field::HD => {
                    // FUN_10038770: `ZT_E_ATTR_NOT_FOUND` (an `Item` without `ItemId`, or a Del
                    // `Item` without `Num`) is tolerated and leaves the block NULL; the DLL's render
                    // then dereferences NULL (crash path, AGENTS.md), the port prints the empty block
                    match HostBlock::parse(pkt) {
                        Ok(b) => m.values[i] = Value::Host(b),
                        Err(e) if e == ZT_E_ATTR_NOT_FOUND => {}
                        Err(e) => return Err(e),
                    }
                }
                Field::B(n) => {
                    // FUN_10037740: the child, its attribute index 0 (whatever its name), decoded
                    let child = pkt.find_child(None, n)?;
                    let (_, v) = pkt.attr_by_index(Some(child), 0)?;
                    let end = v.iter().position(|&c| c == 0).unwrap_or(v.len());
                    let bytes = blob::decode(&v[..end], end)?;
                    m.values[i] = Value::Blob(Some(bytes));
                }
                Field::R(elem, attrs) => {
                    // The row readers (FUN_10036df0, FUN_10036040, ...) answer 0xc1120007 when a row
                    // lacks an attribute, and EVERY caller tolerates exactly that code
                    // (`cmp esi, 0xc1120007 ; jne fail ; xor esi, esi`, e.g. GetFriendsListRes Init
                    // 0x1001a2c6): the message is accepted with the out slots the reader never wrote,
                    // an array NULL (the zeroed local) and a count that is a stale stack slot (the
                    // `this` pointer value). Deviation: that count would crash any reader, so the
                    // list is empty here. The 15 classes: 0x8204 0x4200 0x4203 0x430c 0x5200
                    // 0x5204 0x5207 0x5350 0x5351 0xE106 0xA203 0x4601 0x4602 0x7202 0xE202.
                    match read_rows(pkt, def.id, elem, attrs) {
                        Ok(rows) => m.values[i] = Value::Rows(rows),
                        Err(e) if e == ZT_E_ATTR_NOT_FOUND => m.values[i] = Value::Rows(Vec::new()),
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        Ok(m)
    }
}

/// The receive pipeline of `FUN_10009ad0` (0x10009ad0): the text must end in NUL; UTF-8 to UTF-16
/// (`FUN_10002700`, `MultiByteToWideChar(CP_UTF8, 0, ...)`); `FUN_10033710` parses; the root element
/// name is looked up with `FUN_10033850` (`ZT_E_MISSING_OBJECT` for id 0); the factory
/// `FUN_10004d80` builds the class (`0x8007000e` without one); `InitFromPacket`.
pub fn parse_message(data: &[u8]) -> Result<GunMsg, Hresult> {
    if data.is_empty() || data[data.len() - 1] != 0 {
        return Err(Hresult(0x8007_0057));
    }
    // [ms-docs: multibytetowidechar.md] with dwFlags 0 the function "replaces illegal sequences with
    // U+FFFD" and returns the length including the NUL for cbMultiByte = -1; `from_utf8_lossy` does the
    // same. // open: how many U+FFFD one invalid run becomes (the page says only "fully conforms with the
    // Unicode 4.1 specification").
    let s = String::from_utf8_lossy(&data[..data.len() - 1]);
    let wide: Vec<u16> = s.encode_utf16().take_while(|&c| c != 0).collect();
    let pkt = XmlPacket::parse(&wide)?;
    let id = super::catalog::element_id(pkt.name(None)).ok_or(ZT_E_MISSING_OBJECT)?;
    // the factory FUN_10004d80 has no case for the five local-only ids nor for the six table ids
    // without a class: 0x8007000E
    if super::catalog::LOCAL_ONLY.contains(&id) {
        return Err(Hresult(0x8007_000E));
    }
    let def = super::catalog::by_id(id).ok_or(Hresult(0x8007_000E))?;
    GunMsg::init_from_packet(def, &pkt)
}

/// The row readers: every child named like the row, each attribute required (`0xc1120007` when one
/// is missing, an invalid decimal `0xc112000c`).
///
/// GetFriendsListRes 0x4200, FStart 0x430c, GetListOfRoomsRes 0x5200, ViewListRes 0x7202 and
/// MGetViewListRes 0xe202 use readers that allocate
/// `childCount` rows, fill the slot at each matching CHILD index and count only the matches
/// (`FUN_10036df0` for FriendStatus; observed in the DLL: `Other, FriendStatus u1, FriendStatus u2`
/// gives count 2 and rows `[blank, u1]`, the last row lost); a slot no row filled is zeroed (NULL
/// strings, 0). Every other row class compacts the matching rows.
fn read_rows(
    pkt: &XmlPacket,
    id: u32,
    elem: &str,
    attrs: &[super::catalog::RA],
) -> Result<Vec<Vec<Value>>, Hresult> {
    let by_child_index = matches!(id, 0x4200 | 0x430c | 0x5200 | 0x7202 | 0xe202);
    let blank = || -> Vec<Value> {
        attrs
            .iter()
            .map(|a| if a.1 { Value::Dec(0) } else { Value::Str(None) })
            .collect()
    };
    let mut slots: Vec<Vec<Value>> = Vec::new();
    let mut count = 0usize;
    let mut rows = Vec::new();
    if by_child_index {
        slots = (0..pkt.child_count(None)).map(|_| blank()).collect();
    }
    for c in 0..pkt.child_count(None) {
        let child = pkt.child(None, c)?;
        if !stricmp(pkt.name(Some(child)), elem) {
            continue;
        }
        let mut row = Vec::new();
        for a in attrs.iter() {
            if a.1 {
                row.push(Value::Dec(pkt.attr_decimal(Some(child), a.0)?));
            } else {
                let v = pkt.attr_string(Some(child), a.0)?;
                row.push(Value::Str(Some(String::from_utf16_lossy(v))));
            }
        }
        if by_child_index {
            if let Some(slot) = slots.get_mut(c) {
                *slot = row;
            }
            count += 1;
        } else {
            rows.push(row);
        }
    }
    if by_child_index {
        slots.truncate(count);
        return Ok(slots);
    }
    Ok(rows)
}

/// The order `InitFromPacket` reads the body in (field indices). It is the render order except for
/// five classes (disassembly of their Inits): ConInfo 0x1103 reads Locale, CdKey, AppGuid, Random;
/// PresentTicketCmd 0xf101 Ticket then Email; CallCreate 0x10100 CallType, TargetPersonaName,
/// Description, CallData, LangID; CtEnterEv 0x5350 and CtLeaveEv 0x5351 the Targs list, User, RoomId.
/// Only which error comes first can differ.
fn read_order(def: &MsgDef) -> Vec<usize> {
    let names: &[&str] = match def.id {
        0x1103 => &["Locale", "CdKey", "AppGuid", "Random"],
        0xf101 => &["Ticket", "Email"],
        0x10100 => &[
            "CallType",
            "TargetPersonaName",
            "Description",
            "CallData",
            "LangID",
        ],
        0x5350 | 0x5351 => &["Targs", "User", "RoomId"],
        _ => return (0..def.fields.len()).collect(),
    };
    let mut order: Vec<usize> = names
        .iter()
        .filter_map(|n| {
            def.fields.iter().position(|f| match f {
                Field::S(x) | Field::D(x) | Field::B(x) | Field::R(x, _) => x == n,
                _ => false,
            })
        })
        .collect();
    for i in 0..def.fields.len() {
        if !order.contains(&i) {
            order.push(i);
        }
    }
    order
}
