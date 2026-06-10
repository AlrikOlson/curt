use std::fs;

fn load(p: &str) -> Option<serde_json::Value> {
    let data = fs::read_to_string(p).ok()?;
    serde_json::from_str(&data).ok()
}

fn main() {
    let cfg = load("app.cfg").unwrap_or_else(|| serde_json::json!({}));
    let port = cfg.get("port").cloned().unwrap_or(serde_json::json!(8080));
    println!("{port}");
}
