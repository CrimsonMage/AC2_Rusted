//! `StringTable::FloatToString` and the integer conversions over it (`IntToString`,
//! `UIntToString`, `LIntToString`, `ULIntToString`): numbers in the language's numerals with
//! its grouping and decimal separators and negative format (`StringStateInfo`).
//!
//! Every number goes through x87 doubles: retail ran at 53-bit precision, so the digit loop
//! is plain `f64` arithmetic here, in retail's operation order.

use crate::string_meta_text::wide;
use crate::string_state::StringState;
use crate::string_table::StringTable;

/// `FloatToString`'s work buffer: digits are written backwards from index 62 (63 holds the
/// terminator); running down to index 0 fails the conversion.
const BUFFER_TOP: usize = 62;
const MAX_PRECISION: u16 = 20;

impl StringTable {
    /// The packed header's `currentStateID`.
    pub fn current_state_id(&self) -> u32 {
        self.header.0
    }

    /// `m_base` (negative: the `StringState`'s).
    pub fn number_base(&self) -> i16 {
        self.header.1 as i16
    }

    /// `m_leadingZero` (negative: the `StringState`'s).
    pub fn leading_zero(&self) -> i32 {
        self.header.2 as i32
    }

    /// `m_numDecimalDigits` (negative: the `StringState`'s).
    pub fn num_decimal_digits(&self) -> i16 {
        self.header.3 as i16
    }

    /// `StringTable::FloatToString(result, val, precision, grouping, base)`: `None` where
    /// retail returns 0 (no `StringState`, precision above 20, a base below 2 or beyond the
    /// numerals, or more than 62 units of output). A negative `base` takes the table's, then
    /// the state's.
    pub fn float_to_string(
        &self,
        state: Option<&StringState>,
        value: f64,
        precision: u16,
        grouping: i32,
        base: i16,
    ) -> Option<Vec<u16>> {
        let state = state?;
        if precision > MAX_PRECISION {
            return None;
        }
        let mut base = base;
        if base < 0 {
            base = self.number_base();
            if base < 0 {
                base = state.base as i16;
            }
        }
        let mut leading_zero = self.leading_zero();
        if leading_zero < 0 {
            leading_zero = state.leading_zero;
        }
        let mut min_digits = self.num_decimal_digits();
        if min_digits < 0 {
            min_digits = state.num_decimal_digits as i16;
        }
        if base <= 1 || state.numerals.len() < base as usize {
            return None;
        }
        let digits = digit_loop(
            state,
            value.abs(),
            precision,
            grouping != 0,
            base,
            leading_zero,
            min_digits,
        )?;
        // `fcomp 0.0`: only a value below zero takes the negative format.
        let format = if value < 0.0 {
            state.negative_number_format.clone()
        } else if base == 16 {
            wide("0x%s")
        } else {
            wide("%s")
        };
        Some(format_s(&format, &digits))
    }

    /// `StringTable::IntToString`: the value as a double with no decimals.
    pub fn int_to_string(
        &self,
        state: Option<&StringState>,
        value: i32,
        grouping: i32,
        base: i16,
    ) -> Option<Vec<u16>> {
        self.float_to_string(state, f64::from(value), 0, grouping, base)
    }

    /// `StringTable::UIntToString`.
    pub fn uint_to_string(
        &self,
        state: Option<&StringState>,
        value: u32,
        grouping: i32,
        base: i16,
    ) -> Option<Vec<u16>> {
        self.float_to_string(state, f64::from(value), 0, grouping, base)
    }

    /// `StringTable::LIntToString` (`fild qword`: rounded to 53 bits).
    pub fn lint_to_string(
        &self,
        state: Option<&StringState>,
        value: i64,
        grouping: i32,
        base: i16,
    ) -> Option<Vec<u16>> {
        self.float_to_string(state, value as f64, 0, grouping, base)
    }

    /// `StringTable::ULIntToString`: the low 63 bits as a double plus 2^63 for the top bit,
    /// as the compiler's unsigned conversion does.
    pub fn ulint_to_string(
        &self,
        state: Option<&StringState>,
        value: u64,
        grouping: i32,
        base: i16,
    ) -> Option<Vec<u16>> {
        let low = (value & 0x7FFF_FFFF_FFFF_FFFF) as i64 as f64;
        let high = if value >> 63 != 0 {
            9_223_372_036_854_775_808.0
        } else {
            0.0
        };
        self.float_to_string(state, high + low, 0, grouping, base)
    }
}

/// `_ftol2`: truncation to i64; out of range (and NaN) gives the x87 "integer indefinite".
fn ftol2(x: f64) -> i64 {
    if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&x) {
        x as i64
    } else {
        i64::MIN
    }
}

/// `FloatToString`'s digit loop: digits from the lowest place up, a grouping separator every
/// `grouping_size` integer places, the decimal separator after the fractional digits; stops
/// once the place value passes the number (and, with `leading_zero`, once `min_digits` digits
/// are out).
fn digit_loop(
    state: &StringState,
    remaining: f64,
    precision: u16,
    grouping: bool,
    base: i16,
    leading_zero: i32,
    min_digits: i16,
) -> Option<Vec<u16>> {
    let mut buffer = [0u16; BUFFER_TOP + 2];
    let mut si = BUFFER_TOP;
    let abs_val = if remaining < 1.0 { 1.0 } else { remaining };
    let mut remaining = remaining;
    let base_f = f64::from(base);
    let mut place = -i32::from(precision);
    let mut place_val = base_f.powf(f64::from(place));
    let mut count: i16 = 0;
    loop {
        let shifted = base_f.powf(f64::from(-place)) * remaining + 0.5;
        let digit = (ftol2(shifted) as u64) % (i64::from(base) as u64);
        buffer[si] = state.numerals[digit as usize];
        si -= 1;
        count = count.wrapping_add(1);
        place += 1;
        remaining -= digit as f64 * place_val;
        place_val *= base_f;
        if place_val > abs_val && (leading_zero == 0 || count >= min_digits) {
            return Some(buffer[si + 1..=BUFFER_TOP].to_vec());
        }
        if si == 0 {
            return None;
        }
        if grouping
            && place > 0
            && state.grouping_size != 0
            && place % i32::from(state.grouping_size) == 0
        {
            write_backwards(&mut buffer, &mut si, &state.grouping_separator)?;
        }
        if place == 0 {
            write_backwards(&mut buffer, &mut si, &state.decimal_separator)?;
        }
    }
}

/// Copies a separator into the buffer last unit first, moving the write index down.
fn write_backwards(buffer: &mut [u16], si: &mut usize, text: &[u16]) -> Option<()> {
    for &unit in text.iter().rev() {
        buffer[*si] = unit;
        *si = si.checked_sub(1)?;
    }
    Some(())
}

/// `PStringBase<unsigned short>::sprintf(format, text)` for the one-string formats retail
/// passes here (`%s`, `0x%s`, the state's negative format); `%%` is a percent sign.
fn format_s(format: &[u16], text: &[u16]) -> Vec<u16> {
    const PERCENT: u16 = b'%' as u16;
    let mut out = Vec::with_capacity(format.len() + text.len());
    let mut i = 0;
    let mut used = false;
    while i < format.len() {
        match (format[i], format.get(i + 1).copied()) {
            (PERCENT, Some(PERCENT)) => {
                out.push(PERCENT);
                i += 2;
            }
            (PERCENT, Some(s)) if s == u16::from(b's') && !used => {
                out.extend_from_slice(text);
                used = true;
                i += 2;
            }
            (c, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Legions English state's number settings (base 10, five digits, no leading zero,
    /// groups of three, `.`, `,`, `-%s`).
    pub(crate) fn english() -> StringState {
        StringState {
            base: 10,
            num_decimal_digits: 5,
            leading_zero: 0,
            grouping_size: 3,
            numerals: wide("0123456789ABCDEF"),
            decimal_separator: wide("."),
            grouping_separator: wide(","),
            negative_number_format: wide("-%s"),
            singular: [0, 1, 0, 0, 0],
            ..Default::default()
        }
    }

    fn table() -> StringTable {
        StringTable {
            header: (0, 0xFFFF, 0xFFFF_FFFF, 0xFFFF),
            ..Default::default()
        }
    }

    fn text(v: Option<Vec<u16>>) -> String {
        String::from_utf16_lossy(&v.expect("formats"))
    }

    #[test]
    fn integers_group_by_three_and_negatives_use_the_format() {
        let (t, s) = (table(), english());
        assert_eq!(text(t.int_to_string(Some(&s), 0, 1, -1)), "0");
        assert_eq!(text(t.int_to_string(Some(&s), 7, 1, -1)), "7");
        assert_eq!(text(t.int_to_string(Some(&s), 1000, 1, -1)), "1,000");
        assert_eq!(text(t.int_to_string(Some(&s), 1000, 0, -1)), "1000");
        assert_eq!(
            text(t.int_to_string(Some(&s), -1234567, 1, -1)),
            "-1,234,567"
        );
        assert_eq!(
            text(t.uint_to_string(Some(&s), u32::MAX, 1, -1)),
            "4,294,967,295"
        );
        assert_eq!(text(t.int_to_string(Some(&s), 255, 0, 16)), "0xFF");
        assert_eq!(text(t.lint_to_string(Some(&s), 100, 1, 2)), "1,100,100");
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn floats_round_at_the_precision_and_keep_one_integer_digit() {
        let (t, s) = (table(), english());
        assert_eq!(text(t.float_to_string(Some(&s), 3.14159, 2, 1, -1)), "3.14");
        assert_eq!(text(t.float_to_string(Some(&s), 0.5, 1, 1, -1)), "0.5");
        assert_eq!(text(t.float_to_string(Some(&s), 2.5, 0, 1, -1)), "3");
        assert_eq!(text(t.float_to_string(Some(&s), -0.25, 2, 1, -1)), "-0.25");
        assert_eq!(
            text(t.float_to_string(Some(&s), 12345.5, 1, 1, -1)),
            "12,345.5"
        );
    }

    #[test]
    fn retail_failures_give_none() {
        let (t, s) = (table(), english());
        assert_eq!(t.int_to_string(None, 1, 1, -1), None);
        assert_eq!(t.float_to_string(Some(&s), 1.0, 21, 1, -1), None);
        assert_eq!(t.int_to_string(Some(&s), 1, 1, 1), None);
        assert_eq!(t.int_to_string(Some(&s), 1, 1, 17), None);
        assert_eq!(t.float_to_string(Some(&s), f64::NAN, 0, 1, -1), None);
    }

    #[test]
    fn leading_zero_pads_to_the_minimum_digit_count() {
        let mut t = table();
        t.header.2 = 1;
        assert_eq!(text(t.int_to_string(Some(&english()), 42, 0, -1)), "00042");
    }
}
