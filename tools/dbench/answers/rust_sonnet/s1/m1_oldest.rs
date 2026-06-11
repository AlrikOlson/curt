use std::fs;

fn main() {
    let contents = fs::read_to_string("users.txt").unwrap_or_default();
    let mut oldest_name = String::new();
    let mut oldest_age: i64 = i64::MIN;
    let mut found = false;
    for line in contents.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i64>() {
                if age > oldest_age {
                    oldest_age = age;
                    oldest_name = parts[1].to_string();
                    found = true;
                }
            }
        }
    }
    if found {
        println!("{} {}", oldest_name, oldest_age);
    }
}
