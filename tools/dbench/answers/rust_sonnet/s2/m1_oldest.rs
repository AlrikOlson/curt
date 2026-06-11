use std::fs;

fn main() {
    let contents = fs::read_to_string("users.txt").unwrap_or_default();
    let best = contents.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                parts[2].parse::<i64>().ok().map(|age| (parts[1].to_string(), age))
            } else {
                None
            }
        })
        .max_by_key(|(_, age)| *age);
    if let Some((name, age)) = best {
        println!("{} {}", name, age);
    }
}
