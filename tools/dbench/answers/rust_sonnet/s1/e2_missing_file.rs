use std::fs;

fn extract_string_value(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    let pos = json.find(&search)?;
    let after = &json[pos + search.len()..];
    let colon = after.find(':')? ;
    let after_colon = after[colon + 1..].trim_start();
    if after_colon.starts_with('"') {
        let inner = &after_colon[1..];
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else {
        None
    }
}

fn extract_number_value(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    let pos = json.find(&search)?;
    let after = &json[pos + search.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();
    let end = after_colon.find(|c: char| c == ',' || c == '}' || c == '\n' || c == ' ').unwrap_or(after_colon.len());
    let num_str = after_colon[..end].trim();
    if num_str.is_empty() { None } else { Some(num_str.to_string()) }
}

fn main() {
    let name = fs::read_to_string("missing.cfg")
        .ok()
        .and_then(|contents| extract_string_value(&contents, "name"))
        .unwrap_or_else(|| "default".to_string());
    println!("{}", name);

    let port = fs::read_to_string("app.cfg")
        .ok()
        .and_then(|contents| extract_number_value(&contents, "port"))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    println!("{}", port);
}
