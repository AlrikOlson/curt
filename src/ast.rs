//! AST for curt v0.1. `curt parse` prints this via {:#?}; later phases
//! (fmt/expand/infer/eval) consume it.

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    TypeDecl { name: String, ty: TypeExpr },
    Sig { public: bool, name: String, params: Vec<TypeExpr>, ret: Option<TypeExpr> },
    Equation { name: String, params: Vec<String>, body: Body },
    Destructure { names: Vec<String>, value: Expr },
    Binding { target: Target, ann: Option<TypeExpr>, value: Expr },
    Compound { target: Target, op: String, value: Expr },
    For { pat: Vec<String>, iter: Expr, body: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    Ret(Option<Expr>),
    Go(Expr),
    Expr(Expr),
}

/// Equation bodies may be a single statement (unit-valued) per SPEC §2.2.
#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    Expr(Expr),
    Block(Vec<Stmt>),
    Stmt(Box<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub name: String,
    pub indices: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Named(String),
    Union(Vec<TypeExpr>),
    Record(Vec<(String, TypeExpr)>),
    List(Box<TypeExpr>),
    Fn { params: Vec<TypeExpr>, ret: Box<TypeExpr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(String),
    Str(String),
    Bool(bool),
    Unit,
    Name(String),
    TName(String),
    /// bare `.field` / `.0` projection lambda (SPEC §2.9)
    Proj(String),
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    RecordLit { name: Option<String>, fields: Vec<(String, Expr)> },
    Block(Vec<Stmt>),
    /// flat juxtaposition application; grouping resolved by inference (§2.3)
    App { head: Box<Expr>, args: Vec<Expr> },
    Lambda { params: Vec<String>, body: Box<Expr> },
    Field { recv: Box<Expr>, name: String },
    Index { recv: Box<Expr>, index: Box<Expr> },
    Slice { recv: Box<Expr>, lo: Option<Box<Expr>>, hi: Option<Box<Expr>> },
    Unary { op: String, expr: Box<Expr> },
    Binary { op: String, lhs: Box<Expr>, rhs: Box<Expr> },
    Pipe { stages: Vec<Expr> },
    /// explicit parentheses — a grouping barrier for the pipe/rescue capture
    /// rewrite (SPEC §2.3); stripped by `rewrite_pipes` before eval/check
    Paren(Box<Expr>),
    /// spaced binary `a ? b`
    Rescue { value: Box<Expr>, fallback: Box<Expr> },
    /// glued postfix `x?`
    Propagate(Box<Expr>),
    If { cond: Box<Expr>, then: Vec<Stmt>, els: Option<Box<Expr>> },
    Match { subject: Box<Expr>, arms: Vec<(Pattern, Expr)> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `float x` / `Pt p` — type-narrowing bind
    TypeBind { ty: String, name: String },
    Lit(Expr),
    Tuple(Vec<String>),
    Wildcard,
    Bind(String),
}
