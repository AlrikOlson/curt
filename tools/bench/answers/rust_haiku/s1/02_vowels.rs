fn main() {
    let s = "the quick brown fox jumps over the lazy dog";
    let mut count = 0;

    for ch in s.chars() {
        match ch {
            'a' | 'e' | 'i' | 'o' | 'u' => count += 1,
            _ => {}
        }
    }

    println!("{}", count);
}
