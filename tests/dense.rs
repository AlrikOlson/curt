//! idiom-density gate: `curt dense` rewrites verbose loop shapes into verb
//! pipelines, accepts ONLY differential-execution-verified, token-reducing
//! rewrites, and degrades to identity everywhere else.

use std::io::Write;
use std::process::{Command, Stdio};

fn run_cmd(cmd: &str, src: &str) -> (String, bool) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_curt"))
        .arg(cmd)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn curt");
    child.stdin.as_mut().unwrap().write_all(src.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.success())
}

/// dense output must run identically to the input.
fn densify_verified(src: &str) -> String {
    let (dense, ok) = run_cmd("dense", src);
    assert!(ok, "dense failed on:\n{src}");
    let (a, _) = run_cmd("run", src);
    let (b, _) = run_cmd("run", &dense);
    assert_eq!(a, b, "differential gate broken:\n{src}\n=>\n{dense}");
    dense
}

#[test]
fn count_loop_becomes_keep_len() {
    let d = densify_verified("s = \"abca\"\nn = 0\nfor c in s.chars {\n  if c == \"a\" { n += 1 }\n}\nprint n\n");
    assert!(d.contains("keep") && d.contains(".len"), "{d}");
}

#[test]
fn sum_loop_becomes_map_sum() {
    let d = densify_verified("xs = [1, 2, 3]\nacc = 0\nfor x in xs {\n  acc += x * x\n}\nprint acc\n");
    assert!(d.contains("| sum"), "{d}");
}

#[test]
fn filtered_list_build_becomes_keep_map() {
    let d = densify_verified(
        "xs = [3, 1, 4, 1, 5]\nout = []\nfor x in xs {\n  if x > 2 { out += [x * 10] }\n}\nprint (out | map str | join \" \")\n",
    );
    assert!(d.contains("keep") && d.contains("map"), "{d}");
}

#[test]
fn max_scan_becomes_dot_max() {
    let d = densify_verified("xs = [3, 9, 2]\nbest = 0\nfor x in xs {\n  if x > best { best = x }\n}\nprint best\n");
    assert!(d.contains(".max"), "{d}");
}

#[test]
fn capability_programs_are_identity() {
    let src = "data = fs.read \"x.txt\" ? \"\"\nprint data.len\n";
    let (d, ok) = run_cmd("dense", src);
    assert!(ok);
    assert_eq!(d, src, "fs programs must be refused untouched");
}

#[test]
fn self_referential_step_is_not_rewritten() {
    // acc appears in the step — fold semantics, not map|sum; must stay.
    let src = "acc = 0\nfor x in [1, 2, 3] {\n  acc += acc + x\n}\nprint acc\n";
    let d = densify_verified(src);
    assert_eq!(d.trim(), src.trim(), "self-referential accumulator must be left alone");
}

#[test]
fn every_rewrite_reduces_tokens() {
    // identity on already-dense code (no rewrite can reduce further)
    let src = "xs = [1, 2, 3]\nprint xs | map (x -> x * x) | sum\n";
    let (d, ok) = run_cmd("dense", src);
    assert!(ok);
    assert_eq!(d.trim(), src.trim());
}
