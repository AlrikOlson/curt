sentence = "pack my box with five dozen liquor jugs"
words = sentence.split()
longest = words[0]
for word in words:
    if len(word) > len(longest):
        longest = word
total_len = 0
for word in words:
    total_len += len(word)
avg_len = int(total_len / len(words))
print(longest)
print(avg_len)
