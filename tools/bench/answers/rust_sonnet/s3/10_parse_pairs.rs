fn main() {
    let s = "a=1,b=22,c=333";
    let sum: i64 = s.split(',').map(|pair| {
        let parts: Vec<&str> = pair.split('=').collect();
        parts[1].parse::<i64>().unwrap()
    }).sum();
    println!("{}", sum);
}
