use std::fs;

fn extract_objects(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut buf = String::new();
    for ch in src.chars() {
        match ch {
            '{' => {
                depth += 1;
                buf.push(ch);
            }
            '}' => {
                buf.push(ch);
                depth -= 1;
                if depth == 0 {
                    out.push(buf.clone());
                    buf.clear();
                }
            }
            _ => {
                if depth > 0 { buf.push(ch); }
            }
        }
    }
    out
}

fn str_field(obj: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\"", key);
    let i = obj.find(&pat)?;
    let rest = &obj[i + pat.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn num_field(obj: &str, key: &str) -> Option<f64> {
    let pat = format!("\"{}\"", key);
    let i = obj.find(&pat)?;
    let rest = &obj[i + pat.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c.is_whitespace()).unwrap_or(rest.len());
    rest[..end].trim().parse().ok()
}

fn fmt_f64(v: f64) -> String {
    let s = format!("{:.10}", v);
    let s = s.trim_end_matches('0');
    s.trim_end_matches('.').to_string()
}

fn main() {
    let src = fs::read_to_string("orders.json").unwrap_or_default();
    let objs = extract_objects(&src);
    let mut paid_amt = 0f64;
    let mut open_n = 0usize;
    for obj in &objs {
        match str_field(obj, "status").as_deref() {
            Some("paid") => paid_amt += num_field(obj, "amt").unwrap_or(0.0),
            Some("open") => open_n += 1,
            _ => {}
        }
    }
    println!("{}", fmt_f64(paid_amt));
    println!("{}", open_n);
}
