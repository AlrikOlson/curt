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

fn get_raw_field(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let idx = json.find(&needle)?;
    let rest = json[idx + needle.len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c.is_whitespace()).unwrap_or(rest.len());
    let s = rest[..end].trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn main() {
    let cfg = fs::read_to_string("app.cfg").unwrap_or_default();
    let debug = get_raw_field(&cfg, "debug").as_deref() == Some("true");
    let mode = if debug { "debug" } else { "prod" };
    let host = get_string_field(&cfg, "host").unwrap_or_else(|| "localhost".to_string());
    let port = get_raw_field(&cfg, "port").unwrap_or_else(|| "0".to_string());
    println!("{} {}:{}", mode, host, port);
}
