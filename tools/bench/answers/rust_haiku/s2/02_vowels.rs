fn main() {
    let s = "the quick brown fox jumps over the lazy dog";
    let vowels = "aeiou";
    let count = s.chars().filter(|c| vowels.contains(*c)).count();
    println!("{}", count);
}
