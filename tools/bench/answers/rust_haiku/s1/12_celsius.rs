fn main() {
    let temps = [12.5, 30.0, -5.0];

    let mut max_f = (temps[0] * 9.0 / 5.0) + 32.0;

    for temp in temps.iter() {
        let f = (temp * 9.0 / 5.0) + 32.0;
        if f > max_f {
            max_f = f;
        }
    }

    println!("{}", max_f);
}
