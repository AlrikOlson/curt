fn main() {
    let sentence = "the cat sat on the mat with the cat";
    let words: Vec<&str> = sentence.split_whitespace().collect();

    let mut word_counts: Vec<(String, i32)> = Vec::new();

    for word in &words {
        let mut found = false;
        for (w, count) in &mut word_counts {
            if w == word {
                *count += 1;
                found = true;
                break;
            }
        }
        if !found {
            word_counts.push((word.to_string(), 1));
        }
    }

    for i in 0..word_counts.len() {
        for j in i + 1..word_counts.len() {
            if word_counts[j].1 > word_counts[i].1 {
                word_counts.swap(i, j);
            }
        }
    }

    for i in 0..2 {
        println!("{} {}", word_counts[i].0, word_counts[i].1);
    }
}
