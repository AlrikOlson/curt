use std::fs;

fn main() {
    let txt = fs::read_to_string("users.txt").unwrap_or_default();
    let mut n = 0;
    let mut sum = 0;

    for line in txt.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                n += 1;
                sum += age;
            }
        }
    }

    println!("{}", n);
    if n > 0 {
        println!("{}", sum / n);
    } else {
        println!("0");
    }
}
