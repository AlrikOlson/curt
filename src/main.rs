//! curt CLI. Subcommands in v0.1-a: parse, tokens. fmt/expand/run land in
//! interp-b/c/d and exit 2 until then. No clap: startup <10ms is a gate.

use curt::{lexer, parser};
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
                eprintln!("usage: curt parse <file|->");
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
                eprintln!("usage: curt tokens <file|->");
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
                eprintln!("usage: curt fmt <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match curt::fmt::format(&src) {
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
        "dense" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: curt dense <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match curt::dense::dense(&src) {
                Ok(out) => {
                    print!("{out}");
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
                eprintln!("usage: curt expand <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match curt::parse_source(&src) {
                Ok(ast) => {
                    // type-reveal when the program checks; untyped view otherwise
                    let sigs = curt::infer::check(&ast)
                        .map(|(sigs, _)| sigs.into_iter().collect())
                        .unwrap_or_default();
                    print!("{}", curt::expand::expand(&ast, &sigs));
                    ExitCode::SUCCESS
                }
                Err(d) => {
                    eprintln!("{d}");
                    ExitCode::FAILURE
                }
            }
        }
        "lint" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: curt lint <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match curt::lint::lint(&src) {
                Ok(findings) => {
                    // advisory: findings go to stdout, exit stays 0 — a lint
                    // must never break an agent loop
                    for d in findings {
                        println!("{d}");
                    }
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
                eprintln!("usage: curt check <file|->");
                return ExitCode::from(2);
            };
            let src = match read_input(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match curt::parse_source_spanned(&src).and_then(|(ast, pos)| curt::infer::check_at(&ast, &pos)) {
                Ok((sigs, warnings)) => {
                    for (name, ty) in sigs {
                        println!("{name} :: {ty}");
                    }
                    for w in warnings {
                        eprintln!("warning: {w}");
                    }
                    ExitCode::SUCCESS
                }
                Err(mut d) => {
                    d.replacement = curt::repair::synthesize(&src, &d);
                    eprintln!("{d}");
                    ExitCode::FAILURE
                }
            }
        }
        "run" => {
            // curt run [--fs] [--net] <file|-> [program args...]
            let mut caps = curt::eval::Caps { fs: false, net: false };
            let mut rest = args[2..].iter();
            let mut path: Option<String> = None;
            let mut prog_args: Vec<String> = vec![String::from("curt")];
            for a in rest.by_ref() {
                match a.as_str() {
                    "--fs" => caps.fs = true,
                    "--net" => caps.net = true,
                    other => {
                        path = Some(other.to_string());
                        break;
                    }
                }
            }
            prog_args.extend(rest.cloned());
            let Some(path) = path else {
                eprintln!("usage: curt run [--fs] [--net] <file|-> [args...]");
                return ExitCode::from(2);
            };
            let src = match read_input(&path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match curt::parse_source(&src) {
                Ok(ast) => match curt::eval::Interp::run(&ast, caps, prog_args) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(m) => {
                        // route through Diag so escaping + the typed shape
                        // hold for every stderr diagnostic (SPEC §7)
                        eprintln!("{}", curt::diag::Diag::at("runtime", 0, 0, &m, "inspect the failure and rerun"));
                        ExitCode::FAILURE
                    }
                },
                Err(mut d) => {
                    d.replacement = curt::repair::synthesize(&src, &d);
                    eprintln!("{d}");
                    ExitCode::FAILURE
                }
            }
        }
        "--version" | "version" => {
            println!("curt {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: curt <parse|check|lint|tokens|fmt|expand|dense|run> <file|->");
            ExitCode::from(2)
        }
    }
}
