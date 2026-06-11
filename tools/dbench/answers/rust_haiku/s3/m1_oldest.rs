use std::fs;

fn main() {
    let txt = fs::read_to_string("users.txt").unwrap_or_default();
    let mut best_age = -1;
    let mut best_name = "";

    for line in txt.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                if age > best_age {
                    best_age = age;
                    best_name = parts[1];
                }
            }
        }
    }

    println!("{} {}", best_name, best_age);
}
