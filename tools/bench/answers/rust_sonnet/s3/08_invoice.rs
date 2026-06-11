fn main() {
    let items = [("widget", 4u32, 2.5f64), ("gizmo", 2u32, 7.25f64), ("bolt", 10u32, 0.1f64)];
    let mut total = 0.0f64;
    let mut max_line = 0.0f64;
    let mut max_name = "";
    for &(name, qty, price) in &items {
        let line = qty as f64 * price;
        total += line;
        if line > max_line {
            max_line = line;
            max_name = name;
        }
    }
    println!("{}", total);
    println!("{}", max_name);
}
