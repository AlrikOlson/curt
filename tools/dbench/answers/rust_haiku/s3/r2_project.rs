use std::fs;

fn main() {
    let data = fs::read_to_string("orders.json").unwrap_or_default();
    let mut filtered = Vec::new();

    let buf = data.trim_start_matches('[').trim_end_matches(']');
    for item in buf.split("},{") {
        let obj = item.trim_matches(|c| c == '{' || c == '}');
        let mut id = 0;
        let mut amt = 0.0;

        for kv in obj.split(',') {
            let kv = kv.trim();
            if kv.starts_with("\"id\"") {
                if let Some(idx) = kv.find(':') {
                    id = kv[idx + 1..].trim().parse::<i32>().unwrap_or(0);
                }
            } else if kv.starts_with("\"amt\"") {
                if let Some(idx) = kv.find(':') {
                    amt = kv[idx + 1..].trim().parse::<f64>().unwrap_or(0.0);
                }
            }
        }

        if amt > 25.0 {
            filtered.push((id, amt));
        }
    }

    filtered.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let str_ids: Vec<String> = filtered.iter().map(|(i, _)| i.to_string()).collect();
    println!("{}", str_ids.join(","));
}
