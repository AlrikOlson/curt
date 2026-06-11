sentence = "pack my box with five dozen liquor jugs"
words = sentence.split()
longest = words[0]
for word in words:
    if len(word) > len(longest):
        longest = word
total_length = 0
for word in words:
    total_length += len(word)
avg_length = total_length / len(words)
print(longest)
print(int(avg_length))
