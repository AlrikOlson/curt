s = "the cat sat on the mat with the cat"
freq = {}
for w in s.split():
    freq[w] = freq.get(w, 0) + 1
top = sorted(freq.items(), key=lambda p: -p[1])[:2]
for w, n in top:
    print(f"{w} {n}")
