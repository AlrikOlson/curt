fn main() {
    let s = "aaabbcccca";
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut count = 1;
        while i + count < chars.len() && chars[i + count] == c {
            count += 1;
        }
        result.push(c);
        result.push_str(&count.to_string());
        i += count;
    }
    println!("{}", result);
}
