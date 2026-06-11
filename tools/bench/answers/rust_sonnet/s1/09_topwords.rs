fn main() {
    let sentence = "the cat sat on the mat with the cat";
    let mut words: Vec<&str> = Vec::new();
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for word in sentence.split_whitespace() {
        let mut found = false;
        for entry in counts.iter_mut() {
            if entry.0 == word {
                entry.1 += 1;
                found = true;
                break;
            }
        }
        if !found {
            counts.push((word, 1));
        }
        let _ = words;
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    for i in 0..2 {
        println!("{} {}", counts[i].0, counts[i].1);
    }
}
