//! `curt fmt` — token-level canonical formatter (SPEC §1).
//!
//! Scope (v1, interp-b): a spelling + layout normalizer, not a re-spacer.
//! - Postel spellings canonicalize (the lexer already mapped them): `&&`→and,
//!   `||`→or, `!`→not, `True/False`→true/false, `None`→(), `return`→ret,
//!   `elif`→else if; trailing commas are dropped.
//! - Layout canonicalizes: 2-space indents by brace depth, blank lines capped
//!   at one, comments preserved (own line or trailing), exactly one trailing
//!   newline.
//! - SEMANTIC ADJACENCY IS PRESERVED (the interp-a discovery): `x.f` vs
//!   `x .f`, `x?` vs `x ? y`, `Pt{`/`f(`/`ts[` gluedness all change meaning,
//!   so input gluedness is the default separator — overridden only where
//!   spacing is pure convention (commas, record colons, keywords).
//! - Known limitation (needs parse context, lands with interp-c): an
//!   expression-position `=` is accepted (Postel) but not rewritten to `==`.

use crate::diag::Diag;
use crate::lexer::{lex_raw, Tok, Token};

pub fn format(src: &str) -> Result<String, Diag> {
    let toks = lex_raw(src)?;
    let mut out = String::new();
    let mut depth: usize = 0; // brace depth → indentation
    let mut at_line_start = true;
    let mut prev: Option<&Token> = None;

    let meaningful_next = |from: usize| -> Option<&Tok> {
        toks[from + 1..].iter().map(|t| &t.tok).find(|t| !matches!(t, Tok::Comment(_)))
    };

    for (idx, t) in toks.iter().enumerate() {
        match &t.tok {
            Tok::Eof => break,
            Tok::Newline => {
                out.push('\n');
                if t.blanks >= 1 {
                    out.push('\n'); // cap at one blank line
                }
                at_line_start = true;
                prev = None;
            }
            Tok::Comment(text) => {
                if at_line_start {
                    out.push_str(&"  ".repeat(depth));
                    at_line_start = false;
                } else {
                    out.push(' ');
                }
                out.push_str(text);
                prev = Some(t);
            }
            tok => {
                // trailing-comma drop (Postel)
                if matches!(tok, Tok::Comma)
                    && matches!(meaningful_next(idx), Some(Tok::RBrace | Tok::RBrack | Tok::RParen))
                {
                    continue;
                }
                if matches!(tok, Tok::RBrace) {
                    depth = depth.saturating_sub(1);
                }
                if at_line_start {
                    out.push_str(&"  ".repeat(depth));
                    at_line_start = false;
                } else if let Some(p) = prev {
                    if separator(p, t) {
                        out.push(' ');
                    }
                }
                let text = canonical_text(tok);
                // never weld two word-like tokens (e.g. `!done` → `not done`)
                if !out.ends_with([' ', '\n']) && !out.is_empty() {
                    let last = out.chars().last().unwrap();
                    let first = text.chars().next().unwrap_or(' ');
                    if (last.is_alphanumeric() || last == '_')
                        && (first.is_alphanumeric() || first == '_')
                    {
                        out.push(' ');
                    }
                }
                out.push_str(&text);
                if matches!(tok, Tok::LBrace) {
                    depth += 1;
                }
                prev = Some(t);
            }
        }
    }
    while out.ends_with('\n') {
        out.pop();
    }
    out.push('\n');
    Ok(out)
}

/// Should a space separate `prev` from `cur`? Default: preserve input
/// adjacency — gluedness is semantic in curt (SPEC §1). Overrides only where
/// spacing can never change meaning (bracket edges, separators, colon-left).
fn separator(prev: &Token, cur: &Token) -> bool {
    use Tok::*;
    match (&prev.tok, &cur.tok) {
        // never a space before these
        (_, Comma) | (_, Semi) | (_, RParen) | (_, RBrack) | (_, Colon) => false,
        // never a space after these
        (LParen, _) | (LBrack, _) | (Dot, _) => false,
        // always a space after a semicolon (inline statement separator)
        (Semi, _) => true,
        // everything else: the input's adjacency is the truth
        _ => !cur.glued,
    }
}

fn canonical_text(tok: &Tok) -> String {
    use Tok::*;
    match tok {
        Name(s) | TName(s) | Num(s) | Str(s) => s.clone(),
        Comment(s) => s.clone(),
        If => "if".into(),
        Elif => "else if".into(),
        Else => "else".into(),
        While => "while".into(),
        For => "for".into(),
        In => "in".into(),
        Match => "match".into(),
        Type => "type".into(),
        Ret => "ret".into(),
        Go => "go".into(),
        Pub => "pub".into(),
        And => "and".into(),
        Or => "or".into(),
        Not => "not".into(),
        True => "true".into(),
        False => "false".into(),
        NoneLit => "()".into(),
        EqEq => "==".into(),
        Ne => "!=".into(),
        Le => "<=".into(),
        Ge => ">=".into(),
        Lt => "<".into(),
        Gt => ">".into(),
        Assign => "=".into(),
        PlusEq => "+=".into(),
        MinusEq => "-=".into(),
        StarEq => "*=".into(),
        SlashEq => "/=".into(),
        Arrow => "->".into(),
        DColon => "::".into(),
        Pipe => "|".into(),
        Question => "?".into(),
        Plus => "+".into(),
        Minus => "-".into(),
        Star => "*".into(),
        StarStar => "**".into(),
        Slash => "/".into(),
        Percent => "%".into(),
        Caret => "^".into(),
        Dot => ".".into(),
        Comma => ",".into(),
        Colon => ":".into(),
        Semi => ";".into(),
        LParen => "(".into(),
        RParen => ")".into(),
        LBrack => "[".into(),
        RBrack => "]".into(),
        LBrace => "{".into(),
        RBrace => "}".into(),
        Newline => "\n".into(),
        Eof => String::new(),
    }
}
