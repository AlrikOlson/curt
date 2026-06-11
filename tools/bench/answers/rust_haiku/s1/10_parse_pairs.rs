fn main() {
    let s = "a=1,b=22,c=333";
    let pairs: Vec<&str> = s.split(',').collect();

    let mut sum = 0;

    for pair in pairs.iter() {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() == 2 {
            if let Ok(val) = parts[1].parse::<i32>() {
                sum += val;
            }
        }
    }

    println!("{}", sum);
}
