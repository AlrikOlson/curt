use std::fs;

fn main() {
    let txt = fs::read_to_string("users.txt").unwrap_or_default();
    let mut res = Vec::new();

    for line in txt.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                if age > 25 {
                    res.push(parts[1]);
                }
            }
        }
    }

    println!("{}", res.join(" "));
}
