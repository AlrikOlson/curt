fn main() {
    let mut max_n = 1;
    let mut max_steps = 0;

    for n in 1..=10 {
        let mut num = n;
        let mut steps = 0;

        while num != 1 {
            if num % 2 == 0 {
                num /= 2;
            } else {
                num = 3 * num + 1;
            }
            steps += 1;
        }

        if steps > max_steps {
            max_steps = steps;
            max_n = n;
        }
    }

    println!("{} {}", max_n, max_steps);
}
