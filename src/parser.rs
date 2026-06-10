//! Recursive-descent parser for cmm v0.1, mirroring tools/tokens/grammar.peg.
//!
//! Contracts carried over from the machine-validated PEG:
//! - flat juxtaposition application (grouping is the elaborator's job, SPEC §2.3)
//! - Go-style header brace rule: in for/while/if/match headers `{` always
//!   begins the block (`in_header` excludes brace-initial atoms/args)
//! - `?` adjacency: glued = postfix propagate, spaced = binary rescue
//! - newline terminates expressions; only blocks/match braces span lines
//! - Postel (parser level): `=` as `==` in expression position, `elif`,
//!   trailing commas, glued `f(x, y)` call sugar

use crate::ast::*;
use crate::diag::Diag;
use crate::lexer::{Tok, Token};

pub fn parse(toks: Vec<Token>) -> Result<Vec<Stmt>, Diag> {
    let mut p = Parser { toks, pos: 0, in_header: false };
    p.program()
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
    in_header: bool,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].tok
    }
    fn glued(&self) -> bool {
        self.toks[self.pos].glued
    }
    fn here(&self) -> (u32, u32) {
        (self.toks[self.pos].line, self.toks[self.pos].col)
    }
    fn bump(&mut self) -> Tok {
        let t = self.toks[self.pos].tok.clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == t {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, t: &Tok, what: &str, fix: &str) -> Result<(), Diag> {
        if self.eat(t) {
            Ok(())
        } else {
            let (l, c) = self.here();
            Err(Diag::at("expected", l, c, &format!("expected {what}, found {:?}", self.peek()), fix))
        }
    }
    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Tok::Newline) {
            self.bump();
        }
    }

    fn program(&mut self) -> Result<Vec<Stmt>, Diag> {
        let mut out = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), Tok::Eof) {
            out.push(self.stmt()?);
            if !matches!(self.peek(), Tok::Eof) {
                if matches!(self.peek(), Tok::Newline | Tok::Semi) {
                    self.bump();
                    self.skip_newlines();
                } else {
                    let (l, c) = self.here();
                    return Err(Diag::at("expected", l, c, &format!("expected end of statement, found {:?}", self.peek()), "separate statements with a newline or ;"));
                }
            }
        }
        Ok(out)
    }

    fn stmt(&mut self) -> Result<Stmt, Diag> {
        match self.peek() {
            Tok::Type => self.type_decl(),
            Tok::Pub => {
                self.bump();
                self.sig(true)
            }
            Tok::For => self.for_stmt(),
            Tok::While => self.while_stmt(),
            Tok::Ret => {
                self.bump();
                if matches!(self.peek(), Tok::Newline | Tok::Semi | Tok::RBrace | Tok::Eof) {
                    Ok(Stmt::Ret(None))
                } else {
                    Ok(Stmt::Ret(Some(self.expr()?)))
                }
            }
            Tok::Go => {
                self.bump();
                Ok(Stmt::Go(self.expr()?))
            }
            _ => {
                if matches!(self.peek(), Tok::Name(_)) && matches!(self.peek_at(1), Tok::DColon) {
                    return self.sig(false);
                }
                if let Some(s) = self.try_equation()? {
                    return Ok(s);
                }
                if let Some(s) = self.try_destructure()? {
                    return Ok(s);
                }
                if let Some(s) = self.try_assignish()? {
                    return Ok(s);
                }
                Ok(Stmt::Expr(self.expr()?))
            }
        }
    }

    fn type_decl(&mut self) -> Result<Stmt, Diag> {
        self.bump(); // type
        let name = match self.bump() {
            Tok::TName(n) => n,
            other => {
                let (l, c) = self.here();
                return Err(Diag::at("expected", l, c, &format!("type name must be Capitalized, found {other:?}"), "use an uppercase type name"));
            }
        };
        self.expect(&Tok::Assign, "=", "type Name = ...")?;
        let ty = self.type_expr()?;
        Ok(Stmt::TypeDecl { name, ty })
    }

    fn sig(&mut self, public: bool) -> Result<Stmt, Diag> {
        let name = match self.bump() {
            Tok::Name(n) => n,
            other => {
                let (l, c) = self.here();
                return Err(Diag::at("expected", l, c, &format!("expected function name, found {other:?}"), "pub name :: types -> ret"));
            }
        };
        self.expect(&Tok::DColon, "::", "name :: int int -> int")?;
        let mut params = vec![self.type_expr()?];
        while self.type_start() {
            params.push(self.type_expr()?);
        }
        let ret = if self.eat(&Tok::Arrow) { Some(self.type_expr()?) } else { None };
        Ok(Stmt::Sig { public, name, params, ret })
    }

    fn type_start(&self) -> bool {
        matches!(self.peek(), Tok::Name(_) | Tok::TName(_) | Tok::LBrace | Tok::LBrack | Tok::LParen)
    }

    fn type_expr(&mut self) -> Result<TypeExpr, Diag> {
        let first = self.type_atom()?;
        if matches!(self.peek(), Tok::Pipe) {
            let mut parts = vec![first];
            while self.eat(&Tok::Pipe) {
                parts.push(self.type_atom()?);
            }
            return Ok(TypeExpr::Union(parts));
        }
        Ok(first)
    }

    fn type_atom(&mut self) -> Result<TypeExpr, Diag> {
        match self.peek().clone() {
            Tok::Name(n) => {
                self.bump();
                Ok(TypeExpr::Named(n))
            }
            Tok::TName(n) => {
                self.bump();
                Ok(TypeExpr::Named(n))
            }
            Tok::LBrack => {
                self.bump();
                let inner = self.type_expr()?;
                self.expect(&Tok::RBrack, "]", "[T]")?;
                Ok(TypeExpr::List(Box::new(inner)))
            }
            Tok::LBrace => {
                self.bump();
                let mut fields = Vec::new();
                loop {
                    let fname = match self.bump() {
                        Tok::Name(n) => n,
                        other => {
                            let (l, c) = self.here();
                            return Err(Diag::at("expected", l, c, &format!("expected field name, found {other:?}"), "{x float, y float}"));
                        }
                    };
                    let fty = self.type_expr()?;
                    fields.push((fname, fty));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                    if matches!(self.peek(), Tok::RBrace) {
                        break; // trailing comma
                    }
                }
                self.expect(&Tok::RBrace, "}", "close the record type")?;
                Ok(TypeExpr::Record(fields))
            }
            Tok::LParen => {
                self.bump();
                let mut params = vec![self.type_expr()?];
                while self.type_start() {
                    params.push(self.type_expr()?);
                }
                self.expect(&Tok::Arrow, "->", "(T T -> R)")?;
                let ret = self.type_expr()?;
                self.expect(&Tok::RParen, ")", "close the function type")?;
                Ok(TypeExpr::Fn { params, ret: Box::new(ret) })
            }
            other => {
                let (l, c) = self.here();
                Err(Diag::at("expected", l, c, &format!("expected a type, found {other:?}"), "int | float | str | [T] | {field T}"))
            }
        }
    }

    fn try_equation(&mut self) -> Result<Option<Stmt>, Diag> {
        let save = self.pos;
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                self.bump();
                n
            }
            _ => return Ok(None),
        };
        let mut params = Vec::new();
        while let Tok::Name(p) = self.peek().clone() {
            self.bump();
            params.push(p);
        }
        if params.is_empty() || !matches!(self.peek(), Tok::Assign) {
            self.pos = save;
            return Ok(None);
        }
        self.bump(); // =
        let body = match self.peek() {
            Tok::LBrace => Body::Block(self.block()?),
            Tok::For => Body::Stmt(Box::new(self.for_stmt()?)),
            Tok::While => Body::Stmt(Box::new(self.while_stmt()?)),
            Tok::Go => {
                self.bump();
                Body::Stmt(Box::new(Stmt::Go(self.expr()?)))
            }
            _ => Body::Expr(self.expr()?),
        };
        Ok(Some(Stmt::Equation { name, params, body }))
    }

    fn try_destructure(&mut self) -> Result<Option<Stmt>, Diag> {
        let save = self.pos;
        if !self.eat(&Tok::LParen) {
            return Ok(None);
        }
        let mut names = Vec::new();
        loop {
            match self.peek().clone() {
                Tok::Name(n) => {
                    self.bump();
                    names.push(n);
                }
                _ => {
                    self.pos = save;
                    return Ok(None);
                }
            }
            if self.eat(&Tok::Comma) {
                continue;
            }
            break;
        }
        if names.len() < 2 || !self.eat(&Tok::RParen) || !self.eat(&Tok::Assign) {
            self.pos = save;
            return Ok(None);
        }
        let value = self.expr()?;
        Ok(Some(Stmt::Destructure { names, value }))
    }

    fn try_assignish(&mut self) -> Result<Option<Stmt>, Diag> {
        let save = self.pos;
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                self.bump();
                n
            }
            _ => return Ok(None),
        };
        let mut indices = Vec::new();
        while matches!(self.peek(), Tok::LBrack) && self.glued() {
            self.bump();
            indices.push(self.expr()?);
            if self.expect(&Tok::RBrack, "]", "close the index").is_err() {
                self.pos = save;
                return Ok(None);
            }
        }
        let ann = if indices.is_empty() && matches!(self.peek(), Tok::Colon) {
            self.bump();
            Some(self.type_expr()?)
        } else {
            None
        };
        let target = Target { name, indices };
        match self.peek() {
            Tok::Assign => {
                self.bump();
                let value = self.expr()?;
                Ok(Some(Stmt::Binding { target, ann, value }))
            }
            Tok::PlusEq | Tok::MinusEq | Tok::StarEq | Tok::SlashEq => {
                let op = match self.bump() {
                    Tok::PlusEq => "+=",
                    Tok::MinusEq => "-=",
                    Tok::StarEq => "*=",
                    Tok::SlashEq => "/=",
                    _ => unreachable!(),
                };
                let value = self.expr()?;
                Ok(Some(Stmt::Compound { target, op: op.into(), value }))
            }
            _ => {
                self.pos = save;
                Ok(None)
            }
        }
    }

    fn for_stmt(&mut self) -> Result<Stmt, Diag> {
        self.bump(); // for
        let pat = if matches!(self.peek(), Tok::LParen) {
            self.bump();
            let mut names = Vec::new();
            loop {
                match self.bump() {
                    Tok::Name(n) => names.push(n),
                    other => {
                        let (l, c) = self.here();
                        return Err(Diag::at("expected", l, c, &format!("expected name in for pattern, found {other:?}"), "for (a, b) in pairs"));
                    }
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RParen, ")", "close the pattern")?;
            names
        } else {
            match self.bump() {
                Tok::Name(n) => vec![n],
                other => {
                    let (l, c) = self.here();
                    return Err(Diag::at("expected", l, c, &format!("expected loop variable, found {other:?}"), "for x in xs { ... }"));
                }
            }
        };
        self.expect(&Tok::In, "in", "for x in xs { ... }")?;
        let iter = self.header_expr()?;
        let body = self.block()?;
        Ok(Stmt::For { pat, iter, body })
    }

    fn while_stmt(&mut self) -> Result<Stmt, Diag> {
        self.bump(); // while
        let cond = self.header_expr()?;
        let body = self.block()?;
        Ok(Stmt::While { cond, body })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Diag> {
        // header flag never leaks into block bodies
        let saved = self.in_header;
        self.in_header = false;
        self.expect(&Tok::LBrace, "{", "open a block")?;
        self.skip_newlines();
        let mut out = Vec::new();
        while !matches!(self.peek(), Tok::RBrace | Tok::Eof) {
            out.push(self.stmt()?);
            if matches!(self.peek(), Tok::Newline | Tok::Semi) {
                self.bump();
                self.skip_newlines();
            } else {
                break;
            }
        }
        self.expect(&Tok::RBrace, "}", "close the block")?;
        self.in_header = saved;
        Ok(out)
    }

    fn header_expr(&mut self) -> Result<Expr, Diag> {
        let saved = self.in_header;
        self.in_header = true;
        let e = self.expr();
        self.in_header = saved;
        e
    }

    // ---- expression precedence ladder (SPEC §2.6) ----

    pub fn expr(&mut self) -> Result<Expr, Diag> {
        let mut lhs = self.pipeline()?;
        while matches!(self.peek(), Tok::Question) && !self.glued() {
            self.bump();
            let rhs = self.pipeline()?;
            lhs = Expr::Rescue { value: Box::new(lhs), fallback: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn pipeline(&mut self) -> Result<Expr, Diag> {
        let first = self.or_expr()?;
        if matches!(self.peek(), Tok::Pipe) {
            let mut stages = vec![first];
            while self.eat(&Tok::Pipe) {
                stages.push(self.or_expr()?);
            }
            return Ok(Expr::Pipe { stages });
        }
        Ok(first)
    }

    fn or_expr(&mut self) -> Result<Expr, Diag> {
        let mut lhs = self.and_expr()?;
        while matches!(self.peek(), Tok::Or) {
            self.bump();
            let rhs = self.and_expr()?;
            lhs = Expr::Binary { op: "or".into(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn and_expr(&mut self) -> Result<Expr, Diag> {
        let mut lhs = self.not_expr()?;
        while matches!(self.peek(), Tok::And) {
            self.bump();
            let rhs = self.not_expr()?;
            lhs = Expr::Binary { op: "and".into(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn not_expr(&mut self) -> Result<Expr, Diag> {
        if matches!(self.peek(), Tok::Not) {
            self.bump();
            let inner = self.not_expr()?;
            return Ok(Expr::Unary { op: "not".into(), expr: Box::new(inner) });
        }
        self.cmp()
    }

    fn cmp(&mut self) -> Result<Expr, Diag> {
        let mut lhs = self.add()?;
        loop {
            let op = match self.peek() {
                Tok::EqEq => "==",
                Tok::Assign => "==", // Postel: `=` in expression position
                Tok::Ne => "!=",
                Tok::Le => "<=",
                Tok::Ge => ">=",
                Tok::Lt => "<",
                Tok::Gt => ">",
                Tok::In => "in",
                _ => break,
            };
            self.bump();
            let rhs = self.add()?;
            lhs = Expr::Binary { op: op.into(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn add(&mut self) -> Result<Expr, Diag> {
        let mut lhs = self.mul()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => "+",
                Tok::Minus => "-",
                _ => break,
            };
            self.bump();
            let rhs = self.mul()?;
            lhs = Expr::Binary { op: op.into(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn mul(&mut self) -> Result<Expr, Diag> {
        let mut lhs = self.pow()?;
        loop {
            let op = match self.peek() {
                Tok::Star => "*",
                Tok::Slash => "/",
                Tok::Percent => "%",
                Tok::Caret => "^",
                _ => break,
            };
            self.bump();
            let rhs = self.pow()?;
            lhs = Expr::Binary { op: op.into(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn pow(&mut self) -> Result<Expr, Diag> {
        let mut lhs = self.unary()?;
        while matches!(self.peek(), Tok::StarStar) {
            self.bump();
            let rhs = self.unary()?;
            lhs = Expr::Binary { op: "**".into(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expr, Diag> {
        if matches!(self.peek(), Tok::Minus) {
            self.bump();
            let inner = self.unary()?;
            return Ok(Expr::Unary { op: "-".into(), expr: Box::new(inner) });
        }
        self.app()
    }

    fn app(&mut self) -> Result<Expr, Diag> {
        let head = self.postfix()?;
        let mut args = Vec::new();
        while self.arg_start() {
            if let Some(lam) = self.try_lambda()? {
                args.push(lam);
            } else {
                args.push(self.postfix()?);
            }
        }
        if args.is_empty() {
            Ok(head)
        } else {
            Ok(Expr::App { head: Box::new(head), args })
        }
    }

    fn arg_start(&self) -> bool {
        match self.peek() {
            Tok::Name(_) | Tok::TName(_) | Tok::Num(_) | Tok::Str(_) | Tok::LParen | Tok::LBrack
            | Tok::Dot | Tok::True | Tok::False | Tok::NoneLit => true,
            Tok::LBrace => !self.in_header,
            _ => false,
        }
    }

    fn try_lambda(&mut self) -> Result<Option<Expr>, Diag> {
        let save = self.pos;
        let mut params = Vec::new();
        while let Tok::Name(n) = self.peek().clone() {
            self.bump();
            params.push(n);
        }
        if params.is_empty() || !matches!(self.peek(), Tok::Arrow) {
            self.pos = save;
            return Ok(None);
        }
        self.bump(); // ->
        let body = self.expr()?;
        Ok(Some(Expr::Lambda { params, body: Box::new(body) }))
    }

    fn postfix(&mut self) -> Result<Expr, Diag> {
        let mut e = self.atom()?;
        loop {
            match self.peek() {
                // field access is GLUED (`x.f`); a spaced `.f` is a projection
                // argument — same adjacency rule the PEG enforces structurally
                Tok::Dot if self.glued() => {
                    self.bump();
                    let name = match self.bump() {
                        Tok::Name(n) => n,
                        Tok::Num(n) => n,
                        other => {
                            let (l, c) = self.here();
                            return Err(Diag::at("expected", l, c, &format!("expected field after '.', found {other:?}"), "x.field or tuple.0"));
                        }
                    };
                    e = Expr::Field { recv: Box::new(e), name };
                }
                Tok::LBrack if self.glued() => {
                    self.bump();
                    if self.eat(&Tok::Colon) {
                        let hi = if matches!(self.peek(), Tok::RBrack) { None } else { Some(Box::new(self.expr()?)) };
                        self.expect(&Tok::RBrack, "]", "close the slice")?;
                        e = Expr::Slice { recv: Box::new(e), lo: None, hi };
                    } else {
                        let first = self.expr()?;
                        if self.eat(&Tok::Colon) {
                            let hi = if matches!(self.peek(), Tok::RBrack) { None } else { Some(Box::new(self.expr()?)) };
                            self.expect(&Tok::RBrack, "]", "close the slice")?;
                            e = Expr::Slice { recv: Box::new(e), lo: Some(Box::new(first)), hi };
                        } else {
                            self.expect(&Tok::RBrack, "]", "close the index")?;
                            e = Expr::Index { recv: Box::new(e), index: Box::new(first) };
                        }
                    }
                }
                Tok::LParen if self.glued() => {
                    // Postel call sugar: f(x, y) -> App(f, [x, y])
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Tok::RParen) {
                        loop {
                            args.push(self.expr()?);
                            if !self.eat(&Tok::Comma) {
                                break;
                            }
                            if matches!(self.peek(), Tok::RParen) {
                                break; // trailing comma
                            }
                        }
                    }
                    self.expect(&Tok::RParen, ")", "close the call")?;
                    e = Expr::App { head: Box::new(e), args };
                }
                Tok::Question if self.glued() => {
                    self.bump();
                    e = Expr::Propagate(Box::new(e));
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn atom(&mut self) -> Result<Expr, Diag> {
        match self.peek().clone() {
            Tok::Match => self.match_expr(),
            Tok::If => self.if_expr(),
            Tok::Num(n) => {
                self.bump();
                Ok(Expr::Num(n))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Expr::Str(s))
            }
            Tok::True => {
                self.bump();
                Ok(Expr::Bool(true))
            }
            Tok::False => {
                self.bump();
                Ok(Expr::Bool(false))
            }
            Tok::NoneLit => {
                self.bump();
                Ok(Expr::Unit)
            }
            Tok::Dot => {
                self.bump();
                match self.bump() {
                    Tok::Name(n) => Ok(Expr::Proj(n)),
                    Tok::Num(n) => Ok(Expr::Proj(n)),
                    other => {
                        let (l, c) = self.here();
                        Err(Diag::at("expected", l, c, &format!("expected field after '.', found {other:?}"), ".field is a projection lambda"))
                    }
                }
            }
            Tok::LBrack => {
                self.bump();
                let mut items = Vec::new();
                if !matches!(self.peek(), Tok::RBrack) {
                    loop {
                        items.push(self.expr()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                        if matches!(self.peek(), Tok::RBrack) {
                            break; // trailing comma
                        }
                    }
                }
                self.expect(&Tok::RBrack, "]", "close the list")?;
                Ok(Expr::List(items))
            }
            Tok::LParen => {
                self.bump();
                // a parenthesized lambda: `(x -> x > 1)` — required for
                // inline pipe stages, since a lambda body extends through `|`
                if let Some(lam) = self.try_lambda()? {
                    self.expect(&Tok::RParen, ")", "close the parenthesis")?;
                    return Ok(lam);
                }
                let first = self.expr()?;
                if self.eat(&Tok::Comma) {
                    let mut items = vec![first];
                    loop {
                        items.push(self.expr()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                        if matches!(self.peek(), Tok::RParen) {
                            break;
                        }
                    }
                    self.expect(&Tok::RParen, ")", "close the tuple")?;
                    Ok(Expr::Tuple(items))
                } else {
                    self.expect(&Tok::RParen, ")", "close the parenthesis")?;
                    Ok(first)
                }
            }
            Tok::LBrace if !self.in_header => {
                // {} -> empty record; {name: ...} -> record; else block
                if matches!(self.peek_at(1), Tok::RBrace) {
                    self.bump();
                    self.bump();
                    return Ok(Expr::RecordLit { name: None, fields: Vec::new() });
                }
                if matches!(self.peek_at(1), Tok::Name(_)) && matches!(self.peek_at(2), Tok::Colon) {
                    self.bump();
                    let fields = self.record_fields()?;
                    return Ok(Expr::RecordLit { name: None, fields });
                }
                Ok(Expr::Block(self.block()?))
            }
            Tok::TName(n) => {
                self.bump();
                if matches!(self.peek(), Tok::LBrace) && self.glued() && !self.in_header {
                    self.bump();
                    let fields = self.record_fields()?;
                    return Ok(Expr::RecordLit { name: Some(n), fields });
                }
                Ok(Expr::TName(n))
            }
            Tok::Name(n) => {
                self.bump();
                Ok(Expr::Name(n))
            }
            other => {
                let (l, c) = self.here();
                Err(Diag::at("expected", l, c, &format!("expected an expression, found {other:?}"), "see SPEC §2 for expression forms"))
            }
        }
    }

    fn record_fields(&mut self) -> Result<Vec<(String, Expr)>, Diag> {
        let mut fields = Vec::new();
        loop {
            let name = match self.bump() {
                Tok::Name(n) => n,
                other => {
                    let (l, c) = self.here();
                    return Err(Diag::at("expected", l, c, &format!("expected field name, found {other:?}"), "{name: value}"));
                }
            };
            self.expect(&Tok::Colon, ":", "field syntax is name: value")?;
            let value = self.expr()?;
            fields.push((name, value));
            if !self.eat(&Tok::Comma) {
                break;
            }
            if matches!(self.peek(), Tok::RBrace) {
                break; // trailing comma
            }
        }
        self.expect(&Tok::RBrace, "}", "close the record")?;
        Ok(fields)
    }

    fn if_expr(&mut self) -> Result<Expr, Diag> {
        self.bump(); // if (or elif's if-part, see below)
        let cond = self.header_expr()?;
        let then = self.block()?;
        let els = if matches!(self.peek(), Tok::Else) {
            self.bump();
            if matches!(self.peek(), Tok::If) {
                Some(Box::new(self.if_expr()?))
            } else {
                Some(Box::new(Expr::Block(self.block()?)))
            }
        } else if matches!(self.peek(), Tok::Elif) {
            // Postel: elif == else if
            Some(Box::new(self.if_expr()?))
        } else {
            None
        };
        Ok(Expr::If { cond: Box::new(cond), then, els })
    }

    fn match_expr(&mut self) -> Result<Expr, Diag> {
        self.bump(); // match
        let subject = self.header_expr()?;
        self.expect(&Tok::LBrace, "{", "match v { pat -> expr, ... }")?;
        self.skip_newlines();
        let mut arms = Vec::new();
        loop {
            let pat = self.pattern()?;
            self.expect(&Tok::Arrow, "->", "arm syntax is pattern -> expression")?;
            let body = if matches!(self.peek(), Tok::LBrace) {
                Expr::Block(self.block()?)
            } else {
                self.expr()?
            };
            arms.push((pat, body));
            let mut more = false;
            if self.eat(&Tok::Comma) {
                more = true;
            }
            if matches!(self.peek(), Tok::Newline) {
                self.skip_newlines();
                more = true;
            }
            if matches!(self.peek(), Tok::RBrace) {
                break;
            }
            if !more {
                break;
            }
        }
        self.expect(&Tok::RBrace, "}", "close the match")?;
        Ok(Expr::Match { subject: Box::new(subject), arms })
    }

    fn pattern(&mut self) -> Result<Pattern, Diag> {
        match self.peek().clone() {
            Tok::Str(s) => {
                self.bump();
                Ok(Pattern::Lit(Expr::Str(s)))
            }
            Tok::Num(n) => {
                self.bump();
                Ok(Pattern::Lit(Expr::Num(n)))
            }
            Tok::LParen => {
                self.bump();
                let mut names = Vec::new();
                loop {
                    match self.bump() {
                        Tok::Name(n) => names.push(n),
                        other => {
                            let (l, c) = self.here();
                            return Err(Diag::at("expected", l, c, &format!("expected name in tuple pattern, found {other:?}"), "(a, b) -> ..."));
                        }
                    }
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RParen, ")", "close the tuple pattern")?;
                Ok(Pattern::Tuple(names))
            }
            Tok::TName(t) => {
                self.bump();
                match self.bump() {
                    Tok::Name(n) => Ok(Pattern::TypeBind { ty: t, name: n }),
                    other => {
                        let (l, c) = self.here();
                        Err(Diag::at("expected", l, c, &format!("expected binder after type pattern, found {other:?}"), "Pt p -> ..."))
                    }
                }
            }
            Tok::Name(n) => {
                self.bump();
                if n == "_" {
                    return Ok(Pattern::Wildcard);
                }
                if let Tok::Name(b) = self.peek().clone() {
                    self.bump();
                    return Ok(Pattern::TypeBind { ty: n, name: b });
                }
                Ok(Pattern::Bind(n))
            }
            other => {
                let (l, c) = self.here();
                Err(Diag::at("expected", l, c, &format!("expected a pattern, found {other:?}"), "patterns: float x, \"lit\", 42, (a, b), _, name"))
            }
        }
    }
}
