use std::fs;

fn main() {
    let content = fs::read_to_string("users.txt").unwrap_or_default();
    let mut names = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                if age > 25 {
                    names.push(parts[1]);
                }
            }
        }
    }

    println!("{}", names.join(" "));
}
