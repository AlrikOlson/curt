fn main() {
    let list = [3, 1, 3, 2, 1, 4];
    let mut seen = Vec::new();
    let mut result = Vec::new();
    for &x in &list {
        if !seen.contains(&x) {
            seen.push(x);
            result.push(x.to_string());
        }
    }
    println!("{}", result.join(" "));
}
