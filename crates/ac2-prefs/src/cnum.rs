//! The C library's number parsing as the retail client calls it (`PSUtils::is_int32`,
//! `is_uint32` and `is_double`, with `strtol`, `strtoul` and `strtod` on a 32-bit `long`). A
//! string is a number when the whole of it is consumed and no range error (`ERANGE`) was set. An
//! empty string is a number (its end pointer is at the NUL, value 0), and a string with no digits
//! consumes nothing, so it is not one.

/// One conversion: the value, the index where it stopped, and whether it overflowed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub end: usize,
    pub range_error: bool,
}

fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn digit_value(b: u8) -> Option<u32> {
    match b {
        b'0'..=b'9' => Some(u32::from(b - b'0')),
        b'a'..=b'f' => Some(u32::from(b - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(b - b'A') + 10),
        _ => None,
    }
}

/// The magnitude of a base-0 integer from `start` (after its sign), with its base and end. A
/// string with no digits gives `None` and the end at the start of the string (as `strtol` does).
fn integer(s: &[u8], start: usize) -> (Option<(u128, bool)>, usize) {
    let at = |i: usize| s.get(i).copied().unwrap_or(0);
    let (base, mut i) = if at(start) == b'0'
        && matches!(at(start + 1), b'x' | b'X')
        && digit_value(at(start + 2)).is_some_and(|d| d < 16)
    {
        (16u32, start + 2)
    } else if at(start) == b'0' {
        (8, start)
    } else {
        (10, start)
    };
    let first = i;
    let mut magnitude: u128 = 0;
    let mut overflow = false;
    while let Some(d) = digit_value(at(i)).filter(|d| *d < base) {
        if magnitude > u128::from(u64::MAX) {
            overflow = true;
        } else {
            magnitude = magnitude * u128::from(base) + u128::from(d);
        }
        i += 1;
    }
    if i == first {
        // "0x" with no hex digit after it parses the leading zero only.
        if base == 16 {
            return (Some((0, false)), start + 1);
        }
        return (None, 0);
    }
    (Some((magnitude, overflow)), i)
}

/// `strtol(s, &end, 0)` on a 32-bit `long`.
pub fn strtol(s: &[u8]) -> Parsed<i32> {
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let negative = matches!(s.get(i), Some(b'-'));
    if matches!(s.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let (digits, end) = integer(s, i);
    let Some((magnitude, overflow)) = digits else {
        return Parsed {
            value: 0,
            end: 0,
            range_error: false,
        };
    };
    let limit: u128 = if negative { 1 << 31 } else { (1 << 31) - 1 };
    if overflow || magnitude > limit {
        let value = if negative { i32::MIN } else { i32::MAX };
        return Parsed {
            value,
            end,
            range_error: true,
        };
    }
    let value = if negative {
        (-(magnitude as i128)) as i32
    } else {
        magnitude as i32
    };
    Parsed {
        value,
        end,
        range_error: false,
    }
}

/// `strtoul(s, &end, 0)` on a 32-bit `unsigned long` (a minus sign negates the magnitude).
pub fn strtoul(s: &[u8]) -> Parsed<u32> {
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let negative = matches!(s.get(i), Some(b'-'));
    if matches!(s.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let (digits, end) = integer(s, i);
    let Some((magnitude, overflow)) = digits else {
        return Parsed {
            value: 0,
            end: 0,
            range_error: false,
        };
    };
    if overflow || magnitude > u128::from(u32::MAX) {
        return Parsed {
            value: u32::MAX,
            end,
            range_error: true,
        };
    }
    let value = if negative {
        (magnitude as u32).wrapping_neg()
    } else {
        magnitude as u32
    };
    Parsed {
        value,
        end,
        range_error: false,
    }
}

fn matches_ci(s: &[u8], at: usize, word: &[u8]) -> bool {
    s.len() >= at + word.len() && s[at..at + word.len()].eq_ignore_ascii_case(word)
}

/// `strtod(s, &end)`: decimal, hexadecimal (`0x`, with a binary exponent), `inf`/`infinity` and
/// `nan` (with an optional `(...)`).
pub fn strtod(s: &[u8]) -> Parsed<f64> {
    let none = Parsed {
        value: 0.0,
        end: 0,
        range_error: false,
    };
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let negative = matches!(s.get(i), Some(b'-'));
    if matches!(s.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let signed = |v: f64| if negative { -v } else { v };
    if matches_ci(s, i, b"inf") {
        let end = if matches_ci(s, i, b"infinity") {
            i + 8
        } else {
            i + 3
        };
        return Parsed {
            value: signed(f64::INFINITY),
            end,
            range_error: false,
        };
    }
    if matches_ci(s, i, b"nan") {
        let mut end = i + 3;
        if s.get(end) == Some(&b'(') {
            if let Some(close) = s[end..].iter().position(|&b| b == b')') {
                end += close + 1;
            }
        }
        return Parsed {
            value: f64::NAN,
            end,
            range_error: false,
        };
    }
    if s.get(i) == Some(&b'0') && matches!(s.get(i + 1), Some(b'x') | Some(b'X')) {
        if let Some(parsed) = hex_float(s, i + 2) {
            return Parsed {
                value: signed(parsed.0),
                end: parsed.1,
                range_error: parsed.2,
            };
        }
    }
    // Decimal: digits, an optional point and digits, then an optional exponent.
    let mut j = i;
    let mut digits = 0;
    while s.get(j).is_some_and(u8::is_ascii_digit) {
        j += 1;
        digits += 1;
    }
    if s.get(j) == Some(&b'.') {
        j += 1;
        while s.get(j).is_some_and(u8::is_ascii_digit) {
            j += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return none;
    }
    if matches!(s.get(j), Some(b'e') | Some(b'E')) {
        let mut k = j + 1;
        if matches!(s.get(k), Some(b'+') | Some(b'-')) {
            k += 1;
        }
        if s.get(k).is_some_and(u8::is_ascii_digit) {
            while s.get(k).is_some_and(u8::is_ascii_digit) {
                k += 1;
            }
            j = k;
        }
    }
    let text = std::str::from_utf8(&s[i..j]).unwrap_or("0");
    let magnitude: f64 = text.parse().unwrap_or(0.0);
    let range_error = magnitude.is_infinite()
        || (magnitude == 0.0 && s[i..j].iter().any(|&b| (b'1'..=b'9').contains(&b)));
    Parsed {
        value: signed(magnitude),
        end: j,
        range_error,
    }
}

/// A hexadecimal floating constant after `0x` (from `start`): its value, end, and range error.
fn hex_float(s: &[u8], start: usize) -> Option<(f64, usize, bool)> {
    let mut i = start;
    let mut mantissa = 0.0f64;
    let mut exponent: i64 = 0;
    let mut any = false;
    while let Some(d) = s.get(i).and_then(|&b| digit_value(b)).filter(|d| *d < 16) {
        mantissa = mantissa * 16.0 + f64::from(d);
        any = true;
        i += 1;
    }
    if s.get(i) == Some(&b'.') {
        let mut k = i + 1;
        while let Some(d) = s.get(k).and_then(|&b| digit_value(b)).filter(|d| *d < 16) {
            mantissa = mantissa * 16.0 + f64::from(d);
            exponent -= 4;
            any = true;
            k += 1;
        }
        if any {
            i = k;
        }
    }
    if !any {
        return None;
    }
    if matches!(s.get(i), Some(b'p') | Some(b'P')) {
        let mut k = i + 1;
        let mut negative = false;
        if matches!(s.get(k), Some(b'+') | Some(b'-')) {
            negative = s[k] == b'-';
            k += 1;
        }
        if s.get(k).is_some_and(u8::is_ascii_digit) {
            let mut e: i64 = 0;
            while let Some(d) = s.get(k).filter(|b| b.is_ascii_digit()) {
                e = (e * 10 + i64::from(d - b'0')).min(100_000);
                k += 1;
            }
            exponent += if negative { -e } else { e };
            i = k;
        }
    }
    let value = mantissa * 2f64.powi(exponent.clamp(-100_000, 100_000) as i32);
    Some((value, i, value.is_infinite()))
}

/// `PSUtils::is_int32`: the whole string is a `long` (base 0, no range error).
pub fn is_int32(s: &str) -> bool {
    let p = strtol(s.as_bytes());
    p.end == s.len() && !p.range_error
}

/// `PSUtils::is_uint32`: the whole string is an `unsigned long` (base 0, no range error).
pub fn is_uint32(s: &str) -> bool {
    let p = strtoul(s.as_bytes());
    p.end == s.len() && !p.range_error
}

/// `PSUtils::is_double`: the whole string is a `double` (no range error).
pub fn is_double(s: &str) -> bool {
    let p = strtod(s.as_bytes());
    p.end == s.len() && !p.range_error
}
