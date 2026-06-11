fn main() {
    for num in [13, 64] {
        let mut n = num;
        let mut bits = String::new();

        if n == 0 {
            bits.push('0');
        } else {
            while n > 0 {
                bits.push(if n % 2 == 1 { '1' } else { '0' });
                n /= 2;
            }
            bits = bits.chars().rev().collect();
        }

        println!("{}", bits);
    }
}
