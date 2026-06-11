fn main() {
    let values = ["12", "x", "7", "-", "30"];
    let mut sum = 0;
    let mut bad_count = 0;
    for v in &values {
        match v.parse::<i32>() {
            Ok(n) => sum += n,
            Err(_) => bad_count += 1,
        }
    }
    println!("{}", sum);
    println!("{}", bad_count);
}
