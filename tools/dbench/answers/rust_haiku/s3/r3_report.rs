use std::fs;

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\":", key);
    if let Some(pos) = json.find(&pat) {
        let after = &json[pos + pat.len()..].trim_start();
        if after.starts_with('"') {
            let val = &after[1..];
            if let Some(q) = val.find('"') {
                return Some(val[..q].to_string());
            }
        }
    }
    None
}

fn main() {
    let cfg_data = fs::read_to_string("app.cfg").unwrap_or_default();
    let svc_name = extract_json_string(&cfg_data, "name").unwrap_or_default();

    let ord_data = fs::read_to_string("orders.json").unwrap_or_default();
    let mut p_count = 0;
    let mut p_amt = 0.0;

    let buf = ord_data.trim_start_matches('[').trim_end_matches(']');
    for item in buf.split("},{") {
        let obj = item.trim_matches(|c| c == '{' || c == '}');
        let mut amt = 0.0;
        let mut st = "";

        for kv in obj.split(',') {
            let kv = kv.trim();
            if kv.starts_with("\"amt\"") {
                if let Some(i) = kv.find(':') {
                    amt = kv[i + 1..].trim().parse::<f64>().unwrap_or(0.0);
                }
            } else if kv.starts_with("\"status\"") {
                if let Some(i) = kv.find(':') {
                    st = kv[i + 1..].trim().trim_matches('"');
                }
            }
        }

        if st == "paid" {
            p_count += 1;
            p_amt += amt;
        }
    }

    println!("{}: {} paid, total {}", svc_name, p_count, p_amt);
}
