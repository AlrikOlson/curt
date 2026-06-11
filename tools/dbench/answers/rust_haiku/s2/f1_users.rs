use std::fs;

fn main() {
    let content = fs::read_to_string("users.txt").unwrap_or_default();
    let mut cnt = 0;
    let mut age_sum = 0;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let Ok(age) = parts[2].parse::<i32>() {
                cnt += 1;
                age_sum += age;
            }
        }
    }

    println!("{}", cnt);
    if cnt > 0 {
        println!("{}", age_sum / cnt);
    } else {
        println!("0");
    }
}
