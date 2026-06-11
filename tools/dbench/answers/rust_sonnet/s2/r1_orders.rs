use std::fs;

fn parse_field_str<'a>(obj: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\"", key);
    let pos = obj.find(&needle)?;
    let rest = obj[pos + needle.len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn parse_field_num(obj: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{}\"", key);
    let pos = obj.find(&needle)?;
    let rest = obj[pos + needle.len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c.is_whitespace()).unwrap_or(rest.len());
    rest[..end].trim().parse().ok()
}

fn split_objects(json: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let bytes = json.as_bytes();
    let mut depth = 0i32;
    let mut start = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => {
                if depth == 0 { start = Some(i); }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        result.push(&json[s..=i]);
                        start = None;
                    }
                }
            }
            _ => {}
        }
    }
    result
}

fn format_float(f: f64) -> String {
    let s = format!("{:.10}", f);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn main() {
    let contents = fs::read_to_string("orders.json").unwrap_or_default();
    let objects = split_objects(&contents);
    let mut paid_total = 0f64;
    let mut open_count = 0usize;
    for obj in objects {
        let status = parse_field_str(obj, "status").unwrap_or("");
        let amt = parse_field_num(obj, "amt").unwrap_or(0.0);
        match status {
            "paid" => paid_total += amt,
            "open" => open_count += 1,
            _ => {}
        }
    }
    println!("{}", format_float(paid_total));
    println!("{}", open_count);
}
