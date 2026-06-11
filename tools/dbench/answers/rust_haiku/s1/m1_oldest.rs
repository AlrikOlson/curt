use std::fs;

fn main() {
    let content = fs::read_to_string("users.txt").unwrap_or_default();
    let mut oldest_name = "";
    let mut oldest_age = -1;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                if age > oldest_age {
                    oldest_age = age;
                    oldest_name = parts[1];
                }
            }
        }
    }

    println!("{} {}", oldest_name, oldest_age);
}
