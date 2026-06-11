fn main() {
    let values = ["12", "x", "7", "-", "30"];
    let mut sum = 0i64;
    let mut bad = 0usize;
    for v in &values {
        match v.parse::<i64>() {
            Ok(n) => sum += n,
            Err(_) => bad += 1,
        }
    }
    println!("{}", sum);
    println!("{}", bad);
}
