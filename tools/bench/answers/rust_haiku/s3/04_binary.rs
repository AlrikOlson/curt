fn main() {
    let nums = [13, 64];

    for &n in &nums {
        let mut num = n;
        let mut binary = String::new();

        if num == 0 {
            binary.push('0');
        } else {
            while num > 0 {
                if num % 2 == 1 {
                    binary.push('1');
                } else {
                    binary.push('0');
                }
                num /= 2;
            }
            binary = binary.chars().rev().collect();
        }

        println!("{}", binary);
    }
}
