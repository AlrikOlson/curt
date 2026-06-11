use std::fs;

fn main() {
    let json = fs::read_to_string("orders.json").unwrap_or_default();
    let mut results = Vec::new();

    let trimmed = json.trim_start_matches('[').trim_end_matches(']');
    for obj_raw in trimmed.split("},{") {
        let obj = obj_raw.trim_matches(|c| c == '{' || c == '}');
        let mut id = 0;
        let mut amt = 0.0;

        for field in obj.split(',') {
            let field = field.trim();
            if field.starts_with("\"id\"") {
                if let Some(colon_pos) = field.find(':') {
                    let num_str = field[colon_pos + 1..].trim();
                    id = num_str.parse::<i32>().unwrap_or(0);
                }
            } else if field.starts_with("\"amt\"") {
                if let Some(colon_pos) = field.find(':') {
                    let num_str = field[colon_pos + 1..].trim();
                    amt = num_str.parse::<f64>().unwrap_or(0.0);
                }
            }
        }

        if amt > 25.0 {
            results.push((id, amt));
        }
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let ids: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    println!("{}", ids.join(","));
}
