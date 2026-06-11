use std::fs;

fn main() {
    let contents = fs::read_to_string("users.txt").unwrap_or_default();
    let mut count = 0usize;
    let mut age_sum = 0i64;
    for line in contents.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i64>() {
                count += 1;
                age_sum += age;
            }
        }
    }
    println!("{}", count);
    if count > 0 {
        println!("{}", age_sum / count as i64);
    } else {
        println!("0");
    }
}
