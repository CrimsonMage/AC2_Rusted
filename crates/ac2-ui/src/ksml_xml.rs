//! The XML subset KSML pages use: elements, quoted attributes, character data with the five
//! predefined entities and numeric character references, comments, processing instructions and
//! a DOCTYPE (skipped). Every end tag is checked against the open element, so a malformed page
//! fails here rather than producing a truncated tree.

use std::fmt;

/// An element with its attributes (in document order) and its children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<XmlNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlNode {
    Element(XmlElement),
    /// Character data with entities decoded.
    Text(String),
}

impl XmlElement {
    /// The value of attribute `name`, the first one if it is repeated.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The child elements, in order.
    pub fn elements(&self) -> impl Iterator<Item = &XmlElement> {
        self.children.iter().filter_map(|child| match child {
            XmlNode::Element(element) => Some(element),
            XmlNode::Text(_) => None,
        })
    }
}

/// A well-formedness failure, with the byte offset where it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlError {
    pub offset: usize,
    pub reason: String,
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.reason, self.offset)
    }
}

impl std::error::Error for XmlError {}

/// Parses one document and returns its root element.
pub fn parse_xml(text: &str) -> Result<XmlElement, XmlError> {
    let mut parser = Parser {
        text,
        pos: 0,
        open: Vec::new(),
        root: None,
    };
    parser.run()?;
    match parser.root.take() {
        Some(root) => Ok(root),
        None => Err(parser.error("no root element")),
    }
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
    open: Vec<XmlElement>,
    root: Option<XmlElement>,
}

impl Parser<'_> {
    fn error(&self, reason: &str) -> XmlError {
        XmlError {
            offset: self.pos,
            reason: reason.to_string(),
        }
    }

    fn rest(&self) -> &str {
        &self.text[self.pos..]
    }

    /// Consumes through `end` (inclusive), returning the text before it.
    fn take_through(&mut self, end: &str, what: &str) -> Result<String, XmlError> {
        let rest = self.rest();
        let Some(at) = rest.find(end) else {
            return Err(self.error(what));
        };
        let body = rest[..at].to_string();
        self.pos += at + end.len();
        Ok(body)
    }

    fn run(&mut self) -> Result<(), XmlError> {
        while self.pos < self.text.len() {
            if self.rest().starts_with("<!--") {
                self.pos += 4;
                self.take_through("-->", "unterminated comment")?;
            } else if self.rest().starts_with("<?") {
                self.pos += 2;
                self.take_through("?>", "unterminated processing instruction")?;
            } else if self.rest().starts_with("<!") {
                self.pos += 2;
                self.take_through(">", "unterminated declaration")?;
            } else if self.rest().starts_with("</") {
                self.pos += 2;
                self.end_tag()?;
            } else if self.rest().starts_with('<') {
                self.pos += 1;
                self.start_tag()?;
            } else {
                self.character_data()?;
            }
        }
        if let Some(open) = self.open.last() {
            return Err(XmlError {
                offset: self.text.len(),
                reason: format!("element <{}> is never closed", open.name),
            });
        }
        Ok(())
    }

    fn name(&mut self) -> Result<String, XmlError> {
        let len = self
            .rest()
            .bytes()
            .take_while(|b| !b.is_ascii_whitespace() && !matches!(b, b'/' | b'>' | b'=' | b'<'))
            .count();
        if len == 0 {
            return Err(self.error("expected a name"));
        }
        let name = self.rest()[..len].to_string();
        self.pos += len;
        Ok(name)
    }

    fn skip_space(&mut self) {
        let len = self
            .rest()
            .bytes()
            .take_while(|b| b.is_ascii_whitespace())
            .count();
        self.pos += len;
    }

    fn start_tag(&mut self) -> Result<(), XmlError> {
        let name = self.name()?;
        let mut attributes = Vec::new();
        let self_closing = loop {
            self.skip_space();
            if self.rest().starts_with("/>") {
                self.pos += 2;
                break true;
            }
            if self.rest().starts_with('>') {
                self.pos += 1;
                break false;
            }
            let key = self.name()?;
            self.skip_space();
            if !self.rest().starts_with('=') {
                return Err(self.error("expected '=' after an attribute name"));
            }
            self.pos += 1;
            self.skip_space();
            let quote = match self.rest().as_bytes().first() {
                Some(&q) if q == b'"' || q == b'\'' => q as char,
                _ => return Err(self.error("attribute value must be quoted")),
            };
            self.pos += 1;
            let raw = self.take_through(&quote.to_string(), "unterminated attribute value")?;
            attributes.push((key, decode_entities(&raw)));
        };
        let element = XmlElement {
            name,
            attributes,
            children: Vec::new(),
        };
        if self_closing {
            self.attach(element);
        } else {
            self.open.push(element);
        }
        Ok(())
    }

    fn end_tag(&mut self) -> Result<(), XmlError> {
        let name = self.name()?;
        self.skip_space();
        if !self.rest().starts_with('>') {
            return Err(self.error("expected '>' after an end tag name"));
        }
        self.pos += 1;
        match self.open.pop() {
            Some(element) if element.name == name => {
                self.attach(element);
                Ok(())
            }
            Some(element) => Err(XmlError {
                offset: self.pos,
                reason: format!("end tag </{name}> closes <{}>", element.name),
            }),
            None => Err(self.error("end tag with no open element")),
        }
    }

    fn character_data(&mut self) -> Result<(), XmlError> {
        let len = self.rest().find('<').unwrap_or(self.rest().len());
        let raw = &self.rest()[..len];
        let text = decode_entities(raw);
        self.pos += len;
        match self.open.last_mut() {
            Some(parent) => parent.children.push(XmlNode::Text(text)),
            None if text.trim_matches(is_xml_space).is_empty() => {}
            None => return Err(self.error("character data outside the root element")),
        }
        Ok(())
    }

    /// Hands a finished element to its parent, or makes it the root.
    fn attach(&mut self, element: XmlElement) {
        match self.open.last_mut() {
            Some(parent) => parent.children.push(XmlNode::Element(element)),
            None if self.root.is_none() => self.root = Some(element),
            None => {
                // A second top-level element: keep the first root and drop the rest, which the
                // root check in `parse_xml` would otherwise miss.
                self.root = Some(element);
                self.pos = self.text.len();
            }
        }
    }
}

fn is_xml_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

/// Replaces `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;` and `&#n;` / `&#xn;`. An unknown or
/// unterminated reference stays as written, which is how the shipped pages read it.
pub fn decode_entities(raw: &str) -> String {
    if !raw.contains('&') {
        return raw.to_string();
    }
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        let decoded = after
            .find(';')
            .filter(|end| *end <= 10)
            .and_then(|end| decode_reference(&after[..end]).map(|c| (c, end + 1)));
        match decoded {
            Some((c, used)) => {
                out.push(c);
                rest = &after[used..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn decode_reference(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let digits = name.strip_prefix('#')?;
            let value = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => digits.parse::<u32>().ok()?,
            };
            char::from_u32(value)
        }
    }
}
