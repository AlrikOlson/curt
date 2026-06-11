s = "the quick brown fox jumps over the lazy dog"
count = 0
for ch in s:
    if ch in "aeiou":
        count += 1
print(count)
