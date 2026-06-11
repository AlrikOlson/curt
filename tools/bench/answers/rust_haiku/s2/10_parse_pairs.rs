fn main() {
    let input = "a=1,b=22,c=333";
    let pairs: Vec<&str> = input.split(',').collect();

    let mut sum = 0;

    for pair in pairs {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() == 2 {
            let val: i32 = parts[1].parse().unwrap();
            sum += val;
        }
    }

    println!("{}", sum);
}
