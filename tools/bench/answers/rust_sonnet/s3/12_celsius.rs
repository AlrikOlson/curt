fn main() {
    let temps = [12.5f64, 30.0f64, -5.0f64];
    let max_f = temps.iter().map(|&c| c * 9.0 / 5.0 + 32.0).fold(f64::NEG_INFINITY, f64::max);
    println!("{}", max_f);
}
