words = "the cat sat on the mat with the cat".split()
counts = {}
for w in words:
    counts[w] = counts.get(w, 0) + 1
sorted_words = sorted(counts.items(), key=lambda x: -x[1])
for word, count in sorted_words[:2]:
    print(word, count)
