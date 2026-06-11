use std::fs;

fn main() {
    let text = fs::read_to_string("users.txt").unwrap_or_default();
    let users: Vec<i64> = text.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 { parts[2].parse().ok() } else { None }
        })
        .collect();
    let count = users.len();
    let avg = if count > 0 { users.iter().sum::<i64>() / count as i64 } else { 0 };
    println!("{}", count);
    println!("{}", avg);
}
