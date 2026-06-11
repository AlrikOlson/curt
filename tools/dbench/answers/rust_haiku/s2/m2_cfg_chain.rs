use std::fs;

fn get_bool(json: &str, key: &str) -> Option<bool> {
    let pat = format!("\"{}\":", key);
    if let Some(pos) = json.find(&pat) {
        let rest = &json[pos + pat.len()..].trim_start();
        if rest.starts_with("true") {
            Some(true)
        } else if rest.starts_with("false") {
            Some(false)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_str(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\":", key);
    if let Some(pos) = json.find(&pat) {
        let rest = &json[pos + pat.len()..].trim_start();
        if rest.starts_with('"') {
            let val = &rest[1..];
            if let Some(end) = val.find('"') {
                return Some(val[..end].to_string());
            }
        }
    }
    None
}

fn main() {
    let cfg = fs::read_to_string("app.cfg").unwrap_or_default();

    let is_debug = get_bool(&cfg, "debug").unwrap_or(false);
    let mode = if is_debug { "debug" } else { "prod" };

    let host = get_str(&cfg, "host").unwrap_or_else(|| "localhost".to_string());
    let port = get_str(&cfg, "port").unwrap_or_else(|| "8080".to_string());

    println!("{} {}:{}", mode, host, port);
}
