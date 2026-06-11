use std::fs;

fn extract_json_bool(json: &str, key: &str) -> Option<bool> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = json.find(&search) {
        let start = pos + search.len();
        let remainder = &json[start..].trim_start();
        if remainder.starts_with("true") {
            return Some(true);
        } else if remainder.starts_with("false") {
            return Some(false);
        }
    }
    None
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = json.find(&search) {
        let start = pos + search.len();
        let remainder = &json[start..].trim_start();
        if remainder.starts_with('"') {
            let content = &remainder[1..];
            if let Some(end) = content.find('"') {
                return Some(content[..end].to_string());
            }
        }
    }
    None
}

fn main() {
    let content = fs::read_to_string("app.cfg").unwrap_or_default();

    let debug = extract_json_bool(&content, "debug").unwrap_or(false);
    let mode = if debug { "debug" } else { "prod" };

    let host = extract_json_string(&content, "host").unwrap_or_else(|| "localhost".to_string());

    let port = if let Some(port_str) = extract_json_string(&content, "port") {
        port_str
    } else {
        "8080".to_string()
    };

    println!("{} {}:{}", mode, host, port);
}
