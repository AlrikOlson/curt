use std::fs;

fn get_json_str(json: &str, key: &str) -> Option<String> {
    let key_pattern = format!("\"{}\":", key);
    match json.find(&key_pattern) {
        None => None,
        Some(idx) => {
            let after_key = &json[idx + key_pattern.len()..];
            let trimmed = after_key.trim_start();
            if trimmed.starts_with('"') {
                let val_part = &trimmed[1..];
                match val_part.find('"') {
                    Some(end) => Some(val_part[..end].to_string()),
                    None => None,
                }
            } else {
                None
            }
        }
    }
}

fn get_json_num(json: &str, key: &str) -> Option<i32> {
    let key_pattern = format!("\"{}\":", key);
    match json.find(&key_pattern) {
        None => None,
        Some(idx) => {
            let after_key = &json[idx + key_pattern.len()..];
            let trimmed = after_key.trim_start();
            let end_pos = trimmed.find(|c: char| c == ',' || c == '}').unwrap_or(trimmed.len());
            trimmed[..end_pos].trim().parse::<i32>().ok()
        }
    }
}

fn main() {
    let missing_name = match fs::read_to_string("missing.cfg") {
        Ok(content) => get_json_str(&content, "name").unwrap_or_else(|| "default".to_string()),
        Err(_) => "default".to_string(),
    };
    println!("{}", missing_name);

    let app_port = match fs::read_to_string("app.cfg") {
        Ok(content) => get_json_num(&content, "port").unwrap_or(0),
        Err(_) => 0,
    };
    println!("{}", app_port);
}
