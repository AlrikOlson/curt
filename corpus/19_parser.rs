use std::env;

#[derive(Clone, PartialEq)]
enum Tok {
    Num(f64),
    Sym(String),
}

fn lex(s: &str) -> Vec<Tok> {
    let s = s.replace('(', " ( ").replace(')', " ) ");
    s.split_whitespace()
        .map(|w| match w.parse::<f64>() {
            Ok(n) => Tok::Num(n),
            Err(_) => Tok::Sym(w.to_string()),
        })
        .collect()
}

fn expr(ts: &[Tok]) -> (f64, &[Tok]) {
    let (mut v, mut r) = term(ts);
    while let Some(Tok::Sym(op)) = r.first() {
        if op != "+" && op != "-" {
            break;
        }
        let (v2, r2) = term(&r[1..]);
        v = if op == "+" { v + v2 } else { v - v2 };
        r = r2;
    }
    (v, r)
}

fn term(ts: &[Tok]) -> (f64, &[Tok]) {
    let (mut v, mut r) = factor(ts);
    while let Some(Tok::Sym(op)) = r.first() {
        if op != "*" && op != "/" {
            break;
        }
        let (v2, r2) = factor(&r[1..]);
        v = if op == "*" { v * v2 } else { v / v2 };
        r = r2;
    }
    (v, r)
}

fn factor(ts: &[Tok]) -> (f64, &[Tok]) {
    match &ts[0] {
        Tok::Num(n) => (*n, &ts[1..]),
        Tok::Sym(_) => {
            let (v, r) = expr(&ts[1..]);
            (v, &r[1..])
        }
    }
}

fn main() {
    let arg = env::args().nth(1).unwrap();
    let (v, _) = expr(&lex(&arg));
    println!("{v}");
}
