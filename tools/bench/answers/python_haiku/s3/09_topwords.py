sentence = "the cat sat on the mat with the cat"
words = sentence.split()
freq = {}
for word in words:
    if word in freq:
        freq[word] += 1
    else:
        freq[word] = 1
sorted_words = sorted(freq.items(), key=lambda x: x[1], reverse=True)
for i in range(2):
    print(f"{sorted_words[i][0]} {sorted_words[i][1]}")
