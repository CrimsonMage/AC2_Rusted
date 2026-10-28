//! `gTabooTableAdaptor::CallEFun_FormatStringForAudienceW` (`TabooTableAdaptor::
//! FormatStringForAudienceW`, `TabooTable::FormatStringForAudience`): the spelling a name or
//! title takes before it is shown. The adaptor fetches the `TabooTable` DBObj (a missing table
//! answers 0 and leaves the out string alone); the table then trims the text, collapses runs of
//! blanks to the first blank (`PSUtils::TrimExcessSpaces`) and capitalises every letter that
//! follows a non-letter, lower-casing the rest. The result says whether anything changed. The
//! audience word does not take part in the formatting.

use super::{out_native, Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{Context, EFuns, InterpStatus, Vm};

/// The `TabooTable` enum in the unique-DB group (`DBObj::GetByEnum(0x40000005, 0x40000002, ..)`).
const TABOO_TABLE_ENUM: u32 = 0x4000_0005;
/// `UNIQUEDB_GROUP_ENUM`.
const UNIQUE_DB_GROUP: u32 = 0x4000_0002;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x006B_0003, format_string_for_audience);
}

/// C-locale `iswalpha` (the client runs in the "C" locale).
fn is_alpha(c: u16) -> bool {
    matches!(c, 0x41..=0x5A | 0x61..=0x7A)
}

/// C-locale `towupper` / `towlower`.
fn fold(c: u16, upper: bool) -> u16 {
    match (c, upper) {
        (0x61..=0x7A, true) => c - 0x20,
        (0x41..=0x5A, false) => c + 0x20,
        _ => c,
    }
}

/// `PSUtils::TrimExcessSpaces`: a run of spaces and tabs keeps only its first character; the
/// flag says a run was shortened.
pub(super) fn trim_excess_spaces(text: &[u16]) -> (Vec<u16>, bool) {
    let mut out = Vec::with_capacity(text.len());
    let (mut changed, mut in_run) = (false, false);
    for &c in text {
        if c == 0x20 || c == 0x09 {
            if in_run {
                changed = true;
            } else {
                out.push(c);
            }
            in_run = true;
        } else {
            out.push(c);
            in_run = false;
        }
    }
    (out, changed)
}

/// `TabooTable::FormatStringForAudience`: the formatted text and whether it differs from the
/// input (a trim, a collapsed run, or a changed letter).
pub(super) fn format_for_audience(input: &[u16]) -> (Vec<u16>, bool) {
    let is_blank = |c: &u16| matches!(*c, 0x20 | 0x09 | 0x0D | 0x0A);
    let start = input
        .iter()
        .position(|c| !is_blank(c))
        .unwrap_or(input.len());
    let end = input
        .iter()
        .rposition(|c| !is_blank(c))
        .map_or(start, |i| i + 1);
    let trimmed = &input[start..end];
    let mut changed = trimmed.len() != input.len();
    let (mut text, collapsed) = trim_excess_spaces(trimmed);
    changed |= collapsed;
    let mut previous_alpha = false;
    for c in &mut text {
        let alpha = is_alpha(*c);
        if alpha {
            let set = fold(*c, !previous_alpha);
            changed |= set != *c;
            *c = set;
        }
        previous_alpha = alpha;
    }
    (text, changed)
}

/// `gTabooTableAdaptor::CallEFun_FormatStringForAudienceW(audience, string, out string)`.
fn format_string_for_audience(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (_audience, input, out) = (call.word(2), call.word(3), call.word(4));
    let source = out_native(vm, ctx, input);
    let text = match vm.wstring(source) {
        Some(text) => text.to_vec(),
        None => return vm.bad_this(source),
    };
    let table = vm
        .data()
        .enum_did(TABOO_TABLE_ENUM, UNIQUE_DB_GROUP)
        .is_some_and(|did| did != 0);
    if !table {
        Call::new(ctx).ret(5, 0, INT_RESULT);
        return InterpStatus::Success;
    }
    let (formatted, changed) = format_for_audience(&text);
    let target = out_native(vm, ctx, out);
    match vm.native_mut(target) {
        Some(NativeObject::WString(slot)) => *slot = formatted,
        _ => return vm.bad_this(target),
    }
    Call::new(ctx).ret(5, u32::from(changed), INT_RESULT);
    InterpStatus::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn names_are_trimmed_collapsed_and_capitalised() {
        assert_eq!(format_for_audience(&w("yaraq")), (w("Yaraq"), true));
        assert_eq!(format_for_audience(&w("Yaraq")), (w("Yaraq"), false));
        assert_eq!(
            format_for_audience(&w("  sir  GALAHAD of-the lake ")),
            (w("Sir Galahad Of-The Lake"), true)
        );
        assert_eq!(format_for_audience(&w("")), (w(""), false));
        assert_eq!(format_for_audience(&w("McDuff")), (w("Mcduff"), true));
    }

    #[test]
    fn a_run_of_blanks_keeps_its_first_character() {
        assert_eq!(trim_excess_spaces(&w("a \t b")), (w("a b"), true));
        assert_eq!(trim_excess_spaces(&w("a\t b")), (w("a\tb"), true));
        assert_eq!(trim_excess_spaces(&w("a b")), (w("a b"), false));
    }
}
