fn main() {
    let nums = [4, 8, 15, 16, 23];
    let min = nums.iter().min().unwrap();
    let max = nums.iter().max().unwrap();
    let sum: i64 = nums.iter().sum();
    let mean = sum as f64 / nums.len() as f64;
    println!("{}", min);
    println!("{}", max);
    println!("{}", mean);
}
