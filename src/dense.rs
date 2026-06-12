//! `curt dense` — the verifier-backed idiom densifier (idiom-density chunk).
//!
//! Greedy loop-shape → verb-shape rewrites. Soundness comes from the
//! differential-execution gate, not static analysis: a candidate rewrite is
//! accepted only if (a) the rewritten program's stdout is byte-identical to
//! the original's and (b) the o200k token count strictly decreases. When no
//! rewrite is accepted, dense is the identity. Programs touching
//! capabilities (`fs`/`net`), spawning (`go`) or reading `args` are refused
//! (the gate requires deterministic, pure runs).

use crate::ast::*;
use crate::diag::Diag;
use std::io::Write as _;
use std::process::{Command, Stdio};
#[cfg(feature = "tokens")]
use std::sync::OnceLock;

pub fn dense(src: &str) -> Result<String, Diag> {
    let prog = crate::parse_source(src)?;
    if !pure(src) {
        return Ok(src.to_string());
    }
    let Some(baseline) = run_capture(src) else {
        return Ok(src.to_string()); // original doesn't run clean — leave it
    };
    let mut cur_prog = prog;
    let mut cur_src = src.to_string();
    let mut budget = tokens(&cur_src);
    for _ in 0..16 {
        let mut accepted = false;
        for cand in candidates(&cur_prog) {
            let new_prog = apply(&cur_prog, &cand);
            let new_src = render_program(&new_prog);
            if tokens(&new_src) >= budget {
                continue;
            }
            if run_capture(&new_src).as_deref() == Some(baseline.as_str()) {
                cur_prog = new_prog;
                cur_src = new_src;
                budget = tokens(&cur_src);
                accepted = true;
                break;
            }
        }
        if !accepted {
            break;
        }
    }
    Ok(cur_src)
}

fn pure(src: &str) -> bool {
    !(src.contains("fs.") || src.contains("net.") || src.contains("go ") || src.contains("args"))
}

#[cfg(feature = "tokens")]
fn tokens(s: &str) -> usize {
    static BPE: OnceLock<Option<tiktoken_rs::CoreBPE>> = OnceLock::new();
    match BPE.get_or_init(|| tiktoken_rs::o200k_base().ok()) {
        Some(bpe) => bpe.encode_ordinary(s).len(),
        None => s.len(), // degraded but monotone proxy
    }
}

/// no-tokens build: the documented degraded-but-monotone proxy, statically
#[cfg(not(feature = "tokens"))]
fn tokens(s: &str) -> usize {
    s.len()
}

/// Run a program through our own binary, capturing stdout. None on failure.
fn run_capture(src: &str) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let mut child = Command::new(exe)
        .arg("run")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.as_mut()?.write_all(src.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

// ---- candidate rewrites -------------------------------------------------

/// A rewrite site: a statement-list path + index, replaced by one binding.
struct Cand {
    /// path of block indices from the top (empty = toplevel stmts)
    path: Vec<usize>,
    /// index of the init binding; the For at index+1 is consumed too
    at: usize,
    replacement: Stmt,
}

fn candidates(prog: &[Stmt]) -> Vec<Cand> {
    let mut out = Vec::new();
    walk(prog, &mut Vec::new(), &mut out);
    out
}

fn walk(stmts: &[Stmt], path: &mut Vec<usize>, out: &mut Vec<Cand>) {
    for i in 0..stmts.len() {
        if i + 1 < stmts.len() {
            if let Some(rep) = match_pair(&stmts[i], &stmts[i + 1]) {
                out.push(Cand { path: path.clone(), at: i, replacement: rep });
            }
        }
        // recurse into nested blocks
        match &stmts[i] {
            Stmt::Equation { body: Body::Block(b), .. } | Stmt::For { body: b, .. } | Stmt::While { body: b, .. } => {
                path.push(i);
                walk(b, path, out);
                path.pop();
            }
            _ => {}
        }
    }
}

/// init-binding + for-accumulate → pipeline binding.
fn match_pair(a: &Stmt, b: &Stmt) -> Option<Stmt> {
    let Stmt::Binding { target, ann: None, value: init } = a else { return None };
    if !target.indices.is_empty() {
        return None;
    }
    let Stmt::For { pat, iter, body } = b else { return None };
    let [var] = pat.as_slice() else { return None };
    let acc = &target.name;

    // unwrap an optional single `if cond { inner }` statement
    let (cond, inner): (Option<&Expr>, &Stmt) = match body.as_slice() {
        [Stmt::Expr(Expr::If { cond, then, els: None })] => match then.as_slice() {
            [s] => (Some(cond), s),
            _ => return None,
        },
        [s] => (None, s),
        _ => return None,
    };
    // R4: max/min scan — `best = xs[0]` is not matched (init differs), but
    // `if x > best { best = x }` with init 0/huge appears as a Binding:
    if let (Some(c), Stmt::Binding { target: t2, ann: None, value }) = (cond, inner) {
        if t2.name == *acc && t2.indices.is_empty() {
            if let (Expr::Binary { op, lhs, rhs }, Expr::Name(v)) = (c, value) {
                if v == var {
                    let verb = match op.as_str() {
                        ">" => Some("max"),
                        "<" => Some("min"),
                        _ => None,
                    };
                    if let (Some(verb), Expr::Name(l), Expr::Name(r)) = (verb, &**lhs, &**rhs) {
                        if l == var && r == acc {
                            return Some(Stmt::Binding {
                                target: Target { name: acc.clone(), indices: vec![] },
                                ann: None,
                                value: Expr::Field { recv: Box::new(pipe_head(iter.clone())), name: verb.into() },
                            });
                        }
                    }
                }
            }
        }
    }
    let Stmt::Compound { target: t2, op, value: step } = inner else { return None };
    if op != "+=" || t2.name != *acc || !t2.indices.is_empty() {
        return None;
    }
    // the step must not reference the accumulator itself
    if mentions(step, acc) {
        return None;
    }

    let stage0 = pipe_head(iter.clone());
    let keep_stage = cond.map(|c| stage("keep", lambda(var, c.clone())));

    let pipeline: Expr = match init {
        // numeric accumulators
        Expr::Num(n) if n == "0" || n == "0.0" => {
            if matches!(step, Expr::Num(k) if k == "1") {
                // pure counting: (xs | keep (v -> c)).len
                let mut stages = vec![stage0];
                if let Some(k) = keep_stage {
                    stages.push(k);
                }
                let base = if stages.len() == 1 { stages.remove(0) } else { Expr::Pipe { stages } };
                Expr::Field { recv: Box::new(Expr::Paren(Box::new(base))), name: "len".into() }
            } else {
                let mut stages = vec![stage0];
                if let Some(k) = keep_stage {
                    stages.push(k);
                }
                if !matches!(step, Expr::Name(n) if n == var) {
                    stages.push(stage("map", lambda(var, step.clone())));
                }
                stages.push(Expr::Name("sum".into()));
                Expr::Pipe { stages }
            }
        }
        // list accumulators: out += [E]
        Expr::List(items) if items.is_empty() => {
            let Expr::List(elems) = step else { return None };
            let [elem] = elems.as_slice() else { return None };
            let mut stages = vec![stage0];
            if let Some(k) = keep_stage {
                stages.push(k);
            }
            if !matches!(elem, Expr::Name(n) if n == var) {
                stages.push(stage("map", lambda(var, elem.clone())));
            }
            if stages.len() == 1 {
                return None; // identity copy, not worth a rewrite
            }
            Expr::Pipe { stages }
        }
        // string accumulators: s = s-concat via map|join ""
        Expr::Str(s) if s == "\"\"" => {
            let Expr::Binary { op, lhs: _, rhs: _ } = step else { return None };
            if op != "+" {
                return None;
            }
            // out += E  desugars Compound, step is E itself: join the mapped pieces
            let mut stages = vec![stage0];
            if let Some(k) = keep_stage {
                stages.push(k);
            }
            stages.push(stage("map", lambda(var, step.clone())));
            stages.push(stage_str("join", ""));
            Expr::Pipe { stages }
        }
        _ => return None,
    };

    Some(Stmt::Binding {
        target: Target { name: acc.clone(), indices: vec![] },
        ann: None,
        value: pipeline,
    })
}

fn mentions(e: &Expr, name: &str) -> bool {
    render_expr(e).split(|c: char| !c.is_alphanumeric() && c != '_').any(|w| w == name)
}

fn pipe_head(iter: Expr) -> Expr {
    match iter {
        Expr::App { .. } => Expr::Paren(Box::new(iter)),
        other => other,
    }
}

fn lambda(var: &str, body: Expr) -> Expr {
    Expr::Paren(Box::new(Expr::Lambda { params: vec![var.to_string()], body: Box::new(body) }))
}

fn stage(verb: &str, arg: Expr) -> Expr {
    Expr::App { head: Box::new(Expr::Name(verb.into())), args: vec![arg] }
}

fn stage_str(verb: &str, lit: &str) -> Expr {
    Expr::App { head: Box::new(Expr::Name(verb.into())), args: vec![Expr::Str(format!("\"{lit}\""))] }
}

fn apply(prog: &[Stmt], cand: &Cand) -> Vec<Stmt> {
    fn go(stmts: &[Stmt], path: &[usize], at: usize, rep: &Stmt) -> Vec<Stmt> {
        if path.is_empty() {
            let mut out = stmts.to_vec();
            out.splice(at..at + 2, [rep.clone()]);
            return out;
        }
        let (head, rest) = (path[0], &path[1..]);
        stmts
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i != head {
                    return s.clone();
                }
                match s {
                    Stmt::Equation { name, params, body: Body::Block(b) } => Stmt::Equation {
                        name: name.clone(),
                        params: params.clone(),
                        body: Body::Block(go(b, rest, at, rep)),
                    },
                    Stmt::For { pat, iter, body } => {
                        Stmt::For { pat: pat.clone(), iter: iter.clone(), body: go(body, rest, at, rep) }
                    }
                    Stmt::While { cond, body } => Stmt::While { cond: cond.clone(), body: go(body, rest, at, rep) },
                    other => other.clone(),
                }
            })
            .collect()
    }
    go(prog, &cand.path, cand.at, &cand.replacement)
}

// ---- minimal renderer ---------------------------------------------------
// Conservative parens; the differential gate rejects any infidelity.

fn render_program(prog: &[Stmt]) -> String {
    let mut out = String::new();
    for s in prog {
        render_stmt(s, 0, &mut out);
        out.push('\n');
    }
    out
}

fn ind(n: usize, out: &mut String) {
    out.push_str(&"  ".repeat(n));
}

fn render_stmt(s: &Stmt, d: usize, out: &mut String) {
    ind(d, out);
    match s {
        Stmt::TypeDecl { name, ty } => out.push_str(&format!("type {name} = {}", render_type(ty))),
        Stmt::Sig { public, name, params, ret } => {
            if *public {
                out.push_str("pub ");
            }
            out.push_str(&format!("{name} :: "));
            out.push_str(&params.iter().map(render_type).collect::<Vec<_>>().join(" "));
            if let Some(r) = ret {
                out.push_str(&format!(" -> {}", render_type(r)));
            }
        }
        Stmt::Equation { name, params, body } => {
            out.push_str(&format!("{name} {} =", params.join(" ")));
            match body {
                Body::Expr(e) => {
                    out.push(' ');
                    out.push_str(&render_expr(e));
                }
                Body::Block(b) => {
                    out.push_str(" {\n");
                    for st in b {
                        render_stmt(st, d + 1, out);
                        out.push('\n');
                    }
                    ind(d, out);
                    out.push('}');
                }
                Body::Stmt(st) => {
                    out.push(' ');
                    let mut tmp = String::new();
                    render_stmt(st, 0, &mut tmp);
                    out.push_str(&tmp);
                }
            }
        }
        Stmt::Destructure { names, value } => {
            out.push_str(&format!("({}) = {}", names.join(", "), render_expr(value)));
        }
        Stmt::Binding { target, ann, value } => {
            out.push_str(&render_target(target));
            if let Some(t) = ann {
                out.push_str(&format!(": {}", render_type(t)));
            }
            out.push_str(&format!(" = {}", render_expr(value)));
        }
        Stmt::Compound { target, op, value } => {
            out.push_str(&format!("{} {op} {}", render_target(target), render_expr(value)));
        }
        Stmt::For { pat, iter, body } => {
            let p = if pat.len() == 1 { pat[0].clone() } else { format!("({})", pat.join(", ")) };
            out.push_str(&format!("for {p} in {} {{\n", render_expr(iter)));
            for st in body {
                render_stmt(st, d + 1, out);
                out.push('\n');
            }
            ind(d, out);
            out.push('}');
        }
        Stmt::While { cond, body } => {
            out.push_str(&format!("while {} {{\n", render_expr(cond)));
            for st in body {
                render_stmt(st, d + 1, out);
                out.push('\n');
            }
            ind(d, out);
            out.push('}');
        }
        Stmt::Ret(e) => {
            out.push_str("ret");
            if let Some(e) = e {
                out.push(' ');
                out.push_str(&render_expr(e));
            }
        }
        Stmt::Go(e) => out.push_str(&format!("go {}", render_expr(e))),
        Stmt::Expr(e) => out.push_str(&render_stmt_expr(e, d)),
    }
}

/// Statement-position expressions: if/match render with blocks, not parens.
fn render_stmt_expr(e: &Expr, d: usize) -> String {
    match e {
        Expr::If { .. } | Expr::Match { .. } => render_block_expr(e, d),
        _ => render_expr(e),
    }
}

fn render_block_expr(e: &Expr, d: usize) -> String {
    match e {
        Expr::If { cond, then, els } => {
            let mut s = format!("if {} {{\n", render_expr(cond));
            for st in then {
                render_stmt(st, d + 1, &mut s);
                s.push('\n');
            }
            s.push_str(&"  ".repeat(d));
            s.push('}');
            if let Some(els) = els {
                s.push_str(" else ");
                match &**els {
                    Expr::If { .. } => s.push_str(&render_block_expr(els, d)),
                    Expr::Block(b) => {
                        s.push_str("{\n");
                        for st in b {
                            render_stmt(st, d + 1, &mut s);
                            s.push('\n');
                        }
                        s.push_str(&"  ".repeat(d));
                        s.push('}');
                    }
                    other => {
                        s.push_str("{ ");
                        s.push_str(&render_expr(other));
                        s.push_str(" }");
                    }
                }
            }
            s
        }
        Expr::Match { subject, arms } => {
            let mut s = format!("match {} {{ ", render_expr(subject));
            let rendered: Vec<String> =
                arms.iter().map(|(p, b)| format!("{} -> {}", render_pattern(p), render_expr(b))).collect();
            s.push_str(&rendered.join(", "));
            s.push_str(" }");
            s
        }
        other => render_expr(other),
    }
}

fn render_pattern(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard => "_".into(),
        Pattern::Bind(n) => n.clone(),
        Pattern::TypeBind { ty, name } => format!("{ty} {name}"),
        Pattern::Lit(e) => render_expr(e),
        Pattern::Tuple(names) => format!("({})", names.join(", ")),
    }
}

fn render_target(t: &Target) -> String {
    let mut s = t.name.clone();
    for ix in &t.indices {
        s.push_str(&format!("[{}]", render_expr(ix)));
    }
    s
}

fn atom(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Num(_)
            | Expr::Str(_)
            | Expr::Bool(_)
            | Expr::Unit
            | Expr::Name(_)
            | Expr::TName(_)
            | Expr::Proj(_)
            | Expr::List(_)
            | Expr::Tuple(_)
            | Expr::RecordLit { .. }
            | Expr::Paren(_)
            | Expr::Field { .. }
            | Expr::Index { .. }
            | Expr::Slice { .. }
            | Expr::Propagate(_)
    )
}

fn wrap(e: &Expr) -> String {
    if atom(e) {
        render_expr(e)
    } else {
        format!("({})", render_expr(e))
    }
}

fn render_expr(e: &Expr) -> String {
    match e {
        Expr::Num(n) => n.clone(),
        Expr::Str(s) => s.clone(),
        Expr::Bool(b) => b.to_string(),
        Expr::Unit => "()".into(),
        Expr::Name(n) => n.clone(),
        Expr::TName(n) => n.clone(),
        Expr::Proj(p) => format!(".{p}"),
        Expr::List(items) => format!("[{}]", items.iter().map(render_expr).collect::<Vec<_>>().join(", ")),
        Expr::Tuple(items) => format!("({})", items.iter().map(render_expr).collect::<Vec<_>>().join(", ")),
        Expr::RecordLit { name, fields } => {
            let fs = fields.iter().map(|(n, v)| format!("{n}:{}", render_expr(v))).collect::<Vec<_>>().join(", ");
            format!("{}{{{fs}}}", name.clone().unwrap_or_default())
        }
        Expr::MapLit(entries) => {
            let es = entries.iter().map(|(k, v)| format!("{k}: {}", render_expr(v))).collect::<Vec<_>>().join(", ");
            format!("{{{es}}}")
        }
        Expr::Block(b) => {
            let mut s = String::from("{ ");
            let parts: Vec<String> = b
                .iter()
                .map(|st| {
                    let mut t = String::new();
                    render_stmt(st, 0, &mut t);
                    t
                })
                .collect();
            s.push_str(&parts.join("; "));
            s.push_str(" }");
            s
        }
        Expr::App { head, args } => {
            let h = match &**head {
                Expr::Name(_) | Expr::TName(_) | Expr::Field { .. } | Expr::Paren(_) => render_expr(head),
                other => format!("({})", render_expr(other)),
            };
            let mut s = h;
            for a in args {
                s.push(' ');
                s.push_str(&wrap(a));
            }
            s
        }
        Expr::Lambda { params, body } => {
            // a rendered lambda body must not leak into an enclosing pipe —
            // bodies stop at `|` since v0.2, so parenthesize pipe bodies
            let b = match &**body {
                Expr::Pipe { .. } => format!("({})", render_expr(body)),
                _ => render_expr(body),
            };
            format!("{} -> {}", params.join(" "), b)
        }
        Expr::Field { recv, name } => format!("{}.{name}", wrap_recv(recv)),
        Expr::Index { recv, index } => format!("{}[{}]", wrap_recv(recv), render_expr(index)),
        Expr::Slice { recv, lo, hi } => format!(
            "{}[{}:{}]",
            wrap_recv(recv),
            lo.as_ref().map(|e| render_expr(e)).unwrap_or_default(),
            hi.as_ref().map(|e| render_expr(e)).unwrap_or_default()
        ),
        Expr::Unary { op, expr } => format!("{op}{}", wrap(expr)),
        Expr::Binary { op, lhs, rhs } => {
            format!("{} {op} {}", wrap_operand(lhs), wrap_operand(rhs))
        }
        Expr::Pipe { stages } => stages.iter().map(render_stage).collect::<Vec<_>>().join(" | "),
        Expr::Paren(inner) => format!("({})", render_expr(inner)),
        Expr::Rescue { value, fallback } => format!("{} ? {}", wrap_operand(value), wrap_operand(fallback)),
        Expr::Propagate(inner) => format!("{}?", wrap_recv(inner)),
        Expr::If { .. } | Expr::Match { .. } => render_block_expr(e, 0),
    }
}

fn render_stage(e: &Expr) -> String {
    match e {
        // verbs with args render plainly: `map (x -> e)`, `join ""`
        Expr::App { head, args } => {
            let mut s = render_expr(head);
            for a in args {
                s.push(' ');
                s.push_str(&wrap(a));
            }
            s
        }
        other => render_expr(other),
    }
}

fn wrap_recv(e: &Expr) -> String {
    match e {
        Expr::Name(_) | Expr::TName(_) | Expr::Field { .. } | Expr::Index { .. } | Expr::Paren(_) | Expr::Str(_) => {
            render_expr(e)
        }
        Expr::Num(n) if !n.contains('.') => format!("({n})"),
        other if atom(other) => render_expr(other),
        other => format!("({})", render_expr(other)),
    }
}

fn wrap_operand(e: &Expr) -> String {
    match e {
        Expr::Binary { .. } | Expr::Pipe { .. } | Expr::Rescue { .. } | Expr::Lambda { .. } | Expr::If { .. } | Expr::Match { .. } => {
            format!("({})", render_expr(e))
        }
        _ => render_expr(e),
    }
}

fn render_type(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Named(n) => n.clone(),
        TypeExpr::Union(parts) => parts.iter().map(render_type).collect::<Vec<_>>().join(" | "),
        TypeExpr::Record(fields) => {
            let fs = fields.iter().map(|(n, t)| format!("{n} {}", render_type(t))).collect::<Vec<_>>().join(", ");
            format!("{{{fs}}}")
        }
        TypeExpr::List(inner) => format!("[{}]", render_type(inner)),
        TypeExpr::Fn { params, ret } => format!(
            "({} -> {})",
            params.iter().map(render_type).collect::<Vec<_>>().join(" "),
            render_type(ret)
        ),
    }
}
