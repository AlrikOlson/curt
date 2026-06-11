fn main() {
    let s = "aaabbcccca";
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        let mut count = 1;
        while let Some(&next_ch) = chars.peek() {
            if next_ch == ch {
                count += 1;
                chars.next();
            } else {
                break;
            }
        }
        result.push(ch);
        result.push_str(&count.to_string());
    }

    println!("{}", result);
}
