enum Val {
    Int(i64),
    Float(f64),
    Str(String),
}

fn describe(v: Val) -> String {
    match v {
        Val::Int(n) => format!("int {}", n + 1),
        Val::Float(f) => format!("float {}", f),
        Val::Str(s) => format!("str {}", s.len()),
    }
}

fn main() {
    println!("{}", describe(Val::Int(42)));
    println!("{}", describe(Val::Float(3.5)));
    println!("{}", describe(Val::Str("abc".to_string())));
}
