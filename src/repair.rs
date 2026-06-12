//! Machine-applicable repair synthesis (SPEC §7).
//!
//! For the mechanically-fixable error classes, generate a bounded set of
//! textual candidate edits at the diagnostic's line and verify each by
//! re-running the full parse+check pipeline. A `repair.replacement` payload
//! is emitted only for a candidate that makes the whole program check clean
//! — rustc's "machine-applicable" bar, APR's generate-and-validate shape.
//!
//! Verified-plausible is still not verified-correct (patch overfitting):
//! a payload that checks clean can still compute the wrong thing. The
//! wrong-payload rate is measured, never assumed (tools/bench/diag_tourney.py
//! arm E), per the vz-diag tournament that motivated this module: oracle
//! payloads repaired 32/32 single-shot on haiku vs 21/32 for typed diags
//! alone.

use crate::diag::Diag;

/// Candidate edits tried per diagnostic before giving up. Each costs one
/// in-process parse+check of a small program; the CLI startup gate is safe.
const MAX_CANDIDATES: usize = 24;

/// One whole-line replacement: 1-based line number, new text for that line
/// (may contain embedded newlines — the line splits).
pub type Replacement = (u32, String);

/// Try to synthesize a verified replacement for `d` against `src`.
/// Returns at most one single-line edit; `None` when no bounded candidate
/// survives the parse+check gate.
pub fn synthesize(src: &str, d: &Diag) -> Option<Vec<Replacement>> {
    if d.line == 0 {
        return None;
    }
    let lines: Vec<&str> = src.lines().collect();
    if lines.is_empty() {
        return None;
    }
    // an Eof-anchored diag points one line past the file; edit the last line
    let li = (d.line as usize - 1).min(lines.len() - 1);
    let line = lines[li];
    // a candidate is a set of whole-line edits (0-based); most are single
    let mut cands: Vec<Vec<(usize, String)>> = match d.err.as_str() {
        "expected" => expected_candidates(line, &d.msg, d.col),
        "type_mismatch" => conversion_candidates(line, &d.msg),
        "unknown_name" => rename_candidates(src, line, &d.msg),
        _ => return None,
    }
    .into_iter()
    .map(|c| vec![(li, c)])
    .collect();
    if d.err == "type_mismatch" && d.msg.contains(" is not callable") {
        // adjacent `}` / `{` lines: a multi-line literal missing separators
        // (curt's most-measured footgun) parses as record application
        if let Some(e) = comma_separator_edit(&lines, li) {
            cands.push(e);
        }
    }
    let mut seen: Vec<Vec<(usize, String)>> = Vec::new();
    for cand in cands {
        if cand.iter().all(|(i, n)| lines[*i] == n) || seen.contains(&cand) {
            continue;
        }
        seen.push(cand.clone());
        if seen.len() > MAX_CANDIDATES {
            break;
        }
        if checks_clean(&splice(&lines, &cand)) {
            return Some(
                cand.into_iter().map(|(i, n)| (i as u32 + 1, n)).collect(),
            );
        }
    }
    None
}

/// The verification gate: the patched program must parse AND check clean.
fn checks_clean(src: &str) -> bool {
    crate::parse_source_spanned(src)
        .and_then(|(ast, pos)| crate::infer::check_at(&ast, &pos))
        .is_ok()
}

fn splice(lines: &[&str], edits: &[(usize, String)]) -> String {
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        match edits.iter().find(|(e, _)| *e == i) {
            Some((_, n)) => out.push_str(n),
            None => out.push_str(l),
        }
        out.push('\n');
    }
    out
}

/// Edits appending `,` to every line in the literal block after `li` whose
/// trimmed text ends `}` while the next line opens `{`.
fn comma_separator_edit(lines: &[&str], li: usize) -> Option<Vec<(usize, String)>> {
    let mut edits = Vec::new();
    for i in li..lines.len().saturating_sub(1) {
        let cur = lines[i].trim_end();
        let next = lines[i + 1].trim_start();
        if cur.ends_with('}') && next.starts_with('{') {
            edits.push((i, format!("{cur},")));
        }
        if next.starts_with(']') {
            break;
        }
    }
    if edits.is_empty() { None } else { Some(edits) }
}

/// 1-based column -> byte index into `line`, clamped to the line's end.
fn col_idx(line: &str, col: u32) -> usize {
    let want = col.saturating_sub(1) as usize;
    line.char_indices()
        .nth(want)
        .map(|(i, _)| i)
        .unwrap_or(line.len())
}

// ---- class: expected-token parse errors ----

/// Tokens whose literal text is safe to splice in directly.
const INSERTABLE: &[&str] = &[
    "}", "]", ")", "(", "{", "->", "::", ":", "=", ",", "in",
];

/// Debug-name -> source text for found tokens we know how to remove/replace.
fn found_text(found: &str) -> Option<&'static str> {
    Some(match found {
        "Comma" => ",",
        "Colon" => ":",
        "Semi" => ";",
        "Dot" => ".",
        "Assign" => "=",
        "RBrace" => "}",
        "RBrack" => "]",
        "RParen" => ")",
        "LBrace" => "{",
        "LBrack" => "[",
        "LParen" => "(",
        "Arrow" => "->",
        "Pipe" => "|",
        "Question" => "?",
        _ => return None,
    })
}

fn expected_candidates(line: &str, msg: &str, col: u32) -> Vec<String> {
    let rest = match msg.strip_prefix("expected ") {
        Some(r) => r,
        None => return Vec::new(),
    };
    let (want, found) = match rest.find(", found ") {
        Some(i) => (&rest[..i], &rest[i + ", found ".len()..]),
        None => return Vec::new(),
    };
    let idx = col_idx(line, col);
    let mut v = Vec::new();
    if INSERTABLE.contains(&want) {
        // insert the wanted token before the found one (spaced + tight)
        v.push(format!("{}{} {}", &line[..idx], want, &line[idx..]));
        v.push(format!("{} {} {}", &line[..idx], want, &line[idx..]));
        v.push(format!("{}{}{}", &line[..idx], want, &line[idx..]));
        // a missing closer is often a missing line ending
        v.push(format!("{}{}", line, want));
        v.push(format!("{} {}", line, want));
        // replace the found token with the wanted one; or delete it
        if let Some(ft) = found_text(found) {
            if line[idx..].starts_with(ft) {
                v.push(format!("{}{}{}", &line[..idx], want, &line[idx + ft.len()..]));
                v.push(format!("{}{}", &line[..idx], &line[idx + ft.len()..]));
            }
        }
    } else if want == "end of statement" {
        // `print match ...`: the remainder is an expression the statement
        // head can take parenthesized — try that before splitting the line
        v.push(format!(
            "{} ({})",
            line[..idx].trim_end(),
            line[idx..].trim()
        ));
        // split the line where the parser wanted the statement to end
        v.push(format!(
            "{}\n{}",
            line[..idx].trim_end(),
            line[idx..].trim_start()
        ));
    } else if want == "a type" {
        // the annotation itself doesn't parse: drop it, inference takes over
        if let (Some(c), Some(a)) = (line.find(':'), line.find('=')) {
            if c < a {
                v.push(format!(
                    "{} {}",
                    line[..c].trim_end(),
                    line[a..].trim_end()
                ));
            }
        }
    }
    v
}

// ---- class: type-mismatch conversions ----

/// Identifier and literal atoms of a line, skipping string contents.
/// Returns (start, end) byte ranges.
fn atoms(line: &str) -> Vec<(usize, usize)> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i] as char;
        if c == '"' {
            in_str = !in_str;
            i += 1;
            continue;
        }
        if in_str {
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let s = i;
            while i < b.len() && ((b[i] as char).is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push((s, i));
        } else if c.is_ascii_digit() {
            let s = i;
            while i < b.len() && ((b[i] as char).is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            // trim a trailing dot (method position, not a float)
            let e = if b[i - 1] == b'.' { i - 1 } else { i };
            out.push((s, e));
        } else {
            i += 1;
        }
    }
    out
}

fn conversion_candidates(line: &str, msg: &str) -> Vec<String> {
    // strip the equation prefix the checker adds: "in `f`: expected ..."
    let msg = msg.rfind(": ").map(|i| &msg[i + 2..]).unwrap_or(msg);
    let rest = match msg.strip_prefix("expected ") {
        Some(r) => r,
        None => return Vec::new(),
    };
    let (want, got) = match rest.find(", got ") {
        Some(i) => (&rest[..i], &rest[i + ", got ".len()..]),
        None => return Vec::new(),
    };
    let conv = ["int", "float", "str"];
    let mut targets: Vec<&str> = Vec::new();
    // converting the mismatched side to `want`, or the expected side to `got`
    // (`3 + 34.5` fixes by floating the int side), are both legitimate
    for t in [want, got] {
        if conv.contains(&t) && !targets.contains(&t) {
            targets.push(t);
        }
    }
    if targets.is_empty() {
        return Vec::new();
    }
    let mut v = Vec::new();
    for &(s, e) in &atoms(line) {
        let atom = &line[s..e];
        // skip atoms already in receiver position of a conversion
        let after = line[e..].trim_start();
        if after.starts_with('.') {
            continue;
        }
        for t in &targets {
            if atom == *t {
                continue;
            }
            v.push(format!("{}{}.{}{}", &line[..s], atom, t, &line[e..]));
            // integer literal -> float literal is the idiomatic spelling
            if *t == "float" && atom.bytes().all(|c| c.is_ascii_digit()) {
                v.push(format!("{}{}.0{}", &line[..s], atom, &line[e..]));
            }
        }
    }
    v
}

// ---- class: unknown-name near-miss rename ----

/// Operator spellings models import from other languages.
const ALIASES: &[(&str, &str)] = &[("mod", "%"), ("rem", "%"), ("div", "/")];

const KEYWORDS: &[&str] = &[
    "if", "elif", "else", "while", "for", "in", "match", "type", "ret", "go",
    "pub", "and", "or", "not", "true", "false", "none",
];

fn levenshtein_le1(a: &str, b: &str) -> bool {
    let (al, bl) = (a.len(), b.len());
    if al.abs_diff(bl) > 1 {
        return false;
    }
    let (ab, bb) = (a.as_bytes(), b.as_bytes());
    if al == bl {
        return ab.iter().zip(bb).filter(|(x, y)| x != y).count() <= 1;
    }
    // one insertion: align the longer against the shorter
    let (long, short) = if al > bl { (ab, bb) } else { (bb, ab) };
    let mut skipped = false;
    let (mut i, mut j) = (0, 0);
    while i < long.len() && j < short.len() {
        if long[i] == short[j] {
            i += 1;
            j += 1;
        } else if skipped {
            return false;
        } else {
            skipped = true;
            i += 1;
        }
    }
    true
}

/// Replace whole-word occurrences of `sym` in `line` with `with`.
fn replace_word(line: &str, sym: &str, with: &str) -> String {
    let b = line.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        let boundary_ok = i == 0
            || !((b[i - 1] as char).is_ascii_alphanumeric() || b[i - 1] == b'_');
        if boundary_ok && line[i..].starts_with(sym) {
            let after = i + sym.len();
            let after_ok = after >= b.len()
                || !((b[after] as char).is_ascii_alphanumeric() || b[after] == b'_');
            if after_ok {
                out.push_str(with);
                i = after;
                continue;
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

fn rename_candidates(src: &str, line: &str, msg: &str) -> Vec<String> {
    let sym = match (msg.find('`'), msg.rfind('`')) {
        (Some(a), Some(b)) if b > a + 1 => &msg[a + 1..b],
        _ => return Vec::new(),
    };
    let mut v = Vec::new();
    for (alias, op) in ALIASES {
        if sym == *alias {
            v.push(replace_word(line, sym, op));
        }
    }
    // edit-distance-1 rename against every other identifier in the program
    let mut names: Vec<&str> = Vec::new();
    for (s, e) in atoms_of_src(src) {
        let w = &src[s..e];
        if w != sym
            && !KEYWORDS.contains(&w)
            && !names.contains(&w)
            && levenshtein_le1(sym, w)
        {
            names.push(w);
        }
    }
    for n in names {
        v.push(replace_word(line, sym, n));
    }
    v
}

fn atoms_of_src(src: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut base = 0;
    for line in src.lines() {
        for (s, e) in atoms(line) {
            if line.as_bytes()[s].is_ascii_alphabetic() || line.as_bytes()[s] == b'_' {
                out.push((base + s, base + e));
            }
        }
        base += line.len() + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diag_of(src: &str) -> Diag {
        crate::parse_source_spanned(src)
            .and_then(|(ast, pos)| crate::infer::check_at(&ast, &pos).map(|_| ()))
            .unwrap_err()
    }

    #[test]
    fn expected_token_missing_closer() {
        let src = "print (1 + 2\n";
        let d = diag_of(src);
        assert_eq!(d.err, "expected");
        let r = synthesize(src, &d).expect("payload");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, 1);
        assert_eq!(r[0].1, "print (1 + 2)");
    }

    #[test]
    fn type_conversion_str_plus_int() {
        let src = "print (\"a\" + 1)\n";
        let d = diag_of(src);
        assert_eq!(d.err, "type_mismatch");
        let r = synthesize(src, &d).expect("payload");
        assert_eq!(r[0].1, "print (\"a\" + 1.str)");
        let edits = vec![(0usize, r[0].1.clone())];
        assert!(checks_clean(&splice(&src.lines().collect::<Vec<_>>(), &edits)));
    }

    #[test]
    fn print_match_parenthesized() {
        let src = "r = \"5\".int\nprint match r { int n -> n, err _ -> 0 }\n";
        let d = diag_of(src);
        assert_eq!(d.err, "expected");
        let r = synthesize(src, &d).expect("payload");
        assert_eq!(r[0].1, "print (match r { int n -> n, err _ -> 0 })");
    }

    #[test]
    fn unparseable_annotation_dropped() {
        let src = "totals: {str: float} = {}\nprint totals.len\n";
        let d = diag_of(src);
        assert_eq!(d.err, "expected");
        let r = synthesize(src, &d).expect("payload");
        assert_eq!(r[0].1, "totals = {}");
    }

    #[test]
    fn multiline_literal_missing_commas() {
        let src = "rows = [\n  {a: 1}\n  {a: 2}\n  {a: 3}\n]\nprint rows.len\n";
        let d = diag_of(src);
        assert_eq!(d.err, "type_mismatch");
        let r = synthesize(src, &d).expect("payload");
        assert_eq!(
            r,
            vec![(2, "  {a: 1},".to_string()), (3, "  {a: 2},".to_string())]
        );
    }

    #[test]
    fn unknown_name_mod_alias() {
        let src = "nums = [1, 2, 3]\nodds = nums | keep (n -> n mod 2 != 0)\nprint odds\n";
        let d = diag_of(src);
        assert_eq!(d.err, "unknown_name");
        let r = synthesize(src, &d).expect("payload");
        assert_eq!(r[0].1, "odds = nums | keep (n -> n % 2 != 0)");
    }

    #[test]
    fn unknown_name_edit_distance_rename() {
        let src = "total = 41\nprint (totl + 1)\n";
        let d = diag_of(src);
        assert_eq!(d.err, "unknown_name");
        let r = synthesize(src, &d).expect("payload");
        assert_eq!(r[0].1, "print (total + 1)");
    }

    #[test]
    fn structural_break_yields_none() {
        // fixing this needs a multi-line brace rewrite — no payload
        let src = "check val =\n  if val > 5 { ret 1 }\n  2\nprint check 9\n";
        let d = diag_of(src);
        assert!(synthesize(src, &d).is_none());
    }

    #[test]
    fn runtime_line_zero_yields_none() {
        let d = Diag::at("runtime", 0, 0, "x", "y");
        assert!(synthesize("print 1\n", &d).is_none());
    }
}
