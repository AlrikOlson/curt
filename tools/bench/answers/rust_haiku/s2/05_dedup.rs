fn main() {
    let nums = [3, 1, 3, 2, 1, 4];
    let mut seen = Vec::new();
    let mut result = Vec::new();

    for &n in &nums {
        if !seen.contains(&n) {
            seen.push(n);
            result.push(n);
        }
    }

    let output = result
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", output);
}
