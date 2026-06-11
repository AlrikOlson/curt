use std::fs;

fn str_field(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\"", key);
    let i = json.find(&pat)?;
    let rest = &json[i + pat.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn raw_field(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\"", key);
    let i = json.find(&pat)?;
    let rest = &json[i + pat.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c.is_ascii_whitespace()).unwrap_or(rest.len());
    let s = rest[..end].trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn main() {
    let cfg = fs::read_to_string("app.cfg").unwrap_or_default();
    let debug = raw_field(&cfg, "debug").as_deref() == Some("true");
    let mode = if debug { "debug" } else { "prod" };
    let host = str_field(&cfg, "host").unwrap_or_else(|| "localhost".to_string());
    let port = raw_field(&cfg, "port").unwrap_or_else(|| "0".to_string());
    println!("{} {}:{}", mode, host, port);
}
