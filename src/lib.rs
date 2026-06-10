//! cmm reference implementation library (front-end as of interp-a).

pub mod ast;
pub mod diag;
pub mod expand;
pub mod fmt;
pub mod infer;
pub mod lexer;
pub mod parser;

use diag::Diag;

/// Lex + parse a source string into the program AST.
pub fn parse_source(src: &str) -> Result<Vec<ast::Stmt>, Diag> {
    parser::parse(lexer::lex(src)?)
}
