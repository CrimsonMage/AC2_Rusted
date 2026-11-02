//! The message shapes of the 13 browser, host-data and directory classes that the first catalogue
//! left open (final messages audit N4): generic rows (`PageRes`, `RowPgRes`, `MPageRes`, ...), the
//! generic `Hdrs` child, the `Guid` child of `DsRegisterServer`/`DsAddServerEv`, and the `HostData`
//! block of `HostData`/`MHostData` (`FUN_10038770` reads it, `FUN_10038180` writes it).

use super::msg::Value;
use super::xml_parse::{parse_decimal, NodeId, XmlPacket};
use crate::interfaces::Guid;
use crate::zt::*;
use crate::Hresult;

/// `CLSIDFromString` failure `CO_E_CLASSSTRING` (the audit's observation for `Guid Val`).
pub const CO_E_CLASSSTRING: Hresult = Hresult(0x8004_01F3);

/// A `HostData` item (`FUN_1000d2d0` object: `ItemId` at +0x34, the attribute table `FUN_1000c500`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HostItem {
    pub id: i32,
    /// Attributes in the order they were first added (the render order is the table's, see
    /// [`HostItem::render_order`]).
    pub attrs: Vec<(String, String)>,
}

/// The `HostData` payload: the `New` and `Mod` item tables and the `Del` list (`CRect`, 0x10 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HostBlock {
    pub new: Vec<HostItem>,
    pub modified: Vec<HostItem>,
    pub del: Vec<i32>,
}

/// The hash `LAB_10035880` (0x10035880) of an attribute name: over at most 16 characters
/// `h = h * 0x11 + tolower(c)` (the call at 0x100358a1 is `FUN_10044be5`, the CRT `_tolower`; final
/// messages audit N8 corrects an earlier `isupper` reading).
fn name_hash(name: &str) -> u32 {
    let mut h = 0u32;
    for b in name.bytes().take(16) {
        h = h
            .wrapping_mul(0x11)
            // `movsx edx,al` before the call: the character is sign-extended, then `add edi,eax`.
            .wrapping_add(b.to_ascii_lowercase() as i8 as i32 as u32);
    }
    h
}

#[cfg(test)]
#[test]
fn name_hash_is_lowercase_times_0x11() {
    // h = h*0x11 + tolower(c): "Ab" -> 0x61*0x11 + 0x62 = 0x6D3, case-insensitive; 16 characters at most.
    assert_eq!(name_hash("Ab"), 0x61 * 0x11 + 0x62);
    assert_eq!(name_hash("AB"), name_hash("ab"));
    assert_eq!(
        name_hash("abcdefghijklmnopZZ"),
        name_hash("abcdefghijklmnop")
    );
}

/// A table of `FUN_1000c500` (8 buckets, mask 7): entries are pushed at the head of their bucket
/// (`FUN_1000cab0`, `FUN_1000c8a0`) and walked bucket 0 to 7, head first (`FUN_1000c670`). Returns
/// the indices in that walk order.
fn walk_order(buckets: &[u32]) -> Vec<usize> {
    let mut order = Vec::new();
    for b in 0..8u32 {
        for (i, _) in buckets
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, x)| **x & 7 == b)
        {
            order.push(i);
        }
    }
    order
}

impl HostItem {
    /// `FUN_10034e60`: a name already present (`lstrcmpiA`) gets its value replaced, a new one is
    /// added; a name over 127 characters or a value over 127 is `ZT_E_STR_TOO_LONG`.
    fn set(&mut self, name: &str, value: &str) -> Result<(), Hresult> {
        if name.len() + 1 >= 0x81 || value.encode_utf16().count() + 1 > 0x80 {
            return Err(ZT_E_STR_TOO_LONG);
        }
        match self
            .attrs
            .iter_mut()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
        {
            Some(a) => a.1 = value.to_owned(),
            None => self.attrs.push((name.to_owned(), value.to_owned())),
        }
        Ok(())
    }

    /// The attribute indices in table order (bucket `hash & 7`, newest first).
    pub fn render_order(&self) -> Vec<usize> {
        let h: Vec<u32> = self.attrs.iter().map(|(n, _)| name_hash(n)).collect();
        walk_order(&h)
    }
}

impl HostBlock {
    fn render_items(items: &[HostItem], escape: &dyn Fn(&str) -> String) -> String {
        // the item table is keyed by `ItemId` (`LAB_1000c3b0` is the identity): bucket `id & 7`
        let keys: Vec<u32> = items.iter().map(|i| i.id as u32).collect();
        let mut out = String::new();
        for i in walk_order(&keys) {
            let it = &items[i];
            out.push_str(&format!("<Item ItemId=\"{}\" ", it.id));
            for a in it.render_order() {
                out.push_str(&format!(
                    "{}=\"{}\" ",
                    it.attrs[a].0,
                    escape(&it.attrs[a].1)
                ));
            }
            out.push_str("/>");
        }
        out
    }

    /// `FUN_10038180(.., "HostData", block)`: `<HostData >`, `<New >..</New> `, `<Mod >..</Mod> `,
    /// `<Del >` with `<Item Num="%d" />` per entry, `</Del> `, `</HostData> `.
    pub fn render(&self, escape: &dyn Fn(&str) -> String) -> String {
        let mut out = String::from("<HostData >");
        out.push_str(&format!(
            "<New >{}</New> ",
            Self::render_items(&self.new, escape)
        ));
        out.push_str(&format!(
            "<Mod >{}</Mod> ",
            Self::render_items(&self.modified, escape)
        ));
        out.push_str("<Del >");
        for n in &self.del {
            out.push_str(&format!("<Item Num=\"{n}\" />"));
        }
        out.push_str("</Del> </HostData> ");
        out
    }

    /// `FUN_10038770(pkt, root)`: the `HostData` child of the root, its `New`, `Mod` and `Del`
    /// children (a missing one is `ZT_E_NOT_FOUND`). `Item` children of `New`/`Mod` need `ItemId`
    /// (decimal, else the attribute error) and keep their other attributes; `Item` children of `Del`
    /// are read through `Num`, a string parsed by `FUN_10002a00` (decimal, `FUN_10036270`).
    pub fn parse(pkt: &XmlPacket) -> Result<HostBlock, Hresult> {
        let host = pkt.find_child(None, "HostData")?;
        let new = Self::parse_items(pkt, pkt.find_child(Some(host), "New")?)?;
        let modified = Self::parse_items(pkt, pkt.find_child(Some(host), "Mod")?)?;
        let del_node = pkt.find_child(Some(host), "Del")?;
        let mut del = Vec::new();
        for c in 0..pkt.child_count(Some(del_node)) {
            let child = pkt.child(Some(del_node), c)?;
            if pkt.name(Some(child)).eq_ignore_ascii_case("Item") {
                del.push(parse_decimal(pkt.attr_string(Some(child), "Num")?)?);
            }
        }
        Ok(HostBlock { new, modified, del })
    }

    fn parse_items(pkt: &XmlPacket, node: NodeId) -> Result<Vec<HostItem>, Hresult> {
        let mut items = Vec::new();
        for c in 0..pkt.child_count(Some(node)) {
            let child = pkt.child(Some(node), c)?;
            if !pkt.name(Some(child)).eq_ignore_ascii_case("Item") {
                continue;
            }
            let mut item = HostItem {
                id: pkt.attr_decimal(Some(child), "ItemId")?,
                attrs: Vec::new(),
            };
            for a in 0..pkt.attr_count(Some(child)) {
                let (name, value) = pkt.attr_by_index(Some(child), a)?;
                if name.eq_ignore_ascii_case("ItemId") {
                    continue;
                }
                let end = value.iter().position(|&c| c == 0).unwrap_or(value.len());
                item.set(name, &String::from_utf16_lossy(&value[..end]))?;
            }
            items.push(item);
        }
        Ok(items)
    }
}

/// All attributes of a node by index as a row of strings (`FUN_100370a0`).
pub fn generic_row(pkt: &XmlPacket, node: NodeId) -> Result<Vec<Value>, Hresult> {
    let mut row = Vec::new();
    for a in 0..pkt.attr_count(Some(node)) {
        let (_, v) = pkt.attr_by_index(Some(node), a)?;
        let end = v.iter().position(|&c| c == 0).unwrap_or(v.len());
        row.push(Value::Str(Some(String::from_utf16_lossy(&v[..end]))));
    }
    Ok(row)
}

/// `CLSIDFromString` for `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` (any hex case).
pub fn parse_guid(s: &[u16]) -> Result<Guid, Hresult> {
    let t = String::from_utf16(s).map_err(|_| CO_E_CLASSSTRING)?;
    let b = t.as_bytes();
    if b.len() != 38 || b[0] != b'{' || b[37] != b'}' {
        return Err(CO_E_CLASSSTRING);
    }
    for i in [9, 14, 19, 24] {
        if b[i] != b'-' {
            return Err(CO_E_CLASSSTRING);
        }
    }
    let hex = |r: std::ops::Range<usize>| -> Result<u64, Hresult> {
        let part = &t[r];
        if !part.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(CO_E_CLASSSTRING);
        }
        u64::from_str_radix(part, 16).map_err(|_| CO_E_CLASSSTRING)
    };
    let mut data4 = [0u8; 8];
    let tail = [
        (20, 22),
        (22, 24),
        (25, 27),
        (27, 29),
        (29, 31),
        (31, 33),
        (33, 35),
        (35, 37),
    ];
    for (i, (a, e)) in tail.into_iter().enumerate() {
        data4[i] = hex(a..e)? as u8;
    }
    Ok(Guid {
        data1: hex(1..9)? as u32,
        data2: hex(10..14)? as u16,
        data3: hex(15..19)? as u16,
        data4,
    })
}
