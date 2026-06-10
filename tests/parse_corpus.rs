//! The Rust mirror of the PEG gate: every canonical corpus file must parse.

use std::path::PathBuf;

fn parse_file(name: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus").join(name);
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
    if let Err(d) = cmm::parse_source(&src) {
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

corpus_test!(c01_hello, "01_hello.cmm");
corpus_test!(c02_hyp, "02_hyp.cmm");
corpus_test!(c03_fib, "03_fib.cmm");
corpus_test!(c04_binsearch, "04_binsearch.cmm");
corpus_test!(c05_records, "05_records.cmm");
corpus_test!(c06_union_match, "06_union_match.cmm");
corpus_test!(c07_errors, "07_errors.cmm");
corpus_test!(c08_pipeline, "08_pipeline.cmm");
corpus_test!(c09_strings, "09_strings.cmm");
corpus_test!(c10_group, "10_group.cmm");
corpus_test!(c11_filelines, "11_filelines.cmm");
corpus_test!(c12_fold, "12_fold.cmm");
corpus_test!(c13_tuples, "13_tuples.cmm");
corpus_test!(c14_while_acc, "14_while_acc.cmm");
corpus_test!(c15_spawn, "15_spawn.cmm");
corpus_test!(c16_bitops, "16_bitops.cmm");
corpus_test!(c17_export_ffi, "17_export_ffi.cmm");
corpus_test!(c18_wordfreq, "18_wordfreq.cmm");
corpus_test!(c19_parser, "19_parser.cmm");
corpus_test!(c20_server, "20_server.cmm");
