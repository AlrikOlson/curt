use std::collections::HashMap;

fn main() {
    let sales = [("NY", 50), ("LA", 30), ("NY", 20)];
    let mut g: HashMap<&str, i64> = HashMap::new();
    for (city, amt) in sales {
        *g.entry(city).or_insert(0) += amt;
    }
    for (k, v) in &g {
        println!("{k} {v}");
    }
}
