fn main() {
    let items = [("widget", 4u32, 2.5f64), ("gizmo", 2, 7.25), ("bolt", 10, 0.1)];
    let totals: Vec<f64> = items.iter().map(|&(_, qty, price)| qty as f64 * price).collect();
    let grand_total: f64 = totals.iter().sum();
    let max_idx = totals.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
    println!("{}", grand_total);
    println!("{}", items[max_idx].0);
}
