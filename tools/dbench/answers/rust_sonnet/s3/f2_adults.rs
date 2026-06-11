use std::fs;

fn main() {
    let text = fs::read_to_string("users.txt").unwrap_or_default();
    let names: Vec<&str> = text.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                if let Ok(age) = parts[2].parse::<i64>() {
                    if age > 25 { return Some(parts[1]); }
                }
            }
            None
        })
        .collect();
    println!("{}", names.join(" "));
}
