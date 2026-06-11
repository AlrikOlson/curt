fn collatz_steps(mut n: u64) -> u64 {
    let mut steps = 0;
    while n != 1 {
        if n % 2 == 0 {
            n /= 2;
        } else {
            n = 3 * n + 1;
        }
        steps += 1;
    }
    steps
}

fn main() {
    let mut max_steps = 0u64;
    let mut max_n = 0u64;
    for n in 1..=10 {
        let steps = collatz_steps(n);
        if steps > max_steps {
            max_steps = steps;
            max_n = n;
        }
    }
    println!("{} {}", max_n, max_steps);
}
