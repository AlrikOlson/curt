fn main() {
    let s = "aaabbcccca";
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut count = 0;
        while i < chars.len() && chars[i] == c {
            count += 1;
            i += 1;
        }
        result.push(c);
        result.push_str(&count.to_string());
    }
    println!("{}", result);
}
