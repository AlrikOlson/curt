//! Diagnostics-as-prompts (SPEC §7): single-line JSON designed to be fed
//! back to a model verbatim for one-edit self-repair.

use std::fmt;

#[derive(Debug, Clone)]
pub struct Diag {
    pub err: String,
    pub line: u32,
    pub col: u32,
    pub msg: String,
    pub fix: String,
}

impl Diag {
    pub fn at(err: &str, line: u32, col: u32, msg: &str, fix: &str) -> Self {
        Diag { err: err.into(), line, col, msg: msg.into(), fix: fix.into() }
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"err\":\"{}\",\"at\":\"{}:{}\",\"msg\":\"{}\",\"fix\":\"{}\"}}",
            esc(&self.err),
            self.line,
            self.col,
            esc(&self.msg),
            esc(&self.fix)
        )
    }
}
