fn main() {
    let s = "the quick brown fox jumps over the lazy dog";
    let vowels = "aeiou";
    let mut count = 0;

    for ch in s.chars() {
        if vowels.contains(ch) {
            count += 1;
        }
    }

    println!("{}", count);
}
