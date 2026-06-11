use std::fs;

fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = json.find(&search) {
        let start = pos + search.len();
        let remainder = &json[start..];
        let remainder = remainder.trim_start();
        if remainder.starts_with('"') {
            let content = &remainder[1..];
            if let Some(end) = content.find('"') {
                return Some(content[..end].to_string());
            }
        } else if remainder.starts_with('[') || remainder.starts_with('{') {
            return None;
        } else {
            let end = content
                .find(|c: char| c == ',' || c == '}')
                .unwrap_or(content.len());
            return Some(content[..end].to_string());
        }
    }
    None
}

fn main() {
    let missing_name = if let Ok(content) = fs::read_to_string("missing.cfg") {
        extract_json_value(&content, "name")
            .unwrap_or_else(|| "default".to_string())
    } else {
        "default".to_string()
    };
    println!("{}", missing_name);

    let port = if let Ok(content) = fs::read_to_string("app.cfg") {
        extract_json_value(&content, "port")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0)
    } else {
        0
    };
    println!("{}", port);
}
