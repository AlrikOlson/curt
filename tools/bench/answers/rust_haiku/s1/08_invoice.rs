fn main() {
    let items = [
        ("widget", 4, 2.5),
        ("gizmo", 2, 7.25),
        ("bolt", 10, 0.1),
    ];

    let mut total = 0.0;
    let mut max_item = "";
    let mut max_line_total = 0.0;

    for (name, qty, price) in items.iter() {
        let line_total = (*qty as f64) * price;
        total += line_total;

        if line_total > max_line_total {
            max_line_total = line_total;
            max_item = name;
        }
    }

    println!("{}", total);
    println!("{}", max_item);
}
