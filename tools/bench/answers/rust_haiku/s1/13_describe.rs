fn describe_int(v: i32) -> String {
    format!("int {}", v + 1)
}

fn describe_float(v: f64) -> String {
    format!("float {}", v)
}

fn describe_str(v: &str) -> String {
    format!("str {}", v.len())
}

fn main() {
    println!("{}", describe_int(42));
    println!("{}", describe_float(3.5));
    println!("{}", describe_str("abc"));
}
