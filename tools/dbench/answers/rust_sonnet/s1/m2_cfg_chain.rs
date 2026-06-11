use std::fs;

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

fn extract_raw_value(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    let pos = json.find(&search)?;
    let after = &json[pos + search.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();
    let end = after_colon.find(|c: char| c == ',' || c == '}' || c == '\n').unwrap_or(after_colon.len());
    Some(after_colon[..end].trim().to_string())
}

fn main() {
    let cfg = fs::read_to_string("app.cfg").unwrap_or_default();
    let debug_val = extract_raw_value(&cfg, "debug").unwrap_or_default();
    let mode = if debug_val == "true" { "debug" } else { "prod" };
    let host = extract_string_value(&cfg, "host").unwrap_or_else(|| "localhost".to_string());
    let port = extract_raw_value(&cfg, "port").unwrap_or_else(|| "0".to_string());
    println!("{} {}:{}", mode, host, port);
}
