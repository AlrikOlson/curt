use std::fs;

fn main() {
    let content = fs::read_to_string("users.txt").unwrap_or_default();
    let mut count = 0;
    let mut sum_age = 0;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                count += 1;
                sum_age += age;
            }
        }
    }

    println!("{}", count);
    if count > 0 {
        println!("{}", sum_age / count);
    } else {
        println!("0");
    }
}
