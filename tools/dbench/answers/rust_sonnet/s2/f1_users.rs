use std::fs;

fn main() {
    let contents = fs::read_to_string("users.txt").unwrap_or_default();
    let mut valid = 0usize;
    let mut total_age = 0i64;
    for line in contents.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() == 3 {
            if let Ok(age) = fields[2].parse::<i64>() {
                valid += 1;
                total_age += age;
            }
        }
    }
    println!("{}", valid);
    println!("{}", if valid > 0 { total_age / valid as i64 } else { 0 });
}
