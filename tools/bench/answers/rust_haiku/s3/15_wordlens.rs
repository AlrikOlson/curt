fn main() {
    let sentence = "pack my box with five dozen liquor jugs";
    let words: Vec<&str> = sentence.split_whitespace().collect();

    let mut longest = "";
    for &word in &words {
        if word.len() > longest.len() {
            longest = word;
        }
    }

    let mut total_len = 0;
    for &word in &words {
        total_len += word.len();
    }
    let avg_len = total_len / words.len();

    println!("{}", longest);
    println!("{}", avg_len);
}
