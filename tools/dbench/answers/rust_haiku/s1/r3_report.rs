use std::fs;

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = json.find(&search) {
        let start = pos + search.len();
        let remainder = &json[start..].trim_start();
        if remainder.starts_with('"') {
            let content = &remainder[1..];
            if let Some(end) = content.find('"') {
                return Some(content[..end].to_string());
            }
        }
    }
    None
}

fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = json.find(&search) {
        let start = pos + search.len();
        let remainder = &json[start..].trim_start();
        let end = remainder
            .find(|c: char| c == ',' || c == '}')
            .unwrap_or(remainder.len());
        remainder[..end].trim().parse::<f64>().ok()
    } else {
        None
    }
}

fn main() {
    let cfg_content = fs::read_to_string("app.cfg").unwrap_or_default();
    let name = extract_json_string(&cfg_content, "name").unwrap_or_default();

    let orders_content = fs::read_to_string("orders.json").unwrap_or_default();
    let mut paid_count = 0;
    let mut paid_total = 0.0;

    let orders_content = orders_content.trim_matches(|c| c == '[' || c == ']');
    for obj in orders_content.split("},{") {
        let obj = obj.trim_matches(|c| c == '{' || c == '}');

        let mut amt = 0.0;
        let mut status = "";

        for part in obj.split(',') {
            let part = part.trim();
            if part.starts_with("\"amt\"") {
                if let Some(val_start) = part.find(':') {
                    let val_str = part[val_start + 1..].trim();
                    amt = val_str.parse::<f64>().unwrap_or(0.0);
                }
            } else if part.starts_with("\"status\"") {
                if let Some(val_start) = part.find(':') {
                    let val_str = part[val_start + 1..].trim().trim_matches('"');
                    status = val_str;
                }
            }
        }

        if status == "paid" {
            paid_count += 1;
            paid_total += amt;
        }
    }

    println!("{}: {} paid, total {}", name, paid_count, paid_total);
}
