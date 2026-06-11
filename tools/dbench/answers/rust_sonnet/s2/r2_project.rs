use std::fs;

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

fn main() {
    let contents = fs::read_to_string("orders.json").unwrap_or_default();
    let objects = split_objects(&contents);
    let mut qualifying: Vec<(i64, f64)> = objects.iter()
        .filter_map(|obj| {
            let id = parse_field_num(obj, "id")? as i64;
            let amt = parse_field_num(obj, "amt")?;
            if amt > 25.0 { Some((id, amt)) } else { None }
        })
        .collect();
    qualifying.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let ids: Vec<String> = qualifying.iter().map(|(id, _)| id.to_string()).collect();
    println!("{}", ids.join(","));
}
