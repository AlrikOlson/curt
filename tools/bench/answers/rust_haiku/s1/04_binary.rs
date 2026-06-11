fn main() {
    let nums = [13, 64];

    for num in nums.iter() {
        let mut n = *num;
        let mut binary = String::new();

        if n == 0 {
            binary.push('0');
        } else {
            while n > 0 {
                if n % 2 == 1 {
                    binary.insert(0, '1');
                } else {
                    binary.insert(0, '0');
                }
                n /= 2;
            }
        }

        println!("{}", binary);
    }
}
