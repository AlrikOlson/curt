//! spec-truth gate: every measured divergence/bug from the cheatsheet and
//! token-bench chunks stays fixed. Each test names its origin.

use std::io::Write;
use std::process::{Command, Stdio};

fn run_src(cmd: &str, src: &str) -> (String, String, bool) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_curt"))
        .arg(cmd)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn curt");
    child.stdin.as_mut().unwrap().write_all(src.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

fn run_ok(src: &str, expected: &str) {
    let (out, err, ok) = run_src("run", src);
    assert!(ok, "exit nonzero; stdout={out} stderr={err}");
    assert_eq!(out, expected);
}

/// token-bench interpreter bug: pipe capture must NOT reach inside a
/// parenthesized first stage. This is the exact committed repro from
/// tools/bench/answers/curt_sonnet_r1/s3/10_parse_pairs.curt — a
/// documentation-correct model repair that the old elaborator broke.
#[test]
fn pipe_capture_respects_parens_at_binding() {
    run_ok(
        "s = \"a=1,b=22,c=333\"\ntotal = (s.split \",\") | map (pair -> (pair.split \"=\")[1].int) | sum\nprint total\n",
        "356\n",
    );
}

/// v0.2: the un-parenthesized form pipes the WHOLE left expression —
/// the capture rule is deleted (domain-bench + 4 prior experiments).
#[test]
fn pipe_takes_whole_left_expression() {
    run_ok(
        "s = \"a=1,b=22,c=333\"\ntotal = s.split \",\" | map (p -> (p.split \"=\")[1].int) | sum\nprint total\n",
        "356\n",
    );
}

/// v0.2: a bare lambda pipe stage works and its body stops at `|`
/// (the five-experiment lambda-swallow footgun, deleted by grammar).
#[test]
fn bare_lambda_stage_terminates_at_pipe() {
    run_ok("print ([1, 2, 3] | map x -> x * x | sum)\n", "14\n");
}

/// v0.2: rescue applies to the whole left call — the sheet example
/// `data = fs.read p ? fallback` now means (fs.read p) ? fallback.
#[test]
fn rescue_takes_whole_left_call() {
    run_ok("data = fs.read \"definitely_missing.txt\" ? \"fb\"\nprint data\n", "fb\n");
}

/// domain-bench: err is a match type-pattern, binding the message.
#[test]
fn err_type_pattern_in_match() {
    run_ok(
        "v = \"x\".int\nmsg = match v { err e -> \"bad\", int n -> \"ok\" }\nprint msg\n",
        "bad\n",
    );
}

/// domain-bench: block lambdas inside call parens keep their newlines.
#[test]
fn block_lambda_in_call_parens() {
    run_ok(
        "xs = [\"a b c\", \"bad\"]\nok = xs | keep (l -> {\n  p = l.words\n  p.len == 3\n})\nprint ok.len\n",
        "1\n",
    );
}

/// domain-bench: maps answer field syntax with key lookup.
#[test]
fn map_field_access_falls_back_to_key() {
    run_ok("m = ('{\"a\": 5}').json\nprint m.a\nprint (m.zz ? 0)\n", "5\n0\n");
}

/// domain-bench: "{}" is literal text; single-quoted strings are raw.
#[test]
fn literal_braces_and_single_quoted_strings() {
    run_ok("print \"{}\"\nprint 'no {hole} here'\n", "{}\nno {hole} here\n");
}

/// v0.2: rescue inside the print parens (statement-level rescue on unit
/// is now a loud checker error — see infer rescue-on-unit).
#[test]
fn rescue_capture_respects_parens() {
    run_ok("m = \"a b a\".words.counts\nprint (m[\"z\"] ? 0)\n", "0\n");
}

/// token-bench top failure cause (8 cells): list concatenation with `+`/`+=`.
#[test]
fn list_concat_plus_and_compound() {
    run_ok(
        "seen = []\nseen = seen + [3]\nseen += [1, 4]\nprint (seen | map str | join \" \")\n",
        "3 1 4\n",
    );
}

/// SPEC §5: two-arg range mis-elaborated (`1 is not callable`).
#[test]
fn range_two_arg_forms() {
    run_ok("print (range 1 4)\nfor i in range 2 { print i }\n", "[1, 2, 3]\n0\n1\n");
    let (_, _, ok) = run_src("check", "print (range 1 4)\n");
    assert!(ok, "checker must accept two-arg range");
}

/// cheatsheet divergence 3: mixed-type list literal ran but failed check.
#[test]
fn mixed_list_literal_checks_as_union() {
    let (out, err, ok) = run_src("check", "tag v = match v { int n -> n + 1, str s -> s.len }\nfor x in [7, \"ok\", 12] { print tag x }\n");
    assert!(ok, "mixed list must typecheck as [int | str]: {out}{err}");
}

/// token-bench failure (2 cells, resisted repair): newline after `[`.
#[test]
fn multiline_list_literal() {
    run_ok(
        "items = [\n  {name:\"a\", qty:4},\n  {name:\"b\", qty:2}\n]\nprint items.len\n",
        "2\n",
    );
}

/// Postel: C/Rust char-literal habit canonicalizes to a 1-char string.
#[test]
fn postel_char_literal() {
    run_ok("n = 0\nfor c in \"abca\".chars { if c == 'a' { n += 1 } }\nprint n\n", "2\n");
}

/// Postel: `++` string-concat habit canonicalizes to `+`.
#[test]
fn postel_plus_plus() {
    run_ok("print (\"x\" ++ \"y\")\n", "xy\n");
}

/// token-bench failure (1 cell): user bindings shadow capability namespaces.
#[test]
fn user_binding_shadows_fs_namespace() {
    run_ok("fs = [1.5, 2.5]\nprint fs.max\n", "2.5\n");
}

/// lang-v03-features: string-keyed map literals (tournament winner, 15 tok).
#[test]
fn map_literal_string_keys() {
    run_ok(
        "m = {\"a\": 1, \"b\": 2}\nprint m[\"a\"]\nprint m.b\nm[\"c\"] = 3\nprint m.len\n",
        "1\n2\n3\n",
    );
    // missing key rescues like any err value
    run_ok("m = {\"a\": 1}\nprint (m[\"zz\"] ? 9)\n", "9\n");
    // interop with the pairs verb
    run_ok("cfg = {\"x\": 1, \"y\": 2}\nprint (cfg.pairs | map .v | sum)\n", "3\n");
    // fmt round-trips the literal unchanged
    let (out, _, ok) = run_src("fmt", "m = {\"a\": 1}\nprint m[\"a\"]\n");
    assert!(ok);
    assert_eq!(out, "m = {\"a\": 1}\nprint m[\"a\"]\n");
}

/// lang-v03-features: a record literal stays a record (no silent map merge).
#[test]
fn record_literal_distinct_from_map_literal() {
    run_ok("r = {a: 1}\nprint r.a\n", "1\n");
    let (_, err, ok) = run_src("run", "r = {a: 1}\nprint r[\"a\"]\n");
    assert!(!ok, "records must not be string-indexable");
    assert!(err.contains("not indexable"), "got: {err}");
}

/// multiline-maplit: the largest v4-lane failure class — models write
/// map literals across lines (synth-v2 repair triples + 10/45 v4 cells).
#[test]
fn map_literal_multiline() {
    run_ok(
        "m = {\n  \"a\": 1,\n  \"b\": 2,\n}\nprint m[\"a\"]\nprint m.len\n",
        "1\n2\n",
    );
    // blocks opening with an annotated binding still parse as blocks
    run_ok("f x = {\n  y: int = 2\n  ret y + x\n}\nprint (f 1)\n", "3\n");
    // fmt round-trips the multi-line form verbatim
    let src = "m = {\n  \"a\": 1\n}\nprint m[\"a\"]\n";
    let (out, _, ok) = run_src("fmt", src);
    assert!(ok);
    assert_eq!(out, src);
}

/// multiline-maplit (v0.3.1): records too — newline-separated, comma-less
/// fields are the exact v4 failing shape; annotated-binding blocks are
/// disambiguated by the bare `=` before end-of-line.
#[test]
fn record_literal_multiline() {
    run_ok(
        "r = {\n  name: \"amy\"\n  age: 34\n}\nprint r.name\nprint r.age\n",
        "amy\n34\n",
    );
    run_ok(
        "u = [\"a 1\", \"b 2\"] | map x -> {\n  name: x.words[0]\n  num: x.words[1].int\n}\nprint (u | map .num | sum)\n",
        "3\n",
    );
}
