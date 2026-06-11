fn main() {
    let sentence = "the cat sat on the mat with the cat";
    let words: Vec<&str> = sentence.split_whitespace().collect();

    let mut counts = Vec::new();

    for word in &words {
        let mut found = false;
        for (w, c) in &mut counts {
            if w == word {
                *c += 1;
                found = true;
                break;
            }
        }
        if !found {
            counts.push((word, 1));
        }
    }

    counts.sort_by_key(|&(_, c)| -(c as i32));

    for i in 0..2 {
        let (word, count) = counts[i];
        println!("{} {}", word, count);
    }
}
