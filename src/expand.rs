//! `cmm expand` v1 — the readability-as-view projection (DESIGN P10).
//!
//! Renders the parse tree with sugar spelled out and grouping explicit:
//! projection atoms become lambdas, applications/operators are fully
//! parenthesized, Postel inputs appear canonicalized (the parser already
//! normalized them). Type reveal lands in interp-c.

use crate::ast::*;

pub fn expand(prog: &[Stmt]) -> String {
    let mut out = String::new();
    for s in prog {
        stmt(s, 0, &mut out);
        out.push('\n');
    }
    out
}

fn ind(n: usize, out: &mut String) {
    out.push_str(&"  ".repeat(n));
}

fn stmt(s: &Stmt, depth: usize, out: &mut String) {
    ind(depth, out);
    match s {
        Stmt::TypeDecl { name, ty } => {
            out.push_str(&format!("type {name} = {}", type_expr(ty)));
        }
        Stmt::Sig { public, name, params, ret } => {
            if *public {
                out.push_str("pub ");
            }
            out.push_str(&format!("{name} :: "));
            out.push_str(&params.iter().map(type_expr).collect::<Vec<_>>().join(" "));
            if let Some(r) = ret {
                out.push_str(&format!(" -> {}", type_expr(r)));
            }
        }
        Stmt::Equation { name, params, body } => {
            out.push_str(&format!("{name} {} =", params.join(" ")));
            match body {
                Body::Expr(e) => {
                    out.push(' ');
                    out.push_str(&expr(e, depth));
                }
                Body::Block(stmts) => {
                    out.push_str(" {\n");
                    for st in stmts {
                        stmt(st, depth + 1, out);
                        out.push('\n');
                    }
                    ind(depth, out);
                    out.push('}');
                }
                Body::Stmt(st) => {
                    out.push('\n');
                    stmt(st, depth + 1, out);
                }
            }
        }
        Stmt::Destructure { names, value } => {
            out.push_str(&format!("({}) = {}", names.join(", "), expr(value, depth)));
        }
        Stmt::Binding { target, ann, value } => {
            out.push_str(&target_str(target, depth));
            if let Some(t) = ann {
                out.push_str(&format!(": {}", type_expr(t)));
            }
            out.push_str(&format!(" = {}", expr(value, depth)));
        }
        Stmt::Compound { target, op, value } => {
            out.push_str(&format!("{} {op} {}", target_str(target, depth), expr(value, depth)));
        }
        Stmt::For { pat, iter, body } => {
            let p = if pat.len() == 1 { pat[0].clone() } else { format!("({})", pat.join(", ")) };
            out.push_str(&format!("for {p} in {} {{\n", expr(iter, depth)));
            for st in body {
                stmt(st, depth + 1, out);
                out.push('\n');
            }
            ind(depth, out);
            out.push('}');
        }
        Stmt::While { cond, body } => {
            out.push_str(&format!("while {} {{\n", expr(cond, depth)));
            for st in body {
                stmt(st, depth + 1, out);
                out.push('\n');
            }
            ind(depth, out);
            out.push('}');
        }
        Stmt::Ret(e) => {
            out.push_str("ret");
            if let Some(e) = e {
                out.push(' ');
                out.push_str(&expr(e, depth));
            }
        }
        Stmt::Go(e) => {
            out.push_str(&format!("go {}", expr(e, depth)));
        }
        Stmt::Expr(e) => out.push_str(&expr(e, depth)),
    }
}

fn target_str(t: &Target, depth: usize) -> String {
    let mut s = t.name.clone();
    for ix in &t.indices {
        s.push_str(&format!("[{}]", expr(ix, depth)));
    }
    s
}

fn type_expr(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Named(n) => n.clone(),
        TypeExpr::Union(parts) => parts.iter().map(type_expr).collect::<Vec<_>>().join(" | "),
        TypeExpr::Record(fields) => {
            let inner = fields.iter().map(|(n, t)| format!("{n} {}", type_expr(t))).collect::<Vec<_>>().join(", ");
            format!("{{{inner}}}")
        }
        TypeExpr::List(inner) => format!("[{}]", type_expr(inner)),
        TypeExpr::Fn { params, ret } => {
            format!("({} -> {})", params.iter().map(type_expr).collect::<Vec<_>>().join(" "), type_expr(ret))
        }
    }
}

fn expr(e: &Expr, depth: usize) -> String {
    match e {
        Expr::Num(n) => n.clone(),
        Expr::Str(s) => s.clone(),
        Expr::Bool(b) => b.to_string(),
        Expr::Unit => "()".into(),
        Expr::Name(n) => n.clone(),
        Expr::TName(n) => n.clone(),
        // sugar expansion: bare projection becomes an explicit lambda
        Expr::Proj(p) => format!("(x -> x.{p})"),
        Expr::List(items) => {
            format!("[{}]", items.iter().map(|i| expr(i, depth)).collect::<Vec<_>>().join(", "))
        }
        Expr::Tuple(items) => {
            format!("({})", items.iter().map(|i| expr(i, depth)).collect::<Vec<_>>().join(", "))
        }
        Expr::RecordLit { name, fields } => {
            let inner = fields.iter().map(|(n, v)| format!("{n}: {}", expr(v, depth))).collect::<Vec<_>>().join(", ");
            format!("{}{{{inner}}}", name.clone().unwrap_or_default())
        }
        Expr::Block(stmts) => {
            let mut s = String::from("{\n");
            for st in stmts {
                stmt(st, depth + 1, &mut s);
                s.push('\n');
            }
            s.push_str(&"  ".repeat(depth));
            s.push('}');
            s
        }
        // sugar expansion: applications fully parenthesized, flat args visible
        Expr::App { head, args } => {
            let mut s = format!("({}", expr(head, depth));
            for a in args {
                s.push(' ');
                s.push_str(&expr(a, depth));
            }
            s.push(')');
            s
        }
        Expr::Lambda { params, body } => {
            format!("({} -> {})", params.join(" "), expr(body, depth))
        }
        Expr::Field { recv, name } => format!("{}.{name}", expr(recv, depth)),
        Expr::Index { recv, index } => format!("{}[{}]", expr(recv, depth), expr(index, depth)),
        Expr::Slice { recv, lo, hi } => {
            format!(
                "{}[{}:{}]",
                expr(recv, depth),
                lo.as_ref().map(|e| expr(e, depth)).unwrap_or_default(),
                hi.as_ref().map(|e| expr(e, depth)).unwrap_or_default()
            )
        }
        Expr::Unary { op, expr: inner } => {
            if op == "not" {
                format!("(not {})", expr(inner, depth))
            } else {
                format!("({op}{})", expr(inner, depth))
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            format!("({} {op} {})", expr(lhs, depth), expr(rhs, depth))
        }
        Expr::Pipe { stages } => {
            format!("({})", stages.iter().map(|s| expr(s, depth)).collect::<Vec<_>>().join(" | "))
        }
        Expr::Rescue { value, fallback } => {
            format!("({} ? {})", expr(value, depth), expr(fallback, depth))
        }
        Expr::Propagate(inner) => format!("({}?)", expr(inner, depth)),
        Expr::If { cond, then, els } => {
            let mut s = format!("if {} {{\n", expr(cond, depth));
            for st in then {
                stmt(st, depth + 1, &mut s);
                s.push('\n');
            }
            s.push_str(&"  ".repeat(depth));
            s.push('}');
            if let Some(e) = els {
                s.push_str(" else ");
                s.push_str(&expr(e, depth));
            }
            s
        }
        Expr::Match { subject, arms } => {
            let mut s = format!("match {} {{\n", expr(subject, depth));
            for (p, body) in arms {
                s.push_str(&"  ".repeat(depth + 1));
                s.push_str(&format!("{} -> {},\n", pattern(p), expr(body, depth + 1)));
            }
            s.push_str(&"  ".repeat(depth));
            s.push('}');
            s
        }
    }
}

fn pattern(p: &Pattern) -> String {
    match p {
        Pattern::TypeBind { ty, name } => format!("{ty} {name}"),
        Pattern::Lit(e) => expr(e, 0),
        Pattern::Tuple(names) => format!("({})", names.join(", ")),
        Pattern::Wildcard => "_".into(),
        Pattern::Bind(n) => n.clone(),
    }
}
