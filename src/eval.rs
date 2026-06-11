//! Tree-walk evaluator (interp-d) — `curt run`.
//!
//! Mirrors the checker's elaboration rules exactly (SPEC §2.3): flat
//! applications re-nest surplus args under the k-th argument; pipelines
//! capture the preceding juxtaposition's last argument; pipe stages apply
//! with the piped value appended; UFCS resolves receiver-last.
//!
//! Error model (SPEC §7): `err` is a VALUE and it is contagious — any
//! operation touching an err yields an err. Spaced rescue `x ? y` replaces
//! an err/absent with y; glued `x?` propagates out of the enclosing
//! equation. String literals interpolate `{expr}` at evaluation time.
//!
//! Capabilities: fs/net DENIED unless enabled; a denied or failed host op
//! yields an err value (rescuable), never a crash. `go` is a sequential
//! hint in v0.1 (honest limitation; true threads need Send values).

use crate::ast::*;
use crate::infer::rewrite_pipes;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, Write as IoWrite};
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    UInt(u64),
    Float(f64),
    Str(Rc<String>),
    Bool(bool),
    Unit,
    List(Rc<RefCell<Vec<Value>>>),
    /// insertion-ordered association vec (golden-stdout determinism)
    Map(Rc<RefCell<Vec<(Value, Value)>>>),
    Record(Rc<RefCell<Vec<(String, Value)>>>),
    Tuple(Rc<Vec<Value>>),
    Fn(Rc<Closure>),
    Builtin(&'static str),
    /// a namespace object: fs, net (fields resolve to builtins)
    Ns(&'static str),
    Conn(Rc<RefCell<std::net::TcpStream>>),
    Listener(Rc<std::net::TcpListener>),
    Err(Rc<String>),
}

pub struct Closure {
    pub params: Vec<String>,
    pub body: Body,
    pub env: Env,
}

pub enum Flow {
    Ret(Value),
    Fail(String), // runtime diagnostic (uncatchable in v0.1)
}

type R<T> = Result<T, Flow>;

#[derive(Clone)]
pub struct Env(Rc<RefCell<Scope>>);

pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

impl Env {
    fn new() -> Env {
        Env(Rc::new(RefCell::new(Scope { vars: HashMap::new(), parent: None })))
    }
    fn child(&self) -> Env {
        Env(Rc::new(RefCell::new(Scope { vars: HashMap::new(), parent: Some(self.clone()) })))
    }
    fn get(&self, name: &str) -> Option<Value> {
        let s = self.0.borrow();
        if let Some(v) = s.vars.get(name) {
            return Some(v.clone());
        }
        s.parent.as_ref().and_then(|p| p.get(name))
    }
    fn define(&self, name: &str, v: Value) {
        self.0.borrow_mut().vars.insert(name.to_string(), v);
    }
    /// assign where the name lives (mutation), else define here
    fn assign(&self, name: &str, v: Value) {
        if self.try_assign(name, &v) {
            return;
        }
        self.define(name, v);
    }
    fn try_assign(&self, name: &str, v: &Value) -> bool {
        let mut s = self.0.borrow_mut();
        if let Some(slot) = s.vars.get_mut(name) {
            *slot = v.clone();
            return true;
        }
        let parent = s.parent.clone();
        drop(s);
        parent.map(|p| p.try_assign(name, v)).unwrap_or(false)
    }
}

pub struct Caps {
    pub fs: bool,
    pub net: bool,
}

pub struct Interp {
    pub caps: Caps,
    pub args: Vec<String>,
}

fn err(msg: impl Into<String>) -> Value {
    Value::Err(Rc::new(msg.into()))
}

fn fail(msg: impl Into<String>) -> Flow {
    Flow::Fail(msg.into())
}

impl Interp {
    pub fn run(prog: &[Stmt], caps: Caps, args: Vec<String>) -> Result<(), String> {
        let it = Interp { caps, args };
        let env = Env::new();
        // pass 1: equations become closures (lexical, recursive via env)
        for s in prog {
            if let Stmt::Equation { name, params, body } = s {
                env.define(
                    name,
                    Value::Fn(Rc::new(Closure { params: params.clone(), body: body.clone(), env: env.clone() })),
                );
            }
        }
        for s in prog {
            match it.stmt(s, &env) {
                Ok(()) => {}
                Err(Flow::Ret(_)) => {}
                Err(Flow::Fail(m)) => return Err(m),
            }
        }
        Ok(())
    }

    fn stmt(&self, s: &Stmt, env: &Env) -> R<()> {
        match s {
            Stmt::TypeDecl { .. } | Stmt::Sig { .. } | Stmt::Equation { .. } => Ok(()),
            Stmt::Destructure { names, value } => {
                let v = self.expr(&rewrite_pipes(value), env)?;
                let parts = match &v {
                    Value::Tuple(p) if p.len() == names.len() => p.clone(),
                    other => return Err(fail(format!("cannot destructure {} into {} names", show(other), names.len()))),
                };
                for (n, p) in names.iter().zip(parts.iter()) {
                    env.assign(n, p.clone());
                }
                Ok(())
            }
            Stmt::Binding { target, value, .. } => {
                let v = self.expr(&rewrite_pipes(value), env)?;
                if target.indices.is_empty() {
                    env.assign(&target.name, v);
                } else {
                    self.index_assign(target, v, env)?;
                }
                Ok(())
            }
            Stmt::Compound { target, op, value } => {
                let rhs = self.expr(&rewrite_pipes(value), env)?;
                let cur = if target.indices.is_empty() {
                    env.get(&target.name).ok_or_else(|| fail(format!("`{}` is not defined", target.name)))?
                } else {
                    self.read_target(target, env)?
                };
                let op = &op[..1]; // "+=" -> "+"
                let next = binop(op, &cur, &rhs)?;
                if target.indices.is_empty() {
                    env.assign(&target.name, next);
                } else {
                    self.index_assign(target, next, env)?;
                }
                Ok(())
            }
            Stmt::For { pat, iter, body } => {
                let it_v = self.expr(&rewrite_pipes(iter), env)?;
                // a listener is an infinite accept loop (sequential v0.1):
                // each accepted connection runs the body, then accept again
                if let Value::Listener(l) = &it_v {
                    loop {
                        let Ok((stream, _)) = l.accept() else { break };
                        let scope = env.child();
                        scope.define(&pat[0], Value::Conn(Rc::new(RefCell::new(stream))));
                        for st in body {
                            self.stmt(st, &scope)?;
                        }
                    }
                    return Ok(());
                }
                let items = self.iterate(&it_v)?;
                for item in items {
                    let scope = env.child();
                    if pat.len() == 1 {
                        scope.define(&pat[0], item);
                    } else if let Value::Tuple(parts) = &item {
                        for (n, p) in pat.iter().zip(parts.iter()) {
                            scope.define(n, p.clone());
                        }
                    } else {
                        return Err(fail("for pattern needs a tuple"));
                    }
                    for st in body {
                        self.stmt(st, &scope)?;
                    }
                }
                Ok(())
            }
            Stmt::While { cond, body } => {
                loop {
                    let c = self.expr(&rewrite_pipes(cond), env)?;
                    if !truthy(&c)? {
                        break;
                    }
                    let scope = env.child();
                    for st in body {
                        self.stmt(st, &scope)?;
                    }
                }
                Ok(())
            }
            Stmt::Ret(e) => {
                let v = match e {
                    Some(e) => self.expr(&rewrite_pipes(e), env)?,
                    None => Value::Unit,
                };
                Err(Flow::Ret(v))
            }
            Stmt::Go(e) => {
                // v0.1: sequential (honest limitation — see module docs)
                self.expr(&rewrite_pipes(e), env)?;
                Ok(())
            }
            Stmt::Expr(e) => {
                self.expr(&rewrite_pipes(e), env)?;
                Ok(())
            }
        }
    }

    fn read_target(&self, t: &Target, env: &Env) -> R<Value> {
        let mut v = env.get(&t.name).ok_or_else(|| fail(format!("`{}` is not defined", t.name)))?;
        for ix in &t.indices {
            let i = self.expr(ix, env)?;
            v = index(&v, &i)?;
        }
        Ok(v)
    }

    fn index_assign(&self, t: &Target, v: Value, env: &Env) -> R<()> {
        let base = env.get(&t.name).ok_or_else(|| fail(format!("`{}` is not defined", t.name)))?;
        let mut cur = base;
        for (n, ix) in t.indices.iter().enumerate() {
            let key = self.expr(ix, env)?;
            let last = n == t.indices.len() - 1;
            if last {
                match &cur {
                    Value::List(items) => {
                        let i = as_index(&key, items.borrow().len())?;
                        items.borrow_mut()[i] = v;
                        return Ok(());
                    }
                    Value::Map(pairs) => {
                        let mut ps = pairs.borrow_mut();
                        if let Some(slot) = ps.iter_mut().find(|(k, _)| eq(k, &key)) {
                            slot.1 = v;
                        } else {
                            ps.push((key, v));
                        }
                        return Ok(());
                    }
                    other => return Err(fail(format!("{} is not index-assignable", show(other)))),
                }
            }
            cur = index(&cur, &key)?;
        }
        Ok(())
    }

    fn block(&self, stmts: &[Stmt], env: &Env) -> R<Value> {
        let scope = env.child();
        let mut last = Value::Unit;
        for (i, st) in stmts.iter().enumerate() {
            if i == stmts.len() - 1 {
                if let Stmt::Expr(e) = st {
                    last = self.expr(&rewrite_pipes(e), &scope)?;
                    continue;
                }
            }
            self.stmt(st, &scope)?;
        }
        Ok(last)
    }

    pub fn expr(&self, e: &Expr, env: &Env) -> R<Value> {
        match e {
            Expr::Num(n) => Ok(parse_num(n)),
            Expr::Str(s) => {
                // the lexer keeps the quoted lexeme; values are the inner text
                let inner = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')).unwrap_or(s);
                self.interpolate(inner, env)
            }
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Unit => Ok(Value::Unit),
            Expr::Name(n) => match n.as_str() {
                // user bindings shadow capability namespaces (spec-truth:
                // `fs = ...` then `fs.max` must see the user's list)
                "fs" if env.get(n).is_none() => Ok(Value::Ns("fs")),
                "net" if env.get(n).is_none() => Ok(Value::Ns("net")),
                "args" if env.get(n).is_none() => Ok(Value::List(Rc::new(RefCell::new(
                    self.args.iter().map(|a| Value::Str(Rc::new(a.clone()))).collect(),
                )))),
                _ => env
                    .get(n)
                    .or_else(|| BUILTINS.contains(&n.as_str()).then(|| Value::Builtin(leak(n))))
                    .ok_or_else(|| fail(format!("`{n}` is not defined"))),
            },
            Expr::TName(n) => Err(fail(format!("type `{n}` is not a value"))),
            Expr::Proj(p) => {
                // projection atom = field-access lambda
                let body = Expr::Field { recv: Box::new(Expr::Name("x".into())), name: p.clone() };
                Ok(Value::Fn(Rc::new(Closure { params: vec!["x".into()], body: Body::Expr(body), env: env.clone() })))
            }
            Expr::List(items) => {
                let vs: R<Vec<Value>> = items.iter().map(|i| self.expr(i, env)).collect();
                Ok(Value::List(Rc::new(RefCell::new(vs?))))
            }
            Expr::Tuple(items) => {
                let vs: R<Vec<Value>> = items.iter().map(|i| self.expr(i, env)).collect();
                Ok(Value::Tuple(Rc::new(vs?)))
            }
            Expr::RecordLit { name, fields } => {
                if fields.is_empty() && name.is_none() {
                    return Ok(Value::Map(Rc::new(RefCell::new(Vec::new()))));
                }
                let mut out = Vec::new();
                for (n, v) in fields {
                    out.push((n.clone(), self.expr(v, env)?));
                }
                Ok(Value::Record(Rc::new(RefCell::new(out))))
            }
            Expr::Block(stmts) => self.block(stmts, env),
            Expr::App { head, args } => {
                let f = self.expr(head, env)?;
                let mut vals = Vec::new();
                for a in args {
                    vals.push((a, self.expr(a, env)?));
                }
                self.apply_flat(f, &vals, env)
            }
            Expr::Lambda { params, body } => Ok(Value::Fn(Rc::new(Closure {
                params: params.clone(),
                body: Body::Expr((**body).clone()),
                env: env.clone(),
            }))),
            Expr::Field { recv, name } => {
                let r = self.expr(recv, env)?;
                self.field(&r, name, env)
            }
            Expr::Index { recv, index: ix } => {
                let r = self.expr(recv, env)?;
                if let Value::Err(_) = r {
                    return Ok(r);
                }
                let i = self.expr(ix, env)?;
                index(&r, &i)
            }
            Expr::Slice { recv, lo, hi } => {
                let r = self.expr(recv, env)?;
                let lo = match lo {
                    Some(e) => Some(self.expr(e, env)?),
                    None => None,
                };
                let hi = match hi {
                    Some(e) => Some(self.expr(e, env)?),
                    None => None,
                };
                slice(&r, lo, hi)
            }
            Expr::Unary { op, expr } => {
                let v = self.expr(expr, env)?;
                match op.as_str() {
                    "not" => Ok(Value::Bool(!truthy(&v)?)),
                    "-" => binop("-", &Value::Int(0), &v),
                    other => Err(fail(format!("unknown unary {other}"))),
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                let l = self.expr(lhs, env)?;
                match op.as_str() {
                    "and" => return if truthy(&l)? { self.expr(rhs, env) } else { Ok(Value::Bool(false)) },
                    "or" => return if truthy(&l)? { Ok(Value::Bool(true)) } else { self.expr(rhs, env) },
                    _ => {}
                }
                let r = self.expr(rhs, env)?;
                binop(op, &l, &r)
            }
            Expr::Pipe { stages } => {
                let mut v = self.expr(&stages[0], env)?;
                for s in &stages[1..] {
                    v = self.apply_stage(s, v, env)?;
                }
                Ok(v)
            }
            Expr::Rescue { value, fallback } => {
                let v = self.expr(value, env)?;
                match v {
                    Value::Err(_) => self.expr(fallback, env),
                    other => Ok(other),
                }
            }
            Expr::Paren(inner) => self.expr(inner, env),
            Expr::Propagate(inner) => {
                let v = self.expr(inner, env)?;
                match v {
                    Value::Err(_) => Err(Flow::Ret(v)), // early-return the err
                    other => Ok(other),
                }
            }
            Expr::If { cond, then, els } => {
                let c = self.expr(cond, env)?;
                if truthy(&c)? {
                    self.block(then, env)
                } else if let Some(e) = els {
                    self.expr(e, env)
                } else {
                    Ok(Value::Unit)
                }
            }
            Expr::Match { subject, arms } => {
                let v = self.expr(subject, env)?;
                for (pat, body) in arms {
                    let scope = env.child();
                    if self.pattern_matches(pat, &v, &scope)? {
                        return self.expr(body, &scope);
                    }
                }
                Err(fail(format!("no match arm for {}", show(&v))))
            }
        }
    }

    fn pattern_matches(&self, p: &Pattern, v: &Value, scope: &Env) -> R<bool> {
        Ok(match p {
            Pattern::TypeBind { ty, name } => {
                // `err e` narrows the error case of `T | err` (domain-bench:
                // models wrote exactly this 5x; Zig's catch |err| precedent)
                if ty == "err" {
                    if let Value::Err(msg) = v {
                        scope.define(name, Value::Str(msg.clone()));
                        return Ok(true);
                    }
                    return Ok(false);
                }
                let hit = matches!(
                    (ty.as_str(), v),
                    ("float", Value::Float(_)) | ("float", Value::Int(_))
                        | ("int", Value::Int(_))
                        | ("str", Value::Str(_))
                        | ("bool", Value::Bool(_))
                );
                if hit {
                    scope.define(name, v.clone());
                }
                hit
            }
            Pattern::Lit(l) => {
                let lv = self.expr(l, scope)?;
                eq(&lv, v)
            }
            Pattern::Tuple(names) => match v {
                Value::Tuple(parts) if parts.len() == names.len() => {
                    for (n, part) in names.iter().zip(parts.iter()) {
                        scope.define(n, part.clone());
                    }
                    true
                }
                _ => false,
            },
            Pattern::Wildcard => true,
            Pattern::Bind(n) => {
                scope.define(n, v.clone());
                true
            }
        })
    }

    /// SPEC §2.3 arity re-nesting over already-evaluated (expr, value) args.
    fn apply_flat(&self, f: Value, args: &[(&Expr, Value)], env: &Env) -> R<Value> {
        if let Value::Err(_) = f {
            return Ok(f);
        }
        let k = self.arity(&f);
        match k {
            Some(k) if args.len() > k && k >= 1 => {
                let head_args: Vec<Value> = args[..k - 1].iter().map(|(_, v)| v.clone()).collect();
                let nested_head = args[k - 1].1.clone();
                let nested = self.apply_flat(nested_head, &args[k..], env)?;
                let mut all = head_args;
                all.push(nested);
                self.call(f, all, env)
            }
            _ => {
                let vals: Vec<Value> = args.iter().map(|(_, v)| v.clone()).collect();
                self.call(f, vals, env)
            }
        }
    }

    fn arity(&self, f: &Value) -> Option<usize> {
        match f {
            Value::Fn(c) => Some(c.params.len()),
            // range is 1-or-2 ary (SPEC §5); no fixed arity means no
            // surplus re-nesting, the builtin validates its own args
            Value::Builtin("range") => None,
            Value::Builtin(name) => builtin_arity(name),
            _ => None,
        }
    }

    fn apply_stage(&self, stage: &Expr, piped: Value, env: &Env) -> R<Value> {
        match stage {
            Expr::App { head, args } => {
                let f = self.expr(head, env)?;
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.expr(a, env)?);
                }
                vals.push(piped);
                self.call(f, vals, env)
            }
            other => {
                let f = self.expr(other, env)?;
                self.call(f, vec![piped], env)
            }
        }
    }

    fn call(&self, f: Value, args: Vec<Value>, env: &Env) -> R<Value> {
        // err contagion through arguments of builtins/calls
        if !matches!(f, Value::Fn(_)) {
            if let Some(e) = args.iter().find(|a| matches!(a, Value::Err(_))) {
                return Ok(e.clone());
            }
        }
        match f {
            Value::Fn(c) => {
                if args.len() != c.params.len() {
                    return Err(fail(format!("function expects {} args, got {}", c.params.len(), args.len())));
                }
                let scope = c.env.child();
                for (p, a) in c.params.iter().zip(args) {
                    scope.define(p, a);
                }
                let out = match &c.body {
                    Body::Expr(e) => self.expr(&rewrite_pipes(e), &scope),
                    Body::Block(stmts) => self.block(stmts, &scope),
                    Body::Stmt(st) => self.stmt(st, &scope).map(|_| Value::Unit),
                };
                match out {
                    Err(Flow::Ret(v)) => Ok(v),
                    other => other,
                }
            }
            Value::Builtin(name) => self.builtin(name, args, env),
            other => Err(fail(format!("{} is not callable", show(&other)))),
        }
    }

    fn field(&self, recv: &Value, name: &str, env: &Env) -> R<Value> {
        match recv {
            Value::Err(_) => Ok(recv.clone()),
            Value::Ns("fs") => match name {
                "read" => Ok(Value::Builtin("fs.read")),
                "write" => Ok(Value::Builtin("fs.write")),
                _ => Err(fail(format!("fs has no `{name}`"))),
            },
            Value::Ns("net") => match name {
                "listen" => Ok(Value::Builtin("net.listen")),
                _ => Err(fail(format!("net has no `{name}`"))),
            },
            Value::Record(fields) => fields
                .borrow()
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| fail(format!("no field `{name}`"))),
            Value::Tuple(parts) => name
                .parse::<usize>()
                .ok()
                .and_then(|i| parts.get(i).cloned())
                .ok_or_else(|| fail(format!("tuple has no `.{name}`"))),
            Value::List(items) if name.parse::<usize>().is_ok() => {
                let i: usize = name.parse().unwrap();
                Ok(items.borrow().get(i).cloned().unwrap_or_else(|| err("index out of range")))
            }
            other => {
                // UFCS receiver-last: x.f == f x (unary applied now; n-ary
                // returns a partially-bound builtin via a closure-less trick:
                // evaluate eagerly for unary, else stash receiver)
                if BUILTINS.contains(&name) {
                    match builtin_arity(name) {
                        Some(1) => self.builtin(leak(name), vec![other.clone()], env),
                        _ => Ok(Value::Fn(Rc::new(method_closure(name, other.clone())))),
                    }
                } else if let Value::Map(pairs) = other {
                    // maps answer field syntax with key lookup — models
                    // conflate json maps and records (domain-bench, 1 cell);
                    // a missing key is err, same as m["k"]
                    let hit = pairs.borrow().iter().find(|(k, _)| matches!(k, Value::Str(s) if **s == *name)).map(|(_, v)| v.clone());
                    Ok(hit.unwrap_or_else(|| Value::Err(format!("missing key {name}").into())))
                } else {
                    Err(fail(format!("no field or method `{name}` on {}", show(other))))
                }
            }
        }
    }

    fn iterate(&self, v: &Value) -> R<Vec<Value>> {
        Ok(match v {
            Value::List(items) => items.borrow().clone(),
            Value::Map(pairs) => pairs
                .borrow()
                .iter()
                .map(|(k, val)| {
                    Value::Record(Rc::new(RefCell::new(vec![("k".into(), k.clone()), ("v".into(), val.clone())])))
                })
                .collect(),
            Value::Str(s) => s.chars().map(|c| Value::Str(Rc::new(c.to_string()))).collect(),
            Value::Err(_) => Vec::new(), // iterating an err yields nothing
            other => return Err(fail(format!("cannot iterate {}", show(other)))),
        })
    }

    fn builtin(&self, name: &str, mut args: Vec<Value>, env: &Env) -> R<Value> {
        match name {
            "print" => {
                println!("{}", show(&args[0]));
                Ok(Value::Unit)
            }
            "range" => match args.as_slice() {
                [Value::Int(n)] => Ok(Value::List(Rc::new(RefCell::new((0..*n).map(Value::Int).collect())))),
                [Value::Int(a), Value::Int(b)] => {
                    Ok(Value::List(Rc::new(RefCell::new((*a..*b).map(Value::Int).collect()))))
                }
                _ => Err(fail("range needs int bounds")),
            },
            "len" => Ok(Value::Int(match &args[0] {
                Value::List(i) => i.borrow().len() as i64,
                Value::Str(s) => s.chars().count() as i64,
                Value::Map(m) => m.borrow().len() as i64,
                other => return Err(fail(format!("{} has no len", show(other)))),
            })),
            "sqrt" => match num_f(&args[0]) {
                Some(f) => Ok(Value::Float(f.sqrt())),
                None => Err(fail("sqrt needs a number")),
            },
            "trim" => str1(&args[0], |s| s.trim().to_string()),
            "lower" => str1(&args[0], |s| s.to_lowercase()),
            "upper" => str1(&args[0], |s| s.to_uppercase()),
            "replace" => match (&args[0], &args[1], &args[2]) {
                (Value::Str(from), Value::Str(to), Value::Str(s)) => {
                    Ok(Value::Str(Rc::new(s.replace(&**from, to))))
                }
                _ => Err(fail("replace from to s")),
            },
            "split" => match (&args[0], &args[1]) {
                (Value::Str(sep), Value::Str(s)) => Ok(list(
                    s.split(&**sep).map(|p| Value::Str(Rc::new(p.to_string()))).collect(),
                )),
                _ => Err(fail("split sep s")),
            },
            "words" => match &args[0] {
                Value::Str(s) => Ok(list(s.split_whitespace().map(|w| Value::Str(Rc::new(w.to_string()))).collect())),
                _ => Err(fail("words needs a string")),
            },
            "lines" => match &args[0] {
                Value::Str(s) => Ok(list(s.lines().map(|l| Value::Str(Rc::new(l.to_string()))).collect())),
                Value::Conn(stream) => {
                    // read lines from the socket until EOF
                    let mut out = Vec::new();
                    let s = stream.borrow();
                    let mut reader = std::io::BufReader::new(&*s);
                    let mut buf = String::new();
                    loop {
                        buf.clear();
                        match reader.read_line(&mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(_) => out.push(Value::Str(Rc::new(buf.trim_end_matches(['\r', '\n']).to_string()))),
                        }
                    }
                    Ok(list(out))
                }
                _ => Err(fail("lines needs a string or connection")),
            },
            "chars" => match &args[0] {
                Value::Str(s) => Ok(list(s.chars().map(|c| Value::Str(Rc::new(c.to_string()))).collect())),
                _ => Err(fail("chars needs a string")),
            },
            "bytes" => match &args[0] {
                Value::Str(s) => Ok(list(s.bytes().map(|b| Value::Int(b as i64)).collect())),
                _ => Err(fail("bytes needs a string")),
            },
            "digit" => match &args[0] {
                Value::Str(s) => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))),
                _ => Err(fail("digit needs a string")),
            },
            "int" => match &args[0] {
                Value::Str(s) => Ok(s.trim().parse::<i64>().map(Value::Int).unwrap_or_else(|_| err("not an int"))),
                Value::Float(f) => Ok(Value::Int(*f as i64)),
                v @ Value::Int(_) => Ok(v.clone()),
                _ => Err(fail("int conversion")),
            },
            "float" => match &args[0] {
                Value::Str(s) => Ok(s.trim().parse::<f64>().map(Value::Float).unwrap_or_else(|_| err("not a float"))),
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                v @ Value::Float(_) => Ok(v.clone()),
                _ => Err(fail("float conversion")),
            },
            "str" => Ok(Value::Str(Rc::new(show(&args[0])))),
            "err" => match &args[0] {
                // `err "msg"` constructs an error value (domain-bench: models
                // wrote this form unprompted in 6 cells)
                Value::Str(m) => Ok(Value::Err(m.to_string().into())),
                other => Ok(Value::Err(show(other).into())),
            },
            "json" => match &args[0] {
                Value::Str(s) => Ok(parse_json(s.trim()).unwrap_or_else(|| err("invalid json"))),
                _ => Err(fail("json needs a string")),
            },
            "counts" => match &args[0] {
                Value::List(items) => {
                    let mut pairs: Vec<(Value, Value)> = Vec::new();
                    for item in items.borrow().iter() {
                        if let Some(slot) = pairs.iter_mut().find(|(k, _)| eq(k, item)) {
                            if let Value::Int(n) = &mut slot.1 {
                                *n += 1;
                            }
                        } else {
                            pairs.push((item.clone(), Value::Int(1)));
                        }
                    }
                    Ok(Value::Map(Rc::new(RefCell::new(pairs))))
                }
                _ => Err(fail("counts needs a list")),
            },
            "pairs" => match &args[0] {
                Value::Map(m) => Ok(list(
                    m.borrow()
                        .iter()
                        .map(|(k, v)| {
                            Value::Record(Rc::new(RefCell::new(vec![("k".into(), k.clone()), ("v".into(), v.clone())])))
                        })
                        .collect(),
                )),
                _ => Err(fail("pairs needs a map")),
            },
            "map" => {
                let xs = take_list(args.pop().unwrap())?;
                let f = args.pop().unwrap();
                let mut out = Vec::new();
                for x in xs {
                    out.push(self.call(f.clone(), vec![x], env)?);
                }
                Ok(list(out))
            }
            "keep" => {
                let xs = take_list(args.pop().unwrap())?;
                let f = args.pop().unwrap();
                let mut out = Vec::new();
                for x in xs {
                    if truthy(&self.call(f.clone(), vec![x.clone()], env)?)? {
                        out.push(x);
                    }
                }
                Ok(list(out))
            }
            "top" => {
                let xs = take_list(args.pop().unwrap())?;
                let f = args.pop().unwrap();
                let n = match args.pop() {
                    Some(Value::Int(n)) => n as usize,
                    _ => return Err(fail("top n keyfn list")),
                };
                let mut keyed: Vec<(Value, Value)> = Vec::new();
                for x in xs {
                    let k = self.call(f.clone(), vec![x.clone()], env)?;
                    keyed.push((k, x));
                }
                // stable sort, descending by key
                keyed.sort_by(|a, b| cmp_vals(&b.0, &a.0));
                Ok(list(keyed.into_iter().take(n).map(|(_, x)| x).collect()))
            }
            "sort" => {
                let mut xs = take_list(args.pop().unwrap())?;
                xs.sort_by(cmp_vals);
                Ok(list(xs))
            }
            "rev" => {
                let mut xs = take_list(args.pop().unwrap())?;
                xs.reverse();
                Ok(list(xs))
            }
            "first" | "last" | "min" | "max" => {
                let xs = take_list(args.pop().unwrap())?;
                let v = match name {
                    "first" => xs.first().cloned(),
                    "last" => xs.last().cloned(),
                    "min" => xs.iter().min_by(|a, b| cmp_vals(a, b)).cloned(),
                    _ => xs.iter().max_by(|a, b| cmp_vals(a, b)).cloned(),
                };
                Ok(v.unwrap_or_else(|| err("empty list")))
            }
            "sum" => {
                let xs = take_list(args.pop().unwrap())?;
                let mut acc = Value::Int(0);
                for x in xs {
                    acc = binop("+", &acc, &x)?;
                }
                Ok(acc)
            }
            "fold" => {
                let xs = take_list(args.pop().unwrap())?;
                let f = args.pop().unwrap();
                let mut acc = args.pop().ok_or_else(|| fail("fold init f list"))?;
                for x in xs {
                    acc = self.call(f.clone(), vec![acc, x], env)?;
                }
                Ok(acc)
            }
            "group" => {
                let xs = take_list(args.pop().unwrap())?;
                let f = args.pop().unwrap();
                let mut groups: Vec<(Value, Vec<Value>)> = Vec::new();
                for x in xs {
                    let k = self.call(f.clone(), vec![x.clone()], env)?;
                    if let Some(slot) = groups.iter_mut().find(|(gk, _)| eq(gk, &k)) {
                        slot.1.push(x);
                    } else {
                        groups.push((k, vec![x]));
                    }
                }
                Ok(list(
                    groups
                        .into_iter()
                        .map(|(k, v)| {
                            Value::Record(Rc::new(RefCell::new(vec![("k".into(), k), ("v".into(), list(v))])))
                        })
                        .collect(),
                ))
            }
            "flat" => {
                let xs = take_list(args.pop().unwrap())?;
                let mut out = Vec::new();
                for x in xs {
                    out.extend(take_list(x)?);
                }
                Ok(list(out))
            }
            "join" => match (&args[0], &args[1]) {
                (Value::Str(sep), Value::List(items)) => Ok(Value::Str(Rc::new(
                    items.borrow().iter().map(show).collect::<Vec<_>>().join(sep),
                ))),
                _ => Err(fail("join sep list")),
            },
            "fs.read" => {
                if !self.caps.fs {
                    return Ok(err("fs capability not granted (run with --fs)"));
                }
                match &args[0] {
                    Value::Str(p) => Ok(std::fs::read_to_string(&**p)
                        .map(|s| Value::Str(Rc::new(s)))
                        .unwrap_or_else(|e| err(format!("fs.read {p}: {e}")))),
                    _ => Err(fail("fs.read path")),
                }
            }
            "fs.write" => {
                if !self.caps.fs {
                    return Ok(err("fs capability not granted (run with --fs)"));
                }
                match (&args[0], &args[1]) {
                    (Value::Str(content), Value::Str(p)) => Ok(std::fs::write(&**p, &**content)
                        .map(|_| Value::Unit)
                        .unwrap_or_else(|e| err(format!("fs.write {p}: {e}")))),
                    _ => Err(fail("fs.write content path")),
                }
            }
            "net.listen" => {
                if !self.caps.net {
                    return Ok(err("net capability not granted (run with --net)"));
                }
                match &args[0] {
                    Value::Int(port) => std::net::TcpListener::bind(("127.0.0.1", *port as u16))
                        .map(|l| Value::Listener(Rc::new(l)))
                        .map_err(|e| fail(format!("net.listen {port}: {e}"))),
                    _ => Err(fail("net.listen port")),
                }
            }
            "write" => match (&args[0], &args[1]) {
                (Value::Str(s), Value::Conn(stream)) => {
                    let mut st = stream.borrow_mut();
                    st.write_all(s.as_bytes()).map(|_| Value::Unit).map_err(|e| fail(format!("write: {e}")))
                }
                _ => Err(fail("write s conn")),
            },
            other => Err(fail(format!("builtin `{other}` not implemented"))),
        }
    }

    /// `"{expr}"` interpolation: brace fragments parse and evaluate in scope.
    fn interpolate(&self, s: &str, env: &Env) -> R<Value> {
        if !s.contains('{') {
            return Ok(Value::Str(Rc::new(unescape(s))));
        }
        let mut out = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '{' {
                let mut depth = 1;
                let mut frag = String::new();
                for c2 in chars.by_ref() {
                    match c2 {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    frag.push(c2);
                }
                let prog = crate::parse_source(&format!("{frag}\n"))
                    .map_err(|d| fail(format!("bad interpolation `{{{frag}}}`: {d}")))?;
                if let Some(Stmt::Expr(e)) = prog.first() {
                    let v = self.expr(&rewrite_pipes(e), env)?;
                    out.push_str(&show(&v));
                } else {
                    if frag.trim().is_empty() {
                        // "{}" is a literal brace pair, not an empty hole
                        // (domain-bench: models write "{}" as empty-JSON text)
                        out.push_str("{}");
                        continue;
                    }
                    return Err(fail(format!("interpolation `{{{frag}}}` is not an expression")));
                }
            } else if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(other) => out.push(other),
                    None => {}
                }
            } else {
                out.push(c);
            }
        }
        Ok(Value::Str(Rc::new(out)))
    }
}

fn method_closure(name: &str, recv: Value) -> Closure {
    // n-ary UFCS: x.f -> closure (a b ...) = f a b ... x   (receiver LAST)
    let arity = builtin_arity(name).unwrap_or(2);
    let params: Vec<String> = (0..arity - 1).map(|i| format!("__a{i}")).collect();
    let env = Env::new();
    env.define("__recv", recv);
    let mut args: Vec<Expr> = params.iter().map(|p| Expr::Name(p.clone())).collect();
    args.push(Expr::Name("__recv".into()));
    let body = Expr::App { head: Box::new(Expr::Name(name.to_string())), args };
    Closure { params, body: Body::Expr(body), env }
}

const BUILTINS: &[&str] = &[
    "print", "range", "len", "sqrt", "trim", "lower", "upper", "replace", "split", "words", "lines",
    "chars", "bytes", "digit", "int", "float", "str", "json", "counts", "pairs", "map", "keep", "top",
    "sort", "rev", "first", "last", "min", "max", "sum", "fold", "group", "flat", "join", "write",
    "err",
];

fn builtin_arity(name: &str) -> Option<usize> {
    Some(match name {
        "print" | "len" | "sqrt" | "trim" | "lower" | "upper" | "words" | "lines" | "chars" | "bytes"
        | "digit" | "int" | "float" | "str" | "json" | "counts" | "pairs" | "sort" | "rev" | "first"
        | "last" | "min" | "max" | "sum" | "flat" | "range" | "fs.read" | "net.listen" | "err" => 1,
        "split" | "map" | "keep" | "group" | "join" | "write" | "fs.write" => 2,
        "replace" | "top" | "fold" => 3,
        _ => return None,
    })
}

fn leak(s: &str) -> &'static str {
    // builtin names are a small fixed set; resolve to the static slice
    BUILTINS.iter().find(|b| **b == s).copied().unwrap_or("print")
}

fn list(v: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(v)))
}

fn take_list(v: Value) -> R<Vec<Value>> {
    match v {
        Value::List(items) => Ok(items.borrow().clone()),
        Value::Map(pairs) => Ok(pairs
            .borrow()
            .iter()
            .map(|(k, val)| {
                Value::Record(Rc::new(RefCell::new(vec![("k".into(), k.clone()), ("v".into(), val.clone())])))
            })
            .collect()),
        Value::Err(_) => Ok(Vec::new()),
        other => Err(fail(format!("{} is not a list", show(&other)))),
    }
}

fn str1(v: &Value, f: impl Fn(&str) -> String) -> R<Value> {
    match v {
        Value::Str(s) => Ok(Value::Str(Rc::new(f(s)))),
        other => Err(fail(format!("{} is not a string", show(other)))),
    }
}

fn num_f(v: &Value) -> Option<f64> {
    match v {
        Value::Int(i) => Some(*i as f64),
        Value::UInt(u) => Some(*u as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

fn parse_num(n: &str) -> Value {
    if let Some(stripped) = n.strip_suffix("u64").or_else(|| n.strip_suffix("u32")) {
        return Value::UInt(stripped.parse().unwrap_or(0));
    }
    if let Some(pos) = n.find(['i', 'u']) {
        return Value::Int(n[..pos].parse().unwrap_or(0));
    }
    if n.contains('.') {
        Value::Float(n.parse().unwrap_or(0.0))
    } else {
        Value::Int(n.parse().unwrap_or(0))
    }
}

fn truthy(v: &Value) -> R<bool> {
    match v {
        Value::Bool(b) => Ok(*b),
        Value::Err(_) => Ok(false),
        other => Err(fail(format!("{} is not a bool", show(other)))),
    }
}

fn as_index(v: &Value, len: usize) -> R<usize> {
    match v {
        Value::Int(i) if *i >= 0 && (*i as usize) < len => Ok(*i as usize),
        other => Err(fail(format!("index {} out of range (len {len})", show(other)))),
    }
}

fn index(recv: &Value, i: &Value) -> R<Value> {
    match recv {
        Value::Err(_) => Ok(recv.clone()),
        Value::List(items) => {
            let items = items.borrow();
            match i {
                Value::Int(n) if *n >= 0 && (*n as usize) < items.len() => Ok(items[*n as usize].clone()),
                _ => Ok(err("index out of range")),
            }
        }
        Value::Str(s) => match i {
            Value::Int(n) if *n >= 0 => Ok(s
                .chars()
                .nth(*n as usize)
                .map(|c| Value::Str(Rc::new(c.to_string())))
                .unwrap_or_else(|| err("index out of range"))),
            _ => Ok(err("index out of range")),
        },
        Value::Map(pairs) => Ok(pairs
            .borrow()
            .iter()
            .find(|(k, _)| eq(k, i))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| err(format!("key {} absent", show(i))))),
        other => Err(fail(format!("{} is not indexable", show(other)))),
    }
}

fn slice(recv: &Value, lo: Option<Value>, hi: Option<Value>) -> R<Value> {
    let lo_i = match &lo {
        Some(Value::Int(i)) => *i as usize,
        None => 0,
        _ => return Err(fail("slice bounds must be ints")),
    };
    match recv {
        Value::Err(_) => Ok(recv.clone()),
        Value::List(items) => {
            let items = items.borrow();
            let hi_i = match &hi {
                Some(Value::Int(i)) => (*i as usize).min(items.len()),
                None => items.len(),
                _ => return Err(fail("slice bounds must be ints")),
            };
            Ok(list(items[lo_i.min(items.len())..hi_i].to_vec()))
        }
        Value::Str(s) => {
            let chars: Vec<char> = s.chars().collect();
            let hi_i = match &hi {
                Some(Value::Int(i)) => (*i as usize).min(chars.len()),
                None => chars.len(),
                _ => return Err(fail("slice bounds must be ints")),
            };
            Ok(Value::Str(Rc::new(chars[lo_i.min(chars.len())..hi_i].iter().collect())))
        }
        other => Err(fail(format!("{} is not sliceable", show(other)))),
    }
}

fn eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Err(x), Value::Err(y)) => x == y,
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::UInt(x), Value::UInt(y)) => x == y,
        (Value::Int(x), Value::Float(y)) | (Value::Float(y), Value::Int(x)) => *x as f64 == *y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Unit, Value::Unit) => true,
        (Value::List(x), Value::List(y)) => {
            let (x, y) = (x.borrow(), y.borrow());
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| eq(a, b))
        }
        (Value::Tuple(x), Value::Tuple(y)) => x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| eq(a, b)),
        _ => false,
    }
}

fn cmp_vals(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::UInt(x), Value::UInt(y)) => x.cmp(y),
        (Value::Str(x), Value::Str(y)) => x.cmp(y),
        _ => num_f(a)
            .zip(num_f(b))
            .map(|(x, y)| x.partial_cmp(&y).unwrap_or(Ordering::Equal))
            .unwrap_or(Ordering::Equal),
    }
}

fn binop(op: &str, l: &Value, r: &Value) -> R<Value> {
    use Value::*;
    // equality COMPARES err values instead of propagating them — models
    // test `x == err "..."` deliberately (domain-bench)
    match op {
        "==" => return Ok(Bool(eq(l, r))),
        "!=" => return Ok(Bool(!eq(l, r))),
        _ => {}
    }
    if let Value::Err(_) = l {
        return Ok(l.clone());
    }
    if let Value::Err(_) = r {
        return Ok(r.clone());
    }
    match op {
        "<" | "<=" | ">" | ">=" => {
            let ord = cmp_vals(l, r);
            let b = match op {
                "<" => ord.is_lt(),
                "<=" => ord.is_le(),
                ">" => ord.is_gt(),
                _ => ord.is_ge(),
            };
            return Ok(Bool(b));
        }
        "in" => {
            return Ok(Bool(match (l, r) {
                (Str(needle), Str(hay)) => hay.contains(&**needle),
                (x, List(items)) => items.borrow().iter().any(|i| eq(i, x)),
                (k, Map(pairs)) => pairs.borrow().iter().any(|(key, _)| eq(key, k)),
                _ => false,
            }));
        }
        _ => {}
    }
    // arithmetic: UInt wraps (bit-level work); Int is plain; Float via f64;
    // + concatenates strings
    match (l, r) {
        (Str(a), Str(b)) if op == "+" => Ok(Str(Rc::new(format!("{a}{b}")))),
        // list concatenation (spec-truth: top benchmark failure cause)
        (List(a), List(b)) if op == "+" => {
            let mut out = a.borrow().clone();
            out.extend(b.borrow().iter().cloned());
            Ok(List(Rc::new(RefCell::new(out))))
        }
        (l2, r2) if matches!(l2, UInt(_)) || matches!(r2, UInt(_)) => {
            let as_u = |v: &Value| match v {
                UInt(u) => Some(*u),
                Int(i) if *i >= 0 => Some(*i as u64),
                _ => None,
            };
            let (Some(a), Some(b2)) = (as_u(l2), as_u(r2)) else {
                return Result::Err(fail(format!("cannot {op} {} and {}", show(l2), show(r2))));
            };
            Ok(UInt(match op {
                "+" => a.wrapping_add(b2),
                "-" => a.wrapping_sub(b2),
                "*" => a.wrapping_mul(b2),
                "/" => a.checked_div(b2).unwrap_or(0),
                "%" => a.checked_rem(b2).unwrap_or(0),
                "^" => a ^ b2,
                "**" => a.wrapping_pow(b2 as u32),
                _ => return Result::Err(fail(format!("bad op {op}"))),
            }))
        }
        (Int(a), Int(b)) => Ok(match op {
            "+" => Int(a + b),
            "-" => Int(a - b),
            "*" => Int(a * b),
            "/" => {
                if *b == 0 {
                    err("division by zero")
                } else {
                    Int(a / b)
                }
            }
            "%" => {
                if *b == 0 {
                    err("division by zero")
                } else {
                    Int(a % b)
                }
            }
            "^" => Int(a ^ b),
            "**" => Int(a.pow(*b as u32)),
            _ => return Result::Err(fail(format!("bad op {op}"))),
        }),
        _ => match (num_f(l), num_f(r)) {
            (Some(a), Some(b)) => Ok(match op {
                "+" => Float(a + b),
                "-" => Float(a - b),
                "*" => Float(a * b),
                "/" => Float(a / b),
                "%" => Float(a % b),
                "**" | "^" => Float(a.powf(b)),
                _ => return Result::Err(fail(format!("bad op {op}"))),
            }),
            _ => Result::Err(fail(format!("cannot {op} {} and {}", show(l), show(r)))),
        },
    }
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// minimal JSON reader (corpus 07 reads a config object)
fn parse_json(s: &str) -> Option<Value> {
    let s = s.trim();
    if s.starts_with('{') {
        // extremely small object parser: {"k": v, ...}
        let inner = s.strip_prefix('{')?.strip_suffix('}')?.trim();
        let mut pairs = Vec::new();
        if !inner.is_empty() {
            for part in split_top(inner, ',') {
                let (k, v) = part.split_once(':')?;
                let key = k.trim().strip_prefix('"')?.strip_suffix('"')?.to_string();
                pairs.push((Value::Str(Rc::new(key)), parse_json(v.trim())?));
            }
        }
        Some(Value::Map(Rc::new(RefCell::new(pairs))))
    } else if s.starts_with('[') {
        let inner = s.strip_prefix('[')?.strip_suffix(']')?.trim();
        let mut items = Vec::new();
        if !inner.is_empty() {
            for part in split_top(inner, ',') {
                items.push(parse_json(part.trim())?);
            }
        }
        Some(list(items))
    } else if let Some(stripped) = s.strip_prefix('"') {
        Some(Value::Str(Rc::new(stripped.strip_suffix('"')?.to_string())))
    } else if s == "true" {
        Some(Value::Bool(true))
    } else if s == "false" {
        Some(Value::Bool(false))
    } else if s == "null" {
        Some(Value::Unit)
    } else if s.contains('.') {
        s.parse().ok().map(Value::Float)
    } else {
        s.parse().ok().map(Value::Int)
    }
}

fn split_top(s: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start, mut in_str) = (0usize, 0usize, false);
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '{' | '[' if !in_str => depth += 1,
            '}' | ']' if !in_str => depth = depth.saturating_sub(1),
            c if c == sep && depth == 0 && !in_str => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// Display form (print): machine-simple, deterministic.
pub fn show(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::UInt(u) => u.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Value::Str(s) => (**s).clone(),
        Value::Bool(b) => b.to_string(),
        Value::Unit => "()".into(),
        Value::List(items) => {
            let inner: Vec<String> = items.borrow().iter().map(show).collect();
            format!("[{}]", inner.join(", "))
        }
        Value::Map(pairs) => {
            let inner: Vec<String> = pairs.borrow().iter().map(|(k, v)| format!("{}: {}", show(k), show(v))).collect();
            format!("{{{}}}", inner.join(", "))
        }
        Value::Record(fields) => {
            let inner: Vec<String> = fields.borrow().iter().map(|(n, v)| format!("{n}: {}", show(v))).collect();
            format!("{{{}}}", inner.join(", "))
        }
        Value::Tuple(parts) => {
            let inner: Vec<String> = parts.iter().map(show).collect();
            format!("({})", inner.join(", "))
        }
        Value::Fn(_) | Value::Builtin(_) => "<fn>".into(),
        Value::Ns(n) => format!("<{n}>"),
        Value::Conn(_) => "<conn>".into(),
        Value::Listener(_) => "<listener>".into(),
        Value::Err(m) => format!("err({m})"),
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", show(self))
    }
}
