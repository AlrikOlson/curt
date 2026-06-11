use std::fs;

fn bool_val(json: &str, k: &str) -> Option<bool> {
    let pat = format!("\"{}\":", k);
    if let Some(i) = json.find(&pat) {
        let s = &json[i + pat.len()..].trim_start();
        if s.starts_with("true") {
            Some(true)
        } else if s.starts_with("false") {
            Some(false)
        } else {
            None
        }
    } else {
        None
    }
}

fn str_val(json: &str, k: &str) -> Option<String> {
    let pat = format!("\"{}\":", k);
    if let Some(i) = json.find(&pat) {
        let s = &json[i + pat.len()..].trim_start();
        if s.starts_with('"') {
            let inner = &s[1..];
            if let Some(q) = inner.find('"') {
                return Some(inner[..q].to_string());
            }
        }
    }
    None
}

fn main() {
    let cfg = fs::read_to_string("app.cfg").unwrap_or_default();

    let debug = bool_val(&cfg, "debug").unwrap_or(false);
    let m = if debug { "debug" } else { "prod" };

    let host = str_val(&cfg, "host").unwrap_or_else(|| "localhost".to_string());
    let port = str_val(&cfg, "port").unwrap_or_else(|| "8080".to_string());

    println!("{} {}:{}", m, host, port);
}
