use std::fs;

fn extract_str(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\":", key);
    if let Some(pos) = json.find(&pat) {
        let after = &json[pos + pat.len()..].trim_start();
        if after.starts_with('"') {
            let content = &after[1..];
            if let Some(end) = content.find('"') {
                return Some(content[..end].to_string());
            }
        }
    }
    None
}

fn main() {
    let cfg = fs::read_to_string("app.cfg").unwrap_or_default();
    let name = extract_str(&cfg, "name").unwrap_or_default();

    let orders_json = fs::read_to_string("orders.json").unwrap_or_default();
    let mut paid_count = 0;
    let mut paid_sum = 0.0;

    let trimmed = orders_json.trim_start_matches('[').trim_end_matches(']');
    for obj_raw in trimmed.split("},{") {
        let obj = obj_raw.trim_matches(|c| c == '{' || c == '}');
        let mut amt = 0.0;
        let mut status = "";

        for field in obj.split(',') {
            let f = field.trim();
            if f.starts_with("\"amt\"") {
                if let Some(colon) = f.find(':') {
                    amt = f[colon + 1..].trim().parse::<f64>().unwrap_or(0.0);
                }
            } else if f.starts_with("\"status\"") {
                if let Some(colon) = f.find(':') {
                    status = f[colon + 1..].trim().trim_matches('"');
                }
            }
        }

        if status == "paid" {
            paid_count += 1;
            paid_sum += amt;
        }
    }

    println!("{}: {} paid, total {}", name, paid_count, paid_sum);
}
