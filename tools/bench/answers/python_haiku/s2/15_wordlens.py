s = "pack my box with five dozen liquor jugs"
words = s.split()
longest = words[0]
for word in words:
    if len(word) > len(longest):
        longest = word
total_len = sum(len(word) for word in words)
avg_len = total_len / len(words)
print(longest)
print(int(avg_len))
