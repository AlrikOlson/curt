//! curt reference implementation library (front-end as of interp-a).

pub mod ast;
pub mod dense;
pub mod diag;
pub mod expand;
pub mod fmt;
pub mod eval;
pub mod infer;
pub mod lexer;
pub mod parser;

use diag::Diag;

/// Lex + parse a source string into the program AST.
pub fn parse_source(src: &str) -> Result<Vec<ast::Stmt>, Diag> {
    parser::parse(lexer::lex(src)?)
}

/// A parsed program plus per-statement (line, col) starts.
pub type SpannedProgram = (Vec<ast::Stmt>, Vec<(u32, u32)>);

/// Lex + parse, keeping each toplevel statement's (line, col) start for
/// statement-granularity diagnostics (`infer::check_at`).
pub fn parse_source_spanned(src: &str) -> Result<SpannedProgram, Diag> {
    let pairs = parser::parse_spanned(lexer::lex(src)?)?;
    Ok(pairs.into_iter().unzip())
}
