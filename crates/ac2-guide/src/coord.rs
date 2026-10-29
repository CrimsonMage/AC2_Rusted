//! Finds map coordinates in running text, in the forms `Position::FromRealMapCoords` reads:
//! `%f%c %f%c` (whitespace, possibly none, between) or `%f%c, %f%c`, north/south first, the
//! letters in either case. The site's NPC database also writes `10.5S_31.4W`, so `_` is taken
//! as a separator too. Retail also takes a lone trailing `E` as implied; prose never relies on
//! that, so it is not matched here. Values beyond the map (over 100) are skipped.

use crate::guide::MapCoord;

/// The coordinates in `text` as `(byte range, coordinate)`, in order and not overlapping.
pub fn find(text: &str) -> Vec<(std::ops::Range<usize>, MapCoord)> {
    let s = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let boundary = i == 0 || !(s[i - 1].is_ascii_alphanumeric() || s[i - 1] == b'.');
        if boundary && s[i].is_ascii_digit() {
            if let Some((end, at)) = parse_at(s, i) {
                out.push((i..end, at));
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// One coordinate starting at `start`: its end and value.
fn parse_at(s: &[u8], start: usize) -> Option<(usize, MapCoord)> {
    let (mut at, north) = number(s, start)?;
    let ns = s.get(at)?.to_ascii_uppercase();
    if ns != b'N' && ns != b'S' {
        return None;
    }
    at += 1;
    if matches!(s.get(at), Some(b',' | b'_')) {
        at += 1;
    }
    while s.get(at).is_some_and(|c| c.is_ascii_whitespace()) {
        at += 1;
    }
    if !s.get(at)?.is_ascii_digit() {
        return None;
    }
    let (end, east) = number(s, at)?;
    let ew = s.get(end)?.to_ascii_uppercase();
    if ew != b'E' && ew != b'W' {
        return None;
    }
    let end = end + 1;
    if s.get(end).is_some_and(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let north = if ns == b'S' { -north } else { north };
    let east = if ew == b'W' { -east } else { east };
    (north.abs() <= 100.0 && east.abs() <= 100.0).then_some((end, MapCoord { north, east }))
}

/// Digits with an optional fraction (`swscanf`'s `%f` as the site writes it).
fn number(s: &[u8], start: usize) -> Option<(usize, f32)> {
    let mut end = start;
    while s.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    if s.get(end) == Some(&b'.') && s.get(end + 1).is_some_and(u8::is_ascii_digit) {
        end += 1;
        while s.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
    }
    let v = std::str::from_utf8(&s[start..end]).ok()?.parse().ok()?;
    Some((end, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coords(text: &str) -> Vec<(&str, f32, f32)> {
        find(text)
            .into_iter()
            .map(|(r, c)| (&text[r], c.north, c.east))
            .collect()
    }

    #[test]
    fn site_forms() {
        assert_eq!(coords("at 38.0N 5.5W."), [("38.0N 5.5W", 38.0, -5.5)]);
        assert_eq!(coords("(10.5S_31.4W)"), [("10.5S_31.4W", -10.5, -31.4)]);
        assert_eq!(coords("(51.8S, 39.0E)"), [("51.8S, 39.0E", -51.8, 39.0)]);
        assert_eq!(
            coords("42N, 8.5e then 2.6n13.8E"),
            [("42N, 8.5e", 42.0, 8.5), ("2.6n13.8E", 2.6, 13.8)]
        );
    }

    #[test]
    fn rejects_prose() {
        assert!(coords("level 45 North 3 East").is_empty());
        assert!(coords("x38.0N 5.5W").is_empty());
        assert!(coords("38.0N 5.5West").is_empty());
        assert!(coords("2.6N , 13.8E").is_empty());
        assert!(coords("120.0N 5.5W").is_empty());
    }
}
