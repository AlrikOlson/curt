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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cmm"));
    cmd.arg("run");
    cmd.args(flags);
    cmd.arg(root().join("corpus").join(file));
    cmd.args(args);
    cmd.current_dir(root().join("tests/fixtures"));
    let out = cmd.output().expect("spawn cmm");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.success())
}

fn golden(file: &str, flags: &[&str], args: &[&str], expected: &str) {
    let (out, ok) = run(file, flags, args);
    assert!(ok, "{file}: non-zero exit; stdout: {out}");
    assert_eq!(out, expected, "{file}: stdout mismatch");
}

#[test]
fn golden_01_hello() {
    golden("01_hello.cmm", &[], &[], "hello, world\n");
}

#[test]
fn golden_02_hyp() {
    golden("02_hyp.cmm", &[], &[], "5\n");
}

#[test]
fn golden_03_fib() {
    golden("03_fib.cmm", &[], &[], "832040\n");
}

#[test]
fn golden_04_binsearch() {
    golden("04_binsearch.cmm", &[], &[], "3\n");
}

#[test]
fn golden_05_records() {
    golden("05_records.cmm", &[], &[], "5\n");
}

#[test]
fn golden_06_union_match() {
    golden("06_union_match.cmm", &[], &[], "num 2.5\nsym ok\n");
}

#[test]
fn golden_07_errors_rescue_chain() {
    // no fs capability granted: read yields err, both rescues fire
    golden("07_errors.cmm", &[], &[], "8080\n");
}

#[test]
fn golden_08_pipeline() {
    golden("08_pipeline.cmm", &[], &[], "[a, c]\n");
}

#[test]
fn golden_09_strings() {
    golden("09_strings.cmm", &[], &[], "hello-world\n");
}

#[test]
fn golden_10_group() {
    golden("10_group.cmm", &[], &[], "NY 70\nLA 30\n");
}

#[test]
fn golden_11_filelines() {
    golden("11_filelines.cmm", &["--fs"], &[], "ERR disk full\nERR net down\n");
}

#[test]
fn golden_12_fold() {
    golden("12_fold.cmm", &[], &[], "10\n");
}

#[test]
fn golden_13_tuples() {
    golden("13_tuples.cmm", &[], &[], "1 5\n");
}

#[test]
fn golden_14_while_acc() {
    golden("14_while_acc.cmm", &[], &[], "111\n");
}

#[test]
fn golden_15_spawn() {
    // go is sequential in v0.1 (documented): deterministic order
    golden("15_spawn.cmm", &[], &[], "job 0\njob 1\njob 2\njob 3\n");
}

#[test]
fn golden_16_bitops_fnv1a_u64() {
    // independently computed: FNV-1a("cmm") with wrapping u64
    golden("16_bitops.cmm", &[], &[], "17729526149434277312\n");
}

#[test]
fn golden_17_export_ffi_no_output() {
    golden("17_export_ffi.cmm", &[], &[], "");
}

#[test]
fn golden_18_wordfreq() {
    golden("18_wordfreq.cmm", &["--fs"], &["words.txt"], "the 3\ncat 2\ndog 1\nfish 1\n");
}

#[test]
fn golden_19_parser() {
    golden("19_parser.cmm", &[], &["(1 + 2) * 3"], "9\n");
    golden("19_parser.cmm", &[], &["2 + 3 * 4"], "14\n");
}

#[test]
fn golden_20_server_smoke() {
    // start the server, connect, send a line, expect it uppercased
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmm"))
        .arg("run")
        .arg("--net")
        .arg(root().join("corpus").join("20_server.cmm"))
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
