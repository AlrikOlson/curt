use std::fs;
use std::collections::HashMap;

fn extract_string_value(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    let pos = json.find(&search)?;
    let after = &json[pos + search.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();
    if after_colon.starts_with('"') {
        let inner = &after_colon[1..];
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else {
        None
    }
}

fn parse_objects(json: &str) -> Vec<HashMap<String, String>> {
    let mut objects = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    let chars: Vec<char> = json.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => {
                if depth == 0 { start = Some(i); }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        let obj_str: String = chars[s..=i].iter().collect();
                        objects.push(parse_single_object(&obj_str));
                        start = None;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    objects
}

fn parse_single_object(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let inner = s.trim_start_matches('{').trim_end_matches('}');
    let mut remaining = inner;
    while !remaining.trim().is_empty() {
        let trimmed = remaining.trim();
        if !trimmed.starts_with('"') { break; }
        let after_quote = &trimmed[1..];
        let key_end = match after_quote.find('"') {
            Some(p) => p,
            None => break,
        };
        let key = after_quote[..key_end].to_string();
        let after_key = &after_quote[key_end + 1..];
        let colon_pos = match after_key.find(':') {
            Some(p) => p,
            None => break,
        };
        let value_part = after_key[colon_pos + 1..].trim_start();
        let (value, consumed_len) = if value_part.starts_with('"') {
            let inner_val = &value_part[1..];
            let val_end = inner_val.find('"').unwrap_or(inner_val.len());
            let val = inner_val[..val_end].to_string();
            let consumed = (value_part.as_ptr() as usize - remaining.as_ptr() as usize) + 1 + val_end + 1;
            (val, consumed)
        } else {
            let end = value_part.find(|c: char| c == ',' || c == '}').unwrap_or(value_part.len());
            let val = value_part[..end].trim().to_string();
            let consumed = (value_part.as_ptr() as usize - remaining.as_ptr() as usize) + end;
            (val, consumed)
        };
        map.insert(key, value);
        if consumed_len >= remaining.len() { break; }
        remaining = &remaining[consumed_len..];
        let next = remaining.find(|c: char| c == ',' || c == '"');
        match next {
            Some(p) => {
                let ch = remaining.chars().nth(p).unwrap();
                if ch == ',' {
                    remaining = &remaining[p + 1..];
                } else {
                    remaining = &remaining[p..];
                }
            }
            None => break,
        }
    }
    map
}

fn format_float(f: f64) -> String {
    let s = format!("{:.10}", f);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn main() {
    let cfg = fs::read_to_string("app.cfg").unwrap_or_default();
    let name = extract_string_value(&cfg, "name").unwrap_or_else(|| "unknown".to_string());

    let orders_json = fs::read_to_string("orders.json").unwrap_or_default();
    let objects = parse_objects(&orders_json);
    let mut paid_count = 0usize;
    let mut paid_total = 0f64;
    for obj in &objects {
        if obj.get("status").map(|s| s.as_str()) == Some("paid") {
            paid_count += 1;
            paid_total += obj.get("amt").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        }
    }
    println!("{}: {} paid, total {}", name, paid_count, format_float(paid_total));
}
