//! interp-c gates: inference goldens (incl. arity resolution + union
//! narrowing), exhaustiveness diagnostics, expand type-reveal, corpus check.

use std::collections::HashMap;
use std::path::PathBuf;

fn check_src(src: &str) -> Result<curt::infer::CheckReport, curt::diag::Diag> {
    curt::infer::check(&curt::parse_source(src).unwrap())
}

fn sigs(src: &str) -> HashMap<String, String> {
    check_src(src).unwrap_or_else(|d| panic!("should check: {src:?} -> {d}")).0.into_iter().collect()
}

fn check_err(src: &str) -> curt::diag::Diag {
    check_src(src).expect_err(&format!("should NOT check: {src:?}"))
}

// ---- the corpus gate ----

#[test]
fn corpus_checks_clean_21_of_21() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".curt"))
        .collect();
    names.sort();
    assert_eq!(names.len(), 21);
    for n in names {
        let src = std::fs::read_to_string(dir.join(&n)).unwrap();
        if let Err(d) = check_src(&src) {
            panic!("{n} failed check: {d}");
        }
    }
}

// ---- inference goldens ----

#[test]
fn infer_arithmetic_equation() {
    let s = sigs("hyp a b = (a*a + b*b).sqrt\nprint hyp 3.0 4.0\n");
    assert_eq!(s["hyp"], "(float float -> float)");
}

#[test]
fn infer_recursion_fib() {
    let s = sigs("fib n = if n < 2 { n } else { fib (n-1) + fib (n-2) }\nprint fib 30\n");
    assert_eq!(s["fib"], "(int -> int)");
}

#[test]
fn infer_arity_resolution_print_show() {
    // the spec §2.3 headline case: print (show 2.5)
    let s = sigs("show v = v.str\nprint show 2.5\n");
    assert_eq!(s["show"], "(float -> str)");
}

#[test]
fn infer_arity_resolution_nested_args() {
    // print (hyp 3.0 4.0): surplus args re-nest under hyp
    let src = "hyp a b = (a*a + b*b).sqrt\nprint hyp 3.0 4.0\n";
    assert!(check_src(src).is_ok());
}

#[test]
fn infer_no_partial_application() {
    let d = check_err("add a b = a + b\nv: int = add 1\nprint v\n");
    assert_eq!(d.err, "arity");
    assert!(d.to_string().contains("no partial application"));
}

#[test]
fn infer_pipe_capture_rule() {
    // v0.2: pipe takes the WHOLE left expression (F#/Elixir semantics).
    // `print us | keep ...` is now a loud type error — (print us) is unit —
    // which is the checker catching the print-heads-a-pipe mistake.
    let src = "us = [1, 2, 3]\nprint us | keep (x -> x > 1)\n";
    assert!(check_src(src).is_err(), "print-headed pipes must be a loud type error, not silent capture");
    let good = "us = [1, 2, 3]\nprint (us | keep (x -> x > 1))\n";
    assert!(check_src(good).is_ok());
}

#[test]
fn infer_pipe_stage_appends_piped_value() {
    // NB: an inline lambda used as a pipe stage needs parens — a lambda body
    // extends through `|` (grammar fact, SPEC §2.3 note)
    let src = "v: [int] = [3, 1, 2] | keep (x -> x > 1) | sort\nprint v\n";
    assert!(check_src(src).is_ok());
}

#[test]
fn infer_string_ops() {
    let s = sigs("slug s = s.trim.lower.replace \" \" \"-\"\nprint slug \"A B\"\n");
    assert_eq!(s["slug"], "(str -> str)");
}

#[test]
fn infer_record_nominal() {
    let src = "type Pt = {x float, y float}\np = Pt{x:0.0, y:1.5}\nv: float = p.x\nprint v\n";
    assert!(check_src(src).is_ok());
}

#[test]
fn infer_record_unknown_field_diag() {
    let d = check_err("type Pt = {x float, y float}\np = Pt{x:0.0, y:1.0}\nprint p.z\n");
    assert_eq!(d.err, "unknown_field");
    assert!(d.to_string().contains("available fields: x, y"));
}

#[test]
fn infer_counts_map_types() {
    let src = "m = [\"a\", \"b\", \"a\"].counts\nn: int = m[\"a\"]\nprint n\n";
    assert!(check_src(src).is_ok());
}

#[test]
fn infer_tuple_destructure() {
    let src = "minmax xs = (xs.min, xs.max)\n(lo, hi) = minmax [3, 1, 4]\nv: int = lo\nprint v\n";
    assert!(check_src(src).is_ok());
}

#[test]
fn infer_union_narrowing_in_match() {
    let src = "show :: float | str -> str\nshow v = match v { float x -> \"n {x}\", str s -> s }\nprint show 2.5\n";
    let s = sigs(src);
    assert_eq!(s["show"], "(float | str -> str)");
}

#[test]
fn infer_narrowed_binder_is_member_type() {
    // inside the float arm, x is float (x.sqrt only exists on float)
    let src = "f :: float | str -> float\nf v = match v { float x -> x.sqrt, str s -> 0.0 }\nprint f 2.0\n";
    assert!(check_src(src).is_ok());
}

#[test]
fn infer_int_float_do_not_mix() {
    let d = check_err("v = 1 + 2.5\nprint v\n");
    assert_eq!(d.err, "type_mismatch");
}

#[test]
fn infer_bool_ops_require_bool() {
    let d = check_err("v = 1 and true\nprint v\n");
    assert_eq!(d.err, "type_mismatch");
}

#[test]
fn infer_unknown_name_diag() {
    let d = check_err("print mystery_value_xyz\n");
    assert_eq!(d.err, "unknown_name");
}

#[test]
fn infer_statement_bodied_equation_is_unit() {
    let s = sigs("loop xs = for x in xs { print x }\nloop [1, 2]\n");
    assert!(s["loop"].ends_with("-> ())"), "statement body is unit: {}", s["loop"]);
}

// ---- exhaustiveness (the diagnostic golden) ----

#[test]
fn non_exhaustive_union_match_is_error_with_fix() {
    let d = check_err("f :: float | str -> int\nf v = match v { float x -> 1 }\nprint f 1.0\n");
    assert_eq!(d.err, "non_exhaustive_match");
    let s = d.to_string();
    assert!(s.starts_with("{\"err\":\"non_exhaustive_match\""), "JSON shape: {s}");
    assert!(s.contains("does not cover: str"), "names the missing member: {s}");
    assert!(s.contains("add an arm"), "fix-suggesting: {s}");
}

#[test]
fn catchall_makes_union_match_exhaustive() {
    let src = "f :: float | str -> int\nf v = match v { float x -> 1, _ -> 0 }\nprint f 1.0\n";
    assert!(check_src(src).is_ok());
}

#[test]
fn literal_arm_counts_for_its_member_v01_looseness() {
    // SPEC §2.3 note: corpus 19_parser relies on this
    let src = "f :: float | str -> int\nf v = match v { float x -> 1, \"(\" -> 2 }\nprint f 1.0\n";
    assert!(check_src(src).is_ok());
}

// ---- identifier lint ----

#[test]
fn multi_token_identifier_warns() {
    let (_, warnings) = check_src("user_count_total = 5\nprint user_count_total\n").unwrap();
    assert!(warnings.iter().any(|w| w.contains("user_count_total")), "lint fires: {warnings:?}");
}

#[test]
fn single_token_identifier_no_warning() {
    let (_, warnings) = check_src("buf = 5\nprint buf\n").unwrap();
    assert!(warnings.is_empty(), "no lint for 1-token names: {warnings:?}");
}

// ---- expand type-reveal ----

#[test]
fn expand_reveals_equation_signature() {
    let src = "hyp a b = (a*a + b*b).sqrt\nprint hyp 3.0 4.0\n";
    let ast = curt::parse_source(src).unwrap();
    let sigs: HashMap<String, String> = curt::infer::check(&ast).unwrap().0.into_iter().collect();
    let out = curt::expand::expand(&ast, &sigs);
    assert!(out.contains("hyp :: (float float -> float)"), "{out}");
}

#[test]
fn expand_reveals_recursive_signature() {
    let src = "fib n = if n < 2 { n } else { fib (n-1) + fib (n-2) }\nprint fib 30\n";
    let ast = curt::parse_source(src).unwrap();
    let sigs: HashMap<String, String> = curt::infer::check(&ast).unwrap().0.into_iter().collect();
    let out = curt::expand::expand(&ast, &sigs);
    assert!(out.contains("fib :: (int -> int)"), "{out}");
}

#[test]
fn expand_reveals_union_signature() {
    let src = "show :: float | str -> str\nshow v = match v { float x -> \"n\", str s -> s }\nprint show 2.5\n";
    let ast = curt::parse_source(src).unwrap();
    let sigs: HashMap<String, String> = curt::infer::check(&ast).unwrap().0.into_iter().collect();
    let out = curt::expand::expand(&ast, &sigs);
    assert!(out.contains("show :: (float | str -> str)"), "{out}");
}
