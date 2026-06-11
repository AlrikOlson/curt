//! Lexer for curt v0.1 (SPEC.md §1).
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
    /// trivia: only emitted by `lex_raw` (for fmt); `lex` filters it out
    Comment(String),
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
    /// On `Newline` tokens: number of blank lines following (fmt trivia).
    pub blanks: u8,
}

/// Parser view: trivia-free. Comments dropped; newline runs collapsed.
pub fn lex(src: &str) -> Result<Vec<Token>, Diag> {
    let raw = lex_raw(src)?;
    let mut out: Vec<Token> = Vec::new();
    for t in raw {
        match t.tok {
            Tok::Comment(_) => {}
            Tok::Newline => {
                if !matches!(out.last().map(|p| &p.tok), Some(Tok::Newline) | None) {
                    out.push(t);
                }
            }
            _ => out.push(t),
        }
    }
    // a trailing comment line can leave a dangling Newline before Eof
    while out.len() >= 2
        && matches!(out[out.len() - 1].tok, Tok::Eof)
        && matches!(out[out.len() - 2].tok, Tok::Newline)
    {
        let eof = out.pop().unwrap();
        out.pop();
        out.push(eof);
    }
    Ok(out)
}

/// Full token stream including comments and blank-line counts (for fmt).
pub fn lex_raw(src: &str) -> Result<Vec<Token>, Diag> {
    let b = src.as_bytes();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line: u32 = 1;
    let mut col: u32 = 1;
    let mut prev_end = usize::MAX; // byte index just past the previous token
    let mut delims: Vec<u8> = Vec::new(); // open-delimiter stack; newline suspended only directly inside ( or [

    macro_rules! push {
        ($tok:expr, $start:expr, $startcol:expr) => {{
            out.push(Token { tok: $tok, line, col: $startcol, glued: $start == prev_end, blanks: 0 });
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
                let startcol = col;
                let start = i;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                    col += 1;
                }
                let text = src[start..i].trim_end().to_string();
                out.push(Token { tok: Tok::Comment(text), line, col: startcol, glued: false, blanks: 0 });
            }
            b'\n' => {
                // inside [ ] / ( ) a newline is plain whitespace, not a
                // statement separator (multiline literals, spec-truth) —
                // unless a { } block is open inside (blocks NEED newlines;
                // domain-bench: block lambdas inside call parens flattened)
                if matches!(delims.last(), Some(b'(') | Some(b'[')) {
                    i += 1;
                    line += 1;
                    col = 1;
                    prev_end = usize::MAX;
                    continue;
                }
                // one Newline token per run; extra newlines counted as blanks
                match out.last_mut() {
                    Some(t) if matches!(t.tok, Tok::Newline) => {
                        t.blanks = t.blanks.saturating_add(1);
                    }
                    None => {} // leading blank lines dropped
                    _ => out.push(Token { tok: Tok::Newline, line, col, glued: false, blanks: 0 }),
                }
                i += 1;
                line += 1;
                col = 1;
                prev_end = usize::MAX;
            }
            b'\'' => {
                // Postel: 'x' (single character, optionally escaped) is the
                // C/Rust char-literal habit — canonicalize to a 1-char string
                // (token-bench: 2 cells failed on '\''-vowel comparisons)
                let (start, startcol) = (i, col);
                let rest = &b[i + 1..];
                let (ch, len) = match rest {
                    [b'\\', e, b'\'', ..] => (format!("\\{}", *e as char), 4),
                    [c2, b'\'', ..] if *c2 != b'\'' && *c2 != b'\n' => ((*c2 as char).to_string(), 3),
                    _ if rest.iter().take_while(|c| **c != b'\n').any(|c| *c == b'\'') => {
                        // Postel: '...' multi-char single-quoted string —
                        // canonicalize to a double-quoted string, escaping
                        // any embedded double quotes (domain-bench:
                        // interpolation holes can't carry double quotes)
                        let end = rest.iter().position(|c| *c == b'\'').unwrap();
                        // raw semantics: escape quotes AND interpolation
                        // braces so '...' text never opens a hole
                        let body = String::from_utf8_lossy(&rest[..end])
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('{', "\\{");
                        (body, end + 2)
                    }
                    _ => {
                        return Err(Diag::at(
                            "unexpected_char",
                            line,
                            col,
                            "character '\\'' is not part of curt",
                            "use double quotes for strings",
                        ))
                    }
                };
                push!(Tok::Str(format!("\"{ch}\"")), start, startcol);
                i += len;
                col += len as u32;
                prev_end = i;
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
                    "++" => (Tok::Plus, 2), // Postel: string-concat habit (token-bench)
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
                                &format!("character {:?} is not part of curt", c as char),
                                "remove it or check the SPEC lexical rules",
                            ))
                        }
                    },
                };
                // newline-in-brackets: [ and ( suspend statement separation
                // (multiline list/call literals — token-bench failure 06/08);
                // { } untouched: blocks NEED newlines as separators
                match tok {
                    Tok::LBrack => delims.push(b'['),
                    Tok::LParen => delims.push(b'('),
                    Tok::LBrace => delims.push(b'{'),
                    Tok::RBrack | Tok::RParen | Tok::RBrace => {
                        delims.pop();
                    }
                    _ => {}
                }
                push!(tok, start, startcol);
                i += len;
                col += len as u32;
                prev_end = i;
            }
        }
    }
    // trim trailing newline token; Eof terminates
    while matches!(out.last().map(|t| &t.tok), Some(Tok::Newline)) {
        out.pop();
    }
    out.push(Token { tok: Tok::Eof, line, col, glued: false, blanks: 0 });
    Ok(out)
}
