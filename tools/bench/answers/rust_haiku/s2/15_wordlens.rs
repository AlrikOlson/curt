fn main() {
    let sentence = "pack my box with five dozen liquor jugs";
    let words: Vec<&str> = sentence.split_whitespace().collect();

    let longest = words.iter().max_by_key(|w| w.len()).unwrap();

    let total_len: usize = words.iter().map(|w| w.len()).sum();
    let avg = total_len as i32 / words.len() as i32;

    println!("{}", longest);
    println!("{}", avg);
}
