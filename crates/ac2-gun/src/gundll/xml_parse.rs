//! GunDll's own XML parser and the `IXmlPacket` DOM it builds, ported from the decompile and
//! `decompiled-pdb/gundll/spec/0a_receive_messages.md` 1.3 (verified in `verify_0a_receive_messages.md`).
//!
//! * Tokenizer: `FUN_10038ec0` (0x10038ec0) with the state handlers `FUN_10038cf0`, `FUN_10038dc0`,
//!   `FUN_10038e10`, `FUN_10038e40`, `FUN_10038c50`, `FUN_10038cd0`, `FUN_10038bc0`, `FUN_10038c00`,
//!   `FUN_10038d50`, `FUN_10038b50` and the character-class table `FUN_10038fe0` fills (0x10089a18).
//! * Recursive descent: `FUN_100396f0` (document), `FUN_10039670`/`FUN_10039350` (element and tag
//!   name), `FUN_10039240`/`FUN_10039620`/`FUN_10039550`/`FUN_10039430`/`FUN_10039280` (attributes),
//!   `FUN_100395a0` (end of the start tag), `FUN_100394e0`/`FUN_100393c0`/`FUN_100391f0`/`FUN_100396b0`
//!   (content), `FUN_10039470`/`FUN_10039300` (end tag).
//! * Builder callbacks (vtable 0x1005b3c8): `0x10033470` (start element), `0x10033620` (end element),
//!   `0x100336d0` (text), with `FUN_100332d0` (the four entities), `FUN_100333d0`/`FUN_10033400`
//!   (depth stack, 128 deep), `FUN_10033450` (open elements left over), `FUN_100024a0` (names narrowed).
//! * DOM nodes (magic 0x584d4c54, `FUN_10034350`; children appended in document order,
//!   `FUN_10033a00`) and the `IXmlPacket` accessors (vtable 0x1005c538).
//!
//! // open: the exact HRESULT of a malformed document. The descent functions' return values are lost in
//! the decompile (several are typed `void` but their callers test the result); every grammar violation
//! is `ZT_E_XML_SYNTAX_ERROR` here, as `FUN_100389d0` reports, and the explicit `return 0` paths
//! (`FUN_10039470` after a missing end-tag name) fall to `FUN_10033450`'s open-element check. A document
//! that ends inside a token is also a syntax error here (retail re-reads the last character).

use super::catalog::stricmp;
use crate::zt::*;
use crate::Hresult;

/// A node index into [`XmlPacket`]; `None` in the accessors means the root (`param_2 == 0`).
pub type NodeId = usize;

#[derive(Debug, Clone, Default)]
struct Node {
    /// `+4`: the element name, narrowed.
    name: String,
    /// `+8`: the last text callback's text (wide).
    text: Option<Vec<u16>>,
    /// `+0x10` / `+0x14`: attribute names (narrowed) and values (wide, entities decoded).
    attrs: Vec<(String, Vec<u16>)>,
    /// `+0x20` first child ... `+0x18` next sibling, flattened.
    children: Vec<NodeId>,
}

/// The parsed document (`IXmlPacket`, vtable 0x1005c538).
#[derive(Debug, Clone, Default)]
pub struct XmlPacket {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

// Token codes returned by FUN_10038ec0.
const T_NONE: u32 = 0;
const T_LT: u32 = 1;
const T_GT: u32 = 2;
const T_LT_SLASH: u32 = 3;
const T_SLASH_GT: u32 = 4;
const T_EQ: u32 = 5;
const T_QUOTE: u32 = 6;
const T_NAME: u32 = 7;
const T_VALUE: u32 = 8;
const T_TEXT: u32 = 9;
const T_EOF: u32 = 10;
const T_CONTINUE: u32 = 0xb;
const T_ERROR: u32 = 0xc;

/// `FUN_10038fe0`: the class table at 0x10089a18 (for characters below 0x80): 0 white space (space,
/// TAB, LF, CR), 1 structural (`< > = /`), 2 letter, 3 digit, 4 `"`, 5 `_`, 6 other, 7 NUL.
fn class(c: u16) -> u32 {
    match c {
        0 => 7,
        0x20 | 0x09 | 0x0a | 0x0d => 0,
        0x3c | 0x3e | 0x3d | 0x2f => 1,
        0x22 => 4,
        0x5f => 5,
        0x41..=0x5a | 0x61..=0x7a => 2,
        0x30..=0x39 => 3,
        _ => 6,
    }
}

struct Lexer<'a> {
    text: &'a [u16],
    /// `[0]`: state.
    state: u32,
    /// `[4]`: index of the current character (-1 before the first).
    idx: isize,
    /// `[2]` is the sentinel (the NUL after the end) rather than a character.
    at_end: bool,
    /// `[0xb]`, `[0xc]`: the token's start and length.
    tok_start: isize,
    tok_len: isize,
}

impl<'a> Lexer<'a> {
    fn cur(&self) -> u16 {
        if self.at_end || self.idx < 0 {
            0
        } else {
            self.text[self.idx as usize]
        }
    }

    /// The "unread" step of the handlers: `if (-1 < idx) { idx--; cur = base + idx }`.
    fn unread(&mut self) {
        if self.idx >= 0 {
            self.idx -= 1;
        }
        self.at_end = false;
    }

    /// `FUN_10038ec0`.
    fn next_token(&mut self) -> u32 {
        while !self.at_end {
            // advance: idx + 1 < len ? idx++ : cur = sentinel
            if self.idx + 1 < self.text.len() as isize {
                self.idx += 1;
                self.at_end = false;
            } else {
                self.at_end = true;
            }
            let c = self.cur();
            let code = match self.state {
                0 => self.skip_ws(2, c),
                1 => self.skip_ws(4, c),
                2 => self.state_content(c),
                3 => self.state_text(c),
                4 => self.state_tag(c),
                5 => self.state_value(c),
                6 => self.state_after_value(c),
                7 => self.state_after_lt(c),
                8 => self.state_after_slash(c),
                9 => self.state_name(c),
                _ => return T_ERROR,
            };
            if code != T_CONTINUE {
                return code;
            }
        }
        // the loop ended at the sentinel
        if self.state == 0 {
            self.state = 2;
            return T_NONE;
        }
        if self.state == 2 {
            return T_EOF;
        }
        self.state = 2;
        T_ERROR
    }

    /// `FUN_10038cf0(this, next_state)`: white space is skipped; the first other character is put
    /// back and the state set, token 0.
    fn skip_ws(&mut self, next_state: u32, c: u16) -> u32 {
        if c < 0x80 && class(c) == 0 {
            return T_CONTINUE;
        }
        if c != 0 {
            self.unread();
        }
        self.state = next_state;
        T_NONE
    }

    /// `FUN_10038dc0` (state 2, between tags).
    fn state_content(&mut self, c: u16) -> u32 {
        if c < 0x80 {
            if class(c) == 0 {
                self.state = 0;
                return T_CONTINUE;
            }
            if c == 0x3c {
                self.state = 7;
                return T_CONTINUE;
            }
        }
        self.tok_start = self.idx;
        self.state = 3;
        T_CONTINUE
    }

    /// `FUN_10038e10` (state 3, text).
    fn state_text(&mut self, c: u16) -> u32 {
        if c == 0x3c {
            self.tok_len = self.idx - self.tok_start;
            self.state = 7;
            return T_TEXT;
        }
        T_CONTINUE
    }

    /// `FUN_10038e40` (state 4, inside a tag).
    fn state_tag(&mut self, c: u16) -> u32 {
        if c < 0x80 {
            match class(c) {
                0 => {
                    self.state = 1;
                    return T_CONTINUE;
                }
                1 => {
                    // FUN_10038b50
                    return match c {
                        0x2f => {
                            self.state = 8;
                            T_CONTINUE
                        }
                        0x3c => {
                            self.state = 7;
                            T_CONTINUE
                        }
                        0x3d => T_EQ,
                        0x3e => {
                            self.state = 2;
                            T_GT
                        }
                        _ => T_ERROR,
                    };
                }
                2 => {
                    self.tok_start = self.idx;
                    self.state = 9;
                    return T_CONTINUE;
                }
                4 => {
                    self.tok_start = self.idx + 1;
                    self.state = 5;
                    return T_QUOTE;
                }
                7 => return T_EOF,
                _ => {}
            }
        }
        T_ERROR
    }

    /// `FUN_10038c50` (state 5, attribute value).
    fn state_value(&mut self, c: u16) -> u32 {
        if c == 0x22 {
            self.tok_len = self.idx - self.tok_start;
            self.state = 6;
            self.unread();
            return T_VALUE;
        }
        if c != 0x3c {
            return T_CONTINUE;
        }
        self.unread();
        T_ERROR
    }

    /// `FUN_10038cd0` (state 6): the closing quote.
    fn state_after_value(&mut self, c: u16) -> u32 {
        self.state = 4;
        if c == 0x22 {
            T_QUOTE
        } else {
            T_ERROR
        }
    }

    /// `FUN_10038bc0` (state 7, after `<`).
    fn state_after_lt(&mut self, c: u16) -> u32 {
        self.state = 4;
        if c == 0x2f {
            return T_LT_SLASH;
        }
        self.unread();
        T_LT
    }

    /// `FUN_10038c00` (state 8, after `/` inside a tag).
    fn state_after_slash(&mut self, c: u16) -> u32 {
        self.state = 2;
        if c == 0x3e {
            // `[0xb] = ptr + 1`: the token start moves past the `>` (the end-tag callback reads it)
            self.tok_start = self.idx + 1;
            return T_SLASH_GT;
        }
        self.unread();
        T_ERROR
    }

    /// `FUN_10038d50` (state 9, a name).
    fn state_name(&mut self, c: u16) -> u32 {
        if c > 0x7f {
            return T_ERROR;
        }
        let k = class(c);
        if k > 1 && (k < 4 || k == 5) {
            return T_CONTINUE;
        }
        self.tok_len = self.idx - self.tok_start;
        if self.at_end {
            // the "sentinel" is the static NUL at 0x10089e1c, not the text: the length is a pointer
            // difference between two unrelated addresses, far above 127 (a name token at the end of
            // the text; the end-tag callback then reports ZT_E_STR_TOO_LONG)
            self.tok_len = EOF_NAME_LEN;
        }
        self.unread();
        self.state = 4;
        T_NAME
    }
}

/// The four entities (table at 0x100680c8: name, length, replacement).
const ENTITIES: [(&str, u16); 4] = [
    ("quot;", 0x22),
    ("amp;", 0x26),
    ("lt;", 0x3c),
    ("gt;", 0x3e),
];

/// `FUN_100332d0(ptr, len, &newLen)` (0x100332d0): decodes `&quot; &amp; &lt; &gt;` in place (case
/// sensitive); any other `&` is `ZT_E_BAD_AMP_CODE`.
pub fn decode_entities(s: &[u16]) -> Result<Vec<u16>, Hresult> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0usize;
    while i < s.len() {
        if s[i] == 0x26 {
            let rest = &s[i + 1..];
            let found = ENTITIES.iter().find(|(name, _)| {
                let n: Vec<u16> = name.encode_utf16().collect();
                rest.len() >= n.len() && rest[..n.len()] == n[..]
            });
            match found {
                Some((name, repl)) => {
                    out.push(*repl);
                    i += name.len() + 1;
                }
                None => return Err(ZT_E_BAD_AMP_CODE),
            }
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    Ok(out)
}

/// `FUN_100024a0`: a wide name narrowed to the ANSI code page; 127 or more characters is
/// `ZT_E_STR_TOO_LONG`; a character that cannot be converted `ZT_E_INVALID_CHARACTER` (the lexer only
/// lets ASCII letters, digits and `_` into names, so that cannot occur).
fn narrow(name: &[u16], len: usize) -> Result<String, Hresult> {
    // the length is the token's own (a pointer difference at the end of the text), not the slice's
    if len + 1 >= 0x80 {
        return Err(ZT_E_STR_TOO_LONG);
    }
    let mut s = String::with_capacity(name.len());
    for &c in name {
        if c >= 0x80 {
            return Err(ZT_E_INVALID_CHARACTER);
        }
        s.push(c as u8 as char);
    }
    Ok(s)
}

/// An attribute name narrowed by the start-element callback 0x10033470, which hands `FUN_100024a0`
/// the TAG name's length for every attribute: the `len + 1 < 0x80` test therefore never fails, and the
/// real limit is `FUN_10002320`'s 0x80-byte `WideCharToMultiByte` buffer. A name of 127 characters (128
/// bytes with the NUL) fits; 128 or more fail with `ERROR_INSUFFICIENT_BUFFER` as `0x8007007A`
/// (`GetLastError() & 0xffff | 0x80070000`). [differential run of the DLL's own code, final messages
/// audit N1]
fn narrow_attribute(name: &[u16]) -> Result<String, Hresult> {
    if name.len() >= 0x80 {
        return Err(Hresult(0x8007_007A));
    }
    narrow(name, 0)
}

struct Parser<'a> {
    lx: Lexer<'a>,
    /// `[9]` (0x24): the current token.
    tok: u32,
    /// `[0x438]`, `[0x43c]`: the start tag's name; `[0x434]` attribute entries.
    tag_name: (isize, isize),
    attr_names: Vec<(isize, isize)>,
    attr_values: Vec<(isize, isize)>,
    /// The builder (`FUN_10033710`'s packet, `FUN_100332c0`).
    pkt: XmlPacket,
    cur: Option<NodeId>,
    /// The parent links (`+0x24`) of the open nodes.
    parents: Vec<Option<NodeId>>,
    /// `FUN_100333d0`'s stack of start names.
    stack: Vec<String>,
}

const SYNTAX: Hresult = ZT_E_XML_SYNTAX_ERROR;

/// A name token read at the end of the text has a length that is a difference of two unrelated
/// pointers (`FUN_10038d50`): large. Its exact value is not reproducible; anything over 126 gives the
/// same `ZT_E_STR_TOO_LONG` in `FUN_100024a0`.
const EOF_NAME_LEN: isize = 0x4000_0000;

impl<'a> Parser<'a> {
    fn next(&mut self) -> u32 {
        self.tok = self.lx.next_token();
        self.tok
    }

    fn slice(&self, (start, len): (isize, isize)) -> &'a [u16] {
        let s = start.max(0) as usize;
        let e = (s + len.max(0) as usize).min(self.lx.text.len());
        &self.lx.text[s.min(e)..e]
    }

    /// `FUN_100396f0`.
    fn document(&mut self) -> Result<(), Hresult> {
        let mut t = self.next();
        if t == T_NONE {
            t = self.next();
        }
        if t == T_LT {
            self.element()
        } else {
            Err(SYNTAX)
        }
    }

    /// `FUN_10039670(this, 1)`.
    fn element(&mut self) -> Result<(), Hresult> {
        if self.tok != T_LT {
            return Err(SYNTAX);
        }
        self.start_tag_name()?;
        self.end_of_start_tag()
    }

    /// `FUN_10039350`: `<` then the name, then the attributes.
    fn start_tag_name(&mut self) -> Result<(), Hresult> {
        if self.tok != T_LT {
            return Err(SYNTAX);
        }
        if self.next() != T_NAME {
            return Err(SYNTAX);
        }
        self.tag_name = (self.lx.tok_start, self.lx.tok_len);
        self.attr_names.clear();
        self.attr_values.clear();
        self.next();
        self.attributes(self.tok)
    }

    /// `FUN_10039240`.
    fn attributes(&mut self, tok: u32) -> Result<(), Hresult> {
        if tok == T_NONE {
            let t = self.next();
            return self.attribute_loop(t);
        }
        if tok != T_GT && tok != T_SLASH_GT {
            return Err(SYNTAX);
        }
        Ok(())
    }

    /// `FUN_10039620`.
    fn attribute_loop(&mut self, tok: u32) -> Result<(), Hresult> {
        if tok != T_GT && tok != T_SLASH_GT {
            if tok != T_NAME {
                return Err(SYNTAX);
            }
            self.one_attribute()?;
            return self.attributes(self.tok);
        }
        Ok(())
    }

    /// `FUN_10039550` (the name), `FUN_10039430` (`=`), `FUN_10039280` (the quoted value).
    fn one_attribute(&mut self) -> Result<(), Hresult> {
        // FUN_100390e0 / FUN_10039130: names and values share one entry counter that fails above
        // 0x7d, so the 64th attribute's name is `ZT_E_TOO_MANY_ATTRIBUTES`
        if 2 * self.attr_names.len() > 0x7d {
            return Err(ZT_E_TOO_MANY_ATTRIBUTES);
        }
        let name = (self.lx.tok_start, self.lx.tok_len);
        let mut t = self.next();
        if t == T_NONE {
            t = self.next();
        }
        if t != T_EQ {
            return Err(SYNTAX);
        }
        let mut t = self.next();
        if t == T_NONE {
            t = self.next();
        }
        if t != T_QUOTE || self.next() != T_VALUE {
            return Err(SYNTAX);
        }
        let value = (self.lx.tok_start, self.lx.tok_len);
        self.attr_names.push(name);
        self.attr_values.push(value);
        if self.next() != T_QUOTE {
            return Err(SYNTAX);
        }
        self.next();
        Ok(())
    }

    /// `FUN_100395a0`.
    fn end_of_start_tag(&mut self) -> Result<(), Hresult> {
        match self.tok {
            T_GT => {
                self.next();
                self.start_element()?;
                self.content(self.tok)?;
                self.end_tag(self.tok)
            }
            T_SLASH_GT => {
                self.next();
                self.start_element()?;
                // builder vtable +4 with (0, 0)
                let _ = self.end_element(None);
                Ok(())
            }
            _ => Err(SYNTAX),
        }
    }

    /// `FUN_100394e0`.
    fn content(&mut self, tok: u32) -> Result<(), Hresult> {
        let mut t = tok;
        if t == T_NONE {
            t = self.next();
        }
        match t {
            T_LT | T_LT_SLASH | T_TEXT => self.content_item(t),
            _ => Err(SYNTAX),
        }
    }

    /// `FUN_100393c0`.
    fn content_item(&mut self, tok: u32) -> Result<(), Hresult> {
        match tok {
            T_LT => {
                self.element()?;
                self.after_child(self.tok)
            }
            T_LT_SLASH => Ok(()),
            T_TEXT => {
                let text = (self.lx.tok_start, self.lx.tok_len);
                self.next();
                // FUN_100393c0 discards the callback's HRESULT (`mov eax, edi` with edi = 0): a bad
                // entity leaves the node's text unset and the document parses
                let _ = self.text(self.slice(text));
                Ok(())
            }
            _ => Err(SYNTAX),
        }
    }

    /// `FUN_100391f0` / `FUN_100396b0`: after a child element, more children or the end tag.
    fn after_child(&mut self, tok: u32) -> Result<(), Hresult> {
        let mut t = tok;
        if t == T_NONE {
            t = self.next();
        }
        match t {
            T_LT => {
                self.element()?;
                self.after_child(self.tok)
            }
            T_LT_SLASH => Ok(()),
            _ => Err(SYNTAX),
        }
    }

    /// `FUN_10039470` and `FUN_10039300`. The end callback runs AFTER the token following the name
    /// was read and uses the lexer's current token span (`[0x2c]`, `[0x30]`): for `</a b>` that is
    /// still `a` (the token read is white space), for `</a/>` the span has moved past the `>`.
    fn end_tag(&mut self, tok: u32) -> Result<(), Hresult> {
        if tok != T_LT_SLASH {
            return Err(SYNTAX);
        }
        if self.next() != T_NAME {
            // retail records the syntax error but returns 0; the open element is then caught by
            // FUN_10033450 (ZT_E_MISMATCHED_XML_TAGS)
            return Ok(());
        }
        let mut t = self.next();
        let name = (self.lx.tok_start, self.lx.tok_len);
        self.end_element(Some(name))?;
        if t == T_NONE {
            t = self.next();
        }
        if t == T_GT {
            self.next();
            return Ok(());
        }
        Err(SYNTAX)
    }

    // ---- the builder (vtable 0x1005b3c8) ----

    /// `0x10033470`: names narrowed, attribute values entity-decoded, depth pushed, node created.
    fn start_element(&mut self) -> Result<(), Hresult> {
        let name = narrow(self.slice(self.tag_name), self.tag_name.1.max(0) as usize)?;
        let mut attrs = Vec::new();
        for (n, v) in self.attr_names.iter().zip(&self.attr_values) {
            let an = narrow_attribute(self.slice(*n))?;
            let av = decode_entities(self.slice(*v))?;
            attrs.push((an, av));
        }
        self.attr_names.clear();
        self.attr_values.clear();
        // FUN_100333d0
        if self.stack.len() > 0x7f {
            return Err(ZT_E_NESTING_TOO_DEEP);
        }
        self.stack.push(name.clone());
        // FUN_10034770 / FUN_10034350: the node, attached to the current one
        let id = self.pkt.nodes.len();
        self.pkt.nodes.push(Node {
            name,
            attrs,
            ..Default::default()
        });
        match self.cur {
            Some(parent) => self.pkt.nodes[parent].children.push(id),
            None => {
                if self.pkt.root.is_some() {
                    return Err(Hresult(0x8007_0057));
                }
                self.pkt.root = Some(id);
            }
        }
        self.parents.push(self.cur);
        self.cur = Some(id);
        Ok(())
    }

    /// `0x10033620`: `None` is the `/>` form.
    fn end_element(&mut self, name: Option<(isize, isize)>) -> Result<(), Hresult> {
        // FUN_10033400: pop the depth stack
        let Some(start_name) = self.stack.pop() else {
            return Err(ZT_E_MISMATCHED_XML_TAGS);
        };
        if let Some(n) = name {
            let end_name = narrow(self.slice(n), n.1.max(0) as usize)?;
            // a case-sensitive byte comparison
            if end_name != start_name {
                return Err(ZT_E_MISMATCHED_XML_TAGS);
            }
        }
        // FUN_10033d80: the current node becomes its parent
        self.cur = self.parents.pop().flatten();
        Ok(())
    }

    /// `0x100336d0`: entity-decoded text stored on the current node (`FUN_10033da0`).
    fn text(&mut self, t: &[u16]) -> Result<(), Hresult> {
        let decoded = decode_entities(t)?;
        if let Some(cur) = self.cur {
            self.pkt.nodes[cur].text = Some(decoded);
        }
        Ok(())
    }
}

impl XmlPacket {
    /// `FUN_10033710(text, len, 1, &packet, ..)` (0x10033710): parse, then `FUN_10033450` requires
    /// every opened element to have been closed (`ZT_E_MISMATCHED_XML_TAGS`).
    pub fn parse(text: &[u16]) -> Result<XmlPacket, Hresult> {
        // FUN_10038a30: a zero length is a syntax error
        if text.is_empty() {
            return Err(SYNTAX);
        }
        let mut p = Parser {
            lx: Lexer {
                text,
                state: 2,
                idx: -1,
                at_end: false,
                tok_start: 0,
                tok_len: 0,
            },
            tok: 0,
            tag_name: (0, 0),
            attr_names: Vec::new(),
            attr_values: Vec::new(),
            pkt: XmlPacket::default(),
            cur: None,
            parents: Vec::new(),
            stack: Vec::new(),
        };
        p.document()?;
        if !p.stack.is_empty() {
            return Err(ZT_E_MISMATCHED_XML_TAGS);
        }
        if p.pkt.root.is_none() {
            return Err(SYNTAX);
        }
        Ok(p.pkt)
    }

    fn node(&self, node: Option<NodeId>) -> &Node {
        &self.nodes[node.or(self.root).unwrap_or(0)]
    }

    /// Slot 0x00 `GetName` (0x100343d0).
    pub fn name(&self, node: Option<NodeId>) -> &str {
        &self.node(node).name
    }

    /// Slot 0x04 `GetAttrCount` (0x10034580).
    pub fn attr_count(&self, node: Option<NodeId>) -> usize {
        self.node(node).attrs.len()
    }

    /// Slot 0x08 `GetAttrByIndex` (0x100345b0): `ZT_E_INDEX_TOO_GREAT` past the count.
    pub fn attr_by_index(
        &self,
        node: Option<NodeId>,
        idx: usize,
    ) -> Result<(&str, &[u16]), Hresult> {
        self.node(node)
            .attrs
            .get(idx)
            .map(|(n, v)| (n.as_str(), v.as_slice()))
            .ok_or(ZT_E_INDEX_TOO_GREAT)
    }

    /// Slot 0x10 `GetAttrString` (0x10034400, `FUN_10033a50`): the first attribute whose name
    /// matches `_stricmp`; `ZT_E_ATTR_NOT_FOUND` when none does. The value is the wide string up to
    /// its NUL.
    pub fn attr_string(&self, node: Option<NodeId>, name: &str) -> Result<&[u16], Hresult> {
        self.node(node)
            .attrs
            .iter()
            .find(|(n, _)| stricmp(n, name))
            .map(|(_, v)| {
                let end = v.iter().position(|&c| c == 0).unwrap_or(v.len());
                &v[..end]
            })
            .ok_or(ZT_E_ATTR_NOT_FOUND)
    }

    /// Slot 0x14 `GetAttrDecimal` (0x10033c20): slot 0x10, then `FUN_10002a00`.
    pub fn attr_decimal(&self, node: Option<NodeId>, name: &str) -> Result<i32, Hresult> {
        parse_decimal(self.attr_string(node, name)?)
    }

    /// Slots 0x1c / 0x20 `GetAttrHex` (0x10033cb0, 0x10033cf0): slot 0x10, then `FUN_10002b20`.
    pub fn attr_hex(&self, node: Option<NodeId>, name: &str) -> Result<u32, Hresult> {
        parse_hex(self.attr_string(node, name)?)
    }

    /// Slot 0x24 (0x10034460): the node's text.
    pub fn text(&self, node: Option<NodeId>) -> Option<&[u16]> {
        self.node(node).text.as_deref()
    }

    /// Slot 0x28 `GetChildCount` (0x10034490).
    pub fn child_count(&self, node: Option<NodeId>) -> usize {
        self.node(node).children.len()
    }

    /// Slot 0x2c `GetChild` (0x100344c0): `ZT_E_INDEX_TOO_GREAT` without that many children.
    pub fn child(&self, node: Option<NodeId>, idx: usize) -> Result<NodeId, Hresult> {
        self.node(node)
            .children
            .get(idx)
            .copied()
            .ok_or(ZT_E_INDEX_TOO_GREAT)
    }

    /// Slot 0x30 `FindChildByName` (0x10034520, `FUN_10033b10`): `_stricmp` over the children;
    /// `ZT_E_NOT_FOUND` (0xc1121c04) when none matches.
    pub fn find_child(&self, node: Option<NodeId>, name: &str) -> Result<NodeId, Hresult> {
        self.node(node)
            .children
            .iter()
            .copied()
            .find(|&c| stricmp(&self.nodes[c].name, name))
            .ok_or(ZT_E_NOT_FOUND)
    }
}

/// `FUN_10002a00` (0x10002a00): digits, and one `-` anywhere (a second one, `+`, a space or any other
/// character is `ZT_E_INVALID_CHARACTER`). At a `-` the accumulator still does
/// `acc = prev_digit + acc * 10`, so `"5-"` is -55; no overflow check; an empty string is 0.
pub fn parse_decimal(s: &[u16]) -> Result<i32, Hresult> {
    let mut acc: i32 = 0;
    let mut digit: i32 = 0;
    let mut neg = false;
    for &c in s {
        if c == 0x2d {
            if neg {
                return Err(ZT_E_INVALID_CHARACTER);
            }
            neg = true;
        } else {
            if !(0x30..=0x39).contains(&c) {
                return Err(ZT_E_INVALID_CHARACTER);
            }
            digit = (c - 0x30) as i32;
        }
        acc = digit.wrapping_add(acc.wrapping_mul(10));
    }
    Ok(if neg { acc.wrapping_neg() } else { acc })
}

/// `FUN_10002b20` (0x10002b20): `0-9` is c - 0x30, both `a-f` and `A-F` are c - 0x57 (so upper case
/// gives wrong values), `x` is skipped but the accumulator still adds the previous digit, any other
/// character is `ZT_E_INVALID_CHARACTER`; 32-bit wrap.
pub fn parse_hex(s: &[u16]) -> Result<u32, Hresult> {
    let mut acc: i32 = 0;
    let mut digit: i32 = 0;
    for &c in s {
        match c {
            0x30..=0x39 => digit = (c - 0x30) as i32,
            0x41..=0x46 | 0x61..=0x66 => digit = c as i32 - 0x57,
            0x78 => {}
            _ => return Err(ZT_E_INVALID_CHARACTER),
        }
        acc = acc.wrapping_mul(16).wrapping_add(digit);
    }
    Ok(acc as u32)
}
