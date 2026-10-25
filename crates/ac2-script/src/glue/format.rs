//! The MSVC CRT number formatting the glue's string appends use (`append_float` "%f",
//! `append_double` "%lf", `append_int32_hex` "0x%08X", ...).
//!
//! The old CRT's `%f` takes at most 17 significant digits from the value (`_fltout2`), rounds
//! that digit string half-up to the requested decimals and pads with zeros; infinities and NaNs
//! print as `1.#INF00`, `-1.#IND00` (the x87 default NaN), `1.#QNAN0`.

/// `%f` of a `float` (promoted to double, as varargs do).
pub fn msvc_float(value: f32) -> String {
    msvc_double(f64::from(value))
}

/// `%f` / `%lf` with the default six decimals.
pub fn msvc_double(value: f64) -> String {
    msvc_fixed(value, 6)
}

fn msvc_fixed(value: f64, decimals: usize) -> String {
    if value.is_nan() {
        let bits = value.to_bits();
        let negative = bits >> 63 == 1;
        let quiet = bits & (1 << 51) != 0;
        let indefinite = negative && bits & ((1 << 51) - 1) == 0 && quiet;
        let body = if indefinite {
            "1.#IND"
        } else if quiet {
            "1.#QNAN"
        } else {
            "1.#SNAN"
        };
        return special(negative, body, decimals);
    }
    if value.is_infinite() {
        return special(value < 0.0, "1.#INF", decimals);
    }
    let negative = value.is_sign_negative() && value != 0.0;
    // 17 significant digits, correctly rounded from the binary value.
    let sci = format!("{:.16e}", value.abs());
    let (mantissa, exponent) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let mut digits: Vec<u8> = mantissa.bytes().filter(u8::is_ascii_digit).collect();
    if value == 0.0 {
        digits = vec![b'0'];
    }
    // Position of the decimal point within `digits` (digits[..point] is the integer part).
    let point = exponent + 1;
    // Keep `point + decimals` digits, rounding half-up on the next one.
    let keep = point + decimals as i32;
    let mut point = point;
    if keep < 0 {
        digits = vec![b'0'];
        point = 1;
    } else {
        let keep = keep as usize;
        if digits.len() > keep {
            let round_up = digits[keep] >= b'5';
            digits.truncate(keep);
            if round_up {
                let mut i = digits.len();
                loop {
                    if i == 0 {
                        digits.insert(0, b'1');
                        point += 1;
                        break;
                    }
                    i -= 1;
                    if digits[i] == b'9' {
                        digits[i] = b'0';
                    } else {
                        digits[i] += 1;
                        break;
                    }
                }
            }
        }
    }
    // Lay the digits out around the point, zero-filling both sides.
    let mut int_part = String::new();
    let mut frac_part = String::new();
    for i in 0..(point.max(1) as usize) {
        let index = i as i32 - (point.max(1) - point);
        int_part.push(digit_at(&digits, index));
    }
    for i in 0..decimals {
        frac_part.push(digit_at(&digits, point + i as i32));
    }
    let int_part = int_part.trim_start_matches('0');
    let int_part = if int_part.is_empty() { "0" } else { int_part };
    let all_zero = int_part == "0" && frac_part.bytes().all(|b| b == b'0');
    let sign = if negative && !all_zero { "-" } else { "" };
    if decimals == 0 {
        format!("{sign}{int_part}")
    } else {
        format!("{sign}{int_part}.{frac_part}")
    }
}

fn digit_at(digits: &[u8], index: i32) -> char {
    if index < 0 {
        '0'
    } else {
        digits.get(index as usize).map_or('0', |&d| d as char)
    }
}

fn special(negative: bool, body: &str, decimals: usize) -> String {
    let sign = if negative { "-" } else { "" };
    let have = body.len() - 2;
    let mut out = format!("{sign}{body}");
    for _ in have..decimals {
        out.push('0');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_point_matches_the_old_crt() {
        assert_eq!(msvc_double(0.0), "0.000000");
        assert_eq!(msvc_double(-0.0), "0.000000");
        assert_eq!(msvc_double(1.5), "1.500000");
        assert_eq!(msvc_double(-2.5), "-2.500000");
        // 0.0000015 is 1.49999999999999996e-6: 17 digits give ...15, which rounds up.
        assert_eq!(msvc_double(0.0000015), "0.000002");
        assert_eq!(msvc_double(123456.789), "123456.789000");
        assert_eq!(msvc_float(0.1), "0.100000");
        assert_eq!(msvc_double(0.0000004), "0.000000");
        assert_eq!(msvc_double(-0.0000004), "0.000000");
        assert_eq!(msvc_double(999999.9999996), "1000000.000000");
        // Beyond 17 significant digits the old CRT pads with zeros.
        assert_eq!(msvc_double(1e20), "100000000000000000000.000000");
        assert_eq!(
            msvc_double(123456789012345678901.0),
            "123456789012345680000.000000"
        );
        assert_eq!(msvc_double(f64::INFINITY), "1.#INF00");
        assert_eq!(msvc_double(f64::NEG_INFINITY), "-1.#INF00");
        assert_eq!(
            msvc_double(f64::from_bits(0xFFF8_0000_0000_0000)),
            "-1.#IND00"
        );
        assert_eq!(
            msvc_double(f64::from_bits(0x7FF8_0000_0000_0001)),
            "1.#QNAN0"
        );
    }
}
