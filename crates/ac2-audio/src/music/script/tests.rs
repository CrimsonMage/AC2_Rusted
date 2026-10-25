//! The AudioVBScript subset on synthetic scripts (no game data): each construct the corpus uses,
//! and the errors the subset reports.

use super::{parse, Instance, ScriptHost, Value};

/// A host that records what the script calls, with a fixed groove level.
#[derive(Default)]
struct Recorder {
    groove: f64,
    methods: Vec<(String, String, Vec<f64>)>,
}

impl ScriptHost for Recorder {
    fn constant(&self, name: &str) -> Option<f64> {
        name.eq_ignore_ascii_case("atfinish").then_some(256.0)
    }

    fn function(&mut self, name: &str, args: &[f64]) -> Result<f64, String> {
        match (name.to_ascii_lowercase().as_str(), args.len()) {
            ("getmastergroovelevel", 0) => Ok(self.groove),
            _ => Err(format!("no function {name}")),
        }
    }

    fn method(&mut self, object: &str, method: &str, args: &[f64]) -> Result<(), String> {
        self.methods
            .push((object.to_string(), method.to_string(), args.to_vec()));
        Ok(())
    }
}

/// The corpus's shape: a groove read, an `If`/`ElseIf` on two thresholds, a method per branch.
const CHOOSE: &str = "Dim groovereport\n\
    Sub ChooseGroove\n\
    groovereport = GetMasterGrooveLevel()\n\
    If groovereport < 5 Then\n\
    Seg_Low.play(AtFinish)\n\
    End If\n\
    If groovereport > 9 Then\n\
    Seg_High.play(AtFinish)\n\
    ElseIf groovereport > 4 Then\n\
    Seg_Mid.play(AtFinish)\n\
    End If\n\
    End Sub\n";

fn run(source: &str, routine: &str, host: &mut Recorder) -> Instance {
    let program = parse(source).expect("the script parses");
    let mut instance = Instance::default();
    instance
        .call(&program, routine, host)
        .expect("the routine runs");
    instance
}

fn played(host: &Recorder) -> Vec<String> {
    host.methods
        .iter()
        .map(|(object, method, args)| format!("{object}.{method}{args:?}"))
        .collect()
}

#[test]
fn the_corpus_routine_plays_the_branch_its_groove_level_selects() {
    for (groove, expected) in [
        (0.0, vec!["Seg_Low.play[256.0]"]),
        (4.0, vec!["Seg_Low.play[256.0]"]),
        (5.0, vec!["Seg_Mid.play[256.0]"]),
        (9.0, vec!["Seg_Mid.play[256.0]"]),
        (10.0, vec!["Seg_High.play[256.0]"]),
    ] {
        let mut host = Recorder {
            groove,
            ..Recorder::default()
        };
        run(CHOOSE, "ChooseGroove", &mut host);
        assert_eq!(played(&host), expected, "at groove {groove}");
    }
}

#[test]
fn a_routine_name_is_case_insensitive_and_a_bare_name_calls_a_routine() {
    let source = "Sub Inner\nSeg.play(AtFinish)\nEnd Sub\n\
        Sub Outer\ninner\nINNER\nEnd Sub\n";
    let mut host = Recorder::default();
    run(source, "OUTER", &mut host);
    assert_eq!(host.methods.len(), 2);
}

#[test]
fn an_unset_variable_reads_as_zero_and_arithmetic_follows_precedence() {
    let source = "Dim total\nSub R\nunset = 2 + 3 * 4 - 6 / 2\ntotal = unset * -1\nEnd Sub\n";
    let program = parse(source).unwrap();
    let mut instance = Instance::default();
    instance
        .call(&program, "R", &mut Recorder::default())
        .unwrap();
    assert_eq!(instance.variable("unset"), Some(Value::Number(11.0)));
    assert_eq!(instance.variable("total"), Some(Value::Number(-11.0)));
    let never = "Sub R\nx = nothing + 1\nEnd Sub\n";
    let mut instance = Instance::default();
    instance
        .call(&parse(never).unwrap(), "R", &mut Recorder::default())
        .unwrap();
    assert_eq!(instance.variable("x"), Some(Value::Number(1.0)));
}

#[test]
fn the_comparisons_all_test_numbers() {
    let source = "Sub R\n\
        a = 1 = 1\nb = 1 <> 2\nc = 2 <= 2\nd = 3 >= 4\n\
        If a Then\nok = 1\nEnd If\n\
        If d Then\nbad = 1\nEnd If\n\
        End Sub\n";
    let program = parse(source).unwrap();
    let mut instance = Instance::default();
    instance
        .call(&program, "R", &mut Recorder::default())
        .unwrap();
    assert_eq!(instance.variable("a"), Some(Value::Bool(true)));
    assert_eq!(instance.variable("b"), Some(Value::Bool(true)));
    assert_eq!(instance.variable("c"), Some(Value::Bool(true)));
    assert_eq!(instance.variable("d"), Some(Value::Bool(false)));
    assert_eq!(instance.variable("ok"), Some(Value::Number(1.0)));
    assert_eq!(
        instance.variable("bad"),
        None,
        "the false branch did not run"
    );
}

#[test]
fn the_first_true_branch_runs_and_only_it() {
    let source = "Sub R\nx = -3\nIf x > 0 Then\na = 1\nElseIf x > -5 Then\nb = 1\nElseIf x > -10 Then\nc = 1\nEnd If\nEnd Sub\n";
    let program = parse(source).unwrap();
    let mut instance = Instance::default();
    instance
        .call(&program, "R", &mut Recorder::default())
        .unwrap();
    assert_eq!(instance.variable("a"), None);
    assert_eq!(instance.variable("b"), Some(Value::Number(1.0)));
    assert_eq!(instance.variable("c"), None);
}

#[test]
fn module_variables_persist_per_instance_and_a_new_instance_starts_clean() {
    let source = "Dim count\nSub Bump\ncount = count + 1\nEnd Sub\n";
    let program = parse(source).unwrap();
    let mut first = Instance::default();
    let mut host = Recorder::default();
    first.call(&program, "Bump", &mut host).unwrap();
    first.call(&program, "Bump", &mut host).unwrap();
    assert_eq!(first.variable("count"), Some(Value::Number(2.0)));
    let mut second = Instance::default();
    second.call(&program, "Bump", &mut host).unwrap();
    assert_eq!(second.variable("count"), Some(Value::Number(1.0)));
}

#[test]
fn a_missing_routine_a_missing_function_and_a_bad_method_are_errors_with_their_line() {
    let program = parse("Sub R\nx = 1\nEnd Sub\n").unwrap();
    let err = Instance::default()
        .call(&program, "Nope", &mut Recorder::default())
        .unwrap_err();
    assert!(err.message.contains("Nope"), "{err}");

    let program = parse("Sub R\nx = 1\ny = Unknown()\nEnd Sub\n").unwrap();
    let err = Instance::default()
        .call(&program, "R", &mut Recorder::default())
        .unwrap_err();
    assert_eq!(err.line, 3, "{err}");

    let program = parse("Sub R\nSeg.stop(0)\nEnd Sub\n").unwrap();
    // A host that rejects a method reports it with the line.
    struct Reject;
    impl ScriptHost for Reject {
        fn constant(&self, _: &str) -> Option<f64> {
            None
        }
        fn function(&mut self, _: &str, _: &[f64]) -> Result<f64, String> {
            Err("no".into())
        }
        fn method(&mut self, _: &str, method: &str, _: &[f64]) -> Result<(), String> {
            Err(format!("method {method} is not in the subset"))
        }
    }
    let err = Instance::default()
        .call(&program, "R", &mut Reject)
        .unwrap_err();
    assert!(err.message.contains("stop"), "{err}");
    assert_eq!(err.line, 2, "{err}");
}

#[test]
fn a_routine_that_calls_itself_stops_at_the_depth_limit() {
    let program = parse("Sub R\nR\nEnd Sub\n").unwrap();
    let err = Instance::default()
        .call(&program, "R", &mut Recorder::default())
        .unwrap_err();
    assert!(err.message.contains("nested too deeply"), "{err}");
}

#[test]
fn a_long_routine_is_bounded_by_its_statement_count() {
    let body: String = (0..10_050).map(|i| format!("x = {i}\n")).collect();
    let program = parse(&format!("Sub R\n{body}End Sub\n")).unwrap();
    let err = Instance::default()
        .call(&program, "R", &mut Recorder::default())
        .unwrap_err();
    assert!(err.message.contains("too many statements"), "{err}");
}

#[test]
fn division_by_zero_and_boolean_arithmetic_are_errors() {
    let program = parse("Sub R\nx = 1 / 0\nEnd Sub\n").unwrap();
    assert!(Instance::default()
        .call(&program, "R", &mut Recorder::default())
        .is_err());
    let program = parse("Sub R\nx = (1 < 2) + 1\nEnd Sub\n").unwrap();
    let err = Instance::default()
        .call(&program, "R", &mut Recorder::default())
        .unwrap_err();
    assert!(err.message.contains("boolean"), "{err}");
}

#[test]
fn constructs_outside_the_subset_do_not_parse() {
    for (source, why) in [
        (
            "Sub R\nIf x Then\ny = 1\nElse\ny = 2\nEnd If\nEnd Sub\n",
            "Else",
        ),
        ("Sub R\nSelect Case x\nEnd Select\nEnd Sub\n", "Select"),
        ("Sub R\nx = \"text\"\nEnd Sub\n", "a string literal"),
        ("Sub R\nFunction F\nEnd Function\nEnd Sub\n", "Function"),
        ("x = 1\n", "a statement at module level"),
        ("Sub R\nIf x Then y = 1\nEnd Sub\n", "a single-line If"),
        ("Sub R\nx = 1\n", "an unclosed routine"),
    ] {
        assert!(parse(source).is_err(), "{why} is rejected");
    }
}

#[test]
fn a_comment_and_blank_lines_do_not_change_the_script() {
    let source = "' header\n\nDim a\n\nSub R\n  ' note\n  a = 2 ' trailing\nEnd Sub\n";
    let mut instance = Instance::default();
    instance
        .call(&parse(source).unwrap(), "R", &mut Recorder::default())
        .unwrap();
    assert_eq!(instance.variable("a"), Some(Value::Number(2.0)));
}
