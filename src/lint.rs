//! `curt lint` — the density advisor (SPEC §11 doctrine made active).
//!
//! Flags provably-equivalent cheaper idioms as machine-actionable
//! diagnostics in the SPEC §7 shape: `{"err":"lint",...,"repair":{...,
//! "replacement":[{line,new}]}}`. The motive is measured (think:119): the
//! operator wrote a 17-token match where an 8-token rescue is semantically
//! identical — with the cheatsheet memorized. Models miss dense forms
//! constantly; the lint feeds them back.
//!
//! Rule classes (each equivalence-verified by golden-preservation tests in
//! tests/lint.rs, never assumed):
//!   R1 conversion-match → rescue:
//!      `match X { err _ -> FB, n -> n }`  ≡  `(X ? FB)`
//!      (err binder must be `_`; identity bind arm required)
//!   R2 boolean if → condition:
//!      `if C { true } else { false }`  ≡  `C`     (swapped → `not (C)`)
//!   R3 additive fold → sum:
//!      `xs.fold 0 acc x -> acc + x`  ≡  `xs.sum`  (also the pipe-stage form)
//!
//! Every replacement passes a triple gate before emission: the patched
//! program re-parses AND re-checks clean, and the rewritten statement is
//! strictly cheaper in o200k tokens. A finding whose payload fails the gate
//! ships prose-only (msg + fix hint, no replacement) — advisory, never wrong.

use crate::ast::*;
use crate::diag::Diag;

/// Lint a source string. Returns advisory diagnostics (possibly empty).
/// Parse/check failures yield the underlying diagnostic instead.
pub fn lint(src: &str) -> Result<Vec<Diag>, Diag> {
    let (stmts, pos) = crate::parse_source_spanned(src)?;
    let lines: Vec<&str> = src.lines().collect();
    #[cfg(feature = "tokens")]
    let bpe = tiktoken_rs::o200k_base().ok();
    #[cfg(feature = "tokens")]
    let tok = |s: &str| bpe.as_ref().map(|b| b.encode_ordinary(s).len()).unwrap_or(0);
    // no-tokens build: byte length is the documented monotone proxy — the
    // strictly-cheaper gate still holds (shorter statement => fewer bytes)
    #[cfg(not(feature = "tokens"))]
    let tok = |s: &str| s.len();

    let mut out = Vec::new();
    for (i, s) in stmts.iter().enumerate() {
        let mut hits: Vec<&'static str> = Vec::new();
        let dense = densify_stmt(s, &mut hits);
        if hits.is_empty() {
            continue;
        }
        let start = pos[i].0 as usize;
        let mut end = pos
            .get(i + 1)
            .map(|&(l, _)| (l as usize).saturating_sub(1))
            .unwrap_or(lines.len())
            .max(start);
        // trim blank/comment trailer lines out of the statement extent
        while end > start
            && lines
                .get(end - 1)
                .map(|l| l.trim().is_empty() || l.trim_start().starts_with('#'))
                .unwrap_or(true)
        {
            end -= 1;
        }
        let old_text = lines[start - 1..end.min(lines.len())].join("\n");
        let new_text = render_stmt(&dense);
        let (otok, ntok) = (tok(&old_text), tok(&new_text));
        let mut d = Diag::at(
            "lint",
            start as u32,
            1,
            &format!("denser form: {} ({otok}→{ntok} o200k)", hits.join("+")),
            &new_text,
        );
        // the triple gate: parse+check clean AND strictly cheaper
        if ntok < otok {
            let mut patched: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
            patched[start - 1] = new_text.clone();
            for l in patched.iter_mut().take(end.min(lines.len())).skip(start) {
                l.clear();
            }
            let patched = patched.join("\n") + "\n";
            let clean = crate::parse_source_spanned(&patched)
                .and_then(|(a, p)| crate::infer::check_at(&a, &p))
                .is_ok();
            if clean {
                let mut reps = vec![(start as u32, new_text)];
                for l in (start + 1)..=end.min(lines.len()) {
                    reps.push((l as u32, String::new()));
                }
                d.replacement = Some(reps);
            }
        }
        out.push(d);
    }
    Ok(out)
}

// ---- the rewrite rules ----

fn densify_stmt(s: &Stmt, hits: &mut Vec<&'static str>) -> Stmt {
    let mut e = |x: &Expr| densify(x, hits);
    match s {
        Stmt::Binding { target, ann, value } => {
            Stmt::Binding { target: target.clone(), ann: ann.clone(), value: e(value) }
        }
        Stmt::Compound { target, op, value } => {
            Stmt::Compound { target: target.clone(), op: op.clone(), value: e(value) }
        }
        Stmt::Destructure { names, value } => {
            Stmt::Destructure { names: names.clone(), value: e(value) }
        }
        Stmt::Equation { name, params, body } => Stmt::Equation {
            name: name.clone(),
            params: params.clone(),
            body: match body {
                Body::Expr(x) => Body::Expr(e(x)),
                Body::Block(ss) => Body::Block(ss.iter().map(|s| densify_stmt(s, hits)).collect()),
                Body::Stmt(s) => Body::Stmt(Box::new(densify_stmt(s, hits))),
            },
        },
        Stmt::For { pat, iter, body } => Stmt::For {
            pat: pat.clone(),
            iter: e(iter),
            body: body.iter().map(|s| densify_stmt(s, hits)).collect(),
        },
        Stmt::While { cond, body } => Stmt::While {
            cond: e(cond),
            body: body.iter().map(|s| densify_stmt(s, hits)).collect(),
        },
        Stmt::Ret(v) => Stmt::Ret(v.as_ref().map(&mut e)),
        Stmt::Go(x) => Stmt::Go(e(x)),
        Stmt::Expr(x) => Stmt::Expr(e(x)),
        other => other.clone(),
    }
}

/// Rebuild `x` bottom-up, replacing every rule match with its dense form.
fn densify(x: &Expr, hits: &mut Vec<&'static str>) -> Expr {
    // statement-bearing exprs recurse through densify_stmt (hits threading)
    let e = match x {
        Expr::Block(stmts) => {
            Expr::Block(stmts.iter().map(|s| densify_stmt(s, hits)).collect())
        }
        Expr::If { cond, then, els } => Expr::If {
            cond: Box::new(densify(cond, hits)),
            then: then.iter().map(|s| densify_stmt(s, hits)).collect(),
            els: els.as_ref().map(|e| Box::new(densify(e, hits))),
        },
        other => map_children(other, &mut |c| densify(c, hits)),
    };
    // R1: match X { err _ -> FB, n -> n }  →  (X ? FB)
    if let Expr::Match { subject, arms } = &e {
        if let [(Pattern::TypeBind { ty, name }, fb), (Pattern::Bind(n), Expr::Name(m))] =
            arms.as_slice()
        {
            if ty == "err" && name == "_" && n == m {
                hits.push("match-rescue");
                return Expr::Paren(Box::new(Expr::Rescue {
                    value: subject.clone(),
                    fallback: Box::new(fb.clone()),
                }));
            }
        }
    }
    // R2: if C { true } else { false }  →  C   (swapped → not (C))
    if let Expr::If { cond, then, els: Some(els) } = &e {
        fn lone_bool(e: &Expr) -> Option<bool> {
            match e {
                Expr::Bool(b) => Some(*b),
                Expr::Paren(inner) => lone_bool(inner),
                Expr::Block(stmts) => match stmts.as_slice() {
                    [Stmt::Expr(inner)] => lone_bool(inner),
                    _ => None,
                },
                _ => None,
            }
        }
        let then_bool = match then.as_slice() {
            [Stmt::Expr(inner)] => lone_bool(inner),
            _ => None,
        };
        let els_bool = lone_bool(els);
        if let (Some(t), Some(f)) = (then_bool, els_bool) {
            if t && !f {
                hits.push("bool-if");
                return cond.as_ref().clone();
            }
            if !t && f {
                hits.push("bool-if");
                return Expr::Unary {
                    op: "not".into(),
                    expr: Box::new(Expr::Paren(cond.clone())),
                };
            }
        }
    }
    // R3: xs.fold 0 acc x -> acc + x  →  xs.sum  (and the pipe-stage form).
    // Applications are FLAT juxtaposition (SPEC §2.3), so the fold can sit
    // anywhere in the arg sequence: scan [head, args..] for the window
    // [recv.fold, 0, additive-lambda] and collapse it to recv.sum.
    if let Expr::App { head, args } = &e {
        let mut seq: Vec<Expr> = Vec::with_capacity(args.len() + 1);
        seq.push(head.as_ref().clone());
        seq.extend(args.iter().cloned());
        for i in 0..seq.len().saturating_sub(2) {
            if let Expr::Field { recv, name } = &seq[i] {
                if name == "fold" && is_additive_fold(&seq[i + 1..=i + 2]) {
                    let dense = Expr::Field { recv: recv.clone(), name: "sum".into() };
                    seq.splice(i..=i + 2, [dense]);
                    hits.push("fold-sum");
                    let mut it = seq.into_iter();
                    let h = it.next().unwrap();
                    let rest: Vec<Expr> = it.collect();
                    return if rest.is_empty() {
                        h
                    } else {
                        Expr::App { head: Box::new(h), args: rest }
                    };
                }
            }
        }
    }
    if let Expr::Pipe { stages } = &e {
        let mut changed = false;
        let new_stages: Vec<Expr> = stages
            .iter()
            .map(|st| {
                if let Expr::App { head, args } = st {
                    if matches!(head.as_ref(), Expr::Name(n) if n == "fold")
                        && is_additive_fold(args)
                    {
                        changed = true;
                        return Expr::Name("sum".into());
                    }
                }
                st.clone()
            })
            .collect();
        if changed {
            hits.push("fold-sum");
            return Expr::Pipe { stages: new_stages };
        }
    }
    e
}

fn is_additive_fold(args: &[Expr]) -> bool {
    if let [Expr::Num(z), Expr::Lambda { params, body }] = args {
        if z == "0" && params.len() == 2 {
            if let Expr::Binary { op, lhs, rhs } = body.as_ref() {
                if op == "+" {
                    if let (Expr::Name(a), Expr::Name(b)) = (lhs.as_ref(), rhs.as_ref()) {
                        return (*a == params[0] && *b == params[1])
                            || (*a == params[1] && *b == params[0]);
                    }
                }
            }
        }
    }
    false
}

/// Rebuild an expression with `f` applied to each child (one level).
fn map_children(x: &Expr, f: &mut impl FnMut(&Expr) -> Expr) -> Expr {
    use Expr::*;
    match x {
        List(items) => List(items.iter().map(&mut *f).collect()),
        Tuple(items) => Tuple(items.iter().map(&mut *f).collect()),
        RecordLit { name, fields } => RecordLit {
            name: name.clone(),
            fields: fields.iter().map(|(n, v)| (n.clone(), f(v))).collect(),
        },
        MapLit(entries) => MapLit(entries.iter().map(|(k, v)| (k.clone(), f(v))).collect()),
        App { head, args } => App { head: Box::new(f(head)), args: args.iter().map(&mut *f).collect() },
        Lambda { params, body } => Lambda { params: params.clone(), body: Box::new(f(body)) },
        Field { recv, name } => Field { recv: Box::new(f(recv)), name: name.clone() },
        Index { recv, index } => Index { recv: Box::new(f(recv)), index: Box::new(f(index)) },
        Slice { recv, lo, hi } => Slice {
            recv: Box::new(f(recv)),
            lo: lo.as_ref().map(|e| Box::new(f(e))),
            hi: hi.as_ref().map(|e| Box::new(f(e))),
        },
        Unary { op, expr } => Unary { op: op.clone(), expr: Box::new(f(expr)) },
        Binary { op, lhs, rhs } => {
            Binary { op: op.clone(), lhs: Box::new(f(lhs)), rhs: Box::new(f(rhs)) }
        }
        Pipe { stages } => Pipe { stages: stages.iter().map(&mut *f).collect() },
        Paren(inner) => Paren(Box::new(f(inner))),
        Rescue { value, fallback } => {
            Rescue { value: Box::new(f(value)), fallback: Box::new(f(fallback)) }
        }
        Propagate(inner) => Propagate(Box::new(f(inner))),
        If { cond, then, els } => If {
            cond: Box::new(f(cond)),
            then: then.clone(),
            els: els.as_ref().map(|e| Box::new(f(e))),
        },
        Match { subject, arms } => Match {
            subject: Box::new(f(subject)),
            arms: arms.iter().map(|(p, b)| (p.clone(), f(b))).collect(),
        },
        atom => atom.clone(),
    }
}

// ---- compact source rendering (lint-local; the gate verifies output) ----

fn atomic(e: &Expr) -> bool {
    use Expr::*;
    matches!(
        e,
        Num(_) | Str(_) | Bool(_) | Unit | Name(_) | TName(_) | Proj(_) | List(_) | Tuple(_)
            | Paren(_) | Field { .. } | Index { .. } | Slice { .. } | RecordLit { .. }
            | MapLit(_) | Propagate(_)
    )
}

fn wrap(e: &Expr) -> String {
    if atomic(e) {
        render(e)
    } else {
        format!("({})", render(e))
    }
}

pub fn render(e: &Expr) -> String {
    use Expr::*;
    match e {
        Num(n) => n.clone(),
        Str(s) => s.clone(),
        Bool(b) => b.to_string(),
        Unit => "()".into(),
        Name(n) | TName(n) => n.clone(),
        Proj(p) => format!(".{p}"),
        List(items) => format!("[{}]", items.iter().map(render).collect::<Vec<_>>().join(", ")),
        Tuple(items) => format!("({})", items.iter().map(render).collect::<Vec<_>>().join(", ")),
        RecordLit { name, fields } => format!(
            "{}{{{}}}",
            name.clone().unwrap_or_default(),
            fields.iter().map(|(n, v)| format!("{n}: {}", render(v))).collect::<Vec<_>>().join(", ")
        ),
        MapLit(entries) => format!(
            "{{{}}}",
            entries.iter().map(|(k, v)| format!("{k}: {}", render(v))).collect::<Vec<_>>().join(", ")
        ),
        Block(stmts) => format!(
            "{{ {} }}",
            stmts.iter().map(render_stmt).collect::<Vec<_>>().join("; ")
        ),
        App { head, args } => {
            let mut parts = vec![wrap(head)];
            parts.extend(args.iter().map(wrap));
            parts.join(" ")
        }
        Lambda { params, body } => format!("{} -> {}", params.join(" "), render(body)),
        Field { recv, name } => format!("{}.{name}", wrap(recv)),
        Index { recv, index } => format!("{}[{}]", wrap(recv), render(index)),
        Slice { recv, lo, hi } => format!(
            "{}[{}:{}]",
            wrap(recv),
            lo.as_ref().map(|e| render(e)).unwrap_or_default(),
            hi.as_ref().map(|e| render(e)).unwrap_or_default()
        ),
        Unary { op, expr } => {
            if op == "-" {
                format!("-{}", wrap(expr))
            } else {
                format!("{op} {}", wrap(expr))
            }
        }
        Binary { op, lhs, rhs } => format!("{} {op} {}", wrap(lhs), wrap(rhs)),
        Pipe { stages } => stages.iter().map(render).collect::<Vec<_>>().join(" | "),
        Paren(inner) => format!("({})", render(inner)),
        Rescue { value, fallback } => format!("{} ? {}", wrap(value), wrap(fallback)),
        Propagate(inner) => format!("{}?", wrap(inner)),
        If { cond, then, els } => {
            let head = format!(
                "if {} {{ {} }}",
                render(cond),
                then.iter().map(render_stmt).collect::<Vec<_>>().join("; ")
            );
            match els {
                // `else if ...` chains and already-braced blocks render bare
                Some(e) => match e.as_ref() {
                    If { .. } | Block(_) => format!("{head} else {}", render(e)),
                    _ => format!("{head} else {{ {} }}", render(e)),
                },
                None => head,
            }
        }
        Match { subject, arms } => format!(
            "match {} {{ {} }}",
            wrap(subject),
            arms.iter()
                .map(|(p, b)| format!("{} -> {}", render_pat(p), render(b)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_pat(p: &Pattern) -> String {
    match p {
        Pattern::TypeBind { ty, name } => format!("{ty} {name}"),
        Pattern::Lit(e) => render(e),
        Pattern::Tuple(names) => format!("({})", names.join(", ")),
        Pattern::Wildcard => "_".into(),
        Pattern::Bind(n) => n.clone(),
    }
}

fn render_ty(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Named(n) => n.clone(),
        TypeExpr::Union(parts) => parts.iter().map(render_ty).collect::<Vec<_>>().join(" | "),
        TypeExpr::Record(fields) => format!(
            "{{{}}}",
            fields.iter().map(|(n, t)| format!("{n} {}", render_ty(t))).collect::<Vec<_>>().join(", ")
        ),
        TypeExpr::List(inner) => format!("[{}]", render_ty(inner)),
        TypeExpr::Fn { params, ret } => format!(
            "({} -> {})",
            params.iter().map(render_ty).collect::<Vec<_>>().join(" "),
            render_ty(ret)
        ),
    }
}

fn render_target(t: &Target) -> String {
    let mut out = t.name.clone();
    for ix in &t.indices {
        out.push_str(&format!("[{}]", render(ix)));
    }
    out
}

pub fn render_stmt(s: &Stmt) -> String {
    match s {
        Stmt::TypeDecl { name, ty } => format!("type {name} = {}", render_ty(ty)),
        Stmt::Sig { public, name, params, ret } => format!(
            "{}{name} :: {}{}",
            if *public { "pub " } else { "" },
            params.iter().map(render_ty).collect::<Vec<_>>().join(" "),
            ret.as_ref().map(|r| format!(" -> {}", render_ty(r))).unwrap_or_default()
        ),
        Stmt::Equation { name, params, body } => {
            let head = if params.is_empty() {
                name.clone()
            } else {
                format!("{name} {}", params.join(" "))
            };
            match body {
                Body::Expr(e) => format!("{head} = {}", render(e)),
                Body::Block(stmts) => format!(
                    "{head} = {{ {} }}",
                    stmts.iter().map(render_stmt).collect::<Vec<_>>().join("; ")
                ),
                Body::Stmt(st) => format!("{head} = {}", render_stmt(st)),
            }
        }
        Stmt::Destructure { names, value } => {
            format!("({}) = {}", names.join(", "), render(value))
        }
        Stmt::Binding { target, ann, value } => format!(
            "{}{} = {}",
            render_target(target),
            ann.as_ref().map(|t| format!(": {}", render_ty(t))).unwrap_or_default(),
            render(value)
        ),
        Stmt::Compound { target, op, value } => {
            format!("{} {op} {}", render_target(target), render(value))
        }
        Stmt::For { pat, iter, body } => format!(
            "for {} in {} {{ {} }}",
            if pat.len() == 1 { pat[0].clone() } else { format!("({})", pat.join(", ")) },
            render(iter),
            body.iter().map(render_stmt).collect::<Vec<_>>().join("; ")
        ),
        Stmt::While { cond, body } => format!(
            "while {} {{ {} }}",
            render(cond),
            body.iter().map(render_stmt).collect::<Vec<_>>().join("; ")
        ),
        Stmt::Ret(v) => match v {
            Some(e) => format!("ret {}", render(e)),
            None => "ret".into(),
        },
        Stmt::Go(e) => format!("go {}", render(e)),
        Stmt::Expr(e) => render(e),
    }
}
