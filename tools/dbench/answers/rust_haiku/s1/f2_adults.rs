use std::fs;

fn main() {
    let content = fs::read_to_string("users.txt").unwrap_or_default();
    let mut adults = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                if age > 25 {
                    adults.push(parts[1].to_string());
                }
            }
        }
    }

    println!("{}", adults.join(" "));
}
