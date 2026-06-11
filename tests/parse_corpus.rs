//! The Rust mirror of the PEG gate: every canonical corpus file must parse.

use std::path::PathBuf;

fn parse_file(name: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus").join(name);
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
    if let Err(d) = curt::parse_source(&src) {
        panic!("{name} failed to parse: {d}");
    }
}

macro_rules! corpus_test {
    ($fn_name:ident, $file:expr) => {
        #[test]
        fn $fn_name() {
            parse_file($file);
        }
    };
}

corpus_test!(c01_hello, "01_hello.curt");
corpus_test!(c02_hyp, "02_hyp.curt");
corpus_test!(c03_fib, "03_fib.curt");
corpus_test!(c04_binsearch, "04_binsearch.curt");
corpus_test!(c05_records, "05_records.curt");
corpus_test!(c06_union_match, "06_union_match.curt");
corpus_test!(c07_errors, "07_errors.curt");
corpus_test!(c08_pipeline, "08_pipeline.curt");
corpus_test!(c09_strings, "09_strings.curt");
corpus_test!(c10_group, "10_group.curt");
corpus_test!(c11_filelines, "11_filelines.curt");
corpus_test!(c12_fold, "12_fold.curt");
corpus_test!(c13_tuples, "13_tuples.curt");
corpus_test!(c14_while_acc, "14_while_acc.curt");
corpus_test!(c15_spawn, "15_spawn.curt");
corpus_test!(c16_bitops, "16_bitops.curt");
corpus_test!(c17_export_ffi, "17_export_ffi.curt");
corpus_test!(c18_wordfreq, "18_wordfreq.curt");
corpus_test!(c19_parser, "19_parser.curt");
corpus_test!(c20_server, "20_server.curt");
