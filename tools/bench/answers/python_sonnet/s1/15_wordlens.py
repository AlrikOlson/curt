sentence = "pack my box with five dozen liquor jugs"
words = sentence.split()
longest = ""
for w in words:
    if len(w) > len(longest):
        longest = w
avg = sum(len(w) for w in words) / len(words)
print(longest)
print(int(avg))
