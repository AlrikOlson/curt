use std::collections::HashMap;
use std::{env, fs};

fn main() {
    let path = env::args().nth(1).unwrap();
    let text = fs::read_to_string(path).unwrap().to_lowercase();
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for w in text.split_whitespace() {
        *counts.entry(w).or_insert(0) += 1;
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    for (w, c) in pairs.iter().take(10) {
        println!("{w} {c}");
    }
}
