fn main() {
    let items = [("widget", 4u32, 2.5f64), ("gizmo", 2, 7.25), ("bolt", 10, 0.1)];
    let mut total = 0.0f64;
    let mut max_total = f64::NEG_INFINITY;
    let mut max_name = "";
    for &(name, qty, price) in &items {
        let line = qty as f64 * price;
        total += line;
        if line > max_total {
            max_total = line;
            max_name = name;
        }
    }
    println!("{}", total);
    println!("{}", max_name);
}
