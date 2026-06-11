fn main() {
    let list = [4, 8, 15, 16, 23];
    let mut min = list[0];
    let mut max = list[0];
    let mut sum = 0;

    for &num in &list {
        if num < min {
            min = num;
        }
        if num > max {
            max = num;
        }
        sum += num;
    }

    let mean = sum as f64 / list.len() as f64;

    println!("{}", min);
    println!("{}", max);
    println!("{}", mean);
}
