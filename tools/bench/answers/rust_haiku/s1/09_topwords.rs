fn main() {
    let sentence = "the cat sat on the mat with the cat";
    let words: Vec<&str> = sentence.split_whitespace().collect();

    let mut word_counts: Vec<(&str, i32)> = Vec::new();

    for word in words.iter() {
        let mut found = false;
        for entry in word_counts.iter_mut() {
            if entry.0 == *word {
                entry.1 += 1;
                found = true;
                break;
            }
        }
        if !found {
            word_counts.push((word, 1));
        }
    }

    let mut i = 0;
    while i < word_counts.len() {
        let mut max_idx = i;
        let mut j = i + 1;
        while j < word_counts.len() {
            if word_counts[j].1 > word_counts[max_idx].1 {
                max_idx = j;
            }
            j += 1;
        }
        let temp = word_counts[i];
        word_counts[i] = word_counts[max_idx];
        word_counts[max_idx] = temp;
        i += 1;
    }

    println!("{} {}", word_counts[0].0, word_counts[0].1);
    println!("{} {}", word_counts[1].0, word_counts[1].1);
}
