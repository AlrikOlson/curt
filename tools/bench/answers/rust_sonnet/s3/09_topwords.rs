fn main() {
    let sentence = "the cat sat on the mat with the cat";
    let mut words: Vec<&str> = sentence.split_whitespace().collect();
    words.sort();
    let mut counts: Vec<(&str, usize)> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        let mut c = 0;
        while i < words.len() && words[i] == w {
            c += 1;
            i += 1;
        }
        counts.push((w, c));
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    for (word, count) in counts.iter().take(2) {
        println!("{} {}", word, count);
    }
}
