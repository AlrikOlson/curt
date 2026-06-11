fn main() {
    let mut a = 252u64;
    let mut b = 105u64;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    println!("{}", a);
}
