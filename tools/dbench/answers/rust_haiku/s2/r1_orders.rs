use std::fs;

fn main() {
    let json = fs::read_to_string("orders.json").unwrap_or_default();
    let mut paid_amt = 0.0;
    let mut open_cnt = 0;

    let trimmed = json.trim_start_matches('[').trim_end_matches(']');
    for obj_raw in trimmed.split("},{") {
        let obj = obj_raw.trim_matches(|c| c == '{' || c == '}' || c == '[' || c == ']');
        let mut amt = 0.0;
        let mut status = "";

        for field in obj.split(',') {
            let field = field.trim();
            if field.starts_with("\"amt\"") {
                if let Some(colon_pos) = field.find(':') {
                    let num_str = field[colon_pos + 1..].trim();
                    amt = num_str.parse::<f64>().unwrap_or(0.0);
                }
            } else if field.starts_with("\"status\"") {
                if let Some(colon_pos) = field.find(':') {
                    let val = field[colon_pos + 1..].trim().trim_matches('"');
                    status = val;
                }
            }
        }

        if status == "paid" {
            paid_amt += amt;
        } else if status == "open" {
            open_cnt += 1;
        }
    }

    println!("{}", paid_amt);
    println!("{}", open_cnt);
}
