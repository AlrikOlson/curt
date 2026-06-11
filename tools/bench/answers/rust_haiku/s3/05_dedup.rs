fn main() {
    let list = [3, 1, 3, 2, 1, 4];
    let mut seen = Vec::new();
    let mut result = Vec::new();

    for &num in &list {
        let mut found = false;
        for &s in &seen {
            if s == num {
                found = true;
                break;
            }
        }
        if !found {
            seen.push(num);
            result.push(num);
        }
    }

    let mut output = String::new();
    for (i, &num) in result.iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }
        output.push_str(&num.to_string());
    }

    println!("{}", output);
}
