use std::fs;

fn main() {
    let content = fs::read_to_string("orders.json").unwrap_or_default();
    let mut paid_total = 0.0;
    let mut open_count = 0;

    let content = content.trim_matches(|c| c == '[' || c == ']');
    for obj in content.split("},{") {
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
            paid_total += amt;
        } else if status == "open" {
            open_count += 1;
        }
    }

    println!("{}", paid_total);
    println!("{}", open_count);
}
