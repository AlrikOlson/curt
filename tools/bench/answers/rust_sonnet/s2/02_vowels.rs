fn main() {
    let s = "the quick brown fox jumps over the lazy dog";
    let count = s.chars().filter(|c| "aeiou".contains(*c)).count();
    println!("{}", count);
}
