use std::fs;

fn json_string(data: &str, k: &str) -> Option<String> {
    let key = format!("\"{}\":", k);
    match data.find(&key) {
        None => None,
        Some(i) => {
            let s = &data[i + key.len()..].trim_start();
            if !s.starts_with('"') {
                return None;
            }
            let inner = &s[1..];
            match inner.find('"') {
                None => None,
                Some(j) => Some(inner[..j].to_string()),
            }
        }
    }
}

fn json_int(data: &str, k: &str) -> Option<i32> {
    let key = format!("\"{}\":", k);
    match data.find(&key) {
        None => None,
        Some(i) => {
            let s = &data[i + key.len()..].trim_start();
            let end = s.find(|c: char| c == ',' || c == '}').unwrap_or(s.len());
            s[..end].trim().parse::<i32>().ok()
        }
    }
}

fn main() {
    let name = fs::read_to_string("missing.cfg")
        .ok()
        .and_then(|c| json_string(&c, "name"))
        .unwrap_or_else(|| "default".to_string());
    println!("{}", name);

    let port = fs::read_to_string("app.cfg")
        .ok()
        .and_then(|c| json_int(&c, "port"))
        .unwrap_or(0);
    println!("{}", port);
}
