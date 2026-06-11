use std::fs;

fn read_str_key(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\"", key);
    let start = json.find(&pat)?;
    let tail = &json[start + pat.len()..];
    let colon = tail.find(':')?;
    let vstart = tail[colon + 1..].trim_start();
    let vstart = vstart.strip_prefix('"')?;
    let end = vstart.find('"')?;
    Some(vstart[..end].to_string())
}

fn read_num_key(json: &str, key: &str) -> Option<i64> {
    let pat = format!("\"{}\"", key);
    let start = json.find(&pat)?;
    let tail = &json[start + pat.len()..];
    let colon = tail.find(':')?;
    let vstr = tail[colon + 1..].trim_start();
    let end = vstr.find(|c: char| c == ',' || c == '}' || c.is_ascii_whitespace()).unwrap_or(vstr.len());
    vstr[..end].trim().parse().ok()
}

fn main() {
    let name = fs::read_to_string("missing.cfg")
        .ok()
        .and_then(|s| read_str_key(&s, "name"))
        .unwrap_or_else(|| "default".to_string());
    println!("{}", name);

    let port = fs::read_to_string("app.cfg")
        .ok()
        .and_then(|s| read_num_key(&s, "port"))
        .unwrap_or(0);
    println!("{}", port);
}
