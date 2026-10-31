//! The wire: one XML element per frame behind a big-endian `u32` length (AC2e's
//! `docs/gun-protocol.md`). Elements are flat: a name, ordered attributes and one level of
//! children; no declaration, comments or text.

const MAX_DEPTH: usize = 4;
const MAX_CHILDREN: usize = 256;
const MAX_ATTRS: usize = 16;
/// Largest frame payload either side sends.
pub const MAX_FRAME: usize = 65536;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Elem {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Elem>,
}

impl Elem {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..Self::default()
        }
    }

    pub fn attr(mut self, key: &str, value: impl Into<String>) -> Self {
        self.attrs.push((key.to_owned(), value.into()));
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn encode(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.name);
        for (key, value) in &self.attrs {
            out.push(' ');
            out.push_str(key);
            out.push_str("=\"");
            for c in value.chars() {
                match c {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '"' => out.push_str("&quot;"),
                    c if (c as u32) < 0x20 => out.push_str(&format!("&#{};", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        if self.children.is_empty() {
            out.push_str("/>");
        } else {
            out.push('>');
            for child in &self.children {
                child.write(out);
            }
            out.push_str("</");
            out.push_str(&self.name);
            out.push('>');
        }
    }

    pub fn parse(text: &str) -> Result<Elem, &'static str> {
        let mut parser = Parser {
            bytes: text.as_bytes(),
            at: 0,
        };
        parser.space();
        let elem = parser.element(0)?;
        parser.space();
        if parser.at != parser.bytes.len() {
            return Err("trailing data");
        }
        Ok(elem)
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }

    fn name(&mut self) -> Result<String, &'static str> {
        let start = self.at;
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        {
            self.at += 1;
        }
        if self.at == start || self.at - start > 64 {
            return Err("bad name");
        }
        Ok(String::from_utf8_lossy(&self.bytes[start..self.at]).into_owned())
    }

    fn expect(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.bytes.get(self.at) == Some(&byte) {
            self.at += 1;
            Ok(())
        } else {
            Err("unexpected byte")
        }
    }

    fn element(&mut self, depth: usize) -> Result<Elem, &'static str> {
        if depth > MAX_DEPTH {
            return Err("too deep");
        }
        self.expect(b'<')?;
        let mut elem = Elem::new(&self.name()?);
        loop {
            self.space();
            match self.bytes.get(self.at) {
                Some(b'/') => {
                    self.at += 1;
                    self.expect(b'>')?;
                    return Ok(elem);
                }
                Some(b'>') => {
                    self.at += 1;
                    break;
                }
                Some(_) => {
                    if elem.attrs.len() >= MAX_ATTRS {
                        return Err("too many attributes");
                    }
                    let key = self.name()?;
                    self.space();
                    self.expect(b'=')?;
                    self.space();
                    let value = self.value()?;
                    elem.attrs.push((key, value));
                }
                None => return Err("truncated"),
            }
        }
        loop {
            self.space();
            if self.bytes.get(self.at) == Some(&b'<') && self.bytes.get(self.at + 1) == Some(&b'/')
            {
                self.at += 2;
                let close = self.name()?;
                self.space();
                self.expect(b'>')?;
                return if close == elem.name {
                    Ok(elem)
                } else {
                    Err("mismatched close")
                };
            }
            if elem.children.len() >= MAX_CHILDREN {
                return Err("too many children");
            }
            elem.children.push(self.element(depth + 1)?);
        }
    }

    fn value(&mut self) -> Result<String, &'static str> {
        let quote = *self.bytes.get(self.at).ok_or("truncated")?;
        if quote != b'"' && quote != b'\'' {
            return Err("unquoted value");
        }
        self.at += 1;
        let mut out = Vec::new();
        loop {
            let byte = *self.bytes.get(self.at).ok_or("truncated")?;
            self.at += 1;
            if byte == quote {
                break;
            }
            if byte == b'<' {
                return Err("bad value");
            }
            if byte != b'&' {
                out.push(byte);
                continue;
            }
            let end = self.bytes[self.at..]
                .iter()
                .position(|&b| b == b';')
                .ok_or("bad entity")?;
            let entity = &self.bytes[self.at..self.at + end];
            self.at += end + 1;
            let number = |digits: &[u8], radix: u32| {
                std::str::from_utf8(digits)
                    .ok()
                    .and_then(|d| u32::from_str_radix(d, radix).ok())
                    .and_then(char::from_u32)
                    .ok_or("bad entity")
            };
            let ch = match entity {
                b"amp" => '&',
                b"lt" => '<',
                b"gt" => '>',
                b"quot" => '"',
                b"apos" => '\'',
                [b'#', b'x', rest @ ..] => number(rest, 16)?,
                [b'#', rest @ ..] => number(rest, 10)?,
                _ => return Err("bad entity"),
            };
            let mut buffer = [0u8; 4];
            out.extend_from_slice(ch.encode_utf8(&mut buffer).as_bytes());
        }
        String::from_utf8(out).map_err(|_| "not UTF-8")
    }
}

/// One frame: a big-endian `u32` payload length, then the element's text.
pub fn frame(elem: &Elem) -> Vec<u8> {
    let text = elem.encode();
    let mut out = Vec::with_capacity(4 + text.len());
    out.extend_from_slice(&(text.len() as u32).to_be_bytes());
    out.extend_from_slice(text.as_bytes());
    out
}

/// Splits a byte stream into frames.
#[derive(Debug, Default)]
pub struct FrameReader {
    buffer: Vec<u8>,
}

impl FrameReader {
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// The next complete element; `Ok(None)` when more bytes are needed.
    pub fn next_element(&mut self) -> Result<Option<Elem>, &'static str> {
        if self.buffer.len() < 4 {
            return Ok(None);
        }
        let length = u32::from_be_bytes([
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
        ]) as usize;
        if length == 0 || length > MAX_FRAME {
            return Err("bad frame length");
        }
        if self.buffer.len() < 4 + length {
            return Ok(None);
        }
        let payload: Vec<u8> = self.buffer.drain(..4 + length).skip(4).collect();
        let text = std::str::from_utf8(&payload).map_err(|_| "not UTF-8")?;
        Elem::parse(text).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_is_a_big_endian_length_and_the_element() {
        let elem = Elem::new("FriendAdd")
            .attr("Name", "Bob & \"Al\"@AC2e")
            .attr("seq", "3");
        let text = r#"<FriendAdd Name="Bob &amp; &quot;Al&quot;@AC2e" seq="3"/>"#;
        let bytes = frame(&elem);
        assert_eq!(&bytes[..4], &(text.len() as u32).to_be_bytes());
        assert_eq!(&bytes[4..], text.as_bytes());
        assert_eq!(Elem::parse(text).unwrap(), elem);
    }

    #[test]
    fn the_documented_golden_frame() {
        // AC2e's docs/gun-protocol.md: length 0x24, then the element.
        let elem = Elem::new("FriendAdd")
            .attr("Name", "Bob@AC2e")
            .attr("seq", "3");
        let mut expected = vec![0, 0, 0, 0x24];
        expected.extend_from_slice(br#"<FriendAdd Name="Bob@AC2e" seq="3"/>"#);
        assert_eq!(frame(&elem), expected);
    }

    #[test]
    fn children_control_characters_and_numeric_entities_round_trip() {
        let mut list = Elem::new("GetFriendsListRes").attr("hr", "0x00000000");
        list.children.push(
            Elem::new("Friend")
                .attr("Name", "a\nb<c>")
                .attr("Mode", "9"),
        );
        let text = list.encode();
        assert!(text.contains("a&#10;b&lt;c&gt;"));
        assert_eq!(Elem::parse(&text).unwrap(), list);
        assert_eq!(
            Elem::parse(r#"<A b="&#x41;&#66;"/>"#).unwrap().get("b"),
            Some("AB")
        );
    }

    #[test]
    fn malformed_elements_are_refused() {
        for bad in [
            "",
            "<A",
            "<A b=1/>",
            "<A></B>",
            "<A/><B/>",
            "<A b=\"&bogus;\"/>",
            "text",
            "<A><B><C><D><E><F/></E></D></C></B></A>",
        ] {
            assert!(Elem::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_reader_splits_a_stream_and_rejects_bad_lengths() {
        let mut stream = frame(&Elem::new("A").attr("x", "1"));
        stream.extend(frame(&Elem::new("B")));
        let mut reader = FrameReader::default();
        reader.push(&stream[..3]);
        assert_eq!(reader.next_element(), Ok(None));
        reader.push(&stream[3..10]);
        assert_eq!(reader.next_element(), Ok(None));
        reader.push(&stream[10..]);
        assert_eq!(reader.next_element().unwrap().unwrap().name, "A");
        assert_eq!(reader.next_element().unwrap().unwrap().name, "B");
        assert_eq!(reader.next_element(), Ok(None));
        let mut reader = FrameReader::default();
        reader.push(&[0, 1, 0, 1]);
        assert!(
            reader.next_element().is_err(),
            "65537 bytes is over the limit"
        );
    }
}
