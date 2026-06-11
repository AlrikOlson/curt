use std::fs;

fn extract_objects(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut buf = String::new();
    for ch in src.chars() {
        match ch {
            '{' => { depth += 1; buf.push(ch); }
            '}' => {
                buf.push(ch);
                depth -= 1;
                if depth == 0 { out.push(buf.clone()); buf.clear(); }
            }
            _ => { if depth > 0 { buf.push(ch); } }
        }
    }
    out
}

fn num_field(obj: &str, key: &str) -> Option<f64> {
    let pat = format!("\"{}\"", key);
    let i = obj.find(&pat)?;
    let rest = &obj[i + pat.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c.is_whitespace()).unwrap_or(rest.len());
    rest[..end].trim().parse().ok()
}

fn main() {
    let src = fs::read_to_string("orders.json").unwrap_or_default();
    let objs = extract_objects(&src);
    let mut entries: Vec<(i64, f64)> = objs.iter()
        .filter_map(|obj| {
            let id = num_field(obj, "id")? as i64;
            let amt = num_field(obj, "amt")?;
            if amt > 25.0 { Some((id, amt)) } else { None }
        })
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let result: Vec<String> = entries.iter().map(|(id, _)| id.to_string()).collect();
    println!("{}", result.join(","));
}
