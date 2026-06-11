use std::fs;

fn main() {
    let contents = fs::read_to_string("users.txt").unwrap_or_default();
    let adults: Vec<String> = contents.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                if let Ok(age) = parts[2].parse::<i64>() {
                    if age > 25 {
                        return Some(parts[1].to_string());
                    }
                }
            }
            None
        })
        .collect();
    println!("{}", adults.join(" "));
}
