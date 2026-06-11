fn main() {
    let list = [3, 1, 3, 2, 1, 4];
    let mut seen = Vec::new();
    let mut result = Vec::new();

    for num in list.iter() {
        let mut found = false;
        for s in seen.iter() {
            if *s == *num {
                found = true;
                break;
            }
        }
        if !found {
            result.push(*num);
            seen.push(*num);
        }
    }

    let mut output = String::new();
    for (i, num) in result.iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }
        output.push_str(&num.to_string());
    }

    println!("{}", output);
}
