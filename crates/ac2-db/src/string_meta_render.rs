//! `StringTableMetaLanguage::RenderString` and `RenderNodes` (`metalanguage.cpp`): parse a
//! string's pieces with its variable values, then write the text, the variables (capitalised
//! after `^`, hidden with a negative label) and each choice block's best choice for its
//! variable's letters.
//!
//! A choice's letters score against the variable's (`GetMatchScore`): 2 for each letter the
//! variable has, 2 for each `!x` whose `x` it lacks; a choice without letters scores 1 (the
//! default). The highest positive score wins, a later choice winning a tie.

use crate::string_meta_lex::Lexer;
use crate::string_meta_parse::{parse, Node, NodeKind, RenderErr, RenderErrorCode};
use crate::string_meta_text::{capitalize_first_letter, trim_excess_spaces, unescape_string, wide};
use crate::string_state::{treasure_meta_letters, StringState};

const BANG: u16 = b'!' as u16;

/// `wcschr(letters, c)`: the terminator counts as present.
fn has_letter(letters: &[u16], c: u16) -> bool {
    c == 0 || letters.contains(&c)
}

/// `StringTableMetaLanguage::GetMatchScore(org, match)`.
pub fn get_match_score(org: &[u16], choice: &[u16]) -> i32 {
    if choice.is_empty() {
        return 1;
    }
    let mut score = 0;
    let mut i = 0;
    while i < choice.len() {
        if choice[i] == BANG {
            i += 1;
            if !has_letter(org, choice.get(i).copied().unwrap_or(0)) {
                score += 2;
            }
        } else if has_letter(org, choice[i]) {
            score += 2;
        }
        i += 1;
    }
    score
}

/// `StringTableMetaLanguage::ChooseChoice(var, choiceblock)`.
fn choose_choice<'n>(var: &Node, block: &'n Node) -> Option<&'n Node> {
    let mut best = None;
    let mut best_score = -1;
    for choice in &block.children {
        let score = get_match_score(&var.flags, &choice.flags);
        if score >= best_score && score > 0 {
            best = Some(choice);
            best_score = score;
        }
    }
    best
}

/// `StringTableMetaLanguage::TakeValidLetters(valid, letters)`: the letters that are valid.
fn take_valid_letters(valid: &[u16], letters: &[u16]) -> Vec<u16> {
    letters
        .iter()
        .copied()
        .filter(|&c| has_letter(valid, c))
        .collect()
}

/// The variables named `PRE`, `MID`/`NAME` and `POST` (`RenderString`'s scan of the var
/// names; a later name wins).
#[derive(Debug, Clone, Copy, Default)]
struct ItemParts {
    pre: Option<usize>,
    mid: Option<usize>,
    post: Option<usize>,
}

fn render_error(code: RenderErrorCode, node: &Node, message: &'static str) -> RenderErr {
    RenderErr {
        code,
        pos: node.pos,
        message,
    }
}

/// `StringTableMetaLanguage::RenderNodes(root, finalletters, outsideVars, pre, mid, post)`.
/// The output is cleared only after the duplicate-label checks, and a choice-block error
/// leaves what was rendered before it, as retail's result string does.
fn render_nodes(
    root: &[Node],
    out: &mut Vec<u16>,
    outside: &mut Vec<Vec<u16>>,
    parts: ItemParts,
    state: Option<&StringState>,
) -> Result<(), RenderErr> {
    let vars: Vec<&Node> = root.iter().filter(|n| n.kind == NodeKind::Var).collect();
    let has_default_block = root
        .iter()
        .any(|n| n.kind == NodeKind::ChoiceBlock && n.id == 0);
    let mut item_rule = false;
    for node in root.iter().filter(|n| n.kind == NodeKind::OutsideVar) {
        item_rule |= node.text == wide("item");
        outside.push(node.text.clone());
    }
    for (i, var) in vars.iter().enumerate().skip(1) {
        for earlier in vars[..i].iter().filter(|w| w.id == var.id) {
            if earlier.id != 0 {
                let message = "multiple variables with the same ID; ambiguity";
                return Err(render_error(RenderErrorCode::DuplicateIds, var, message));
            }
            if has_default_block {
                let message = "multiple un-numbered variables; ambiguity";
                return Err(render_error(RenderErrorCode::DuplicateIds, var, message));
            }
        }
    }
    out.clear();
    let mut item = [Vec::new(), Vec::new(), Vec::new()];
    let mut counter = 0usize;
    for node in root {
        match node.kind {
            NodeKind::Text => out.extend_from_slice(&node.text),
            NodeKind::Var if node.id >= 0 => {
                if item_rule {
                    if Some(counter) == parts.pre {
                        item[0] = node.flags.clone();
                    } else if Some(counter) == parts.mid {
                        item[1] = node.flags.clone();
                    } else if Some(counter) == parts.post {
                        item[2] = node.flags.clone();
                    }
                    counter += 1;
                }
                if node.force_capital {
                    out.extend(capitalize_first_letter(&node.text));
                } else {
                    out.extend_from_slice(&node.text);
                }
            }
            NodeKind::Var | NodeKind::OutsideVar => {}
            NodeKind::ChoiceBlock => {
                let var = vars.iter().find(|v| v.id == node.id).or_else(|| {
                    (node.id == 0)
                        .then(|| vars.iter().find(|v| v.id == -1))
                        .flatten()
                });
                let Some(var) = var else {
                    let message = if node.id == 0 {
                        "no default variable exists to match with this default choice block"
                    } else {
                        "no variable has the same ID as this choice block"
                    };
                    return Err(render_error(RenderErrorCode::SyntaxError, node, message));
                };
                if let Some(choice) = choose_choice(var, node) {
                    out.extend_from_slice(&choice.text);
                }
            }
        }
    }
    if item_rule {
        let valid = treasure_meta_letters(state);
        let mut letters = Vec::new();
        for (valid, flags) in valid.iter().zip(&item) {
            letters.extend(take_valid_letters(valid, flags));
        }
        out.extend(wide(" ["));
        out.extend(letters);
        out.push(u16::from(b']'));
    }
    Ok(())
}

/// `StringTableMetaLanguage::RenderString(textparts, varparts, varnames, result, flags, 1)`.
///
/// `out` holds the caller's text going in (retail's result string) and the rendered text on
/// success; on failure it keeps what retail leaves there. Returns the `{{...}}` blocks'
/// texts. With `PRE`/`POST` and `MID`/`NAME` among the variable names (treasure names), the
/// `PRE` and `POST` values are unescaped first. Unless a `{{keepspaces}}` block is present,
/// the result's leading whitespace is trimmed and runs of blanks collapse.
pub fn render_string(
    pieces: &[Option<Vec<u16>>],
    values: &[Vec<u16>],
    variable_names: &[Vec<u16>],
    state: Option<&StringState>,
    out: &mut Vec<u16>,
) -> Result<Vec<Vec<u16>>, RenderErr> {
    let mut parts = ItemParts::default();
    for (i, name) in variable_names.iter().enumerate() {
        match String::from_utf16_lossy(name).as_str() {
            "PRE" => parts.pre = Some(i),
            "MID" | "NAME" => parts.mid = Some(i),
            "POST" => parts.post = Some(i),
            _ => {}
        }
    }
    let lexer = if parts.mid.is_some() && (parts.pre.is_some() || parts.post.is_some()) {
        let mut values = values.to_vec();
        for i in [parts.pre, parts.post].into_iter().flatten() {
            if let Some(value) = values.get_mut(i) {
                *value = unescape_string(value);
            }
        }
        Lexer::load(pieces, &values)
    } else {
        Lexer::load(pieces, values)
    };
    let root = parse(lexer, state)?;
    let mut outside = Vec::new();
    render_nodes(&root, out, &mut outside, parts, state)?;
    if !outside.contains(&wide("keepspaces")) {
        trim_excess_spaces(out);
    }
    Ok(outside)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(pieces: &[&str], values: &[&str], names: &[&str]) -> Result<String, RenderErr> {
        let pieces: Vec<_> = pieces.iter().map(|p| Some(wide(p))).collect();
        let values: Vec<_> = values.iter().map(|v| wide(v)).collect();
        let names: Vec<_> = names.iter().map(|v| wide(v)).collect();
        let mut out = wide("<error>");
        render_string(&pieces, &values, &names, None, &mut out)
            .map(|_| String::from_utf16_lossy(&out))
    }

    fn ok(pieces: &[&str], values: &[&str]) -> String {
        render(pieces, values, &[]).expect("renders")
    }

    #[test]
    fn scores_count_present_and_absent_letters() {
        assert_eq!(get_match_score(&wide("mnM"), &wide("")), 1);
        assert_eq!(get_match_score(&wide("mnM"), &wide("m")), 2);
        assert_eq!(get_match_score(&wide("mnM"), &wide("!n")), 0);
        assert_eq!(get_match_score(&wide(""), &wide("!n")), 2);
        assert_eq!(get_match_score(&wide("fn"), &wide("fx")), 2);
        // A trailing `!` tests the terminator, which is always "present".
        assert_eq!(get_match_score(&wide(""), &wide("!")), 0);
    }

    #[test]
    fn gender_and_article_choices_follow_the_variable_letters() {
        let says = ["#1:{The[!n]} #1:", " says, \\q#2:", "\\q"];
        assert_eq!(ok(&says, &["Bob [M]", "hi"]), "Bob says, \"hi\"");
        assert_eq!(ok(&says, &["guard", "hi"]), "The guard says, \"hi\"");
        let pronoun = ["Ask ", ". {He[m]|She[f]|It} knows."];
        assert_eq!(ok(&pronoun, &["Ann [F]"]), "Ask Ann. She knows.");
        assert_eq!(ok(&pronoun, &["Bob [M]"]), "Ask Bob. He knows.");
        assert_eq!(ok(&pronoun, &["a rock"]), "Ask a rock. It knows.");
    }

    #[test]
    fn plurals_choose_by_the_singular_flag() {
        let days = ["#1:", " #1:{days|day[1]}"];
        assert_eq!(ok(&days, &["1"]), "1 day");
        assert_eq!(ok(&days, &["3"]), "3 days");
        assert_eq!(ok(&days, &["0"]), "0 days");
    }

    #[test]
    fn hidden_and_capitalised_variables() {
        // `#-1:` hides a variable that still drives an unlabelled block.
        assert_eq!(ok(&["#-1:", "{coin[1]|coins}"], &["1"]), "coin");
        assert_eq!(ok(&["^", " waves."], &["the guard"]), "The guard waves.");
        assert_eq!(ok(&["#2:^", " #2:{is[1]|are}"], &["one"]), "One are");
    }

    #[test]
    fn spaces_collapse_unless_kept() {
        assert_eq!(ok(&["  a  ", "  b"], &[""]), "a b");
        assert_eq!(ok(&["{{keepspaces}}  a  b"], &[]), "  a  b");
    }

    #[test]
    fn treasure_names_append_their_valid_letters() {
        let names = ["PRE", "NAME", "POST"];
        let got = render(
            &["{{item}}", " ", " ", ""],
            &["Ancient [vx]", "Sword [mz]", "of Fire [s]"],
            &names,
        );
        assert_eq!(got.unwrap(), "Ancient Sword of Fire [vms]");
    }

    #[test]
    fn errors_keep_retail_output() {
        // Duplicate labels fail before the output is cleared.
        let got = render(&["#1:", "#1:", ""], &["a", "b"], &[]);
        assert_eq!(got.unwrap_err().code, RenderErrorCode::DuplicateIds);
        // Two unlabelled variables are fine without a default block...
        assert_eq!(ok(&["", " and ", ""], &["a", "b"]), "a and b");
        // ...but ambiguous with one.
        let got = render(&["", " ", " {x|y[1]}"], &["a", "b"], &[]);
        assert_eq!(got.unwrap_err().code, RenderErrorCode::DuplicateIds);
        // A block without its variable keeps the text rendered before it.
        let pieces: Vec<_> = ["x #3:{a|b[1]}"].iter().map(|p| Some(wide(p))).collect();
        let mut out = wide("<error>");
        let err = render_string(&pieces, &[], &[], None, &mut out).unwrap_err();
        assert_eq!(err.code, RenderErrorCode::SyntaxError);
        assert_eq!(String::from_utf16_lossy(&out), "x ");
        // A syntax error leaves the caller's text.
        let pieces = vec![Some(wide("{a||b}"))];
        let mut out = wide("<error>");
        assert!(render_string(&pieces, &[], &[], None, &mut out).is_err());
        assert_eq!(String::from_utf16_lossy(&out), "<error>");
    }
}
