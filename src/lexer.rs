//! Lexer for cmm v0.1 (SPEC.md §1).
//!
//! Spans drive two SPEC rules the parser needs:
//! - `glued`: token starts exactly where the previous one ended (no
//!   whitespace). Postfix propagate-`?` and `Pt{...}` record literals and
//!   `f(x, y)` call sugar all require gluedness.
//! - Postel mappings happen here when they are pure token substitutions
//!   (`&&`→`and`, `||`→`or`, `!`→`not`, `True/False`, `None`, `return`,
//!   `elif`).

use crate::diag::Diag;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Name(String),
    TName(String),
    Num(String),
    Str(String),
    // keywords
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Match,
    Type,
    Ret,
    Go,
    Pub,
    And,
    Or,
    Not,
    True,
    False,
    NoneLit,
    // operators / punctuation
    EqEq,
    Ne,
    Le,
    Ge,
    Lt,
    Gt,
    Assign,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Arrow,
    DColon,
    Pipe,
    Question,
    Plus,
    Minus,
    Star,
    StarStar,
    Slash,
    Percent,
    Caret,
    Dot,
    Comma,
    Colon,
    Semi,
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub col: u32,
    /// No whitespace between this token and the previous one.
    pub glued: bool,
}

pub fn lex(src: &str) -> Result<Vec<Token>, Diag> {
    let b = src.as_bytes();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line: u32 = 1;
    let mut col: u32 = 1;
    let mut prev_end = usize::MAX; // byte index just past the previous token

    macro_rules! push {
        ($tok:expr, $start:expr, $startcol:expr) => {{
            out.push(Token { tok: $tok, line, col: $startcol, glued: $start == prev_end });
        }};
    }

    while i < b.len() {
        let c = b[i];
        match c {
            b' ' | b'\t' | b'\r' => {
                i += 1;
                col += 1;
            }
            b'#' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'\n' => {
                // collapse a run of newlines (with comments/indentation) into one
                if !matches!(out.last().map(|t| &t.tok), Some(Tok::Newline) | None) {
                    out.push(Token { tok: Tok::Newline, line, col, glued: false });
                }
                i += 1;
                line += 1;
                col = 1;
                prev_end = usize::MAX;
            }
            b'"' => {
                let (start, startcol) = (i, col);
                i += 1;
                col += 1;
                let mut s = String::from("\"");
                loop {
                    if i >= b.len() {
                        return Err(Diag::at("unterminated_string", line, startcol, "string never closes", "add a closing \""));
                    }
                    let ch = b[i];
                    if ch == b'\\' && i + 1 < b.len() {
                        s.push(b[i] as char);
                        s.push(b[i + 1] as char);
                        i += 2;
                        col += 2;
                        continue;
                    }
                    if ch == b'\n' {
                        return Err(Diag::at("unterminated_string", line, startcol, "newline inside string", "close the string before end of line"));
                    }
                    s.push(ch as char);
                    i += 1;
                    col += 1;
                    if ch == b'"' {
                        break;
                    }
                }
                push!(Tok::Str(s), start, startcol);
                prev_end = i;
            }
            b'0'..=b'9' => {
                let (start, startcol) = (i, col);
                let mut s = String::new();
                while i < b.len() && b[i].is_ascii_digit() {
                    s.push(b[i] as char);
                    i += 1;
                    col += 1;
                }
                if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
                    s.push('.');
                    i += 1;
                    col += 1;
                    while i < b.len() && b[i].is_ascii_digit() {
                        s.push(b[i] as char);
                        i += 1;
                        col += 1;
                    }
                }
                // optional width suffix i8..u64
                if i < b.len() && (b[i] == b'i' || b[i] == b'u') {
                    let save = (i, col);
                    let mut suf = String::new();
                    suf.push(b[i] as char);
                    i += 1;
                    col += 1;
                    while i < b.len() && b[i].is_ascii_digit() {
                        suf.push(b[i] as char);
                        i += 1;
                        col += 1;
                    }
                    if matches!(suf.as_str(), "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64") {
                        s.push_str(&suf);
                    } else {
                        (i, col) = save;
                    }
                }
                push!(Tok::Num(s), start, startcol);
                prev_end = i;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let (start, startcol) = (i, col);
                let mut s = String::new();
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    s.push(b[i] as char);
                    i += 1;
                    col += 1;
                }
                let tok = match s.as_str() {
                    "if" => Tok::If,
                    "elif" => Tok::Elif,
                    "else" => Tok::Else,
                    "while" => Tok::While,
                    "for" => Tok::For,
                    "in" => Tok::In,
                    "match" => Tok::Match,
                    "type" => Tok::Type,
                    "ret" => Tok::Ret,
                    "return" => Tok::Ret, // Postel
                    "go" => Tok::Go,
                    "pub" => Tok::Pub,
                    "and" => Tok::And,
                    "or" => Tok::Or,
                    "not" => Tok::Not,
                    "true" => Tok::True,
                    "false" => Tok::False,
                    "True" => Tok::True,   // Postel
                    "False" => Tok::False, // Postel
                    "None" => Tok::NoneLit, // Postel
                    _ => {
                        if s.as_bytes()[0].is_ascii_uppercase() {
                            Tok::TName(s)
                        } else {
                            Tok::Name(s)
                        }
                    }
                };
                push!(tok, start, startcol);
                prev_end = i;
            }
            _ => {
                let (start, startcol) = (i, col);
                let two = if i + 1 < b.len() { &src[i..i + 2] } else { "" };
                let (tok, len) = match two {
                    "==" => (Tok::EqEq, 2),
                    "!=" => (Tok::Ne, 2),
                    "<=" => (Tok::Le, 2),
                    ">=" => (Tok::Ge, 2),
                    "+=" => (Tok::PlusEq, 2),
                    "-=" => (Tok::MinusEq, 2),
                    "*=" => (Tok::StarEq, 2),
                    "/=" => (Tok::SlashEq, 2),
                    "->" => (Tok::Arrow, 2),
                    "::" => (Tok::DColon, 2),
                    "**" => (Tok::StarStar, 2),
                    "&&" => (Tok::And, 2), // Postel
                    "||" => (Tok::Or, 2),  // Postel
                    _ => match c {
                        b'=' => (Tok::Assign, 1),
                        b'<' => (Tok::Lt, 1),
                        b'>' => (Tok::Gt, 1),
                        b'|' => (Tok::Pipe, 1),
                        b'?' => (Tok::Question, 1),
                        b'+' => (Tok::Plus, 1),
                        b'-' => (Tok::Minus, 1),
                        b'*' => (Tok::Star, 1),
                        b'/' => (Tok::Slash, 1),
                        b'%' => (Tok::Percent, 1),
                        b'^' => (Tok::Caret, 1),
                        b'.' => (Tok::Dot, 1),
                        b',' => (Tok::Comma, 1),
                        b':' => (Tok::Colon, 1),
                        b';' => (Tok::Semi, 1),
                        b'(' => (Tok::LParen, 1),
                        b')' => (Tok::RParen, 1),
                        b'[' => (Tok::LBrack, 1),
                        b']' => (Tok::RBrack, 1),
                        b'{' => (Tok::LBrace, 1),
                        b'}' => (Tok::RBrace, 1),
                        b'!' => (Tok::Not, 1), // Postel (bare !)
                        _ => {
                            return Err(Diag::at(
                                "unexpected_char",
                                line,
                                col,
                                &format!("character {:?} is not part of cmm", c as char),
                                "remove it or check the SPEC lexical rules",
                            ))
                        }
                    },
                };
                push!(tok, start, startcol);
                i += len;
                col += len as u32;
                prev_end = i;
            }
        }
    }
    // trim trailing newline token; parser treats Eof as terminator
    while matches!(out.last().map(|t| &t.tok), Some(Tok::Newline)) {
        out.pop();
    }
    out.push(Token { tok: Tok::Eof, line, col, glued: false });
    Ok(out)
}
