use std::fs;

fn main() {
    let content = fs::read_to_string("orders.json").unwrap_or_default();
    let mut orders = Vec::new();

    let content = content.trim_matches(|c| c == '[' || c == ']');
    for obj in content.split("},{") {
        let obj = obj.trim_matches(|c| c == '{' || c == '}');

        let mut id = 0;
        let mut amt = 0.0;

        for part in obj.split(',') {
            let part = part.trim();
            if part.starts_with("\"id\"") {
                if let Some(val_start) = part.find(':') {
                    let val_str = part[val_start + 1..].trim();
                    id = val_str.parse::<i32>().unwrap_or(0);
                }
            } else if part.starts_with("\"amt\"") {
                if let Some(val_start) = part.find(':') {
                    let val_str = part[val_start + 1..].trim();
                    amt = val_str.parse::<f64>().unwrap_or(0.0);
                }
            }
        }

        if amt > 25.0 {
            orders.push((id, amt));
        }
    }

    orders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let ids: Vec<String> = orders.iter().map(|o| o.0.to_string()).collect();
    println!("{}", ids.join(","));
}
