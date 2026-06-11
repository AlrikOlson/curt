fn main() {
    let s = "aaabbcccca";
    let bytes = s.as_bytes();
    let mut result = String::new();
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i];
        let mut count = 1;
        while i + count < bytes.len() && bytes[i + count] == ch {
            count += 1;
        }
        result.push(ch as char);
        result.push_str(&count.to_string());
        i += count;
    }

    println!("{}", result);
}
