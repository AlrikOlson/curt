//! Diagnostics-as-prompts (SPEC §7): single-line JSON designed to be fed
//! back to a model verbatim for one-edit self-repair.
//!
//! Shape (adopted 2026-06-12 from the vz-diag tournament, 32 errors × 4
//! renderings, haiku: typed fields beat the canned prose hint +9.4pp
//! turn-1 repair success at 1.13× tokens; tools/bench/tourney/):
//!
//!   {"err":CODE,"at":"L:C",<typed fields>,"repair":{"id":ID,"summary":S}}
//!
//! Typed fields are derived from the message where the derivation is
//! mechanical — `want`/`got` (SPEC §7's original vocabulary) from
//! "expected X, got|found Y", `symbol` from "`X` is not defined",
//! `callee` from "X is not callable". Where no derivation applies, the
//! prose `msg` and per-site `fix` hint are retained so no information is
//! lost. `repair` is always present: a stable operation id + summary per
//! error code (the measured steelman of typed repair identifiers).

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

/// Stable repair operation per error code (tournament arm-B table).
fn repair_op(err: &str) -> (&'static str, &'static str) {
    match err {
        "type_mismatch" => ("align-types", "make the operand types agree"),
        "expected" => ("fix-syntax", "correct the syntax at the span"),
        "unknown_name" => ("define-or-rename", "define the name or fix its spelling"),
        "unknown_field" => ("use-existing-field", "use a field the record declares"),
        "unexpected_char" => ("remove-char", "remove the invalid character"),
        _ => ("manual-review", "inspect the diagnostic and repair manually"),
    }
}

/// Mechanical typed-field derivation from the prose message.
/// Returns rendered JSON fields (without leading comma) or None.
fn typed_fields(msg: &str) -> Option<String> {
    if let Some(rest) = msg.strip_prefix("expected ") {
        for sep in [", got ", ", found "] {
            if let Some(i) = rest.find(sep) {
                let (want, got) = (&rest[..i], &rest[i + sep.len()..]);
                return Some(format!(
                    "\"want\":\"{}\",\"got\":\"{}\"",
                    esc(want),
                    esc(got)
                ));
            }
        }
        return None;
    }
    if let Some(i) = msg.find(" is not defined") {
        if i + " is not defined".len() == msg.len() {
            let sym = msg[..i].trim_matches('`');
            return Some(format!("\"symbol\":\"{}\"", esc(sym)));
        }
    }
    if let Some(i) = msg.find(" is not callable") {
        if i + " is not callable".len() == msg.len() {
            return Some(format!("\"callee\":\"{}\"", esc(&msg[..i])));
        }
    }
    None
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (rid, rsum) = repair_op(&self.err);
        let body = match typed_fields(&self.msg) {
            Some(t) => t,
            None => format!(
                "\"msg\":\"{}\",\"fix\":\"{}\"",
                esc(&self.msg),
                esc(&self.fix)
            ),
        };
        write!(
            f,
            "{{\"err\":\"{}\",\"at\":\"{}:{}\",{},\"repair\":{{\"id\":\"{}\",\"summary\":\"{}\"}}}}",
            esc(&self.err),
            self.line,
            self.col,
            body,
            rid,
            rsum
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_want_got() {
        let d = Diag::at("type_mismatch", 3, 1, "expected int, got ()", "ignored hint");
        let s = d.to_string();
        assert_eq!(
            s,
            "{\"err\":\"type_mismatch\",\"at\":\"3:1\",\"want\":\"int\",\"got\":\"()\",\
             \"repair\":{\"id\":\"align-types\",\"summary\":\"make the operand types agree\"}}"
        );
    }

    #[test]
    fn typed_found_variant_and_syntax_repair() {
        let d = Diag::at("expected", 8, 8, "expected }, found Comma", "close the block");
        let s = d.to_string();
        assert!(s.contains("\"want\":\"}\""), "{s}");
        assert!(s.contains("\"got\":\"Comma\""), "{s}");
        assert!(s.contains("\"id\":\"fix-syntax\""), "{s}");
        assert!(!s.contains("close the block"), "canned hint replaced: {s}");
    }

    #[test]
    fn symbol_and_callee() {
        let d = Diag::at("unknown_name", 4, 1, "`mod` is not defined", "bind it first");
        assert!(d.to_string().contains("\"symbol\":\"mod\""));
        let d = Diag::at("type_mismatch", 2, 1, "str is not callable", "h");
        assert!(d.to_string().contains("\"callee\":\"str\""));
    }

    #[test]
    fn fallback_keeps_msg_and_fix_and_is_valid_json() {
        let d = Diag::at("unterminated_string", 1, 5, "string never closes", "add a closing \"");
        let s = d.to_string();
        assert!(s.contains("\"msg\":\"string never closes\""), "{s}");
        assert!(s.contains("\"fix\":\"add a closing \\\"\""), "{s}");
        assert!(s.contains("\"id\":\"manual-review\""), "{s}");
        // escaping check: the inner quote arrives escaped, brace balance holds
        assert_eq!(s.matches('{').count(), s.matches('}').count(), "{s}");
    }
}
