words = "a b b c b".split()
m = {}
for w in words:
    m[w] = m.get(w, 0) + 1
print(m.get("b", 0))
print(m.get("x", 0))
