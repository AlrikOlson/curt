fn main() {
    let list = [4, 8, 15, 16, 23];
    let min = list.iter().copied().min().unwrap();
    let max = list.iter().copied().max().unwrap();
    let sum: i64 = list.iter().sum();
    let mean = sum as f64 / list.len() as f64;
    println!("{}", min);
    println!("{}", max);
    println!("{}", mean);
}
