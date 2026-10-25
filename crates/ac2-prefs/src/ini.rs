//! The Win32 INI file the preferences live in (`GetPrivateProfileStringA` and
//! `WritePrivateProfileStringA`, which `Preferences::LoadValue_Internal` and `Preference::Save`
//! call). The file is kept as read: each line keeps its bytes and its line ending, so a file that
//! is read and written back, or written with the same values, is identical byte for byte. Text is
//! ANSI: one byte per `char` (Latin-1), as the Win32 calls see it.
//!
//! As traced: a section is `[name]`, a key is `key=value` under the section header, and the
//! names match case-insensitively. A read trims the spaces around the key and the value, skips
//! `;` comment lines, and returns the first match. A write replaces the key's line in place, or
//! inserts `key=value` before the next section header (or at the end of the file), and appends
//! `[section]` at the end when the section is missing. New lines end in CRLF. Not verified against
//! Windows: the exact place a new key takes inside a section, and whether a replaced key keeps
//! the spelling it had in the file.

/// One line of the file: its text without the line ending, and the line ending (empty for a last
/// line that has none).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    text: Vec<u8>,
    eol: &'static [u8],
}

const CRLF: &[u8] = b"\r\n";

/// An INI file, held as its lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IniFile {
    lines: Vec<Line>,
}

/// Latin-1 text of `bytes` (one `char` per byte).
pub fn text_of(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// The bytes of `text` (each `char` below 256 as one byte; others as `?`).
pub fn bytes_of(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
        .collect()
}

fn trim(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .map_or(start, |i| i + 1);
    &bytes[start..end]
}

/// The section a header line names, when it is one.
fn header_name(text: &[u8]) -> Option<&[u8]> {
    let line = trim(text);
    if line.first() != Some(&b'[') {
        return None;
    }
    let close = line.iter().position(|&b| b == b']')?;
    Some(trim(&line[1..close]))
}

/// The key and value of a `key=value` line (`None` for a comment, a header or a line without `=`).
fn key_value(text: &[u8]) -> Option<(&[u8], &[u8])> {
    let line = trim(text);
    if line.is_empty() || line[0] == b';' || line[0] == b'[' {
        return None;
    }
    let eq = text.iter().position(|&b| b == b'=')?;
    Some((trim(&text[..eq]), trim(&text[eq + 1..])))
}

impl IniFile {
    /// The file's lines, split at `\n` (a `\r` before it is part of the line ending).
    pub fn parse(bytes: &[u8]) -> Self {
        let mut lines = Vec::new();
        let mut rest = bytes;
        while !rest.is_empty() {
            match rest.iter().position(|&b| b == b'\n') {
                Some(i) => {
                    let (text, eol) = if i > 0 && rest[i - 1] == b'\r' {
                        (&rest[..i - 1], CRLF)
                    } else {
                        (&rest[..i], &b"\n"[..])
                    };
                    lines.push(Line {
                        text: text.to_vec(),
                        eol: if eol == CRLF { CRLF } else { b"\n" },
                    });
                    rest = &rest[i + 1..];
                }
                None => {
                    lines.push(Line {
                        text: rest.to_vec(),
                        eol: b"",
                    });
                    rest = &[];
                }
            }
        }
        Self { lines }
    }

    /// The file's bytes, as they were read with the written changes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for line in &self.lines {
            out.extend_from_slice(&line.text);
            out.extend_from_slice(line.eol);
        }
        out
    }

    /// The first `key` of `section`'s value (`GetPrivateProfileStringA`), or `None` when the
    /// section or the key is missing (the caller's default then applies).
    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        let (start, end) = self.section_span(section.as_bytes())?;
        self.lines[start..end].iter().find_map(|line| {
            let (k, v) = key_value(&line.text)?;
            k.eq_ignore_ascii_case(key.as_bytes()).then(|| text_of(v))
        })
    }

    /// `WritePrivateProfileStringA`: the key's value under `section`, replaced or added.
    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        let line = Line {
            text: [key.as_bytes(), b"=", &bytes_of(value)].concat(),
            eol: CRLF,
        };
        match self.section_span(section.as_bytes()) {
            Some((start, end)) => {
                let found = self.lines[start..end].iter().position(|l| {
                    key_value(&l.text).is_some_and(|(k, _)| k.eq_ignore_ascii_case(key.as_bytes()))
                });
                match found {
                    Some(i) => self.lines[start + i].text = line.text,
                    None => {
                        self.terminate_last();
                        self.lines.insert(end, line);
                    }
                }
            }
            None => {
                self.terminate_last();
                self.lines.push(Line {
                    text: [b"[", section.as_bytes(), b"]"].concat(),
                    eol: CRLF,
                });
                self.lines.push(line);
            }
        }
    }

    /// The line span (header excluded) of the first `section` and the index of the next header.
    fn section_span(&self, section: &[u8]) -> Option<(usize, usize)> {
        let header = self
            .lines
            .iter()
            .position(|l| header_name(&l.text).is_some_and(|n| n.eq_ignore_ascii_case(section)))?;
        let end = self.lines[header + 1..]
            .iter()
            .position(|l| header_name(&l.text).is_some())
            .map_or(self.lines.len(), |i| header + 1 + i);
        Some((header + 1, end))
    }

    /// A last line without a line ending gets CRLF before anything is appended after it.
    fn terminate_last(&mut self) {
        if let Some(last) = self.lines.last_mut() {
            if last.eol.is_empty() {
                last.eol = CRLF;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unchanged_file_is_written_back_byte_for_byte() {
        let bytes =
            b"; comment\r\n[Sound]\r\nFXVolume = 1.000000\nDisable=0\r\n\r\n[Camera]\r\nX=1";
        assert_eq!(IniFile::parse(bytes).to_bytes(), bytes.to_vec());
    }

    #[test]
    fn reads_trim_spaces_and_match_names_without_case() {
        let ini = IniFile::parse(b"[ Sound ]\r\n  Volume  =  0.5  \r\n; Volume=9\r\n");
        assert_eq!(ini.get("sound", "volume").as_deref(), Some("0.5"));
        assert_eq!(ini.get("sound", "missing"), None);
        assert_eq!(ini.get("camera", "volume"), None);
    }

    #[test]
    fn a_new_section_is_appended_as_crlf_lines() {
        let mut ini = IniFile::parse(b"[A]\r\nx=1\r\n");
        ini.set("B", "y", "2");
        assert_eq!(ini.to_bytes(), b"[A]\r\nx=1\r\n[B]\r\ny=2\r\n".to_vec());
    }

    #[test]
    fn a_new_key_goes_before_the_next_header_and_an_old_one_is_replaced_in_place() {
        let mut ini = IniFile::parse(b"[A]\r\nx=1\r\n\r\n[B]\r\ny=2\r\n");
        ini.set("A", "z", "3");
        ini.set("A", "X", "9");
        // The replaced key is written with the name it was set under, and the new key goes
        // at the next header's place (after the blank line before it, so right above that header).
        assert_eq!(
            ini.to_bytes(),
            b"[A]\r\nX=9\r\n\r\nz=3\r\n[B]\r\ny=2\r\n".to_vec()
        );
    }

    #[test]
    fn a_last_line_without_a_line_ending_gets_one_before_an_append() {
        let mut ini = IniFile::parse(b"[A]\nx=1");
        ini.set("A", "y", "2");
        assert_eq!(ini.to_bytes(), b"[A]\nx=1\r\ny=2\r\n".to_vec());
    }
}
