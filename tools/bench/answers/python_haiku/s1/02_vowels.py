s = "the quick brown fox jumps over the lazy dog"
vowels = "aeiou"
count = 0
for char in s:
    if char in vowels:
        count += 1
print(count)
