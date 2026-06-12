//! densify-lint gates: each rule class is EQUIVALENCE-VERIFIED — applying
//! the lint's replacement payloads must preserve program output exactly
//! (golden preservation), and the rewritten program must be strictly
//! cheaper in o200k tokens. The flagship's four runtime goldens must
//! survive a fully-linted rewrite byte-identically.

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_src(src: &str, flags: &[&str], args: &[&str]) -> (String, bool) {
    let dir = std::env::temp_dir().join(format!("curt-lint-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("p{}.curt", src.len()));
    std::fs::write(&f, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_curt"));
    cmd.arg("run").args(flags).arg(&f).args(args);
    cmd.current_dir(root().join("tests/fixtures"));
    let out = cmd.output().expect("spawn curt");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.success())
}

/// Apply every replacement payload the lint emits for `src`.
fn apply_lint(src: &str) -> (String, usize) {
    let findings = curt::lint::lint(src).expect("lints");
    let mut lines: Vec<String> = src.lines().map(|l| l.to_string()).collect();
    let mut applied = 0;
    for d in &findings {
        if let Some(reps) = &d.replacement {
            for (line, new) in reps {
                lines[*line as usize - 1] = new.clone();
            }
            applied += 1;
        }
    }
    (lines.join("\n") + "\n", applied)
}

/// The core equivalence gate: lint fixes preserve output and shrink tokens.
fn assert_lint_preserves(src: &str, expect_rules: usize) {
    let (patched, applied) = apply_lint(src);
    assert_eq!(applied, expect_rules, "expected {expect_rules} applied fixes\n{patched}");
    assert!(patched.len() < src.len(), "patched must be smaller:\n{patched}");
    let (before, ok_b) = run_src(src, &[], &[]);
    let (after, ok_a) = run_src(&patched, &[], &[]);
    assert!(ok_b && ok_a, "both must run clean\nbefore ok={ok_b} after ok={ok_a}\n{patched}");
    assert_eq!(before, after, "output must be preserved\n{patched}");
}

#[test]
fn match_rescue_preserves_output() {
    // both paths: conversion succeeds and fails
    assert_lint_preserves(
        "v = \"7\"\nr = match v.int { err _ -> 0, n -> n }\nprint r\nw = \"x\"\ns = match w.int { err _ -> -1, m -> m }\nprint s\n",
        2,
    );
}

#[test]
fn bool_if_preserves_output() {
    assert_lint_preserves(
        "x = \"hello\"\nb = if (x.len) > 3 { true } else { false }\nprint b\nc = if (x.len) > 99 { true } else { false }\nprint c\n",
        2,
    );
}

#[test]
fn bool_if_negated_preserves_output() {
    assert_lint_preserves(
        "x = \"hello\"\nb = if (x.len) > 3 { false } else { true }\nprint b\n",
        1,
    );
}

#[test]
fn fold_sum_preserves_output() {
    // UFCS receiver form and pipe-stage form, both operand orders
    assert_lint_preserves(
        "total = [1,2,3].fold 0 acc q -> acc + q\nprint total\nys = [4,5,6] | fold 0 a b -> b + a\nprint ys\n",
        2,
    );
}

#[test]
fn used_err_binder_is_not_rewritten() {
    // `err e` with e USED in the body must NOT lint to a rescue
    let src = "v = \"x\"\nr = match v.int { err e -> e.len, n -> n }\nprint r\n";
    let findings = curt::lint::lint(src).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn nonzero_fold_seed_is_not_rewritten() {
    let src = "t = [1,2,3].fold 10 acc q -> acc + q\nprint t\n";
    let findings = curt::lint::lint(src).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn flagship_lint_fixes_keep_all_four_goldens() {
    let src = std::fs::read_to_string(root().join("corpus/22_logmill.curt")).unwrap();
    let (patched, applied) = apply_lint(&src);
    assert!(applied >= 1, "the proven think:119 miss must be flagged");
    for args in [
        vec!["logmill.json"],
        vec![],
        vec!["badjob.json"],
        vec!["ghost.json"],
    ] {
        let (before, ok_b) = run_src(&src, &["--fs"], &args);
        let (after, ok_a) = run_src(&patched, &["--fs"], &args);
        assert!(ok_b && ok_a, "flagship must run clean, args {args:?}");
        assert_eq!(before, after, "flagship golden changed under lint fixes, args {args:?}");
    }
}
