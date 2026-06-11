s = "the cat sat on the mat with the cat"
words = s.split()
freq = {}
for word in words:
    freq[word] = freq.get(word, 0) + 1
sorted_words = sorted(freq.items(), key=lambda x: -x[1])
for i in range(2):
    print(f"{sorted_words[i][0]} {sorted_words[i][1]}")
