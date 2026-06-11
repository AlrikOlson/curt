fn main() {
    let inputs = ["12", "x", "7", "-", "30"];
    let mut total = 0i64;
    let mut failures = 0usize;
    for s in inputs.iter() {
        match s.parse::<i64>() {
            Ok(v) => total += v,
            Err(_) => failures += 1,
        }
    }
    println!("{}", total);
    println!("{}", failures);
}
