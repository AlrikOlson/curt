use std::fs;

fn main() {
    let content = fs::read_to_string("users.txt").unwrap_or_default();
    let mut max_age = -1;
    let mut max_name = "";

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                if age > max_age {
                    max_age = age;
                    max_name = parts[1];
                }
            }
        }
    }

    println!("{} {}", max_name, max_age);
}
