//! The MSVC CRT string routines `gwpstring` calls: `wcstol`/`wcstoul`/`_wcstoi64`/`_wcstoui64`
//! with base 0, `wcstod`, `wcscmp`, and the C-locale `_wcsicmp`/`_wcslwr`/`_wcsupr`.

/// C-locale `iswspace`.
fn is_space(c: u16) -> bool {
    matches!(c, 0x20 | 0x09..=0x0D)
}

/// C-locale `towlower` (only A-Z fold).
pub fn to_lower(c: u16) -> u16 {
    if (u16::from(b'A')..=u16::from(b'Z')).contains(&c) {
        c + 32
    } else {
        c
    }
}

/// C-locale `towupper`.
pub fn to_upper(c: u16) -> u16 {
    if (u16::from(b'a')..=u16::from(b'z')).contains(&c) {
        c - 32
    } else {
        c
    }
}

/// The characters up to the first NUL (C string view).
fn c_str(s: &[u16]) -> &[u16] {
    let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    &s[..end]
}

/// MSVC `wcscmp`: -1, 0 or 1.
pub fn wcscmp(a: &[u16], b: &[u16]) -> i32 {
    match c_str(a).cmp(c_str(b)) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// MSVC `_wcsicmp` in the C locale: the difference of the first folded characters that differ.
pub fn wcsicmp(a: &[u16], b: &[u16]) -> i32 {
    let (a, b) = (c_str(a), c_str(b));
    let mut i = 0;
    loop {
        let f = a.get(i).copied().map_or(0, to_lower);
        let l = b.get(i).copied().map_or(0, to_lower);
        if f == 0 || f != l {
            return i32::from(f) - i32::from(l);
        }
        i += 1;
    }
}

/// `wcsstr`: the index of the first occurrence of `needle` (an empty needle matches at 0).
pub fn wcsstr(hay: &[u16], needle: &[u16]) -> Option<usize> {
    let (hay, needle) = (c_str(hay), c_str(needle));
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The parsed magnitude of a base-0 integer and whether it overflowed `limit`.
fn parse_unsigned(s: &[u16]) -> (bool, u128, bool) {
    let s = c_str(s);
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let mut negative = false;
    if i < s.len() && (s[i] == u16::from(b'+') || s[i] == u16::from(b'-')) {
        negative = s[i] == u16::from(b'-');
        i += 1;
    }
    let mut base = 10;
    if i < s.len() && s[i] == u16::from(b'0') {
        base = 8;
        if i + 1 < s.len() && (s[i + 1] | 0x20) == u16::from(b'x') {
            base = 16;
            i += 2;
        }
    }
    let mut value: u128 = 0;
    let mut any = false;
    let mut overflow = false;
    while i < s.len() {
        let c = s[i];
        let digit = match c {
            0x30..=0x39 => u32::from(c - 0x30),
            0x41..=0x5A => u32::from(c - 0x41 + 10),
            0x61..=0x7A => u32::from(c - 0x61 + 10),
            _ => break,
        };
        if digit >= base {
            break;
        }
        any = true;
        value = value * u128::from(base) + u128::from(digit);
        if value > u128::from(u64::MAX) {
            overflow = true;
            value = u128::from(u64::MAX) + 1;
        }
        i += 1;
    }
    if !any {
        return (false, 0, false);
    }
    (negative, value, overflow)
}

/// `wcstol(s, NULL, 0)`.
pub fn wcstol(s: &[u16]) -> i32 {
    let (negative, v, _) = parse_unsigned(s);
    if negative {
        if v > 0x8000_0000 {
            i32::MIN
        } else {
            (v as i64).wrapping_neg() as i32
        }
    } else if v > i32::MAX as u128 {
        i32::MAX
    } else {
        v as i32
    }
}

/// `wcstoul(s, NULL, 0)`: a leading minus negates the unsigned result.
pub fn wcstoul(s: &[u16]) -> u32 {
    let (negative, v, _) = parse_unsigned(s);
    if v > u128::from(u32::MAX) {
        return u32::MAX;
    }
    let v = v as u32;
    if negative {
        v.wrapping_neg()
    } else {
        v
    }
}

/// `_wcstoi64(s, NULL, 0)`.
pub fn wcstoi64(s: &[u16]) -> i64 {
    let (negative, v, overflow) = parse_unsigned(s);
    if negative {
        if overflow || v > 0x8000_0000_0000_0000 {
            i64::MIN
        } else {
            (v as u64).wrapping_neg() as i64
        }
    } else if overflow || v > i64::MAX as u128 {
        i64::MAX
    } else {
        v as i64
    }
}

/// `_wcstoui64(s, NULL, 0)`.
pub fn wcstoui64(s: &[u16]) -> u64 {
    let (negative, v, overflow) = parse_unsigned(s);
    if overflow {
        return u64::MAX;
    }
    let v = v as u64;
    if negative {
        v.wrapping_neg()
    } else {
        v
    }
}

/// `wcstod(s, NULL)`: the longest decimal floating-point prefix, 0 if there is none.
pub fn wcstod(s: &[u16]) -> f64 {
    let s = c_str(s);
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let start = i;
    let ascii = |j: usize| {
        s.get(j)
            .map_or(0u8, |&c| if c < 0x80 { c as u8 } else { 0 })
    };
    if matches!(ascii(i), b'+' | b'-') {
        i += 1;
    }
    let mut digits = 0;
    while ascii(i).is_ascii_digit() {
        i += 1;
        digits += 1;
    }
    if ascii(i) == b'.' {
        i += 1;
        while ascii(i).is_ascii_digit() {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return 0.0;
    }
    if matches!(ascii(i), b'e' | b'E') {
        let mut j = i + 1;
        if matches!(ascii(j), b'+' | b'-') {
            j += 1;
        }
        if ascii(j).is_ascii_digit() {
            while ascii(j).is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }
    let text: String = s[start..i].iter().map(|&c| c as u8 as char).collect();
    text.parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn crt_parsing_and_comparison() {
        assert_eq!(wcstol(&w("  -42xyz")), -42);
        assert_eq!(wcstol(&w("0x1F")), 31);
        assert_eq!(wcstol(&w("017")), 15);
        assert_eq!(wcstol(&w("99999999999")), i32::MAX);
        assert_eq!(wcstol(&w("abc")), 0);
        assert_eq!(wcstoul(&w("-1")), u32::MAX);
        assert_eq!(wcstoi64(&w("-9223372036854775808")), i64::MIN);
        assert_eq!(wcstoui64(&w("0xFFFFFFFFFFFFFFFFF")), u64::MAX);
        assert_eq!(wcstod(&w(" 1.5e2kg")), 150.0);
        assert_eq!(wcstod(&w(".e1")), 0.0);
        assert_eq!(wcscmp(&w("abc"), &w("abd")), -1);
        assert_eq!(wcsicmp(&w("ABC"), &w("abc")), 0);
        assert_eq!(wcsicmp(&w("a"), &w("c")), -2);
        assert_eq!(wcsstr(&w("hello"), &w("ll")), Some(2));
    }
}
