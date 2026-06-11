fn main() {
    let values = ["12", "x", "7", "-", "30"];
    let mut sum = 0i64;
    let mut unparseable = 0usize;
    for s in &values {
        match s.parse::<i64>() {
            Ok(n) => sum += n,
            Err(_) => unparseable += 1,
        }
    }
    println!("{}", sum);
    println!("{}", unparseable);
}
