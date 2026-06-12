//! flagship gate: corpus/22_logmill.curt must exercise the full v0.3
//! construct checklist, asserted on the parsed AST as written (the synth-v1
//! rawstr audit showed text-level presence checks can be silently defeated).
//! Raw-string and multi-line-literal SPELLING is asserted textually because
//! neither the AST nor the token stream records spelling — see roadmap
//! chunk fmt-rawstr; the discoveries match-recordarm and sig-err-any are
//! also pinned by this program's design (records are data carriers, the
//! pub :: signature sits on an equation whose contract the checker holds).

use curt::ast::*;

#[derive(Default, Debug)]
struct Seen {
    pipeline: bool,
    match_narrow_tys: std::collections::BTreeSet<String>,
    match_err: bool,
    rescue: bool,
    rescue_chain: bool,
    propagate: bool,
    maplit: bool,
    record_decl: bool,
    union_decl: bool,
    record_lit: bool,
    annotation: bool,
    pub_sig: bool,
    numjoin: bool,
    fs_read: bool,
    json_verb: bool,
    go_stmt: bool,
    for_loop: bool,
    while_loop: bool,
}

fn walk_stmts(stmts: &[Stmt], seen: &mut Seen) {
    for s in stmts {
        walk_stmt(s, seen);
    }
}

fn walk_stmt(s: &Stmt, seen: &mut Seen) {
    match s {
        Stmt::TypeDecl { ty, .. } => match ty {
            TypeExpr::Record(_) => seen.record_decl = true,
            TypeExpr::Union(_) => seen.union_decl = true,
            _ => {}
        },
        Stmt::Sig { public, .. } => {
            if *public {
                seen.pub_sig = true;
            }
        }
        Stmt::Equation { body, .. } => match body {
            Body::Expr(e) => walk_expr(e, seen),
            Body::Block(b) => walk_stmts(b, seen),
            Body::Stmt(st) => walk_stmt(st, seen),
        },
        Stmt::Destructure { value, .. } => walk_expr(value, seen),
        Stmt::Binding { ann, value, .. } => {
            if ann.is_some() {
                seen.annotation = true;
            }
            walk_expr(value, seen);
        }
        Stmt::Compound { value, .. } => walk_expr(value, seen),
        Stmt::For { iter, body, .. } => {
            seen.for_loop = true;
            walk_expr(iter, seen);
            walk_stmts(body, seen);
        }
        Stmt::While { cond, body } => {
            seen.while_loop = true;
            walk_expr(cond, seen);
            walk_stmts(body, seen);
        }
        Stmt::Ret(e) => {
            if let Some(e) = e {
                walk_expr(e, seen);
            }
        }
        Stmt::Go(e) => {
            seen.go_stmt = true;
            walk_expr(e, seen);
        }
        Stmt::Expr(e) => walk_expr(e, seen),
    }
}

fn contains_rescue(e: &Expr) -> bool {
    let mut found = false;
    visit(e, &mut |x| {
        if matches!(x, Expr::Rescue { .. }) {
            found = true;
        }
    });
    found
}

fn ends_in_float(e: &Expr) -> bool {
    matches!(e, Expr::Field { name, .. } if name == "float")
}

fn visit(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(e);
    match e {
        Expr::List(xs) | Expr::Tuple(xs) => xs.iter().for_each(|x| visit(x, f)),
        Expr::RecordLit { fields, .. } => fields.iter().for_each(|(_, x)| visit(x, f)),
        Expr::MapLit(fields) => fields.iter().for_each(|(_, x)| visit(x, f)),
        Expr::Block(stmts) => stmts.iter().for_each(|s| {
            if let Stmt::Expr(x) = s {
                visit(x, f)
            }
        }),
        Expr::App { head, args } => {
            visit(head, f);
            args.iter().for_each(|x| visit(x, f));
        }
        Expr::Lambda { body, .. } => visit(body, f),
        Expr::Field { recv, .. } => visit(recv, f),
        Expr::Index { recv, index } => {
            visit(recv, f);
            visit(index, f);
        }
        Expr::Slice { recv, lo, hi } => {
            visit(recv, f);
            if let Some(lo) = lo {
                visit(lo, f);
            }
            if let Some(hi) = hi {
                visit(hi, f);
            }
        }
        Expr::Unary { expr, .. } => visit(expr, f),
        Expr::Binary { lhs, rhs, .. } => {
            visit(lhs, f);
            visit(rhs, f);
        }
        Expr::Pipe { stages } => stages.iter().for_each(|x| visit(x, f)),
        Expr::Paren(inner) | Expr::Propagate(inner) => visit(inner, f),
        Expr::Rescue { value, fallback } => {
            visit(value, f);
            visit(fallback, f);
        }
        Expr::If { cond, then, els } => {
            visit(cond, f);
            then.iter().for_each(|s| {
                if let Stmt::Expr(x) = s {
                    visit(x, f)
                }
            });
            if let Some(els) = els {
                visit(els, f);
            }
        }
        Expr::Match { subject, arms } => {
            visit(subject, f);
            arms.iter().for_each(|(_, x)| visit(x, f));
        }
        _ => {}
    }
}

fn walk_expr(e: &Expr, seen: &mut Seen) {
    match e {
        Expr::Pipe { stages } => {
            seen.pipeline = true;
            stages.iter().for_each(|x| walk_expr(x, seen));
        }
        Expr::Rescue { value, fallback } => {
            seen.rescue = true;
            if contains_rescue(value) || contains_rescue(fallback) {
                seen.rescue_chain = true;
            }
            walk_expr(value, seen);
            walk_expr(fallback, seen);
        }
        Expr::Propagate(inner) => {
            seen.propagate = true;
            walk_expr(inner, seen);
        }
        Expr::MapLit(fields) => {
            seen.maplit = true;
            fields.iter().for_each(|(_, x)| walk_expr(x, seen));
        }
        Expr::RecordLit { fields, .. } => {
            seen.record_lit = true;
            fields.iter().for_each(|(_, x)| walk_expr(x, seen));
        }
        Expr::Match { subject, arms } => {
            walk_expr(subject, seen);
            for (pat, body) in arms {
                if let Pattern::TypeBind { ty, .. } = pat {
                    if ty == "err" {
                        seen.match_err = true;
                    } else {
                        seen.match_narrow_tys.insert(ty.clone());
                    }
                }
                walk_expr(body, seen);
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            if matches!(op.as_str(), "+" | "-" | "*" | "/")
                && (ends_in_float(lhs) ^ ends_in_float(rhs))
            {
                seen.numjoin = true;
            }
            walk_expr(lhs, seen);
            walk_expr(rhs, seen);
        }
        Expr::Field { recv, name } => {
            if name == "read" && matches!(recv.as_ref(), Expr::Name(n) if n == "fs") {
                seen.fs_read = true;
            }
            if name == "json" {
                seen.json_verb = true;
            }
            walk_expr(recv, seen);
        }
        Expr::App { head, args } => {
            if matches!(head.as_ref(), Expr::Name(n) if n == "json") {
                seen.json_verb = true;
            }
            walk_expr(head, seen);
            args.iter().for_each(|x| walk_expr(x, seen));
        }
        Expr::Block(stmts) => walk_stmts(stmts, seen),
        Expr::If { cond, then, els } => {
            walk_expr(cond, seen);
            walk_stmts(then, seen);
            if let Some(els) = els {
                walk_expr(els, seen);
            }
        }
        Expr::List(xs) | Expr::Tuple(xs) => xs.iter().for_each(|x| walk_expr(x, seen)),
        Expr::Lambda { body, .. } => walk_expr(body, seen),
        Expr::Index { recv, index } => {
            walk_expr(recv, seen);
            walk_expr(index, seen);
        }
        Expr::Slice { recv, lo, hi } => {
            walk_expr(recv, seen);
            if let Some(lo) = lo {
                walk_expr(lo, seen);
            }
            if let Some(hi) = hi {
                walk_expr(hi, seen);
            }
        }
        Expr::Unary { expr, .. } => walk_expr(expr, seen),
        Expr::Paren(inner) => walk_expr(inner, seen),
        _ => {}
    }
}

#[test]
fn flagship_exercises_the_construct_checklist() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/22_logmill.curt"),
    )
    .expect("read flagship source");
    let prog = curt::parse_source(&src).expect("flagship parses");

    let mut seen = Seen::default();
    walk_stmts(&prog, &mut seen);

    // AST-level assertions (16 constructs)
    assert!(seen.pipeline, "pipeline");
    assert!(
        seen.match_narrow_tys.len() >= 2,
        "match narrows >=2 distinct types, got {:?}",
        seen.match_narrow_tys
    );
    assert!(seen.match_err, "match on err");
    assert!(seen.rescue, "rescue");
    assert!(seen.rescue_chain, "rescue chain (nested rescue)");
    assert!(seen.propagate, "propagate ?");
    assert!(seen.maplit, "map literal");
    assert!(seen.record_decl, "record type decl");
    assert!(seen.union_decl, "union type decl");
    assert!(seen.record_lit, "record literal");
    assert!(seen.annotation, "binding annotation");
    assert!(seen.pub_sig, "pub :: signature");
    assert!(seen.numjoin, "numeric join (arith with exactly one .float side)");
    assert!(seen.fs_read, "fs.read");
    assert!(seen.json_verb, "json verb");
    assert!(seen.go_stmt, "go statement");
    assert!(seen.for_loop, "for loop");
    assert!(seen.while_loop, "while loop");

    // Textual assertions (spelling is not recorded in AST/tokens — fmt-rawstr)
    assert!(src.contains("= '"), "raw string '...' bound at top level");
    assert!(
        src.lines().any(|l| l.trim_end().ends_with("= {")),
        "multi-line map literal (binding opens brace at EOL)"
    );
    assert!(
        src.lines().any(|l| {
            let t = l.trim_end();
            t.ends_with('{')
                && t.len() >= 2
                && t[..t.len() - 1]
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_ascii_alphanumeric())
                && t[..t.len() - 1]
                    .rsplit(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .next()
                    .is_some_and(|w| w.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        }),
        "multi-line record literal (TName opens brace at EOL)"
    );
}
