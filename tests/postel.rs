//! Postel set (SPEC §10): predictable slips parse; they are never errors.

use cmm::ast::{Expr, Stmt};

fn ok(src: &str) -> Vec<Stmt> {
    cmm::parse_source(src).unwrap_or_else(|d| panic!("should parse: {src:?} -> {d}"))
}

#[test]
fn eq_as_eqeq_in_expression_position() {
    let ast = ok("if x = 1 { print x }");
    let Stmt::Expr(Expr::If { cond, .. }) = &ast[0] else { panic!("want if") };
    let Expr::Binary { op, .. } = cond.as_ref() else { panic!("want binary cond") };
    assert_eq!(op, "==");
}

#[test]
fn ampamp_pipepipe_as_words() {
    let ast = ok("ok = a && b || c");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!("want binding") };
    let Expr::Binary { op, .. } = value else { panic!("want or at top") };
    assert_eq!(op, "or");
}

#[test]
fn bang_as_not() {
    let ast = ok("v = !done");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    assert!(matches!(value, Expr::Unary { op, .. } if op == "not"));
}

#[test]
fn bang_eq_is_still_ne() {
    let ast = ok("v = a != b");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    assert!(matches!(value, Expr::Binary { op, .. } if op == "!="));
}

#[test]
fn python_bool_and_none_literals() {
    let ast = ok("a = True\nb = False\nc = None");
    assert!(matches!(&ast[0], Stmt::Binding { value: Expr::Bool(true), .. }));
    assert!(matches!(&ast[1], Stmt::Binding { value: Expr::Bool(false), .. }));
    assert!(matches!(&ast[2], Stmt::Binding { value: Expr::Unit, .. }));
}

#[test]
fn return_maps_to_ret() {
    let ast = ok("f x = { return x }");
    let Stmt::Equation { body, .. } = &ast[0] else { panic!() };
    let cmm::ast::Body::Block(stmts) = body else { panic!() };
    assert!(matches!(&stmts[0], Stmt::Ret(Some(_))));
}

#[test]
fn elif_chain() {
    let ast = ok("sign x = if x < 0 { -1 } elif x > 0 { 1 } else { 0 }");
    let Stmt::Equation { body, .. } = &ast[0] else { panic!() };
    let cmm::ast::Body::Expr(Expr::If { els, .. }) = body else { panic!("want if body") };
    assert!(matches!(els.as_deref(), Some(Expr::If { .. })), "elif becomes nested if");
}

#[test]
fn trailing_commas_in_list_and_record() {
    ok("xs = [1, 2, 3,]");
    ok("r = {a: 1, b: 2,}");
}

#[test]
fn paren_call_sugar() {
    let ast = ok("v = f(x, y)");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    let Expr::App { args, .. } = value else { panic!("want call-sugar App") };
    assert_eq!(args.len(), 2);
}

#[test]
fn paren_call_sugar_python_print() {
    let ast = ok("print(\"hi\")");
    let Stmt::Expr(Expr::App { head, args }) = &ast[0] else { panic!("want App") };
    assert!(matches!(head.as_ref(), Expr::Name(n) if n == "print"));
    assert_eq!(args.len(), 1);
}

#[test]
fn spaced_paren_is_juxtaposition_not_call() {
    let ast = ok("v = f (x)");
    let Stmt::Binding { value, .. } = &ast[0] else { panic!() };
    let Expr::App { args, .. } = value else { panic!("want App") };
    assert_eq!(args.len(), 1, "f (x) is juxtaposition with one paren arg");
}

#[test]
fn canonical_forms_still_parse() {
    ok("ok = a and b or not c");
    ok("v = x == 1");
}
