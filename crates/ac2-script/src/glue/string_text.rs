//! `gStringInfo` text: `IsValid`, `eq`/`ne`, `GetString`, `InqString`, `GetLogString` and
//! `GetStringWithMetaTags` (`StringInfo::InqStringInternal` and its callers in
//! `stringtable/StringInfo.cpp`).
//!
//! Tables come from [`ScriptData::string_table`] (ac2-db's decoder); the rendering of a table
//! string with its variables is [`ScriptData::render_string`], the host's text hook.

use super::{Call, INT_RESULT};
use crate::native::{GetStringError, NativeObject, StringInfo, StringVarValue};
use crate::vm::{tag, Context, EFuns, InterpStatus, ScriptData, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x006A_0010, get_string);
    efuns.register(0x006A_0011, inq_string);
    efuns.register(0x006A_0017, is_valid);
    efuns.register(0x006A_0018, get_log_string);
    efuns.register(0x006A_001A, eq);
    efuns.register(0x006A_001B, ne);
    efuns.register(0x006A_001D, get_string_with_meta_tags);
}

/// `PStringBase<unsigned short>::whitespace_string`.
const WHITESPACE: [u16; 4] = [b' ' as u16, b'\t' as u16, b'\r' as u16, b'\n' as u16];
const BACKSLASH: u16 = b'\\' as u16;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `StringTableMetaLanguage::StripMetaLetters`: cut at the first `[` not escaped by a
/// backslash (`PSUtils::wcschr_skipescapes`), then trim trailing whitespace.
pub(crate) fn strip_meta_letters(text: &[u16]) -> Vec<u16> {
    let mut i = 0;
    let mut cut = None;
    while i < text.len() {
        match text[i] {
            BACKSLASH => i += 1,
            c if c == u16::from(b'[') => {
                cut = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let Some(cut) = cut else {
        return text.to_vec();
    };
    let mut out = text[..cut].to_vec();
    while out.last().is_some_and(|c| WHITESPACE.contains(c)) {
        out.pop();
    }
    out
}

/// `StringTableMetaLanguage::UnescapeString`: `\q` becomes `"`, `\x` becomes `x` (a trailing
/// backslash takes the terminator, as retail's indexing does).
pub(crate) fn unescape_string(text: &[u16]) -> Vec<u16> {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text[i] == BACKSLASH {
            let next = text.get(i + 1).copied().unwrap_or(0);
            out.push(if next == u16::from(b'q') {
                u16::from(b'"')
            } else {
                next
            });
            i += 2;
        } else {
            out.push(text[i]);
            i += 1;
        }
    }
    out
}

/// `StringInfo::InqStringInternal(result, bRetainMetas)`: the text (or the error text retail
/// leaves in `result`) and the error code.
pub(crate) fn inq_string_internal(
    data: &dyn ScriptData,
    info: &StringInfo,
    retain_metas: bool,
) -> (Vec<u16>, Result<(), GetStringError>) {
    let strip = |t: Vec<u16>| {
        if retain_metas {
            t
        } else {
            strip_meta_letters(&t)
        }
    };
    if let Some(literal) = &info.literal {
        return (strip(literal.clone()), Ok(()));
    }
    let error = wide(&format!(
        "<string table error; token: 0x{:08X} tableDID: 0x{:08X}>",
        info.string_id, info.table_id
    ));
    if info.string_id == 0 || info.table_id == 0 {
        return (error, Err(GetStringError::Uninitialized));
    }
    let Some(table) = data.string_table(info.table_id) else {
        return (error, Err(GetStringError::MissingTable));
    };
    match data.render_string(&table, info) {
        Ok(text) => (strip(text), Ok(())),
        Err(e) => (error, Err(e)),
    }
}

/// `StringInfo::InqString(result, bRetainMetas)`: unescaped on success.
pub(crate) fn inq(
    data: &dyn ScriptData,
    info: &StringInfo,
    retain_metas: bool,
) -> (Vec<u16>, bool) {
    match inq_string_internal(data, info, retain_metas) {
        (text, Ok(())) => (unescape_string(&text), true),
        (text, Err(_)) => (text, false),
    }
}

/// `StringInfo::IsValid(bValidateChildren)`.
pub(crate) fn valid(data: &dyn ScriptData, info: &StringInfo, children: bool) -> bool {
    if info.literal.is_some() {
        return true;
    }
    if info.string_id == 0 || info.table_id == 0 {
        return false;
    }
    match data.string_table(info.table_id) {
        Some(table) if table.get(info.string_id).is_some() => {}
        _ => return false,
    }
    !children
        || info.variables.iter().all(|v| match &v.value {
            StringVarValue::StringInfo(nested) => valid(data, nested, children),
            _ => true,
        })
}

/// `StringInfo::operator==`: ids and literal flag; then the literal text, or the same
/// variables by id with equal values.
pub(crate) fn equal(a: &StringInfo, b: &StringInfo) -> bool {
    if a.string_id != b.string_id
        || a.table_id != b.table_id
        || a.literal.is_some() != b.literal.is_some()
    {
        return false;
    }
    if let (Some(x), Some(y)) = (&a.literal, &b.literal) {
        return x == y;
    }
    a.variables.len() == b.variables.len()
        && a.variables.iter().all(|v| {
            b.variables
                .iter()
                .find(|w| w.id == v.id)
                .is_some_and(|w| w.kind == v.kind && w.value == v.value)
        })
}

fn this_info(vm: &Vm, reference: u32) -> Option<StringInfo> {
    match vm.native(reference)? {
        NativeObject::StringInfo(info) => Some(info.clone()),
        _ => None,
    }
}

/// Returns a new `wpstring` in word 2 (`VM::CopyNativeObject(0x76, …)`).
fn ret_text(
    vm: &mut Vm,
    ctx: &mut Context,
    f: fn(&dyn ScriptData, &StringInfo) -> Vec<u16>,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(info) = this_info(vm, this) else {
        return vm.bad_this(this);
    };
    let text = f(vm.data(), &info);
    let r = vm.new_wstring(text);
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

/// `StringInfo::GetString(bRetainMetas = 0)`.
fn get_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_text(vm, ctx, |d, i| inq(d, i, false).0)
}

/// `StringInfo::GetStringWithMetaTags`: `GetString` keeping the meta letters.
fn get_string_with_meta_tags(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_text(vm, ctx, |d, i| inq(d, i, true).0)
}

/// `StringInfo::GetLogString`: the text, or why it could not be rendered.
fn get_log_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_text(vm, ctx, |d, i| match inq_string_internal(d, i, false) {
        (text, Ok(())) => text,
        (_, Err(e)) => wide(&format!(
            "<could not render string: table 0x{:08X} token 0x{:08X}. Reason = {}>",
            i.table_id, i.string_id, e as u32
        )),
    })
}

/// `gStringInfo::CallEFun_InqString(out wpstring)`: `StringInfo::InqString(result, 0)` into
/// the referenced string; returns 1/0.
fn inq_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, out) = (call.this(), call.word(2));
    let Some(info) = this_info(vm, this) else {
        return vm.bad_this(this);
    };
    let (text, ok) = inq(vm.data(), &info, false);
    if let Some(target) = vm.read_through(ctx, out) {
        if let Some(NativeObject::WString(s)) = vm.native_mut(target) {
            *s = text;
        }
    }
    Call::new(ctx).ret(3, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

/// `StringInfo::IsValid(1)`.
fn is_valid(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(info) = this_info(vm, this) else {
        return vm.bad_this(this);
    };
    let ok = valid(vm.data(), &info, true);
    Call::new(ctx).ret(2, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

fn compare(vm: &mut Vm, ctx: &mut Context, want: bool) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, other) = (call.this(), call.word(2));
    let Some(a) = this_info(vm, this) else {
        return vm.bad_this(this);
    };
    let Some(b) = this_info(vm, other) else {
        return vm.bad_this(other);
    };
    let same = this == other || equal(&a, &b);
    Call::new(ctx).ret(3, u32::from(same == want), INT_RESULT);
    InterpStatus::Success
}

fn eq(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, true)
}

fn ne(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::StringVar;
    use crate::vm::StateRef;
    use ac2_db::{StringTable, StringTableString};
    use std::ops::Deref;

    struct Tables(StringTable);

    impl ScriptData for Tables {
        fn wstate(&self, _did: u32) -> Option<StateRef> {
            None
        }
        fn string_table(&self, did: u32) -> Option<Box<dyn Deref<Target = StringTable>>> {
            (did == 0x2500_0001).then(|| Box::new(Box::new(self.0.clone())) as _)
        }
    }

    fn piece(s: &str) -> Option<Vec<u16>> {
        Some(wide(s))
    }

    fn data() -> Tables {
        let mut t = StringTable::default();
        let plain = StringTableString {
            pieces: vec![piece("Fire "), piece("Ward \\q[ms]")],
            ..Default::default()
        };
        let vars = StringTableString {
            pieces: vec![piece("Hit "), piece(" times")],
            variables: vec![7],
            ..Default::default()
        };
        t.strings.insert(1, plain);
        t.strings.insert(2, vars);
        Tables(t)
    }

    fn info(string_id: u32) -> StringInfo {
        StringInfo {
            string_id,
            table_id: 0x2500_0001,
            ..StringInfo::default()
        }
    }

    #[test]
    fn meta_letters_strip_at_the_first_unescaped_bracket_and_escapes_unescape() {
        assert_eq!(strip_meta_letters(&wide("Sword  [ms]")), wide("Sword"));
        assert_eq!(strip_meta_letters(&wide("a\\[b [x]")), wide("a\\[b"));
        assert_eq!(strip_meta_letters(&wide("none ")), wide("none "));
        assert_eq!(
            unescape_string(&wide("say \\qhi\\q \\[x\\]")),
            wide("say \"hi\" [x]")
        );
        assert_eq!(unescape_string(&wide("end\\")), vec![101, 110, 100, 0]);
    }

    #[test]
    fn inq_string_renders_plain_strings_and_reports_retail_errors() {
        let d = data();
        assert_eq!(inq(&d, &info(1), false), (wide("Fire Ward \""), true));
        assert_eq!(inq(&d, &info(1), true), (wide("Fire Ward \"[ms]"), true));
        let (text, ok) = inq(&d, &info(2), false);
        assert!(!ok);
        assert_eq!(
            text,
            wide("<string table error; token: 0x00000002 tableDID: 0x25000001>")
        );
        let missing = inq_string_internal(&d, &info(9), false).1;
        assert_eq!(missing, Err(GetStringError::MissingString));
        let mut no_table = info(1);
        no_table.table_id = 0x2500_0002;
        assert_eq!(
            inq_string_internal(&d, &no_table, false).1,
            Err(GetStringError::MissingTable)
        );
        assert_eq!(
            inq_string_internal(&d, &StringInfo::default(), false).1,
            Err(GetStringError::Uninitialized)
        );
        let literal = StringInfo {
            literal: Some(wide("lit [m]")),
            ..StringInfo::default()
        };
        assert_eq!(inq(&d, &literal, false), (wide("lit"), true));
    }

    #[test]
    fn is_valid_checks_the_table_string_and_nested_infos() {
        let d = data();
        assert!(valid(&d, &info(1), true));
        assert!(!valid(&d, &info(9), true));
        let mut outer = info(2);
        outer.variables.push(StringVar {
            id: 7,
            kind: 9,
            value: StringVarValue::StringInfo(Box::new(info(9))),
        });
        assert!(!valid(&d, &outer, true));
        assert!(valid(&d, &outer, false));
        let mut same = info(2);
        same.variables = outer.variables.clone();
        assert!(equal(&outer, &same));
        assert!(!equal(&outer, &info(2)));
    }
}
