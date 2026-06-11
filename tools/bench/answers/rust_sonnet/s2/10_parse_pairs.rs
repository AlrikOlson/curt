fn main() {
    let s = "a=1,b=22,c=333";
    let sum: i64 = s.split(',')
        .map(|pair| pair.split('=').nth(1).unwrap().parse::<i64>().unwrap())
        .sum();
    println!("{}", sum);
}
