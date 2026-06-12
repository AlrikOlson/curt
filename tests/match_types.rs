//! match-recordarm gate: record-type match arms narrow STRUCTURALLY at
//! runtime (the checker is structural; match agrees), and undeclared arm
//! types are rejected — no silent wrong-arm path survives.

use std::io::Write;
use std::process::{Command, Stdio};

/// run `curt <mode> -` with the program on stdin (no temp files needed)
fn curt(mode: &str, src: &str) -> (String, String, bool) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_curt"))
        .arg(mode)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn curt");
    child.stdin.take().unwrap().write_all(src.as_bytes()).unwrap();
    let out = child.wait_with_output().expect("wait curt");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

#[test]
fn record_arm_matches_declared_literal() {
    // the original silent-wrong-arm repro (think:115) now selects Pt
    let (out, _, ok) = curt(
        "run",
        "type Pt = {x int}\np = Pt{x: 1}\nprint (match p { Pt q -> \"pt\", _ -> \"other\" })\n",
    );
    assert!(ok);
    assert_eq!(out, "pt\n");
}

#[test]
fn record_arm_matches_structurally() {
    // an untagged literal with the right shape matches — runtime mirrors
    // the structural checker (the tournament decision)
    let (out, _, ok) = curt(
        "run",
        "type Pt = {x int}\nq = {x: 2}\nprint (match q { Pt r -> \"pt {r.x}\", _ -> \"other\" })\n",
    );
    assert!(ok);
    assert_eq!(out, "pt 2\n");
}

#[test]
fn shape_mismatch_falls_through() {
    let (out, _, ok) = curt(
        "run",
        "type Pt = {x int}\nw = {y: \"s\"}\nprint (match w { Pt r -> \"pt\", _ -> \"other\" })\n",
    );
    assert!(ok);
    assert_eq!(out, "other\n");
}

#[test]
fn union_of_records_selects_by_shape() {
    let src = "type A = {a int}\ntype B = {b str}\nshow v = match v { A x -> \"a {x.a}\", B y -> \"b {y.b}\", _ -> \"?\" }\nprint (show A{a: 1})\nprint (show B{b: \"z\"})\n";
    let (out, _, ok) = curt("run", src);
    assert!(ok);
    assert_eq!(out, "a 1\nb z\n");
}

#[test]
fn undeclared_arm_type_rejected_by_check_and_run() {
    let src = "v = 1\nprint (match v { Foo q -> \"f\", _ -> \"o\" })\n";
    let (_, cerr, cok) = curt("check", src);
    assert!(!cok, "check must reject an undeclared arm type");
    assert!(cerr.contains("unknown_name") && cerr.contains("Foo"), "{cerr}");
    let (_, rerr, rok) = curt("run", src);
    assert!(!rok, "run must fail loudly, never silently skip the arm");
    assert!(rerr.contains("not declared"), "{rerr}");
}
