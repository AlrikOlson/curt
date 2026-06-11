fn to_binary(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut bits = Vec::new();
    while n > 0 {
        bits.push((n % 2).to_string());
        n /= 2;
    }
    bits.reverse();
    bits.join("")
}

fn main() {
    println!("{}", to_binary(13));
    println!("{}", to_binary(64));
}
