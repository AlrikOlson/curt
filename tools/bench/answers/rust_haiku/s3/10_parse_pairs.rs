fn main() {
    let s = "a=1,b=22,c=333";
    let pairs: Vec<&str> = s.split(',').collect();
    let mut sum = 0;

    for pair in pairs {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() == 2 {
            let value: i32 = parts[1].parse().unwrap_or(0);
            sum += value;
        }
    }

    println!("{}", sum);
}
