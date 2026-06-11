use std::fs;

fn main() {
    let data = fs::read_to_string("orders.json").unwrap_or_default();
    let mut paid_total = 0.0;
    let mut open_total = 0;

    let buf = data.trim_start_matches('[').trim_end_matches(']');
    for item in buf.split("},{") {
        let obj = item.trim_matches(|c| c == '{' || c == '}');
        let mut amt = 0.0;
        let mut st = "";

        for kv in obj.split(',') {
            let kv = kv.trim();
            if kv.starts_with("\"amt\"") {
                if let Some(idx) = kv.find(':') {
                    amt = kv[idx + 1..].trim().parse::<f64>().unwrap_or(0.0);
                }
            } else if kv.starts_with("\"status\"") {
                if let Some(idx) = kv.find(':') {
                    st = kv[idx + 1..].trim().trim_matches('"');
                }
            }
        }

        if st == "paid" {
            paid_total += amt;
        } else if st == "open" {
            open_total += 1;
        }
    }

    println!("{}", paid_total);
    println!("{}", open_total);
}
