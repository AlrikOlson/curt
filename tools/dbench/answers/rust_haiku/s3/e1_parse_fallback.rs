fn main() {
    let nums = ["12", "x", "7", "-", "30"];
    let mut sum = 0;
    let mut bad = 0;

    for s in &nums {
        match s.parse::<i32>() {
            Ok(n) => sum += n,
            Err(_) => bad += 1,
        }
    }

    println!("{}", sum);
    println!("{}", bad);
}
