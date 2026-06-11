s = "the quick brown fox jumps over the lazy dog"
count = sum(1 for c in s if c in "aeiou")
print(count)
