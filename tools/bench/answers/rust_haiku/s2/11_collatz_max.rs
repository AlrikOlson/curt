fn main() {
    let mut max_n = 1;
    let mut max_steps = 0;

    for n in 1..=10 {
        let mut current = n;
        let mut steps = 0;

        while current != 1 {
            if current % 2 == 0 {
                current /= 2;
            } else {
                current = 3 * current + 1;
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
