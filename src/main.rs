//! cmm CLI. Subcommands in v0.1-a: parse, tokens. fmt/expand/run land in
//! interp-b/c/d and exit 2 until then. No clap: startup <10ms is a gate.

use cmm::{lexer, parser};
use std::process::ExitCode;

fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s).map_err(|e| e.to_string())?;
        Ok(s)
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    match cmd {
        "parse" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: cmm parse <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match lexer::lex(&src).and_then(parser::parse) {
                Ok(ast) => {
                    println!("{ast:#?}");
                    ExitCode::SUCCESS
                }
                Err(d) => {
                    eprintln!("{d}");
                    ExitCode::FAILURE
                }
            }
        }
        "tokens" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: cmm tokens <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            // o200k ranks load lazily here only; the parse path stays fast.
            match tiktoken_rs::o200k_base() {
                Ok(bpe) => {
                    println!("{}", bpe.encode_ordinary(&src).len());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tokenizer: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "fmt" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: cmm fmt <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match cmm::fmt::format(&src) {
                Ok(s) => {
                    print!("{s}");
                    ExitCode::SUCCESS
                }
                Err(d) => {
                    eprintln!("{d}");
                    ExitCode::FAILURE
                }
            }
        }
        "expand" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: cmm expand <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match cmm::parse_source(&src) {
                Ok(ast) => {
                    // type-reveal when the program checks; untyped view otherwise
                    let sigs = cmm::infer::check(&ast)
                        .map(|(sigs, _)| sigs.into_iter().collect())
                        .unwrap_or_default();
                    print!("{}", cmm::expand::expand(&ast, &sigs));
                    ExitCode::SUCCESS
                }
                Err(d) => {
                    eprintln!("{d}");
                    ExitCode::FAILURE
                }
            }
        }
        "check" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: cmm check <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match cmm::parse_source(&src).and_then(|ast| cmm::infer::check(&ast)) {
                Ok((sigs, warnings)) => {
                    for (name, ty) in sigs {
                        println!("{name} :: {ty}");
                    }
                    for w in warnings {
                        eprintln!("warning: {w}");
                    }
                    ExitCode::SUCCESS
                }
                Err(d) => {
                    eprintln!("{d}");
                    ExitCode::FAILURE
                }
            }
        }
        "run" => {
            eprintln!("cmm run: not implemented yet (lands in interp-d; see ROADMAP.md)");
            ExitCode::from(2)
        }
        "--version" | "version" => {
            println!("cmm {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: cmm <parse|check|tokens|fmt|expand|run> <file|->");
            ExitCode::from(2)
        }
    }
}
