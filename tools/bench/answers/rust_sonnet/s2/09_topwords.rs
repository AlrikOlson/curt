fn main() {
    let sentence = "the cat sat on the mat with the cat";
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for word in sentence.split_whitespace() {
        if let Some(entry) = counts.iter_mut().find(|(w, _)| *w == word) {
            entry.1 += 1;
        } else {
            counts.push((word, 1));
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    for (word, count) in counts.iter().take(2) {
        println!("{} {}", word, count);
    }
}
