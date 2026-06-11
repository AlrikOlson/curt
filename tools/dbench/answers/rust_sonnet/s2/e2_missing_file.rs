use std::fs;

fn get_string_field(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let idx = json.find(&needle)?;
    let rest = json[idx + needle.len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn get_number_field(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let idx = json.find(&needle)?;
    let rest = json[idx + needle.len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c.is_whitespace()).unwrap_or(rest.len());
    let s = rest[..end].trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn main() {
    let name = fs::read_to_string("missing.cfg")
        .ok()
        .and_then(|s| get_string_field(&s, "name"))
        .unwrap_or_else(|| "default".to_string());
    println!("{}", name);

    let port = fs::read_to_string("app.cfg")
        .ok()
        .and_then(|s| get_number_field(&s, "port"))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    println!("{}", port);
}
