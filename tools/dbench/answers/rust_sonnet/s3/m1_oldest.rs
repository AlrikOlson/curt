use std::fs;

fn main() {
    let text = fs::read_to_string("users.txt").unwrap_or_default();
    let mut oldest: Option<(String, i64)> = None;
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i64>() {
                let is_older = oldest.as_ref().map_or(true, |(_, prev)| age > *prev);
                if is_older {
                    oldest = Some((parts[1].to_string(), age));
                }
            }
        }
    }
    if let Some((name, age)) = oldest {
        println!("{} {}", name, age);
    }
}
