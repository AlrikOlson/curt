sentence = "the cat sat on the mat with the cat"
words = sentence.split()
counts = {}
for word in words:
    if word in counts:
        counts[word] += 1
    else:
        counts[word] = 1
sorted_words = sorted(counts.items(), key=lambda x: x[1], reverse=True)
for i in range(2):
    word, count = sorted_words[i]
    print(f"{word} {count}")
