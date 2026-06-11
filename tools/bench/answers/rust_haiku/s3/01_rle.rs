fn main() {
    let s = "aaabbcccca";
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();

    while i < chars.len() {
        let ch = chars[i];
        let mut count = 1;
        while i + count < chars.len() && chars[i + count] == ch {
            count += 1;
        }
        result.push(ch);
        result.push_str(&count.to_string());
        i += count;
    }

    println!("{}", result);
}
