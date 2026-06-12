//! interp-d gate: every corpus program executes with golden stdout.
//! fs programs run with cwd = tests/fixtures; 20_server gets a live TCP
//! smoke test.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run(file: &str, flags: &[&str], args: &[&str]) -> (String, bool) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_curt"));
    cmd.arg("run");
    cmd.args(flags);
    cmd.arg(root().join("corpus").join(file));
    cmd.args(args);
    cmd.current_dir(root().join("tests/fixtures"));
    let out = cmd.output().expect("spawn curt");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.success())
}

fn golden(file: &str, flags: &[&str], args: &[&str], expected: &str) {
    let (out, ok) = run(file, flags, args);
    assert!(ok, "{file}: non-zero exit; stdout: {out}");
    assert_eq!(out, expected, "{file}: stdout mismatch");
}

#[test]
fn golden_01_hello() {
    golden("01_hello.curt", &[], &[], "hello, world\n");
}

#[test]
fn golden_02_hyp() {
    golden("02_hyp.curt", &[], &[], "5\n");
}

#[test]
fn golden_03_fib() {
    golden("03_fib.curt", &[], &[], "832040\n");
}

#[test]
fn golden_04_binsearch() {
    golden("04_binsearch.curt", &[], &[], "3\n");
}

#[test]
fn golden_05_records() {
    golden("05_records.curt", &[], &[], "5\n");
}

#[test]
fn golden_06_union_match() {
    golden("06_union_match.curt", &[], &[], "num 2.5\nsym ok\n");
}

#[test]
fn golden_07_errors_rescue_chain() {
    // no fs capability granted: read yields err, both rescues fire
    golden("07_errors.curt", &[], &[], "8080\n");
}

#[test]
fn golden_08_pipeline() {
    golden("08_pipeline.curt", &[], &[], "[a, c]\n");
}

#[test]
fn golden_09_strings() {
    golden("09_strings.curt", &[], &[], "hello-world\n");
}

#[test]
fn golden_10_group() {
    golden("10_group.curt", &[], &[], "NY 70\nLA 30\n");
}

#[test]
fn golden_11_filelines() {
    golden("11_filelines.curt", &["--fs"], &[], "ERR disk full\nERR net down\n");
}

#[test]
fn golden_12_fold() {
    golden("12_fold.curt", &[], &[], "10\n");
}

#[test]
fn golden_13_tuples() {
    golden("13_tuples.curt", &[], &[], "1 5\n");
}

#[test]
fn golden_14_while_acc() {
    golden("14_while_acc.curt", &[], &[], "111\n");
}

#[test]
fn golden_15_spawn() {
    // go is sequential in v0.1 (documented): deterministic order
    golden("15_spawn.curt", &[], &[], "job 0\njob 1\njob 2\njob 3\n");
}

#[test]
fn golden_16_bitops_fnv1a_u64() {
    // independently computed: FNV-1a("curt") with wrapping u64
    golden("16_bitops.curt", &[], &[], "5518724359090551763\n");
}

#[test]
fn golden_17_export_ffi_no_output() {
    golden("17_export_ffi.curt", &[], &[], "");
}

#[test]
fn golden_18_wordfreq() {
    golden("18_wordfreq.curt", &["--fs"], &["words.txt"], "the 3\ncat 2\ndog 1\nfish 1\n");
}

#[test]
fn golden_19_parser() {
    golden("19_parser.curt", &[], &["(1 + 2) * 3"], "9\n");
    golden("19_parser.curt", &[], &["2 + 3 * 4"], "14\n");
}

#[test]
fn golden_21_append() {
    golden("21_append.curt", &[], &[], "3 1 4 1 5 9\ni7\nsok\n29\n");
}

#[test]
fn golden_22_logmill() {
    let full = "== svc health ==\nfiles 2 missing 1\nlines 16 reqs 11 errs 3 bad 2\napi: reqs 6 avg 50 med 47.5 score 4\ndb: reqs 4 avg 100 med 95.5 score 7\nauth: reqs 1 avg 40 med 40 score 0\nerrors by svc: db 2, api 1\npeak minute: 12:00 x7\nslow >=120ms: api 130, db 200\ntop 2 by traffic: api 6, db 4\nworst: db score 7\napi ######\ndb ####\nauth #\nscanned 2 files, 16 lines, 2 bad\n";
    let dflt = "== logmill ==\nfiles 1 missing 0\nlines 8 reqs 6 errs 1 bad 1\napi: reqs 4 avg 60 med 60 score 1\ndb: reqs 2 avg 87.75 med 95.5 score 3\nerrors by svc: db 1\npeak minute: 12:00 x5\nslow >=150ms: none\ntop 1 by traffic: api 4\nworst: db score 3\napi ####\ndb ##\nscanned 1 files, 8 lines, 1 bad\n";
    // the four spec-resolution paths, each golden:
    // explicit job file / args rescue / malformed JSON err arm / missing file rescue
    golden("22_logmill.curt", &["--fs"], &["logmill.json"], full);
    golden("22_logmill.curt", &["--fs"], &[], full);
    let noted = format!("note: bad job spec, using defaults\n{dflt}");
    golden("22_logmill.curt", &["--fs"], &["badjob.json"], &noted);
    golden("22_logmill.curt", &["--fs"], &["ghost.json"], dflt);
}

#[test]
fn golden_20_server_smoke() {
    // start the server, connect, send a line, expect it uppercased
    let mut child = Command::new(env!("CARGO_BIN_EXE_curt"))
        .arg("run")
        .arg("--net")
        .arg(root().join("corpus").join("20_server.curt"))
        .spawn()
        .expect("spawn server");
    // wait for the listener
    let mut stream = None;
    for _ in 0..50 {
        match std::net::TcpStream::connect("127.0.0.1:8080") {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
        }
    }
    let mut stream = stream.expect("server did not start");
    stream.write_all(b"hi server\n").expect("write");
    stream.shutdown(std::net::Shutdown::Write).expect("shutdown write");
    let mut buf = String::new();
    stream.read_to_string(&mut buf).expect("read");
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(buf, "HI SERVER\n", "server must uppercase the line");
}
