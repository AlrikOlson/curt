//! Gradual HM-lite type inference for curt v0.1 (SPEC §3) — `curt check`.
//!
//! Two elaboration rules live here (both SPEC-bound):
//! 1. ARITY RESOLUTION (§2.3): applications parse flat; when a head of arity
//!    k receives n>k arguments, the surplus re-nests under the k-th argument
//!    (`print show 2.5` ⇒ `print (show 2.5)`). n<k is a no-currying error.
//! 2. PIPE CAPTURE (§2.6 note): a pipeline captures the last argument of a
//!    preceding juxtaposition: `print us | keep .active` ⇒
//!    `print (us | keep .active)`. Later stages apply with the piped value
//!    appended as their final argument.
//!
//! Gradual edges: host values (net streams, json, args) are `Any`, which
//! unifies with everything. Goldens assert concrete types only where derivable.

use crate::ast::*;
use crate::diag::Diag;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Float,
    Str,
    Bool,
    Unit,
    List(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
    Tuple(Vec<Ty>),
    Record { name: Option<String>, fields: Vec<(String, Ty)> },
    Union(Vec<Ty>),
    Fn { params: Vec<Ty>, ret: Box<Ty> },
    Var(usize),
    Any,
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "int"),
            Ty::Float => write!(f, "float"),
            Ty::Str => write!(f, "str"),
            Ty::Bool => write!(f, "bool"),
            Ty::Unit => write!(f, "()"),
            Ty::List(t) => write!(f, "[{t}]"),
            Ty::Map(k, v) => write!(f, "{{{k}: {v}}}"),
            Ty::Tuple(ts) => {
                let inner: Vec<String> = ts.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", inner.join(", "))
            }
            Ty::Record { name: Some(n), .. } => write!(f, "{n}"),
            Ty::Record { name: None, fields } => {
                let inner: Vec<String> = fields.iter().map(|(n, t)| format!("{n} {t}")).collect();
                write!(f, "{{{}}}", inner.join(", "))
            }
            Ty::Union(parts) => {
                let inner: Vec<String> = parts.iter().map(|t| t.to_string()).collect();
                write!(f, "{}", inner.join(" | "))
            }
            Ty::Fn { params, ret } => {
                let ps: Vec<String> = params.iter().map(|t| t.to_string()).collect();
                write!(f, "({} -> {ret})", ps.join(" "))
            }
            Ty::Var(i) => write!(f, "{}", var_name(*i)),
            Ty::Any => write!(f, "any"),
        }
    }
}

fn var_name(i: usize) -> String {
    let c = (b'a' + (i % 26) as u8) as char;
    if i < 26 {
        c.to_string()
    } else {
        format!("{c}{}", i / 26)
    }
}

pub struct Checker {
    bind: Vec<Option<Ty>>,
    env: Vec<HashMap<String, Ty>>,
    types: HashMap<String, Ty>, // nominal records
    /// type vars owned by fn/lambda parameters: pinned by USAGE, never by
    /// branch-merging (a fn's result var, by contrast, IS pinned by merging)
    param_vars: std::collections::HashSet<usize>,
    pub sigs: Vec<(String, Ty)>, // toplevel equation signatures, in order
    pub warnings: Vec<String>,
}

type Res<T> = Result<T, Diag>;

fn err(kind: &str, msg: &str, fix: &str) -> Diag {
    Diag::at(kind, 0, 0, msg, fix)
}

impl Checker {
    pub fn new() -> Self {
        let mut c = Checker { bind: Vec::new(), env: vec![HashMap::new()], types: HashMap::new(), param_vars: std::collections::HashSet::new(), sigs: Vec::new(), warnings: Vec::new() };
        c.install_stdlib();
        c
    }

    fn fresh(&mut self) -> Ty {
        self.bind.push(None);
        Ty::Var(self.bind.len() - 1)
    }

    pub fn resolve(&self, t: &Ty) -> Ty {
        match t {
            Ty::Var(i) => match &self.bind[*i] {
                Some(inner) => self.resolve(inner),
                None => t.clone(),
            },
            Ty::List(x) => Ty::List(Box::new(self.resolve(x))),
            Ty::Map(k, v) => Ty::Map(Box::new(self.resolve(k)), Box::new(self.resolve(v))),
            Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(|x| self.resolve(x)).collect()),
            Ty::Union(ts) => Ty::Union(ts.iter().map(|x| self.resolve(x)).collect()),
            Ty::Fn { params, ret } => Ty::Fn { params: params.iter().map(|x| self.resolve(x)).collect(), ret: Box::new(self.resolve(ret)) },
            Ty::Record { name, fields } => Ty::Record { name: name.clone(), fields: fields.iter().map(|(n, x)| (n.clone(), self.resolve(x))).collect() },
            other => other.clone(),
        }
    }

    fn unify(&mut self, a: &Ty, b: &Ty) -> Res<()> {
        let (a, b) = (self.resolve(a), self.resolve(b));
        match (&a, &b) {
            // equality first: Var(i) == Var(i) must NOT self-bind (cycle)
            (x, y) if x == y => Ok(()),
            (Ty::Any, _) | (_, Ty::Any) => Ok(()),
            (Ty::Var(i), _) => {
                // occurs check: a cyclic binding collapses to Any (gradual)
                self.bind[*i] = Some(if occurs(*i, &b) { Ty::Any } else { b.clone() });
                Ok(())
            }
            (_, Ty::Var(i)) => {
                self.bind[*i] = Some(if occurs(*i, &a) { Ty::Any } else { a.clone() });
                Ok(())
            }
            (Ty::List(x), Ty::List(y)) => self.unify(x, y),
            (Ty::Map(k1, v1), Ty::Map(k2, v2)) => {
                self.unify(k1, k2)?;
                self.unify(v1, v2)
            }
            (Ty::Tuple(xs), Ty::Tuple(ys)) if xs.len() == ys.len() => {
                for (x, y) in xs.iter().zip(ys) {
                    self.unify(x, y)?;
                }
                Ok(())
            }
            (Ty::Fn { params: p1, ret: r1 }, Ty::Fn { params: p2, ret: r2 }) if p1.len() == p2.len() => {
                for (x, y) in p1.iter().zip(p2) {
                    self.unify(x, y)?;
                }
                self.unify(r1, r2)
            }
            // a member fits its union (untagged unions, SPEC §3)
            (x, Ty::Union(parts)) if parts.iter().any(|p| self.resolve(p) == *x) => Ok(()),
            (Ty::Union(parts), y) if parts.iter().any(|p| self.resolve(p) == *y) => Ok(()),
            (Ty::Record { name: Some(n1), .. }, Ty::Record { name: Some(n2), .. }) if n1 == n2 => Ok(()),
            (Ty::Record { fields: f1, .. }, Ty::Record { fields: f2, .. }) => {
                // structural: shared field names must agree
                for (n, t1) in f1 {
                    if let Some((_, t2)) = f2.iter().find(|(m, _)| m == n) {
                        self.unify(t1, t2)?;
                    }
                }
                Ok(())
            }
            (x, y) => Err(err("type_mismatch", &format!("expected {y}, got {x}"), "check the operand types or add a conversion (.int / .float / .str)")),
        }
    }

    /// Directed check: does `actual` fit where `expected` is required?
    /// Like unify, plus int literals WIDEN to float in argument/annotation/
    /// field positions (SPEC §3 numerics). Binary operators stay strict
    /// (`1 + 2.5` remains a type error).
    fn fits(&mut self, actual: &Ty, expected: &Ty) -> Res<()> {
        let (a, e) = (self.resolve(actual), self.resolve(expected));
        match (&a, &e) {
            (Ty::Int, Ty::Float) => Ok(()),
            (Ty::List(x), Ty::List(y)) => self.fits(x, y),
            (Ty::Tuple(xs), Ty::Tuple(ys)) if xs.len() == ys.len() => {
                for (x, y) in xs.iter().zip(ys) {
                    self.fits(x, y)?;
                }
                Ok(())
            }
            (Ty::Record { name: n1, fields: f1 }, Ty::Record { name: n2, fields: f2 }) => {
                if let (Some(x), Some(y)) = (n1, n2) {
                    if x != y {
                        return Err(err("type_mismatch", &format!("expected {y}, got {x}"), "record types differ"));
                    }
                }
                for (n, t1) in f1 {
                    if let Some((_, t2)) = f2.iter().find(|(m, _)| m == n) {
                        self.fits(t1, t2)?;
                    }
                }
                Ok(())
            }
            _ => self.unify(&a, &e),
        }
    }

    // ---- environment ----

    fn lookup(&self, name: &str) -> Option<Ty> {
        self.env.iter().rev().find_map(|scope| scope.get(name).cloned())
    }

    fn define(&mut self, name: &str, ty: Ty) {
        self.env.last_mut().unwrap().insert(name.to_string(), ty);
    }

    fn scoped<T>(&mut self, f: impl FnOnce(&mut Self) -> Res<T>) -> Res<T> {
        self.env.push(HashMap::new());
        let r = f(self);
        self.env.pop();
        r
    }

    fn install_stdlib(&mut self) {
        // host edges are Any (gradual); generic verbs instantiate per use via
        // fresh vars in `stdlib_call` below. Plain values here.
        for (name, ty) in [
            ("args", Ty::Any),
            ("print", Ty::Fn { params: vec![Ty::Any], ret: Box::new(Ty::Unit) }),
            ("range", Ty::Fn { params: vec![Ty::Int], ret: Box::new(Ty::List(Box::new(Ty::Int))) }),
            ("fs", Ty::Any),
            ("net", Ty::Any),
        ] {
            self.define(name, ty);
        }
    }

    /// Generic stdlib verbs: instantiated with fresh vars at each use.
    fn stdlib_fn(&mut self, name: &str) -> Option<Ty> {
        let a = self.fresh();
        let b = self.fresh();
        let f1 = |p: Vec<Ty>, r: Ty| Ty::Fn { params: p, ret: Box::new(r) };
        Some(match name {
            "map" => f1(vec![f1(vec![a.clone()], b.clone()), Ty::List(Box::new(a))], Ty::List(Box::new(b))),
            "keep" => f1(vec![f1(vec![a.clone()], Ty::Bool), Ty::List(Box::new(a.clone()))], Ty::List(Box::new(a))),
            "top" => f1(vec![Ty::Int, f1(vec![a.clone()], Ty::Any), Ty::List(Box::new(a.clone()))], Ty::List(Box::new(a))),
            "sort" | "rev" | "flat" => f1(vec![Ty::List(Box::new(a.clone()))], Ty::List(Box::new(a))),
            "first" | "last" | "min" | "max" => f1(vec![Ty::List(Box::new(a.clone()))], a),
            "sum" => f1(vec![Ty::List(Box::new(a.clone()))], a),
            "fold" => f1(vec![b.clone(), f1(vec![b.clone(), a.clone()], b.clone()), Ty::List(Box::new(a))], b),
            "join" => f1(vec![Ty::Str, Ty::List(Box::new(a))], Ty::Str),
            "group" => f1(
                vec![f1(vec![a.clone()], b.clone()), Ty::List(Box::new(a.clone()))],
                Ty::List(Box::new(Ty::Record { name: None, fields: vec![("k".into(), b), ("v".into(), Ty::List(Box::new(a)))] })),
            ),
            "counts" => f1(vec![Ty::List(Box::new(a.clone()))], Ty::Map(Box::new(a), Box::new(Ty::Int))),
            "pairs" => f1(
                vec![Ty::Map(Box::new(a.clone()), Box::new(b.clone()))],
                Ty::List(Box::new(Ty::Record { name: None, fields: vec![("k".into(), a), ("v".into(), b)] })),
            ),
            "len" => f1(vec![Ty::Any], Ty::Int),
            "words" | "lines" | "chars" => f1(vec![Ty::Str], Ty::List(Box::new(Ty::Str))),
            "bytes" => f1(vec![Ty::Str], Ty::List(Box::new(Ty::Int))),
            "trim" | "lower" | "upper" => f1(vec![Ty::Str], Ty::Str),
            "replace" => f1(vec![Ty::Str, Ty::Str, Ty::Str], Ty::Str),
            "split" => f1(vec![Ty::Str, Ty::Str], Ty::List(Box::new(Ty::Str))),
            "write" => f1(vec![Ty::Str, Ty::Any], Ty::Unit),
            "sqrt" => f1(vec![Ty::Float], Ty::Float),
            "int" => f1(vec![Ty::Any], Ty::Int),
            "float" => f1(vec![Ty::Any], Ty::Float),
            "str" => f1(vec![Ty::Any], Ty::Str),
            "json" => f1(vec![Ty::Str], Ty::Any),
            "err" => f1(vec![Ty::Str], Ty::Any),
            "digit" => f1(vec![Ty::Str], Ty::Bool),
            _ => return None,
        })
    }

    // ---- elaboration rule 2: trailing-operator capture (SPEC §2.3) ----
    // a pipeline OR a spaced rescue captures the last argument of a
    // preceding juxtaposition: `print us | keep ...` == print (us | ...);
    // `print m["k"] ? 8080` == print (m["k"] ? 8080)

    fn rewrite_pipes(e: &Expr) -> Expr {
        rewrite_sugar(e)
    }

    // ---- program checking ----

    pub fn check_program(&mut self, prog: &[Stmt]) -> Res<()> {
        // pass 1: nominal types + signatures, then equation pre-declarations
        // (an explicit `::` signature is the contract — never overwritten)
        let mut signed: Vec<&str> = Vec::new();
        for s in prog {
            match s {
                Stmt::TypeDecl { name, ty } => {
                    let t = self.type_from(ty);
                    self.types.insert(name.clone(), t);
                }
                Stmt::Sig { name, params, ret, .. } => {
                    let p: Vec<Ty> = params.iter().map(|t| self.type_from(t)).collect();
                    let r = ret.as_ref().map(|t| self.type_from(t)).unwrap_or(Ty::Unit);
                    self.define(name, Ty::Fn { params: p, ret: Box::new(r) });
                    signed.push(name);
                }
                _ => {}
            }
        }
        for s in prog {
            if let Stmt::Equation { name, params, .. } = s {
                if signed.contains(&name.as_str()) {
                    continue;
                }
                let params: Vec<Ty> = params.iter().map(|_| self.fresh()).collect();
                let ret = self.fresh();
                self.define(name, Ty::Fn { params, ret: Box::new(ret) });
            }
        }
        // pass 2: bodies + toplevel statements
        for s in prog {
            self.stmt(s)?;
        }
        // collect rendered signatures for expand
        for s in prog {
            if let Stmt::Equation { name, .. } = s {
                if let Some(t) = self.lookup(name) {
                    let t = self.resolve(&t);
                    self.sigs.push((name.clone(), t));
                }
            }
        }
        Ok(())
    }

    fn type_from(&mut self, t: &TypeExpr) -> Ty {
        match t {
            TypeExpr::Named(n) => match n.as_str() {
                "int" => Ty::Int,
                "float" => Ty::Float,
                "str" => Ty::Str,
                "bool" => Ty::Bool,
                "bytes" => Ty::List(Box::new(Ty::Int)),
                _ => self.types.get(n).cloned().unwrap_or(Ty::Any),
            },
            TypeExpr::Union(parts) => Ty::Union(parts.iter().map(|p| self.type_from(p)).collect()),
            TypeExpr::Record(fields) => Ty::Record { name: None, fields: fields.iter().map(|(n, t)| (n.clone(), self.type_from(t))).collect() },
            TypeExpr::List(inner) => Ty::List(Box::new(self.type_from(inner))),
            TypeExpr::Fn { params, ret } => Ty::Fn { params: params.iter().map(|p| self.type_from(p)).collect(), ret: Box::new(self.type_from(ret)) },
        }
    }

    fn stmt(&mut self, s: &Stmt) -> Res<()> {
        match s {
            Stmt::TypeDecl { .. } | Stmt::Sig { .. } => Ok(()),
            Stmt::Equation { name, params, body } => {
                let fn_ty = self.lookup(name).unwrap();
                let (ptys, rty) = match self.resolve(&fn_ty) {
                    Ty::Fn { params, ret } => (params, *ret),
                    _ => return Err(err("internal", "equation without fn type", "")),
                };
                self.scoped(|c| {
                    for (p, t) in params.iter().zip(&ptys) {
                        c.lint_name(p);
                        if let Ty::Var(i) = t {
                            c.param_vars.insert(*i);
                        }
                        c.define(p, t.clone());
                    }
                    let bt = match body {
                        Body::Expr(e) => c.expr(&Self::rewrite_pipes(e))?,
                        Body::Block(stmts) => c.block(stmts)?,
                        Body::Stmt(st) => {
                            c.stmt(st)?;
                            Ty::Unit
                        }
                    };
                    c.unify(&bt, &rty)
                })?;
                self.lint_name(name);
                Ok(())
            }
            Stmt::Destructure { names, value } => {
                let vt = self.expr(&Self::rewrite_pipes(value))?;
                let parts: Vec<Ty> = names.iter().map(|_| self.fresh()).collect();
                self.unify(&vt, &Ty::Tuple(parts.clone()))?;
                for (n, t) in names.iter().zip(parts) {
                    self.define(n, t);
                }
                Ok(())
            }
            Stmt::Binding { target, ann, value } => {
                let vt = self.expr(&Self::rewrite_pipes(value))?;
                if let Some(a) = ann {
                    let at = self.type_from(a);
                    self.fits(&vt, &at)?;
                }
                if target.indices.is_empty() {
                    self.lint_name(&target.name);
                    // rebinding unifies with prior type when present (mutation)
                    if let Some(prev) = self.lookup(&target.name) {
                        // allow shadowing with a new type only at top scope redefinition;
                        // v0.1 keeps it simple: unify (mutation-compatible)
                        if self.unify(&prev, &vt).is_err() {
                            self.define(&target.name, vt);
                            return Ok(());
                        }
                    }
                    self.define(&target.name, vt);
                } else {
                    let mut t = self.lookup(&target.name).ok_or_else(|| err("unknown_name", &format!("`{}` is not defined", target.name), "bind it first"))?;
                    for ix in &target.indices {
                        let it = self.expr(ix)?;
                        t = self.index_result(&t, &it)?;
                    }
                    self.unify(&vt, &t)?;
                }
                Ok(())
            }
            Stmt::Compound { target, value, .. } => {
                let mut t = self
                    .lookup(&target.name)
                    .ok_or_else(|| err("unknown_name", &format!("`{}` is not defined", target.name), "bind it before compound assignment"))?;
                for ix in &target.indices {
                    let it = self.expr(ix)?;
                    t = self.index_result(&t, &it)?;
                }
                let vt = self.expr(&Self::rewrite_pipes(value))?;
                self.unify(&t, &vt)
            }
            Stmt::For { pat, iter, body } => {
                let it = self.expr(&Self::rewrite_pipes(iter))?;
                let elem = self.iter_elem(&it)?;
                self.scoped(|c| {
                    if pat.len() == 1 {
                        c.define(&pat[0], elem);
                    } else {
                        let parts: Vec<Ty> = pat.iter().map(|_| c.fresh()).collect();
                        c.unify(&elem, &Ty::Tuple(parts.clone()))?;
                        for (n, t) in pat.iter().zip(parts) {
                            c.define(n, t);
                        }
                    }
                    for st in body {
                        c.stmt(st)?;
                    }
                    Ok(())
                })
            }
            Stmt::While { cond, body } => {
                let ct = self.expr(&Self::rewrite_pipes(cond))?;
                self.unify(&ct, &Ty::Bool)?;
                self.scoped(|c| {
                    for st in body {
                        c.stmt(st)?;
                    }
                    Ok(())
                })
            }
            Stmt::Ret(e) => {
                if let Some(e) = e {
                    self.expr(&Self::rewrite_pipes(e))?;
                }
                Ok(())
            }
            Stmt::Go(e) => {
                self.expr(&Self::rewrite_pipes(e))?;
                Ok(())
            }
            Stmt::Expr(e) => {
                self.expr(&Self::rewrite_pipes(e))?;
                Ok(())
            }
        }
    }

    fn block(&mut self, stmts: &[Stmt]) -> Res<Ty> {
        self.scoped(|c| {
            let mut last = Ty::Unit;
            for (i, st) in stmts.iter().enumerate() {
                if i == stmts.len() - 1 {
                    if let Stmt::Expr(e) = st {
                        last = c.expr(&Self::rewrite_pipes(e))?;
                        continue;
                    }
                }
                c.stmt(st)?;
            }
            Ok(last)
        })
    }

    fn iter_elem(&mut self, t: &Ty) -> Res<Ty> {
        match self.resolve(t) {
            Ty::List(e) => Ok(*e),
            Ty::Map(k, v) => Ok(Ty::Record { name: None, fields: vec![("k".into(), *k), ("v".into(), *v)] }),
            Ty::Str => Ok(Ty::Str),
            Ty::Any => Ok(Ty::Any),
            Ty::Var(_) => {
                let e = self.fresh();
                self.unify(t, &Ty::List(Box::new(e.clone())))?;
                Ok(e)
            }
            other => Err(err("type_mismatch", &format!("cannot iterate over {other}"), "iterate a list, map, string, or stream")),
        }
    }

    fn index_result(&mut self, t: &Ty, idx: &Ty) -> Res<Ty> {
        match self.resolve(t) {
            Ty::List(e) => {
                self.unify(idx, &Ty::Int)?;
                Ok(*e)
            }
            Ty::Map(k, v) => {
                self.unify(idx, &k)?;
                Ok(*v)
            }
            Ty::Str => Ok(Ty::Str),
            Ty::Any => Ok(Ty::Any),
            // an unbound receiver stays gradual: list/map/str all index
            Ty::Var(_) => Ok(Ty::Any),
            other => Err(err("type_mismatch", &format!("{other} is not indexable"), "index a list, map, or string")),
        }
    }

    // ---- expressions ----

    pub fn expr(&mut self, e: &Expr) -> Res<Ty> {
        match e {
            Expr::Num(n) => Ok(if n.contains('.') { Ty::Float } else { Ty::Int }),
            Expr::Str(_) => Ok(Ty::Str),
            Expr::Bool(_) => Ok(Ty::Bool),
            Expr::Unit => Ok(Ty::Unit),
            Expr::Name(n) => {
                if let Some(t) = self.lookup(n) {
                    Ok(t)
                } else if let Some(t) = self.stdlib_fn(n) {
                    Ok(t)
                } else {
                    Err(err("unknown_name", &format!("`{n}` is not defined"), "bind it first or check the spelling"))
                }
            }
            Expr::TName(n) => self.types.get(n).cloned().ok_or_else(|| err("unknown_name", &format!("type `{n}` is not declared"), "declare it: type Name = {{...}}")),
            Expr::Proj(_) => Ok(Ty::Fn { params: vec![Ty::Any], ret: Box::new(Ty::Any) }),
            Expr::List(items) => {
                // homogeneous literals unify into one element type; mixed
                // literals widen to a union (spec-truth: [7, "ok"] is a valid
                // [int | str] — checker and evaluator must agree)
                let tys: Res<Vec<Ty>> = items.iter().map(|i| self.expr(i)).collect();
                let tys = tys?;
                let elem = self.fresh();
                let mut homogeneous = true;
                for t in &tys {
                    if self.unify(t, &elem).is_err() {
                        homogeneous = false;
                        break;
                    }
                }
                if homogeneous {
                    // empty {} literal is a map; empty [] is a generic list
                    return Ok(Ty::List(Box::new(elem)));
                }
                let mut parts: Vec<Ty> = Vec::new();
                for t in &tys {
                    let r = self.resolve(t);
                    if !parts.iter().any(|p| p.to_string() == r.to_string()) {
                        parts.push(r);
                    }
                }
                Ok(Ty::List(Box::new(Ty::Union(parts))))
            }
            Expr::Tuple(items) => {
                let ts: Res<Vec<Ty>> = items.iter().map(|i| self.expr(i)).collect();
                Ok(Ty::Tuple(ts?))
            }
            Expr::RecordLit { name, fields } => {
                if fields.is_empty() && name.is_none() {
                    // `{}` is the empty map (corpus 18_wordfreq counting style)
                    return Ok(Ty::Map(Box::new(self.fresh()), Box::new(self.fresh())));
                }
                let fts: Res<Vec<(String, Ty)>> = fields.iter().map(|(n, v)| Ok((n.clone(), self.expr(v)?))).collect();
                let lit = Ty::Record { name: name.clone(), fields: fts? };
                if let Some(n) = name {
                    let nominal = self.types.get(n).cloned().ok_or_else(|| err("unknown_name", &format!("type `{n}` is not declared"), "declare it: type Name = {{...}}"))?;
                    self.fits(&lit, &nominal)?;
                    return Ok(nominal);
                }
                Ok(lit)
            }
            Expr::Block(stmts) => self.block(stmts),
            Expr::App { head, args } => self.app(head, args),
            Expr::Lambda { params, body } => {
                let ptys: Vec<Ty> = params.iter().map(|_| self.fresh()).collect();
                for t in &ptys {
                    if let Ty::Var(i) = t {
                        self.param_vars.insert(*i);
                    }
                }
                self.scoped(|c| {
                    for (p, t) in params.iter().zip(&ptys) {
                        c.define(p, t.clone());
                    }
                    let bt = c.expr(body)?;
                    Ok(Ty::Fn { params: ptys.clone(), ret: Box::new(bt) })
                })
            }
            Expr::Field { recv, name } => {
                let rt = self.expr(recv)?;
                self.field(&rt, name)
            }
            Expr::Index { recv, index } => {
                let rt = self.expr(recv)?;
                let it = self.expr(index)?;
                self.index_result(&rt, &it)
            }
            Expr::Slice { recv, lo, hi } => {
                let rt = self.expr(recv)?;
                if let Some(l) = lo {
                    let lt = self.expr(l)?;
                    self.unify(&lt, &Ty::Int)?;
                }
                if let Some(h) = hi {
                    let ht = self.expr(h)?;
                    self.unify(&ht, &Ty::Int)?;
                }
                // slicing preserves the sequence type
                Ok(self.resolve(&rt))
            }
            Expr::Unary { op, expr } => {
                let t = self.expr(expr)?;
                if op == "not" {
                    self.unify(&t, &Ty::Bool)?;
                    Ok(Ty::Bool)
                } else {
                    Ok(self.resolve(&t))
                }
            }
            Expr::Binary { op, lhs, rhs } => {
                let lt = self.expr(lhs)?;
                let rt = self.expr(rhs)?;
                match op.as_str() {
                    "and" | "or" => {
                        self.unify(&lt, &Ty::Bool)?;
                        self.unify(&rt, &Ty::Bool)?;
                        Ok(Ty::Bool)
                    }
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                        self.unify(&lt, &rt)?;
                        Ok(Ty::Bool)
                    }
                    "in" => Ok(Ty::Bool),
                    _ => {
                        // numeric / string ops: operands agree; result follows
                        self.unify(&lt, &rt)?;
                        Ok(self.resolve(&lt))
                    }
                }
            }
            Expr::Pipe { stages } => {
                let mut v = self.expr(&stages[0])?;
                for s in &stages[1..] {
                    v = self.apply_stage(s, v)?;
                }
                Ok(v)
            }
            Expr::Rescue { value, fallback } => {
                let vt = self.expr(value)?;
                // `print x ? y` rescues print's unit and silently discards y
                // (v0.2 whole-expression rescue) — catch it loudly
                if matches!(self.resolve(&vt), Ty::Unit) {
                    return Err(err(
                        "type_mismatch",
                        "rescue on unit — the left side never fails",
                        "rescue the value, not the statement: print (x ? fallback)",
                    ));
                }
                let ft = self.expr(fallback)?;
                // fallback replaces an err/absent value; both sides flow out
                if self.unify(&vt, &ft).is_err() {
                    return Ok(Ty::Any);
                }
                Ok(self.resolve(&vt))
            }
            Expr::Paren(inner) => self.expr(inner),
            Expr::Propagate(inner) => self.expr(inner),
            Expr::If { cond, then, els } => {
                let ct = self.expr(cond)?;
                self.unify(&ct, &Ty::Bool)?;
                let tt = self.block(then)?;
                if let Some(e) = els {
                    let et = self.expr(e)?;
                    let (rt, re) = (self.resolve(&tt), self.resolve(&et));
                    // a PARAMETER's var must not be pinned by branch-merging
                    // (a result var, e.g. a recursive call's return, should be)
                    let is_param = |t: &Ty| matches!(t, Ty::Var(i) if self.param_vars.contains(i));
                    if is_param(&rt) || is_param(&re) {
                        return Ok(Ty::Any);
                    }
                    if self.unify(&rt, &re).is_err() {
                        return Ok(Ty::Any);
                    }
                    Ok(self.resolve(&rt))
                } else {
                    Ok(Ty::Unit)
                }
            }
            Expr::Match { subject, arms } => self.match_expr(subject, arms),
        }
    }

    /// Elaboration rule 1: flat-application arity resolution (SPEC §2.3).
    fn app(&mut self, head: &Expr, args: &[Expr]) -> Res<Ty> {
        // `range` is 1-or-2 ary (SPEC §5: range n / range a b) — bypass the
        // surplus re-nesting that mis-elaborated `range 1 16` (spec-truth)
        if let Expr::Name(n) = head {
            if n == "range" && args.len() == 2 {
                for a in args {
                    let at = self.expr(a)?;
                    self.fits(&at, &Ty::Int)?;
                }
                return Ok(Ty::List(Box::new(Ty::Int)));
            }
        }
        let ht = self.expr(head)?;
        match self.resolve(&ht) {
            Ty::Fn { params, ret } => {
                let k = params.len();
                if args.len() == k {
                    for (a, p) in args.iter().zip(&params) {
                        let at = self.expr(a)?;
                        self.fits(&at, p)?;
                    }
                    Ok(*ret)
                } else if args.len() > k && k >= 1 {
                    // surplus args re-nest under the k-th argument
                    for (a, p) in args[..k - 1].iter().zip(&params) {
                        let at = self.expr(a)?;
                        self.fits(&at, p)?;
                    }
                    let nested = self.app(&args[k - 1], &args[k..])?;
                    self.fits(&nested, &params[k - 1])?;
                    Ok(*ret)
                } else {
                    Err(err(
                        "arity",
                        &format!("function expects {k} argument(s), got {}", args.len()),
                        "curt v0.1 has no partial application — pass all arguments",
                    ))
                }
            }
            Ty::Any | Ty::Var(_) => {
                for a in args {
                    self.expr(a)?;
                }
                Ok(Ty::Any)
            }
            other => Err(err("type_mismatch", &format!("{other} is not callable"), "only functions can be applied")),
        }
    }

    /// Pipe stage: apply with the piped value appended as the final argument.
    fn apply_stage(&mut self, stage: &Expr, piped: Ty) -> Res<Ty> {
        let with_piped = |args: &[Expr]| -> Vec<Expr> { args.to_vec() };
        match stage {
            Expr::App { head, args } => {
                let ht = self.expr(head)?;
                match self.resolve(&ht) {
                    Ty::Fn { params, ret } if params.len() == args.len() + 1 => {
                        for (a, p) in with_piped(args).iter().zip(&params) {
                            let at = self.expr(a)?;
                            self.fits(&at, p)?;
                        }
                        self.fits(&piped, params.last().unwrap())?;
                        Ok(*ret)
                    }
                    Ty::Any | Ty::Var(_) => {
                        for a in args {
                            self.expr(a)?;
                        }
                        Ok(Ty::Any)
                    }
                    other => Err(err("type_mismatch", &format!("pipe stage {other} cannot take the piped value"), "a stage must be a function with one free trailing parameter")),
                }
            }
            // bare verb / lambda / projection stage: unary application
            other => {
                let st = self.expr(other)?;
                match self.resolve(&st) {
                    Ty::Fn { params, ret } if params.len() == 1 => {
                        self.fits(&piped, &params[0])?;
                        Ok(*ret)
                    }
                    Ty::Any | Ty::Var(_) => Ok(Ty::Any),
                    o => Err(err("type_mismatch", &format!("pipe stage {o} is not applicable"), "use a unary function as a pipe stage")),
                }
            }
        }
    }

    fn field(&mut self, recv: &Ty, name: &str) -> Res<Ty> {
        match self.resolve(recv) {
            Ty::Record { fields, name: rname } => fields
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, t)| t.clone())
                .ok_or_else(|| {
                    let near: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                    err(
                        "unknown_field",
                        &format!("no field `{name}` on {}", rname.unwrap_or_else(|| "record".into())),
                        &format!("available fields: {}", near.join(", ")),
                    )
                }),
            Ty::Tuple(parts) => name
                .parse::<usize>()
                .ok()
                .and_then(|i| parts.get(i).cloned())
                .ok_or_else(|| err("unknown_field", &format!("tuple has no element `{name}`"), "use .0, .1, … within bounds")),
            Ty::Any => Ok(Ty::Any),
            recv_t => {
                // UFCS: x.f desugars to f x — try the stdlib verb table
                if let Some(Ty::Fn { params, ret }) = self.stdlib_fn(name) {
                    // receiver is the LAST parameter: x.f a == f a x
                    // (consistent with pipe stages appending the piped value)
                    if let Some(last) = params.last() {
                        self.fits(&recv_t, last)?;
                    }
                    if params.len() == 1 {
                        return Ok(*ret);
                    }
                    // method with leading args: return the remaining-arg fn
                    return Ok(Ty::Fn { params: params[..params.len() - 1].to_vec(), ret });
                }
                if matches!(recv_t, Ty::Var(_)) {
                    return Ok(Ty::Any);
                }
                Err(err("unknown_field", &format!("no field or method `{name}` on {recv_t}"), "check the stdlib verb list in SPEC §9"))
            }
        }
    }

    fn match_expr(&mut self, subject: &Expr, arms: &[(Pattern, Expr)]) -> Res<Ty> {
        let st = self.expr(subject)?;
        let mut subject_ty = self.resolve(&st);
        // an unbound subject takes its type FROM the arms: the union of the
        // matched member types (corpus 06: show v = match v { float x, str s })
        if matches!(subject_ty, Ty::Var(_)) {
            let mut members: Vec<Ty> = Vec::new();
            for (pat, _) in arms {
                let m = match pat {
                    Pattern::TypeBind { ty, .. } => Some(self.type_from(&TypeExpr::Named(ty.clone()))),
                    Pattern::Lit(Expr::Str(_)) => Some(Ty::Str),
                    Pattern::Lit(Expr::Num(n)) => Some(if n.contains('.') { Ty::Float } else { Ty::Int }),
                    _ => None,
                };
                if let Some(m) = m {
                    if !members.contains(&m) {
                        members.push(m);
                    }
                }
            }
            if members.len() > 1 {
                self.unify(&subject_ty, &Ty::Union(members))?;
                subject_ty = self.resolve(&st);
            } else if members.len() == 1 {
                self.unify(&subject_ty, &members[0])?;
                subject_ty = self.resolve(&st);
            }
        }
        let mut remaining: Vec<Ty> = match &subject_ty {
            Ty::Union(parts) => parts.iter().map(|p| self.resolve(p)).collect(),
            _ => Vec::new(),
        };
        let mut has_catchall = false;
        let result = self.fresh();
        for (pat, body) in arms {
            self.scoped(|c| {
                match pat {
                    Pattern::TypeBind { ty, name } => {
                        let nt = c.type_from(&TypeExpr::Named(ty.clone()));
                        // narrowing: binder gets the member type inside the arm
                        c.define(name, nt.clone());
                        remaining.retain(|m| *m != nt);
                    }
                    Pattern::Lit(l) => {
                        let lt = c.expr(l)?;
                        // v0.1 looseness (SPEC note): a literal arm counts as
                        // covering its member type
                        let lt = c.resolve(&lt);
                        remaining.retain(|m| *m != lt);
                        if remaining.is_empty() && !matches!(subject_ty, Ty::Union(_)) {
                            c.unify(&lt, &subject_ty)?;
                        }
                    }
                    Pattern::Tuple(names) => {
                        let parts: Vec<Ty> = names.iter().map(|_| c.fresh()).collect();
                        c.unify(&subject_ty, &Ty::Tuple(parts.clone()))?;
                        for (n, t) in names.iter().zip(parts) {
                            c.define(n, t);
                        }
                        has_catchall = true;
                    }
                    Pattern::Wildcard => has_catchall = true,
                    Pattern::Bind(n) => {
                        c.define(n, subject_ty.clone());
                        has_catchall = true;
                    }
                }
                let bt = c.expr(body)?;
                if c.unify(&bt, &result).is_err() {
                    // heterogeneous arm results degrade gracefully
                    return Ok(());
                }
                Ok(())
            })?;
        }
        if matches!(subject_ty, Ty::Union(_)) && !remaining.is_empty() && !has_catchall {
            let missing: Vec<String> = remaining.iter().map(|t| t.to_string()).collect();
            return Err(err(
                "non_exhaustive_match",
                &format!("match on {subject_ty} does not cover: {}", missing.join(", ")),
                &format!("add an arm `{} x -> ...` or a catch-all `_ -> ...`", missing[0]),
            ));
        }
        Ok(self.resolve(&result))
    }

    fn lint_name(&mut self, name: &str) {
        if name.len() > 3 && token_cost(name) > 1 {
            self.warnings.push(format!("identifier `{name}` costs {} tokens — prefer a single-token name (SPEC §1)", token_cost(name)));
        }
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

/// Does Var(i) occur inside ty? (Guard against cyclic bindings.)
fn occurs(i: usize, ty: &Ty) -> bool {
    match ty {
        Ty::Var(j) => *j == i,
        Ty::List(t) => occurs(i, t),
        Ty::Map(k, v) => occurs(i, k) || occurs(i, v),
        Ty::Tuple(ts) | Ty::Union(ts) => ts.iter().any(|t| occurs(i, t)),
        Ty::Record { fields, .. } => fields.iter().any(|(_, t)| occurs(i, t)),
        Ty::Fn { params, ret } => params.iter().any(|t| occurs(i, t)) || occurs(i, ret),
        _ => false,
    }
}

/// Lazy o200k token cost for the identifier lint (loads ranks on first use).
fn token_cost(name: &str) -> usize {
    use std::sync::OnceLock;
    static BPE: OnceLock<Option<tiktoken_rs::CoreBPE>> = OnceLock::new();
    match BPE.get_or_init(|| tiktoken_rs::o200k_base().ok()) {
        Some(bpe) => bpe.encode_ordinary(&format!(" {name}")).len(),
        None => 1,
    }
}

/// Elaboration rewrite shared by the checker AND the evaluator (interp-d):
/// pipe capture + rescue capture over a preceding juxtaposition.
pub fn rewrite_pipes(e: &Expr) -> Expr {
    rewrite_sugar(e)
}

fn rewrite_sugar(e: &Expr) -> Expr {
    match e {
        // v0.2: NO capture rewrites. `f x | g` pipes the result of `f x`
        // and `f x ? y` rescues the result of `f x` — the whole left
        // expression, like every shipped pipeline language (F#, Elixir).
        // The v0.1 capture-last-argument rule was a measured five-experiment
        // footgun and is deleted, not patched.
        // parens have done their job (grouping + capture barrier) — strip
        Expr::Paren(inner) => rewrite_sugar(inner),
        other => map_expr(other, &rewrite_sugar),
    }
}

/// Structural map over child expressions (used by the pipe rewrite).
fn map_expr(e: &Expr, f: &dyn Fn(&Expr) -> Expr) -> Expr {
    match e {
        Expr::List(items) => Expr::List(items.iter().map(f).collect()),
        Expr::Tuple(items) => Expr::Tuple(items.iter().map(f).collect()),
        Expr::RecordLit { name, fields } => Expr::RecordLit { name: name.clone(), fields: fields.iter().map(|(n, v)| (n.clone(), f(v))).collect() },
        Expr::App { head, args } => Expr::App { head: Box::new(f(head)), args: args.iter().map(f).collect() },
        Expr::Lambda { params, body } => Expr::Lambda { params: params.clone(), body: Box::new(f(body)) },
        Expr::Field { recv, name } => Expr::Field { recv: Box::new(f(recv)), name: name.clone() },
        Expr::Index { recv, index } => Expr::Index { recv: Box::new(f(recv)), index: Box::new(f(index)) },
        Expr::Slice { recv, lo, hi } => Expr::Slice {
            recv: Box::new(f(recv)),
            lo: lo.as_ref().map(|e| Box::new(f(e))),
            hi: hi.as_ref().map(|e| Box::new(f(e))),
        },
        Expr::Unary { op, expr } => Expr::Unary { op: op.clone(), expr: Box::new(f(expr)) },
        Expr::Binary { op, lhs, rhs } => Expr::Binary { op: op.clone(), lhs: Box::new(f(lhs)), rhs: Box::new(f(rhs)) },
        Expr::Rescue { value, fallback } => Expr::Rescue { value: Box::new(f(value)), fallback: Box::new(f(fallback)) },
        Expr::Paren(inner) => Expr::Paren(Box::new(f(inner))),
        Expr::Propagate(inner) => Expr::Propagate(Box::new(f(inner))),
        Expr::If { cond, then, els } => Expr::If { cond: Box::new(f(cond)), then: then.clone(), els: els.as_ref().map(|e| Box::new(f(e))) },
        Expr::Match { subject, arms } => Expr::Match { subject: Box::new(f(subject)), arms: arms.iter().map(|(p, b)| (p.clone(), f(b))).collect() },
        other => other.clone(),
    }
}

/// (equation name, rendered type) pairs plus lint warnings.
pub type CheckReport = (Vec<(String, String)>, Vec<String>);

/// Check a program; returns (equation signatures, warnings).
pub fn check(prog: &[Stmt]) -> Result<CheckReport, Diag> {
    let mut c = Checker::new();
    c.check_program(prog)?;
    let sigs = c.sigs.iter().map(|(n, t)| (n.clone(), c.resolve(t).to_string())).collect();
    Ok((sigs, c.warnings))
}
