//! Lexer + parser unit tests for the SPEC rules the corpus exercises implicitly.

use curt::ast::{Body, Expr, Pattern, Stmt, TypeExpr};
use curt::lexer::{lex, Tok};

fn ok(src: &str) -> Vec<Stmt> {
    curt::parse_source(src).unwrap_or_else(|d| panic!("should parse: {src:?} -> {d}"))
}

// ---- lexer ----

#[test]
fn lex_glued_question_vs_spaced() {
    let toks = lex("a? b ? c").unwrap();
    let qs: Vec<bool> = toks.iter().filter(|t| t.tok == Tok::Question).map(|t| t.glued).collect();
    assert_eq!(qs, vec![true, false]);
}

#[test]
fn lex_number_suffix() {
    let toks = lex("h = 7u64 + 9").unwrap();
    assert!(toks.iter().any(|t| t.tok == Tok::Num("7u64".into())));
    assert!(toks.iter().any(|t| t.tok == Tok::Num("9".into())));
}

#[test]
fn lex_bad_suffix_not_consumed() {
    let toks = lex("7up").unwrap();
    assert!(toks.iter().any(|t| t.tok == Tok::Num("7".into())));
    assert!(toks.iter().any(|t| t.tok == Tok::Name("up".into())));
}

#[test]
fn lex_string_escapes_and_interp_braces() {
    let toks = lex(r#"print "{p.k} {p.v}\n""#).unwrap();
    assert!(toks.iter().any(|t| matches!(&t.tok, Tok::Str(s) if s.contains("{p.k}") && s.contains("\\n"))));
}

#[test]
fn lex_comments_and_newline_collapse() {
    let toks = lex("a = 1 # comment\n\n# full line\n\nb = 2").unwrap();
    let newlines = toks.iter().filter(|t| t.tok == Tok::Newline).count();
    assert_eq!(newlines, 1, "newline runs collapse to one separator");
}

#[test]
fn lex_unterminated_string_is_diag() {
    let err = lex("s = \"oops").unwrap_err();
    assert_eq!(err.err, "unterminated_string");
}

// ---- parser ----

#[test]
fn equation_simple() {
    let ast = ok("hyp a b = (a*a + b*b).sqrt");
    let Stmt::Equation { name, params, body } = &ast[0] else { panic!() };
    assert_eq!(name, "hyp");
    assert_eq!(params, &["a", "b"]);
    assert!(matches!(body, Body::Expr(Expr::Field { .. })));
}

#[test]
fn equation_statement_body_server_rule() {
    let ast = ok("handle c = for ln in c.lines { c.write ln }");
    let Stmt::Equation { body, .. } = &ast[0] else { panic!() };
    assert!(matches!(body, Body::Stmt(s) if matches!(**s, Stmt::For { .. })));
}

#[test]
fn header_brace_rule_for() {
    // `{` after the iterable must start the block even though m.pairs could
    // syntactically be followed by a record argument in expression position.
    let ast = ok("for p in m.pairs { print p }");
    let Stmt::For { iter, body, .. } = &ast[0] else { panic!() };
    assert!(matches!(iter, Expr::Field { .. }));
    assert_eq!(body.len(), 1);
}

#[test]
fn header_brace_rule_through_lambda() {
    // the lambda body inside a for-header must also stop at `{`
    let ast = ok("for ln in xs | keep x -> e in x { print ln }");
    assert!(matches!(&ast[0], Stmt::For { .. }));
}

#[test]
fn glued_question_propagates() {
    let ast = ok("f s = { v = parse s?; v }");
    let Stmt::Equation { body: Body::Block(stmts), .. } = &ast[0] else { panic!() };
    let Stmt::Binding { value, .. } = &stmts[0] else { panic!() };
    let Expr::App { args, .. } = value else { panic!("want App") };
    assert!(matches!(&args[0], Expr::Propagate(_)), "glued ? is postfix propagate");
}

#[test]
fn spaced_question_rescues() {
    let ast = ok("cfg = load p ? {}");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    assert!(matches!(value, Expr::Rescue { .. }));
}

#[test]
fn record_literal_named_and_anon() {
    let ast = ok("p = Pt{x:0, y:0}\nq = {name:\"a\", score:9}");
    assert!(matches!(&ast[0], Stmt::Binding { value: Expr::RecordLit { name: Some(n), .. }, .. } if n == "Pt"));
    assert!(matches!(&ast[1], Stmt::Binding { value: Expr::RecordLit { name: None, fields }, .. } if fields.len() == 2));
}

#[test]
fn index_requires_glued_bracket() {
    let ast = ok("v = bs [1, 2] 7");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    let Expr::App { args, .. } = value else { panic!("want App: spaced [ is a list argument") };
    assert!(matches!(&args[0], Expr::List(items) if items.len() == 2));
}

#[test]
fn slice_forms() {
    let ast = ok("a = ts[1:]\nb = ts[0]\nc = s[i:j]");
    assert!(matches!(&ast[0], Stmt::Binding { value: Expr::Slice { lo: Some(_), hi: None, .. }, .. }));
    assert!(matches!(&ast[1], Stmt::Binding { value: Expr::Index { .. }, .. }));
    assert!(matches!(&ast[2], Stmt::Binding { value: Expr::Slice { lo: Some(_), hi: Some(_), .. }, .. }));
}

#[test]
fn match_with_typebind_literal_and_block_arm() {
    let ast = ok("factor ts = match ts[0] {\n  float x -> (x, ts[1:]),\n  \"(\" -> { v = expr ts; v }\n}");
    let Stmt::Equation { body: Body::Expr(Expr::Match { arms, .. }), .. } = &ast[0] else { panic!() };
    assert_eq!(arms.len(), 2);
    assert!(matches!(&arms[0].0, Pattern::TypeBind { ty, .. } if ty == "float"));
    assert!(matches!(&arms[1].0, Pattern::Lit(Expr::Str(_))));
}

#[test]
fn destructure_statement() {
    let ast = ok("(lo, hi) = minmax xs");
    assert!(matches!(&ast[0], Stmt::Destructure { names, .. } if names == &["lo", "hi"]));
}

#[test]
fn pub_sig_with_types() {
    let ast = ok("pub add :: int int -> int\nadd a b = a + b");
    let Stmt::Sig { public, name, params, ret } = &ast[0] else { panic!() };
    assert!(*public);
    assert_eq!(name, "add");
    assert_eq!(params.len(), 2);
    assert!(matches!(ret, Some(TypeExpr::Named(n)) if n == "int"));
}

#[test]
fn union_type_in_sig() {
    let ast = ok("show :: float | str -> str");
    let Stmt::Sig { params, .. } = &ast[0] else { panic!() };
    assert!(matches!(&params[0], TypeExpr::Union(parts) if parts.len() == 2));
}

#[test]
fn type_record_decl() {
    let ast = ok("type Pt = {x float, y float}");
    assert!(matches!(&ast[0], Stmt::TypeDecl { name, ty: TypeExpr::Record(f) } if name == "Pt" && f.len() == 2));
}

#[test]
fn projection_atom_as_argument() {
    let ast = ok("best = top 3 .score");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    let Expr::App { args, .. } = value else { panic!() };
    assert!(matches!(&args[1], Expr::Proj(p) if p == "score"));
}

#[test]
fn tuple_field_projection() {
    let ast = ok("v = (expr ts).0");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    assert!(matches!(value, Expr::Field { name, .. } if name == "0"));
}

#[test]
fn pipeline_stages() {
    let ast = ok("v = xs | keep .active | map .name");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    assert!(matches!(value, Expr::Pipe { stages } if stages.len() == 3));
}

#[test]
fn multiline_block_with_semicolons() {
    let ast = ok("f ts = {\n  (v, r) = g ts; (v, r)\n}");
    let Stmt::Equation { body: Body::Block(stmts), .. } = &ast[0] else { panic!() };
    assert_eq!(stmts.len(), 2);
}

#[test]
fn compound_assign_on_index_target() {
    let ast = ok("m[w] += 1");
    assert!(matches!(&ast[0], Stmt::Compound { op, .. } if op == "+="));
}

#[test]
fn parse_error_is_fix_suggesting_json() {
    let d = curt::parse_source("type lower = {x float}").unwrap_err();
    let s = d.to_string();
    assert!(s.starts_with("{\"err\":"), "diagnostic is single-line JSON: {s}");
    assert!(s.contains("\"fix\":"));
}

#[test]
fn annotated_binding() {
    let ast = ok("x: int = 1");
    assert!(matches!(&ast[0], Stmt::Binding { ann: Some(TypeExpr::Named(t)), .. } if t == "int"));
}

#[test]
fn go_and_range() {
    let ast = ok("for i in range 4 { go work i }");
    let Stmt::For { body, .. } = &ast[0] else { panic!() };
    assert!(matches!(&body[0], Stmt::Go(Expr::App { .. })));
}

#[test]
fn runtime_diag_with_nested_quotes_is_valid_json() {
    // diag-esc-runtime regression: main.rs hand-rolled the runtime JSON
    // without esc(), so quote-bearing messages (nested interpolation
    // diags) emitted invalid JSON. Exact-match the deterministic repro.
    use std::io::Write;
    use std::process::Command;
    let dir = std::env::temp_dir().join("curt_diag_esc_test");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("repro.curt");
    let mut fh = std::fs::File::create(&f).unwrap();
    writeln!(fh, "print \"{{\\\"num\\\": 1}}\"").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_curt"))
        .args(["run", f.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    let line = stderr.trim().lines().last().unwrap();
    assert_eq!(
        line,
        "{\"err\":\"runtime\",\"at\":\"0:0\",\"msg\":\"bad interpolation `{\\\\\\\"num\\\\\\\": 1}`: \
         {\\\"err\\\":\\\"unexpected_char\\\",\\\"at\\\":\\\"1:1\\\",\\\"msg\\\":\\\"character '\\\\\\\\\\\\\\\\' is not part of curt\\\",\
         \\\"fix\\\":\\\"remove it or check the SPEC lexical rules\\\",\\\"repair\\\":{\\\"id\\\":\\\"remove-char\\\",\
         \\\"summary\\\":\\\"remove the invalid character\\\"}}\",\"fix\":\"inspect the failure and rerun\",\
         \"repair\":{\"id\":\"manual-review\",\"summary\":\"inspect the diagnostic and repair manually\"}}"
    );
}
